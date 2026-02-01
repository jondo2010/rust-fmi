//! Traits for the different instance types.

use crate::{
    Error, EventFlags,
    fmi3::{Fmi3Error, Fmi3Res, binding},
};

/// Represents a single variable dependency relationship.
///
/// This structure encapsulates all the information about how one variable depends
/// on another, including element indices for array variables and the type of dependency.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDependency {
    /// Element index of the dependent variable (0 = all elements, 1+ = specific element)
    pub dependent_element_index: usize,
    /// Value reference of the independent variable this dependency relates to
    pub independent: binding::fmi3ValueReference,
    /// Element index of the independent variable (0 = all elements, 1+ = specific element)
    pub independent_element_index: usize,
    /// The kind/type of dependency relationship
    pub dependency_kind: binding::fmi3DependencyKind,
}

macro_rules! default_getter_setter {
    ($name:ident, $ty:ty) => {
        paste::paste! {
            /// Get the values of the specified variable references.
            ///
            /// See <https://fmi-standard.org/docs/3.0.1/#get-and-set-variable-values>
            fn [<get_ $name>](&mut self, _vrs: &[binding::fmi3ValueReference], _values: &mut [$ty]) -> Result<Fmi3Res, Fmi3Error> {
                unimplemented!();
            }
            /// Set the values of the specified variable references.
            ///
            /// See <https://fmi-standard.org/docs/3.0.1/#get-and-set-variable-values>
            fn [<set_ $name>](&mut self, _vrs: &[binding::fmi3ValueReference], _values: &[$ty]) -> Result<Fmi3Res, Fmi3Error> {
                unimplemented!();
            }
        }
    };
}

/// FMI Getter / Setter interface
pub trait GetSet {
    default_getter_setter!(boolean, bool);
    default_getter_setter!(float32, f32);
    default_getter_setter!(float64, f64);
    default_getter_setter!(int8, i8);
    default_getter_setter!(int16, i16);
    default_getter_setter!(int32, i32);
    default_getter_setter!(int64, i64);
    default_getter_setter!(uint8, u8);
    default_getter_setter!(uint16, u16);
    default_getter_setter!(uint32, u32);
    default_getter_setter!(uint64, u64);
    fn get_string(
        &mut self,
        _vrs: &[binding::fmi3ValueReference],
        _values: &mut [std::ffi::CString],
    ) -> Result<(), Fmi3Error> {
        unimplemented!();
    }

    fn set_string(
        &mut self,
        _vrs: &[binding::fmi3ValueReference],
        _values: &[std::ffi::CString],
    ) -> Result<(), Fmi3Error> {
        unimplemented!();
    }

    /// Get binary values from the FMU.
    ///
    /// The FMU provides pointers to its internal binary data, which we copy into the
    /// user-provided buffers. If any user buffer is too small for the corresponding
    /// binary data, an error is returned.
    ///
    /// Returns the actual sizes of the binary data that was copied.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#get-and-set-variable-values>
    fn get_binary(
        &mut self,
        _vrs: &[binding::fmi3ValueReference],
        _values: &mut [&mut [u8]],
    ) -> Result<Vec<usize>, Fmi3Error> {
        unimplemented!()
    }

    /// Set binary values in the FMU.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#get-and-set-variable-values>
    fn set_binary(
        &mut self,
        _vrs: &[binding::fmi3ValueReference],
        _values: &[&[u8]],
    ) -> Result<(), Fmi3Error> {
        unimplemented!()
    }

    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3GetFMUState>
    #[cfg(false)]
    fn get_fmu_state<Tag>(
        &mut self,
        state: Option<Fmu3State<'_, Tag>>,
    ) -> Result<Fmu3State<'_, Tag>, Error>;

    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3SetFMUState>
    #[cfg(false)]
    fn set_fmu_state<Tag>(&mut self, state: &Fmu3State<'_, Tag>) -> Fmi3Status;

    fn get_clock(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &mut [binding::fmi3Clock],
    ) -> Result<Fmi3Res, Fmi3Error>;

    fn set_clock(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &[binding::fmi3Clock],
    ) -> Result<Fmi3Res, Fmi3Error>;
}

/// Interface common to all FMI3 instance types
pub trait Common: GetSet {
    /// The FMI-standard version string
    fn get_version(&self) -> &str;

    /// The function controls the debug logging that is output by the FMU
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3SetDebugLogging>
    fn set_debug_logging(
        &mut self,
        logging_on: bool,
        categories: &[&str],
    ) -> Result<Fmi3Res, Fmi3Error>;

