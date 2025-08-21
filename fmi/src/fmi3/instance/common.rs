use std::mem::MaybeUninit;

use crate::{
    fmi3::{
        binding,
        import::Fmi3Import,
        instance::Instance,
        traits::{Common, GetSet},
        Fmi3Error, Fmi3Status,
    },
    traits::{FmiImport, FmiStatus},
};

macro_rules! impl_getter_setter {
    ($ty:ty, $get:ident, $set:ident, $fmi_get:ident, $fmi_set:ident) => {
        fn $get(&mut self, vrs: &[binding::fmi3ValueReference], values: &mut [$ty]) -> Fmi3Status {
            unsafe {
                self.binding.$fmi_get(
                    self.ptr,
                    vrs.as_ptr(),
                    vrs.len() as _,
                    values.as_mut_ptr(),
                    values.len() as _,
                )
            }
            .into()
        }

        fn $set(&mut self, vrs: &[binding::fmi3ValueReference], values: &[$ty]) -> Fmi3Status {
            unsafe {
                self.binding.$fmi_set(
                    self.ptr,
                    vrs.as_ptr(),
                    vrs.len() as _,
                    values.as_ptr(),
                    values.len() as _,
                )
            }
            .into()
        }
    };
}

impl<'a, Tag> GetSet for Instance<'a, Tag> {
    type ValueRef = <Fmi3Import as FmiImport>::ValueRef;
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
        vrs: &[Self::ValueRef],
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
        vrs: &[Self::ValueRef],
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
                        i, value_buffer.len(), fmu_size
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
}

impl<'a, Tag> Common for Instance<'a, Tag> {
    fn get_version(&self) -> &str {
        unsafe { std::ffi::CStr::from_ptr(self.binding.fmi3GetVersion()) }
            .to_str()
            .expect("Invalid version string")
    }

    fn set_debug_logging(&mut self, logging_on: bool, categories: &[&str]) -> Fmi3Status {
        let cats_vec = categories
            .iter()
            .map(|cat| std::ffi::CString::new(cat.as_bytes()).expect("Error building CString"))
            .collect::<Vec<_>>();

        let cats_vec_ptrs = cats_vec
            .iter()
            .map(|cat| cat.as_c_str().as_ptr())
            .collect::<Vec<_>>();

        unsafe {
            self.binding.fmi3SetDebugLogging(
                self.ptr,
                logging_on,
                cats_vec_ptrs.len() as _,
                cats_vec_ptrs.as_ptr() as *const binding::fmi3String,
            )
        }
        .into()
    }

    fn enter_configuration_mode(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3EnterConfigurationMode(self.ptr) }.into()
    }

    fn exit_configuration_mode(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3ExitConfigurationMode(self.ptr) }.into()
    }

    fn enter_initialization_mode(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Fmi3Status {
        unsafe {
            self.binding.fmi3EnterInitializationMode(
                self.ptr,
                tolerance.is_some(),
                tolerance.unwrap_or_default(),
                start_time,
                stop_time.is_some(),
                stop_time.unwrap_or_default(),
            )
        }
        .into()
    }

    fn exit_initialization_mode(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3ExitInitializationMode(self.ptr) }.into()
    }

    fn enter_event_mode(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3EnterEventMode(self.ptr) }.into()
    }

    fn terminate(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3Terminate(self.ptr) }.into()
    }

    fn reset(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3Reset(self.ptr) }.into()
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
        discrete_states_need_update: &mut bool,
        terminate_simulation: &mut bool,
        nominals_of_continuous_states_changed: &mut bool,
        values_of_continuous_states_changed: &mut bool,
        next_event_time: &mut Option<f64>,
    ) -> Fmi3Status {
        let mut next_event_time_defined = false;
        let mut next_event_time_value = 0.0;

        let status: Fmi3Status = unsafe {
            self.binding.fmi3UpdateDiscreteStates(
                self.ptr,
                discrete_states_need_update as _,
                terminate_simulation as _,
                nominals_of_continuous_states_changed as _,
                values_of_continuous_states_changed as _,
                &mut next_event_time_defined as _,
                &mut next_event_time_value as _,
            )
        }
        .into();

        *next_event_time = if next_event_time_defined {
            Some(next_event_time_value)
        } else {
            None
        };

        status
    }
}
