use std::sync::Arc;

use arrow::{
    array::{
        ArrayBuilder, BinaryBuilder, BooleanBuilder, Float32Builder, Float64Builder, Int16Builder,
        Int32Builder, Int64Builder, Int8Builder, ListBuilder, StringBuilder, UInt16Builder,
        UInt32Builder, UInt64Builder, UInt8Builder, make_builder,
    },
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use fmi::traits::FmiInstance;
use std::collections::{HashMap, HashSet};

use crate::{options::OutputOptions, sim::traits::ImportSchemaBuilder};

pub mod arrow_ipc;
pub mod csv;
#[cfg(feature = "mcap")]
pub mod mcap;

pub use arrow_ipc::ArrowIpcSink;
pub use csv::CsvSink;
#[cfg(feature = "mcap")]
pub use mcap::McapSink;

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
pub struct TerminalSchemaBinding<VR> {
    pub terminal_name: String,
    pub schema_name: Option<String>,
    pub schema_url: Option<String>,
    pub fields: Vec<TerminalFieldBinding<VR>>,
}

#[derive(Debug, Clone)]
pub struct TerminalFieldBinding<VR> {
    pub vr: VR,
    pub field_name: String,
}

#[derive(Debug, Clone)]
pub struct TerminalChannelBinding {
    pub terminal_name: String,
    pub schema_name: Option<String>,
    pub schema_url: Option<String>,
    pub fields: Vec<TerminalFieldIndex>,
}

#[derive(Debug, Clone)]
pub struct TerminalFieldIndex {
    pub column_index: usize,
    pub field_name: String,
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
    pub terminal_bindings: Vec<TerminalSchemaBinding<VR>>,
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

pub fn resolve_terminal_channel_bindings<VR: Eq + std::hash::Hash + Copy>(
    columns: &[OutputColumn<VR>],
    bindings: &[TerminalSchemaBinding<VR>],
) -> Vec<TerminalChannelBinding> {
    let mut lookup = HashMap::new();
    for (idx, column) in columns.iter().enumerate() {
        lookup.insert(column.vr, idx + 1);
    }

    let mut resolved = Vec::new();
    for binding in bindings {
        if binding.schema_name.is_none() && binding.schema_url.is_none() {
            log::warn!(
                "Terminal '{}' has no foxglove schema annotation; skipping typed channel",
                binding.terminal_name
            );
            continue;
        }
        let mut fields = Vec::new();
        for field in &binding.fields {
            if let Some(col_idx) = lookup.get(&field.vr) {
                fields.push(TerminalFieldIndex {
                    column_index: *col_idx,
                    field_name: field.field_name.clone(),
                });
            } else {
                log::warn!(
                    "Terminal '{}' field '{}' not in output columns; skipping field",
                    binding.terminal_name,
                    field.field_name
                );
            }
        }
        if fields.is_empty() {
            log::warn!(
                "Terminal '{}' has no resolvable output fields; skipping typed channel",
                binding.terminal_name
            );
            continue;
        }
        resolved.push(TerminalChannelBinding {
            terminal_name: binding.terminal_name.clone(),
            schema_name: binding.schema_name.clone(),
            schema_url: binding.schema_url.clone(),
            fields,
        });
    }
    resolved
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
                let builder = make_column_builder(col.kind, col.field.data_type(), capacity);
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

fn make_column_builder(kind: OutputKind, dtype: &DataType, capacity: usize) -> Box<dyn ArrayBuilder> {
    match dtype {
        DataType::FixedSizeList(child, len) => match kind {
            OutputKind::Boolean | OutputKind::Clock => Box::new(
                arrow::array::FixedSizeListBuilder::new(BooleanBuilder::with_capacity(capacity), *len)
                    .with_field(child.as_ref().clone()),
            ),
            OutputKind::Int8 => Box::new(
                arrow::array::FixedSizeListBuilder::new(Int8Builder::with_capacity(capacity), *len)
                    .with_field(child.as_ref().clone()),
            ),
            OutputKind::Int16 => Box::new(
                arrow::array::FixedSizeListBuilder::new(Int16Builder::with_capacity(capacity), *len)
                    .with_field(child.as_ref().clone()),
            ),
            OutputKind::Int32 => Box::new(
                arrow::array::FixedSizeListBuilder::new(Int32Builder::with_capacity(capacity), *len)
                    .with_field(child.as_ref().clone()),
            ),
            OutputKind::Int64 => Box::new(
                arrow::array::FixedSizeListBuilder::new(Int64Builder::with_capacity(capacity), *len)
                    .with_field(child.as_ref().clone()),
            ),
            OutputKind::UInt8 => Box::new(
                arrow::array::FixedSizeListBuilder::new(UInt8Builder::with_capacity(capacity), *len)
                    .with_field(child.as_ref().clone()),
            ),
            OutputKind::UInt16 => Box::new(
                arrow::array::FixedSizeListBuilder::new(UInt16Builder::with_capacity(capacity), *len)
                    .with_field(child.as_ref().clone()),
            ),
            OutputKind::UInt32 => Box::new(
                arrow::array::FixedSizeListBuilder::new(UInt32Builder::with_capacity(capacity), *len)
                    .with_field(child.as_ref().clone()),
            ),
            OutputKind::UInt64 => Box::new(
                arrow::array::FixedSizeListBuilder::new(UInt64Builder::with_capacity(capacity), *len)
                    .with_field(child.as_ref().clone()),
            ),
            OutputKind::Float32 => Box::new(
                arrow::array::FixedSizeListBuilder::new(Float32Builder::with_capacity(capacity), *len)
                    .with_field(child.as_ref().clone()),
            ),
            OutputKind::Float64 => Box::new(
                arrow::array::FixedSizeListBuilder::new(Float64Builder::with_capacity(capacity), *len)
                    .with_field(child.as_ref().clone()),
            ),
            OutputKind::Utf8 => Box::new(arrow::array::FixedSizeListBuilder::new(
                StringBuilder::with_capacity(capacity, capacity * 8),
                *len,
            ).with_field(child.as_ref().clone())),
            OutputKind::Binary => Box::new(arrow::array::FixedSizeListBuilder::new(
                BinaryBuilder::with_capacity(capacity, capacity * 8),
                *len,
            ).with_field(child.as_ref().clone())),
        },
        DataType::List(child) => match kind {
            OutputKind::Boolean | OutputKind::Clock => Box::new(
                ListBuilder::new(BooleanBuilder::with_capacity(capacity)).with_field(child.as_ref().clone()),
            ),
            OutputKind::Int8 => Box::new(
                ListBuilder::new(Int8Builder::with_capacity(capacity)).with_field(child.as_ref().clone()),
            ),
            OutputKind::Int16 => Box::new(
                ListBuilder::new(Int16Builder::with_capacity(capacity)).with_field(child.as_ref().clone()),
            ),
            OutputKind::Int32 => Box::new(
                ListBuilder::new(Int32Builder::with_capacity(capacity)).with_field(child.as_ref().clone()),
            ),
            OutputKind::Int64 => Box::new(
                ListBuilder::new(Int64Builder::with_capacity(capacity)).with_field(child.as_ref().clone()),
            ),
            OutputKind::UInt8 => Box::new(
                ListBuilder::new(UInt8Builder::with_capacity(capacity)).with_field(child.as_ref().clone()),
            ),
            OutputKind::UInt16 => Box::new(
                ListBuilder::new(UInt16Builder::with_capacity(capacity)).with_field(child.as_ref().clone()),
            ),
            OutputKind::UInt32 => Box::new(
                ListBuilder::new(UInt32Builder::with_capacity(capacity)).with_field(child.as_ref().clone()),
            ),
            OutputKind::UInt64 => Box::new(
                ListBuilder::new(UInt64Builder::with_capacity(capacity)).with_field(child.as_ref().clone()),
            ),
            OutputKind::Float32 => Box::new(
                ListBuilder::new(Float32Builder::with_capacity(capacity)).with_field(child.as_ref().clone()),
            ),
            OutputKind::Float64 => Box::new(
                ListBuilder::new(Float64Builder::with_capacity(capacity)).with_field(child.as_ref().clone()),
            ),
            OutputKind::Utf8 => Box::new(ListBuilder::new(StringBuilder::with_capacity(
                capacity,
                capacity * 8,
            )).with_field(child.as_ref().clone())),
            OutputKind::Binary => Box::new(ListBuilder::new(BinaryBuilder::with_capacity(
                capacity,
                capacity * 8,
            )).with_field(child.as_ref().clone())),
        },
        _ => make_builder(dtype, capacity),
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

    let terminal_bindings = match import.output_terminal_bindings() {
        Ok(bindings) => bindings,
        Err(err) => {
            log::warn!("Failed to load terminal bindings: {err}");
            Vec::new()
        }
    };

    Ok(OutputPlan {
        schema,
        columns,
        flush_policy,
        byte_estimate_per_row,
        terminal_bindings,
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
