//! FMI3-specific input and output implementation

use arrow::{
    array::{
        downcast_array, ArrayRef, AsArray, BinaryBuilder, BooleanBuilder, Float32Array,
        Float32Builder, Float64Array, Float64Builder, Int16Builder, Int32Builder, Int64Builder,
        Int8Builder, UInt16Array, UInt16Builder, UInt32Array, UInt32Builder, UInt64Array,
        UInt64Builder, UInt8Array, UInt8Builder,
    },
    datatypes::{
        DataType, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type, UInt16Type,
        UInt32Type, UInt64Type, UInt8Type,
    },
};
use fmi::{fmi3::instance::Common, traits::FmiInstance};

use crate::sim::{
    interpolation::{Interpolate, PreLookup},
    io::Recorder,
    traits::{InstanceRecordValues, InstanceSetValues},
    RecorderState,
};

use itertools::Itertools;

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

macro_rules! impl_record_values {
    ($inst:ty) => {
        impl InstanceRecordValues for $inst {
            fn record_outputs(
                &mut self,
                time: f64,
                recorder: &mut RecorderState<Self>,
            ) -> anyhow::Result<()> {
                log::trace!("Recording variables at time {}", time);

                recorder.time.append_value(time);
                for Recorder {
                    field,
                    value_reference: vr,
                    builder,
                } in &mut recorder.recorders
                {
                    match field.data_type() {
                        DataType::Boolean => {
                            impl_recorder!(get_boolean, BooleanBuilder, self, vr, builder)
                        }
                        DataType::Int8 => {
                            impl_recorder!(get_int8, Int8Builder, self, vr, builder)
                        }
                        DataType::Int16 => {
                            impl_recorder!(get_int16, Int16Builder, self, vr, builder)
                        }
                        DataType::Int32 => {
                            impl_recorder!(get_int32, Int32Builder, self, vr, builder)
                        }
                        DataType::Int64 => {
                            impl_recorder!(get_int64, Int64Builder, self, vr, builder)
                        }
                        DataType::UInt8 => {
                            impl_recorder!(get_uint8, UInt8Builder, self, vr, builder)
                        }
                        DataType::UInt16 => {
                            impl_recorder!(get_uint16, UInt16Builder, self, vr, builder)
                        }
                        DataType::UInt32 => {
                            impl_recorder!(get_uint32, UInt32Builder, self, vr, builder)
                        }
                        DataType::UInt64 => {
                            impl_recorder!(get_uint64, UInt64Builder, self, vr, builder)
                        }
                        DataType::Float32 => {
                            impl_recorder!(get_float32, Float32Builder, self, vr, builder)
                        }
                        DataType::Float64 => {
                            impl_recorder!(get_float64, Float64Builder, self, vr, builder)
                        }
                        DataType::Binary => {
                            let mut value = [std::default::Default::default()];
                            self.get_binary(&[*vr], &mut value).ok()?;
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
    };
}

macro_rules! impl_set_values {
    ($t:ty) => {
        impl InstanceSetValues for $t {
            fn set_array(&mut self, vrs: &[Self::ValueReference], values: &ArrayRef) {
                match values.data_type() {
                    DataType::Boolean => {
                        let values = values.as_boolean().iter().map(|x| x.unwrap()).collect_vec();
                        self.set_boolean(vrs, &values);
                    }
                    DataType::Int8 => {
                        self.set_int8(vrs, values.as_primitive::<Int8Type>().values());
                    }
                    DataType::Int16 => {
                        self.set_int16(vrs, values.as_primitive::<Int16Type>().values());
                    }
                    DataType::Int32 => {
                        self.set_int32(vrs, values.as_primitive::<Int32Type>().values());
                    }
                    DataType::Int64 => {
                        self.set_int64(vrs, values.as_primitive::<Int64Type>().values());
                    }
                    DataType::UInt8 => {
                        self.set_uint8(vrs, values.as_primitive::<UInt8Type>().values());
                    }
                    DataType::UInt16 => {
                        self.set_uint16(vrs, values.as_primitive::<UInt16Type>().values());
                    }
                    DataType::UInt32 => {
                        self.set_uint32(vrs, values.as_primitive::<UInt32Type>().values());
                    }
                    DataType::UInt64 => {
                        self.set_uint64(vrs, values.as_primitive::<UInt64Type>().values());
                    }
                    DataType::Float16 => {
                        unimplemented!()
                    }
                    DataType::Float32 => {
                        self.set_float32(vrs, values.as_primitive::<Float32Type>().values());
                    }
                    DataType::Float64 => {
                        self.set_float64(vrs, values.as_primitive::<Float64Type>().values());
                    }
                    DataType::Binary => {
                        self.set_binary(vrs, values.as_binary::<i32>().iter().flatten());
                    }
                    DataType::FixedSizeBinary(_) => todo!(),
                    DataType::LargeBinary => todo!(),
                    DataType::Utf8 => {
                        self.set_string(vrs, values.as_string::<i32>().iter().flatten());
                    }
                    DataType::LargeUtf8 => todo!(),
                    _ => unimplemented!("Unsupported data type"),
                }
            }

            fn set_interpolated<I: Interpolate>(
                &mut self,
                vr: <Self as FmiInstance>::ValueReference,
                pl: &PreLookup,
                array: &ArrayRef,
            ) -> anyhow::Result<()> {
                match array.data_type() {
                    DataType::Boolean => todo!(),
                    DataType::Int8 => {
                        let array = array.as_primitive::<Int8Type>();
                        let value = I::interpolate(pl, &array);
                        self.set_int8(&[vr], &[value]).ok()?;
                    }
                    DataType::Int16 => {
                        let array = array.as_primitive::<Int16Type>();
                        let value = I::interpolate(pl, &array);
                        self.set_int16(&[vr], &[value]).ok()?;
                    }
                    DataType::Int32 => {
                        let array = array.as_primitive::<Int32Type>();
                        let value = I::interpolate(pl, &array);
                        self.set_int32(&[vr], &[value]).ok()?;
                    }
                    DataType::Int64 => {
                        let array = array.as_primitive::<Int64Type>();
                        let value = I::interpolate(pl, &array);
                        self.set_int64(&[vr], &[value]).ok()?;
                    }
                    DataType::UInt8 => {
                        let array: UInt8Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_uint8(&[vr], &[value]).ok()?;
                    }
                    DataType::UInt16 => {
                        let array: UInt16Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_uint16(&[vr], &[value]).ok()?;
                    }
                    DataType::UInt32 => {
                        let array: UInt32Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_uint32(&[vr], &[value]).ok()?;
                    }
                    DataType::UInt64 => {
                        let array: UInt64Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_uint64(&[vr], &[value]).ok()?;
                    }
                    DataType::Float32 => {
                        let array: Float32Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_float32(&[vr], &[value]).ok()?;
                    }
                    DataType::Float64 => {
                        let array: Float64Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_float64(&[vr], &[value]).ok()?;
                    }
                    DataType::Binary => todo!(),
                    DataType::Utf8 => todo!(),
                    _ => unimplemented!("Unsupported data type: {:?}", array.data_type()),
                }
                Ok(())
            }
        }
    };
}

#[cfg(feature = "cs")]
impl_set_values!(fmi::fmi3::instance::InstanceCS<'_>);
#[cfg(feature = "cs")]
impl_record_values!(fmi::fmi3::instance::InstanceCS<'_>);

#[cfg(feature = "me")]
impl_set_values!(fmi::fmi3::instance::InstanceME<'_>);
#[cfg(feature = "me")]
impl_record_values!(fmi::fmi3::instance::InstanceME<'_>);
