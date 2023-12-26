//! Traits for different instance types ([ModelExchange], [CoSimulation]).

use crate::fmi2::{EventInfo, StatusKind};

use super::{binding, Fmi2Status};

/// Interface common to both ModelExchange and CoSimulation
pub trait Common {
    /// The instance name
    fn name(&self) -> &str;

    /// The FMI-standard version string
    fn version(&self) -> &str;

    fn set_debug_logging(&self, logging_on: bool, categories: &[&str]) -> Fmi2Status;

    /// Informs the FMU to setup the experiment. This function can be called after `instantiate()`
    /// and before `enter_initialization_mode()` is called.
    ///
    /// ## Tolerance control
    /// * Under ModelExchange: If tolerance = Some(..) then the model is called with a numerical
    ///   integration scheme where the step size is controlled by using `tolerance` for error
    ///   estimation (usually as relative tolerance). In such a case, all numerical algorithms used
    ///   inside the model (for example to solve non-linear algebraic equations) should also operate
    ///   with an error estimation of an appropriate smaller relative tolerance.
    /// * Under CoSimulation: If tolerance = Some(..) then the communication interval of the slave
    ///   is controlled by error estimation. In case the slave utilizes a numerical integrator with
    ///   variable step size and error estimation, it is suggested to use `tolerance` for the error
    ///   estimation of the internal integrator (usually as relative tolerance). An FMU for
    ///   Co-Simulation might ignore this argument.
    ///
    /// ## Start and Stop times
    /// The arguments `start_time` and `stop_time can be used to check whether the model is valid
    /// within the given boundaries. Argument `start_time` is the fixed initial value of the
    /// independent variable [if the independent variable is "time", `start_time` is the starting
    /// time of initializaton]. If `stop_time` is `Some(..)`, then `stop_time` is the defined final
    /// value of the independent variable [if the independent variable is "time", `stop_time` is
    /// the stop time of the simulation] and if the environment tries to compute past `stop_time`
    /// the FMU has to return `Error`. If `stop_time` is `None()`, then no final value of the
    /// independent variable is defined.
    fn setup_experiment(
        &self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Fmi2Status;

    /// Informs the FMU to enter Initialization Mode.
    ///
    /// Before calling this function, all variables with attribute
    /// `<ScalarVariable initial = "exact" or "approx">` can be set with the `set()` function.
    /// *Setting other variables is not allowed*. Furthermore, `setup_experiment()` must be called
    /// at least once before calling `enter_initialization_mode()`, in order that `start_time` is
    /// defined.
    fn enter_initialization_mode(&self) -> Fmi2Status;

    /// Informs the FMU to exit Initialization Mode.
    ///
    /// Under ModelExchange this function switches off all initialization equations and the FMU
    /// enters implicitely Event Mode, that is all continuous-time and active discrete-time
    /// equations are available.
    fn exit_initialization_mode(&self) -> Fmi2Status;

    /// Informs the FMU that the simulation run is terminated.
    ///
    /// After calling this function, the final values of all variables can be inquired with the
    /// fmi2GetXXX(..) functions. It is not allowed to call this function after one of the
    /// functions returned with a status flag of fmi2Error or fmi2Fatal.
    fn terminate(&self) -> Fmi2Status;

    /// Is called by the environment to reset the FMU after a simulation run.
    ///
    /// The FMU goes into the same state as if fmi2Instantiate would have been called. All
    /// variables have their default values. Before starting a new run, fmi2SetupExperiment and
    /// fmi2EnterInitializationMode have to be called.
    fn reset(&self) -> Fmi2Status;

    fn get_real(
        &self,
        sv: &[binding::fmi2ValueReference],
        v: &mut [binding::fmi2Real],
    ) -> Fmi2Status;
    fn get_integer(
        &self,
        sv: &[binding::fmi2ValueReference],
        v: &mut [binding::fmi2Integer],
    ) -> Fmi2Status;
    fn get_boolean(
        &self,
        sv: &[binding::fmi2ValueReference],
        v: &mut [binding::fmi2Boolean],
    ) -> Fmi2Status;
    fn get_string(
        &self,
        sv: &[binding::fmi2ValueReference],
        v: &mut [binding::fmi2String],
    ) -> Fmi2Status;

    /// Set real values
    ///
    /// # Arguments
    /// * `vrs` - a slice of `fmi::fmi2ValueReference` ValueReferences
    /// * `values` - a slice of `fmi::fmi2Real` values to set
    fn set_real(
        &self,
        vrs: &[binding::fmi2ValueReference],
        values: &[binding::fmi2Real],
    ) -> Fmi2Status;

    /// Set integer values
    ///
    /// # Arguments
    /// * `vrs` - a slice of `fmi::fmi2ValueReference` ValueReferences
    /// * `values` - a slice of `fmi::fmi2Integer` values to set
    fn set_integer(
        &self,
        vrs: &[binding::fmi2ValueReference],
        values: &[binding::fmi2Integer],
    ) -> Fmi2Status;

    fn set_boolean(
        &self,
        vrs: &[binding::fmi2ValueReference],
        values: &mut [binding::fmi2Boolean],
    ) -> Fmi2Status;

    fn set_string(
        &self,
        vrs: &[binding::fmi2ValueReference],
        values: &[binding::fmi2String],
    ) -> Fmi2Status;

    // fn get_fmu_state(&self) -> Result<FmuState>;
    // fn set_fmu_state(&self, state: &FmuState<Self::Api>) -> Result<()>;
    // fn free_fmu_state(&self, state: FmuState<Self::Api>) -> Result<()>;
    //
    // Serializes the data which is referenced by pointer FMUstate and copies this data in to the
    // byte slice of length size, that must be provided by the environment.
    // fn serialize_fmu_state(&self, state: &FmuState<Self::Api>) -> Result<Vec<u8>>;
    //
    // Deserializes the byte vector data into an FmuState
    // fn deserialize_fmu_state(&self, data: &Vec<u8>) -> Result<FmuState<Self::Api>>;

    /// It is optionally possible to provide evaluation of partial derivatives for an FMU. For Model
    /// Exchange, this means computing the partial derivatives at a particular time instant. For
    /// Co-Simulation, this means to compute the partial derivatives at a particular communication
    /// point. One function is provided to compute directional derivatives. This function can be
    /// used to construct the desired partial derivative matrices.
    fn get_directional_derivative(
        &self,
        unknown_vrs: &[binding::fmi2ValueReference],
        known_vrs: &[binding::fmi2ValueReference],
        dv_known_values: &[binding::fmi2Real],
        dv_unknown_values: &mut [binding::fmi2Real],
    ) -> Fmi2Status;
}

pub trait ModelExchange: Common {
    // fn set_fmu_state(&self, state: fmi2FMUstate) -> Result<()>;

