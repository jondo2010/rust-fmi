use super::ModelInstance;
use crate::fmi3::{Model, ModelState, traits::ModelLoggingCategory};
use fmi::{
    EventFlags, InterfaceType,
    fmi3::{Common, Fmi3Error, Fmi3Res, binding},
};

impl<F> Common for ModelInstance<F>
where
    F: Model<ValueRef = binding::fmi3ValueReference>,
{
    fn get_version(&self) -> &str {
        // Safety: binding::fmi3Version is a null-terminated byte array representing the version string
        unsafe { str::from_utf8_unchecked(binding::fmi3Version) }
    }

    fn set_debug_logging(
        &mut self,
        logging_on: bool,
        categories: &[&str],
    ) -> Result<Fmi3Res, Fmi3Error> {
        for &cat in categories.iter() {
            if let Some(cat) = cat
                .parse::<F::LoggingCategory>()
                .ok()
                .and_then(|level| self.context.logging_on.get_mut(&level))
            {
                *cat = logging_on;
            } else {
                self.context.log(
                    Fmi3Error::Error,
                    F::LoggingCategory::default(),
                    format_args!("Unknown logging category {cat}"),
                );
                return Err(Fmi3Error::Error);
            }
        }
        Ok(Fmi3Res::OK)
    }

    fn enter_initialization_mode(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK,
            Default::default(),
            format_args!(
                "enter_initialization_mode(tolerance: {tolerance:?}, start_time: {start_time:?}, stop_time: {stop_time:?})",
            ),
        );

        match self.state {
            ModelState::Instantiated => {
                self.state = ModelState::InitializationMode;
                Ok(Fmi3Res::OK)
            }
            _ => {
                self.context.log(
                    Fmi3Error::Error,
                    F::LoggingCategory::default(),
                    format_args!(
                        "enter_initialization_mode() called in invalid state {:?}",
                        self.state
                    ),
                );
                Err(Fmi3Error::Error)
            }
        }
    }

    fn exit_initialization_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK,
            Default::default(),
            format_args!("exit_initialization_mode()"),
        );

        match self.interface_type() {
            InterfaceType::ModelExchange => {
                self.state = ModelState::EventMode;
            }
            InterfaceType::CoSimulation => {
                //TODO support event mode switch
                let event_mode_used = false;
                if event_mode_used {
                    self.state = ModelState::EventMode;
                } else {
                    self.state = ModelState::StepMode;
                }
            }
            InterfaceType::ScheduledExecution => {
                self.state = ModelState::ClockActivationMode;
            }
        }

        Ok(Fmi3Res::OK)
    }

    fn terminate(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.context
            .log(Fmi3Res::OK, Default::default(), format_args!("terminate()"));
        self.state = ModelState::Terminated;
        Ok(Fmi3Res::OK)
    }

    fn reset(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.state = ModelState::Instantiated;
        self.context.time = 0.0;
        self.model.set_start_values();
        Ok(Fmi3Res::OK)
    }

    fn enter_configuration_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        todo!()
    }

    fn exit_configuration_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        todo!()
    }

    fn enter_event_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK,
            F::LoggingCategory::trace_category(),
            format_args!("enter_event_mode()"),
        );
        self.state = ModelState::EventMode;
        Ok(Fmi3Res::OK)
    }

    fn update_discrete_states(
        &mut self,
        event_flags: &mut EventFlags,
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK,
            F::LoggingCategory::trace_category(),
            format_args!("update_discrete_states()"),
        );
        self.model.event_update(&self.context, event_flags)
    }

    fn get_number_of_variable_dependencies(
        &mut self,
        _vr: Self::ValueRef,
    ) -> Result<usize, Fmi3Error> {
        // Default implementation: no dependencies
        Ok(0)
    }

    fn get_variable_dependencies(
        &mut self,
        _dependent: Self::ValueRef,
    ) -> Result<Vec<fmi::fmi3::VariableDependency<Self::ValueRef>>, Fmi3Error> {
        // Default implementation: no dependencies
        Ok(Vec::new())
    }
}
