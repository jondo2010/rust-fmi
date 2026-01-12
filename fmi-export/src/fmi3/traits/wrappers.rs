//! Traits that implement safe wrappers around the C-typed APIs
//!
//! # Notes:
//!
//! 1. Exported C-ABI functions delegate directly to these trait functions.
//!   - the entire API must invariably be available through these traits.
//! 2. The instantiation functions:
//!  - must fail here if the model doesn't "support" the requested interface
use ::std::ffi::CString;

use crate::{
    checked_deref_cs, checked_deref_me, dispatch_by_instance_type,
    fmi3::{
        ModelGetSetStates, ModelInstance, UserModel,
        instance::{
            LogMessageClosure,
            context::BasicContext,
        },
        traits::{ModelGetSet, UserModelCS, UserModelME},
    },
};

use super::{Context, Model};

use fmi::fmi3::CoSimulation;

use ::fmi::fmi3::Fmi3Status;
use fmi::fmi3::{Common, Fmi3Res, GetSet, ModelExchange, ScheduledExecution, binding};

/// Safely dereferences an FMI instance pointer for Model Exchange instances.
#[macro_export]
macro_rules! checked_deref_me {
    ($ptr:expr, $ty:ty) => {{
        if ($ptr as *mut ::std::os::raw::c_void).is_null() {
            eprintln!("Invalid FMU instance");
            return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
        }
        let instance = unsafe {
            &mut *($ptr as *mut $crate::fmi3::ModelInstance<
                $ty,
                $crate::fmi3::instance::context::BasicContext<$ty>,
            >)
        };
        instance
    }};
}

/// Safely dereferences an FMI instance pointer for Co-Simulation instances with wrapper context.
#[macro_export]
macro_rules! checked_deref_cs {
    ($ptr:expr, $ty:ty) => {{
        if ($ptr as *mut ::std::os::raw::c_void).is_null() {
            eprintln!("Invalid FMU instance");
            return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
        }
        let instance = unsafe {
            &mut *($ptr as *mut $crate::fmi3::ModelInstance<
                $ty,
                $crate::fmi3::instance::context::BasicContext<$ty>,
            >)
        };
        instance
    }};
}