    /// The model enters Event Mode from the Continuous-Time Mode and discrete-time equations may
    /// become active (and relations are not "frozen").
    fn enter_event_mode(&self) -> Fmi2Status;

    /// The FMU is in Event Mode and the super dense time is incremented by this call. If the super dense time before a
    /// call to [`ModelExchange::new_discrete_states`] was `(tR,tI)` then the time instant after the call is
    /// `(tR,tI + 1)`.
    ///
    /// If returned EventInfo.new_discrete_states_needed = true, the FMU should stay in Event Mode and the FMU requires
    /// to set new inputs to the FMU (`set_XXX` on inputs), to compute and get the outputs (`get_XXX` on outputs) and to
    /// call `new_discrete_states()` again.
    ///
    /// Depending on the connection with other FMUs, the environment shall
    ///   * call [`Common::terminate`], if `terminate_simulation = true` is returned by at least one FMU,
    ///   * call [`ModelExchange::enter_continuous_time_mode`] if all FMUs return `new_discrete_states_needed = false`.
    ///   * stay in Event Mode otherwise.
    fn new_discrete_states(&self, event_info: &mut EventInfo) -> Fmi2Status;

    /// The model enters Continuous-Time Mode and all discrete-time equations become inactive and
    /// all relations are "frozen".
    ///
    /// This function has to be called when changing from Event Mode (after the global event
    /// iteration in Event Mode over all involved FMUs and other models has converged) into
    /// Continuous-Time Mode.
    fn enter_continuous_time_mode(&self) -> Fmi2Status;

