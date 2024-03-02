use std::ffi::CString;

use crate::{
    fmi3::{binding, import, logger, Fmi3Status},
    traits::FmiImport,
    Error,
};

use super::{CoSimulation, Instance, CS};

impl<'a> Instance<'a, CS> {
    /// Returns a new CoSimulation instance.
    ///
    /// An FMU can be instantiated many times (provided capability flag
    /// `can_be_instantiated_only_once_per_process = false`).
    ///
    /// Arguments:
    /// * `instance_name`: a unique identifier for the FMU instance. It is used to name the
    ///   instance, for example, in error or information messages generated by one of the fmi3XXX
    ///   functions. The argument `instance_name` must be a non empty string (in other words, must
    ///   have at least one character that is not a white space). [If only one FMU is simulated,
    ///   either the attribute `model_name` of [`crate::fmi3::schema::FmiModelDescription`] or the
    ///   attribute `model_identifier` of <`ModelExchange|CoSimulation|ScheduledExecution`> can be
    ///   used as `instance_name`.]
    ///
    /// * `visible`: Defines that the interaction with the user should be reduced to a minimum (no
    ///   application window, no plotting, no animation, etc.). In other words, the FMU is executed
    ///   in batch mode. If `visible = true`, the FMU is executed in interactive mode, and the FMU
    ///   might require to explicitly acknowledge start of simulation / instantiation /
    ///   initialization (acknowledgment is non-blocking).
    ///
    /// * `logging_on`: If `= false`, then any logging is disabled and the logMessage callback
    ///   function must not be called by the FMU. If `logging_on = true`, then all <LogCategories>
    ///   are enabled. The function [`traits::Common::set_debug_logging`] gives more detailed
    ///   control about enabling specific <LogCategories>.
    ///
    /// * `event_mode_used`: If `= true` the importer can handle events. The flag may only be
    ///   `true`, if `has_event_mode = true`, otherwise the FMU must raise an error. For FMUs that
    ///   have clocks, `event_mode_used = true` is required.
    ///
    /// * `early_return_allowed`: If `= true` the importer can handle early return. Only in this
    ///   case, [`traits::CoSimulation::do_step()`] may return with `early_return = true`.
    ///
    /// * `required_intermediate_variables`: An array of the value references of all input variables
    ///   that the simulation algorithm intends to set and all output variables it intends to get
    ///   during intermediate updates. This set may be empty (nRequiredIntermediateVariables = 0)
    ///   when the simulation algorithm does not intend to use intermediate update. Only the
    ///   variables in requiredIntermediateVariables may be accessed by the simulation algorithm
    ///   using fmi3Set{VariableType} and fmi3Get{VariableType} during Intermediate Update Mode. All
    ///   variables referenced in this set must be marked with the attribute intermediateUpdate =
    ///   "true" in modelDescription.xml.
    ///
    /// See: https://fmi-standard.org/docs/3.0.1/#fmi3InstantiateCoSimulation
    pub fn new(
        import: &'a import::Fmi3Import,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
        event_mode_used: bool,
        early_return_allowed: bool,
        required_intermediate_variables: &[binding::fmi3ValueReference],
    ) -> Result<Self, Error> {
        let model_description = import.model_description();

        let name = instance_name.to_owned();

        let co_simulation = model_description
            .co_simulation
            .as_ref()
            .ok_or(Error::UnsupportedFmuType("CoSimulation".to_owned()))?;

        log::debug!(
            "Instantiating CS: {} '{name}'",
            co_simulation.model_identifier
        );

        let binding = import.binding(&co_simulation.model_identifier)?;

        let instance_name = CString::new(instance_name).expect("Invalid instance name");
        let instantiation_token = CString::new(model_description.instantiation_token.as_bytes())
            .expect("Invalid instantiation token");
        let resource_path =
            CString::new(import.resource_url().as_str()).expect("Invalid resource path");

        // instanceEnvironment is a pointer that must be passed to fmi3IntermediateUpdateCallback,
        // fmi3ClockUpdateCallback, and fmi3LogMessageCallback to allow the simulation environment
        // an efficient way to identify the calling FMU.

        let instance = unsafe {
            binding.fmi3InstantiateCoSimulation(
                instance_name.as_ptr() as binding::fmi3String,
                instantiation_token.as_ptr() as binding::fmi3String,
                resource_path.as_ptr() as binding::fmi3String,
                visible as binding::fmi3Boolean,
                logging_on as binding::fmi3Boolean,
                event_mode_used as binding::fmi3Boolean,
                early_return_allowed as binding::fmi3Boolean,
                required_intermediate_variables.as_ptr(),
                required_intermediate_variables.len() as _,
                std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                Some(logger::callback_log),
                // intermediateUpdate: fmi3IntermediateUpdateCallback,
                None,
            )
        };

        if instance.is_null() {
            return Err(Error::Instantiation);
        }

        Ok(Self {
            binding,
            ptr: instance,
            model_description,
            name,
            _tag: std::marker::PhantomData,
        })
    }
}

impl<'a> CoSimulation for Instance<'a, CS> {
    fn enter_step_mode(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3EnterStepMode(self.ptr) }.into()
    }

    fn do_step(
        &mut self,
        current_communication_point: f64,
        communication_step_size: f64,
        no_set_fmustate_prior_to_current_point: bool,
        event_handling_needed: &mut bool,
        terminate_simulation: &mut bool,
        early_return: &mut bool,
        last_successful_time: &mut f64,
    ) -> Fmi3Status {
        unsafe {
            self.binding.fmi3DoStep(
                self.ptr,
                current_communication_point,
                communication_step_size,
                no_set_fmustate_prior_to_current_point,
                event_handling_needed,
                terminate_simulation,
                early_return,
                last_successful_time,
            )
        }
        .into()
    }
}
