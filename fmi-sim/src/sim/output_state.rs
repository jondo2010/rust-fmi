use std::sync::Arc;

use arrow::{
    array::{
        self, ArrayBuilder, BinaryBuilder, BooleanBuilder, Float32Builder, Float64Builder,
        Int16Builder, Int32Builder, Int64Builder, Int8Builder, UInt16Builder, UInt32Builder,
        UInt64Builder, UInt8Builder,
    },
    datatypes::{DataType, Schema},
    record_batch::RecordBatch,
};
use fmi::fmi3::{
    import::Fmi3Import,
    instance::{Common, Instance},
};

use super::{params::SimParams, schema_builder::FmiSchemaBuilder};

pub struct OutputState {
    output_schema: Schema,
    recorders: Vec<Box<dyn ArrayBuilder>>,
    outputs: Vec<(usize, u32)>,
}

impl OutputState {
    pub fn new(import: &Fmi3Import, params: &SimParams) -> Self {
        let output_schema = import.outputs_schema();

        let num_points =
            ((params.stop_time - params.start_time) / params.output_interval).ceil() as usize;

        let recorders = output_schema
            .fields()
            .iter()
            .map(|field| array::make_builder(field.data_type(), num_points))
            .collect();

        let outputs = import.outputs(&output_schema);

        Self {
            output_schema,
            recorders,
            outputs,
        }
    }

    pub fn record_variables<Tag>(
        &mut self,
        inst: &mut Instance<'_, Tag>,
        time: f64,
    ) -> anyhow::Result<()> {
        log::trace!("Recording variables at time {}", time);

        let time_idx = self
            .output_schema
            .index_of("time")
            .expect("time column not found");
        self.recorders[time_idx]
            .as_any_mut()
            .downcast_mut::<Float64Builder>()
            .expect("time column is not Float64")
            .append_value(time);

        for (column_index, vr) in &self.outputs {
            let col = self.output_schema.field(*column_index);

            match col.data_type() {
                DataType::Boolean => {
                    let mut value = [false];
                    inst.get_boolean(&[*vr], &mut value).ok()?;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<BooleanBuilder>()
                        .expect("column is not Boolean")
                        .append_value(value[0]);
                }
                DataType::Int8 => {
                    let mut value = [0];
                    inst.get_int8(&[*vr], &mut value).ok()?;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Int8Builder>()
                        .expect("column is not Int8")
                        .append_value(value[0]);
                }
                DataType::Int16 => {
                    let mut value = [0];
                    inst.get_int16(&[*vr], &mut value).ok()?;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Int16Builder>()
                        .expect("column is not Int16")
                        .append_value(value[0]);
                }
                DataType::Int32 => {
                    let mut value = [0];
                    inst.get_int32(&[*vr], &mut value).ok()?;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Int32Builder>()
                        .expect("column is not Int32")
                        .append_value(value[0]);
                }
                DataType::Int64 => {
                    let mut value = [0];
                    inst.get_int64(&[*vr], &mut value).ok()?;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Int64Builder>()
                        .expect("column is not Int64")
                        .append_value(value[0]);
                }
                DataType::UInt8 => {
                    let mut value = [0];
                    inst.get_uint8(&[*vr], &mut value).ok()?;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<UInt8Builder>()
                        .expect("column is not UInt8")
                        .append_value(value[0]);
                }
                DataType::UInt16 => {
                    let mut value = [0];
                    inst.get_uint16(&[*vr], &mut value).ok()?;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<UInt16Builder>()
                        .expect("column is not UInt16")
                        .append_value(value[0]);
                }
                DataType::UInt32 => {
                    let mut value = [0];
                    inst.get_uint32(&[*vr], &mut value).ok()?;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<UInt32Builder>()
                        .expect("column is not UInt32")
                        .append_value(value[0]);
                }
                DataType::UInt64 => {
                    let mut value = [0];
                    inst.get_uint64(&[*vr], &mut value).ok()?;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<UInt64Builder>()
                        .expect("column is not UInt64")
                        .append_value(value[0]);
                }
                DataType::Float32 => {
                    let mut value = [0.0];
                    inst.get_float32(&[*vr], &mut value).ok()?;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Float32Builder>()
                        .expect("column is not Float32")
                        .append_value(value[0]);
                }
                DataType::Float64 => {
                    let mut value = [0.0];
                    inst.get_float64(&[*vr], &mut value).ok()?;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Float64Builder>()
                        .expect("column is not Float64")
                        .append_value(value[0]);
                }
                DataType::Binary => {
                    let mut value = [vec![]];
                    inst.get_binary(&[*vr], &mut value).ok()?;
                    let [value] = value;
                    self.recorders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<BinaryBuilder>()
                        .expect("column is not Binary")
                        .append_value(value)
                }
                _ => unimplemented!("Unsupported data type: {:?}", col.data_type()),
            }
        }

        Ok(())
    }

    /// Finish the output state and return the RecordBatch.
    pub fn finish(self) -> RecordBatch {
        let columns = self
            .recorders
            .into_iter()
            .map(|mut builder| builder.finish())
            .collect::<Vec<_>>();

        RecordBatch::try_new(Arc::new(self.output_schema), columns)
            .expect("Failed to create RecordBatch")
    }
}
