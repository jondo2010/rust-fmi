#[macro_export]
macro_rules! checked_deref {
    ($ptr:expr, $ty:ty, $trait:ty) => {{
        if $ptr.is_null() {
            log::error!("Invalid FMU instance");
            return binding::fmi3Status_fmi3Error;
        }
        let instance = unsafe { &mut *($ptr as *mut crate::fmi3::ModelInstance<$ty>) };
        instance
    }};
}

#[macro_export]
macro_rules! export_fmu {
    ($ty:ty) => {
        use crate::checked_deref;
        use crate::fmi3::binding;

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn fmi3GetVersion() -> *const ::std::os::raw::c_char {
            binding::fmi3Version.as_ptr() as *const _
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn fmi3SetDebugLogging(
            instance: binding::fmi3Instance,
            logging_on: binding::fmi3Boolean,
            n_categories: usize,
            categories: *const binding::fmi3String,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty, Fmi3Instance);
            let categories = std::slice::from_raw_parts(categories, n_categories)
                .into_iter()
                .filter_map(|cat| std::ffi::CStr::from_ptr(*cat).to_str().ok())
                .collect::<Vec<_>>();
            instance.set_debug_logging(logging_on, &categories).into()
        }

        /* Creation and destruction of FMU instances */
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3InstantiateModelExchange(
            instance_name: binding::fmi3String,
            _instantiation_token: binding::fmi3String,
            resource_path: binding::fmi3String,
            _visible: binding::fmi3Boolean,
            logging_on: binding::fmi3Boolean,
            _instance_environment: binding::fmi3InstanceEnvironment,
            log_message: binding::fmi3LogMessageCallback,
        ) -> binding::fmi3Instance {
            let name = std::ffi::CStr::from_ptr(instance_name)
                .to_string_lossy()
                .into_owned();
            let resource_path = std::path::PathBuf::from(
                std::ffi::CStr::from_ptr(resource_path)
                    .to_string_lossy()
                    .into_owned(),
            );
            let instance = crate::fmi3::ModelInstance::<$ty>::new(
                name,
                resource_path,
                logging_on,
                log_message,
            );
            let this: Box<dyn ::fmi::fmi3::Common<ValueRef = binding::fmi3ValueReference>> =
                Box::new(instance);
            Box::into_raw(this) as binding::fmi3Instance
        }
        //#define fmi3InstantiateCoSimulation          fmi3FullName(fmi3InstantiateCoSimulation)
        //#define fmi3InstantiateScheduledExecution    fmi3FullName(fmi3InstantiateScheduledExecution)

        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3FreeInstance(instance: binding::fmi3Instance) {
            if instance.is_null() {
                log::error!("Invalid FMU instance");
                return;
            }
            let _this = Box::from_raw(instance as *mut crate::fmi3::ModelInstance<$ty>);
            // instance will be dropped here, freeing resources
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3EnterInitializationMode(
            instance: binding::fmi3Instance,
            tolerance_defined: binding::fmi3Boolean,
            tolerance: binding::fmi3Float64,
            start_time: binding::fmi3Float64,
            stop_time_defined: binding::fmi3Boolean,
            stop_time: binding::fmi3Float64,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty, Common);
            let tolerance = tolerance_defined.then_some(tolerance);
            let stop_time = stop_time_defined.then_some(stop_time);
            instance
                .enter_initialization_mode(tolerance, start_time, stop_time)
                .into()
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3ExitInitializationMode(
            instance: binding::fmi3Instance,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty, Common);
            instance.exit_initialization_mode().into()
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3EnterEventMode(
            instance: binding::fmi3Instance,
        ) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty, Common);
            instance.enter_event_mode().into()
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3Terminate(instance: binding::fmi3Instance) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty, Common);
            instance.terminate().into()
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn fmi3Reset(instance: binding::fmi3Instance) -> binding::fmi3Status {
            let instance = checked_deref!(instance, $ty, Common);
            instance.reset().into()
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe fn fmi3GetFloat64(
            instance: binding::fmi3Instance,
            value_references: *const binding::fmi3ValueReference,
            n_value_references: usize,
            values: *mut binding::fmi3Float64,
            n_values: usize,
        ) -> binding::fmi3Status {
            let instance = crate::checked_deref!(instance, $ty, Common);
            let value_refs = std::slice::from_raw_parts(value_references, n_value_references);
            let values = std::slice::from_raw_parts_mut(values, n_values);
            ::fmi::fmi3::Common::get_float64(instance, value_refs, values).into()
        }
    };
}
