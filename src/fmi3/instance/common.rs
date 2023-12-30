use crate::fmi3::{binding, Fmi3Status};

use super::{traits, DiscreteStates, Instance};

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
    fn name(&self) -> &str {
        &self.model.model_name
    }

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

    #[cfg(disabled)]
    fn get_fmu_state<T>(
        &mut self,
        state: Option<Fmu3State<'_, T>>,
    ) -> Result<Fmu3State<'_, T>, Error> {
        unsafe { self.binding.fmi3GetFMUState(self.instance, FMUState) }
    }

    fn update_discrete_states(&mut self, states: &mut DiscreteStates) -> Fmi3Status {
        let mut next_event_time_defined = false;
        let mut next_event_time = 0.0;

        let status: Fmi3Status = unsafe {
            self.binding.fmi3UpdateDiscreteStates(
                self.instance,
                &mut states.discrete_states_need_update as _,
                &mut states.terminate_simulation as _,
                &mut states.nominals_of_continuous_states_changed as _,
                &mut states.values_of_continuous_states_changed as _,
                &mut next_event_time_defined as _,
                &mut next_event_time as _,
            )
        }
        .into();

        if next_event_time_defined {
            states.next_event_time = Some(next_event_time);
        }

        status
    }
}
