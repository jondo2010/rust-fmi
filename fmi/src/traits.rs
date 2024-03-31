use fmi_schema::{
    traits::{DefaultExperiment, FmiModelDescription},
    MajorVersion,
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
    fn resource_url(&self) -> url::Url {
        url::Url::from_file_path(self.archive_path().join("resources"))
            .expect("Error forming resource location URL")
    }

    /// Get a reference to the raw-schema model description
    fn model_description(&self) -> &Self::ModelDescription;

    /// Load the plugin shared library and return the raw bindings.
    fn binding(&self, model_identifier: &str) -> Result<Self::Binding, Error>;
}

/// FMI status trait
pub trait FmiStatus {
    type Res;
    type Err: Into<Error>;
    /// Convert to [`Result<Self::Res, Self::Err>`]
    fn ok(self) -> Result<Self::Res, Self::Err>;
    /// Check if the status is an error
    fn is_error(&self) -> bool;
}

/// Generic FMI instance trait
pub trait FmiInstance {
    type ModelDescription: FmiModelDescription + DefaultExperiment;
    type Import: FmiImport<ModelDescription = Self::ModelDescription, ValueRef = Self::ValueRef>;
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
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3SetDebugLogging]
    fn set_debug_logging(&mut self, logging_on: bool, categories: &[&str]) -> Self::Status;

    /// Changes state to [`Terminated`](https://fmi-standard.org/docs/3.0.1/#Terminated).
    ///
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3Terminate]
    fn terminate(&mut self) -> Self::Status;

    /// Is called by the environment to reset the FMU after a simulation run.
    /// The FMU goes into the same state as if newly created. All variables have their default
    /// values. Before starting a new run [`Common::enter_initialization_mode()`] has to be called.
    ///
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3Reset]
    fn reset(&mut self) -> Self::Status;

    /// Get the number of values required to store the continuous states. Array dimensions are expanded.
    fn get_number_of_continuous_state_values(&mut self) -> usize;

    /// Get the number of values required to store the event indicators. Array dimensions are expanded.
    fn get_number_of_event_indicator_values(&mut self) -> usize;
}

/// Generic FMI ModelExchange trait
pub trait FmiModelExchange: FmiInstance {
    fn enter_continuous_time_mode(&mut self) -> Self::Status;

    fn enter_event_mode(&mut self) -> Self::Status;

    /// This function is called to signal a converged solution at the current super-dense time
    /// instant. `update_discrete_states` must be called at least once per super-dense time
    /// instant.
    ///
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3UpdateDiscreteStates]
    fn update_discrete_states(
        &mut self,
        discrete_states_need_update: &mut bool,
        terminate_simulation: &mut bool,
        nominals_of_continuous_states_changed: &mut bool,
        values_of_continuous_states_changed: &mut bool,
        next_event_time: &mut Option<f64>,
    ) -> Self::Status;

    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
        enter_event_mode: &mut bool,
        terminate_simulation: &mut bool,
    ) -> Self::Status;

    fn set_time(&mut self, time: f64) -> Self::Status;

    fn get_continuous_states(&mut self, continuous_states: &mut [f64]) -> Self::Status;
    fn set_continuous_states(&mut self, states: &[f64]) -> Self::Status;

    fn get_continuous_state_derivatives(&mut self, derivatives: &mut [f64]) -> Self::Status;
    fn get_nominals_of_continuous_states(&mut self, nominals: &mut [f64]) -> Self::Status;

    fn get_event_indicators(&mut self, event_indicators: &mut [f64]) -> Self::Status;
    fn get_number_of_event_indicators(
        &self,
        number_of_event_indicators: &mut usize,
    ) -> Self::Status;
}
