//! FMI 3.0 instance interface

use super::{binding, schema, Fmi3Status};

mod co_simulation {}
mod scheduled_execution {}
mod model_exchange;
pub mod traits;

/// Tag for Model Exchange instances
pub struct ME;
/// Tag for Co-Simulation instances
pub struct CS;
/// Tag for Scheduled Execution instances
pub struct SE;

pub struct Instance<'a, Tag> {
    /// Raw FMI 3.0 bindings
    binding: binding::Fmi3Binding,
    /// Pointer to the raw FMI 3.0 instance
    instance: binding::fmi3Instance,
    /// Derived model description
    model: &'a schema::FmiModelDescription,
    //model: &'a model::ModelDescription,
    _tag: std::marker::PhantomData<&'a Tag>,
}

impl<'a, Tag> Drop for Instance<'a, Tag> {
    fn drop(&mut self) {
        unsafe {
            log::trace!("Freeing instance {:?}", self.instance);
            self.binding.fmi3FreeInstance(self.instance);
        }
    }
}

pub struct Fmu3State<'a, Tag> {
    instance: Instance<'a, Tag>,
    /// Pointer to the raw FMI 3.0 state
    state: binding::fmi3FMUState,
}

impl<'a, Tag> Drop for Fmu3State<'a, Tag> {
    fn drop(&mut self) {
        unsafe {
            log::trace!("Freeing state {:?}", self.state);
            self.instance
                .binding
                .fmi3FreeFMUState(self.instance.instance, &mut self.state);
        }
    }
}

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

        unsafe {
            self.binding.fmi3SetDebugLogging(
                self.instance,
                logging_on,
                cats_vec.len() as _,
                cats_vec.as_ptr() as _,
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

    #[cfg(disabled)]
    fn get_fmu_state<T>(
        &mut self,
        state: Option<Fmu3State<'_, T>>,
    ) -> Result<Fmu3State<'_, T>, Error> {
        unsafe { self.binding.fmi3GetFMUState(self.instance, FMUState) }
    }

    fn update_discrete_states(&mut self) -> Result<traits::DiscreteStates, super::Fmi3Err> {
        let mut res = traits::DiscreteStates::default();

        let mut next_event_time_defined = false;
        let mut next_event_time = 0.0;

        let status: Fmi3Status = unsafe {
            self.binding.fmi3UpdateDiscreteStates(
                self.instance,
                &mut res.discrete_states_need_update as _,
                &mut res.terminate_simulation as _,
                &mut res.nominals_of_continuous_states_changed as _,
                &mut res.values_of_continuous_states_changed as _,
                &mut next_event_time_defined as _,
                &mut next_event_time as _,
            )
        }
        .into();

        if next_event_time_defined {
            res.next_event_time = Some(next_event_time);
        }

        status.ok().map(|_| res)
    }
}
