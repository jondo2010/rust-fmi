use crate::{
    fmi3::{binding, model},
    FmiResult, FmiStatus,
};

pub mod traits {
    use crate::{fmi3::model::LogCategoryKey, FmiResult, FmiStatus};

    /// Interface common to all instance types
    pub trait Common: std::hash::Hash {
        /// The instance name
        fn name(&self) -> &str;

        /// The FMI-standard version string
        fn get_version(&self) -> FmiResult<&str>;

        /// The function controls the debug logging that is output by the FMU
        fn set_debug_logging(&self, logging_on: bool, categories: &[LogCategoryKey]);

        /// Is called by the environment to reset the FMU after a simulation run.
        /// The FMU goes into the same state as if newly created. All variables have their default
        /// values. Before starting a new run [`enter_initialization_mode()`] has to be called.
        fn reset(&self) -> FmiResult<()>;

        /// Changes state to `Initialization Mode`.
        ///
        /// tolerance depend on the interface type:
        /// * Model Exchange: If toleranceDefined = fmi3True, then the model is called with a numerical integration scheme where the step size is controlled by using tolerance for error estimation (usually as relative tolerance). In such a case all numerical algorithms used inside the model (for example, to solve nonlinear algebraic equations) should also operate with an error estimation of an appropriate smaller relative tolerance.
        /// * Co-Simulation: If toleranceDefined = fmi3True, then the communication step size of the FMU is controlled by error estimation. In case the FMU utilizes a numerical integrator with variable step size and error estimation, it is suggested to use tolerance for the error estimation of the integrator (usually as relative tolerance).
        /// An FMU for Co-Simulation might ignore this argument.
        fn enter_initialization_mode_type(
            &self,
            tolerance: Option<f64>,
            start_time: f64,
            stop_time: Option<f64>,
        ) -> FmiResult<()>;
    }
}

pub struct Instance<'a> {
    /// Raw FMI 3.0 bindings
    binding: binding::Fmi3Binding,
    /// Pointer to the raw FMI 3.0 instance
    instance_ptr: binding::fmi3Instance,
    /// Derived model description
    model: &'a model::ModelDescription,
}

impl<'a> Instance<'a> {
    pub fn new(
        binding: binding::Fmi3Binding,
        instance: binding::fmi3Instance,
        model: &'a model::ModelDescription,
    ) -> Self {
        Self {
            binding,
            instance_ptr: instance,
            model,
        }
    }

    pub fn get_version(&self) -> &str {
        unsafe { std::ffi::CStr::from_ptr(self.binding.fmi3GetVersion()) }
            .to_str()
            .unwrap()
    }

    /// Set a new value for the independent variable (typically a time instant).
    pub fn set_time(&self, time: f64) -> FmiResult<()> {
        let res = unsafe { self.binding.fmi3SetTime(self.instance_ptr, time) };

        //match FmiStatus::from(res) {
        //    FmiStatus::Ok => Ok(()),
        //}

        Ok(())
    }
}

impl<'a> Drop for Instance<'a> {
    fn drop(&mut self) {
        unsafe {
            self.binding.fmi3FreeInstance(self.instance_ptr);
        }
    }
}
