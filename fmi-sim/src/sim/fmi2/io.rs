//! FMI2-specific input and output implementation

use arrow::{
    array::{
        ArrayRef, AsArray, BooleanBuilder, Float64Array, Float64Builder, Int32Builder,
        StringBuilder, downcast_array,
    },
    datatypes::{DataType, Float64Type, Int32Type},
};
use fmi::{fmi2::instance::Common, traits::FmiInstance};
use itertools::Itertools;

use crate::sim::{
    RecorderState,
    interpolation::{Interpolate, PreLookup},
    io::Recorder,
    traits::{InstRecordValues, InstSetValues},
};

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
                            let mut value = [std::default::Default::default()];
                            self.get_boolean(&[*vr], &mut value)?;
                            builder
                                .as_any_mut()
                                .downcast_mut::<BooleanBuilder>()
                                .expect(concat!("column is not ", stringify!($builder_type)))
                                .append_value(value[0] > 0);
                        }
                        DataType::Int32 => {
                            impl_recorder!(get_integer, Int32Builder, self, vr, builder)
                        }
                        DataType::Float64 => {
                            impl_recorder!(get_real, Float64Builder, self, vr, builder)
                        }
                        DataType::Utf8 => {
                            let mut values = vec![std::ffi::CString::new("").unwrap()];
                            if self.get_string(&[*vr], &mut values).is_ok() {
                                let string_value = values[0].to_str().unwrap_or("");
                                builder
                                    .as_any_mut()
                                    .downcast_mut::<StringBuilder>()
                                    .expect("column is not StringBuilder")
                                    .append_value(string_value);
                            } else {
                                // Handle error case by appending empty string
                                builder
                                    .as_any_mut()
                                    .downcast_mut::<StringBuilder>()
                                    .expect("column is not StringBuilder")
                                    .append_value("");
                            }
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
                        let values = values
                            .as_boolean()
                            .iter()
                            .map(|x| x.unwrap() as i32)
                            .collect_vec();
                        let _ = self.set_boolean(vrs, &values);
                    }
                    DataType::Int32 => {
                        let _ = self.set_integer(vrs, values.as_primitive::<Int32Type>().values());
                    }
                    DataType::Float64 => {
                        let _ = self.set_real(vrs, values.as_primitive::<Float64Type>().values());
                    }
                    DataType::Utf8 => {
                        let cstrings: Vec<std::ffi::CString> = values
                            .as_string::<i32>()
                            .iter()
                            .flatten()
                            .map(|s| std::ffi::CString::new(s).unwrap())
                            .collect();
                        let _ = self.set_string(vrs, &cstrings);
                    }
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
                    DataType::Int32 => {
                        let array = array.as_primitive::<Int32Type>();
                        let value = I::interpolate(pl, &array);
                        self.set_integer(&[vr], &[value])?;
                    }
                    DataType::Float64 => {
                        let array: Float64Array = downcast_array(&array);
                        let value = I::interpolate(pl, &array);
                        self.set_real(&[vr], &[value])?;
                    }
                    _ => unimplemented!("Unsupported data type: {:?}", array.data_type()),
                }
                Ok(())
            }
        }
    };
}

#[cfg(feature = "cs")]
impl_set_values!(fmi::fmi2::instance::InstanceCS);
#[cfg(feature = "cs")]
impl_record_values!(fmi::fmi2::instance::InstanceCS);

#[cfg(feature = "me")]
impl_set_values!(fmi::fmi2::instance::InstanceME);
#[cfg(feature = "me")]
impl_record_values!(fmi::fmi2::instance::InstanceME);
