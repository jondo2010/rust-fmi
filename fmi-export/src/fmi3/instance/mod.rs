use std::path::PathBuf;

use fmi::fmi3::{Fmi3Error, Fmi3Status, binding};

use crate::fmi3::{
    ModelLoggingCategory, ModelState, UserModel,
    traits::{Context, Model},
};

mod common;
pub mod context;
mod get_set;
mod impl_cs_me_wrapper;
mod impl_me;
mod impl_se;

pub type LogMessageClosure = Box<dyn Fn(Fmi3Status, &str, std::fmt::Arguments<'_>) + Send + Sync>;

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
