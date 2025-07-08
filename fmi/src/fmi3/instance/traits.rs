//! Traits for the different instance types.

use crate::traits::FmiInstance;

use super::Fmi3Status;

/// Interface common to all instance types
pub trait Common: FmiInstance {
    /// The FMI-standard version string
    fn get_version(&self) -> &str;

    /// The function controls the debug logging that is output by the FMU
    ///
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3SetDebugLogging]
    fn set_debug_logging(&mut self, logging_on: bool, categories: &[&str]) -> Fmi3Status;

    /// Changes state to Reconfiguration Mode.
    ///
    /// If the importer needs to change structural parameters, it must move the FMU into Configuration Mode using `enter_configuration_mode()`.
    ///
    /// [`Common::enter_configuration_mode()`] must not be called if the FMU contains no tunable structural parameters (i.e. with `causality` =
    /// [`crate::fmi3::schema::Causality::StructuralParameter`] and `variability` = [`crate::fmi3::schema::Variability::Tunable`]).
    ///
    /// See [https://fmi-standard.org/docs/3.0/#fmi3EnterConfigurationMode]
    fn enter_configuration_mode(&mut self) -> Fmi3Status;

    /// Exits the Configuration Mode and returns to state Instantiated.
    ///
    /// See [https://fmi-standard.org/docs/3.0/#fmi3ExitConfigurationMode]
    fn exit_configuration_mode(&mut self) -> Fmi3Status;

    /// Changes state to `Initialization Mode`.
    ///
    /// tolerance depend on the interface type:
    /// * Model Exchange: If `tolerance = Some(True)`, then the model is called with a numerical
    ///   integration scheme where the step size is controlled by using tolerance for error
    ///   estimation (usually as relative tolerance). In such a case all numerical algorithms used
    ///   inside the model (for example, to solve nonlinear algebraic equations) should also operate
    ///   with an error estimation of an appropriate smaller relative tolerance.
    /// * Co-Simulation: If `tolerance = Some(True)`, then the communication step size of the FMU is
    ///   controlled by error estimation. In case the FMU utilizes a numerical integrator with
    ///   variable step size and error estimation, it is suggested to use tolerance for the error
    ///   estimation of the integrator (usually as relative tolerance).
    ///
    /// An FMU for Co-Simulation might ignore this argument.
    fn enter_initialization_mode(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Fmi3Status;

    /// Changes the state, depending on the instance type:
    /// * Model Exchange: Event Mode
    /// * Co-Simulation:
    ///     * `event_mode_used = true`: Event Mode
    ///     * `event_mode_used = false`: Step Mode
    /// * Scheduled Execution: Clock Activation Mode.
    fn exit_initialization_mode(&mut self) -> Fmi3Status;

    /// This function changes the state to Event Mode.
    ///
    /// The importer must call `enter_event_mode` when any of the following conditions are met:
    /// * time has reached nextEventTime as returned by fmi3UpdateDiscreteStates, or
    /// * the signs of the event indicators signal an event according to Section 3.1.1, or
    /// * the FMU returned with enterEventMode = fmi3True from fmi3CompletedIntegratorStep, or
    /// * the importer plans discrete changes to inputs, or an input Clock needs to be set.
    ///
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3EnterEventMode]
    fn enter_event_mode(&mut self) -> Fmi3Status;

    /// Changes state to [`Terminated`](https://fmi-standard.org/docs/3.0.1/#Terminated).
    ///
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3Terminate]
    fn terminate(&mut self) -> Fmi3Status;

    /// Is called by the environment to reset the FMU after a simulation run.
    /// The FMU goes into the same state as if newly created. All variables have their default
    /// values. Before starting a new run [`Common::enter_initialization_mode()`] has to be called.
    ///
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3Reset]
    fn reset(&mut self) -> Fmi3Status;

    /// See [https://fmi-standard.org/docs/3.0.1/#get-and-set-variable-values]
    fn get_boolean(&mut self, vrs: &[Self::ValueRef], values: &mut [bool]) -> Fmi3Status;
    fn get_float32(&mut self, vrs: &[Self::ValueRef], values: &mut [f32]) -> Fmi3Status;
    fn get_float64(&mut self, vrs: &[Self::ValueRef], values: &mut [f64]) -> Fmi3Status;
    fn get_int8(&mut self, vrs: &[Self::ValueRef], values: &mut [i8]) -> Fmi3Status;
    fn get_int16(&mut self, vrs: &[Self::ValueRef], values: &mut [i16]) -> Fmi3Status;
    fn get_int32(&mut self, vrs: &[Self::ValueRef], values: &mut [i32]) -> Fmi3Status;
    fn get_int64(&mut self, vrs: &[Self::ValueRef], values: &mut [i64]) -> Fmi3Status;
    fn get_uint8(&mut self, vrs: &[Self::ValueRef], values: &mut [u8]) -> Fmi3Status;
    fn get_uint16(&mut self, vrs: &[Self::ValueRef], values: &mut [u16]) -> Fmi3Status;
    fn get_uint32(&mut self, vrs: &[Self::ValueRef], values: &mut [u32]) -> Fmi3Status;
    fn get_uint64(&mut self, vrs: &[Self::ValueRef], values: &mut [u64]) -> Fmi3Status;
    fn get_string(&mut self, vrs: &[Self::ValueRef], values: &mut [String]) -> Fmi3Status;
    fn get_binary(&mut self, vrs: &[Self::ValueRef], values: &mut [Vec<u8>]) -> Fmi3Status;

    fn set_boolean(&mut self, vrs: &[Self::ValueRef], values: &[bool]) -> Fmi3Status;
    fn set_float32(&mut self, vrs: &[Self::ValueRef], values: &[f32]) -> Fmi3Status;
    fn set_float64(&mut self, vrs: &[Self::ValueRef], values: &[f64]) -> Fmi3Status;
    fn set_int8(&mut self, vrs: &[Self::ValueRef], values: &[i8]) -> Fmi3Status;
    fn set_int16(&mut self, vrs: &[Self::ValueRef], values: &[i16]) -> Fmi3Status;
    fn set_int32(&mut self, vrs: &[Self::ValueRef], values: &[i32]) -> Fmi3Status;
    fn set_int64(&mut self, vrs: &[Self::ValueRef], values: &[i64]) -> Fmi3Status;
    fn set_uint8(&mut self, vrs: &[Self::ValueRef], values: &[u8]) -> Fmi3Status;
    fn set_uint16(&mut self, vrs: &[Self::ValueRef], values: &[u16]) -> Fmi3Status;
    fn set_uint32(&mut self, vrs: &[Self::ValueRef], values: &[u32]) -> Fmi3Status;
    fn set_uint64(&mut self, vrs: &[Self::ValueRef], values: &[u64]) -> Fmi3Status;
    fn set_string<'b>(
        &mut self,
        vrs: &[Self::ValueRef],
        values: impl Iterator<Item = &'b str>,
    ) -> Fmi3Status;
    fn set_binary<'b>(
        &mut self,
        vrs: &[Self::ValueRef],
        values: impl Iterator<Item = &'b [u8]>,
    ) -> Fmi3Status;

    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3GetFMUState]
    #[cfg(false)]
    fn get_fmu_state<Tag>(
        &mut self,
        state: Option<Fmu3State<'_, Tag>>,
    ) -> Result<Fmu3State<'_, Tag>, Error>;

    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3SetFMUState]
    #[cfg(false)]
    fn set_fmu_state<Tag>(&mut self, state: &Fmu3State<'_, Tag>) -> Fmi3Status;

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
    ) -> Fmi3Status;
}

