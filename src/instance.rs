use crate::FmiStatus;

use super::{fmi2, logger, model_descr, FmiError, Import, Result};
use log::trace;
use std::{ffi::CString, sync::Arc};

impl Default for fmi2::CallbackFunctions {
    fn default() -> Self {
        fmi2::CallbackFunctions {
            logger: Some(logger::callback_logger_handler),
            allocate_memory: Some(libc::calloc),
            free_memory: Some(libc::free),
            step_finished: None,
            component_environment: std::ptr::null_mut::<std::os::raw::c_void>(),
        }
    }
}

/// Check the internal consistency of the FMU by comparing the TypesPlatform and FMI versions
/// from the library and the Model Description XML
fn check_consistency(import: &Import, common: &fmi2::Common) -> Result<()> {
    let types_platform =
        unsafe { std::ffi::CStr::from_ptr(common.get_types_platform()) }.to_bytes_with_nul();

    if types_platform != fmi2::fmi2TypesPlatform {
        return Err(FmiError::TypesPlatformMismatch {
            found: types_platform.into(),
        });
    }

    let fmi_version = unsafe { std::ffi::CStr::from_ptr(common.get_version()) }.to_bytes();
    if fmi_version != import.descr.fmi_version.as_bytes() {
        return Err(FmiError::FmiVersionMismatch {
            found: fmi_version.into(),
            expected: import.descr.fmi_version.as_bytes().into(),
        });
    }

    Ok(())
}

/// Interface common to both ModelExchange and CoSimulation
pub trait Common: std::hash::Hash {
    /// The instance name
    fn name(&self) -> &str;

    /// The FMI-standard version string
    fn version(&self) -> Result<&str>;

    fn set_debug_logging(&self, logging_on: bool, categories: &[&str]) -> Result<FmiStatus>;

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
    ) -> Result<FmiStatus>;

    /// Informs the FMU to enter Initialization Mode.
    ///
    /// Before calling this function, all variables with attribute
    /// `<ScalarVariable initial = "exact" or "approx">` can be set with the `set()` function.
    /// *Setting other variables is not allowed*. Furthermore, `setup_experiment()` must be called
    /// at least once before calling `enter_initialization_mode()`, in order that `start_time` is
    /// defined.
    fn enter_initialization_mode(&self) -> Result<FmiStatus>;

    /// Informs the FMU to exit Initialization Mode.
    ///
    /// Under ModelExchange this function switches off all initialization equations and the FMU
    /// enters implicitely Event Mode, that is all continuous-time and active discrete-time
    /// equations are available.
    fn exit_initialization_mode(&self) -> Result<FmiStatus>;

    /// Informs the FMU that the simulation run is terminated.
    ///
    /// After calling this function, the final values of all variables can be inquired with the
    /// fmi2GetXXX(..) functions. It is not allowed to call this function after one of the
    /// functions returned with a status flag of fmi2Error or fmi2Fatal.
    fn terminate(&self) -> Result<FmiStatus>;

    /// Is called by the environment to reset the FMU after a simulation run.
    ///
    /// The FMU goes into the same state as if fmi2Instantiate would have been called. All
    /// variables have their default values. Before starting a new run, fmi2SetupExperiment and
    /// fmi2EnterInitializationMode have to be called.
    fn reset(&self) -> Result<FmiStatus>;

    fn get_real(&self, sv: &model_descr::ScalarVariable) -> Result<fmi2::fmi2Real>;
    fn get_integer(&self, sv: &model_descr::ScalarVariable) -> Result<fmi2::fmi2Integer>;
    fn get_boolean(&self, sv: &model_descr::ScalarVariable) -> Result<fmi2::fmi2Boolean>;
    fn get_string(&self, sv: &model_descr::ScalarVariable) -> Result<fmi2::fmi2String>;

    /// Set real values
    ///
    /// # Arguments
    /// * `vrs` - a slice of `fmi::fmi2ValueReference` ValueReferences
    /// * `values` - a slice of `fmi::fmi2Real` values to set
    fn set_real(
        &self,
        vrs: &[model_descr::ValueReference],
        values: &[fmi2::fmi2Real],
    ) -> Result<FmiStatus>;

    /// Set integer values
    ///
    /// # Arguments
    /// * `vrs` - a slice of `fmi::fmi2ValueReference` ValueReferences
    /// * `values` - a slice of `fmi::fmi2Integer` values to set
    fn set_integer(
        &self,
        vrs: &[model_descr::ValueReference],
        values: &[fmi2::fmi2Integer],
    ) -> Result<FmiStatus>;

    fn set_boolean(
        &self,
        vrs: &[fmi2::fmi2ValueReference],
        values: &[fmi2::fmi2Boolean],
    ) -> Result<FmiStatus>;

    fn set_string(
        &self,
        vrs: &[fmi2::fmi2ValueReference],
        values: &[fmi2::fmi2String],
    ) -> Result<FmiStatus>;

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
        unknown_vrs: &[fmi2::fmi2ValueReference],
        known_vrs: &[fmi2::fmi2ValueReference],
        dv_known_values: &[fmi2::fmi2Real],
        dv_unknown_values: &mut [fmi2::fmi2Real],
    ) -> Result<FmiStatus>;
}

