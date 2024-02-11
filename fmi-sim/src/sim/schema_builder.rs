use arrow::datatypes::{DataType, Field, Fields, Schema};
use fmi::{fmi3::import::Fmi3Import, FmiImport};

pub trait FmiSchemaBuilder {
    /// Build the schema for the inputs of the model.
    fn inputs_schema(&self) -> Schema;
    /// Build the schema for the outputs of the model.
    fn outputs_schema(&self) -> Schema;
    /// Build a list of Schema column indices and value references for the continuous inputs.
    fn continuous_inputs(&self, schema: &Schema) -> Vec<(usize, u32)>;
    /// Build a list of Schema column indices and value references for the discrete inputs.
    fn discrete_inputs(&self, schema: &Schema) -> Vec<(usize, u32)>;
    /// Build a list of Schema column indices and value references for the outputs.
    fn outputs(&self, schema: &Schema) -> Vec<(usize, u32)>;
}

impl FmiSchemaBuilder for Fmi3Import {
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

    fn continuous_inputs(&self, schema: &Schema) -> Vec<(usize, u32)> {
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

    fn discrete_inputs(&self, schema: &Schema) -> Vec<(usize, u32)> {
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

    fn outputs(&self, schema: &Schema) -> Vec<(usize, u32)> {
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

#[cfg(feature = "disable")]
#[test_log::test]
fn test_input_csv() {
    let import = fmi::Import::new("../data/reference_fmus/3.0/Feedthrough.fmu")
        .unwrap()
        .as_fmi3()
        .unwrap();

    let schema = import.inputs_schema();

    let data = crate::sim::read_csv("../data/feedthrough_in.csv").unwrap();

    println!(
        "{}",
        arrow::util::pretty::pretty_format_batches(&[data]).unwrap()
    );

    // let time_array: Float64Array =
    // arrow::array::downcast_array(data[0].column_by_name("time").unwrap());
}
