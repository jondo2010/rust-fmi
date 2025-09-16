/// Generates getter and setter functions for FMI3 data types.
#[macro_export]
macro_rules! generate_getset_functions {
    ($ty:ty, $type_name:ident, $fmi_type:ty) => {
        $crate::paste::paste! {
            #[unsafe(export_name = stringify!([<fmi3Get $type_name>]))]
            #[cfg_attr(coverage_nightly, coverage(off))]
            pub unsafe extern "C" fn [<fmi3_get_ $type_name:snake>](
                instance: ::fmi::fmi3::binding::fmi3Instance,
                value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
                n_value_references: usize,
                values: *mut $fmi_type,
                n_values: usize,
            ) -> ::fmi::fmi3::binding::fmi3Status {
                <$ty as $crate::fmi3::Fmi3Common>::[<fmi3_get_ $type_name:snake>](
                    instance,
                    value_references,
                    n_value_references,
                    values,
                    n_values
                )
            }

            #[unsafe(export_name = stringify!([<fmi3Set $type_name>]))]
            #[cfg_attr(coverage_nightly, coverage(off))]
            pub unsafe extern "C" fn [<fmi3_set_ $type_name:snake>](
                instance: ::fmi::fmi3::binding::fmi3Instance,
                value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
                n_value_references: usize,
                values: *const $fmi_type,
                n_values: usize,
            ) -> ::fmi::fmi3::binding::fmi3Status {
                <$ty as $crate::fmi3::Fmi3Common>::[<fmi3_set_ $type_name:snake>](
                    instance,
                    value_references,
                    n_value_references,
                    values,
                    n_values
                )
            }
        }
    };
}