pub trait ModelExchange: Common {
    // fn set_fmu_state(&self, state: fmi2FMUstate) -> Result<()>;

    /// The model enters Event Mode from the Continuous-Time Mode and discrete-time equations may
    /// become active (and relations are not "frozen").
    fn enter_event_mode(&self) -> Result<FmiStatus>;

    /// The FMU is in Event Mode and the super dense time is incremented by this call. If the super
    /// dense time before a call to `new_discrete_states` was (tR,tI) then the time instant after
    /// the call is (tR,tI + 1).
    ///
    /// If returned EventInfo.new_discrete_states_needed = true, the FMU should stay in Event Mode
    /// and the FMU requires to set new inputs to the FMU (`set_XXX` on inputs), to compute and
    /// get the outputs (get_XXX on outputs) and to call `new_discrete_states` again.
    /// Depending on the connection with other FMUs, the environment shall
    ///     * call `terminate`, if `terminate_simulation` = true is returned by at least one FMU,
    ///     * call `enter_continuous_time_mode` if all FMUs return `new_discrete_states_needed` =
    ///       false.
    ///     * stay in Event Mode otherwise.
    fn new_discrete_states(&self, event_info: &mut fmi2::EventInfo) -> Result<FmiStatus>;

    /// The model enters Continuous-Time Mode and all discrete-time equations become inactive and
    /// all relations are "frozen".
    ///
    /// This function has to be called when changing from Event Mode (after the global event
    /// iteration in Event Mode over all involved FMUs and other models has converged) into
    /// Continuous-Time Mode.
    fn enter_continuous_time_mode(&self) -> Result<FmiStatus>;

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
    ) -> Result<(bool, bool)>;

    /// Set a new time instant and re-initialize caching of variables that depend on time, provided
    /// the newly provided time value is different to the previously set time value (variables that
    /// depend solely on constants or parameters need not to be newly computed in the sequel, but
    /// the previously computed values can be reused).
    fn set_time(&self, time: f64) -> Result<FmiStatus>;

    /// Set a new (continuous) state vector and re-initialize caching of variables that depend on
    /// the states. Argument nx is the length of vector x and is provided for checking purposes
    /// (variables that depend solely on constants, parameters, time, and inputs do not need to be
    /// newly computed in the sequel, but the previously computed values can be reused).
    /// Note, the continuous states might also be changed in Event Mode.
    /// Note: fmi2Status = fmi2Discard is possible.
    fn set_continuous_states(&self, states: &[f64]) -> Result<FmiStatus>;

    /// Compute state derivatives and event indicators at the current time instant and for the
    /// current states. The derivatives are returned as a vector with “nx” elements.
    fn get_derivatives(&self, dx: &mut [f64]) -> Result<FmiStatus>;

    /// A state event is triggered when the domain of an event indicator changes from zj > 0 to zj ≤
    /// 0 or vice versa. The FMU must guarantee that at an event restart zj ≠ 0, for example by
    /// shifting zj with a small value. Furthermore, zj should be scaled in the FMU with its nominal
    /// value (so all elements of the returned vector “eventIndicators” should be in the order of
    /// “one”). The event indicators are returned as a vector with “ni” elements.
    fn get_event_indicators(&self, events: &mut [f64]) -> Result<FmiStatus>;

    /// Return the new (continuous) state vector x.
    /// This function has to be called directly after calling function `enter_continuous_time_mode`
    /// if it returns with eventInfo->valuesOfContinuousStatesChanged = true (indicating that the
    /// (continuous-time) state vector has changed).
    fn get_continuous_states(&self, x: &mut [f64]) -> Result<FmiStatus>;

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
    fn get_nominals_of_continuous_states(&self) -> Result<&[f64]>;
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
    ) -> Result<FmiStatus>;

    /// Cancel a running asynchronous step.
    ///
    /// Can be called if `do_step(...)` returned `Pending` in order to stop the current
    /// asynchronous execution. The master calls this function if e.g. the co-simulation run is
    /// stopped by the user or one of the slaves. Afterwards it is only allowed to call the
    /// functions `terminate()` or `reset()`.
    fn cancel_step(&self) -> Result<FmiStatus>;

    /// Inquire into slave status during asynchronous step.
    fn get_status(&self, kind: fmi2::fmi2StatusKind) -> Result<FmiStatus>;
}

