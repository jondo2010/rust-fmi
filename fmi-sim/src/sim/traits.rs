use arrow::{
    array::{ArrayRef, RecordBatch},
    datatypes::{Field, Schema},
};
use fmi::traits::{FmiImport, FmiInstance};

use crate::{
    options::{CoSimulationOptions, ModelExchangeOptions},
    Error,
};

use super::{
    interpolation::{Interpolate, PreLookup},
    io::StartValues,
    RecorderState,
};

/// Interface for building the Arrow schema for the inputs and outputs of an FMU.
pub trait FmiSchemaBuilder: FmiImport {
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

pub trait InstanceSetValues: FmiInstance {
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

pub trait InstanceRecordValues: FmiInstance + Sized {
    fn record_outputs(
        &mut self,
        time: f64,
        recorder: &mut RecorderState<Self>,
    ) -> anyhow::Result<()>;
}

pub trait FmiSim {
    /// Simulate the model using Model Exchange.
    fn simulate_me(
        &self,
        options: &ModelExchangeOptions,
        input_data: Option<RecordBatch>,
    ) -> Result<RecordBatch, Error>;

    /// Simulate the model using Co-Simulation.
    fn simulate_cs(
        &self,
        options: &CoSimulationOptions,
        input_data: Option<RecordBatch>,
    ) -> Result<RecordBatch, Error>;
}
