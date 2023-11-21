use std::ffi::CString;

use crate::{
    fmi3::{binding, import, logger::callback_log, model, FmiStatus},
    import::FmiImport,
    FmiError, FmiResult,
};

use super::{traits, Instance, ME};

impl<'a> Instance<'a, ME> {
    pub fn new(
        import: &'a import::Fmi3,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> FmiResult<Self> {
        let binding = import.raw_bindings()?;
        let model = import.model();

        let instance_name = CString::new(instance_name).expect("Invalid instance name");
        let instantiation_token = CString::new(model.instantiation_token.as_bytes())
            .expect("Invalid instantiation token");
        let resource_path =
            CString::new(import.resource_url().as_str()).expect("Invalid resource path");

        let instance = unsafe {
            binding.fmi3InstantiateModelExchange(
                instance_name.as_ptr() as binding::fmi3String,
                instantiation_token.as_ptr() as binding::fmi3String,
                resource_path.as_ptr() as binding::fmi3String,
                visible,
                logging_on,
                std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                Some(callback_log),
            )
        };

        if instance.is_null() {
            return Err(FmiError::Instantiation);
        }

        Ok(Self {
            binding,
            instance,
            model,
            _tag: std::marker::PhantomData,
        })
    }

    fn get_values(
        &mut self,
        variables: &[model::VariableKey],
        //values: &mut [T],
    ) -> FmiResult<()> {
        variables.iter().map(|&key| {
            let var = self
                .model
                .model_variables
                .get(key)
                .expect("Invalid variable key");
        });

        //assert_eq!(value_references.len(), values.len());

        /*
        let res: FmiStatus = unsafe {
            self.binding.fmi3GetFloat32Real(
                self.instance,
                value_references.as_ptr(),
                value_references.len(),
                values.as_mut_ptr() as *mut f32,
                values.len(),
            )
        }
        .into();
        res.into()
        */
        todo!()
    }
}

impl<'a> traits::ModelExchange for Instance<'a, ME> {
    fn enter_continuous_time_mode(&mut self) -> FmiResult<()> {
        let res: FmiStatus =
            unsafe { self.binding.fmi3EnterContinuousTimeMode(self.instance) }.into();
        res.into()
    }

    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
    ) -> FmiResult<(bool, bool)> {
        let mut enter_event_mode = false;
        let mut terminate_simulation = false;
        let res: FmiStatus = unsafe {
            self.binding.fmi3CompletedIntegratorStep(
                self.instance,
                no_set_fmu_state_prior,
                &mut enter_event_mode,
                &mut terminate_simulation,
            )
        }
        .into();
        FmiResult::from(res).map(|_| (enter_event_mode, terminate_simulation))
    }

    fn set_time(&mut self, time: f64) -> FmiResult<()> {
        let res: FmiStatus = unsafe { self.binding.fmi3SetTime(self.instance, time) }.into();
        res.into()
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> FmiResult<()> {
        let res: FmiStatus = unsafe {
            self.binding
                .fmi3SetContinuousStates(self.instance, states.as_ptr(), states.len())
        }
        .into();
        res.into()
    }

    fn get_continuous_state_derivatives(&mut self, derivatives: &mut [f64]) -> FmiResult<()> {
        assert_eq!(
            derivatives.len(),
            self.model
                .model_structure
                .continuous_state_derivatives
                .len()
        );

        let res: FmiStatus = unsafe {
            self.binding.fmi3GetContinuousStateDerivatives(
                self.instance,
                derivatives.as_mut_ptr(),
                derivatives.len(),
            )
        }
        .into();
        res.into()
    }
}
