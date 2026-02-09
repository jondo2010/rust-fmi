use std::{collections::BTreeMap, io::BufWriter, path::Path, sync::Arc};

use anyhow::Context;
use arrow::array::{Array, Float64Array};
use arrow::datatypes::Schema;
use arrow::json::writer::{make_encoder, EncoderOptions, NullableEncoder};
use arrow::record_batch::RecordBatch;
use mcap::records::MessageHeader;
use mcap::Writer;

use super::{OutputSink, TerminalChannelBinding};

const MCAP_CHANNEL: &str = "fmi-sim/output";
const MCAP_ENCODING: &str = "json";

pub struct McapSink {
    writer: Writer<BufWriter<std::fs::File>>,
    row_channel_id: u16,
    terminal_channels: Vec<TerminalChannel>,
    sequence: u32,
}

struct TerminalChannel {
    channel_id: u16,
    fields: Vec<TerminalFieldIndex>,
}

struct TerminalFieldIndex {
    column_index: usize,
    field_name: String,
}

impl McapSink {
    pub fn new<P: AsRef<Path>>(
        path: P,
        _schema: Arc<Schema>,
        terminal_bindings: Vec<TerminalChannelBinding>,
    ) -> anyhow::Result<Self> {
        let file = std::fs::File::create(path).context("Failed to create MCAP output file")?;
        let mut writer = Writer::new(BufWriter::new(file)).context("Failed to open MCAP writer")?;
        let row_channel_id = writer
            .add_channel(0, MCAP_CHANNEL, MCAP_ENCODING, &BTreeMap::new())
            .context("Failed to register MCAP channel")?;
        let terminal_channels = register_terminal_channels(&mut writer, terminal_bindings)?;
        Ok(Self {
            writer,
            row_channel_id,
            terminal_channels,
            sequence: 0,
        })
    }
}

impl OutputSink for McapSink {
    fn write_batch(&mut self, batch: &RecordBatch) -> anyhow::Result<()> {
        let time_array = batch
            .column(0)
            .as_any()
            .downcast_ref::<Float64Array>()
            .context("Expected time column to be Float64")?;
        let schema = batch.schema();
        let fields = schema.fields();
        let options = EncoderOptions::default();
        let mut encoders: Vec<NullableEncoder<'_>> = Vec::with_capacity(batch.num_columns());
        for (field, column) in fields.iter().zip(batch.columns()) {
            let encoder = make_encoder(field, column.as_ref(), &options)
                .context("Failed to build JSON encoder")?;
            encoders.push(encoder);
        }

        let mut payload = Vec::new();
        for row in 0..batch.num_rows() {
            let time = if time_array.is_null(row) {
                0.0
            } else {
                time_array.value(row)
            };
            let log_time = time_to_nanos(time);
            payload.clear();
            write_row_json(fields.as_ref(), &mut encoders, row, &mut payload);
            let header = MessageHeader {
                channel_id: self.row_channel_id,
                sequence: self.sequence,
                log_time,
                publish_time: log_time,
            };
            self.sequence = self.sequence.wrapping_add(1);
            self.writer
                .write_to_known_channel(&header, &payload)
                .context("Failed to write MCAP message")?;

            for terminal in &mut self.terminal_channels {
                payload.clear();
                write_terminal_json(&terminal.fields, &mut encoders, row, &mut payload);
                let header = MessageHeader {
                    channel_id: terminal.channel_id,
                    sequence: self.sequence,
                    log_time,
                    publish_time: log_time,
                };
                self.sequence = self.sequence.wrapping_add(1);
                self.writer
                    .write_to_known_channel(&header, &payload)
                    .context("Failed to write terminal MCAP message")?;
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> anyhow::Result<()> {
        self.writer
            .finish()
            .map(|_| ())
            .context("Failed to finalize MCAP writer")
    }
}

fn time_to_nanos(time: f64) -> u64 {
    if time.is_finite() && time >= 0.0 {
        (time * 1_000_000_000.0).round() as u64
    } else {
        0
    }
}

fn write_row_json(
    fields: &[Arc<arrow::datatypes::Field>],
    encoders: &mut [NullableEncoder<'_>],
    row: usize,
    out: &mut Vec<u8>,
) {
    out.push(b'{');
    for (idx, field) in fields.iter().enumerate() {
        if idx > 0 {
            out.push(b',');
        }
        write_json_string(field.name(), out);
        out.push(b':');
        if encoders[idx].is_null(row) {
            out.extend_from_slice(b"null");
        } else {
            encoders[idx].encode(row, out);
        }
    }
    out.push(b'}');
}

fn write_terminal_json(
    fields: &[TerminalFieldIndex],
    encoders: &mut [NullableEncoder<'_>],
    row: usize,
    out: &mut Vec<u8>,
) {
    out.push(b'{');
    for (idx, field) in fields.iter().enumerate() {
        if idx > 0 {
            out.push(b',');
        }
        write_json_string(&field.field_name, out);
        out.push(b':');
        let encoder = &mut encoders[field.column_index];
        if encoder.is_null(row) {
            out.extend_from_slice(b"null");
        } else {
            encoder.encode(row, out);
        }
    }
    out.push(b'}');
}

fn register_terminal_channels(
    writer: &mut Writer<BufWriter<std::fs::File>>,
    bindings: Vec<TerminalChannelBinding>,
) -> anyhow::Result<Vec<TerminalChannel>> {
    let mut channels = Vec::new();
    for binding in bindings {
        let mut metadata = BTreeMap::new();
        if let Some(name) = binding.schema_name.as_deref() {
            metadata.insert("foxglove.schema".to_string(), name.to_string());
        }
        if let Some(url) = binding.schema_url.as_deref() {
            metadata.insert("foxglove.schema_url".to_string(), url.to_string());
        }
        let channel_name = format!("fmi-sim/terminal/{}", binding.terminal_name);
        let channel_id = writer
            .add_channel(0, &channel_name, MCAP_ENCODING, &metadata)
            .context("Failed to register terminal MCAP channel")?;
        let fields = binding
            .fields
            .into_iter()
            .map(|field| TerminalFieldIndex {
                column_index: field.column_index,
                field_name: field.field_name,
            })
            .collect();
        channels.push(TerminalChannel { channel_id, fields });
    }
    Ok(channels)
}

fn write_json_string(value: &str, out: &mut Vec<u8>) {
    out.push(b'\"');
    for ch in value.chars() {
        match ch {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            ch if (ch as u32) <= 0x1F => {
                let mut buf = [0u8; 6];
                let encoded = format!("\\u{:04x}", ch as u32);
                buf[..encoded.len()].copy_from_slice(encoded.as_bytes());
                out.extend_from_slice(&buf[..encoded.len()]);
            }
            ch => {
                let mut buf = [0u8; 4];
                let encoded = ch.encode_utf8(&mut buf);
                out.extend_from_slice(encoded.as_bytes());
            }
        }
    }
    out.push(b'\"');
}
