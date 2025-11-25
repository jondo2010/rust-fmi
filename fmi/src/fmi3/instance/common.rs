use std::mem::MaybeUninit;

use crate::{
    EventFlags,
    fmi3::{
        Fmi3Error, Fmi3Res, Fmi3Status, VariableDependency, binding,
        instance::Instance,
        traits::{Common, GetSet},
    },
    traits::FmiStatus,
};

macro_rules! impl_getter_setter {
    ($ty:ty, $get:ident, $set:ident, $fmi_get:ident, $fmi_set:ident) => {
        fn $get(
            &mut self,
            vrs: &[binding::fmi3ValueReference],
            values: &mut [$ty],
        ) -> Result<Fmi3Res, Fmi3Error> {
            Fmi3Status::from(unsafe {
                self.binding.$fmi_get(
                    self.ptr,
                    vrs.as_ptr(),
                    vrs.len() as _,
                    values.as_mut_ptr(),
                    values.len() as _,
                )
            })
            .ok()
        }

        fn $set(
            &mut self,
            vrs: &[binding::fmi3ValueReference],
            values: &[$ty],
        ) -> Result<Fmi3Res, Fmi3Error> {
            Fmi3Status::from(unsafe {
                self.binding.$fmi_set(
                    self.ptr,
                    vrs.as_ptr(),
                    vrs.len() as _,
                    values.as_ptr(),
                    values.len() as _,
                )
            })
            .ok()
        }
    };
}

impl<Tag> GetSet for Instance<Tag> {
    impl_getter_setter!(
        bool,
        get_boolean,
        set_boolean,
        fmi3GetBoolean,
        fmi3SetBoolean
    );
    impl_getter_setter!(
        f32,
        get_float32,
        set_float32,
        fmi3GetFloat32,
        fmi3SetFloat32
    );
    impl_getter_setter!(
        f64,
        get_float64,
        set_float64,
        fmi3GetFloat64,
        fmi3SetFloat64
    );
    impl_getter_setter!(i8, get_int8, set_int8, fmi3GetInt8, fmi3SetInt8);
    impl_getter_setter!(i16, get_int16, set_int16, fmi3GetInt16, fmi3SetInt16);
    impl_getter_setter!(i32, get_int32, set_int32, fmi3GetInt32, fmi3SetInt32);
    impl_getter_setter!(i64, get_int64, set_int64, fmi3GetInt64, fmi3SetInt64);
    impl_getter_setter!(u8, get_uint8, set_uint8, fmi3GetUInt8, fmi3SetUInt8);
    impl_getter_setter!(u16, get_uint16, set_uint16, fmi3GetUInt16, fmi3SetUInt16);
    impl_getter_setter!(u32, get_uint32, set_uint32, fmi3GetUInt32, fmi3SetUInt32);
    impl_getter_setter!(u64, get_uint64, set_uint64, fmi3GetUInt64, fmi3SetUInt64);

    fn get_string(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &mut [std::ffi::CString],
    ) -> Result<(), Fmi3Error> {
        let n_values = values.len();

        const STACK_THRESHOLD: usize = 32;

        if n_values <= STACK_THRESHOLD {
            // Stack-allocated array for small cases
            let mut stack_ptrs: [MaybeUninit<*const binding::fmi3Char>; STACK_THRESHOLD] =
                unsafe { MaybeUninit::uninit().assume_init() };

            Fmi3Status::from(unsafe {
                self.binding.fmi3GetString(
                    self.ptr,
                    vrs.as_ptr(),
                    vrs.len() as _,
                    stack_ptrs.as_mut_ptr() as *mut binding::fmi3String,
                    n_values,
                )
            })
            .ok()?;

            // Copy the C strings into the output CString values
            for (i, value) in values.iter_mut().enumerate() {
                let ptr = unsafe { stack_ptrs[i].assume_init() };
                if ptr.is_null() {
                    return Err(Fmi3Error::Error);
                }
                let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
                // Copy the string data into the provided CString
                *value = std::ffi::CString::new(cstr.to_bytes()).map_err(|_| Fmi3Error::Error)?;
            }
        } else {
            // Heap allocation for large arrays
            let mut value_ptrs: Vec<*const binding::fmi3Char> = vec![std::ptr::null(); n_values];

            Fmi3Status::from(unsafe {
                self.binding.fmi3GetString(
                    self.ptr,
                    vrs.as_ptr(),
                    vrs.len(),
                    value_ptrs.as_mut_ptr(),
                    n_values,
                )
            })
            .ok()?;

            // Copy the C strings into the output CString values
            for (value, ptr) in values.iter_mut().zip(value_ptrs.iter()) {
                if ptr.is_null() {
                    return Err(Fmi3Error::Error);
                }
                let cstr = unsafe { std::ffi::CStr::from_ptr(*ptr) };
                // Copy the string data into the provided CString
                *value = std::ffi::CString::new(cstr.to_bytes()).map_err(|_| Fmi3Error::Error)?;
            }
        }

        Ok(())
    }

