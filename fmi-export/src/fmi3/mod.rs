//! ## Design
//!
//! The [`crate::export_fmu`] macro generates the necessary C-API bindings for the exported FMU. Many of
//! these bindings operate on a [`binding::fmi3Instance`], which is an opaque pointer to an instance of [`ModelInstance`].
//!
//! `ModelInstance` implements the [`Common`] trait, which provides the actual implementation of the FMI 3.0 API.
//! All user-model-specific functions are delegated to the [`ModelData`] trait, which the user model must implement.

use std::{collections::BTreeMap, ffi::CString, path::PathBuf};

use fmi::fmi3::{binding, Common, Fmi3Error, Fmi3Res, Fmi3Status, GetSet};

mod macros;

enum ModelState {
    StartAndEnd,
    ConfigurationMode,
    Instantiated,
    InitializationMode,
    EventMode,
    ContinuousTimeMode,
    StepMode,
    ClockActivationMode,
    StepDiscarded,
    ReconfigurationMode,
    IntermediateUpdateMode,
    Terminated,
}

/// Trait for FMU 3.0
pub trait ModelData: Default + GetSet {
    type ValueRef: Copy
        + From<binding::fmi3ValueReference>
        + Into<binding::fmi3ValueReference>
        + PartialEq
        + 'static;

    /// Set start values
    fn set_start_values(&mut self);

    /// Calculate values
    fn calculate_values(&mut self) -> Fmi3Status;

    fn configurate(&mut self) -> Fmi3Status {
        todo!();

        /*
        #ifdef HAS_EVENT_INDICATORS
            comp->nz = getNumberOfEventIndicators(comp);

            if (comp->nz > 0) {
                CALL(s_reallocate(comp, (void**)& comp->prez, comp->nz * sizeof(double)));
                CALL(s_reallocate(comp, (void**)&comp->z, comp->nz * sizeof(double)));
            }

            CALL(getEventIndicators(comp, comp->prez, comp->nz));
        #endif

        #ifdef HAS_CONTINUOUS_STATES
            comp->nx = getNumberOfContinuousStates(comp);

            if (comp->nx > 0) {
                CALL(s_reallocate(comp, (void**)&comp->x, comp->nx * sizeof(double)));
                CALL(s_reallocate(comp, (void**)&comp->dx, comp->nx * sizeof(double)));
            }
        #endif
        */
        Fmi3Res::OK.into()
    }
}

/// An exportable FMU instance
pub struct ModelInstance<F: ModelData> {
    start_time: f64,
    stop_time: f64,
    time: f64,
    instance_name: String,
    resource_path: PathBuf,
    /// Map of logging categories to their enabled state.
    /// This is used to track which categories are enabled for logging.
    logging_on: BTreeMap<log::Level, bool>,
    /// Callback for logging messages.
    log_message: binding::fmi3LogMessageCallback,
    state: ModelState,

    // internal solver steps
    n_steps: usize,

    is_dirty_values: bool,
    model_data: F,
    _marker: std::marker::PhantomData<F>,
}

