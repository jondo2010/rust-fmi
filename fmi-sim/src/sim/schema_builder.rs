use arrow::datatypes::{DataType, Field, Fields, Schema};
use fmi::{fmi3::import::Fmi3Import, FmiImport};

pub trait FmiSchemaBuilder {
    type ValueReference;

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

impl FmiSchemaBuilder for Fmi3Import {
    type ValueReference = u32;

    fn inputs_schema(&self) -> Schema {
        let input_fields = self
            .model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| v.causality() == fmi::fmi3::schema::Causality::Input)
            .map(|v| Field::new(v.name(), v.data_type().into(), false))
            .collect::<Fields>();

        Schema::new(input_fields)
    }

    fn outputs_schema(&self) -> Schema {
        let time = Field::new("time", DataType::Float64, false);
        let output_fields = self
            .model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| v.causality() == fmi::fmi3::schema::Causality::Output)
            .map(|v| Field::new(v.name(), v.data_type().into(), false))
            .chain(std::iter::once(time))
            .collect::<Fields>();

        Schema::new(output_fields)
    }

    fn continuous_inputs(&self, schema: &Schema) -> Vec<(usize, Self::ValueReference)> {
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter_map(|v| {
                (v.causality() == fmi::fmi3::schema::Causality::Input
                    && v.variability() == fmi::fmi3::schema::Variability::Continuous)
                    .then(|| (schema.index_of(v.name()).unwrap(), v.value_reference()))
            })
            .collect::<Vec<_>>()
    }

    fn discrete_inputs(&self, schema: &Schema) -> Vec<(usize, Self::ValueReference)> {
        use fmi::fmi3::schema::{Causality, Variability};
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter_map(|v| {
                (v.causality() == Causality::Input
                    && (v.variability() == Variability::Discrete
                        || v.variability() == Variability::Tunable))
                    .then(|| (schema.index_of(v.name()).unwrap(), v.value_reference()))
            })
            .collect::<Vec<_>>()
    }

    fn outputs(&self, schema: &Schema) -> Vec<(usize, Self::ValueReference)> {
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter_map(|v| {
                (v.causality() == fmi::fmi3::schema::Causality::Output)
                    .then(|| (schema.index_of(v.name()).unwrap(), v.value_reference()))
            })
            .collect::<Vec<_>>()
    }
}
