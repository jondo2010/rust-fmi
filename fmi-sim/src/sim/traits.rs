use arrow::{
    array::ArrayRef,
    datatypes::{Field, Schema},
    record_batch::RecordBatch,
};
use fmi::traits::{FmiImport, FmiInstance};

use crate::Error;

use super::{
    interpolation::{Interpolate, PreLookup},
    io::StartValues,
    params::SimParams,
};

pub trait FmiSchemaBuilder: FmiImport {
    /// Build the schema for the inputs of the model.
    fn inputs_schema(&self) -> Schema;
    /// Build the schema for the outputs of the model.
    fn outputs_schema(&self) -> Schema;
    /// Build a list of (Field, ValueReference) for the continuous inputs.
    fn continuous_inputs(&self) -> impl Iterator<Item = (Field, Self::ValueReference)> + '_;
    /// Build a list of Schema column (index, ValueReference) for the discrete inputs.
    fn discrete_inputs(&self) -> impl Iterator<Item = (Field, Self::ValueReference)> + '_;
    /// Build a list of Schema column (index, ValueReference) for the outputs.
    fn outputs(&self) -> impl Iterator<Item = (Field, Self::ValueReference)> + '_;

    /// Parse a list of "var=value" strings.
    ///
    /// # Returns
    /// A tuple of two lists of (ValueReference, Array) tuples. The first list contains any variable with
    /// `Causality = StructuralParameter` and the second list contains regular parameters.
    fn parse_start_values(
        &self,
        start_values: &[String],
    ) -> anyhow::Result<StartValues<Self::ValueReference>>;
}

pub trait InstanceSetValues: FmiInstance {
    fn set_array(
        &mut self,
        vrs: &[<Self as FmiInstance>::ValueReference],
        values: &arrow::array::ArrayRef,
    );
    fn set_interpolated<I: Interpolate>(
        &mut self,
        vr: <Self as FmiInstance>::ValueReference,
        pl: &PreLookup,
        array: &ArrayRef,
    ) -> anyhow::Result<()>;
}

pub trait SimInput: Sized {
    type Inst: FmiInstance;

    fn new(
        import: &<Self::Inst as FmiInstance>::Import,
        input_data: Option<RecordBatch>,
    ) -> anyhow::Result<Self>;

    fn apply_input<I: Interpolate>(
        &mut self,
        time: f64,
        inst: &mut Self::Inst,
        discrete: bool,
        continuous: bool,
        after_event: bool,
    ) -> anyhow::Result<()>;

    fn next_input_event(&self, time: f64) -> f64;
}

pub trait SimOutput {
    type Inst: FmiInstance;
    fn new(import: &<Self::Inst as FmiInstance>::Import, sim_params: &SimParams) -> Self;
    fn record_outputs(&mut self, time: f64, inst: &mut Self::Inst) -> anyhow::Result<()>;
}

pub trait SimTrait<'a>: Sized {
    /*
    type Import: FmiImport + FmiSchemaBuilder;
    type InputState;
    type OutputState;

    fn new(
        import: &'a Self::Import,
        sim_params: SimParams,
        input_state: Self::InputState,
        output_state: Self::OutputState,
    ) -> anyhow::Result<Self>;
    */

    fn main_loop(&mut self) -> Result<(), Error>;
}
