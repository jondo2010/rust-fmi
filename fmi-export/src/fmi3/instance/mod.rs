use fmi::fmi3::{Fmi3Error, Fmi3Status, binding};

use crate::fmi3::{
    ModelState, UserModel,
    traits::{Context, Model},
};

mod common;
pub mod context;
mod get_set;
mod impl_cs;
mod impl_me;
mod impl_se;

pub type LogMessageClosure = Box<dyn Fn(Fmi3Status, &str, std::fmt::Arguments<'_>) + Send + Sync>;
pub type IntermediateUpdateClosure =
    Box<dyn Fn(f64, bool, bool, bool, bool) -> Option<f64> + Send + Sync>;

/// An exportable FMU instance, generic over model type M and context type C
#[repr(C)]
pub struct ModelInstance<M, C>
where
    M: UserModel,
    C: Context<M>,
{
    /// The instance type (public for FFI access)
    pub(crate) instance_type: fmi::InterfaceType,
    /// The name of this instance
    instance_name: String,
    /// Context for the model instance
    context: C,
    /// Current state of the model instance
    state: ModelState,
    /// Do we need to re-evaluate the model equations?
    is_dirty_values: bool,
    /// Scheduled-Execution only: the set of countdown Clock value references whose current
    /// interval has *already been observed* by `fmi3GetInterval*` since that Clock's last
    /// tick.
    ///
    /// The FMI 3.0.1 spec tracks the interval qualifier **per Clock**: once a Clock's
    /// interval is reported with `fmi3IntervalChanged`, subsequent reads with no intervening
    /// `activate_model_partition` *for that Clock* must return `fmi3IntervalUnchanged`. So a
    /// Clock is removed from this set when it ticks (its next read is `Changed` again) and
    /// inserted when its interval is read. A Clock not in the set that reports an interval is
    /// `Changed`; a Clock whose model returns no interval is `fmi3IntervalNotYetKnown` (and
    /// is not inserted). A single instance-wide boolean would be wrong here: activating
    /// Clock A must not make Clock B look `Changed`, and reading A must not clear B.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3GetIntervalDecimal>
    intervals_observed: std::collections::HashSet<binding::fmi3ValueReference>,
    /// The user-defined model
    model: M,
}

impl<M, C> ModelInstance<M, C>
where
    M: Model + UserModel,
    C: Context<M>,
{
    pub fn new(
        name: String,
        instantiation_token: &str,
        context: C,
        instance_type: fmi::InterfaceType,
    ) -> Result<Self, Fmi3Error> {
        // Validate the instantiation token using the compile-time constant
        if instantiation_token != M::INSTANTIATION_TOKEN {
            eprintln!(
                "Instantiation token mismatch. Expected: '{}', got: '{}'",
                M::INSTANTIATION_TOKEN,
                instantiation_token
            );
            return Err(Fmi3Error::Error);
        }

        let mut instance = Self {
            instance_name: name,
            context,
            state: ModelState::Instantiated,
            instance_type,
            is_dirty_values: true,
            intervals_observed: std::collections::HashSet::new(),
            model: M::default(),
        };

        // Set start values for the model
        instance.model.set_start_values();

        Ok(instance)
    }

    pub fn instance_name(&self) -> &str {
        &self.instance_name
    }

    pub fn instance_type(&self) -> fmi::InterfaceType {
        self.instance_type
    }

    pub fn context(&self) -> &C {
        &self.context
    }

    #[inline]
    pub fn assert_instance_type(&self, expected: fmi::InterfaceType) -> Result<(), Fmi3Error> {
        if self.instance_type != expected {
            self.context.log(
                Fmi3Error::Error.into(),
                M::LoggingCategory::default(),
                format_args!(
                    "Instance type mismatch. Expected: {:?}, got: {:?}",
                    expected, self.instance_type
                ),
            );
            return Err(Fmi3Error::Error);
        }
        Ok(())
    }

    /// Validate that a variable can be set in the current model state
    fn validate_variable_setting(&self, vr: binding::fmi3ValueReference) -> Result<(), Fmi3Error> {
        match M::validate_variable_setting(vr, &self.state) {
            Ok(()) => Ok(()),
            Err(message) => {
                self.context.log(
                    Fmi3Error::Error.into(),
                    M::LoggingCategory::default(),
                    format_args!("Variable setting error for VR {vr}: {message}"),
                );
                Err(Fmi3Error::Error)
            }
        }
    }
}
