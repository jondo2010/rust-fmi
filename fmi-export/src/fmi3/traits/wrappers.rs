//! Traits that implement safe wrappers around the C-typed APIs
use crate::checked_deref;

use super::Model;

use fmi::fmi3::{Fmi3Res, binding};

/// Safely dereferences an FMI instance pointer and validates it.
#[macro_export]
macro_rules! checked_deref {
    ($ptr:expr, $ty:ty) => {{
        if ($ptr as *mut ::std::os::raw::c_void).is_null() {
            eprintln!("Invalid FMU instance");
            return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
        }
        let instance = unsafe { &mut *($ptr as *mut $crate::fmi3::ModelInstance<$ty>) };
        instance
    }};
}

#[macro_export]
macro_rules! wrapper_getset_functions {
    ($type_name:ident, $fmi_type:ty, $get_method:ident, $set_method:ident) => {
        $crate::paste::paste! {
            unsafe extern "C" fn [<fmi3_get_ $type_name:snake>](
                instance: binding::fmi3Instance,
                value_references: *const binding::fmi3ValueReference,
                n_value_references: usize,
                values: *mut $fmi_type,
                n_values: usize,
            ) -> binding::fmi3Status {
                let instance = $crate::checked_deref!(instance, Self);

                // Validate array lengths match
                if n_value_references != n_values {
                    eprintln!("FMI3: Array length mismatch in fmi3Get{}: value_references={}, values={}",
                             stringify!($type_name), n_value_references, n_values);
                    return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
                }

                let value_refs = unsafe { std::slice::from_raw_parts(value_references, n_value_references) };
                let values = unsafe { std::slice::from_raw_parts_mut(values, n_values) };

                match <$crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::GetSet>::$get_method(
                    instance, value_refs, values
                ) {
                    Ok(res) => {
                        let status: ::fmi::fmi3::Fmi3Status = res.into();
                        status.into()
                    }
                    Err(_) => binding::fmi3Status_fmi3Error,
                }
            }

            unsafe extern "C" fn [<fmi3_set_ $type_name:snake>](
                instance: binding::fmi3Instance,
                value_references: *const binding::fmi3ValueReference,
                n_value_references: usize,
                values: *const $fmi_type,
                n_values: usize,
            ) -> binding::fmi3Status {
                let instance = $crate::checked_deref!(instance, Self);

                // Validate array lengths match
                if n_value_references != n_values {
                    eprintln!("FMI3: Array length mismatch in fmi3Set{}: value_references={}, values={}",
                             stringify!($type_name), n_value_references, n_values);
                    return binding::fmi3Status_fmi3Error;
                }

                let value_refs = unsafe { std::slice::from_raw_parts(value_references, n_value_references) };
                let values = unsafe { std::slice::from_raw_parts(values, n_values) };

                match <$crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::GetSet>::$set_method(
                    instance, value_refs, values
                ) {
                    Ok(res) => {
                        let status: ::fmi::fmi3::Fmi3Status = res.into();
                        status.into()
                    }
                    Err(_) => binding::fmi3Status_fmi3Error,
                }
            }
        }
    };
}

pub trait Fmi3Common: Model + Sized {
    #[inline(always)]
    unsafe fn fmi3_get_version() -> *const ::std::os::raw::c_char {
        binding::fmi3Version.as_ptr() as *const _
    }

