use std::{path::Path, sync::Arc};

use anyhow::Context;
use arrow::{
    array::{
        Array, ArrayBuilder, ArrayRef, Float64Builder, FixedSizeListArray, ListArray, StringArray,
        make_builder,
    },
    datatypes::{DataType, Field, Schema},
    json::writer::{make_encoder, EncoderOptions},
    record_batch::RecordBatch,
};
use arrow_ipc::writer::StreamWriter;
use fmi::traits::FmiInstance;
use std::collections::HashSet;

use crate::{options::OutputOptions, sim::traits::ImportSchemaBuilder};

const DEFAULT_BINARY_ESTIMATE: usize = 128;
const DEFAULT_STRING_ESTIMATE: usize = 64;
const DEFAULT_LIST_LEN_ESTIMATE: usize = 8;
const DEFAULT_TARGET_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputKind {
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Utf8,
    Binary,
    Clock,
}

#[derive(Debug, Clone)]
pub struct OutputColumn<VR> {
    pub field: Field,
    pub vr: VR,
    pub kind: OutputKind,
    pub binary_max_size: Option<usize>,
    pub array_dims: Vec<OutputDimension<VR>>,
}

pub struct OutputColumnState {
    pub builder: Box<dyn ArrayBuilder>,
}

#[derive(Debug, Clone)]
pub enum OutputDimension<VR> {
    Fixed(usize),
    Variable(VR),
}

pub struct OutputPlan<VR> {
    pub schema: Arc<Schema>,
    pub columns: Vec<OutputColumn<VR>>,
    pub flush_policy: FlushPolicy,
    pub byte_estimate_per_row: usize,
}

pub struct FlushPolicy {
    pub target_rows: usize,
    pub target_bytes: usize,
}

impl FlushPolicy {
    pub fn should_flush(&self, current_rows: usize, current_bytes: usize) -> bool {
        current_rows >= self.target_rows || current_bytes >= self.target_bytes
    }
}

pub trait OutputSink {
    fn write_batch(&mut self, batch: &RecordBatch) -> anyhow::Result<()>;
    fn finish(&mut self) -> anyhow::Result<()>;
}

pub struct ArrowIpcSink {
    writer: StreamWriter<std::fs::File>,
}

impl ArrowIpcSink {
    pub fn new<P: AsRef<Path>>(path: P, schema: Arc<Schema>) -> anyhow::Result<Self> {
        let file = std::fs::File::create(path).context("Failed to create output file")?;
        let writer = StreamWriter::try_new(file, &schema).context("Failed to open IPC writer")?;
        Ok(Self { writer })
    }
}

impl OutputSink for ArrowIpcSink {
    fn write_batch(&mut self, batch: &RecordBatch) -> anyhow::Result<()> {
        self.writer
            .write(batch)
            .context("Failed to write Arrow IPC batch")
    }

    fn finish(&mut self) -> anyhow::Result<()> {
        self.writer.finish().context("Failed to finalize IPC writer")
    }
}

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

pub struct NullSink;

impl OutputSink for NullSink {
    fn write_batch(&mut self, _batch: &RecordBatch) -> anyhow::Result<()> {
        Ok(())
    }

