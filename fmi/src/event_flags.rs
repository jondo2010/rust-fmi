/// Event flags used in calls to `update_discrete_states`/`new_discrete_states`
#[derive(Default, Debug, PartialEq)]
pub struct EventFlags {
    /// The importer must stay in Event Mode for another event iteration, starting a new
    /// super-dense time instant.
    pub discrete_states_need_update: bool,
    /// The FMU requests to stop the simulation and the importer must call [`Common::terminate()`].
    pub terminate_simulation: bool,
    /// At least one nominal value of the states has changed and can be inquired with
    /// [`crate::fmi3::ModelExchange::get_nominals_of_continuous_states()`]. This argument is only valid in
    /// Model Exchange.
    pub nominals_of_continuous_states_changed: bool,
    /// At least one continuous state has changed its value because it was re-initialized (see <https://fmi-standard.org/docs/3.0.1/#reinit>).
    pub values_of_continuous_states_changed: bool,
    /// The absolute time of the next time event 𝑇next. The importer must compute up to
    /// `next_event_time` (or if needed slightly further) and then enter Event Mode using
    /// [`Common::enter_event_mode()`]. The FMU must handle this time event during the Event
    /// Mode that is entered by the first call to [`Common::enter_event_mode()`], at or after
    /// `next_event_time`.
    pub next_event_time: Option<f64>,
}

impl EventFlags {
    /// Reset all event flags to their default state.
    pub fn reset(&mut self) {
        self.discrete_states_need_update = false;
        self.terminate_simulation = false;
        self.nominals_of_continuous_states_changed = false;
        self.values_of_continuous_states_changed = false;
        self.next_event_time = None;
    }

    #[cfg(feature = "fmi2")]
    /// Update the event flags from the FMI2 event information.
    pub(crate) fn update_from_fmi2_event_info(
        &mut self,
        event_info: crate::fmi2::binding::fmi2EventInfo,
    ) {
        self.discrete_states_need_update = event_info.newDiscreteStatesNeeded != 0;
        self.terminate_simulation = event_info.terminateSimulation != 0;
        self.nominals_of_continuous_states_changed =
            event_info.nominalsOfContinuousStatesChanged != 0;
        self.values_of_continuous_states_changed = event_info.valuesOfContinuousStatesChanged != 0;
        self.next_event_time = if event_info.nextEventTimeDefined != 0 {
            Some(event_info.nextEventTime)
        } else {
            None
        };
    }
}
