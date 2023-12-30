use std::{io::Seek as _, path::Path, sync::Arc};

use arrow::{
    array::PrimitiveArray,
    csv::{reader::Format, ReaderBuilder},
    datatypes::{ArrowNativeType, ArrowNativeTypeOp, ArrowPrimitiveType, Field, Fields, Schema},
    record_batch::RecordBatch,
};
use fmi::import::FmiImport as _;

pub trait FmiSchemaBuilder {
    /// Build the schema for the inputs of the model.
    fn inputs_schema(&self) -> Schema;
    /// Build a list of Schema column indices and value references for the continuous inputs.
    fn continuous_inputs(&self, schema: &Schema) -> Vec<(usize, u32)>;
    /// Build a list of Schema column indices and value references for the discrete inputs.
    fn discrete_inputs(&self, schema: &Schema) -> Vec<(usize, u32)>;
}

impl FmiSchemaBuilder for fmi::fmi3::schema::ModelVariables {
    fn inputs_schema(&self) -> Schema {
        let input_fields = self
            .iter_abstract()
            .filter(|v| v.causality() == fmi::fmi3::schema::Causality::Input)
            .map(|v| Field::new(v.name(), v.r#type().into(), false))
            .collect::<Fields>();

        Schema::new(input_fields)
    }

    fn continuous_inputs(&self, schema: &Schema) -> Vec<(usize, u32)> {
        self.iter_abstract()
            .filter_map(|v| {
                (v.causality() == fmi::fmi3::schema::Causality::Input
                    && v.variability() == fmi::fmi3::schema::Variability::Continuous)
                    .then(|| (schema.index_of(v.name()).unwrap(), v.value_reference()))
            })
            .collect::<Vec<_>>()
    }

    fn discrete_inputs(&self, schema: &Schema) -> Vec<(usize, u32)> {
        self.iter_abstract()
            .filter_map(|v| {
                (v.causality() == fmi::fmi3::schema::Causality::Input
                    && v.variability() == fmi::fmi3::schema::Variability::Discrete)
                    .then(|| (schema.index_of(v.name()).unwrap(), v.value_reference()))
            })
            .collect::<Vec<_>>()
    }
}

pub fn csv_input<P>(path: P, input_schema: &Schema) -> anyhow::Result<RecordBatch>
where
    P: AsRef<Path>,
{
    let mut file = std::fs::File::open(path)?;

    // Infer the schema with the first 100 records
    let (file_schema, _) = Format::default()
        .with_header(true)
        .infer_schema(&file, Some(100))?;
    file.rewind()?;

    let time = Arc::new(arrow::datatypes::Field::new(
        "time",
        arrow::datatypes::DataType::Float64,
        false,
    ));

    // Build a projection based on the input schema and the file schema.
    // Input fields that are not in the file schema are ignored.
    let input_projection = input_schema
        .fields()
        .iter()
        .chain(std::iter::once(&time))
        .filter_map(|input_field| {
            file_schema.index_of(input_field.name()).ok().map(|idx| {
                let file_dt = file_schema.field(idx).data_type();
                let input_dt = input_field.data_type();

                // Check if the file data type is compatible with the input data type.
                let dt_match = file_dt == input_dt
                    || file_dt.primitive_width() >= input_dt.primitive_width()
                        //&& file_dt.is_signed_integer() == input_dt.is_signed_integer()
                        //&& file_dt.is_unsigned_integer() == input_dt.is_unsigned_integer()
                        && file_dt.is_floating() == input_dt.is_floating();

                dt_match.then(|| idx).ok_or(anyhow::anyhow!(
                    "Input field {} has type {:?} but file field {} has type {:?}",
                    input_field.name(),
                    input_field.data_type(),
                    file_schema.field(idx).name(),
                    file_schema.field(idx).data_type()
                ))
            })
        })
        .collect::<Result<Vec<usize>, _>>()?;

    let reader = ReaderBuilder::new(Arc::new(file_schema))
        .with_header(true)
        .with_projection(input_projection)
        .build(file)?;

    let batches = reader.collect::<Result<Vec<_>, _>>()?;

    Ok(arrow::compute::concat_batches(
        &batches[0].schema(),
        &batches,
    )?)
}

#[test]
fn test_input_csv() {
    let import = fmi::Import::new("../data/reference_fmus/3.0/Feedthrough.fmu")
        .unwrap()
        .as_fmi3()
        .unwrap();

    let schema = import.model_description().model_variables.inputs_schema();

    let data = csv_input("../data/feedthrough_in.csv", &schema).unwrap();

    println!(
        "{}",
        arrow::util::pretty::pretty_format_batches(&[data]).unwrap()
    );

    //let time_array: Float64Array = arrow::array::downcast_array(data[0].column_by_name("time").unwrap());
}
