use fmi::fmi2::import::Fmi2Import;

use crate::{
    Error,
    options::{CoSimulationOptions, ModelExchangeOptions, OutputFormat},
    sim::{
        InputState, SimState, SimStateTrait,
        output::{ArrowIpcSink, CsvSink, NullSink, OutputRecorder, build_output_plan},
        traits::{ImportSchemaBuilder, SimInitialize},
    },
};

use super::{SimStats, params::SimParams, traits::FmiSim};

#[cfg(feature = "cs")]
mod cs;
mod io;
#[cfg(feature = "me")]
mod me;
mod schema;

impl FmiSim for Fmi2Import {
    #[cfg(feature = "me")]
    fn simulate_me(
        &self,
        options: &ModelExchangeOptions,
        output: &crate::options::OutputOptions,
        input_data: Option<arrow::array::RecordBatch>,
    ) -> Result<SimStats, Error> {
        use crate::sim::{solver, traits::SimMe};
        use fmi::{fmi2::instance::InstanceME, traits::FmiImport};

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
                }
            } else {
                Box::new(NullSink)
            };
        let recorder_state: OutputRecorder<fmi::fmi2::instance::Instance<fmi::ME>> =
            OutputRecorder::from_plan(plan, sink)?;

        let nx = self.model_description().num_states();
        let nz = self.model_description().num_event_indicators();

        let solver: solver::Euler = solver::Solver::<InstanceME>::new(
            sim_params.start_time,
            sim_params.tolerance.unwrap_or_default(),
            nx,
            nz,
            (),
        );

        let mut sim_state =
            SimState::<InstanceME>::new(self, sim_params, input_state, recorder_state)?;
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
        use fmi::{fmi2::instance::InstanceCS, traits::FmiImport};

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
                }
            } else {
                Box::new(NullSink)
            };
        let recorder_state: OutputRecorder<fmi::fmi2::instance::Instance<fmi::CS>> =
            OutputRecorder::from_plan(plan, sink)?;

        let mut sim_state =
            SimState::<InstanceCS>::new(self, sim_params, input_state, recorder_state)?;
        sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
        let stats = sim_state.main_loop().map_err(fmi::Error::from)?;
        sim_state.recorder_state.finish()?;

        Ok(stats)
    }
}