    fn finish(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct OutputRecorder<Inst: FmiInstance> {
    pub schema: Arc<Schema>,
    pub time_builder: Float64Builder,
    pub columns: Vec<(OutputColumn<Inst::ValueRef>, OutputColumnState)>,
    pub flush_policy: FlushPolicy,
    pub sink: Box<dyn OutputSink>,
    pub row_count: usize,
    pub byte_estimate_per_row: usize,
}

impl<Inst: FmiInstance> OutputRecorder<Inst> {
    pub fn from_plan(
        plan: OutputPlan<Inst::ValueRef>,
        sink: Box<dyn OutputSink>,
    ) -> anyhow::Result<Self> {
        let capacity = plan.flush_policy.target_rows.max(1);
        let time_builder = Float64Builder::with_capacity(capacity);
        let columns = plan
            .columns
            .into_iter()
            .map(|col| {
                let builder = make_builder(col.field.data_type(), capacity);
                (col, OutputColumnState { builder })
            })
            .collect();

        Ok(Self {
            schema: plan.schema,
            time_builder,
            columns,
            flush_policy: plan.flush_policy,
            sink,
            row_count: 0,
            byte_estimate_per_row: plan.byte_estimate_per_row,
        })
    }

    pub fn maybe_flush(&mut self) -> anyhow::Result<()> {
        let current_rows = self.row_count;
        let current_bytes = current_rows.saturating_mul(self.byte_estimate_per_row);
        if self.flush_policy.should_flush(current_rows, current_bytes) {
            self.flush()?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        if self.row_count == 0 {
            return Ok(());
        }
        let batch = self.finish_batch()?;
        self.sink.write_batch(&batch)?;
        self.row_count = 0;
        Ok(())
    }

    pub fn finish(&mut self) -> anyhow::Result<()> {
        self.flush()?;
        self.sink.finish()
    }

    fn finish_batch(&mut self) -> anyhow::Result<RecordBatch> {
        let mut arrays = Vec::with_capacity(self.columns.len() + 1);
        let time_array = Arc::new(self.time_builder.finish());
        arrays.push(time_array as _);
        for (_column, state) in &mut self.columns {
            arrays.push(state.builder.finish());
        }
        Ok(RecordBatch::try_new(self.schema.clone(), arrays)?)
    }
}

pub fn estimate_row_bytes<VR>(columns: &[OutputColumn<VR>]) -> usize {
    columns
        .iter()
        .map(|col| estimate_field_bytes(&col.field, col.binary_max_size))
        .sum()
}

pub fn build_output_plan<Import: ImportSchemaBuilder>(
    import: &Import,
    vrs: &[Import::ValueRef],
    output_opts: &OutputOptions,
) -> anyhow::Result<OutputPlan<Import::ValueRef>>
where
    Import::ValueRef: Eq + std::hash::Hash + Copy + std::fmt::Debug,
{
    let mut seen = HashSet::new();
    let mut columns = Vec::new();

    for vr in vrs {
        if !seen.insert(*vr) {
            log::warn!("Duplicate output VR {:?} ignored", vr);
            continue;
        }
        let field = import.output_field_for_vr(*vr)?;
        let kind = import.output_kind_for_vr(*vr)?;
        let array_dims = import.output_array_dims_for_vr(*vr)?;
        let binary_max_size = if matches!(kind, OutputKind::Binary) {
            import.binary_max_size(*vr)
        } else {
            None
        };
        columns.push(OutputColumn {
            field,
            vr: *vr,
            kind,
            binary_max_size,
            array_dims,
        });
    }

    let mut fields = Vec::with_capacity(columns.len() + 1);
    fields.push(Field::new("time", DataType::Float64, false));
    fields.extend(columns.iter().map(|c| c.field.clone()));
    let schema = Arc::new(Schema::new(fields));
    let byte_estimate_per_row = estimate_row_bytes(&columns) + 8;
    let row_estimate = output_opts
        .row_size_override
        .unwrap_or(byte_estimate_per_row)
        .max(1);
    let target_bytes = output_opts.flush_bytes.unwrap_or(DEFAULT_TARGET_BYTES);
    let target_rows = output_opts
        .flush_rows
        .unwrap_or_else(|| (target_bytes / row_estimate).max(1));
    let flush_policy = FlushPolicy {
        target_rows,
        target_bytes,
    };

    Ok(OutputPlan {
        schema,
        columns,
        flush_policy,
        byte_estimate_per_row,
    })
}

fn estimate_field_bytes(field: &Field, binary_max_size: Option<usize>) -> usize {
    match field.data_type() {
        DataType::Boolean => 1,
        DataType::Int8 | DataType::UInt8 => 1,
        DataType::Int16 | DataType::UInt16 => 2,
        DataType::Int32 | DataType::UInt32 | DataType::Float32 => 4,
        DataType::Int64 | DataType::UInt64 | DataType::Float64 => 8,
        DataType::Utf8 => DEFAULT_STRING_ESTIMATE,
        DataType::Binary => binary_max_size.unwrap_or(DEFAULT_BINARY_ESTIMATE),
        DataType::FixedSizeList(child, len) => {
            (*len as usize) * estimate_field_bytes(child.as_ref(), binary_max_size)
        }
        DataType::List(child) => {
            DEFAULT_LIST_LEN_ESTIMATE * estimate_field_bytes(child.as_ref(), binary_max_size)
        }
        _ => DEFAULT_STRING_ESTIMATE,
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
