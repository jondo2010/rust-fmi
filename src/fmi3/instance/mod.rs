use crate::{
    fmi3::{binding, FmiStatus},
    Error,
};

use super::schema;

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

impl<'a, Tag> traits::Common for Instance<'a, Tag> {
    fn name(&self) -> &str {
        &self.model.model_name
    }

    fn get_version(&self) -> &str {
        unsafe { std::ffi::CStr::from_ptr(self.binding.fmi3GetVersion()) }
            .to_str()
            .expect("Invalid version string")
    }

    #[cfg(feature = "disabled")]
    fn set_debug_logging(
        &mut self,
        logging_on: bool,
        //categories: &[model::LogCategoryKey],
        categories: impl Iterator<Item = model::LogCategoryKey>,
    ) -> FmiResult<()> {
        let cats_vec = categories
            .map(|cat| {
                let cat_name = &self.model.log_categories[cat].name;
                std::ffi::CString::new(cat_name.as_bytes()).expect("Error building CString")
            })
            .collect::<Vec<_>>();

        let res: FmiStatus = unsafe {
            self.binding.fmi3SetDebugLogging(
                self.instance,
                logging_on,
                cats_vec.len() as _,
                cats_vec.as_ptr() as _,
            )
        }
        .into();

        Ok(())
    }

    fn enter_initialization_mode(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Result<(), Error> {
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

    fn exit_initialization_mode(&mut self) -> Result<(), Error> {
        let res: FmiStatus =
            unsafe { self.binding.fmi3ExitInitializationMode(self.instance) }.into();
        res.into()
    }

    fn enter_event_mode(&mut self) -> Result<(), Error> {
        let res: FmiStatus = unsafe { self.binding.fmi3EnterEventMode(self.instance) }.into();
        res.into()
    }

    fn terminate(&mut self) -> Result<(), Error> {
        let res: FmiStatus = unsafe { self.binding.fmi3Terminate(self.instance) }.into();
        res.into()
    }

    fn reset(&mut self) -> Result<(), Error> {
        let res: FmiStatus = unsafe { self.binding.fmi3Reset(self.instance) }.into();
        res.into()
    }
}
