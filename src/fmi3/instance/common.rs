use std::mem::MaybeUninit;

use crate::fmi3::{binding, Fmi3Status};

use super::{traits, Instance};

macro_rules! impl_getter_setter {
    ($ty:ty, $get:ident, $set:ident, $fmi_get:ident, $fmi_set:ident) => {
        fn $get(&mut self, vrs: &[binding::fmi3ValueReference], values: &mut [$ty]) -> Fmi3Status {
            unsafe {
                self.binding.$fmi_get(
                    self.instance,
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
                    self.instance,
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

impl<'a, Tag> traits::Common for Instance<'a, Tag> {
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
                self.instance,
                logging_on,
                cats_vec_ptrs.len() as _,
                cats_vec_ptrs.as_ptr() as *const binding::fmi3String,
            )
        }
        .into()
    }

    fn enter_initialization_mode(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Fmi3Status {
        unsafe {
            self.binding.fmi3EnterInitializationMode(
                self.instance,
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
        unsafe { self.binding.fmi3ExitInitializationMode(self.instance) }.into()
    }

    fn enter_event_mode(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3EnterEventMode(self.instance) }.into()
    }

    fn terminate(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3Terminate(self.instance) }.into()
    }

    fn reset(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3Reset(self.instance) }.into()
    }

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
        values: &mut [String],
    ) -> Fmi3Status {
        unsafe {
            // Create a mutable vec of fmi3String with uninitialized values to pass to ffi
            let mut ret_values = MaybeUninit::<Vec<binding::fmi3String>>::uninit();
            let stat = self.binding.fmi3GetString(
                self.instance,
                vrs.as_ptr(),
                vrs.len() as _,
                ret_values.assume_init_mut().as_mut_ptr(),
                ret_values.assume_init_ref().len() as _,
            );
            for (v, ret) in ret_values
                .assume_init_ref()
                .into_iter()
                .zip(values.iter_mut())
            {
                *ret = std::ffi::CStr::from_ptr(*v)
                    .to_str()
                    .expect("Error converting C string")
                    .to_string();
            }
            stat
        }
        .into()
    }

    fn set_string<'b>(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: impl Iterator<Item = &'b str>,
    ) -> Fmi3Status {
        let values = values
            .map(|s| std::ffi::CString::new(s.as_bytes()).expect("Error building CString"))
            .collect::<Vec<_>>();

        let ptrs = values
            .iter()
            .map(|s| s.as_c_str().as_ptr())
            .collect::<Vec<_>>();

        unsafe {
            self.binding.fmi3SetString(
                self.instance,
                vrs.as_ptr(),
                vrs.len() as _,
                ptrs.as_ptr(),
                values.len() as _,
            )
        }
        .into()
    }

    fn get_binary(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &mut [Vec<u8>],
    ) -> Fmi3Status {
        assert_eq!(vrs.len(), values.len());
        let mut value_sizes = vec![0; values.len()];
        let mut value_ptrs = unsafe {
            // Safety: `MaybeUninit` is guaranteed to be initialized by `fmi3GetBinary`
            vec![MaybeUninit::<binding::fmi3Binary>::zeroed().assume_init(); values.len()]
        };

        let status: Fmi3Status = unsafe {
            self.binding.fmi3GetBinary(
                self.instance,
                vrs.as_ptr(),
                vrs.len() as _,
                value_sizes.as_mut_ptr(),
                value_ptrs.as_mut_ptr(),
                values.len() as _,
            )
        }
        .into();

        if status.is_error() {
            return status;
        }

        unsafe {
            // Copy the binary data into `values`
            // Safety: `value_sizes` and `value_ptrs` are guaranteed to be initialized by
            // `fmi3GetBinary`
            for (i, (size, ptr)) in value_sizes.iter().zip(value_ptrs.iter()).enumerate() {
                values[i] = std::slice::from_raw_parts(*ptr, *size).to_vec();
            }
        }
        status
    }

    fn set_binary<'b>(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: impl Iterator<Item = &'b [u8]>,
    ) -> Fmi3Status {
        let values = values.collect::<Vec<_>>();
        let value_sizes = values.iter().map(|v| v.len()).collect::<Vec<_>>();
        unsafe {
            self.binding.fmi3SetBinary(
                self.instance,
                vrs.as_ptr(),
                vrs.len() as _,
                value_sizes.as_ptr(),
                values.as_ptr() as *const binding::fmi3Binary,
                values.len() as _,
            )
        }
        .into()
    }

    #[cfg(disabled)]
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
                self.instance,
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
