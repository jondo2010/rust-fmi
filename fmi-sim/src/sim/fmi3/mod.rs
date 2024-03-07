use std::path::Path;

use anyhow::Context;
use fmi::{fmi3::instance::Common, traits::FmiInstance};

use super::{
    interpolation::Linear,
    io::StartValues,
    solver::Solver,
    traits::{FmiSchemaBuilder, InstanceSetValues, SimOutput},
    SimState,
};

#[cfg(feature = "cs")]
mod cs;
mod io;
#[cfg(feature = "me")]
mod me;
mod schema;

#[cfg(feature = "cs")]
pub use cs::co_simulation;
#[cfg(feature = "me")]
pub use me::model_exchange;

impl<Inst, S> SimState<Inst, S>
where
    Inst: FmiInstance + InstanceSetValues,
    Inst::Import: FmiSchemaBuilder,
    S: Solver<Inst>,
{
}

impl<Inst, S> SimState<Inst, S>
where
    Inst: Common,
    Inst::Import: FmiSchemaBuilder,
    S: Solver<Inst>,
{
    fn apply_start_values(
        &mut self,
        start_values: &StartValues<Inst::ValueReference>,
    ) -> anyhow::Result<()> {
        if !start_values.structural_parameters.is_empty() {
            self.inst.enter_configuration_mode().ok()?;
            for (vr, ary) in &start_values.structural_parameters {
                log::trace!("Setting structural parameter `{}`", (*vr).into());
                self.inst.set_array(&[(*vr)], &ary);
            }
            self.inst.exit_configuration_mode().ok()?;
        }

        start_values.variables.iter().for_each(|(vr, ary)| {
            self.inst.set_array(&[*vr], ary);
        });

        Ok(())
    }

    fn initialize<P>(
        &mut self,
        start_values: StartValues<Inst::ValueReference>,
        initial_fmu_state_file: Option<P>,
    ) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        if let Some(_initial_state_file) = &initial_fmu_state_file {
            unimplemented!("initial_fmu_state_file");
            // self.inst.restore_fmu_state_from_file(initial_state_file)?;
        }

        // set start values
        self.apply_start_values(&start_values)?;

        self.input_state.apply_input::<Linear>(
            self.sim_params.start_time,
            &mut self.inst,
            true,
            true,
            false,
        )?;

        // Default initialization
        if initial_fmu_state_file.is_none() {
            self.default_initialize()?;
        }

        Ok(())
    }

    fn default_initialize(&mut self) -> anyhow::Result<()> {
        self.inst
            .enter_initialization_mode(
                self.sim_params.tolerance,
                self.time,
                Some(self.sim_params.stop_time),
            )
            .ok()
            .context("enter_initialization_mode")?;

        self.inst
            .exit_initialization_mode()
            .ok()
            .context("exit_initialization_mode")?;

        if self.sim_params.event_mode_used {
            // update discrete states
            let mut discrete_states_need_update = true;
            let mut nominals_of_continuous_states_changed = false;
            let mut values_of_continuous_states_changed = false;
            while discrete_states_need_update {
                let mut terminate_simulation = false;

                self.inst
                    .update_discrete_states(
                        &mut discrete_states_need_update,
                        &mut terminate_simulation,
                        &mut nominals_of_continuous_states_changed,
                        &mut values_of_continuous_states_changed,
                        &mut self.next_event_time,
                    )
                    .ok()
                    .context("update_discrete_states")?;

                if terminate_simulation {
                    self.inst.terminate().ok().context("terminate")?;
                    anyhow::bail!("update_discrete_states() requested termination.");
                }
            }
        }

        Ok(())
    }

    fn handle_events(
        &mut self,
        input_event: bool,
        terminate_simulation: &mut bool,
    ) -> Result<bool, anyhow::Error> {
        self.output_state
            .record_outputs(self.time, &mut self.inst)?;
        self.inst
            .enter_event_mode()
            .ok()
            .context("enter_event_mode")?;
        if input_event {
            self.input_state
                .apply_input::<Linear>(self.time, &mut self.inst, true, true, true)?;
        }
        let mut reset_solver = false;
        let mut discrete_states_need_update = true;
        let mut nominals_of_continuous_states_changed = false;
        let mut values_of_continuous_states_changed = false;
        while discrete_states_need_update {
            self.inst
                .update_discrete_states(
                    &mut discrete_states_need_update,
                    terminate_simulation,
                    &mut nominals_of_continuous_states_changed,
                    &mut values_of_continuous_states_changed,
                    &mut self.next_event_time,
                )
                .ok()
                .context("update_discrete_states")?;

            if *terminate_simulation {
                break;
            }
            reset_solver |=
                nominals_of_continuous_states_changed || values_of_continuous_states_changed;
        }
        Ok(reset_solver)
    }
}

#[cfg(feature = "disable")]
impl<'a, Inst, S> SimState<Inst, S>
where
    Inst: FmiInstance + Common + Model,
    Inst::Import: FmiSchemaBuilder,
    S: Solver<Inst>,
    Self: SimTrait<
        'a,
        Import = Inst::Import,
        InputState = InputState<Inst>,
        OutputState = OutputState<Inst>,
    >,
{
    fn new_from_options(
        import: &'a Inst::Import,
        options: &CommonOptions,
        event_mode_used: bool,
        early_return_allowed: bool,
    ) -> anyhow::Result<Self> {
        let sim_params = SimParams::new_from_options(
            options,
            import.model_description(),
            event_mode_used,
            early_return_allowed,
        )?;

        // Read optional input data from file
        let input_data = options
            .input_file
            .as_ref()
            .map(super::util::read_csv)
            .transpose()
            .context("Reading input file")?;

        let input_state = InputState::new(import, input_data)?;
        let output_state = OutputState::new(import, &sim_params);

        SimState::<Inst>::new(import, sim_params, input_state, output_state)
    }
}
