use anyhow::Context;
use arrow::{array::ArrayRef, datatypes::Schema};
use fmi::traits::{FmiImport, FmiInstance};

use super::{
    interpolation::{Interpolate, PreLookup},
    options::SimOptions,
    params::SimParams,
    InputState, OutputState,
};

pub trait FmiSchemaBuilder: FmiImport {
    /// Build the schema for the inputs of the model.
    fn inputs_schema(&self) -> Schema;
    /// Build the schema for the outputs of the model.
    fn outputs_schema(&self) -> Schema;
    /// Build a list of Schema column (index, ValueReference) for the continuous inputs.
    fn continuous_inputs(&self, schema: &Schema) -> Vec<(usize, Self::ValueReference)>;
    /// Build a list of Schema column (index, ValueReference) for the discrete inputs.
    fn discrete_inputs(&self, schema: &Schema) -> Vec<(usize, Self::ValueReference)>;
    /// Build a list of Schema column (index, ValueReference) for the outputs.
    fn outputs(&self, schema: &Schema) -> Vec<(usize, Self::ValueReference)>;
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

pub trait InputState2 {
    type Instance: InstanceSetValues;
    fn apply_input<I: Interpolate>(
        &self,
        time: f64,
        instance: &mut Self::Instance,
        discrete: bool,
        continuous: bool,
        after_event: bool,
    ) -> anyhow::Result<()>;
}

pub trait SimTrait<'a>: Sized {
    type Import: FmiImport + FmiSchemaBuilder;
    type InputState;
    type OutputState;

    fn new(
        import: &'a Self::Import,
        sim_params: SimParams,
        input_state: Self::InputState,
        output_state: Self::OutputState,
    ) -> anyhow::Result<Self>;

    fn new_from_options(import: &'a Self::Import, options: &SimOptions) -> anyhow::Result<Self> {
        let sim_params = SimParams::new_from_options(options, import.model_description())?;

        // Read optional input data from file
        let input_data = options
            .input_file
            .as_ref()
            .map(|path| super::util::read_csv(path))
            .transpose()
            .context("Reading input file")?;

        let input_state = InputState::new(import, input_data)?;
        let output_state = OutputState::new(import, &sim_params);

        Self::new(import, sim_params, input_state, output_state)
    }

    fn main_loop(&mut self) -> anyhow::Result<()>;
}