/// Interface for Model Exchange instances
pub trait ModelExchange: Common {
    fn enter_continuous_time_mode(&mut self) -> Fmi3Status;

    /// This function is called after every completed step of the integrator provided the capability
    /// flag [`schema::interface_type::Fmi3ModelExchange::needs_completed_integrator_step`] =
    /// true.
    ///
    /// The importer must have set valid values for time, continuous inputs and continuous
    /// states prior to calling this function to evaluate 𝐟comp with valid right-hand side data.
    ///
    /// Arguments:
    ///
    /// * `no_set_fmu_state_prior`: if `set_fmu_state()` will no longer be called for time instants
    ///   prior to current time in this simulation run.
    ///
    /// `enter_event_mode` signals that the importer must call [`Common::enter_event_mode()`] to handle a step event.
    ///
    /// When `terminate_simulation` = true, the FMU requests to stop the simulation and the
    /// importer must call [`Common::terminate()`].
    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
        enter_event_mode: &mut bool,
        terminate_simulation: &mut bool,
    ) -> Self::Status;

    /// Set a new value for the independent variable (typically a time instant).
    ///
    /// Argument time is the new value for the real part 𝑡𝑅 of 𝑡:=(𝑡𝑅,0). It refers to the unit of
    /// the independent variable. time must be larger or equal to:
    /// * `startTime`,
    /// * the time at the second last call to fmi3CompletedIntegratorStep,
    /// * the time at the last call to fmi3EnterEventMode.
    ///
    /// [This allows limited simulation backward in time. As soon as an event occurs
    /// (fmi3EnterEventMode was called), going back in time is impossible, because
    /// fmi3EnterEventMode / fmi3UpdateDiscreteStates can only compute the next discrete state,
    /// not the previous one.]
    ///
    /// See: [https://fmi-standard.org/docs/3.0.1/#fmi3SetTime]
    fn set_time(&mut self, time: f64) -> Fmi3Status;

    /// Set new continuous state values.
    ///
    /// Arguments:
    ///
    /// * `states`: the new values for each continuous state. The order of the continuousStates
    ///   vector must be the same as the ordered list of elements in
    ///   [`crate::fmi3::schema::ModelStructure::continuous_state_derivatives`].
    fn set_continuous_states(&mut self, states: &[f64]) -> Fmi3Status;

    /// Return the current continuous state vector.
    ///
    /// Arguments:
    ///
    /// * `continuous_states`: returns the values for each continuous state with the same convention
    ///   for the order as defined for [`ModelExchange::set_continuous_states()`].
    ///
    /// See: [https://fmi-standard.org/docs/3.0.1/#fmi3GetContinuousStates]
    fn get_continuous_states(&mut self, continuous_states: &mut [f64]) -> Fmi3Status;

    /// Fetch the first-order derivatives with respect to the independent variable (usually
    /// time) of the continuous states.
    ///
    /// Returns:
    /// [`crate::fmi3::Fmi3Err::Discard`] if the FMU was not able to compute the derivatives
    /// according to 𝐟cont because, for example, a numerical issue, such as division by zero,
    /// occurred.
    fn get_continuous_state_derivatives(&mut self, states: &mut [f64]) -> Fmi3Status;

    /// Return the nominal values of the continuous states.
    ///
    /// Returns:
    ///
    /// * `nominals`: returns the nominal values for each continuous state with the same convention
    ///   for the order as defined for [`ModelExchange::set_continuous_states()`]. If the FMU does
    ///   not have information about the nominal value of a continuous state i, a nominal value
    ///   nominals[i] = 1.0 should be returned. It is required that nominals[i] > 0.0.
    ///
    /// This function should always be called after calling function
    /// [`ModelExchange::update_discrete_states()`], if `nominals_of_continuous_states_changed =
    /// true`, since then the nominal values of the continuous states have changed [for example,
    /// because the mapping of the continuous states to variables has changed because of internal
    /// dynamic state selection].
    ///
    /// See: [https://fmi-standard.org/docs/3.0.1/#fmi3GetNominalsOfContinuousStates]
    fn get_nominals_of_continuous_states(&mut self, nominals: &mut [f64]) -> Fmi3Status;

    /// Returns the event indicators signaling state events by their sign changes.
    ///
    /// Arguments:
    /// * `event_indicators`: returns the values for the event indicators in the order defined by
    ///   the ordered list of XML elements <EventIndicator>.
    ///
    /// [fmi3Status = fmi3Discard should be returned if the FMU was not able to compute the event
    /// indicators according to 𝐟cont because, for example, a numerical issue, such as division
    /// by zero, occurred.]
    ///
    /// See: [https://fmi-standard.org/docs/3.0.1/#fmi3GetEventIndicators]
    fn get_event_indicators(&mut self, event_indicators: &mut [f64]) -> Fmi3Status;

    /// This function returns the number of event indicators.
    ///
    /// See: [https://fmi-standard.org/docs/3.0/#fmi3GetNumberOfEventIndicators]
    fn get_number_of_event_indicators(&self, number_of_event_indicators: &mut usize) -> Fmi3Status;
}