/// An Instance is templated around an FMU Api, and holds state for the API container,
/// callbacks struct, and the internal instantiated component.
pub struct Instance<A: fmi2::FmiApi> {
    /// Instance name
    name: String,

    /// The model description
    descr: Arc<model_descr::ModelDescription>,

    /// API Container
    container: dlopen::wrapper::Container<A>,

    /// Callbacks struct
    #[allow(dead_code)]
    callbacks: Box<fmi2::CallbackFunctions>,

    /// Instantiated component
    component: fmi2::fmi2Component,
}

// We assume here that the exported FMUs are thread-safe (true for OpenModelica)
unsafe impl<A: fmi2::FmiApi> Send for Instance<A> {}
unsafe impl<A: fmi2::FmiApi> Sync for Instance<A> {}

/// FmuState wraps the FMUstate pointer and is used for managing FMU state
pub struct FmuState<'a, A: fmi2::FmiApi> {
    state: fmi2::fmi2FMUstate,
    container: &'a dlopen::wrapper::Container<A>,
    component: &'a fmi2::fmi2Component,
}

impl<'a, A: fmi2::FmiApi> FmuState<'a, A> {}

impl<'a, A: fmi2::FmiApi> Drop for FmuState<'a, A> {
    fn drop(&mut self) {
        trace!("Freeing FmuState");
        unsafe {
            self.container.common().free_fmu_state(
                *self.component,
                &mut self.state as *mut *mut core::ffi::c_void,
            );
        }
    }
}

pub type InstanceME = Instance<fmi2::Fmi2ME>;
pub type InstanceCS = Instance<fmi2::Fmi2CS>;

impl<A> PartialEq for Instance<A>
where
    A: fmi2::FmiApi,
{
    fn eq(&self, other: &Instance<A>) -> bool {
        self.name() == other.name()
    }
}

impl<A> Eq for Instance<A> where A: fmi2::FmiApi {}

impl<A> std::hash::Hash for Instance<A>
where
    A: fmi2::FmiApi,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state);
    }
}

impl<A> Instance<A>
where
    A: fmi2::FmiApi,
{
    pub fn num_states(&self) -> usize {
        self.descr.num_states()
    }
    pub fn num_event_indicators(&self) -> usize {
        self.descr.num_event_indicators()
    }
}

