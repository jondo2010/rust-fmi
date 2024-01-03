//! FMI 3.0 instance interface

use crate::FmiInstance;

use super::{binding, schema, Fmi3Status};

mod co_simulation;
mod scheduled_execution {}
mod common;
mod model_exchange;
mod traits;

pub use traits::{CoSimulation, Common, ModelExchange, ScheduledExecution};

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
    /// Model description
    model_description: &'a schema::FmiModelDescription,
    /// Instance name
    name: String,
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

impl<'a, Tag> FmiInstance for Instance<'a, Tag> {
    type ModelDescription = &'a schema::FmiModelDescription;

    fn name(&self) -> &str {
        &self.name
    }

    fn get_version(&self) -> &str {
        <Self as Common>::get_version(self)
    }

    fn model_description(&self) -> &Self::ModelDescription {
        &self.model_description
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

/// Return value of [`Common::update_discrete_states()`]
#[derive(Default, Debug, PartialEq)]
pub struct DiscreteStates {
    /// The importer must stay in Event Mode for another event iteration, starting a new
    /// super-dense time instant.
    pub discrete_states_need_update: bool,
    /// The FMU requests to stop the simulation and the importer must call [`Common::terminate()`].
    pub terminate_simulation: bool,
    /// At least one nominal value of the states has changed and can be inquired with
    /// [`ModelExchange::get_nominals_of_continuous_states()`]. This argument is only valid in
    /// Model Exchange.
    pub nominals_of_continuous_states_changed: bool,
    /// At least one continuous state has changed its value because it was re-initialized (see [https://fmi-standard.org/docs/3.0.1/#reinit]).
    pub values_of_continuous_states_changed: bool,
    /// The absolute time of the next time event 𝑇next. The importer must compute up to
    /// `next_event_time` (or if needed slightly further) and then enter Event Mode using
    /// [`Common::enter_event_mode()`]. The FMU must handle this time event during the Event
    /// Mode that is entered by the first call to [`Common::enter_event_mode()`], at or after
    /// `next_event_time`.
    pub next_event_time: Option<f64>,
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    #[test_log::test]
    #[cfg(feature = "fmi3")]
    fn test_instance() {
        use crate::{
            fmi3::instance::traits::{Common, ModelExchange},
            import::FmiImport as _,
            Import,
        };

        let import = Import::new("data/reference_fmus/3.0/BouncingBall.fmu")
            .unwrap()
            .as_fmi3()
            .unwrap();

        let mut inst1 = import.instantiate_me("inst1", true, true).unwrap();
        assert_eq!(inst1.get_version(), "3.0");
        let log_cats: Vec<_> = import
            .model_description()
            .log_categories
            .as_ref()
            .unwrap()
            .categories
            .iter()
            .map(|x| x.name.as_str())
            .collect();
        inst1.set_debug_logging(true, &log_cats).ok().unwrap();
        inst1
            .enter_initialization_mode(None, 0.0, None)
            .ok()
            .unwrap();
        inst1.exit_initialization_mode().ok().unwrap();
        inst1.set_time(1234.0).ok().unwrap();

        inst1.enter_continuous_time_mode().ok().unwrap();

        let states = (0..import
            .model_description()
            .model_structure
            .continuous_state_derivative
            .len())
            .map(|x| x as f64)
            .collect::<Vec<_>>();

        inst1.set_continuous_states(&states).ok().unwrap();
        let (enter_event_mode, terminate_simulation) =
            inst1.completed_integrator_step(false).unwrap();
        assert_eq!(enter_event_mode, false);
        assert_eq!(terminate_simulation, false);

        let mut ders = vec![0.0; states.len()];
        inst1
            .get_continuous_state_derivatives(ders.as_mut_slice())
            .ok()
            .unwrap();
        assert_eq!(ders, vec![1.0, -9.81]);
    }
}
