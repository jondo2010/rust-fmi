use std::{path::Path, sync::Arc};

use anyhow::Context;
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use arrow_ipc::writer::StreamWriter;

use super::OutputSink;

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
