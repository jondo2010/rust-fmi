use crate::{
    fmi3::{binding, model, FmiStatus},
    FmiError, FmiResult,
};

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
    model: &'a model::ModelDescription,

    _tag: std::marker::PhantomData<Tag>,
}

impl<'a, Tag> Drop for Instance<'a, Tag> {
    fn drop(&mut self) {
        unsafe {
            self.binding.fmi3FreeInstance(self.instance);
        }
    }
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

    fn set_debug_logging(
        &mut self,
        logging_on: bool,
        //categories: &[model::LogCategoryKey],
        categories: impl Iterator<Item = model::LogCategoryKey>,
    ) -> FmiResult<()> {
        let cats_vec = categories
            .map(|cat| self.model.log_categories[cat].name.as_str().as_ptr())
            .collect::<Vec<_>>();

        let res: FmiStatus = unsafe {
            self.binding.fmi3SetDebugLogging(
                self.instance,
                logging_on,
                cats_vec.len() as _,
                cats_vec.as_slice().as_ptr() as _,
            )
        }
        .into();

        Ok(())
    }

    fn reset(&mut self) -> FmiResult<()> {
        let res: FmiStatus = unsafe { self.binding.fmi3Reset(self.instance) }.into();
        match res {
            FmiStatus::Ok => Ok(()),
            FmiStatus::Error => Err(FmiError::FmiStatusError),
            FmiStatus::Discard => Err(FmiError::FmiStatusDiscard),
            FmiStatus::Fatal => Err(FmiError::FmiStatusFatal),
            //FmiStatus::Warning => Err(FmiError::FmiStatusWarning),
            _ => unreachable!("Invalid status"),
        }
    }

    fn enter_initialization_mode(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> FmiResult<()> {
        let res: FmiStatus = unsafe {
            self.binding.fmi3EnterInitializationMode(
                self.instance,
                tolerance.is_some(),
                tolerance.unwrap_or_default(),
                start_time,
                stop_time.is_some(),
                stop_time.unwrap_or_default(),
            )
        }
        .into();
        res.into()
    }

    fn exit_initialization_mode(&mut self) -> FmiResult<()> {
        let res: FmiStatus =
            unsafe { self.binding.fmi3ExitInitializationMode(self.instance) }.into();
        res.into()
    }
}
