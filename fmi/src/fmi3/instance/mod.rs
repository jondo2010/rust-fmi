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

pub type InstanceME<'a> = Instance<'a, ME>;
pub type InstanceCS<'a> = Instance<'a, CS>;
pub type InstanceSE<'a> = Instance<'a, SE>;

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
