use std::{path::Path, sync::Arc};

use anyhow::Context;
use arrow::array::{Array, ArrayRef, FixedSizeListArray, ListArray, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::json::writer::{make_encoder, EncoderOptions};
use arrow::record_batch::RecordBatch;

use super::OutputSink;

pub struct CsvSink {
    writer: arrow::csv::writer::Writer<std::fs::File>,
}

impl CsvSink {
    pub fn new<P: AsRef<Path>>(path: P, _schema: Arc<Schema>) -> anyhow::Result<Self> {
        let file = std::fs::File::create(path).context("Failed to create CSV output file")?;
        let writer = arrow::csv::writer::WriterBuilder::new()
            .with_header(true)
            .build(file);
        Ok(Self { writer })
    }
}

impl OutputSink for CsvSink {
    fn write_batch(&mut self, batch: &RecordBatch) -> anyhow::Result<()> {
        let batch = stringify_list_columns(batch)?;
        self.writer.write(&batch).context("Failed to write CSV batch")
    }

    fn finish(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

fn stringify_list_columns(batch: &RecordBatch) -> anyhow::Result<RecordBatch> {
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(batch.num_columns());
    let mut fields: Vec<Field> = Vec::with_capacity(batch.num_columns());

    for (field, column) in batch.schema().fields().iter().zip(batch.columns()) {
        match field.data_type() {
            DataType::FixedSizeList(_, _) | DataType::List(_) => {
                let string_array = list_array_to_string(column)?;
                let new_field = Field::new(field.name(), DataType::Utf8, field.is_nullable());
                columns.push(Arc::new(string_array));
                fields.push(new_field);
            }
            _ => {
                columns.push(column.clone());
                fields.push(field.as_ref().clone());
            }
        }
    }

    Ok(RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)?)
}

fn list_array_to_string(array: &ArrayRef) -> anyhow::Result<StringArray> {
    if let Some(list) = array.as_any().downcast_ref::<ListArray>() {
        return list_to_string(list);
    }
    if let Some(list) = array.as_any().downcast_ref::<FixedSizeListArray>() {
        return fixed_list_to_string(list);
    }
    Err(anyhow::anyhow!("Unsupported list array type"))
}

fn list_to_string(list: &ListArray) -> anyhow::Result<StringArray> {
    let field = Arc::new(Field::new("list", list.data_type().clone(), true));
    let options = EncoderOptions::default();
    let mut encoder = make_encoder(&field, list, &options)
        .context("Failed to build JSON encoder for list column")?;
    let mut out = Vec::with_capacity(list.len());
    for i in 0..list.len() {
        if encoder.is_null(i) {
            out.push(None);
            continue;
        }
        let mut buf = Vec::new();
        encoder.encode(i, &mut buf);
        let json = String::from_utf8(buf).context("Failed to decode JSON string")?;
        out.push(Some(json));
    }
    Ok(StringArray::from(out))
}

fn fixed_list_to_string(list: &FixedSizeListArray) -> anyhow::Result<StringArray> {
    let field = Arc::new(Field::new("list", list.data_type().clone(), true));
    let options = EncoderOptions::default();
    let mut encoder = make_encoder(&field, list, &options)
        .context("Failed to build JSON encoder for fixed list column")?;
    let mut out = Vec::with_capacity(list.len());
    for i in 0..list.len() {
        if encoder.is_null(i) {
            out.push(None);
            continue;
        }
        let mut buf = Vec::new();
        encoder.encode(i, &mut buf);
        let json = String::from_utf8(buf).context("Failed to decode JSON string")?;
        out.push(Some(json));
    }
    Ok(StringArray::from(out))
}
