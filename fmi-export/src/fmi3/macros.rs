#[macro_export]
macro_rules! checked_deref {
    ($ptr:expr, $ty:ty) => {{
        if $ptr.is_null() {
            eprintln!("Invalid FMU instance");
            return binding::fmi3Status_fmi3Error;
        }
        let instance = unsafe { &mut *($ptr as *mut fmi_export::fmi3::ModelInstance<$ty>) };
        instance
    }};
}

#[macro_export]
macro_rules! export_fmu {
    ($ty:ty) => {
        use fmi::fmi3::{Common, Fmi3Status, GetSet, ModelExchange, binding};
        use fmi_export::checked_deref;

        /// Export the model description as a Rust str symbol
        /// This allows extracting the XML from the compiled dylib
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub static FMI3_MODEL_DESCRIPTION: &'static str = <$ty>::MODEL_DESCRIPTION;

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn fmi3GetVersion() -> *const ::std::os::raw::c_char {
            binding::fmi3Version.as_ptr() as *const _
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn fmi3SetDebugLogging(
            instance: binding::fmi3Instance,
            logging_on: binding::fmi3Boolean,
            n_categories: usize,
            categories: *const binding::fmi3String,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            let categories = std::slice::from_raw_parts(categories, n_categories)
                .into_iter()
                .filter_map(|cat| std::ffi::CStr::from_ptr(*cat).to_str().ok())
                .collect::<Vec<_>>();
            match instance.set_debug_logging(logging_on, &categories) {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        /* Creation and destruction of FMU instances */
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3InstantiateModelExchange(
            instance_name: binding::fmi3String,
            instantiation_token: binding::fmi3String,
            resource_path: binding::fmi3String,
            _visible: binding::fmi3Boolean,
            logging_on: binding::fmi3Boolean,
            _instance_environment: binding::fmi3InstanceEnvironment,
            log_message: binding::fmi3LogMessageCallback,
        ) -> binding::fmi3Instance {
            let name = std::ffi::CStr::from_ptr(instance_name)
                .to_string_lossy()
                .into_owned();
            let token = std::ffi::CStr::from_ptr(instantiation_token).to_string_lossy();
            let resource_path = std::path::PathBuf::from(
                std::ffi::CStr::from_ptr(resource_path)
                    .to_string_lossy()
                    .into_owned(),
            );

            match fmi_export::fmi3::ModelInstance::<$ty>::new(
                name,
                resource_path,
                logging_on,
                log_message,
                &token,
            ) {
                Ok(instance) => {
                    let this: Box<dyn ::fmi::fmi3::Common<ValueRef = binding::fmi3ValueReference>> =
                        Box::new(instance);
                    Box::into_raw(this) as binding::fmi3Instance
                }
                Err(_) => {
                    eprintln!("Failed to instantiate FMU: invalid instantiation token");
                    std::ptr::null_mut()
                }
            }
        }
        //#define fmi3InstantiateCoSimulation          fmi3FullName(fmi3InstantiateCoSimulation)
        //#define fmi3InstantiateScheduledExecution    fmi3FullName(fmi3InstantiateScheduledExecution)

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3FreeInstance(instance: binding::fmi3Instance) {
            if instance.is_null() {
                eprintln!("Invalid FMU instance");
                return;
            }
            let _this = Box::from_raw(instance as *mut fmi_export::fmi3::ModelInstance<$ty>);
            // instance will be dropped here, freeing resources
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3EnterInitializationMode(
            instance: binding::fmi3Instance,
            tolerance_defined: binding::fmi3Boolean,
            tolerance: binding::fmi3Float64,
            start_time: binding::fmi3Float64,
            stop_time_defined: binding::fmi3Boolean,
            stop_time: binding::fmi3Float64,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            let tolerance = tolerance_defined.then_some(tolerance);
            let stop_time = stop_time_defined.then_some(stop_time);
            match instance.enter_initialization_mode(tolerance, start_time, stop_time) {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3ExitInitializationMode(
            instance: binding::fmi3Instance,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            match instance.exit_initialization_mode() {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3EnterEventMode(
            instance: binding::fmi3Instance,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            match instance.enter_event_mode() {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3EnterContinuousTimeMode(
            instance: binding::fmi3Instance,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            match instance.enter_continuous_time_mode() {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3SetTime(
            instance: binding::fmi3Instance,
            time: binding::fmi3Float64,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            match instance.set_time(time) {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3SetContinuousStates(
            instance: binding::fmi3Instance,
            continuous_states: *const binding::fmi3Float64,
            n_continuous_states: usize,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            let states = std::slice::from_raw_parts(continuous_states, n_continuous_states);
            match instance.set_continuous_states(states) {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3GetContinuousStates(
            instance: binding::fmi3Instance,
            continuous_states: *mut binding::fmi3Float64,
            n_continuous_states: usize,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            let states = std::slice::from_raw_parts_mut(continuous_states, n_continuous_states);
            match instance.get_continuous_states(states) {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3GetContinuousStateDerivatives(
            instance: binding::fmi3Instance,
            derivatives: *mut binding::fmi3Float64,
            n_continuous_states: usize,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            let derivs = std::slice::from_raw_parts_mut(derivatives, n_continuous_states);
            match instance.get_continuous_state_derivatives(derivs) {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3GetNumberOfEventIndicators(
            instance: binding::fmi3Instance,
            n_event_indicators: *mut usize,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            match instance.get_number_of_event_indicators() {
                Ok(n) => {
                    *n_event_indicators = n;
                    binding::fmi3Status_fmi3OK
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3GetEventIndicators(
            instance: binding::fmi3Instance,
            event_indicators: *mut binding::fmi3Float64,
            n_event_indicators: usize,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            let indicators = std::slice::from_raw_parts_mut(event_indicators, n_event_indicators);
            match instance.get_event_indicators(indicators) {
                Ok(_) => binding::fmi3Status_fmi3OK,
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3CompletedIntegratorStep(
            instance: binding::fmi3Instance,
            no_set_fmu_state_prior: binding::fmi3Boolean,
            enter_event_mode: *mut binding::fmi3Boolean,
            terminate_simulation: *mut binding::fmi3Boolean,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            let mut enter_event = false;
            let mut terminate = false;
            match instance.completed_integrator_step(
                no_set_fmu_state_prior,
                &mut enter_event,
                &mut terminate,
            ) {
                Ok(_) => {
                    *enter_event_mode = enter_event;
                    *terminate_simulation = terminate;
                    binding::fmi3Status_fmi3OK
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3Terminate(instance: binding::fmi3Instance) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            match instance.terminate() {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3Reset(instance: binding::fmi3Instance) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            match instance.reset() {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe fn fmi3GetFloat64(
            instance: binding::fmi3Instance,
            value_references: *const binding::fmi3ValueReference,
            n_value_references: usize,
            values: *mut binding::fmi3Float64,
            n_values: usize,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty);
            let value_refs = std::slice::from_raw_parts(value_references, n_value_references);
            let values = std::slice::from_raw_parts_mut(values, n_values);
            match instance.get_float64(value_refs, values) {
                Ok(res) => {
                    let status: Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => binding::fmi3Status_fmi3Error,
            }
        }
    };
}