    /// Changes state to Reconfiguration Mode.
    ///
    /// If the importer needs to change structural parameters, it must move the FMU into Configuration Mode using `enter_configuration_mode()`.
    ///
    /// [`Common::enter_configuration_mode()`] must not be called if the FMU contains no tunable structural parameters (i.e. with `causality` =
    /// [`crate::fmi3::schema::Causality::StructuralParameter`] and `variability` = [`crate::fmi3::schema::Variability::Tunable`]).
    ///
    /// See <https://fmi-standard.org/docs/3.0/#fmi3EnterConfigurationMode>
    fn enter_configuration_mode(&mut self) -> Result<Fmi3Res, Fmi3Error>;

    /// Exits the Configuration Mode and returns to state Instantiated.
    ///
    /// See <https://fmi-standard.org/docs/3.0/#fmi3ExitConfigurationMode>
    fn exit_configuration_mode(&mut self) -> Result<Fmi3Res, Fmi3Error>;

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
    ) -> Result<Fmi3Res, Fmi3Error>;

    /// Changes the state, depending on the instance type:
    /// * Model Exchange: Event Mode
    /// * Co-Simulation:
    ///     * `event_mode_used = true`: Event Mode
    ///     * `event_mode_used = false`: Step Mode
    /// * Scheduled Execution: Clock Activation Mode.
    fn exit_initialization_mode(&mut self) -> Result<Fmi3Res, Fmi3Error>;

    /// This function changes the state to Event Mode.
    ///
    /// The importer must call `enter_event_mode` when any of the following conditions are met:
    /// * time has reached nextEventTime as returned by fmi3UpdateDiscreteStates, or
    /// * the signs of the event indicators signal an event according to Section 3.1.1, or
    /// * the FMU returned with enterEventMode = fmi3True from fmi3CompletedIntegratorStep, or
    /// * the importer plans discrete changes to inputs, or an input Clock needs to be set.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3EnterEventMode>
    fn enter_event_mode(&mut self) -> Result<Fmi3Res, Fmi3Error>;

    /// Changes state to [`Terminated`](https://fmi-standard.org/docs/3.0.1/#Terminated).
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3Terminate>
    fn terminate(&mut self) -> Result<Fmi3Res, Fmi3Error>;

    /// Is called by the environment to reset the FMU after a simulation run.
    /// The FMU goes into the same state as if newly created. All variables have their default
    /// values. Before starting a new run [`Common::enter_initialization_mode()`] has to be called.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3Reset>
    fn reset(&mut self) -> Result<Fmi3Res, Fmi3Error>;

    /// This function is called to signal a converged solution at the current super-dense time
    /// instant. `update_discrete_states` must be called at least once per super-dense time
    /// instant.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3UpdateDiscreteStates>
    fn update_discrete_states(
        &mut self,
        event_flags: &mut EventFlags,
    ) -> Result<Fmi3Res, Fmi3Error>;

    /// This function returns the number of dependencies for a given variable.
    fn get_number_of_variable_dependencies(
        &mut self,
        vr: binding::fmi3ValueReference,
    ) -> Result<usize, Fmi3Error>;

    /// This function returns the dependency information for a single variable.
    ///
    /// Returns a vector of dependencies for the specified dependent variable.
    /// Each dependency describes how an element of the dependent variable
    /// depends on an element of an independent variable.
    ///
    /// The returned dependencies correspond to either:
    /// - Initial dependencies (if called before `exit_initialization_mode`)
    /// - Runtime dependencies (if called after `exit_initialization_mode`)
    ///
    /// The dependency information becomes invalid when structural parameters
    /// linked to the variable or its dependencies are modified.
    ///
    /// # Arguments
    /// * `dependent` - Value reference of the variable for which dependencies are requested
    ///
    /// # Returns
    /// * `Ok(dependencies)` - Vector of dependency relationships
    /// * `Err(Fmi3Error)` - If the operation fails
    fn get_variable_dependencies(
        &mut self,
        dependent: binding::fmi3ValueReference,
    ) -> Result<Vec<VariableDependency>, Fmi3Error>;
}

/// Interface for Model Exchange instances
pub trait ModelExchange: Common {
    /// This function must be called to change from Event Mode into Continuous-Time Mode in Model Exchange.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3EnterContinuousTimeMode>
    fn enter_continuous_time_mode(&mut self) -> Result<Fmi3Res, Fmi3Error>;