    fn set_string(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &[std::ffi::CString],
    ) -> Result<(), Fmi3Error> {
        let n_values = values.len();

        const STACK_THRESHOLD: usize = 32;

        if n_values <= STACK_THRESHOLD {
            // Stack-allocated array for small cases
            let mut stack_ptrs: [MaybeUninit<*const binding::fmi3Char>; STACK_THRESHOLD] =
                unsafe { MaybeUninit::uninit().assume_init() };

            for (i, value) in values.iter().enumerate() {
                stack_ptrs[i] = MaybeUninit::new(value.as_ptr());
            }

            Fmi3Status::from(unsafe {
                self.binding.fmi3SetString(
                    self.ptr,
                    vrs.as_ptr(),
                    vrs.len() as _,
                    stack_ptrs.as_mut_ptr() as *const binding::fmi3String,
                    n_values,
                )
            })
            .ok()?;
        } else {
            // Heap allocation for large arrays
            let value_ptrs: Vec<*const binding::fmi3Char> =
                values.iter().map(|value| value.as_ptr()).collect();

            Fmi3Status::from(unsafe {
                self.binding.fmi3SetString(
                    self.ptr,
                    vrs.as_ptr(),
                    vrs.len() as _,
                    value_ptrs.as_ptr(),
                    n_values,
                )
            })
            .ok()?;
        }

        Ok(())
    }

    fn get_binary(
        &mut self,
        value_references: &[binding::fmi3ValueReference],
        value_buffers: &mut [&mut [u8]],
    ) -> Result<Vec<usize>, Fmi3Error> {
        let n_value_references = value_references.len();
        let n_values = value_buffers.len();

        let mut value_sizes = vec![0usize; n_values];

        // Use stack allocation for small arrays, heap for large ones
        const STACK_THRESHOLD: usize = 32;

        // Helper function to copy data from FMU pointers to user buffers
        let copy_binary_data = |value_ptrs: &[*const u8],
                                value_sizes: &[usize],
                                value_buffers: &mut [&mut [u8]]|
         -> Result<(), Fmi3Error> {
            for (i, value_buffer) in value_buffers.iter_mut().enumerate() {
                let fmu_ptr = value_ptrs[i];
                let fmu_size = value_sizes[i];

                if fmu_ptr.is_null() {
                    log::error!("FMU returned null pointer for binary value {}", i);
                    return Err(Fmi3Error::Error);
                }

                if value_buffer.len() < fmu_size {
                    log::error!(
                        "User buffer too small for binary value {}: buffer size = {}, required size = {}",
                        i,
                        value_buffer.len(),
                        fmu_size
                    );
                    return Err(Fmi3Error::Error);
                }

                // Copy the data from FMU buffer to user buffer
                unsafe {
                    std::ptr::copy_nonoverlapping(fmu_ptr, value_buffer.as_mut_ptr(), fmu_size);
                }
            }
            Ok(())
        };

        if n_values <= STACK_THRESHOLD {
            // Stack-allocated array for small cases
            let mut stack_ptrs: [MaybeUninit<*const u8>; STACK_THRESHOLD] =
                [const { MaybeUninit::uninit() }; STACK_THRESHOLD];

            // Initialize pointers to null for the FMU to populate
            for ptr in stack_ptrs.iter_mut() {
                *ptr = MaybeUninit::new(std::ptr::null());
            }

            Fmi3Status::from(unsafe {
                self.binding.fmi3GetBinary(
                    self.ptr,
                    value_references.as_ptr(),
                    n_value_references,
                    value_sizes.as_mut_ptr(),
                    stack_ptrs.as_mut_ptr() as *mut *const u8,
                    n_values,
                )
            })
            .ok()?;

            // Convert stack pointers to slice for copy function
            let ptr_slice: Vec<*const u8> = (0..n_values)
                .map(|i| unsafe { stack_ptrs[i].assume_init() })
                .collect();

            copy_binary_data(&ptr_slice, &value_sizes, value_buffers)?;
        } else {
            // Heap allocation for large arrays
            let mut value_ptrs: Vec<*const u8> = vec![std::ptr::null(); n_values];

            Fmi3Status::from(unsafe {
                self.binding.fmi3GetBinary(
                    self.ptr,
                    value_references.as_ptr(),
                    n_value_references,
                    value_sizes.as_mut_ptr(),
                    value_ptrs.as_mut_ptr(),
                    n_values,
                )
            })
            .ok()?;

            copy_binary_data(&value_ptrs, &value_sizes, value_buffers)?;
        }

        Ok(value_sizes)
    }