impl InstanceME {
    /// Initialize a new Instance from an Import
    pub fn new(
        import: &Import,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> Result<InstanceME> {
        let callbacks = Box::<fmi2::CallbackFunctions>::default();
        let me = import.container_me()?;
        check_consistency(import, &me.common)?;

        let comp = unsafe {
            let instance_name = CString::new(instance_name).expect("Error building CString");
            let guid = CString::new(import.descr.guid.as_bytes()).expect("Error building CString");
            let resource_url =
                CString::new(import.resource_url().as_str()).expect("Error building CString");

            me.common.instantiate(
                instance_name.as_ptr(),
                fmi2::fmi2Type::ModelExchange,
                guid.as_ptr(),                   // guid
                resource_url.as_ptr(),           // fmu_resource_location
                &*callbacks,                     // functions
                visible as fmi2::fmi2Boolean,    // visible
                logging_on as fmi2::fmi2Boolean, // logging_on
            )
        };
        if comp.is_null() {
            return Err(FmiError::Instantiation);
        }
        trace!("Created ME component {:?}", comp);

        Ok(Instance {
            name: instance_name.to_owned(),
            descr: import.descr.clone(),
            container: me,
            callbacks,
            component: comp,
        })
    }

    /// Helper for event iteration
    /// Returned tuple is (nominals_of_continuous_states_changed,
    /// values_of_continuous_states_changed)
    pub fn do_event_iteration(&self) -> Result<(bool, bool)> {
        let mut event_info = fmi2::EventInfo {
            new_discrete_states_needed: fmi2::fmi2True,
            terminate_simulation: fmi2::fmi2False,
            nominals_of_continuous_states_changed: fmi2::fmi2False,
            values_of_continuous_states_changed: fmi2::fmi2False,
            next_event_time_defined: fmi2::fmi2False,
            next_event_time: 0.0,
        };

        while (event_info.new_discrete_states_needed == fmi2::fmi2True)
            && (event_info.terminate_simulation == fmi2::fmi2False)
        {
            trace!("Iterating while new_discrete_states_needed=true");
            self.new_discrete_states(&mut event_info)?;
        }

        assert_eq!(
            event_info.terminate_simulation,
            fmi2::fmi2False,
            "terminate_simulation in=true do_event_iteration!"
        );

        Ok((
            event_info.nominals_of_continuous_states_changed == fmi2::fmi2True,
            event_info.values_of_continuous_states_changed == fmi2::fmi2True,
        ))
    }
}

impl ModelExchange for InstanceME {
    fn enter_event_mode(&self) -> Result<FmiStatus> {
        unsafe { self.container.me.enter_event_mode(self.component) }.into()
    }

    fn new_discrete_states(&self, event_info: &mut fmi2::EventInfo) -> Result<FmiStatus> {
        unsafe {
            self.container
                .me
                .new_discrete_states(self.component, event_info)
        }
        .into()
    }

    fn enter_continuous_time_mode(&self) -> Result<FmiStatus> {
        unsafe { self.container.me.enter_continuous_time_mode(self.component) }.into()
    }

    fn completed_integrator_step(
        &self,
        no_set_fmu_state_prior_to_current_point: bool,
    ) -> Result<(bool, bool)> {
        // The returned tuple are the flags (enter_event_mode, terminate_simulation)
        let mut enter_event_mode = fmi2::fmi2False;
        let mut terminate_simulation = fmi2::fmi2False;
        let res: Result<FmiStatus> = unsafe {
            self.container.me.completed_integrator_step(
                self.component,
                no_set_fmu_state_prior_to_current_point as fmi2::fmi2Boolean,
                &mut enter_event_mode,
                &mut terminate_simulation,
            )
        }
        .into();
        res.and(Ok((
            enter_event_mode == fmi2::fmi2True,
            terminate_simulation == fmi2::fmi2True,
        )))
    }

    fn set_time(&self, time: f64) -> Result<FmiStatus> {
        unsafe {
            self.container
                .me
                .set_time(self.component, time as fmi2::fmi2Real)
        }
        .into()
    }

    fn set_continuous_states(&self, states: &[f64]) -> Result<FmiStatus> {
        assert!(states.len() == self.descr.num_states());
        unsafe {
            self.container
                .me
                .set_continuous_states(self.component, states.as_ptr(), states.len())
        }
        .into()
    }

    fn get_derivatives(&self, dx: &mut [f64]) -> Result<FmiStatus> {
        assert!(
            dx.len() == self.descr.num_states(),
            "Out slice `dx` should have the same length as the number of states!"
        );
        unsafe {
            self.container
                .me
                .get_derivatives(self.component, dx.as_mut_ptr(), dx.len())
        }
        .into()
    }

