//! Traits for the different instance types.

use crate::Error;

/// Interface common to all instance types
pub trait Common {
    /// The instance name
    fn name(&self) -> &str;

    /// The FMI-standard version string
    fn get_version(&self) -> &str;

    /// The function controls the debug logging that is output by the FMU
    #[cfg(feature = "disabled")]
    fn set_debug_logging(
        &mut self,
        logging_on: bool,
        //categories: &[LogCategoryKey],
        categories: impl Iterator<Item = fmi3::model::LogCategoryKey>,
    ) -> FmiResult<()>;

    /// Changes state to `Initialization Mode`.
    ///
    /// tolerance depend on the interface type:
    /// * Model Exchange: If `tolerance = Some(True)`, then the model is called with a numerical
    ///   integration scheme where the step size is controlled by using tolerance for error
    ///   estimation (usually as relative tolerance). In such a case all numerical algorithms
    ///   used inside the model (for example, to solve nonlinear algebraic equations) should
    ///   also operate with an error estimation of an appropriate smaller relative tolerance.
    /// * Co-Simulation: If `tolerance = Some(True)`, then the communication step size of the
    ///   FMU is controlled by error estimation. In case the FMU utilizes a numerical integrator
    ///   with variable step size and error estimation, it is suggested to use tolerance for the
    ///   error estimation of the integrator (usually as relative tolerance).
    /// An FMU for Co-Simulation might ignore this argument.
    fn enter_initialization_mode(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Result<(), Error>;

    /// Changes the state, depending on the instance type:
    /// * Model Exchange: Event Mode
    /// * Co-Simulation:
    ///     event_mode_used = true: Event Mode
    ///     event_mode_used = false: Step Mode
    /// * Scheduled Execution: Clock Activation Mode.
    fn exit_initialization_mode(&mut self) -> Result<(), Error>;

    /// This function changes the state to Event Mode.
    ///
    /// The importer must call fmi3EnterEventMode when any of the following conditions are met:
    /// * time has reached nextEventTime as returned by fmi3UpdateDiscreteStates, or
    /// * the signs of the event indicators signal an event according to Section 3.1.1, or
    /// * the FMU returned with enterEventMode = fmi3True from fmi3CompletedIntegratorStep, or
    /// * the importer plans discrete changes to inputs, or an input Clock needs to be set.
    ///
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3EnterEventMode]
    fn enter_event_mode(&mut self) -> Result<(), Error>;

    /// Changes state to [`Terminated`](https://fmi-standard.org/docs/3.0.1/#Terminated).
    ///
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3Terminate]
    fn terminate(&mut self) -> Result<(), Error>;

    /// Is called by the environment to reset the FMU after a simulation run.
    /// The FMU goes into the same state as if newly created. All variables have their default
    /// values. Before starting a new run [`enter_initialization_mode()`] has to be called.
    ///
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3Reset]
    fn reset(&mut self) -> Result<(), Error>;
}

/// Interface for Model Exchange instances
pub trait ModelExchange: Common {
    /// This function must be called to change from Event Mode into Continuous-Time Mode in Model Exchange.
    fn enter_continuous_time_mode(&mut self) -> Result<(), Error>;

    /// This function is called after every completed step of the integrator provided the capability flag
    /// [`schema::interface_type::Fmi3ModelExchange::needs_completed_integrator_step`] = true.
    ///
    /// The importer must have set valid values for time, continuous inputs and continuous
    /// states prior to calling this function to evaluate 𝐟comp with valid right-hand side data.
    ///
    /// Arguments:
    ///
    /// * `no_set_fmu_state_prior`: if `set_fmu_state()` will no longer be called for time
    ///                             instants prior to current time in this simulation run.
    ///
    /// Returns: `(enter_event_mode, terminate_simulation)`
    ///
    /// The return value `enter_event_mode` signals that the importer must call
    /// [`enter_event_mode()`] to handle a step event.
    /// When `terminate_simulation` = true, the FMU requests to stop the simulation and the
    /// importer must call [`terminate()`].
    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
    ) -> Result<(bool, bool), Error>;

    /// Set a new value for the independent variable (typically a time instant).
    fn set_time(&mut self, time: f64) -> Result<(), Error>;

    /// Set new continuous state values.
    ///
    /// Arguments:
    ///
    /// * `states`: the new values for each continuous state. The order of the continuousStates
    ///             vector must be the same as the ordered list of elements in
    ///             [`model::ModelStructure::continuous_state_derivatives`].
    fn set_continuous_states(&mut self, states: &[f64]) -> Result<(), Error>;

    /// Fetch the first-order derivatives with respect to the independent variable (usually
    /// time) of the continuous states.
    ///
    /// Returns:
    /// [`FmiResult::Discard`] if the FMU was not able to compute the derivatives according to
    /// 𝐟cont because, for example, a numerical issue, such as division by zero, occurred.
    fn get_continuous_state_derivatives(&mut self, derivatives: &mut [f64]) -> Result<(), Error>;
}

/// Interface for Co-Simulation instances
pub trait CoSimulation: Common {}

/// Interface for Scheduled instances
pub trait Scheduled: Common {}