    fn set_binary(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &[&[u8]],
    ) -> Result<(), Fmi3Error> {
        let n_value_references = vrs.len();
        let n_values = values.len();

        let value_sizes: Vec<usize> = values.iter().map(|v| v.len()).collect();
        let value_ptrs: Vec<*const u8> = values.iter().map(|v| v.as_ptr()).collect();

        Fmi3Status::from(unsafe {
            self.binding.fmi3SetBinary(
                self.ptr,
                vrs.as_ptr(),
                n_value_references,
                value_sizes.as_ptr(),
                value_ptrs.as_ptr() as *const binding::fmi3Binary,
                n_values,
            )
        })
        .ok()?;

        Ok(())
    }

    fn get_clock(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &mut [binding::fmi3Clock],
    ) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe {
            self.binding
                .fmi3GetClock(self.ptr, vrs.as_ptr(), vrs.len() as _, values.as_mut_ptr())
        })
        .ok()
    }

    fn set_clock(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &[binding::fmi3Clock],
    ) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe {
            self.binding
                .fmi3SetClock(self.ptr, vrs.as_ptr(), vrs.len() as _, values.as_ptr())
        })
        .ok()
    }
}

impl<Tag> Common for Instance<Tag> {
    fn get_version(&self) -> &str {
        unsafe { std::ffi::CStr::from_ptr(self.binding.fmi3GetVersion()) }
            .to_str()
            .expect("Invalid version string")
    }

    fn set_debug_logging(
        &mut self,
        logging_on: bool,
        categories: &[&str],
    ) -> Result<Fmi3Res, Fmi3Error> {
        let cats_vec = categories
            .iter()
            .map(|cat| std::ffi::CString::new(cat.as_bytes()).expect("Error building CString"))
            .collect::<Vec<_>>();

        let cats_vec_ptrs = cats_vec
            .iter()
            .map(|cat| cat.as_c_str().as_ptr())
            .collect::<Vec<_>>();

        Fmi3Status::from(unsafe {
            self.binding.fmi3SetDebugLogging(
                self.ptr,
                logging_on,
                cats_vec_ptrs.len() as _,
                cats_vec_ptrs.as_ptr() as *const binding::fmi3String,
            )
        })
        .ok()
    }

