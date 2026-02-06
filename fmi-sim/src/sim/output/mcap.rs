use std::{collections::BTreeMap, io::BufWriter, path::Path, sync::Arc};

use anyhow::Context;
use arrow::array::{Array, Float64Array};
use arrow::datatypes::Schema;
use arrow::json::writer::{make_encoder, EncoderOptions, NullableEncoder};
use arrow::record_batch::RecordBatch;
use mcap::records::MessageHeader;
use mcap::Writer;

use super::OutputSink;

const MCAP_CHANNEL: &str = "fmi-sim/output";
const MCAP_ENCODING: &str = "json";

pub struct McapSink {
    writer: Writer<BufWriter<std::fs::File>>,
    channel_id: u16,
    sequence: u32,
}

impl McapSink {
    pub fn new<P: AsRef<Path>>(path: P, _schema: Arc<Schema>) -> anyhow::Result<Self> {
        let file = std::fs::File::create(path).context("Failed to create MCAP output file")?;
        let mut writer = Writer::new(BufWriter::new(file)).context("Failed to open MCAP writer")?;
        let channel_id = writer
            .add_channel(0, MCAP_CHANNEL, MCAP_ENCODING, &BTreeMap::new())
            .context("Failed to register MCAP channel")?;
        Ok(Self {
            writer,
            channel_id,
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
                channel_id: self.channel_id,
                sequence: self.sequence,
                log_time,
                publish_time: log_time,
            };
            self.sequence = self.sequence.wrapping_add(1);
            self.writer
                .write_to_known_channel(&header, &payload)
                .context("Failed to write MCAP message")?;
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
