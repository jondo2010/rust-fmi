use std::{io::Seek, path::Path, sync::Arc};

use arrow::{
    csv::{reader::Format, ReaderBuilder},
    record_batch::RecordBatch,
};

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

    let reader = ReaderBuilder::new(Arc::new(file_schema))
        .with_header(true)
        //.with_projection(input_projection)
        .build(file)?;

    let batches = reader.collect::<Result<Vec<_>, _>>()?;

    Ok(arrow::compute::concat_batches(
        &batches[0].schema(),
        &batches,
    )?)
}