    #[inline(always)]
    unsafe fn fmi3_set_debug_logging(
        instance: binding::fmi3Instance,
        logging_on: binding::fmi3Boolean,
        n_categories: usize,
        categories: *const binding::fmi3String,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        let categories = unsafe { std::slice::from_raw_parts(categories, n_categories) }
            .into_iter()
            .filter_map(|cat| unsafe { std::ffi::CStr::from_ptr(*cat) }.to_str().ok())
            .collect::<::std::vec::Vec<_>>();
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::Common>::set_debug_logging(
            instance,
            logging_on,
            &categories,
        ) {
            Ok(res) => {
                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe extern "C" fn fmi3_instantiate_model_exchange(
        instance_name: binding::fmi3String,
        instantiation_token: binding::fmi3String,
        resource_path: binding::fmi3String,
        _visible: binding::fmi3Boolean,
        logging_on: binding::fmi3Boolean,
        _instance_environment: binding::fmi3InstanceEnvironment,
        log_message: binding::fmi3LogMessageCallback,
    ) -> binding::fmi3Instance {
        let name = unsafe { ::std::ffi::CStr::from_ptr(instance_name) }
            .to_string_lossy()
            .into_owned();
        let token = unsafe { ::std::ffi::CStr::from_ptr(instantiation_token) }.to_string_lossy();
        let resource_path = ::std::path::PathBuf::from(
            unsafe { ::std::ffi::CStr::from_ptr(resource_path) }
                .to_string_lossy()
                .into_owned(),
        );

        if let Some(log_message) = log_message {
            // Wrap the C callback in a Rust closure
            let log_message = Box::new(
                move |status: ::fmi::fmi3::Fmi3Status,
                      category: &str,
                      args: std::fmt::Arguments<'_>| {
                    let category_c = ::std::ffi::CString::new(category).unwrap_or_default();
                    let message_c = ::std::ffi::CString::new(args.to_string()).unwrap_or_default();
                    unsafe {
                        log_message(
                            std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                            status.into(),
                            category_c.as_ptr(),
                            message_c.as_ptr(),
                        )
                    };
                },
            );

            match crate::fmi3::ModelInstance::<Self>::new(
                name,
                resource_path,
                logging_on,
                log_message,
                &token,
            ) {
                Ok(instance) => {
                    let this: ::std::boxed::Box<dyn ::fmi::fmi3::Common> =
                        ::std::boxed::Box::new(instance);
                    ::std::boxed::Box::into_raw(this) as binding::fmi3Instance
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

    #[inline(always)]
    unsafe extern "C" fn fmi3_instantiate_co_simulation(
        instance_name: binding::fmi3String,
        instantiation_token: binding::fmi3String,
        resource_path: binding::fmi3String,
        visible: binding::fmi3Boolean,
        logging_on: binding::fmi3Boolean,
        event_mode_used: binding::fmi3Boolean,
        early_return_allowed: binding::fmi3Boolean,
        required_intermediate_variables: *const binding::fmi3ValueReference,
        n_required_intermediate_variables: usize,
        instance_environment: binding::fmi3InstanceEnvironment,
        log_message: binding::fmi3LogMessageCallback,
        intermediate_update: binding::fmi3IntermediateUpdateCallback,
    ) -> binding::fmi3Instance {
        let name = unsafe { ::std::ffi::CStr::from_ptr(instance_name) }
            .to_string_lossy()
            .into_owned();
        let token = unsafe { ::std::ffi::CStr::from_ptr(instantiation_token) }.to_string_lossy();
        let resource_path = ::std::path::PathBuf::from(
            unsafe { ::std::ffi::CStr::from_ptr(resource_path) }
                .to_string_lossy()
                .into_owned(),
        );

        panic!("Co-Simulation not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_instantiate_scheduled_execution(
        instance_name: binding::fmi3String,
        instantiation_token: binding::fmi3String,
        resource_path: binding::fmi3String,
        visible: binding::fmi3Boolean,
        logging_on: binding::fmi3Boolean,
        instance_environment: binding::fmi3InstanceEnvironment,
        log_message: binding::fmi3LogMessageCallback,
        clock_update: binding::fmi3ClockUpdateCallback,
        lock_preemption: binding::fmi3LockPreemptionCallback,
        unlock_preemption: binding::fmi3UnlockPreemptionCallback,
    ) -> binding::fmi3Instance {
        let name = unsafe { ::std::ffi::CStr::from_ptr(instance_name) }
            .to_string_lossy()
            .into_owned();
        let token = unsafe { ::std::ffi::CStr::from_ptr(instantiation_token) }.to_string_lossy();
        let resource_path = ::std::path::PathBuf::from(
            unsafe { ::std::ffi::CStr::from_ptr(resource_path) }
                .to_string_lossy()
                .into_owned(),
        );

        todo!("Scheduled-Execution not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_free_instance(instance: binding::fmi3Instance) {
        if instance.is_null() {
            eprintln!("Invalid FMU instance");
            return;
        }
        let _this = unsafe {
            ::std::boxed::Box::from_raw(instance as *mut crate::fmi3::ModelInstance<Self>)
        };
        _this.context().log(
            Fmi3Res::OK,
            Default::default(),
            format_args!("{}: fmi3FreeInstance()", _this.instance_name()),
        );
        // instance will be dropped here, freeing resources
    }

    #[inline(always)]
    unsafe fn fmi3_enter_initialization_mode(
        instance: binding::fmi3Instance,
        tolerance_defined: binding::fmi3Boolean,
        tolerance: binding::fmi3Float64,
        start_time: binding::fmi3Float64,
        stop_time_defined: binding::fmi3Boolean,
        stop_time: binding::fmi3Float64,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        let tolerance = tolerance_defined.then_some(tolerance);
        let stop_time = stop_time_defined.then_some(stop_time);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::Common>::enter_initialization_mode(
            instance, tolerance, start_time, stop_time,
        ) {
            Ok(res) => {
                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_exit_initialization_mode(
        instance: binding::fmi3Instance,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::Common>::exit_initialization_mode(
            instance,
        ) {
            Ok(res) => {
                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_enter_event_mode(instance: binding::fmi3Instance) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::Common>::enter_event_mode(instance)
        {
            Ok(res) => {
                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_terminate(instance: binding::fmi3Instance) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::Common>::terminate(instance) {
            Ok(res) => {
                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_reset(instance: binding::fmi3Instance) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::Common>::reset(instance) {
            Ok(res) => {
                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    // FMU State functions
    #[inline(always)]
    unsafe fn fmi3_get_fmu_state(
        _instance: binding::fmi3Instance,
        _fmu_state: *mut binding::fmi3FMUState,
    ) -> binding::fmi3Status {
        todo!("FMU state not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_set_fmu_state(
        _instance: binding::fmi3Instance,
        _fmu_state: binding::fmi3FMUState,
    ) -> binding::fmi3Status {
        todo!("FMU state not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_free_fmu_state(
        _instance: binding::fmi3Instance,
        _fmu_state: *mut binding::fmi3FMUState,
    ) -> binding::fmi3Status {
        todo!("FMU state not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_serialized_fmu_state_size(
        _instance: binding::fmi3Instance,
        _fmu_state: binding::fmi3FMUState,
        _size: *mut usize,
    ) -> binding::fmi3Status {
        todo!("FMU state not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_serialize_fmu_state(
        _instance: binding::fmi3Instance,
        _fmu_state: binding::fmi3FMUState,
        _serialized_state: *mut binding::fmi3Byte,
        _size: usize,
    ) -> binding::fmi3Status {
        todo!("FMU state not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_deserialize_fmu_state(
        _instance: binding::fmi3Instance,
        _serialized_state: *const binding::fmi3Byte,
        _size: usize,
        _fmu_state: *mut binding::fmi3FMUState,
    ) -> binding::fmi3Status {
        todo!("FMU state not yet implemented");
    }

    // Derivative functions
    #[inline(always)]
    unsafe fn fmi3_get_directional_derivative(
        instance: binding::fmi3Instance,
        _unknowns: *const binding::fmi3ValueReference,
        _n_unknowns: usize,
        _knowns: *const binding::fmi3ValueReference,
        _n_knowns: usize,
        _seed: *const binding::fmi3Float64,
        _n_seed: usize,
        _sensitivity: *mut binding::fmi3Float64,
        _n_sensitivity: usize,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Directional derivative not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_get_adjoint_derivative(
        instance: binding::fmi3Instance,
        _unknowns: *const binding::fmi3ValueReference,
        _n_unknowns: usize,
        _knowns: *const binding::fmi3ValueReference,
        _n_knowns: usize,
        _seed: *const binding::fmi3Float64,
        _n_seed: usize,
        _sensitivity: *mut binding::fmi3Float64,
        _n_sensitivity: usize,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Adjoint derivative not yet implemented");
    }

    // Configuration mode functions
    #[inline(always)]
    unsafe fn fmi3_enter_configuration_mode(
        instance: binding::fmi3Instance,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::Common>::enter_configuration_mode(
            instance,
        ) {
            Ok(res) => {
                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_exit_configuration_mode(instance: binding::fmi3Instance) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::Common>::exit_configuration_mode(
            instance,
        ) {
            Ok(res) => {
                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    // Clock related functions
    #[inline(always)]
    unsafe fn fmi3_get_interval_decimal(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _intervals: *mut binding::fmi3Float64,
        _qualifiers: *mut binding::fmi3IntervalQualifier,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_get_interval_fraction(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _counters: *mut binding::fmi3UInt64,
        _resolutions: *mut binding::fmi3UInt64,
        _qualifiers: *mut binding::fmi3IntervalQualifier,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_get_shift_decimal(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _shifts: *mut binding::fmi3Float64,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_get_shift_fraction(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _counters: *mut binding::fmi3UInt64,
        _resolutions: *mut binding::fmi3UInt64,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_set_interval_decimal(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _intervals: *const binding::fmi3Float64,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_set_interval_fraction(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _counters: *const binding::fmi3UInt64,
        _resolutions: *const binding::fmi3UInt64,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_set_shift_decimal(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _shifts: *const binding::fmi3Float64,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_set_shift_fraction(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _counters: *const binding::fmi3UInt64,
        _resolutions: *const binding::fmi3UInt64,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_evaluate_discrete_states(
        instance: binding::fmi3Instance,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Discrete states not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_update_discrete_states(
        instance: binding::fmi3Instance,
        discrete_states_need_update: *mut binding::fmi3Boolean,
        terminate_simulation: *mut binding::fmi3Boolean,
        nominals_of_continuous_states_changed: *mut binding::fmi3Boolean,
        values_of_continuous_states_changed: *mut binding::fmi3Boolean,
        next_event_time_defined: *mut binding::fmi3Boolean,
        next_event_time: *mut binding::fmi3Float64,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        let mut event_flags = ::fmi::EventFlags::default();

        // next_time_event is potentially used as an in-out parameter
        if unsafe { *next_event_time_defined } {
            event_flags.next_event_time = Some(unsafe { *next_event_time });
        }

        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::Common>::update_discrete_states(
            instance,
            &mut event_flags,
        ) {
            Ok(res) => {
                unsafe {
                    *discrete_states_need_update = event_flags.discrete_states_need_update;
                    *terminate_simulation = event_flags.terminate_simulation;
                    *nominals_of_continuous_states_changed =
                        event_flags.nominals_of_continuous_states_changed;
                    *values_of_continuous_states_changed =
                        event_flags.values_of_continuous_states_changed;

                    if let Some(event_time) = event_flags.next_event_time {
                        *next_event_time_defined = true;
                        *next_event_time = event_time;
                    } else {
                        *next_event_time_defined = false;
                        *next_event_time = 0.0;
                    }
                }

                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    wrapper_getset_functions!(Float64, binding::fmi3Float64, get_float64, set_float64);
    wrapper_getset_functions!(Float32, binding::fmi3Float32, get_float32, set_float32);
    wrapper_getset_functions!(Int64, binding::fmi3Int64, get_int64, set_int64);
    wrapper_getset_functions!(Int32, binding::fmi3Int32, get_int32, set_int32);
    wrapper_getset_functions!(Int16, binding::fmi3Int16, get_int16, set_int16);
    wrapper_getset_functions!(Int8, binding::fmi3Int8, get_int8, set_int8);
    wrapper_getset_functions!(UInt64, binding::fmi3UInt64, get_uint64, set_uint64);
    wrapper_getset_functions!(UInt32, binding::fmi3UInt32, get_uint32, set_uint32);
    wrapper_getset_functions!(UInt16, binding::fmi3UInt16, get_uint16, set_uint16);
    wrapper_getset_functions!(UInt8, binding::fmi3UInt8, get_uint8, set_uint8);
    wrapper_getset_functions!(Boolean, binding::fmi3Boolean, get_boolean, set_boolean);

    // String functions
    #[inline(always)]
    unsafe fn fmi3_get_string(
        instance: binding::fmi3Instance,
        value_references: *const binding::fmi3ValueReference,
        n_value_references: usize,
        values: *mut binding::fmi3String,
        n_values: usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);

        if n_value_references != n_values {
            eprintln!(
                "FMI3: Array length mismatch in fmi3GetString: value_references={}, values={}",
                n_value_references, n_values
            );
            return binding::fmi3Status_fmi3Error;
        }

        let value_refs =
            unsafe { ::std::slice::from_raw_parts(value_references, n_value_references) };

        // Create temporary buffer for CString results
        let mut temp_strings = vec![::std::ffi::CString::default(); n_values];

        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::GetSet>::get_string(
            instance,
            value_refs,
            &mut temp_strings,
        ) {
            Ok(_) => {
                // Copy C string pointers to output array
                let values_slice = unsafe { ::std::slice::from_raw_parts_mut(values, n_values) };
                for (i, cstring) in temp_strings.iter().enumerate() {
                    values_slice[i] = cstring.as_ptr();
                }
                binding::fmi3Status_fmi3OK
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_set_string(
        instance: binding::fmi3Instance,
        value_references: *const binding::fmi3ValueReference,
        n_value_references: usize,
        values: *const binding::fmi3String,
        n_values: usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);

        if n_value_references != n_values {
            eprintln!(
                "FMI3: Array length mismatch in fmi3SetString: value_references={}, values={}",
                n_value_references, n_values
            );
            return binding::fmi3Status_fmi3Error;
        }

        let value_refs =
            unsafe { ::std::slice::from_raw_parts(value_references, n_value_references) };
        let string_ptrs = unsafe { ::std::slice::from_raw_parts(values, n_values) };

        // Convert C strings to CString objects
        let mut temp_strings = Vec::with_capacity(n_values);
        for &ptr in string_ptrs {
            if ptr.is_null() {
                temp_strings.push(::std::ffi::CString::default());
            } else {
                let cstring = unsafe { ::std::ffi::CStr::from_ptr(ptr) }.to_owned();
                temp_strings.push(cstring);
            }
        }

        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::GetSet>::set_string(
            instance,
            value_refs,
            &temp_strings,
        ) {
            Ok(_) => binding::fmi3Status_fmi3OK,
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    // Binary functions
    #[inline(always)]
    unsafe fn fmi3_get_binary(
        instance: binding::fmi3Instance,
        value_references: *const binding::fmi3ValueReference,
        n_value_references: usize,
        value_sizes: *mut usize,
        values: *mut *mut binding::fmi3Byte,
        n_values: usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);

        if n_value_references != n_values {
            eprintln!(
                "FMI3: Array length mismatch in fmi3GetBinary: value_references={}, values={}",
                n_value_references, n_values
            );
            return binding::fmi3Status_fmi3Error;
        }

        let value_refs =
            unsafe { ::std::slice::from_raw_parts(value_references, n_value_references) };
        let sizes_slice = unsafe { ::std::slice::from_raw_parts_mut(value_sizes, n_values) };
        let values_slice = unsafe { ::std::slice::from_raw_parts_mut(values, n_values) };

        // Create temporary buffers for binary data
        let mut temp_buffers: Vec<&mut [u8]> = Vec::with_capacity(n_values);
        for i in 0..n_values {
            if values_slice[i].is_null() || sizes_slice[i] == 0 {
                temp_buffers.push(&mut []);
            } else {
                let buffer =
                    unsafe { ::std::slice::from_raw_parts_mut(values_slice[i], sizes_slice[i]) };
                temp_buffers.push(buffer);
            }
        }

        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::GetSet>::get_binary(
            instance,
            value_refs,
            &mut temp_buffers,
        ) {
            Ok(actual_sizes) => {
                // Update the actual sizes
                for (i, &size) in actual_sizes.iter().enumerate() {
                    sizes_slice[i] = size;
                }
                binding::fmi3Status_fmi3OK
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_set_binary(
        instance: binding::fmi3Instance,
        value_references: *const binding::fmi3ValueReference,
        n_value_references: usize,
        value_sizes: *const usize,
        values: *const *const binding::fmi3Byte,
        n_values: usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);

        if n_value_references != n_values {
            eprintln!(
                "FMI3: Array length mismatch in fmi3SetBinary: value_references={}, values={}",
                n_value_references, n_values
            );
            return binding::fmi3Status_fmi3Error;
        }

        let value_refs =
            unsafe { ::std::slice::from_raw_parts(value_references, n_value_references) };
        let sizes_slice = unsafe { ::std::slice::from_raw_parts(value_sizes, n_values) };
        let values_slice = unsafe { ::std::slice::from_raw_parts(values, n_values) };

        // Create temporary slices for binary data
        let mut temp_buffers: Vec<&[u8]> = Vec::with_capacity(n_values);
        for i in 0..n_values {
            if values_slice[i].is_null() || sizes_slice[i] == 0 {
                temp_buffers.push(&[]);
            } else {
                let buffer =
                    unsafe { ::std::slice::from_raw_parts(values_slice[i], sizes_slice[i]) };
                temp_buffers.push(buffer);
            }
        }

        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::GetSet>::set_binary(
            instance,
            value_refs,
            &temp_buffers,
        ) {
            Ok(_) => binding::fmi3Status_fmi3OK,
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    // Clock functions
    #[inline(always)]
    unsafe fn fmi3_get_clock(
        instance: binding::fmi3Instance,
        value_references: *const binding::fmi3ValueReference,
        n_value_references: usize,
        values: *mut binding::fmi3Clock,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        let value_refs =
            unsafe { ::std::slice::from_raw_parts(value_references, n_value_references) };
        let values_slice = unsafe { ::std::slice::from_raw_parts_mut(values, n_value_references) };
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::GetSet>::get_clock(
            instance,
            value_refs,
            values_slice,
        ) {
            Ok(res) => {
                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_set_clock(
        instance: binding::fmi3Instance,
        value_references: *const binding::fmi3ValueReference,
        n_value_references: usize,
        values: *const binding::fmi3Clock,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        let value_refs =
            unsafe { ::std::slice::from_raw_parts(value_references, n_value_references) };
        let values_slice = unsafe { ::std::slice::from_raw_parts(values, n_value_references) };
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::GetSet>::set_clock(
            instance,
            value_refs,
            values_slice,
        ) {
            Ok(res) => {
                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    // Variable Dependency functions
    #[inline(always)]
    unsafe fn fmi3_get_number_of_variable_dependencies(
        instance: binding::fmi3Instance,
        value_reference: binding::fmi3ValueReference,
        n_dependencies: *mut usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::Common>::get_number_of_variable_dependencies(instance, value_reference) {
        Ok(res) => {
            unsafe { *n_dependencies = res; }
            binding::fmi3Status_fmi3OK
        }
        Err(_) => binding::fmi3Status_fmi3Error,
    }
    }

    #[inline(always)]
    unsafe fn fmi3_get_variable_dependencies(
        instance: binding::fmi3Instance,
        dependent: binding::fmi3ValueReference,
        element_indices_of_dependent: *mut usize,
        independents: *mut binding::fmi3ValueReference,
        element_indices_of_independents: *mut usize,
        dependency_kinds: *mut binding::fmi3DependencyKind,
        n_dependencies: usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);

        // Convert the value reference to our trait's ValueRef type
        let dependent_vr = dependent.into();

        // Call the Rust method to get dependencies
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::Common>::get_variable_dependencies(
            instance,
            dependent_vr,
        ) {
            Ok(dependencies) => {
                // Check if the caller provided enough space
                if dependencies.len() > n_dependencies {
                    eprintln!(
                        "Buffer too small: {} dependencies returned but only {} allocated",
                        dependencies.len(),
                        n_dependencies
                    );
                    return binding::fmi3Status_fmi3Error;
                }

                // Copy dependency data to the C arrays
                for (i, dep) in dependencies.iter().enumerate() {
                    if i >= n_dependencies {
                        break; // Safety check
                    }

                    unsafe {
                        // Set element index of dependent
                        *element_indices_of_dependent.add(i) = dep.dependent_element_index;

                        // Set independent value reference
                        *independents.add(i) = dep.independent.into();

                        // Set element index of independent
                        *element_indices_of_independents.add(i) = dep.independent_element_index;

                        // Set dependency kind
                        *dependency_kinds.add(i) = dep.dependency_kind;
                    }
                }

                binding::fmi3Status_fmi3OK
            }
            Err(e) => {
                eprintln!("Failed to get variable dependencies: {:?}", e);
                ::fmi::fmi3::Fmi3Status::from(e).into()
            }
        }
    }
}

// Model Exchange trait
pub trait Fmi3ModelExchange: Fmi3Common {
    #[inline(always)]
    unsafe fn fmi3_enter_continuous_time_mode(
        instance: binding::fmi3Instance,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::ModelExchange>::enter_continuous_time_mode(instance) {
        Ok(res) => {
            let status: ::fmi::fmi3::Fmi3Status = res.into();
            status.into()
        }
        Err(_) => binding::fmi3Status_fmi3Error,
    }
    }

    #[inline(always)]
    unsafe fn fmi3_completed_integrator_step(
        instance: binding::fmi3Instance,
        no_set_fmu_state_prior: binding::fmi3Boolean,
        enter_event_mode: *mut binding::fmi3Boolean,
        terminate_simulation: *mut binding::fmi3Boolean,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        let mut enter_event = false;
        let mut terminate = false;
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::ModelExchange>::completed_integrator_step(
        instance,
        no_set_fmu_state_prior,
        &mut enter_event,
        &mut terminate,
    ) {
        Ok(_) => {
            unsafe {
                *enter_event_mode = enter_event;
                *terminate_simulation = terminate;
            }
            binding::fmi3Status_fmi3OK
        }
        Err(_) => binding::fmi3Status_fmi3Error,
    }
    }

    #[inline(always)]
    unsafe fn fmi3_set_time(
        instance: binding::fmi3Instance,
        time: binding::fmi3Float64,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::ModelExchange>::set_time(
            instance, time,
        ) {
            Ok(res) => {
                let status: ::fmi::fmi3::Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_set_continuous_states(
        instance: binding::fmi3Instance,
        continuous_states: *const binding::fmi3Float64,
        n_continuous_states: usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        let states =
            unsafe { ::std::slice::from_raw_parts(continuous_states, n_continuous_states) };
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::ModelExchange>::set_continuous_states(instance, states) {
        Ok(res) => {
            let status: ::fmi::fmi3::Fmi3Status = res.into();
            status.into()
        }
        Err(_) => binding::fmi3Status_fmi3Error,
    }
    }

    #[inline(always)]
    unsafe fn fmi3_get_continuous_state_derivatives(
        instance: binding::fmi3Instance,
        derivatives: *mut binding::fmi3Float64,
        n_continuous_states: usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        let derivs = unsafe { ::std::slice::from_raw_parts_mut(derivatives, n_continuous_states) };
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::ModelExchange>::get_continuous_state_derivatives(instance, derivs) {
        Ok(res) => {
            let status: ::fmi::fmi3::Fmi3Status = res.into();
            status.into()
        }
        Err(_) => binding::fmi3Status_fmi3Error,
    }
    }

    #[inline(always)]
    unsafe fn fmi3_get_event_indicators(
        instance: binding::fmi3Instance,
        event_indicators: *mut binding::fmi3Float64,
        n_event_indicators: usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        let indicators =
            unsafe { ::std::slice::from_raw_parts_mut(event_indicators, n_event_indicators) };
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::ModelExchange>::get_event_indicators(
            instance, indicators,
        ) {
            Ok(_) => binding::fmi3Status_fmi3OK,
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_get_continuous_states(
        instance: binding::fmi3Instance,
        continuous_states: *mut binding::fmi3Float64,
        n_continuous_states: usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        let states =
            unsafe { ::std::slice::from_raw_parts_mut(continuous_states, n_continuous_states) };
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::ModelExchange>::get_continuous_states(instance, states) {
        Ok(res) => {
            let status: ::fmi::fmi3::Fmi3Status = res.into();
            status.into()
        }
        Err(_) => binding::fmi3Status_fmi3Error,
    }
    }

    #[inline(always)]
    unsafe fn fmi3_get_nominals_of_continuous_states(
        instance: binding::fmi3Instance,
        nominals: *mut binding::fmi3Float64,
        n_continuous_states: usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        let nominals = unsafe { ::std::slice::from_raw_parts_mut(nominals, n_continuous_states) };
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::ModelExchange>::get_nominals_of_continuous_states(instance, nominals) {
        Ok(res) => {
            let status: ::fmi::fmi3::Fmi3Status = res.into();
            status.into()
        }
        Err(_) => binding::fmi3Status_fmi3Error,
    }
    }

    #[inline(always)]
    unsafe fn fmi3_get_number_of_event_indicators(
        instance: binding::fmi3Instance,
        n_event_indicators: *mut usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::ModelExchange>::get_number_of_event_indicators(instance) {
        Ok(n) => {
            unsafe { *n_event_indicators = n; }
            binding::fmi3Status_fmi3OK
        }
        Err(_) => binding::fmi3Status_fmi3Error,
    }
    }

    #[inline(always)]
    unsafe fn fmi3_get_number_of_continuous_states(
        instance: binding::fmi3Instance,
        n_continuous_states: *mut usize,
    ) -> binding::fmi3Status {
        let instance = checked_deref!(instance, Self);
        match <crate::fmi3::ModelInstance<Self> as ::fmi::fmi3::ModelExchange>::get_number_of_continuous_states(instance) {
        Ok(n) => {
            unsafe { *n_continuous_states = n; }
            binding::fmi3Status_fmi3OK
        }
        Err(_) => binding::fmi3Status_fmi3Error,
    }
    }
}

// Co-Simulation trait
pub trait Fmi3CoSimulation: Fmi3Common {
    #[inline(always)]
    unsafe fn fmi3_enter_step_mode(instance: binding::fmi3Instance) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Co-Simulation not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_get_output_derivatives(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _orders: *const binding::fmi3Int32,
        _values: *mut binding::fmi3Float64,
        _n_values: usize,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Co-Simulation not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_do_step(
        instance: binding::fmi3Instance,
        _current_communication_point: binding::fmi3Float64,
        _communication_step_size: binding::fmi3Float64,
        _no_set_fmu_state_prior_to_current_point: binding::fmi3Boolean,
        _event_handling_needed: *mut binding::fmi3Boolean,
        _terminate_simulation: *mut binding::fmi3Boolean,
        _early_return: *mut binding::fmi3Boolean,
        _last_successful_time: *mut binding::fmi3Float64,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Co-Simulation not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_activate_model_partition(
        instance: binding::fmi3Instance,
        _clock_reference: binding::fmi3ValueReference,
        _activation_time: binding::fmi3Float64,
    ) -> binding::fmi3Status {
        let _instance = checked_deref!(instance, Self);
        todo!("Co-Simulation not yet implemented");
    }
}

// Automatic implementations for all models
impl<T> Fmi3Common for T where T: Model {}
impl<T> Fmi3ModelExchange for T where T: Model + Fmi3Common {}
impl<T> Fmi3CoSimulation for T where T: Model + Fmi3Common {}