/// Main macro for exporting an FMI 3.0 model as a shared library.
///
/// This macro generates all the required C API functions for an FMI 3.0 Functional Mockup Unit (FMU).
/// It creates the complete interface required by an FMU importer.
///
/// # Parameters
///
/// - `$ty`: The model type that implements the `Model` trait
///
/// # Requirements
///
/// The model type `$ty` must implement:
/// - `Model` trait: Provides model metadata and core simulation functionality
/// - `Default`: For creating initial model instances
/// - `UserModel`: For user-defined model behavior
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
        #[unsafe(export_name = "FMI3_MODEL_VARIABLES")]
        pub static FMI3_MODEL_VARIABLES: &'static str =
            <$ty as ::fmi_export::fmi3::Model>::MODEL_VARIABLES_XML;

        #[unsafe(export_name = "FMI3_MODEL_STRUCTURE")]
        pub static FMI3_MODEL_STRUCTURE: &'static str =
            <$ty as ::fmi_export::fmi3::Model>::MODEL_STRUCTURE_XML;

        #[unsafe(export_name = "FMI3_INSTANTIATION_TOKEN")]
        pub static FMI3_INSTANTIATION_TOKEN: &'static str =
            <$ty as ::fmi_export::fmi3::Model>::INSTANTIATION_TOKEN;

        // Inquire version numbers and set debug logging

        #[unsafe(export_name = "fmi3GetVersion")]
        #[cfg_attr(coverage_nightly, coverage(off))]
        pub unsafe extern "C" fn fmi3_get_version() -> *const ::std::os::raw::c_char {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_version()
        }

        #[unsafe(export_name = "fmi3SetDebugLogging")]
        #[cfg_attr(coverage_nightly, coverage(off))]
        pub unsafe extern "C" fn fmi3_set_debug_logging(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            logging_on: ::fmi::fmi3::binding::fmi3Boolean,
            n_categories: usize,
            categories: *const ::fmi::fmi3::binding::fmi3String,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_set_debug_logging(
                instance,
                logging_on,
                n_categories,
                categories,
            )
        }

        // Creation and destruction of FMU instances

        #[unsafe(export_name = "fmi3InstantiateModelExchange")]
        #[cfg_attr(coverage_nightly, coverage(off))]
        unsafe extern "C" fn fmi3_instantiate_model_exchange(
            instance_name: ::fmi::fmi3::binding::fmi3String,
            instantiation_token: ::fmi::fmi3::binding::fmi3String,
            resource_path: ::fmi::fmi3::binding::fmi3String,
            visible: ::fmi::fmi3::binding::fmi3Boolean,
            logging_on: ::fmi::fmi3::binding::fmi3Boolean,
            instance_environment: ::fmi::fmi3::binding::fmi3InstanceEnvironment,
            log_message: ::fmi::fmi3::binding::fmi3LogMessageCallback,
        ) -> ::fmi::fmi3::binding::fmi3Instance {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_instantiate_model_exchange(
                instance_name,
                instantiation_token,
                resource_path,
                visible,
                logging_on,
                instance_environment,
                log_message,
            )
        }

        #[unsafe(export_name = "fmi3InstantiateCoSimulation")]
        #[cfg_attr(coverage_nightly, coverage(off))]
        unsafe extern "C" fn fmi3_instantiate_co_simulation(
            instance_name: ::fmi::fmi3::binding::fmi3String,
            instantiation_token: ::fmi::fmi3::binding::fmi3String,
            resource_path: ::fmi::fmi3::binding::fmi3String,
            visible: ::fmi::fmi3::binding::fmi3Boolean,
            logging_on: ::fmi::fmi3::binding::fmi3Boolean,
            event_mode_used: ::fmi::fmi3::binding::fmi3Boolean,
            early_return_allowed: ::fmi::fmi3::binding::fmi3Boolean,
            required_intermediate_variables: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_required_intermediate_variables: usize,
            instance_environment: ::fmi::fmi3::binding::fmi3InstanceEnvironment,
            log_message: ::fmi::fmi3::binding::fmi3LogMessageCallback,
            intermediate_update: ::fmi::fmi3::binding::fmi3IntermediateUpdateCallback,
        ) -> ::fmi::fmi3::binding::fmi3Instance {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_instantiate_co_simulation(
                instance_name,
                instantiation_token,
                resource_path,
                visible,
                logging_on,
                event_mode_used,
                early_return_allowed,
                required_intermediate_variables,
                n_required_intermediate_variables,
                instance_environment,
                log_message,
                intermediate_update,
            )
        }

        #[unsafe(export_name = "fmi3InstantiateScheduledExecution")]
        #[cfg_attr(coverage_nightly, coverage(off))]
        unsafe extern "C" fn fmi3_instantiate_scheduled_execution(
            instance_name: ::fmi::fmi3::binding::fmi3String,
            instantiation_token: ::fmi::fmi3::binding::fmi3String,
            resource_path: ::fmi::fmi3::binding::fmi3String,
            visible: ::fmi::fmi3::binding::fmi3Boolean,
            logging_on: ::fmi::fmi3::binding::fmi3Boolean,
            instance_environment: ::fmi::fmi3::binding::fmi3InstanceEnvironment,
            log_message: ::fmi::fmi3::binding::fmi3LogMessageCallback,
            clock_update: ::fmi::fmi3::binding::fmi3ClockUpdateCallback,
            lock_preemption: ::fmi::fmi3::binding::fmi3LockPreemptionCallback,
            unlock_preemption: ::fmi::fmi3::binding::fmi3UnlockPreemptionCallback,
        ) -> ::fmi::fmi3::binding::fmi3Instance {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_instantiate_scheduled_execution(
                instance_name,
                instantiation_token,
                resource_path,
                visible,
                logging_on,
                instance_environment,
                log_message,
                clock_update,
                lock_preemption,
                unlock_preemption,
            )
        }

        #[unsafe(export_name = "fmi3FreeInstance")]
        #[cfg_attr(coverage_nightly, coverage(off))]
        unsafe extern "C" fn fmi3_free_instance(instance: ::fmi::fmi3::binding::fmi3Instance) {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_free_instance(instance)
        }

        // Enter and exit initialization mode, terminate and reset

        #[unsafe(export_name = "fmi3EnterInitializationMode")]
        #[cfg_attr(coverage_nightly, coverage(off))]
        unsafe extern "C" fn fmi3_enter_initialization_mode(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            tolerance_defined: ::fmi::fmi3::binding::fmi3Boolean,
            tolerance: ::fmi::fmi3::binding::fmi3Float64,
            start_time: ::fmi::fmi3::binding::fmi3Float64,
            stop_time_defined: ::fmi::fmi3::binding::fmi3Boolean,
            stop_time: ::fmi::fmi3::binding::fmi3Float64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_enter_initialization_mode(
                instance,
                tolerance_defined,
                tolerance,
                start_time,
                stop_time_defined,
                stop_time,
            )
        }

        #[unsafe(export_name = "fmi3ExitInitializationMode")]
        #[cfg_attr(coverage_nightly, coverage(off))]
        unsafe extern "C" fn fmi3_exit_initialization_mode(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_exit_initialization_mode(instance)
        }

        #[unsafe(export_name = "fmi3EnterEventMode")]
        #[cfg_attr(coverage_nightly, coverage(off))]
        unsafe extern "C" fn fmi3_enter_event_mode(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_enter_event_mode(instance)
        }

        #[unsafe(export_name = "fmi3Terminate")]
        #[cfg_attr(coverage_nightly, coverage(off))]
        unsafe extern "C" fn fmi3_terminate(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_terminate(instance)
        }

        #[unsafe(export_name = "fmi3Reset")]
        #[cfg_attr(coverage_nightly, coverage(off))]
        unsafe extern "C" fn fmi3_reset(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_reset(instance)
        }

        // Getting and setting variable values

        $crate::generate_getset_functions!($ty, Float64, ::fmi::fmi3::binding::fmi3Float64);
        $crate::generate_getset_functions!($ty, Float32, ::fmi::fmi3::binding::fmi3Float32);
        $crate::generate_getset_functions!($ty, Int64, ::fmi::fmi3::binding::fmi3Int64);
        $crate::generate_getset_functions!($ty, Int32, ::fmi::fmi3::binding::fmi3Int32);
        $crate::generate_getset_functions!($ty, Int16, ::fmi::fmi3::binding::fmi3Int16);
        $crate::generate_getset_functions!($ty, Int8, ::fmi::fmi3::binding::fmi3Int8);
        $crate::generate_getset_functions!($ty, UInt64, ::fmi::fmi3::binding::fmi3UInt64);
        $crate::generate_getset_functions!($ty, UInt32, ::fmi::fmi3::binding::fmi3UInt32);
        $crate::generate_getset_functions!($ty, UInt16, ::fmi::fmi3::binding::fmi3UInt16);
        $crate::generate_getset_functions!($ty, UInt8, ::fmi::fmi3::binding::fmi3UInt8);
        $crate::generate_getset_functions!($ty, Boolean, ::fmi::fmi3::binding::fmi3Boolean);

        // String and Binary types need special handling due to their different signatures
        #[unsafe(export_name = "fmi3GetString")]
        pub unsafe extern "C" fn fmi3_get_string(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            values: *mut ::fmi::fmi3::binding::fmi3String,
            n_values: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_string(
                instance,
                value_references,
                n_value_references,
                values,
                n_values,
            )
        }

        #[unsafe(export_name = "fmi3SetString")]
        pub unsafe extern "C" fn fmi3_set_string(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            values: *const ::fmi::fmi3::binding::fmi3String,
            n_values: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_set_string(
                instance,
                value_references,
                n_value_references,
                values,
                n_values,
            )
        }

        #[unsafe(export_name = "fmi3GetBinary")]
        pub unsafe extern "C" fn fmi3_get_binary(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            value_sizes: *mut usize,
            values: *mut *mut ::fmi::fmi3::binding::fmi3Byte,
            n_values: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_binary(
                instance,
                value_references,
                n_value_references,
                value_sizes,
                values,
                n_values,
            )
        }

        #[unsafe(export_name = "fmi3SetBinary")]
        pub unsafe extern "C" fn fmi3_set_binary(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            value_sizes: *const usize,
            values: *const *const ::fmi::fmi3::binding::fmi3Byte,
            n_values: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_set_binary(
                instance,
                value_references,
                n_value_references,
                value_sizes,
                values,
                n_values,
            )
        }

        #[unsafe(export_name = "fmi3GetClock")]
        pub unsafe extern "C" fn fmi3_get_clock(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            values: *mut ::fmi::fmi3::binding::fmi3Clock,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_clock(
                instance,
                value_references,
                n_value_references,
                values,
            )
        }

        #[unsafe(export_name = "fmi3SetClock")]
        pub unsafe extern "C" fn fmi3_set_clock(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            values: *const ::fmi::fmi3::binding::fmi3Clock,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_set_clock(
                instance,
                value_references,
                n_value_references,
                values,
            )
        }

        // Getting Variable Dependency Information

        #[unsafe(export_name = "fmi3GetNumberOfVariableDependencies")]
        unsafe extern "C" fn fmi3_get_number_of_variable_dependencies(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_reference: ::fmi::fmi3::binding::fmi3ValueReference,
            n_dependencies: *mut usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_number_of_variable_dependencies(
                instance,
                value_reference,
                n_dependencies,
            )
        }

        #[unsafe(export_name = "fmi3GetVariableDependencies")]
        unsafe extern "C" fn fmi3_get_variable_dependencies(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            dependent: ::fmi::fmi3::binding::fmi3ValueReference,
            element_indices_of_dependent: *mut usize,
            independents: *mut ::fmi::fmi3::binding::fmi3ValueReference,
            element_indices_of_independents: *mut usize,
            dependency_kinds: *mut ::fmi::fmi3::binding::fmi3DependencyKind,
            n_dependencies: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_variable_dependencies(
                instance,
                dependent,
                element_indices_of_dependent,
                independents,
                element_indices_of_independents,
                dependency_kinds,
                n_dependencies,
            )
        }

        // Getting and setting the internal FMU state

        #[unsafe(export_name = "fmi3GetFMUState")]
        unsafe extern "C" fn fmi3_get_fmu_state(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            fmu_state: *mut ::fmi::fmi3::binding::fmi3FMUState,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_fmu_state(instance, fmu_state)
        }

        #[unsafe(export_name = "fmi3SetFMUState")]
        unsafe extern "C" fn fmi3_set_fmu_state(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            fmu_state: ::fmi::fmi3::binding::fmi3FMUState,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_set_fmu_state(instance, fmu_state)
        }

        #[unsafe(export_name = "fmi3FreeFMUState")]
        unsafe extern "C" fn fmi3_free_fmu_state(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            fmu_state: *mut ::fmi::fmi3::binding::fmi3FMUState,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_free_fmu_state(instance, fmu_state)
        }

        #[unsafe(export_name = "fmi3SerializedFMUStateSize")]
        unsafe fn fmi3_serialized_fmu_state_size(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            fmu_state: ::fmi::fmi3::binding::fmi3FMUState,
            size: *mut usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_serialized_fmu_state_size(
                instance, fmu_state, size,
            )
        }

        #[unsafe(export_name = "fmi3SerializeFMUState")]
        unsafe fn fmi3_serialize_fmu_state(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            fmu_state: ::fmi::fmi3::binding::fmi3FMUState,
            serialized_state: *mut ::fmi::fmi3::binding::fmi3Byte,
            size: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_serialize_fmu_state(
                instance,
                fmu_state,
                serialized_state,
                size,
            )
        }

        #[unsafe(export_name = "fmi3DeserializeFMUState")]
        pub unsafe fn fmi3_deserialize_fmu_state(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            serialized_state: *const ::fmi::fmi3::binding::fmi3Byte,
            size: usize,
            fmu_state: *mut ::fmi::fmi3::binding::fmi3FMUState,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_deserialize_fmu_state(
                instance,
                serialized_state,
                size,
                fmu_state,
            )
        }

        // Getting partial derivatives

        #[unsafe(export_name = "fmi3GetDirectionalDerivative")]
        unsafe extern "C" fn fmi3_get_directional_derivative(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            unknowns: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_unknowns: usize,
            knowns: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_knowns: usize,
            seed: *const ::fmi::fmi3::binding::fmi3Float64,
            n_seed: usize,
            sensitivity: *mut ::fmi::fmi3::binding::fmi3Float64,
            n_sensitivity: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_directional_derivative(
                instance,
                unknowns,
                n_unknowns,
                knowns,
                n_knowns,
                seed,
                n_seed,
                sensitivity,
                n_sensitivity,
            )
        }

        #[unsafe(export_name = "fmi3GetAdjointDerivative")]
        unsafe extern "C" fn fmi3_get_adjoint_derivative(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            unknowns: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_unknowns: usize,
            knowns: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_knowns: usize,
            seed: *const ::fmi::fmi3::binding::fmi3Float64,
            n_seed: usize,
            sensitivity: *mut ::fmi::fmi3::binding::fmi3Float64,
            n_sensitivity: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_adjoint_derivative(
                instance,
                unknowns,
                n_unknowns,
                knowns,
                n_knowns,
                seed,
                n_seed,
                sensitivity,
                n_sensitivity,
            )
        }

        // Entering and exiting the Configuration or Reconfiguration Mode

        #[unsafe(export_name = "fmi3EnterConfigurationMode")]
        unsafe extern "C" fn fmi3_enter_configuration_mode(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_enter_configuration_mode(instance)
        }

        #[unsafe(export_name = "fmi3ExitConfigurationMode")]
        unsafe extern "C" fn fmi3_exit_configuration_mode(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_exit_configuration_mode(instance)
        }

        // Clock related functions

        #[unsafe(export_name = "fmi3GetIntervalDecimal")]
        unsafe fn fmi3_get_interval_decimal(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            intervals: *mut ::fmi::fmi3::binding::fmi3Float64,
            qualifiers: *mut ::fmi::fmi3::binding::fmi3IntervalQualifier,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_interval_decimal(
                instance,
                value_references,
                n_value_references,
                intervals,
                qualifiers,
            )
        }

        #[unsafe(export_name = "fmi3GetIntervalFraction")]
        unsafe extern "C" fn fmi3_get_interval_fraction(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            counters: *mut ::fmi::fmi3::binding::fmi3UInt64,
            resolutions: *mut ::fmi::fmi3::binding::fmi3UInt64,
            qualifiers: *mut ::fmi::fmi3::binding::fmi3IntervalQualifier,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_interval_fraction(
                instance,
                value_references,
                n_value_references,
                counters,
                resolutions,
                qualifiers,
            )
        }

        #[unsafe(export_name = "fmi3GetShiftDecimal")]
        unsafe extern "C" fn fmi3_get_shift_decimal(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            shifts: *mut ::fmi::fmi3::binding::fmi3Float64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_shift_decimal(
                instance,
                value_references,
                n_value_references,
                shifts,
            )
        }

        #[unsafe(export_name = "fmi3GetShiftFraction")]
        unsafe extern "C" fn fmi3_get_shift_fraction(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            counters: *mut ::fmi::fmi3::binding::fmi3UInt64,
            resolutions: *mut ::fmi::fmi3::binding::fmi3UInt64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_get_shift_fraction(
                instance,
                value_references,
                n_value_references,
                counters,
                resolutions,
            )
        }

        #[unsafe(export_name = "fmi3SetIntervalDecimal")]
        unsafe extern "C" fn fmi3_set_interval_decimal(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            intervals: *const ::fmi::fmi3::binding::fmi3Float64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_set_interval_decimal(
                instance,
                value_references,
                n_value_references,
                intervals,
            )
        }

        #[unsafe(export_name = "fmi3SetIntervalFraction")]
        unsafe extern "C" fn fmi3_set_interval_fraction(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            counters: *const ::fmi::fmi3::binding::fmi3UInt64,
            resolutions: *const ::fmi::fmi3::binding::fmi3UInt64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_set_interval_fraction(
                instance,
                value_references,
                n_value_references,
                counters,
                resolutions,
            )
        }

        #[unsafe(export_name = "fmi3SetShiftDecimal")]
        unsafe extern "C" fn fmi3_set_shift_decimal(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            shifts: *const ::fmi::fmi3::binding::fmi3Float64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_set_shift_decimal(
                instance,
                value_references,
                n_value_references,
                shifts,
            )
        }

        #[unsafe(export_name = "fmi3SetShiftFraction")]
        unsafe extern "C" fn fmi3_set_shift_fraction(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            counters: *const ::fmi::fmi3::binding::fmi3UInt64,
            resolutions: *const ::fmi::fmi3::binding::fmi3UInt64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_set_shift_fraction(
                instance,
                value_references,
                n_value_references,
                counters,
                resolutions,
            )
        }

        #[unsafe(export_name = "fmi3EvaluateDiscreteStates")]
        unsafe extern "C" fn fmi3_evaluate_discrete_states(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_evaluate_discrete_states(instance)
        }

        #[unsafe(export_name = "fmi3UpdateDiscreteStates")]
        unsafe extern "C" fn fmi3_update_discrete_states(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            discrete_states_need_update: *mut ::fmi::fmi3::binding::fmi3Boolean,
            terminate_simulation: *mut ::fmi::fmi3::binding::fmi3Boolean,
            nominals_of_continuous_states_changed: *mut ::fmi::fmi3::binding::fmi3Boolean,
            values_of_continuous_states_changed: *mut ::fmi::fmi3::binding::fmi3Boolean,
            next_event_time_defined: *mut ::fmi::fmi3::binding::fmi3Boolean,
            next_event_time: *mut ::fmi::fmi3::binding::fmi3Float64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3Common>::fmi3_update_discrete_states(
                instance,
                discrete_states_need_update,
                terminate_simulation,
                nominals_of_continuous_states_changed,
                values_of_continuous_states_changed,
                next_event_time_defined,
                next_event_time,
            )
        }

        // # Functions for Model Exchange

        #[unsafe(export_name = "fmi3EnterContinuousTimeMode")]
        unsafe extern "C" fn fmi3_enter_continuous_time_mode(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3ModelExchange>::fmi3_enter_continuous_time_mode(instance)
        }

        #[unsafe(export_name = "fmi3CompletedIntegratorStep")]
        unsafe extern "C" fn fmi3_completed_integrator_step(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            no_set_fmu_state_prior: ::fmi::fmi3::binding::fmi3Boolean,
            enter_event_mode: *mut ::fmi::fmi3::binding::fmi3Boolean,
            terminate_simulation: *mut ::fmi::fmi3::binding::fmi3Boolean,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3ModelExchange>::fmi3_completed_integrator_step(
                instance,
                no_set_fmu_state_prior,
                enter_event_mode,
                terminate_simulation,
            )
        }

        // Providing independent variables and re-initialization of caching

        #[unsafe(export_name = "fmi3SetTime")]
        unsafe extern "C" fn fmi3_set_time(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            time: ::fmi::fmi3::binding::fmi3Float64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3ModelExchange>::fmi3_set_time(instance, time)
        }

        #[unsafe(export_name = "fmi3SetContinuousStates")]
        unsafe extern "C" fn fmi3_set_continuous_states(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            continuous_states: *const ::fmi::fmi3::binding::fmi3Float64,
            n_continuous_states: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3ModelExchange>::fmi3_set_continuous_states(
                instance,
                continuous_states,
                n_continuous_states,
            )
        }

        // Evaluation of the model equations

        #[unsafe(export_name = "fmi3GetContinuousStateDerivatives")]
        unsafe extern "C" fn fmi3_get_continuous_state_derivatives(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            derivatives: *mut ::fmi::fmi3::binding::fmi3Float64,
            n_continuous_states: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3ModelExchange>::fmi3_get_continuous_state_derivatives(
                instance,
                derivatives,
                n_continuous_states,
            )
        }

        #[unsafe(export_name = "fmi3GetEventIndicators")]
        unsafe extern "C" fn fmi3_get_event_indicators(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            event_indicators: *mut ::fmi::fmi3::binding::fmi3Float64,
            n_event_indicators: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3ModelExchange>::fmi3_get_event_indicators(
                instance,
                event_indicators,
                n_event_indicators,
            )
        }

        #[unsafe(export_name = "fmi3GetContinuousStates")]
        unsafe extern "C" fn fmi3_get_continuous_states(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            continuous_states: *mut ::fmi::fmi3::binding::fmi3Float64,
            n_continuous_states: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3ModelExchange>::fmi3_get_continuous_states(
                instance,
                continuous_states,
                n_continuous_states,
            )
        }

        #[unsafe(export_name = "fmi3GetNominalsOfContinuousStates")]
        unsafe extern "C" fn fmi3_get_nominals_of_continuous_states(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            nominals: *mut ::fmi::fmi3::binding::fmi3Float64,
            n_continuous_states: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3ModelExchange>::fmi3_get_nominals_of_continuous_states(
                instance,
                nominals,
                n_continuous_states,
            )
        }

        #[unsafe(export_name = "fmi3GetNumberOfEventIndicators")]
        unsafe extern "C" fn fmi3_get_number_of_event_indicators(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            n_event_indicators: *mut usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3ModelExchange>::fmi3_get_number_of_event_indicators(
                instance,
                n_event_indicators,
            )
        }

        #[unsafe(export_name = "fmi3GetNumberOfContinuousStates")]
        unsafe extern "C" fn fmi3_get_number_of_continuous_states(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            n_continuous_states: *mut usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3ModelExchange>::fmi3_get_number_of_continuous_states(
                instance,
                n_continuous_states,
            )
        }

        // # Functions for Co-Simulation

        #[unsafe(export_name = "fmi3EnterStepMode")]
        unsafe extern "C" fn fmi3_enter_step_mode(
            instance: ::fmi::fmi3::binding::fmi3Instance,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3CoSimulation>::fmi3_enter_step_mode(instance)
        }

        #[unsafe(export_name = "fmi3GetOutputDerivatives")]
        unsafe extern "C" fn fmi3_get_output_derivatives(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            value_references: *const ::fmi::fmi3::binding::fmi3ValueReference,
            n_value_references: usize,
            orders: *const ::fmi::fmi3::binding::fmi3Int32,
            values: *mut ::fmi::fmi3::binding::fmi3Float64,
            n_values: usize,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3CoSimulation>::fmi3_get_output_derivatives(
                instance,
                value_references,
                n_value_references,
                orders,
                values,
                n_values,
            )
        }

        #[unsafe(export_name = "fmi3DoStep")]
        unsafe extern "C" fn fmi3_do_step(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            current_communication_point: ::fmi::fmi3::binding::fmi3Float64,
            communication_step_size: ::fmi::fmi3::binding::fmi3Float64,
            no_set_fmu_state_prior_to_current_point: ::fmi::fmi3::binding::fmi3Boolean,
            event_handling_needed: *mut ::fmi::fmi3::binding::fmi3Boolean,
            terminate_simulation: *mut ::fmi::fmi3::binding::fmi3Boolean,
            early_return: *mut ::fmi::fmi3::binding::fmi3Boolean,
            last_successful_time: *mut ::fmi::fmi3::binding::fmi3Float64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3CoSimulation>::fmi3_do_step(
                instance,
                current_communication_point,
                communication_step_size,
                no_set_fmu_state_prior_to_current_point,
                event_handling_needed,
                terminate_simulation,
                early_return,
                last_successful_time,
            )
        }

        #[unsafe(export_name = "fmi3ActivateModelPartition")]
        unsafe extern "C" fn fmi3_activate_model_partition(
            instance: ::fmi::fmi3::binding::fmi3Instance,
            clock_reference: ::fmi::fmi3::binding::fmi3ValueReference,
            activation_time: ::fmi::fmi3::binding::fmi3Float64,
        ) -> ::fmi::fmi3::binding::fmi3Status {
            <$ty as $crate::fmi3::Fmi3CoSimulation>::fmi3_activate_model_partition(
                instance,
                clock_reference,
                activation_time,
            )
        }
    };
}