    fn enter_configuration_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe { self.binding.fmi3EnterConfigurationMode(self.ptr) }).ok()
    }

    fn exit_configuration_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe { self.binding.fmi3ExitConfigurationMode(self.ptr) }).ok()
    }

    fn enter_initialization_mode(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe {
            self.binding.fmi3EnterInitializationMode(
                self.ptr,
                tolerance.is_some(),
                tolerance.unwrap_or_default(),
                start_time,
                stop_time.is_some(),
                stop_time.unwrap_or_default(),
            )
        })
        .ok()
    }

    fn exit_initialization_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe { self.binding.fmi3ExitInitializationMode(self.ptr) }).ok()
    }

    fn enter_event_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe { self.binding.fmi3EnterEventMode(self.ptr) }).ok()
    }

    fn terminate(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe { self.binding.fmi3Terminate(self.ptr) }).ok()
    }

    fn reset(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe { self.binding.fmi3Reset(self.ptr) }).ok()
    }

    #[cfg(false)]
    fn get_fmu_state<T>(
        &mut self,
        state: Option<Fmu3State<'_, T>>,
    ) -> Result<Fmu3State<'_, T>, Error> {
        unsafe { self.binding.fmi3GetFMUState(self.instance, FMUState) }
    }

    fn update_discrete_states(
        &mut self,
        event_flags: &mut EventFlags,
    ) -> Result<Fmi3Res, Fmi3Error> {
        let mut next_event_time_defined = event_flags.next_event_time.is_some();
        let mut next_event_time_value = event_flags.next_event_time.unwrap_or_default();

        let status: Fmi3Status = unsafe {
            self.binding.fmi3UpdateDiscreteStates(
                self.ptr,
                &mut event_flags.discrete_states_need_update as _,
                &mut event_flags.terminate_simulation as _,
                &mut event_flags.nominals_of_continuous_states_changed as _,
                &mut event_flags.values_of_continuous_states_changed as _,
                &mut next_event_time_defined as _,
                &mut next_event_time_value as _,
            )
        }
        .into();

        event_flags.next_event_time = if next_event_time_defined {
            Some(next_event_time_value)
        } else {
            None
        };

        status.ok()
    }

    fn get_number_of_variable_dependencies(
        &mut self,
        vr: binding::fmi3ValueReference,
    ) -> Result<usize, Fmi3Error> {
        let mut n_dependencies: usize = 0;
        Fmi3Status::from(unsafe {
            self.binding.fmi3GetNumberOfVariableDependencies(
                self.ptr,
                vr.into(),
                &mut n_dependencies as *mut usize,
            )
        })
        .ok()?;
        Ok(n_dependencies)
    }

    fn get_variable_dependencies(
        &mut self,
        dependent: binding::fmi3ValueReference,
    ) -> Result<Vec<VariableDependency>, Fmi3Error> {
        let n_dependencies = self.get_number_of_variable_dependencies(dependent)?;

        if n_dependencies == 0 {
            return Ok(Vec::new());
        }

        let mut element_indices_of_dependent = vec![MaybeUninit::<usize>::uninit(); n_dependencies];
        let mut independents =
            vec![MaybeUninit::<binding::fmi3ValueReference>::uninit(); n_dependencies];
        let mut element_indices_of_independents =
            vec![MaybeUninit::<usize>::uninit(); n_dependencies];
        let mut dependency_kinds =
            vec![MaybeUninit::<binding::fmi3DependencyKind>::uninit(); n_dependencies];

        Fmi3Status::from(unsafe {
            self.binding.fmi3GetVariableDependencies(
                self.ptr,
                dependent.into(),
                element_indices_of_dependent.as_mut_ptr() as *mut usize,
                independents.as_mut_ptr() as *mut binding::fmi3ValueReference,
                element_indices_of_independents.as_mut_ptr() as *mut usize,
                dependency_kinds.as_mut_ptr() as *mut binding::fmi3DependencyKind,
                n_dependencies,
            )
        })
        .ok()?;

        // Convert MaybeUninit arrays to initialized values
        let element_indices_of_dependent: Vec<usize> = element_indices_of_dependent
            .into_iter()
            .map(|d| unsafe { d.assume_init() })
            .collect();
        let independents: Vec<binding::fmi3ValueReference> = independents
            .into_iter()
            .map(|d| unsafe { d.assume_init() })
            .collect();
        let element_indices_of_independents: Vec<usize> = element_indices_of_independents
            .into_iter()
            .map(|d| unsafe { d.assume_init() })
            .collect();
        let dependency_kinds: Vec<binding::fmi3DependencyKind> = dependency_kinds
            .into_iter()
            .map(|k| unsafe { k.assume_init() })
            .collect();

        // Combine into VariableDependency structs
        let result = element_indices_of_dependent
            .into_iter()
            .zip(independents.into_iter())
            .zip(element_indices_of_independents.into_iter())
            .zip(dependency_kinds.into_iter())
            .map(
                |(((dep_idx, indep_vr), indep_idx), kind)| crate::fmi3::VariableDependency {
                    dependent_element_index: dep_idx,
                    independent: indep_vr,
                    independent_element_index: indep_idx,
                    dependency_kind: kind,
                },
            )
            .collect();

        Ok(result)
    }
}