impl<F: ModelData> ModelInstance<F> {
    pub fn new(
        name: String,
        resource_path: PathBuf,
        logging_on: bool,
        log_message: binding::fmi3LogMessageCallback,
    ) -> Self {
        let logging_on = log::Level::iter()
            .map(|level| (level, logging_on))
            .collect();
        Self {
            start_time: 0.0,
            stop_time: 1.0,
            time: 0.0,
            instance_name: name,
            resource_path,
            logging_on,
            log_message,
            state: ModelState::Instantiated,
            n_steps: 0,
            is_dirty_values: false,
            model_data: F::default(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn log(&self, level: log::Level, message: &str) {
        if let Some(enabled) = self.logging_on.get(&level) {
            if *enabled {
                let status: Fmi3Status = Fmi3Res::OK.into();
                let category = CString::new(level.to_string()).expect("Invalid category name");
                let message = CString::new(message).expect("Invalid message");

                unsafe {
                    (self.log_message.unwrap())(
                        std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                        status.into(),
                        category.as_ptr() as binding::fmi3String,
                        message.as_ptr() as binding::fmi3String,
                    )
                };
            }
        }
    }
}

impl<F> GetSet for ModelInstance<F>
where
    F: ModelData,
{
    type ValueRef = binding::fmi3ValueReference;

    fn get_float32(&mut self, _vrs: &[Self::ValueRef], _values: &mut [f32]) -> Fmi3Status {
        self.model_data.get_float32(_vrs, _values)
    }

    fn get_float64(&mut self, _vrs: &[Self::ValueRef], _values: &mut [f64]) -> Fmi3Status {
        self.model_data.get_float64(_vrs, _values)
    }
}

impl<F> Common for ModelInstance<F>
where
    F: ModelData,
{
    fn get_version(&self) -> &str {
        // Safety: binding::fmi3Version is a byte array representing the version string
        unsafe { str::from_utf8_unchecked(binding::fmi3Version) }
    }

    fn set_debug_logging(&mut self, logging_on: bool, categories: &[&str]) -> Fmi3Status {
        for &cat in categories.iter() {
            if let Some(cat) = cat
                .parse::<log::Level>()
                .ok()
                .and_then(|level| self.logging_on.get_mut(&level))
            {
                *cat = logging_on;
            } else {
                log::warn!("Unknown logging category: {cat}");
                return Fmi3Error::Error.into();
            }
        }
        Fmi3Res::OK.into()
    }

    fn enter_initialization_mode(
        &mut self,
        _tolerance: Option<f64>,
        _start_time: f64,
        _stop_time: Option<f64>,
    ) -> Fmi3Status {
        match self.state {
            ModelState::Instantiated => {
                // Transition to INITIALIZATION_MODE
                self.state = ModelState::InitializationMode;
                //self.log("info", "Entering initialization mode");
                Fmi3Res::OK.into()
            }
            _ => {
                //this.log( "error", "Cannot enter initialization mode from current state",);
                Fmi3Error::Error.into()
            }
        }
    }

    fn exit_initialization_mode(&mut self) -> Fmi3Status {
        // if values were set and no fmi3GetXXX triggered update before,
        // ensure calculated values are updated now
        if self.is_dirty_values {
            self.model_data.calculate_values();
            self.is_dirty_values = false;
        }

        /*
        switch (S->type) {
            case ModelExchange:
                S->state = EventMode;
                break;
            case CoSimulation:
                S->state = S->eventModeUsed ? EventMode : StepMode;
                break;
            case ScheduledExecution:
                S->state = ClockActivationMode;
                break;
        }
        */

        self.model_data.configurate();
        Fmi3Res::OK.into()
    }

    fn terminate(&mut self) -> Fmi3Status {
        self.state = ModelState::Terminated;
        Fmi3Res::OK.into()
    }

    fn reset(&mut self) -> Fmi3Status {
        self.state = ModelState::Instantiated;
        self.start_time = 0.0;
        self.time = 0.0;
        self.n_steps = 0;
        self.model_data.set_start_values();
        Fmi3Res::OK.into()
    }

    fn enter_configuration_mode(&mut self) -> Fmi3Status {
        todo!()
    }

    fn exit_configuration_mode(&mut self) -> Fmi3Status {
        todo!()
    }

    fn enter_event_mode(&mut self) -> Fmi3Status {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(false)]
    #[test]
    fn test_export1() {
        #[derive(Default)]
        struct BouncingBall {
            h: f64,
            v: f64,
            g: f64,
            e: f64,
        }

        impl ModelData for BouncingBall {
            fn set_start_values(&mut self) {
                self.h = 1.0;
                self.v = 0.0;
                self.g = -9.81;
                self.e = 0.7;
            }
        }

        //export_fmu!(BouncingBall);
    }

    #[test]
    fn test_van_der_pol() {
        use crate::store::{vr_pack, Desc, ScalarType, Store, Tables};

        // VR enum with type safety and vr_pack encoding
        #[repr(u32)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum ValRef {
            Time = 0,  // offset 0
            X0 = 1,    // offset 1
            DerX0 = 2, // offset 2
            X1 = 3,    // offset 3
            DerX1 = 4, // offset 4
            Mu = 5,    // offset 5
        }

        impl From<binding::fmi3ValueReference> for ValRef {
            fn from(value: binding::fmi3ValueReference) -> Self {
                match value {
                    0 => ValRef::Time,
                    1 => ValRef::X0,
                    2 => ValRef::DerX0,
                    3 => ValRef::X1,
                    4 => ValRef::DerX1,
                    5 => ValRef::Mu,
                    _ => panic!("Invalid value reference: {}", value),
                }
            }
        }

        impl From<ValRef> for binding::fmi3ValueReference {
            fn from(value: ValRef) -> Self {
                value as u32
            }
        }

        #[derive(Debug, Default)]
        struct VanDerPol {
            x0: f64,
            der_x0: f64,
            x1: f64,
            der_x1: f64,
            mu: f64,
        }

        impl GetSet for VanDerPol {
            type ValueRef = binding::fmi3ValueReference;

            fn get_float64(&mut self, vrs: &[Self::ValueRef], values: &mut [f64]) -> Fmi3Status {
                for (vr, value) in vrs.iter().zip(values.iter_mut()) {
                    match ValRef::from(*vr) {
                        ValRef::Time => *value = self.x0, // Time is x0 for this model
                        ValRef::X0 => *value = self.x0,
                        ValRef::DerX0 => *value = self.der_x0,
                        ValRef::X1 => *value = self.x1,
                        ValRef::DerX1 => *value = self.der_x1,
                        ValRef::Mu => *value = self.mu,
                    }
                }
                Fmi3Res::OK.into()
            }

            fn set_float64(&mut self, vrs: &[Self::ValueRef], values: &[f64]) -> Fmi3Status {
                for (vr, value) in vrs.iter().zip(values.iter()) {
                    match ValRef::from(*vr) {
                        ValRef::Time => self.x0 = *value,
                        ValRef::X0 => self.x0 = *value,
                        ValRef::DerX0 => self.der_x0 = *value,
                        ValRef::X1 => self.x1 = *value,
                        ValRef::DerX1 => self.der_x1 = *value,
                        ValRef::Mu => self.mu = *value,
                    }
                }
                Fmi3Res::OK.into()
            }
        }

        impl ModelData for VanDerPol {
            type ValueRef = ValRef;

            fn set_start_values(&mut self) {
                // Set Van Der Pol initial conditions
                self.x0 = 2.0;
                self.x1 = 0.0;
                self.mu = 1.0;
            }

            fn calculate_values(&mut self) -> Fmi3Status {
                // Get current values from state
                let x0 = self.x0;
                let x1 = self.x1;
                let mu = self.mu;

                // Calculate derivatives using Van Der Pol equation
                self.der_x0 = x1;
                self.der_x1 = mu * ((1.0 - x0 * x0) * x1) - x0;

                Fmi3Res::OK.into()
            }
        }

        // Test the new implementation with ModelInstance
        let mut instance: ModelInstance<VanDerPol> = ModelInstance::new(
            "test".to_string(),
            std::path::PathBuf::from("/tmp"),
            false,
            None, // No logging callback
        );
        instance.enter_initialization_mode(None, 0.0, None);
        let mut vals = [0.0f64; 3];
        instance.get_float64(
            &[ValRef::X0 as u32, ValRef::X1 as u32, ValRef::Mu as u32],
            &mut vals,
        );
        assert_eq!(vals[0], 2.0);
        assert_eq!(vals[1], 0.0);
        assert_eq!(vals[2], 1.0);

        // Test calculate_values
        instance.exit_initialization_mode();
        let mut ders = [0.0f64; 2];
        instance.get_float64(&[ValRef::DerX0 as u32, ValRef::DerX1 as u32], &mut ders);
        assert_eq!(ders[0], 0.0); // der_x0 = x1 = 0.0
        assert_eq!(ders[1], -2.0); // mu * ((1 - x0*x0) * x1) - x0 = 1 * ((1 - 4) * 0) - 2 = -2

        crate::export_fmu!(VanDerPol);
    }
}