    /// Complete integrator step and return enterEventMode.
    ///
    /// This function must be called by the environment after every completed step of the
    /// integrator provided the capability flag completedIntegratorStepNotNeeded = false.
    /// Argument `no_set_fmu_state_prior_to_current_point` is true if `set_fmu_state` will no
    /// longer be called for time instants prior to current time in this simulation run [the FMU
    /// can use this flag to flush a result buffer].
    ///
    /// The returned tuple are the flags (enter_event_mode, terminate_simulation)
    fn completed_integrator_step(
        &self,
        no_set_fmu_state_prior_to_current_point: bool,
    ) -> (Fmi2Status, bool, bool);

    /// Set a new time instant and re-initialize caching of variables that depend on time, provided
    /// the newly provided time value is different to the previously set time value (variables that
    /// depend solely on constants or parameters need not to be newly computed in the sequel, but
    /// the previously computed values can be reused).
    fn set_time(&self, time: f64) -> Fmi2Status;

    /// Set a new (continuous) state vector and re-initialize caching of variables that depend on
    /// the states. Argument nx is the length of vector x and is provided for checking purposes
    /// (variables that depend solely on constants, parameters, time, and inputs do not need to be
    /// newly computed in the sequel, but the previously computed values can be reused).
    /// Note, the continuous states might also be changed in Event Mode.
    /// Note: fmi2Status = fmi2Discard is possible.
    fn set_continuous_states(&self, states: &[f64]) -> Fmi2Status;

    /// Compute state derivatives and event indicators at the current time instant and for the
    /// current states. The derivatives are returned as a vector with “nx” elements.
    fn get_derivatives(&self, dx: &mut [f64]) -> Fmi2Status;

    /// A state event is triggered when the domain of an event indicator changes from zj > 0 to zj ≤
    /// 0 or vice versa. The FMU must guarantee that at an event restart zj ≠ 0, for example by
    /// shifting zj with a small value. Furthermore, zj should be scaled in the FMU with its nominal
    /// value (so all elements of the returned vector “eventIndicators” should be in the order of
    /// “one”). The event indicators are returned as a vector with “ni” elements.
    fn get_event_indicators(&self, events: &mut [f64]) -> Fmi2Status;

    /// Return the new (continuous) state vector x.
    /// This function has to be called directly after calling function `enter_continuous_time_mode`
    /// if it returns with eventInfo->valuesOfContinuousStatesChanged = true (indicating that the
    /// (continuous-time) state vector has changed).
    fn get_continuous_states(&self, x: &mut [f64]) -> Fmi2Status;

    /// Return the nominal values of the continuous states. This function should always be called
    /// after calling function new_discrete_states if it returns with
    /// eventInfo->nominals_of_continuous_states = true since then the nominal values of the
    /// continuous states have changed [e.g. because the association of the continuous states to
    /// variables has changed due to internal dynamic state selection].
    ///
    /// If the FMU does not have information about the nominal value of a continuous state i, a
    /// nominal value x_nominal[i] = 1.0 should be returned.
    ///
    /// Note, it is required that x_nominal[i] > 0.0 [Typically, the nominal values of the
    /// continuous states are used to compute the absolute tolerance required by the integrator.
    /// Example: absoluteTolerance[i] = 0.01*tolerance*x_nominal[i];]
    fn get_nominals_of_continuous_states(&self, nominals: &mut [f64]) -> Fmi2Status;
}

pub trait CoSimulation: Common {
    /// The computation of a time step is started.
    ///
    /// Depending on the internal state of the slave and the last call of `do_step(...)`, the slave
    /// has to decide which action is to be done before the step is computed.
    ///
    /// # Arguments
    /// * `current_communication_point` - the current communication point of the master.
    /// * `communication_step_size` - the communication step size.
    /// * `new_step` - If true, accept the last computed step, and start another.
    fn do_step(
        &self,
        current_communication_point: f64,
        communication_step_size: f64,
        new_step: bool,
    ) -> Fmi2Status;

    /// Cancel a running asynchronous step.
    ///
    /// Can be called if `do_step(...)` returned `Pending` in order to stop the current
    /// asynchronous execution. The master calls this function if e.g. the co-simulation run is
    /// stopped by the user or one of the slaves. Afterwards it is only allowed to call the
    /// functions `terminate()` or `reset()`.
    fn cancel_step(&self) -> Fmi2Status;

    /// Inquire into slave status during asynchronous step.
    fn get_status(&self, kind: StatusKind) -> Fmi2Status;
}
