use arrow::{
    array::ArrayRef,
    datatypes::{Field, Schema},
};
use fmi::traits::{FmiImport, FmiInstance};

use super::{
    interpolation::{Interpolate, PreLookup},
    io::StartValues,
};

/// Interface for building the Arrow schema for the inputs and outputs of an FMU.
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