    /// This function is called after every completed step of the integrator provided the capability
    /// flag [`crate::fmi3::schema::Fmi3ModelExchange::needs_completed_integrator_step`] =
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
    ) -> Result<Fmi3Res, Fmi3Error>;

    /// Set a new value for the independent variable (typically a time instant).
    ///
    /// Argument time is the new value for the real part `𝑡𝑅` of `𝑡:=(𝑡𝑅,0)`. It refers to the unit of
    /// the independent variable. time must be larger or equal to:
    /// * `start_time`,
    /// * the time at the second last call to [`ModelExchange::completed_integrator_step`],
    /// * the time at the last call to [`Common::enter_event_mode`].
    ///
    /// This allows limited simulation backward in time. As soon as an event occurs
    /// ([`Common::enter_event_mode`] was called), going back in time is impossible, because
    /// [`Common::enter_event_mode`] / [`Common::update_discrete_states`] can only compute the next
    /// discrete state, not the previous one.
    ///
    /// See: <https://fmi-standard.org/docs/3.0.1/#fmi3SetTime>
    fn set_time(&mut self, time: f64) -> Result<Fmi3Res, Fmi3Error>;

    /// Set new continuous state values.
    ///
    /// Arguments:
    ///
    /// * `states`: the new values for each continuous state. The order of the continuousStates
    ///   vector must be the same as the ordered list of elements in
    ///   [`crate::fmi3::schema::ModelStructure::continuous_state_derivatives`].
    fn set_continuous_states(&mut self, states: &[f64]) -> Result<Fmi3Res, Fmi3Error>;

    /// Return the current continuous state vector.
    ///
    /// Arguments:
    ///
    /// * `continuous_states`: returns the values for each continuous state with the same convention
    ///   for the order as defined for [`ModelExchange::set_continuous_states()`].
    ///
    /// See: <https://fmi-standard.org/docs/3.0.1/#fmi3GetContinuousStates>
    fn get_continuous_states(
        &mut self,
        continuous_states: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error>;

    /// Fetch the first-order derivatives with respect to the independent variable (usually
    /// time) of the continuous states.
    ///
    /// Returns:
    /// [`crate::fmi3::Fmi3Error::Discard`] if the FMU was not able to compute the derivatives
    /// according to 𝐟cont because, for example, a numerical issue, such as division by zero,
    /// occurred.
    fn get_continuous_state_derivatives(
        &mut self,
        states: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error>;

    /// Return the nominal values of the continuous states.
    ///
    /// Returns:
    ///
    /// * `nominals`: returns the nominal values for each continuous state with the same convention
    ///   for the order as defined for [`ModelExchange::set_continuous_states()`]. If the FMU does
    ///   not have information about the nominal value of a continuous state i, a nominal value
    ///   `nominals[i] = 1.0` should be returned. It is required that `nominals[i] > 0.0`.
    ///
    /// This function should always be called after calling function
    /// [`Common::update_discrete_states()`], if `nominals_of_continuous_states_changed =
    /// true`, since then the nominal values of the continuous states have changed (for example,
    /// because the mapping of the continuous states to variables has changed because of internal
    /// dynamic state selection).
    ///
    /// See: <https://fmi-standard.org/docs/3.0.1/#fmi3GetNominalsOfContinuousStates>
    fn get_nominals_of_continuous_states(
        &mut self,
        nominals: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error>;

    /// Returns the event indicators signaling state events by their sign changes.
    ///
    /// Arguments:
    /// * `event_indicators`: returns the values for the event indicators in the order defined by
    ///   the ordered list of XML elements `<EventIndicator>`.
    ///
    /// Returns:
    /// * `Ok(true)` if the event indicators were successfully computed
    /// * `Ok(false)` if the FMU was not able to compute the event indicators according to 𝐟cont
    ///   because, for example, a numerical issue such as division by zero occurred (corresponding
    ///   to the C API returning `fmi3Discard`)
    /// * `Err(Fmi3Error)` for other error conditions
    ///
    /// See: <https://fmi-standard.org/docs/3.0.1/#fmi3GetEventIndicators>
    fn get_event_indicators(&mut self, _event_indicators: &mut [f64]) -> Result<bool, Fmi3Error> {
        unimplemented!()
    }

    /// This function returns the number of event indicators.
    ///
    /// See: <https://fmi-standard.org/docs/3.0/#fmi3GetNumberOfEventIndicators>
    fn get_number_of_event_indicators(&mut self) -> Result<usize, Fmi3Error>;

    /// This function returns the number of continuous states.
    ///
    /// See: <https://fmi-standard.org/docs/3.0/#fmi3GetNumberOfContinuousStates>
    fn get_number_of_continuous_states(&mut self) -> Result<usize, Fmi3Error>;
}

/// Interface for Co-Simulation instances
pub trait CoSimulation: Common {
    /// This function must be called to change from Event Mode into Step Mode in Co-Simulation.
    fn enter_step_mode(&mut self) -> Result<Fmi3Res, Fmi3Error>;

    /// The returned values correspond to the derivatives at the current time of the FMU. For example, after a
    /// successful call to [`CoSimulation::do_step`], the returned values are related to the end of the communication
    /// step.
    ///
    /// Arguments:
    /// * `vrs`: the variables whose derivatives shall be retrieved. If multiple derivatives of a variable shall be
    /// retrieved, list the value reference multiple times.
    /// * `orders`: the orders of the respective derivative (1 means the first derivative, 2 means the second
    /// derivative, …​, 0 is not allowed). If multiple derivatives of a variable shall be retrieved, its value reference
    /// must occur multiple times in valueReferences aligned with the corresponding orders array.
    /// * `values`: the values of the derivatives are returned in this array. The order of the values corresponds to the
    /// order of the value references. Array elements are laid out contiguously.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3GetOutputDerivatives>
    fn get_output_derivatives(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        orders: &[i32],
        values: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error>;

    /// The importer requests the computation of the next time step.
    ///
    /// Arguments:
    /// * `current_communication_point`: the current communication point of the importer (`t_i`). At the first call of
    /// `do_step`, must be equal to the argument `start_time` of `enter_initialization_mode`.
    /// * `communication_step_size`: the communication step size (`h_i`). Must be >0.0. The FMU is expected to compute until
    /// time `t_i+1 = t_i + h_i`.
    ///
    /// See: <https://fmi-standard.org/docs/3.0.1/#fmi3DoStep>
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
    ) -> Result<Fmi3Res, Fmi3Error>;
}

/// Interface for Scheduled instances
///
/// The Scheduled Execution interface provides support for concurrent computation of model partitions on a single computational resource (e.g. CPU-core).
///
/// See <https://fmi-standard.org/docs/3.0.1/#fmi-for-scheduled-execution>
pub trait ScheduledExecution: Common {
    /// Each `activate_model_partition` call relates to one input Clock which triggers the computation of its associated
    /// model partition.
    ///
    /// Arguments:
    /// * `clock_reference`: `ValueReference` of the input Clock associated with the model partition which shall be
    /// activated.
    /// * `activation_time`: value of the independent variable of the assigned Clock tick time ti [typically: simulation
    /// (i.e. virtual) time] (which is known to the simulation algorithm).
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3ActivateModelPartition>
    fn activate_model_partition(
        &mut self,
        clock_reference: binding::fmi3ValueReference,
        activation_time: f64,
    ) -> Result<Fmi3Res, Fmi3Error>;
}

/// Fmi3Model trait for types that support instantiating instances.
pub trait Fmi3Model {
    type InstanceME: ModelExchange;
    type InstanceCS: CoSimulation;
    type InstanceSE: ScheduledExecution;

    /// Create a new instance of the FMU for Model-Exchange
    ///
    /// See [`crate::fmi3::instance::InstanceME::new`] for more information.
    fn instantiate_me(
        &self,
        _instance_name: &str,
        _visible: bool,
        _logging_on: bool,
    ) -> Result<Self::InstanceME, Error> {
        Err(Error::UnsupportedInterface(
            "Model-Exchange is not supported".to_string(),
        ))
    }

    /// Create a new instance of the FMU for Co-Simulation
    ///
    /// See [`crate::fmi3::instance::InstanceCS::new`] for more information.
    fn instantiate_cs(
        &self,
        _instance_name: &str,
        _visible: bool,
        _logging_on: bool,
        _event_mode_used: bool,
        _early_return_allowed: bool,
        _required_intermediate_variables: &[binding::fmi3ValueReference],
    ) -> Result<Self::InstanceCS, Error> {
        Err(Error::UnsupportedInterface(
            "Co-Simulation is not supported".to_string(),
        ))
    }

    /// Create a new instance of the FMU for Scheduled Execution
    ///
    /// See [`crate::fmi3::instance::InstanceSE::new`] for more information.
    fn instantiate_se(
        &self,
        _instance_name: &str,
        _visible: bool,
        _logging_on: bool,
    ) -> Result<Self::InstanceSE, Error> {
        Err(Error::UnsupportedInterface(
            "Scheduled Execution is not supported".to_string(),
        ))
    }
}
