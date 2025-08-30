#[macro_export]
macro_rules! checked_deref {
    ($ptr:expr, $ty:ty) => {{
        if ($ptr as *mut ::std::os::raw::c_void).is_null() {
            eprintln!("Invalid FMU instance");
            return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
        }
        let instance = unsafe { &mut *($ptr as *mut ::fmi_export::fmi3::ModelInstance<$ty>) };
        instance
    }};
}

#[macro_export]
macro_rules! export_fmu {
    ($ty:ty) => {
        /// Export the model description as a Rust str symbol
        /// This allows extracting the XML from the compiled dylib
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub static FMI3_MODEL_DESCRIPTION: &'static str = <$ty as ::fmi_export::fmi3::Model>::MODEL_DESCRIPTION;

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn fmi3GetVersion() -> *const ::std::os::raw::c_char {
            ::fmi::fmi3::binding::fmi3Version.as_ptr() as *const _
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn fmi3SetDebugLogging(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            logging_on: ::fmi::fmi3::binding::fmi3Boolean,
            n_categories: usize,
            categories: *const ::fmi::fmi3::binding::fmi3String,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            let categories = ::std::slice::from_raw_parts(categories, n_categories)
                .into_iter()
                .filter_map(|cat| ::std::ffi::CStr::from_ptr(*cat).to_str().ok())
                .collect::<::std::vec::Vec<_>>();
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::Common>::set_debug_logging(instance, logging_on, &categories) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        /* Creation and destruction of FMU instances */
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3InstantiateModelExchange(
            instance_name: ::fmi::fmi3::binding::fmi3String,
            instantiation_token: ::fmi::fmi3::binding::fmi3String,
            resource_path: ::fmi::fmi3::binding::fmi3String,
            _visible: ::fmi::fmi3::binding::fmi3Boolean,
            logging_on: ::fmi::fmi3::binding::fmi3Boolean,
            _instance_environment: ::fmi::fmi3::binding::fmi3InstanceEnvironment,
            log_message: ::fmi::fmi3::binding::fmi3LogMessageCallback,
        ) -> ::fmi::fmi3::binding::fmi3Instance {
            let name = ::std::ffi::CStr::from_ptr(instance_name)
                .to_string_lossy()
                .into_owned();
            let token = ::std::ffi::CStr::from_ptr(instantiation_token).to_string_lossy();
            let resource_path = ::std::path::PathBuf::from(
                ::std::ffi::CStr::from_ptr(resource_path)
                    .to_string_lossy()
                    .into_owned(),
            );

            if let Some(log_message) = log_message {
                // Wrap the C callback in a Rust closure
                let log_message =
                    Box::new(move |status: ::fmi::fmi3::Fmi3Status, category: &str, args: std::fmt::Arguments<'_>| {
                        let category_c = ::std::ffi::CString::new(category).unwrap_or_default();
                        let message_c = ::std::ffi::CString::new(args.to_string()).unwrap_or_default();
                        unsafe {
                            log_message(
                                std::ptr::null_mut() as ::fmi::fmi3::binding::fmi3InstanceEnvironment,
                                status.into(),
                                category_c.as_ptr(),
                                message_c.as_ptr(),
                            )
                        };
                    });

                match ::fmi_export::fmi3::ModelInstance::<$ty>::new(
                    name,
                    resource_path,
                    logging_on,
                    log_message,
                    &token,
                ) {
                    Ok(instance) => {
                        let this: ::std::boxed::Box<dyn ::fmi::fmi3::Common<ValueRef = ::fmi::fmi3::binding::fmi3ValueReference>> =
                            ::std::boxed::Box::new(instance);
                        ::std::boxed::Box::into_raw(this) as ::fmi::fmi3::binding::fmi3Instance
                    }
                    Err(_) => {
                        eprintln!("Failed to instantiate FMU: invalid instantiation token");
                        ::std::ptr::null_mut()
                    }
                }
            } else {
                eprintln!("Error: No log message callback provided");
                return ::std::ptr::null_mut();
            }
        }
        //#define fmi3InstantiateCoSimulation          fmi3FullName(fmi3InstantiateCoSimulation)
        //#define fmi3InstantiateScheduledExecution    fmi3FullName(fmi3InstantiateScheduledExecution)

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3FreeInstance(instance: ::fmi::fmi3::binding::fmi3Instance) {
            if instance.is_null() {
                eprintln!("Invalid FMU instance");
                return;
            }
            let _this = ::std::boxed::Box::from_raw(instance as *mut ::fmi_export::fmi3::ModelInstance<$ty>);
            _this.context().log(
                Fmi3Res::OK,
                Default::default(),
                format_args!("{}: fmi3FreeInstance()", _this.instance_name()),
            );
            // instance will be dropped here, freeing resources
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3EnterInitializationMode(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            tolerance_defined: ::fmi::fmi3::binding::fmi3Boolean,
            tolerance: ::fmi::fmi3::binding::fmi3Float64,
            start_time: ::fmi::fmi3::binding::fmi3Float64,
            stop_time_defined: ::fmi::fmi3::binding::fmi3Boolean,
            stop_time: ::fmi::fmi3::binding::fmi3Float64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            let tolerance = tolerance_defined.then_some(tolerance);
            let stop_time = stop_time_defined.then_some(stop_time);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::Common>::enter_initialization_mode(instance, tolerance, start_time, stop_time) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3ExitInitializationMode(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::Common>::exit_initialization_mode(instance) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3EnterEventMode(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::Common>::enter_event_mode(instance) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3EnterContinuousTimeMode(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::ModelExchange>::enter_continuous_time_mode(instance) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3SetTime(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            time: ::fmi::fmi3::binding::fmi3Float64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::ModelExchange>::set_time(instance, time) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3SetContinuousStates(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            continuous_states: *const ::fmi::fmi3::binding::fmi3Float64,
            n_continuous_states: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            let states = ::std::slice::from_raw_parts(continuous_states, n_continuous_states);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::ModelExchange>::set_continuous_states(instance, states) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3GetContinuousStates(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            continuous_states: *mut ::fmi::fmi3::binding::fmi3Float64,
            n_continuous_states: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            let states = ::std::slice::from_raw_parts_mut(continuous_states, n_continuous_states);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::ModelExchange>::get_continuous_states(instance, states) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3GetContinuousStateDerivatives(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            derivatives: *mut ::fmi::fmi3::binding::fmi3Float64,
            n_continuous_states: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            let derivs = ::std::slice::from_raw_parts_mut(derivatives, n_continuous_states);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::ModelExchange>::get_continuous_state_derivatives(instance, derivs) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3GetNumberOfEventIndicators(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            n_event_indicators: *mut usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::ModelExchange>::get_number_of_event_indicators(instance) {
                Ok(n) => {
                    *n_event_indicators = n;
                    ::fmi::fmi3::binding::fmi3Status_fmi3OK
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3GetEventIndicators(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            event_indicators: *mut ::fmi::fmi3::binding::fmi3Float64,
            n_event_indicators: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            let indicators = ::std::slice::from_raw_parts_mut(event_indicators, n_event_indicators);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::ModelExchange>::get_event_indicators(instance, indicators) {
                Ok(_) => ::fmi::fmi3::binding::fmi3Status_fmi3OK,
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3CompletedIntegratorStep(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            no_set_fmu_state_prior: ::fmi::fmi3::binding::fmi3Boolean,
            enter_event_mode: *mut ::fmi::fmi3::binding::fmi3Boolean,
            terminate_simulation: *mut ::fmi::fmi3::binding::fmi3Boolean,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            let mut enter_event = false;
            let mut terminate = false;
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::ModelExchange>::completed_integrator_step(
                instance,
                no_set_fmu_state_prior,
                &mut enter_event,
                &mut terminate,
            ) {
                Ok(_) => {
                    *enter_event_mode = enter_event;
                    *terminate_simulation = terminate;
                    ::fmi::fmi3::binding::fmi3Status_fmi3OK
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3Terminate(instance: ::fmi::fmi3::binding::fmi3Instance) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::Common>::terminate(instance) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3Reset(instance: ::fmi::fmi3::binding::fmi3Instance) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::Common>::reset(instance) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3UpdateDiscreteStates(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            discrete_states_need_update: *mut ::fmi::fmi3::binding::fmi3Boolean,
            terminate_simulation: *mut ::fmi::fmi3::binding::fmi3Boolean,
            nominals_of_continuous_states_changed: *mut ::fmi::fmi3::binding::fmi3Boolean,
            values_of_continuous_states_changed: *mut ::fmi::fmi3::binding::fmi3Boolean,
            next_event_time_defined: *mut ::fmi::fmi3::binding::fmi3Boolean,
            next_event_time: *mut ::fmi::fmi3::binding::fmi3Float64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);

            let discrete_states_need_update: &mut bool = &mut *discrete_states_need_update;
            let terminate_simulation: &mut bool = &mut *terminate_simulation;
            let nominals_of_continuous_states_changed: &mut bool = &mut *nominals_of_continuous_states_changed;
            let values_of_continuous_states_changed: &mut bool = &mut *values_of_continuous_states_changed;

            let mut event_time = None;

            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::Common>::update_discrete_states(
                instance,
                discrete_states_need_update,
                terminate_simulation,
                nominals_of_continuous_states_changed,
                values_of_continuous_states_changed,
                &mut event_time,
            ) {
                Ok(res) => {
                    if let Some(event_time) = event_time {
                        *next_event_time_defined = true;
                        *next_event_time = event_time;
                    } else {
                        *next_event_time_defined = false;
                        *next_event_time = 0.0;
                    }

                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe fn fmi3GetFloat64(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            values: *mut ::fmi::fmi3::binding::fmi3Float64,
            n_values: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);
            let value_refs = ::std::slice::from_raw_parts(value_references, n_value_references);
            let values = ::std::slice::from_raw_parts_mut(values, n_values);
            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::GetSet>::get_float64(instance, value_refs, values) {
                Ok(res) => {
                    let status: ::fmi::fmi3::Fmi3Status = res.into();
                    status.into()
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }
    };
}