    fn get_event_indicators(&self, events: &mut [f64]) -> Result<FmiStatus> {
        assert!(events.len() == self.descr.num_event_indicators());
        unsafe {
            self.container.me.get_event_indicators(
                self.component,
                events.as_mut_ptr(),
                events.len(),
            )
        }
        .into()
    }

    fn get_continuous_states(&self, states: &mut [f64]) -> Result<FmiStatus> {
        assert!(states.len() == self.descr.num_states());
        unsafe {
            self.container.me.get_continuous_states(
                self.component,
                states.as_mut_ptr(),
                states.len(),
            )
        }
        .into()
    }

    fn get_nominals_of_continuous_states(&self) -> Result<&[f64]> {
        unimplemented!();
    }
}

impl InstanceCS {
    /// Initialize a new Instance from an Import
    pub fn new(
        import: &Import,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> Result<InstanceCS> {
        let callbacks = Box::<fmi2::CallbackFunctions>::default();
        let cs = import.container_cs()?;
        check_consistency(import, &cs.common)?;

        let comp = unsafe {
            let instance_name = CString::new(instance_name).expect("Error building CString");
            let guid = CString::new(import.descr.guid.as_bytes()).expect("Error building CString");
            let resource_url =
                CString::new(import.resource_url().as_str()).expect("Error building CString");
            cs.common.instantiate(
                instance_name.as_ptr(),
                fmi2::fmi2Type::CoSimulation,
                guid.as_ptr(),                   // guid
                resource_url.as_ptr(),           // fmu_resource_location
                &*callbacks,                     // functions
                visible as fmi2::fmi2Boolean,    // visible
                logging_on as fmi2::fmi2Boolean, // logging_on
            )
        };
        if comp.is_null() {
            return Err(FmiError::Instantiation);
        }
        trace!("Created CS component {:?}", comp);

        let instance = Instance {
            name: instance_name.to_owned(),
            descr: import.descr.clone(),
            container: cs,
            callbacks,
            component: comp,
        };

        Ok(instance)
    }
}

impl CoSimulation for InstanceCS {
    fn do_step(
        &self,
        current_communication_point: f64,
        communication_step_size: f64,
        new_step: bool,
    ) -> Result<FmiStatus> {
        unsafe {
            self.container.cs.do_step(
                self.component,
                current_communication_point,
                communication_step_size,
                new_step as fmi2::fmi2Boolean,
            )
        }
        .into()
    }

    fn cancel_step(&self) -> Result<FmiStatus> {
        unsafe { self.container.cs.cancel_step(self.component) }.into()
    }

    fn get_status(&self, kind: fmi2::fmi2StatusKind) -> Result<FmiStatus> {
        let mut ret = fmi2::fmi2Status::OK;
        let _ = Result::<FmiStatus>::from(unsafe {
            self.container.cs.get_status(self.component, kind, &mut ret)
        })?;
        ret.into()
    }
}

impl<A> Common for Instance<A>
where
    A: fmi2::FmiApi,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> Result<&str> {
        unsafe { std::ffi::CStr::from_ptr(self.container.common().get_version()).to_str() }
            .map_err(FmiError::from)
    }

    fn set_debug_logging(&self, logging_on: bool, categories: &[&str]) -> Result<FmiStatus> {
        let category_cstr = categories
            .iter()
            .map(|c| CString::new(*c).unwrap())
            .collect::<Vec<_>>();

        let category_ptrs: Vec<_> = category_cstr.iter().map(|c| c.as_ptr()).collect();

        unsafe {
            self.container.common().set_debug_logging(
                self.component,
                logging_on as fmi2::fmi2Boolean,
                category_ptrs.len(),
                category_ptrs.as_ptr(),
            )
        }
        .into()
    }

