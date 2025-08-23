//! Traits for generic FMI handling that apply to both FMI2 and FMI3.

use fmi_schema::{
    MajorVersion,
    traits::{DefaultExperiment, FmiModelDescription},
};

use crate::Error;

/// Generic FMI import trait
pub trait FmiImport: Sized {
    /// The type of the major version
    const MAJOR_VERSION: MajorVersion;

    /// The raw parsed XML schema type
    type ModelDescription: FmiModelDescription + DefaultExperiment;

    /// The raw FMI bindings type
    type Binding;

    /// The type of the value reference used by the FMI API.
    type ValueRef;

    /// Create a new FMI import from a directory containing the unzipped FMU
    fn new(dir: tempfile::TempDir, schema_xml: &str) -> Result<Self, Error>;

    /// Return the path to the extracted FMU
    fn archive_path(&self) -> &std::path::Path;

    /// Get the path to the shared library
    fn shared_lib_path(&self, model_identifier: &str) -> Result<std::path::PathBuf, Error>;

    /// Return the path to the resources directory
    fn resource_path(&self) -> std::path::PathBuf {
        self.archive_path().join("resources")
    }

    /// Return a canonical string representation of the resource path
    fn canonical_resource_path_string(&self) -> String;

    /// Get a reference to the raw-schema model description
    fn model_description(&self) -> &Self::ModelDescription;

    /// Load the plugin shared library and return the raw bindings.
    fn binding(&self, model_identifier: &str) -> Result<Self::Binding, Error>;
}

/// FMI status trait
pub trait FmiStatus {
    type Res;
    type Err: Into<Error> + std::fmt::Debug;

    /// Convert to [`Result<Self::Res, Self::Err>`]
    fn ok(self) -> Result<Self::Res, Self::Err>;
    /// Check if the status is an error
    fn is_error(&self) -> bool;
}

/// Result type alias for FMI instances
///
/// This is essentially a `Result<FmiStatus, FmiError>` and maps the FMI result codes onto
/// the Rust error handling model.
pub type InstanceResult<S, R = <<S as FmiInstance>::Status as FmiStatus>::Res> =
    Result<R, <<S as FmiInstance>::Status as FmiStatus>::Err>;

/// Generic FMI instance trait
pub trait FmiInstance {
    type ModelDescription: FmiModelDescription + DefaultExperiment;
    type ValueRef: Copy + From<u32> + Into<u32>;
    type Status: FmiStatus;

    /// Get the instance name
    fn name(&self) -> &str;

    /// Get the version of the FMU
    fn get_version(&self) -> &str;

    /// Get the model description of the FMU
    fn model_description(&self) -> &Self::ModelDescription;

    /// The function controls the debug logging that is output by the FMU
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3SetDebugLogging>
    fn set_debug_logging(&mut self, logging_on: bool, categories: &[&str]) -> InstanceResult<Self>;

    fn enter_initialization_mode(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> InstanceResult<Self>;

    fn exit_initialization_mode(&mut self) -> InstanceResult<Self>;

    /// Changes state to [`Terminated`](https://fmi-standard.org/docs/3.0.1/#Terminated).
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3Terminate>
    fn terminate(&mut self) -> InstanceResult<Self>;

    /// Is called by the environment to reset the FMU after a simulation run.
    /// The FMU goes into the same state as if newly created. All variables have their default
    /// values. Before starting a new run [`FmiInstance::enter_initialization_mode()`] has to be called.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3Reset>
    fn reset(&mut self) -> InstanceResult<Self>;

    /// Get the number of values required to store the continuous states. Array dimensions are expanded.
    fn get_number_of_continuous_state_values(&mut self) -> usize;

    /// Get the number of values required to store the event indicators. Array dimensions are expanded.
    fn get_number_of_event_indicator_values(&mut self) -> usize;
}

/// Generic FMI ModelExchange trait
pub trait FmiModelExchange: FmiInstance {
    fn enter_continuous_time_mode(&mut self) -> InstanceResult<Self>;

    fn enter_event_mode(&mut self) -> InstanceResult<Self>;

    /// This function is called to signal a converged solution at the current super-dense time
    /// instant. `update_discrete_states` must be called at least once per super-dense time
    /// instant.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3UpdateDiscreteStates>
    fn update_discrete_states(
        &mut self,
        discrete_states_need_update: &mut bool,
        terminate_simulation: &mut bool,
        nominals_of_continuous_states_changed: &mut bool,
        values_of_continuous_states_changed: &mut bool,
        next_event_time: &mut Option<f64>,
    ) -> InstanceResult<Self>;

    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
        enter_event_mode: &mut bool,
        terminate_simulation: &mut bool,
    ) -> InstanceResult<Self>;

    fn set_time(&mut self, time: f64) -> InstanceResult<Self>;

    fn get_continuous_states(&mut self, continuous_states: &mut [f64]) -> InstanceResult<Self>;
    fn set_continuous_states(&mut self, states: &[f64]) -> InstanceResult<Self>;

    fn get_continuous_state_derivatives(&mut self, derivatives: &mut [f64])
    -> InstanceResult<Self>;
    fn get_nominals_of_continuous_states(&mut self, nominals: &mut [f64]) -> InstanceResult<Self>;

    fn get_event_indicators(&mut self, event_indicators: &mut [f64]) -> InstanceResult<Self, bool>;
    fn get_number_of_event_indicators(&mut self) -> InstanceResult<Self, usize>;
}

/// Event handling interface for ME in FMI2.0 and both ME and CS interfaces in FMI3.0
pub trait FmiEventHandler: FmiInstance {
    fn enter_event_mode(&mut self) -> InstanceResult<Self>;

    fn update_discrete_states(
        &mut self,
        discrete_states_need_update: &mut bool,
        terminate_simulation: &mut bool,
        nominals_of_continuous_states_changed: &mut bool,
        values_of_continuous_states_changed: &mut bool,
        next_event_time: &mut Option<f64>,
    ) -> InstanceResult<Self>;
}
