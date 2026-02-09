use fmi::{
    fmi3::{Common, import::Fmi3Import},
    traits::{FmiImport, FmiInstance},
};

use crate::{
    Error,
    options::{CoSimulationOptions, ModelExchangeOptions, OutputFormat},
    sim::{
        InputState, SimState, SimStateTrait,
        output::{ArrowIpcSink, CsvSink, NullSink, OutputRecorder, build_output_plan},
        params::SimParams,
        traits::{ImportSchemaBuilder, SimInitialize},
    },
};

use super::{
    SimStats,
    io::StartValues,
    traits::{FmiSim, InstSetValues},
};

#[cfg(feature = "cs")]
mod cs;
mod io;
#[cfg(feature = "me")]
mod me;
mod schema;

macro_rules! impl_sim_apply_start_values {
    ($inst:ty) => {
        impl super::traits::SimApplyStartValues<$inst> for super::SimState<$inst> {
            fn apply_start_values(
                &mut self,
                start_values: &StartValues<<$inst as FmiInstance>::ValueRef>,
            ) -> Result<(), Error> {
                if !start_values.structural_parameters.is_empty() {
                    self.inst
                        .enter_configuration_mode()
                        .map_err(fmi::Error::from)?;
                    for (vr, ary) in &start_values.structural_parameters {
                        //log::trace!("Setting structural parameter `{}`", (*vr).into());
                        self.inst.set_array(&[(*vr)], ary);
                    }
                    self.inst
                        .exit_configuration_mode()
                        .map_err(fmi::Error::from)?;
                }

                start_values.variables.iter().for_each(|(vr, ary)| {
                    self.inst.set_array(&[*vr], ary);
                });

                Ok(())
            }
        }
    };
}

#[cfg(feature = "me")]
impl_sim_apply_start_values!(fmi::fmi3::instance::InstanceME);
#[cfg(feature = "cs")]
impl_sim_apply_start_values!(fmi::fmi3::instance::InstanceCS);

impl FmiSim for Fmi3Import {
    #[cfg(feature = "me")]
    fn simulate_me(
        &self,
        options: &ModelExchangeOptions,
        output: &crate::options::OutputOptions,
        input_data: Option<arrow::array::RecordBatch>,
    ) -> Result<SimStats, Error> {
        use crate::sim::{solver, traits::SimMe};
        use fmi::fmi3::{ModelExchange, instance::InstanceME};

        let sim_params =
            SimParams::new_from_options(&options.common, self.model_description(), true, false);

        let start_values = self.parse_start_values(&options.common.initial_values)?;
        let input_state = InputState::new(self, input_data)?;
        let output_vrs: Vec<_> = self.outputs().map(|(_, vr)| vr).collect();
        let plan = build_output_plan(self, &output_vrs, output)?;
        let sink: Box<dyn crate::sim::output::OutputSink> =
            if let Some(path) = output.output_path.as_ref() {
                match output.output_format {
                    OutputFormat::ArrowIpc => Box::new(ArrowIpcSink::new(path, plan.schema.clone())?),
                    OutputFormat::Csv => Box::new(CsvSink::new(path, plan.schema.clone())?),
                    #[cfg(feature = "mcap")]
                    OutputFormat::Mcap => Box::new(crate::sim::output::McapSink::new(
                        path,
                        plan.schema.clone(),
                        crate::sim::output::resolve_terminal_channel_bindings(
                            &plan.columns,
                            &plan.terminal_bindings,
                        ),
                    )?),
                }
            } else {
                Box::new(NullSink)
            };
        let recorder_state: OutputRecorder<fmi::fmi3::instance::InstanceME> =
            OutputRecorder::from_plan(plan, sink)?;

        let start_time = sim_params.start_time;
        let tol = sim_params.tolerance.unwrap_or_default();

        let mut sim_state =
            SimState::<InstanceME>::new(self, sim_params, input_state, recorder_state)?;

        let nx = sim_state
            .inst
            .get_number_of_continuous_states()
            .map_err(|e| Error::from(fmi::Error::from(e)))?;
        let nz = sim_state
            .inst
            .get_number_of_event_indicators()
            .map_err(|e| Error::from(fmi::Error::from(e)))?;

        let solver: solver::Euler = solver::Solver::<InstanceME>::new(start_time, tol, nx, nz, ());

        sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
        let stats = sim_state.main_loop(solver)?;
        sim_state.recorder_state.finish()?;

        Ok(stats)
    }

    #[cfg(feature = "cs")]
    fn simulate_cs(
        &self,
        options: &CoSimulationOptions,
        output: &crate::options::OutputOptions,
        input_data: Option<arrow::array::RecordBatch>,
    ) -> Result<SimStats, Error> {
        use fmi::fmi3::instance::InstanceCS;

        let sim_params = SimParams::new_from_options(
            &options.common,
            self.model_description(),
            options.event_mode_used,
            options.early_return_allowed,
        );

        let start_values = self.parse_start_values(&options.common.initial_values)?;
        let input_state = InputState::new(self, input_data)?;
        let output_vrs: Vec<_> = self.outputs().map(|(_, vr)| vr).collect();
        let plan = build_output_plan(self, &output_vrs, output)?;
        let sink: Box<dyn crate::sim::output::OutputSink> =
            if let Some(path) = output.output_path.as_ref() {
                match output.output_format {
                    OutputFormat::ArrowIpc => Box::new(ArrowIpcSink::new(path, plan.schema.clone())?),
                    OutputFormat::Csv => Box::new(CsvSink::new(path, plan.schema.clone())?),
                    #[cfg(feature = "mcap")]
                    OutputFormat::Mcap => Box::new(crate::sim::output::McapSink::new(
                        path,
                        plan.schema.clone(),
                        crate::sim::output::resolve_terminal_channel_bindings(
                            &plan.columns,
                            &plan.terminal_bindings,
                        ),
                    )?),
                }
            } else {
                Box::new(NullSink)
            };
        let output_state: OutputRecorder<fmi::fmi3::instance::InstanceCS> =
            OutputRecorder::from_plan(plan, sink)?;

        let mut sim_state =
            SimState::<InstanceCS>::new(self, sim_params, input_state, output_state)?;
        sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
        let stats = sim_state.main_loop()?;
        sim_state.recorder_state.finish()?;

        Ok(stats)
    }
}
