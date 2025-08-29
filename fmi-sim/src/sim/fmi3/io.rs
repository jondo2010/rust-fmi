//! FMI3-specific input and output implementation

use arrow::{
    array::{
        ArrayRef, AsArray, BinaryBuilder, BooleanBuilder, Float32Array, Float32Builder,
        Float64Array, Float64Builder, Int8Builder, Int16Builder, Int32Builder, Int64Builder,
        StringBuilder, UInt8Array, UInt8Builder, UInt16Array, UInt16Builder, UInt32Array,
        UInt32Builder, UInt64Array, UInt64Builder, downcast_array,
    },
    datatypes::{
        DataType, Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type, UInt8Type,
        UInt16Type, UInt32Type, UInt64Type,
    },
};

use crate::sim::{
    RecorderState,
    interpolation::{Interpolate, PreLookup},
    io::Recorder,
    traits::{InstRecordValues, InstSetValues},
};

use fmi::{fmi3::GetSet, traits::FmiInstance};

use itertools::Itertools;

macro_rules! impl_recorder {
    ($getter:ident, $builder_type:ident, $inst:expr, $vr:ident, $builder:ident) => {{
        let mut value = [std::default::Default::default()];
        $inst.$getter(&[*$vr], &mut value)?;
        $builder
            .as_any_mut()
            .downcast_mut::<$builder_type>()
            .expect(concat!("column is not ", stringify!($builder_type)))
            .append_value(value[0]);
    }};
}

macro_rules! impl_record_values {
    ($inst:ty) => {
        impl InstRecordValues for $inst {
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
                            // Use a reasonable buffer size for binary data
                            let mut data = vec![0u8; 1024];
                            let mut value = [data.as_mut_slice()];
                            let sizes = self.get_binary(&[*vr], &mut value)?;
                            let actual_size = sizes.get(0).copied().unwrap_or(0);
                            data.truncate(actual_size);
                            builder
                                .as_any_mut()
                                .downcast_mut::<BinaryBuilder>()
                                .expect("column is not Binary")
                                .append_value(data);
                        }
                        DataType::Utf8 => {
                            let mut values = [std::ffi::CString::new("").unwrap()];
                            let _ = self.get_string(&[*vr], &mut values);
                            let string_value = values[0].to_string_lossy();
                            builder
                                .as_any_mut()
                                .downcast_mut::<StringBuilder>()
                                .expect("column is not Utf8")
                                .append_value(string_value);
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
        impl InstSetValues for $t {
            fn set_array(&mut self, vrs: &[Self::ValueRef], values: &ArrayRef) {
                match values.data_type() {
                    DataType::Boolean => {
                        let values = values.as_boolean().iter().map(|x| x.unwrap()).collect_vec();
                        self.set_boolean(vrs, &values).unwrap();
                    }
                    DataType::Int8 => {
                        self.set_int8(vrs, values.as_primitive::<Int8Type>().values()).unwrap();
                    }
                    DataType::Int16 => {
                        self.set_int16(vrs, values.as_primitive::<Int16Type>().values()).unwrap();
                    }
                    DataType::Int32 => {
                        self.set_int32(vrs, values.as_primitive::<Int32Type>().values()).unwrap();
                    }
                    DataType::Int64 => {
                        self.set_int64(vrs, values.as_primitive::<Int64Type>().values()).unwrap();
                    }
                    DataType::UInt8 => {
                        self.set_uint8(vrs, values.as_primitive::<UInt8Type>().values()).unwrap();
                    }
                    DataType::UInt16 => {
                        self.set_uint16(vrs, values.as_primitive::<UInt16Type>().values()).unwrap();
                    }
                    DataType::UInt32 => {
                        self.set_uint32(vrs, values.as_primitive::<UInt32Type>().values()).unwrap();
                    }
                    DataType::UInt64 => {
                        self.set_uint64(vrs, values.as_primitive::<UInt64Type>().values()).unwrap();
                    }
                    DataType::Float16 => {
                        unimplemented!()
                    }
                    DataType::Float32 => {
                        self.set_float32(vrs, values.as_primitive::<Float32Type>().values()).unwrap();
                    }
                    DataType::Float64 => {
                        self.set_float64(vrs, values.as_primitive::<Float64Type>().values()).unwrap();
                    }
                    DataType::Binary => {
                        let binary_refs: Vec<&[u8]> = values
                            .as_binary::<i32>()
                            .iter()
                            .filter_map(|opt| opt) // Filter out None values
                            .collect();
                        let _ = self.set_binary(vrs, &binary_refs);
                    }
                    DataType::FixedSizeBinary(_) => todo!(),
                    DataType::LargeBinary => todo!(),
                    DataType::Utf8 => {
                        let string_values: Vec<std::ffi::CString> = values
                            .as_string::<i32>()
                            .iter()
                            .filter_map(|opt| opt) // Filter out None values
                            .map(|s| std::ffi::CString::new(s).unwrap())
                            .collect();
                        let _ = self.set_string(vrs, &string_values);
                    }
                    DataType::LargeUtf8 => todo!(),
                    _ => unimplemented!("Unsupported data type"),
                }
            }

            fn set_interpolated<I: Interpolate>(
                &mut self,
                vr: <Self as FmiInstance>::ValueRef,
                pl: &PreLookup,
                array: &ArrayRef,
            ) -> anyhow::Result<()> {
                match array.data_type() {
                    DataType::Boolean => todo!(),
                    DataType::Int8 => {
                        let array = array.as_primitive::<Int8Type>();
                        let value = I::interpolate(pl, &array);
                        self.set_int8(&[vr], &[value])?;
                    }
                    DataType::Int16 => {
                        let array = array.as_primitive::<Int16Type>();
                        let value = I::interpolate(pl, &array);
                        self.set_int16(&[vr], &[value])?;
                    }
                    DataType::Int32 => {
                        let array = array.as_primitive::<Int32Type>();
                        let value = I::interpolate(pl, &array);
                        self.set_int32(&[vr], &[value])?;
                    }
                    DataType::Int64 => {
                        let array = array.as_primitive::<Int64Type>();
                        let value = I::interpolate(pl, &array);
                        self.set_int64(&[vr], &[value])?;
                    }
                    DataType::UInt8 => {
                        let array: UInt8Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_uint8(&[vr], &[value])?;
                    }
                    DataType::UInt16 => {
                        let array: UInt16Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_uint16(&[vr], &[value])?;
                    }
                    DataType::UInt32 => {
                        let array: UInt32Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_uint32(&[vr], &[value])?;
                    }
                    DataType::UInt64 => {
                        let array: UInt64Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_uint64(&[vr], &[value])?;
                    }
                    DataType::Float32 => {
                        let array: Float32Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_float32(&[vr], &[value])?;
                    }
                    DataType::Float64 => {
                        let array: Float64Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_float64(&[vr], &[value])?;
                    }
                    DataType::Binary => todo!(),
                    DataType::Utf8 => {
                        // For string interpolation, we use the next index value (no real interpolation for strings)
                        let array = array.as_string::<i32>();
                        let index = pl.next_index().min(array.iter().count().saturating_sub(1));
                        if let Some(Some(value)) = array.iter().nth(index) {
                            let cstring = std::ffi::CString::new(value).unwrap();
                            let _ = self.set_string(&[vr], &[cstring]);
                        }
                    }
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
