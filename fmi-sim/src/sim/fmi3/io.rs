//! FMI3-specific input and output implementation

use anyhow::Context;
use arrow::{
    array::{
        downcast_array, make_builder, BinaryBuilder, BooleanBuilder, Float32Builder, Float64Array,
        Float64Builder, Int16Builder, Int32Builder, Int64Builder, Int8Builder, UInt16Builder,
        UInt32Builder, UInt64Builder, UInt8Builder,
    },
    datatypes::DataType,
    downcast_primitive_array,
};
use fmi::{fmi3::instance::Common, traits::FmiInstance};

use crate::sim::{
    interpolation::{find_index, Interpolate, PreLookup},
    io::Recorder,
    params::SimParams,
    traits::{FmiSchemaBuilder, InstanceSetValues},
    InputState, OutputState,
};

macro_rules! impl_recorder {
    ($getter:ident, $builder_type:ident, $inst:expr, $vr:ident, $builder:ident) => {{
        let mut value = [std::default::Default::default()];
        $inst.$getter(&[*$vr], &mut value).ok()?;
        $builder
            .as_any_mut()
            .downcast_mut::<$builder_type>()
            .expect(concat!("column is not ", stringify!($builder_type)))
            .append_value(value[0]);
    }};
}

impl<Inst> OutputState<Inst>
where
    Inst: Common,
    Inst::Import: FmiSchemaBuilder,
{
    pub fn record_outputs(&mut self, time: f64, inst: &mut Inst) -> anyhow::Result<()> {
        log::trace!("Recording variables at time {}", time);

        self.time.append_value(time);

        for Recorder {
            field,
            value_reference: vr,
            builder,
        } in &mut self.recorders
        {
            match field.data_type() {
                DataType::Boolean => {
                    impl_recorder!(get_boolean, BooleanBuilder, inst, vr, builder)
                }
                DataType::Int8 => {
                    impl_recorder!(get_int8, Int8Builder, inst, vr, builder)
                }
                DataType::Int16 => {
                    impl_recorder!(get_int16, Int16Builder, inst, vr, builder)
                }
                DataType::Int32 => {
                    impl_recorder!(get_int32, Int32Builder, inst, vr, builder)
                }
                DataType::Int64 => {
                    impl_recorder!(get_int64, Int64Builder, inst, vr, builder)
                }
                DataType::UInt8 => {
                    impl_recorder!(get_uint8, UInt8Builder, inst, vr, builder)
                }
                DataType::UInt16 => {
                    impl_recorder!(get_uint16, UInt16Builder, inst, vr, builder)
                }
                DataType::UInt32 => {
                    impl_recorder!(get_uint32, UInt32Builder, inst, vr, builder)
                }
                DataType::UInt64 => {
                    impl_recorder!(get_uint64, UInt64Builder, inst, vr, builder)
                }
                DataType::Float32 => {
                    impl_recorder!(get_float32, Float32Builder, inst, vr, builder)
                }
                DataType::Float64 => {
                    impl_recorder!(get_float64, Float64Builder, inst, vr, builder)
                }
                DataType::Binary => {
                    let mut value = [std::default::Default::default()];
                    inst.get_binary(&[*vr], &mut value).ok()?;
                    let [value] = value;
                    builder
                        .as_any_mut()
                        .downcast_mut::<BinaryBuilder>()
                        .expect("column is not Binary")
                        .append_value(value);
                }
                _ => unimplemented!("Unsupported data type: {:?}", field.data_type()),
            }
        }

        Ok(())
    }
}
