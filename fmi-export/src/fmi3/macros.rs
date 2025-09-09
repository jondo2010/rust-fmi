/// Safely dereferences an FMI instance pointer and validates it.
///
/// This macro performs null-pointer checks and safely casts the opaque FMI instance
/// pointer to a mutable reference to the actual ModelInstance type.
///
/// # Safety
///
/// This macro performs unsafe pointer dereferencing. The caller must ensure that:
/// - The pointer was originally created by this library's instantiation functions
/// - The pointer has not been freed via `fmi3FreeInstance`
/// - The pointer points to a valid `ModelInstance<$ty>`
///
/// # Parameters
///
/// - `$ptr`: The FMI instance pointer to dereference
/// - `$ty`: The model type that the instance should contain
///
/// # Returns
///
/// Returns a mutable reference to the ModelInstance, or returns early with
/// `fmi3Status_fmi3Error` if the pointer is null.
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

/// Generates getter and setter functions for FMI3 data types.
///
/// This macro creates both `fmi3Get{TypeName}` and `fmi3Set{TypeName}` functions
/// with proper error handling, parameter validation, and consistent API behavior.
///
/// # Parameters
///
/// - `$ty`: The model type
/// - `$type_name`: The FMI type name (e.g., Float64, Int32, Boolean)
/// - `$fmi_type`: The corresponding FMI C type (e.g., fmi3Float64)
/// - `$get_method`: The trait method name for getting values (e.g., get_float64)
/// - `$set_method`: The trait method name for setting values (e.g., set_float64)
///
/// # Generated Functions
///
/// - `fmi3Get{TypeName}`: Retrieves values from the model
/// - `fmi3Set{TypeName}`: Sets values in the model
///
/// Both functions include:
/// - Null pointer validation
/// - Array length validation
/// - Proper error handling and status conversion
/// - Safe slice creation from raw pointers
#[macro_export]
macro_rules! generate_getset_functions {
    ($ty:ty, $type_name:ident, $fmi_type:ty, $get_method:ident, $set_method:ident) => {
        $crate::paste::paste! {
            #[unsafe(no_mangle)]
            #[allow(non_snake_case)]
            pub unsafe extern "C" fn [<fmi3Get $type_name>](
                instance: ::fmi::fmi3::binding::fmi3Instance,
                value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
                n_value_references: usize,
                values: *mut $fmi_type,
                n_values: usize,
            ) -> ::fmi::fmi3::binding::fmi3Status {
                let instance = $crate::checked_deref!(instance, $ty);
                
                // Validate array lengths match
                if n_value_references != n_values {
                    eprintln!("FMI3: Array length mismatch in fmi3Get{}: value_references={}, values={}", 
                             stringify!($type_name), n_value_references, n_values);
                    return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
                }
                
                let value_refs = ::std::slice::from_raw_parts(value_references, n_value_references);
                let values = ::std::slice::from_raw_parts_mut(values, n_values);
                
                match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::GetSet>::$get_method(
                    instance, value_refs, values
                ) {
                    Ok(res) => {
                        let status: ::fmi::fmi3::Fmi3Status = res.into();
                        status.into()
                    }
                    Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
                }
            }

            #[unsafe(no_mangle)]
            #[allow(non_snake_case)]
            pub unsafe extern "C" fn [<fmi3Set $type_name>](
                instance: ::fmi::fmi3::binding::fmi3Instance,
                value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
                n_value_references: usize,
                values: *const $fmi_type,
                n_values: usize,
            ) -> ::fmi::fmi3::binding::fmi3Status {
                let instance = $crate::checked_deref!(instance, $ty);
                
                // Validate array lengths match
                if n_value_references != n_values {
                    eprintln!("FMI3: Array length mismatch in fmi3Set{}: value_references={}, values={}", 
                             stringify!($type_name), n_value_references, n_values);
                    return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
                }
                
                let value_refs = ::std::slice::from_raw_parts(value_references, n_value_references);
                let values = ::std::slice::from_raw_parts(values, n_values);
                
                match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::GetSet>::$set_method(
                    instance, value_refs, values
                ) {
                    Ok(res) => {
                        let status: ::fmi::fmi3::Fmi3Status = res.into();
                        status.into()
                    }
                    Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
                }
            }
        }
    };
}/// Main macro for exporting an FMI 3.0 model as a shared library.
///
/// This macro generates all the required C API functions for an FMI 3.0 Functional Mockup Unit (FMU).
/// It creates the complete interface needed for Model Exchange simulation mode, including:
///
/// - Static exports for model metadata (variables, structure, instantiation token)
/// - Instance lifecycle functions (instantiate, free)
/// - Initialization and mode transition functions
/// - Model Exchange specific functions (continuous states, derivatives, events)
/// - Simulation control functions (terminate, reset, update discrete states)
///
/// # Usage
///
/// ## Basic Export
/// ```ignore
/// // Export a model type as an FMU
/// fmi_export::export_fmu!(MyModel);
/// ```
///
/// ## With Getter/Setter Functions
/// ```ignore
/// // Export with variable access functions
/// fmi_export::export_fmu!(MyModel);
/// fmi_export::export_fmu!(GetSet);
/// ```
///
/// # Parameters
///
/// - `$ty`: The model type that implements the `Model` trait
/// - `GetSet`: Optional parameter to generate getter/setter functions for all FMI data types
///
/// # Requirements
///
/// The model type `$ty` must implement:
/// - `Model` trait: Provides model metadata and core simulation functionality
/// - `Default`: For creating initial model instances
/// - `UserModel`: For user-defined model behavior
///
/// # Generated Functions
///
/// ## Metadata Functions
/// - `fmi3GetVersion()`: Returns FMI version string
///
/// ## Lifecycle Functions
/// - `fmi3InstantiateModelExchange()`: Creates new model instance
/// - `fmi3FreeInstance()`: Destroys model instance and frees resources
///
/// ## Initialization Functions
/// - `fmi3EnterInitializationMode()`: Enters initialization mode
/// - `fmi3ExitInitializationMode()`: Exits initialization mode and finalizes setup
///
/// ## Mode Transition Functions
/// - `fmi3EnterEventMode()`: Enters event mode for discrete computations
/// - `fmi3EnterContinuousTimeMode()`: Enters continuous time mode
///
/// ## Model Exchange Functions
/// - `fmi3SetTime()`: Sets current simulation time
/// - `fmi3SetContinuousStates()`: Sets continuous state values
/// - `fmi3GetContinuousStates()`: Gets continuous state values
/// - `fmi3GetContinuousStateDerivatives()`: Gets state derivatives
/// - `fmi3GetNumberOfEventIndicators()`: Gets event indicator count
/// - `fmi3GetEventIndicators()`: Gets event indicator values
/// - `fmi3CompletedIntegratorStep()`: Signals integrator step completion
///
/// ## Control Functions
/// - `fmi3SetDebugLogging()`: Configures debug logging
/// - `fmi3Terminate()`: Terminates simulation
/// - `fmi3Reset()`: Resets model to initial state
/// - `fmi3UpdateDiscreteStates()`: Updates discrete states and events
///
/// ## Variable Access Functions (when GetSet is specified)
/// - `fmi3Get*()` / `fmi3Set*()`: For all FMI data types (Float64, Int32, Boolean, etc.)
/// - Includes proper handling of String and Binary types
///
/// # Static Exports
///
/// The macro also exports static symbols that can be extracted from the compiled library:
/// - `FMI3_MODEL_VARIABLES`: XML description of model variables
/// - `FMI3_MODEL_STRUCTURE`: XML description of model structure
/// - `FMI3_INSTANTIATION_TOKEN`: Unique token for model validation
///
/// # Safety
///
/// All generated functions include appropriate safety checks:
/// - Null pointer validation for instance parameters
/// - Array bounds checking for multi-value operations
/// - Proper error handling and status reporting
/// - Safe conversion between C and Rust data types
#[macro_export]
macro_rules! export_fmu {
    ($ty:ty) => {
        /// Export the model components as separate Rust str symbols
        /// This allows extracting the individual XML components from the compiled dylib
        #[unsafe(no_mangle)]
        pub static FMI3_MODEL_VARIABLES: &'static str = <$ty as ::fmi_export::fmi3::Model>::MODEL_VARIABLES_XML;

        #[unsafe(no_mangle)]
        pub static FMI3_MODEL_STRUCTURE: &'static str = <$ty as ::fmi_export::fmi3::Model>::MODEL_STRUCTURE_XML;

        #[unsafe(no_mangle)]
        pub static FMI3_INSTANTIATION_TOKEN: &'static str = <$ty as ::fmi_export::fmi3::Model>::INSTANTIATION_TOKEN;

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
            let mut event_flags = ::fmi::EventFlags::default();

            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::Common>::update_discrete_states(
                instance,
                &mut event_flags
            ) {
                Ok(res) => {
                    *discrete_states_need_update = event_flags.discrete_states_need_update;
                    *terminate_simulation = event_flags.terminate_simulation;
                    *nominals_of_continuous_states_changed = event_flags.nominals_of_continuous_states_changed;
                    *values_of_continuous_states_changed = event_flags.values_of_continuous_states_changed;

                    if let Some(event_time) = event_flags.next_event_time {
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

        // Generate getter/setter functions for all FMI3 basic data types
        $crate::generate_getset_functions!($ty, Float64, ::fmi::fmi3::binding::fmi3Float64, get_float64, set_float64);
        $crate::generate_getset_functions!($ty, Float32, ::fmi::fmi3::binding::fmi3Float32, get_float32, set_float32);
        $crate::generate_getset_functions!($ty, Int64, ::fmi::fmi3::binding::fmi3Int64, get_int64, set_int64);
        $crate::generate_getset_functions!($ty, Int32, ::fmi::fmi3::binding::fmi3Int32, get_int32, set_int32);
        $crate::generate_getset_functions!($ty, Int16, ::fmi::fmi3::binding::fmi3Int16, get_int16, set_int16);
        $crate::generate_getset_functions!($ty, Int8, ::fmi::fmi3::binding::fmi3Int8, get_int8, set_int8);
        $crate::generate_getset_functions!($ty, UInt64, ::fmi::fmi3::binding::fmi3UInt64, get_uint64, set_uint64);
        $crate::generate_getset_functions!($ty, UInt32, ::fmi::fmi3::binding::fmi3UInt32, get_uint32, set_uint32);
        $crate::generate_getset_functions!($ty, UInt16, ::fmi::fmi3::binding::fmi3UInt16, get_uint16, set_uint16);
        $crate::generate_getset_functions!($ty, UInt8, ::fmi::fmi3::binding::fmi3UInt8, get_uint8, set_uint8);
        $crate::generate_getset_functions!($ty, Boolean, ::fmi::fmi3::binding::fmi3Boolean, get_boolean, set_boolean);

        // String and Binary types need special handling due to their different signatures
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn fmi3GetString(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            values: *mut ::fmi::fmi3::binding::fmi3String,
            n_values: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);

            if n_value_references != n_values {
                eprintln!("FMI3: Array length mismatch in fmi3GetString: value_references={}, values={}",
                         n_value_references, n_values);
                return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
            }

            let value_refs = ::std::slice::from_raw_parts(value_references, n_value_references);

            // Create temporary buffer for CString results
            let mut temp_strings = vec![::std::ffi::CString::default(); n_values];

            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::GetSet>::get_string(
                instance, value_refs, &mut temp_strings
            ) {
                Ok(_) => {
                    // Copy C string pointers to output array
                    let values_slice = ::std::slice::from_raw_parts_mut(values, n_values);
                    for (i, cstring) in temp_strings.iter().enumerate() {
                        values_slice[i] = cstring.as_ptr();
                    }
                    ::fmi::fmi3::binding::fmi3Status_fmi3OK
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn fmi3SetString(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            values: *const ::fmi::fmi3::binding::fmi3String,
            n_values: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);

            if n_value_references != n_values {
                eprintln!("FMI3: Array length mismatch in fmi3SetString: value_references={}, values={}",
                         n_value_references, n_values);
                return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
            }

            let value_refs = ::std::slice::from_raw_parts(value_references, n_value_references);
            let string_ptrs = ::std::slice::from_raw_parts(values, n_values);

            // Convert C strings to CString objects
            let mut temp_strings = Vec::with_capacity(n_values);
            for &ptr in string_ptrs {
                if ptr.is_null() {
                    temp_strings.push(::std::ffi::CString::default());
                } else {
                    let cstring = ::std::ffi::CStr::from_ptr(ptr).to_owned();
                    temp_strings.push(cstring);
                }
            }

            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::GetSet>::set_string(
                instance, value_refs, &temp_strings
            ) {
                Ok(_) => ::fmi::fmi3::binding::fmi3Status_fmi3OK,
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn fmi3GetBinary(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            value_sizes: *mut usize,
            values: *mut *mut ::fmi::fmi3::binding::fmi3Byte,
            n_values: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);

            if n_value_references != n_values {
                eprintln!("FMI3: Array length mismatch in fmi3GetBinary: value_references={}, values={}",
                         n_value_references, n_values);
                return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
            }

            let value_refs = ::std::slice::from_raw_parts(value_references, n_value_references);
            let sizes_slice = ::std::slice::from_raw_parts_mut(value_sizes, n_values);
            let values_slice = ::std::slice::from_raw_parts_mut(values, n_values);

            // Create temporary buffers for binary data
            let mut temp_buffers: Vec<&mut [u8]> = Vec::with_capacity(n_values);
            for i in 0..n_values {
                if values_slice[i].is_null() || sizes_slice[i] == 0 {
                    temp_buffers.push(&mut []);
                } else {
                    let buffer = ::std::slice::from_raw_parts_mut(values_slice[i], sizes_slice[i]);
                    temp_buffers.push(buffer);
                }
            }

            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::GetSet>::get_binary(
                instance, value_refs, &mut temp_buffers
            ) {
                Ok(actual_sizes) => {
                    // Update the actual sizes
                    for (i, &size) in actual_sizes.iter().enumerate() {
                        sizes_slice[i] = size;
                    }
                    ::fmi::fmi3::binding::fmi3Status_fmi3OK
                }
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn fmi3SetBinary(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            value_sizes: *const usize,
            values: *const *const ::fmi::fmi3::binding::fmi3Byte,
            n_values: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            let instance = $crate::checked_deref!(instance, $ty);

            if n_value_references != n_values {
                eprintln!("FMI3: Array length mismatch in fmi3SetBinary: value_references={}, values={}",
                         n_value_references, n_values);
                return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
            }

            let value_refs = ::std::slice::from_raw_parts(value_references, n_value_references);
            let sizes_slice = ::std::slice::from_raw_parts(value_sizes, n_values);
            let values_slice = ::std::slice::from_raw_parts(values, n_values);

            // Create temporary slices for binary data
            let mut temp_buffers: Vec<&[u8]> = Vec::with_capacity(n_values);
            for i in 0..n_values {
                if values_slice[i].is_null() || sizes_slice[i] == 0 {
                    temp_buffers.push(&[]);
                } else {
                    let buffer = ::std::slice::from_raw_parts(values_slice[i], sizes_slice[i]);
                    temp_buffers.push(buffer);
                }
            }

            match <::fmi_export::fmi3::ModelInstance<$ty> as ::fmi::fmi3::GetSet>::set_binary(
                instance, value_refs, &temp_buffers
            ) {
                Ok(_) => ::fmi::fmi3::binding::fmi3Status_fmi3OK,
                Err(_) => ::fmi::fmi3::binding::fmi3Status_fmi3Error,
            }
        }
    }
}