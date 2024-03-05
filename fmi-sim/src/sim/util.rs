use std::{io::Seek, path::Path, sync::Arc};

use arrow::{
    csv::{reader::Format, ReaderBuilder},
    datatypes::{Field, Schema, SchemaRef},
    record_batch::RecordBatch,
};
use comfy_table::Table;
use itertools::Itertools;

/// Read a CSV file into a single RecordBatch.
pub fn read_csv<P>(path: P) -> anyhow::Result<RecordBatch>
where
    P: AsRef<Path>,
{
    let mut file = std::fs::File::open(&path)?;

    // Infer the schema with the first 100 records
    let (file_schema, _) = Format::default()
        .with_header(true)
        .infer_schema(&file, Some(100))?;
    file.rewind()?;

    log::debug!(
        "Read CSV file {:?}, with schema: {:?}",
        path.as_ref(),
        file_schema
            .fields()
            .iter()
            .map(|f| f.name())
            .collect::<Vec<_>>()
    );

    let _time = Arc::new(arrow::datatypes::Field::new(
        "time",
        arrow::datatypes::DataType::Float64,
        false,
    ));

    // Create a non-nullible schema from the file schema
    let file_schema = Arc::new(Schema::new(
        file_schema
            .fields()
            .iter()
            .map(|f| Arc::new(Field::new(f.name(), f.data_type().clone(), false)) as Arc<Field>)
            .collect::<Vec<_>>(),
    ));

    let reader = ReaderBuilder::new(file_schema)
        .with_header(true)
        //.with_projection(input_projection)
        .build(file)?;

    let batches = reader.collect::<Result<Vec<_>, _>>()?;

    Ok(arrow::compute::concat_batches(
        &batches[0].schema(),
        &batches,
    )?)
}

/// Format the projected fields in a human-readable format
pub fn pretty_format_projection(
    input_data_schema: Arc<Schema>,
    model_input_schema: Arc<Schema>,
    time_field: Arc<Field>,
) -> impl std::fmt::Display {
    let mut table = Table::new();
    table.load_preset(comfy_table::presets::ASCII_BORDERS_ONLY_CONDENSED);
    table.set_header(vec!["Name", "Input Type", "Model Type"]);
    let rows_iter = input_data_schema.fields().iter().map(|input_field| {
        let model_field_name = model_input_schema
            .fields()
            .iter()
            .chain(std::iter::once(&time_field))
            .find(|model_field| model_field.name() == input_field.name())
            .map(|model_field| model_field.data_type());
        vec![
            input_field.name().to_string(),
            input_field.data_type().to_string(),
            model_field_name
                .map(|t| t.to_string())
                .unwrap_or("-None-".to_string()),
        ]
    });
    table.add_rows(rows_iter);
    table
}

/// Transform the `input_data` to match the `model_input_schema`. Input data columns are projected and
/// cast to the corresponding input schema columns.
///
/// This is necessary because the `input_data` may have extra columns or have different datatypes.
pub fn project_input_data(
    input_data: &RecordBatch,
    model_input_schema: SchemaRef,
) -> anyhow::Result<RecordBatch> {
    let input_data_schema = input_data.schema();

    let time_field = Arc::new(Field::new(
        "time",
        arrow::datatypes::DataType::Float64,
        false,
    ));

    let (projected_fields, projected_columns): (Vec<_>, Vec<_>) = model_input_schema
        .fields()
        .iter()
        .chain(std::iter::once(&time_field))
        .filter_map(|field| {
            input_data.column_by_name(field.name()).map(|col| {
                arrow::compute::cast(col, field.data_type())
                    .map(|col| (field.clone(), col))
                    .map_err(|_| anyhow::anyhow!("Error casting type"))
            })
        })
        .process_results(|pairs| pairs.unzip())?;

    log::debug!(
        "Projected input data schema:\n{}",
        pretty_format_projection(input_data_schema, model_input_schema, time_field)
    );

    let input_data_schema = Arc::new(Schema::new(projected_fields));
    RecordBatch::try_new(input_data_schema, projected_columns).map_err(anyhow::Error::from)
}