/// Interface for Co-Simulation instances
pub trait CoSimulation: Common {
    /// This function must be called to change from Event Mode into Step Mode in Co-Simulation.
    fn enter_step_mode(&mut self) -> Fmi3Status;

    /// The importer requests the computation of the next time step.
    ///
    /// Arguments:
    /// See: [https://fmi-standard.org/docs/3.0.1/#fmi3DoStep]
    #[allow(clippy::too_many_arguments)]
    fn do_step(
        &mut self,
        current_communication_point: f64,
        communication_step_size: f64,
        no_set_fmu_state_prior_to_current_point: bool,
        event_handling_needed: &mut bool,
        terminate_simulation: &mut bool,
        early_return: &mut bool,
        last_successful_time: &mut f64,
    ) -> Fmi3Status;
}

/// Interface for Scheduled instances
///
/// The Scheduled Execution interface provides support for concurrent computation of model partitions on a single computational resource (e.g. CPU-core).
///
/// See [https://fmi-standard.org/docs/3.0.1/#fmi-for-scheduled-execution]
pub trait ScheduledExecution: Common {
    /// Each `activate_model_partition` call relates to one input Clock which triggers the computation of its associated model partition.
    ///
    /// Arguments:
    /// * `clock_reference`: `ValueReference` of the input Clock associated with the model partition which shall be activated.
    /// * `activation_time`: value of the independent variable of the assigned Clock tick time 𝑡𝑖 [typically: simulation (i.e. virtual) time] (which is known to the simulation algorithm).
    ///
    /// See [https://fmi-standard.org/docs/3.0.1/#fmi3ActivateModelPartition]
    fn activate_model_partition(
        &mut self,
        clock_reference: Self::ValueRef,
        activation_time: f64,
    ) -> Fmi3Status;
}
