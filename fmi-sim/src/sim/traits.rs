use std::path::Path;

use arrow::{
    array::{ArrayRef, RecordBatch},
    datatypes::{Field, Schema},
};
use fmi::traits::{FmiImport, FmiInstance};

use crate::{
    Error,
    options::{CoSimulationOptions, ModelExchangeOptions},
};

use super::{
    RecorderState, SimStats,
    interpolation::{Interpolate, PreLookup},
    io::StartValues,
    solver::Solver,
};

/// Interface for building the Arrow schema for the inputs and outputs of an FMU.
pub trait ImportSchemaBuilder: FmiImport {
    /// Build the schema for the inputs of the model.
    fn inputs_schema(&self) -> Schema;
    /// Build the schema for the outputs of the model.
    fn outputs_schema(&self) -> Schema;
    /// Build a list of (Field, ValueReference) for the continuous inputs.
    fn continuous_inputs(&self) -> impl Iterator<Item = (Field, Self::ValueRef)> + '_;
    /// Build a list of Schema column (index, ValueReference) for the discrete inputs.
    fn discrete_inputs(&self) -> impl Iterator<Item = (Field, Self::ValueRef)> + '_;
    /// Build a list of Schema column (index, ValueReference) for the outputs.
    fn outputs(&self) -> impl Iterator<Item = (Field, Self::ValueRef)> + '_;
    /// Parse a list of "var=value" strings.
    ///
    /// # Returns
    /// A tuple of two lists of (ValueReference, Array) tuples. The first list contains any variable with
    /// `Causality = StructuralParameter` and the second list contains regular parameters.
    fn parse_start_values(
        &self,
        start_values: &[String],
    ) -> anyhow::Result<StartValues<Self::ValueRef>>;
}

pub trait InstSetValues: FmiInstance {
    fn set_array(
        &mut self,
        vrs: &[<Self as FmiInstance>::ValueRef],
        values: &arrow::array::ArrayRef,
    );
    fn set_interpolated<I: Interpolate>(
        &mut self,
        vr: <Self as FmiInstance>::ValueRef,
        pl: &PreLookup,
        array: &ArrayRef,
    ) -> anyhow::Result<()>;
}

pub trait InstRecordValues: FmiInstance + Sized {
    fn record_outputs(
        &mut self,
        time: f64,
        recorder: &mut RecorderState<Self>,
    ) -> anyhow::Result<()>;
}

/// Interface for handling events in the simulation.
/// Implemented by ME in fmi2 and ME+CS in fmi3.
pub trait SimHandleEvents {
    fn handle_events(
        &mut self,
        time: f64,
        input_event: bool,
        terminate_simulation: &mut bool,
    ) -> Result<bool, Error>;
}

pub trait SimMe<Inst> {
    /// Main loop of the model-exchange simulation
    fn main_loop<S>(&mut self, solver_params: S::Params) -> Result<SimStats, Error>
    where
        S: Solver<Inst>;
}

pub trait SimDefaultInitialize {
    fn default_initialize(&mut self) -> Result<(), Error>;
}

pub trait SimApplyStartValues<Inst: FmiInstance> {
    fn apply_start_values(
        &mut self,
        start_values: &StartValues<Inst::ValueRef>,
    ) -> Result<(), Error>;
}

pub trait SimInitialize<Inst: FmiInstance>: SimDefaultInitialize {
    fn initialize<P: AsRef<Path>>(
        &mut self,
        start_values: StartValues<Inst::ValueRef>,
        fmu_state_file: Option<P>,
    ) -> Result<(), Error>;
}

pub trait FmiSim: FmiImport + ImportSchemaBuilder {
    /// Simulate the model using Model Exchange.
    #[cfg(feature = "me")]
    fn simulate_me(
        &self,
        options: &ModelExchangeOptions,
        input_data: Option<RecordBatch>,
    ) -> Result<(RecordBatch, SimStats), Error>;

    /// Simulate the model using Co-Simulation.
    #[cfg(feature = "cs")]
    fn simulate_cs(
        &self,
        options: &CoSimulationOptions,
        input_data: Option<RecordBatch>,
    ) -> Result<(RecordBatch, SimStats), Error>;
}
