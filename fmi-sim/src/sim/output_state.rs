use std::{default, sync::Arc};

use arrow::{
    array::{
        self, ArrayBuilder, BinaryBuilder, BooleanBuilder, Float32Builder, Float64Builder,
        Int16Builder, Int32Builder, Int64Builder, Int8Builder, UInt16Builder, UInt32Builder,
        UInt64Builder, UInt8Builder,
    },
    datatypes::{DataType, Schema},
    record_batch::RecordBatch,
};
use fmi::fmi3::{import::Fmi3Import, instance::Common};

use super::{params::SimParams, traits::FmiSchemaBuilder};

macro_rules! impl_recorder {
    ($getter:ident, $builder_type:ident, $inst:ident, $vr:ident, $self:ident, $column_index:ident) => {{
        let mut value = [default::Default::default()];
        $inst.$getter(&[*$vr], &mut value).ok()?;
        $self.recorders[*$column_index]
            .as_any_mut()
            .downcast_mut::<$builder_type>()
            .expect(concat!("column is not ", stringify!($builder_type)))
            .append_value(value[0]);
    }};
}

pub struct OutputState {
    output_schema: Schema,
    recorders: Vec<Box<dyn ArrayBuilder>>,
    outputs: Vec<(usize, u32)>,
}

impl OutputState {
    pub fn new(import: &Fmi3Import, sim_params: &SimParams) -> Self {
        let output_schema = import.outputs_schema();

        let num_points = ((sim_params.stop_time - sim_params.start_time)
            / sim_params.output_interval)
            .ceil() as usize;

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

    pub fn record_variables<Inst: Common>(
        &mut self,
        inst: &mut Inst,
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
                    impl_recorder!(get_boolean, BooleanBuilder, inst, vr, self, column_index)
                }
                DataType::Int8 => {
                    impl_recorder!(get_int8, Int8Builder, inst, vr, self, column_index)
                }
                DataType::Int16 => {
                    impl_recorder!(get_int16, Int16Builder, inst, vr, self, column_index)
                }
                DataType::Int32 => {
                    impl_recorder!(get_int32, Int32Builder, inst, vr, self, column_index)
                }
                DataType::Int64 => {
                    impl_recorder!(get_int64, Int64Builder, inst, vr, self, column_index)
                }
                DataType::UInt8 => {
                    impl_recorder!(get_uint8, UInt8Builder, inst, vr, self, column_index)
                }
                DataType::UInt16 => {
                    impl_recorder!(get_uint16, UInt16Builder, inst, vr, self, column_index)
                }
                DataType::UInt32 => {
                    impl_recorder!(get_uint32, UInt32Builder, inst, vr, self, column_index)
                }
                DataType::UInt64 => {
                    impl_recorder!(get_uint64, UInt64Builder, inst, vr, self, column_index)
                }
                DataType::Float32 => {
                    impl_recorder!(get_float32, Float32Builder, inst, vr, self, column_index)
                }
                DataType::Float64 => {
                    impl_recorder!(get_float64, Float64Builder, inst, vr, self, column_index)
                }
                DataType::Binary => {
                    let mut value = [default::Default::default()];
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
