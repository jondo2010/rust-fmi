//! ## Design
//!
//! The [`crate::export_fmu`] macro generates the necessary C-API bindings for the exported FMU.
//! Many of these bindings operate on a [`binding::fmi3Instance`], which is an opaque pointer to an
//! instance of [`ModelInstance`].
//!
//! [`ModelInstance`] implements the [`Common`] trait, which provides the actual implementation of
//! the FMI 3.0 API. All user-model-specific functions are delegated to the [`Model`] trait,
//! which the user model must implement.

use std::{collections::BTreeMap, ffi::CString, path::PathBuf};

use fmi::fmi3::{Common, Fmi3Error, Fmi3Res, Fmi3Status, GetSet, binding, schema};

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

/// Model trait, to be implemented by the user model
pub trait Model: Default + GetSet {
    const MODEL_NAME: &'static str;
    const MODEL_DESCRIPTION: &'static str;

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

    /// Describe the model variables for this model
    fn model_variables() -> schema::ModelVariables {
        // should be implemented by the user model
        schema::ModelVariables {
            ..Default::default()
        }
    }

    /// Describe the model structure for this model
    fn model_structure() -> schema::ModelStructure {
        // should be implemented by the user model
        schema::ModelStructure {
            ..Default::default()
        }
    }

    /// Build a model description for this model
    fn model_description() -> schema::Fmi3ModelDescription {
        schema::Fmi3ModelDescription {
            fmi_version: unsafe { str::from_utf8_unchecked(binding::fmi3Version).to_owned() },
            model_name: Self::MODEL_NAME.to_owned(),
            instantiation_token: String::new(),
            description: Some(Self::MODEL_DESCRIPTION.to_owned()),
            generation_tool: Some("rust-fmi".to_owned()),
            generation_date_and_time: Some(chrono::Utc::now().to_rfc3339()),
            model_variables: Self::model_variables(),
            model_structure: Self::model_structure(),
            ..Default::default()
        }
    }
}

/// An exportable FMU instance
pub struct ModelInstance<M: Model> {
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
    model: M,
    _marker: std::marker::PhantomData<M>,
}

impl<F: Model> ModelInstance<F> {
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
            model: F::default(),
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
    F: Model<ValueRef = binding::fmi3ValueReference>,
{
    type ValueRef = binding::fmi3ValueReference;

    fn get_float32(
        &mut self,
        vrs: &[Self::ValueRef],
        values: &mut [f32],
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.model.get_float32(vrs, values)
    }

    fn get_float64(
        &mut self,
        vrs: &[Self::ValueRef],
        values: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.model.get_float64(vrs, values)
    }
}

impl<F> Common for ModelInstance<F>
where
    F: Model<ValueRef = binding::fmi3ValueReference>,
{
    fn get_version(&self) -> &str {
        // Safety: binding::fmi3Version is a null-terminated byte array representing the version string
        unsafe { str::from_utf8_unchecked(binding::fmi3Version) }
    }

    fn set_debug_logging(
        &mut self,
        logging_on: bool,
        categories: &[&str],
    ) -> Result<Fmi3Res, Fmi3Error> {
        for &cat in categories.iter() {
            if let Some(cat) = cat
                .parse::<log::Level>()
                .ok()
                .and_then(|level| self.logging_on.get_mut(&level))
            {
                *cat = logging_on;
            } else {
                log::warn!("Unknown logging category: {cat}");
                return Err(Fmi3Error::Error);
            }
        }
        Ok(Fmi3Res::OK)
    }

    fn enter_initialization_mode(
        &mut self,
        _tolerance: Option<f64>,
        _start_time: f64,
        _stop_time: Option<f64>,
    ) -> Result<Fmi3Res, Fmi3Error> {
        match self.state {
            ModelState::Instantiated => {
                // Transition to INITIALIZATION_MODE
                self.state = ModelState::InitializationMode;
                //self.log("info", "Entering initialization mode");
                Ok(Fmi3Res::OK)
            }
            _ => {
                //this.log( "error", "Cannot enter initialization mode from current state",);
                Err(Fmi3Error::Error)
            }
        }
    }

    fn exit_initialization_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        // if values were set and no fmi3GetXXX triggered update before,
        // ensure calculated values are updated now
        if self.is_dirty_values {
            self.model.calculate_values();
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

        self.model.configurate();
        Ok(Fmi3Res::OK)
    }

    fn terminate(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.state = ModelState::Terminated;
        Ok(Fmi3Res::OK)
    }

    fn reset(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.state = ModelState::Instantiated;
        self.start_time = 0.0;
        self.time = 0.0;
        self.n_steps = 0;
        self.model.set_start_values();
        Ok(Fmi3Res::OK)
    }

    fn enter_configuration_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        todo!()
    }

    fn exit_configuration_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        todo!()
    }

    fn enter_event_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        todo!();
    }
}