    fn setup_experiment(
        &self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Result<FmiStatus> {
        unsafe {
            self.container.common().setup_experiment(
                self.component,
                tolerance.is_some() as fmi2::fmi2Boolean,
                tolerance.unwrap_or(0.0),
                start_time,
                stop_time.is_some() as fmi2::fmi2Boolean,
                stop_time.unwrap_or(0.0),
            )
        }
        .into()
    }

    fn enter_initialization_mode(&self) -> Result<FmiStatus> {
        unsafe {
            self.container
                .common()
                .enter_initialization_mode(self.component)
        }
        .into()
    }

    fn exit_initialization_mode(&self) -> Result<FmiStatus> {
        unsafe {
            self.container
                .common()
                .exit_initialization_mode(self.component)
        }
        .into()
    }

    fn terminate(&self) -> Result<FmiStatus> {
        unsafe { self.container.common().terminate(self.component) }.into()
    }

    fn reset(&self) -> Result<FmiStatus> {
        unsafe { self.container.common().reset(self.component) }.into()
    }

    fn get_real(&self, sv: &model_descr::ScalarVariable) -> Result<fmi2::fmi2Real> {
        let mut ret: fmi2::fmi2Real = 0.0;
        let res: Result<FmiStatus> = unsafe {
            self.container
                .common()
                .get_real(self.component, &sv.value_reference.0, 1, &mut ret)
        }
        .into();
        res.and(Ok(ret))
    }

    fn get_integer(&self, sv: &model_descr::ScalarVariable) -> Result<fmi2::fmi2Integer> {
        let mut ret: fmi2::fmi2Integer = 0;
        let res: Result<FmiStatus> = unsafe {
            self.container
                .common()
                .get_integer(self.component, &sv.value_reference.0, 1, &mut ret)
        }
        .into();
        res.and(Ok(ret))
    }

    fn get_boolean(&self, sv: &model_descr::ScalarVariable) -> Result<fmi2::fmi2Boolean> {
        let mut ret: fmi2::fmi2Boolean = 0;
        let res: Result<FmiStatus> = unsafe {
            self.container
                .common()
                .get_boolean(self.component, &sv.value_reference.0, 1, &mut ret)
        }
        .into();
        res.and(Ok(ret))
    }

    fn get_string(&self, _sv: &model_descr::ScalarVariable) -> Result<fmi2::fmi2String> {
        unimplemented!()
    }

    fn set_real(
        &self,
        vrs: &[model_descr::ValueReference],
        values: &[fmi2::fmi2Real],
    ) -> Result<FmiStatus> {
        assert!(vrs.len() == values.len());
        unsafe {
            self.container.common().set_real(
                self.component,
                vrs.as_ptr() as *const u32,
                values.len(),
                values.as_ptr(),
            )
        }
        .into()
    }

    // fn set_real(&self, sv: &model_descr::ScalarVariable, value: f64) -> Result<()> {
    // let vr = sv.value_reference as fmi::fmi2ValueReference;
    // let vr = &vr as *const fmi::fmi2ValueReference;
    // handle_status_u32(unsafe {
    // self.container
    // .common()
    // .set_real(self.component, vr, 1, &value as *const fmi::fmi2Real)
    // })
    // }

    fn set_integer(
        &self,
        vrs: &[model_descr::ValueReference],
        values: &[fmi2::fmi2Integer],
    ) -> Result<FmiStatus> {
        unsafe {
            self.container.common().set_integer(
                self.component,
                vrs.as_ptr() as *const u32,
                values.len(),
                values.as_ptr(),
            )
        }
        .into()
    }

    fn set_boolean(
        &self,
        vrs: &[fmi2::fmi2ValueReference],
        values: &[fmi2::fmi2Boolean],
    ) -> Result<FmiStatus> {
        unsafe {
            self.container.common().set_boolean(
                self.component,
                vrs.as_ptr(),
                values.len(),
                values.as_ptr(),
            )
        }
        .into()
    }

    fn set_string(
        &self,
        _vrs: &[fmi2::fmi2ValueReference],
        _values: &[fmi2::fmi2String],
    ) -> Result<FmiStatus> {
        unimplemented!()
    }

    // fn get_fmu_state(&self, state: *mut fmi2FMUstate) -> Result<()> {}

    // fn set_fmu_state(&self, state: &[u8]) -> Result<()> {}

    fn get_directional_derivative(
        &self,
        unknown_vrs: &[fmi2::fmi2ValueReference],
        known_vrs: &[fmi2::fmi2ValueReference],
        dv_known_values: &[fmi2::fmi2Real],
        dv_unknown_values: &mut [fmi2::fmi2Real],
    ) -> Result<FmiStatus> {
        assert!(unknown_vrs.len() == dv_unknown_values.len());
        assert!(known_vrs.len() == dv_unknown_values.len());
        unsafe {
            self.container.common().get_directional_derivative(
                self.component,
                unknown_vrs.as_ptr(),
                unknown_vrs.len(),
                known_vrs.as_ptr(),
                known_vrs.len(),
                dv_known_values.as_ptr(),
                dv_unknown_values.as_mut_ptr(),
            )
        }
        .into()
    }
}

impl<A> Drop for Instance<A>
where
    A: fmi2::FmiApi,
{
    fn drop(&mut self) {
        trace!("Freeing component {:?}", self.component);
        unsafe { self.container.common().free_instance(self.component) };
    }
}

impl<A> std::fmt::Debug for Instance<A>
where
    A: fmi2::FmiApi,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Instance {} {{Import {}, {:?}}}",
            self.name(),
            self.descr.model_name,
            self.component,
        )
    }
}