/// Dispatches a method call based on the runtime instance_type.
/// This is used for Common trait methods that must work for any instance type (ME/CS/SE).
#[macro_export]
macro_rules! dispatch_by_instance_type {
    ($ptr:expr, $ty:ty, $method:ident $(, $arg:expr)*) => {{
        if ($ptr as *mut ::std::os::raw::c_void).is_null() {
            eprintln!("Invalid FMU instance");
            return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
        }

        // Read instance_type field directly to determine which concrete type to use
        // Safety: instance_type is the first field for all ModelInstance<M, C> types
        let instance_type = unsafe {
            let temp = $ptr as *const $crate::fmi3::ModelInstance<$ty, $crate::fmi3::instance::context::BasicContext<$ty>>;
            // Read the first field directly instead of calling a method
            (*temp).instance_type
        };

        match instance_type {
            fmi::InterfaceType::ModelExchange => {
                let instance = unsafe {
                    &mut *($ptr as *mut $crate::fmi3::ModelInstance<$ty, $crate::fmi3::instance::context::BasicContext<$ty>>)
                };
                instance.$method($($arg),*)
            }
            fmi::InterfaceType::CoSimulation => {
                let instance = unsafe {
                    &mut *($ptr as *mut $crate::fmi3::ModelInstance<$ty, $crate::fmi3::instance::context::BasicContext<$ty>>)
                };
                instance.$method($($arg),*)
            }
            fmi::InterfaceType::ScheduledExecution => {
                let instance = unsafe {
                    &mut *($ptr as *mut $crate::fmi3::ModelInstance<$ty, $crate::fmi3::instance::context::BasicContext<$ty>>)
                };
                instance.$method($($arg),*)
            }
        }
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
                // Validate array lengths match
                if n_value_references != n_values {
                    eprintln!("FMI3: Array length mismatch in fmi3Get{}: value_references={}, values={}",
                             stringify!($type_name), n_value_references, n_values);
                    return ::fmi::fmi3::binding::fmi3Status_fmi3Error;
                }

                let value_refs = unsafe { std::slice::from_raw_parts(value_references, n_value_references) };
                let values = unsafe { std::slice::from_raw_parts_mut(values, n_values) };

                match $crate::dispatch_by_instance_type!(instance, Self, $get_method, value_refs, values) {
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
                // Validate array lengths match
                if n_value_references != n_values {
                    eprintln!("FMI3: Array length mismatch in fmi3Set{}: value_references={}, values={}",
                             stringify!($type_name), n_value_references, n_values);
                    return binding::fmi3Status_fmi3Error;
                }

                let value_refs = unsafe { std::slice::from_raw_parts(value_references, n_value_references) };
                let values = unsafe { std::slice::from_raw_parts(values, n_values) };

                match $crate::dispatch_by_instance_type!(instance, Self, $set_method, value_refs, values) {
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

pub trait Fmi3Common: Model + UserModel + ModelGetSet<Self> + ModelGetSetStates + Sized
where
    Self: 'static,
{
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
        let categories = unsafe { std::slice::from_raw_parts(categories, n_categories) }
            .into_iter()
            .filter_map(|cat| unsafe { std::ffi::CStr::from_ptr(*cat) }.to_str().ok())
            .collect::<::std::vec::Vec<_>>();
        match dispatch_by_instance_type!(instance, Self, set_debug_logging, logging_on, &categories)
        {
            Ok(res) => {
                let status: Fmi3Status = res.into();
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

        // Wrap the C callback in a Rust closure
        let log_message: LogMessageClosure = if let Some(cb) = log_message {
            Box::new(
                move |status: Fmi3Status, category: &str, args: std::fmt::Arguments<'_>| {
                    let category_c = CString::new(category).unwrap_or_default();
                    let message_c = CString::new(args.to_string()).unwrap_or_default();
                    unsafe {
                        cb(
                            std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                            status.into(),
                            category_c.as_ptr(),
                            message_c.as_ptr(),
                        )
                    };
                },
            )
        } else {
            Box::new(
                move |status: Fmi3Status, category: &str, args: std::fmt::Arguments<'_>| {
                    let category_c = CString::new(category).unwrap_or_default();
                    let message_c = CString::new(args.to_string()).unwrap_or_default();
                    eprintln!(
                        "Log (status: {:?}, category: {}): {}",
                        status,
                        category_c.to_string_lossy(),
                        message_c.to_string_lossy()
                    );
                },
            )
        };

        if !Self::SUPPORTS_MODEL_EXCHANGE {
            eprintln!("Model Exchange not supported by this FMU");
            return ::std::ptr::null_mut();
        }

        let context = BasicContext::new(logging_on, log_message, resource_path, false);

        match crate::fmi3::ModelInstance::<Self, BasicContext<Self>>::new(
            name,
            &token,
            context,
            fmi::InterfaceType::ModelExchange,
        ) {
            Ok(instance) => ::std::boxed::Box::into_raw(::std::boxed::Box::new(instance))
                as binding::fmi3Instance,
            Err(_) => {
                eprintln!("Failed to instantiate FMU: invalid instantiation token");
                ::std::ptr::null_mut()
            }
        }
    }

    #[inline(always)]
    unsafe extern "C" fn fmi3_instantiate_co_simulation(
        instance_name: binding::fmi3String,
        instantiation_token: binding::fmi3String,
        resource_path: binding::fmi3String,
        _visible: binding::fmi3Boolean,
        _logging_on: binding::fmi3Boolean,
        _event_mode_used: binding::fmi3Boolean,
        _early_return_allowed: binding::fmi3Boolean,
        _required_intermediate_variables: *const binding::fmi3ValueReference,
        _n_required_intermediate_variables: usize,
        _instance_environment: binding::fmi3InstanceEnvironment,
        _log_message: binding::fmi3LogMessageCallback,
        _intermediate_update: binding::fmi3IntermediateUpdateCallback,
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

        let intermediate_update = _intermediate_update.map(|cb| {
            Box::new(
                move |time: f64,
                      variable_set_requested: bool,
                      variable_get_allowed: bool,
                      step_finished: bool,
                      can_return_early: bool|
                      -> Option<f64> {
                    let mut early_return_requested: binding::fmi3Boolean = false.into();
                    let mut early_return_time: binding::fmi3Float64 = 0.0;
                    unsafe {
                        cb(
                            std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                            time,
                            variable_set_requested.into(),
                            variable_get_allowed.into(),
                            step_finished.into(),
                            can_return_early.into(),
                            &mut early_return_requested as *mut binding::fmi3Boolean,
                            &mut early_return_time as *mut binding::fmi3Float64,
                        )
                    };

                    if early_return_requested.into() {
                        Some(early_return_time)
                    } else {
                        None
                    }
                },
            )
        });

        // Check if Co-Simulation is supported
        if !Self::SUPPORTS_CO_SIMULATION {
            eprintln!("Co-Simulation not supported by this FMU");
            return ::std::ptr::null_mut();
        }

        let logging_on = _logging_on.into();
        let log_message: LogMessageClosure = if let Some(cb) = _log_message {
            Box::new(
                move |status: Fmi3Status, category: &str, args: std::fmt::Arguments<'_>| {
                    let category_c = CString::new(category).unwrap_or_default();
                    let message_c = CString::new(args.to_string()).unwrap_or_default();
                    unsafe {
                        cb(
                            std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                            status.into(),
                            category_c.as_ptr(),
                            message_c.as_ptr(),
                        )
                    };
                },
            )
        } else {
            Box::new(
                move |status: Fmi3Status, category: &str, args: std::fmt::Arguments<'_>| {
                    let category_c = CString::new(category).unwrap_or_default();
                    let message_c = CString::new(args.to_string()).unwrap_or_default();
                    eprintln!(
                        "Log (status: {:?}, category: {}): {}",
                        status,
                        category_c.to_string_lossy(),
                        message_c.to_string_lossy()
                    );
                },
            )
        };

        let early_return_allowed = _early_return_allowed.into();
        let context = BasicContext::new(
            logging_on,
            log_message,
            resource_path,
            early_return_allowed,
        );

        match crate::fmi3::ModelInstance::<Self, BasicContext<Self>>::new(
            name,
            &token,
            context,
            fmi::InterfaceType::CoSimulation,
        ) {
            Ok(instance) => {
                ::std::boxed::Box::into_raw(::std::boxed::Box::new(instance)) as binding::fmi3Instance
            }
            Err(_) => {
                eprintln!("Failed to instantiate FMU: invalid instantiation token");
                ::std::ptr::null_mut()
            }
        }
    }

    #[inline(always)]
    unsafe fn fmi3_instantiate_scheduled_execution(
        instance_name: binding::fmi3String,
        instantiation_token: binding::fmi3String,
        resource_path: binding::fmi3String,
        _visible: binding::fmi3Boolean,
        _logging_on: binding::fmi3Boolean,
        _instance_environment: binding::fmi3InstanceEnvironment,
        _log_message: binding::fmi3LogMessageCallback,
        _clock_update: binding::fmi3ClockUpdateCallback,
        _lock_preemption: binding::fmi3LockPreemptionCallback,
        _unlock_preemption: binding::fmi3UnlockPreemptionCallback,
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

        // Read instance_type to determine which concrete type to use
        // Safety: instance_type is at the same offset for all ModelInstance<M, C> types
        let instance_type = unsafe {
            let temp = instance
                as *const crate::fmi3::ModelInstance<
                    Self,
                    crate::fmi3::instance::context::BasicContext<Self>,
                >;
            (*temp).instance_type()
        };

        // Drop the correct concrete type based on instance_type
        match instance_type {
            fmi::InterfaceType::ModelExchange => {
                let _this = unsafe {
                    ::std::boxed::Box::from_raw(
                        instance
                            as *mut crate::fmi3::ModelInstance<
                                Self,
                                crate::fmi3::instance::context::BasicContext<Self>,
                            >,
                    )
                };
                _this.context().log(
                    Fmi3Res::OK.into(),
                    Default::default(),
                    format_args!("{}: fmi3FreeInstance()", _this.instance_name()),
                );
                // _this dropped here
            }
            fmi::InterfaceType::CoSimulation => {
                let _this = unsafe {
                    ::std::boxed::Box::from_raw(
                        instance
                            as *mut crate::fmi3::ModelInstance<
                                Self,
                                crate::fmi3::instance::context::BasicContext<Self>,
                            >,
                    )
                };
                _this.context().log(
                    Fmi3Res::OK.into(),
                    Default::default(),
                    format_args!("{}: fmi3FreeInstance()", _this.instance_name()),
                );
                // _this dropped here
            }
            fmi::InterfaceType::ScheduledExecution => {
                // TODO: Add SEContext when implemented
                eprintln!("Scheduled Execution not yet implemented");
            }
        }
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
        let tolerance = tolerance_defined.then_some(tolerance);
        let stop_time = stop_time_defined.then_some(stop_time);
        match dispatch_by_instance_type!(
            instance,
            Self,
            enter_initialization_mode,
            tolerance,
            start_time,
            stop_time
        ) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_exit_initialization_mode(
        instance: binding::fmi3Instance,
    ) -> binding::fmi3Status {
        match dispatch_by_instance_type!(instance, Self, exit_initialization_mode) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_enter_event_mode(instance: binding::fmi3Instance) -> binding::fmi3Status {
        match dispatch_by_instance_type!(instance, Self, enter_event_mode) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_terminate(instance: binding::fmi3Instance) -> binding::fmi3Status {
        match dispatch_by_instance_type!(instance, Self, terminate) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_reset(instance: binding::fmi3Instance) -> binding::fmi3Status {
        match dispatch_by_instance_type!(instance, Self, reset) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
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
        //let _instance = checked_deref_me!(instance, Self);
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
        //let _instance = checked_deref_me!(instance, Self);
        todo!("Adjoint derivative not yet implemented");
    }

    // Configuration mode functions
    #[inline(always)]
    unsafe fn fmi3_enter_configuration_mode(
        instance: binding::fmi3Instance,
    ) -> binding::fmi3Status {
        match dispatch_by_instance_type!(instance, Self, enter_configuration_mode) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_exit_configuration_mode(instance: binding::fmi3Instance) -> binding::fmi3Status {
        match dispatch_by_instance_type!(instance, Self, exit_configuration_mode) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
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
        //let _instance = checked_deref_me!(instance, Self);
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
        //let _instance = checked_deref_me!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_get_shift_decimal(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _shifts: *mut binding::fmi3Float64,
    ) -> binding::fmi3Status {
        //let _instance = checked_deref_me!(instance, Self);
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
        //let _instance = checked_deref_me!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_set_interval_decimal(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _intervals: *const binding::fmi3Float64,
    ) -> binding::fmi3Status {
        //let _instance = checked_deref_me!(instance, Self);
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
        //let _instance = checked_deref_me!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_set_shift_decimal(
        instance: binding::fmi3Instance,
        _value_references: *const binding::fmi3ValueReference,
        _n_value_references: usize,
        _shifts: *const binding::fmi3Float64,
    ) -> binding::fmi3Status {
        //let _instance = checked_deref_me!(instance, Self);
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
        //let _instance = checked_deref_me!(instance, Self);
        todo!("Clock interval not yet implemented");
    }

    #[inline(always)]
    unsafe fn fmi3_evaluate_discrete_states(
        instance: binding::fmi3Instance,
    ) -> binding::fmi3Status {
        //let _instance = checked_deref_me!(instance, Self);
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
        let mut event_flags = ::fmi::EventFlags::default();

        // next_time_event is potentially used as an in-out parameter
        if unsafe { *next_event_time_defined } {
            event_flags.next_event_time = Some(unsafe { *next_event_time });
        }

        match dispatch_by_instance_type!(instance, Self, update_discrete_states, &mut event_flags) {
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

                let status: Fmi3Status = res.into();
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

        match dispatch_by_instance_type!(
            instance,
            Self,
            get_string,
            value_refs,
            &mut temp_strings
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
                temp_strings.push(CString::default());
            } else {
                let cstring = unsafe { ::std::ffi::CStr::from_ptr(ptr) }.to_owned();
                temp_strings.push(cstring);
            }
        }

        match dispatch_by_instance_type!(instance, Self, set_string, value_refs, &temp_strings) {
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

        match dispatch_by_instance_type!(instance, Self, get_binary, value_refs, &mut temp_buffers) {
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

        match dispatch_by_instance_type!(instance, Self, set_binary, value_refs, &temp_buffers) {
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
        let value_refs =
            unsafe { ::std::slice::from_raw_parts(value_references, n_value_references) };
        let values_slice = unsafe { ::std::slice::from_raw_parts_mut(values, n_value_references) };
        match dispatch_by_instance_type!(instance, Self, get_clock, value_refs, values_slice) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
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
        let value_refs =
            unsafe { ::std::slice::from_raw_parts(value_references, n_value_references) };
        let values_slice = unsafe { ::std::slice::from_raw_parts(values, n_value_references) };
        match dispatch_by_instance_type!(instance, Self, set_clock, value_refs, values_slice) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
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
        match dispatch_by_instance_type!(
            instance,
            Self,
            get_number_of_variable_dependencies,
            value_reference
        ) {
            Ok(res) => {
                unsafe {
                    *n_dependencies = res;
                }
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
        // Convert the value reference to our trait's ValueRef type
        let dependent_vr = dependent.into();

        // Call the Rust method to get dependencies
        match dispatch_by_instance_type!(
            instance,
            Self,
            get_variable_dependencies,
            dependent_vr
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
                Fmi3Status::from(e).into()
            }
        }
    }
}

// Model Exchange trait
pub trait Fmi3ModelExchange: Fmi3Common + ModelGetSetStates + UserModelME
where
    ModelInstance<Self, BasicContext<Self>>: fmi::fmi3::ModelExchange,
{
    #[inline(always)]
    unsafe fn fmi3_enter_continuous_time_mode(
        instance: binding::fmi3Instance,
    ) -> binding::fmi3Status {
        match dispatch_by_instance_type!(instance, Self, enter_continuous_time_mode) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
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
        let mut enter_event = false;
        let mut terminate = false;
        match dispatch_by_instance_type!(
            instance,
            Self,
            completed_integrator_step,
            no_set_fmu_state_prior,
            &mut enter_event,
            &mut terminate
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
        match dispatch_by_instance_type!(instance, Self, set_time, time) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
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
        let states =
            unsafe { ::std::slice::from_raw_parts(continuous_states, n_continuous_states) };
        match dispatch_by_instance_type!(instance, Self, set_continuous_states, states) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
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
        let derivs = unsafe { ::std::slice::from_raw_parts_mut(derivatives, n_continuous_states) };
        match dispatch_by_instance_type!(
            instance,
            Self,
            get_continuous_state_derivatives,
            derivs
        ) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
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
        let indicators =
            unsafe { ::std::slice::from_raw_parts_mut(event_indicators, n_event_indicators) };
        match dispatch_by_instance_type!(instance, Self, get_event_indicators, indicators) {
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
        let states =
            unsafe { ::std::slice::from_raw_parts_mut(continuous_states, n_continuous_states) };
        match dispatch_by_instance_type!(instance, Self, get_continuous_states, states) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
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
        let nominals = unsafe { ::std::slice::from_raw_parts_mut(nominals, n_continuous_states) };
        match dispatch_by_instance_type!(
            instance,
            Self,
            get_nominals_of_continuous_states,
            nominals
        ) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
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
        match dispatch_by_instance_type!(instance, Self, get_number_of_event_indicators) {
            Ok(n) => {
                unsafe {
                    *n_event_indicators = n;
                }
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
        match dispatch_by_instance_type!(instance, Self, get_number_of_continuous_states) {
            Ok(n) => {
                unsafe {
                    *n_continuous_states = n;
                }
                binding::fmi3Status_fmi3OK
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }
}

// Co-Simulation trait
pub trait Fmi3CoSimulation: Fmi3Common + ModelGetSetStates + UserModelCS
where
    ModelInstance<Self, BasicContext<Self>>: fmi::fmi3::CoSimulation,
{
    #[inline(always)]
    unsafe fn fmi3_enter_step_mode(instance: binding::fmi3Instance) -> binding::fmi3Status {
        match dispatch_by_instance_type!(instance, Self, enter_step_mode) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_get_output_derivatives(
        instance: binding::fmi3Instance,
        value_references: *const binding::fmi3ValueReference,
        n_value_references: usize,
        orders: *const binding::fmi3Int32,
        values: *mut binding::fmi3Float64,
        n_values: usize,
    ) -> binding::fmi3Status {
        let value_refs =
            unsafe { ::std::slice::from_raw_parts(value_references, n_value_references) };
        let orders_slice = unsafe { ::std::slice::from_raw_parts(orders, n_value_references) };
        let values_slice = unsafe { ::std::slice::from_raw_parts_mut(values, n_values) };
        match dispatch_by_instance_type!(
            instance,
            Self,
            get_output_derivatives,
            value_refs,
            orders_slice,
            values_slice
        ) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
                status.into()
            }
            Err(_) => binding::fmi3Status_fmi3Error,
        }
    }

    #[inline(always)]
    unsafe fn fmi3_do_step(
        instance: binding::fmi3Instance,
        current_communication_point: binding::fmi3Float64,
        communication_step_size: binding::fmi3Float64,
        no_set_fmu_state_prior_to_current_point: binding::fmi3Boolean,
        event_handling_needed: *mut binding::fmi3Boolean,
        terminate_simulation: *mut binding::fmi3Boolean,
        early_return: *mut binding::fmi3Boolean,
        last_successful_time: *mut binding::fmi3Float64,
    ) -> binding::fmi3Status {
        // Handle optional output pointers gracefully; if null, write into local temps.
        let mut event_handling_needed_tmp = false;
        let mut terminate_simulation_tmp = false;
        let mut early_return_tmp = false;
        let mut last_successful_time_tmp = 0.0;

        let event_handling_needed = unsafe {
            if event_handling_needed.is_null() {
                &mut event_handling_needed_tmp
            } else {
                &mut *event_handling_needed
            }
        };
        let terminate_simulation = unsafe {
            if terminate_simulation.is_null() {
                &mut terminate_simulation_tmp
            } else {
                &mut *terminate_simulation
            }
        };
        let early_return = unsafe {
            if early_return.is_null() {
                &mut early_return_tmp
            } else {
                &mut *early_return
            }
        };
        let last_successful_time = unsafe {
            if last_successful_time.is_null() {
                &mut last_successful_time_tmp
            } else {
                &mut *last_successful_time
            }
        };

        match dispatch_by_instance_type!(
            instance,
            Self,
            do_step,
            current_communication_point,
            communication_step_size,
            no_set_fmu_state_prior_to_current_point,
            event_handling_needed,
            terminate_simulation,
            early_return,
            last_successful_time
        ) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
                status.into()
            }
            Err(e) => fmi::fmi3::Fmi3Status::from(e).into(),
        }
    }
}

pub trait Fmi3ScheduledExecution: Fmi3Common + ModelGetSetStates
where
    ModelInstance<Self, BasicContext<Self>>: fmi::fmi3::ScheduledExecution,
{
    #[inline(always)]
    unsafe fn fmi3_activate_model_partition(
        instance: binding::fmi3Instance,
        clock_reference: binding::fmi3ValueReference,
        activation_time: binding::fmi3Float64,
    ) -> binding::fmi3Status {
        match dispatch_by_instance_type!(
            instance,
            Self,
            activate_model_partition,
            clock_reference.into(),
            activation_time
        ) {
            Ok(res) => {
                let status: Fmi3Status = res.into();
                status.into()
            }
            Err(e) => fmi::fmi3::Fmi3Status::from(e).into(),
        }
    }
}

// Automatic implementations for all models
impl<T> Fmi3Common for T where T: Model + UserModel + ModelGetSet<Self> + ModelGetSetStates + 'static {}

impl<T> Fmi3ModelExchange for T
where
    T: Model + UserModel + UserModelME + Fmi3Common + ModelGetSetStates + 'static,
    ModelInstance<T, BasicContext<T>>: fmi::fmi3::ModelExchange,
{
}

impl<T> Fmi3CoSimulation for T
where
    T: Model + ModelGetSetStates + UserModel + UserModelCS + Fmi3Common + 'static,
    ModelInstance<T, BasicContext<T>>: fmi::fmi3::CoSimulation,
{
}

impl<T> Fmi3ScheduledExecution for T where
    T: Model + UserModel + ModelGetSetStates + Fmi3Common + 'static
{
}