// TODO Make this work on other targets
#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_me() {
        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap();

        let instance1 = InstanceME::new(&import, "inst1", false, true).unwrap();
        assert_eq!(instance1.version().unwrap(), "2.0");

        let categories = &import
            .descr
            .log_categories
            .as_ref()
            .unwrap()
            .categories
            .iter()
            .map(|cat| cat.name.as_ref())
            .collect::<Vec<&str>>();

        instance1
            .set_debug_logging(true, categories)
            .expect("set_debug_logging");
        instance1
            .setup_experiment(Some(1.0e-6_f64), 0.0, None)
            .expect("setup_experiment");
        instance1
            .enter_initialization_mode()
            .expect("enter_initialization_mode");
        instance1
            .exit_initialization_mode()
            .expect("exit_initialization_mode");
        instance1.terminate().expect("terminate");
        instance1.reset().expect("reset");
    }

    /// Tests on variable module requiring an instance.
    #[cfg(feature = "disable")]
    #[test]
    fn test_variable() {
        use crate::{model_descr::ModelDescriptionError, Var};
        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap();

        let inst = InstanceME::new(&import, "inst1", false, true).unwrap();

        let mut vars = import.descr().get_model_variables();
        let _ = Var::from_scalar_variable(&inst, vars.next().unwrap().1);

        assert!(matches!(
            Var::from_name(&inst, "false"),
            Err(FmiError::ModelDescr(
                ModelDescriptionError::VariableNotFound { .. }
            ))
        ));
    }

    #[cfg(feature = "disable")]
    #[test]
    fn test_instance_cs() {
        use crate::{Value, Var};
        use assert_approx_eq::assert_approx_eq;

        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap();

        let instance1 = InstanceCS::new(&import, "inst1", false, true).unwrap();
        assert_eq!(instance1.version().unwrap(), "2.0");

        instance1
            .setup_experiment(Some(1.0e-6_f64), 0.0, None)
            .expect("setup_experiment");

        instance1
            .enter_initialization_mode()
            .expect("enter_initialization_mode");

        let param = Var::from_name(&instance1, "freqHz").expect("freqHz parameter from_name");
        param
            .set(&Value::Real(2.0f64))
            .expect("set freqHz parameter");

        instance1
            .exit_initialization_mode()
            .expect("exit_initialization_mode");

        let y = Var::from_name(&instance1, "y").expect("get y");

        if let Value::Real(y_val) = y.get().expect("get y value") {
            assert_approx_eq!(y_val, 0.0, 1.0e-6);
        }

        instance1.do_step(0.0, 0.125, false).expect("do_step");

        if let Value::Real(y_val) = y.get().expect("get y value") {
            assert_approx_eq!(y_val, 1.0, 1.0e-6);
        }
    }
}
