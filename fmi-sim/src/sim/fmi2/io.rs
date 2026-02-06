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
    interpolation::{Interpolate, PreLookup},
    output::{OutputKind, OutputRecorder},
    traits::{InstRecordValues, InstSetValues},
};

fn record_outputs_fmi2<T>(
    inst: &mut T,
    time: f64,
    recorder: &mut OutputRecorder<T>,
) -> anyhow::Result<()>
where
    T: Common + FmiInstance,
    T::ValueRef: Copy + Into<u32>,
{
    log::trace!("Recording variables at time {}", time);

    recorder.time_builder.append_value(time);
    for (column, state) in &mut recorder.columns {
        log::trace!("Recording variable of kind {:?}", column.kind);
        match column.kind {
            OutputKind::Boolean => {
                let mut value = [std::default::Default::default()];
                let vr_u32: u32 = column.vr.into();
                inst.get_boolean(&[vr_u32], &mut value)?;
                state
                    .builder
                    .as_any_mut()
                    .downcast_mut::<BooleanBuilder>()
                    .expect("column is not BooleanBuilder")
                    .append_value(value[0] > 0);
            }
            OutputKind::Int32 => {
                let mut value = [std::default::Default::default()];
                let vr_u32: u32 = column.vr.into();
                inst.get_integer(&[vr_u32], &mut value)?;
                state
                    .builder
                    .as_any_mut()
                    .downcast_mut::<Int32Builder>()
                    .expect("column is not Int32Builder")
                    .append_value(value[0]);
            }
            OutputKind::Float64 => {
                let mut value = [std::default::Default::default()];
                let vr_u32: u32 = column.vr.into();
                inst.get_real(&[vr_u32], &mut value)?;
                state
                    .builder
                    .as_any_mut()
                    .downcast_mut::<Float64Builder>()
                    .expect("column is not Float64Builder")
                    .append_value(value[0]);
            }
            OutputKind::Utf8 => {
                let mut values = vec![std::ffi::CString::new("").unwrap()];
                let vr_u32: u32 = column.vr.into();
                if inst.get_string(&[vr_u32], &mut values).is_ok() {
                    let string_value = values[0].to_str().unwrap_or("");
                    state
                        .builder
                        .as_any_mut()
                        .downcast_mut::<StringBuilder>()
                        .expect("column is not StringBuilder")
                        .append_value(string_value);
                } else {
                    state
                        .builder
                        .as_any_mut()
                        .downcast_mut::<StringBuilder>()
                        .expect("column is not StringBuilder")
                        .append_value("");
                }
            }
            _ => unimplemented!("Unsupported output kind: {:?}", column.kind),
        }
    }

    recorder.row_count += 1;
    recorder.maybe_flush()?;
    Ok(())
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

#[cfg(feature = "me")]
impl_set_values!(fmi::fmi2::instance::InstanceME);

#[cfg(feature = "cs")]
impl InstRecordValues for fmi::fmi2::instance::InstanceCS {
    fn record_outputs(
        &mut self,
        time: f64,
        recorder: &mut OutputRecorder<Self>,
    ) -> anyhow::Result<()> {
        record_outputs_fmi2(self, time, recorder)
    }
}

#[cfg(feature = "me")]
impl InstRecordValues for fmi::fmi2::instance::InstanceME {
    fn record_outputs(
        &mut self,
        time: f64,
        recorder: &mut OutputRecorder<Self>,
    ) -> anyhow::Result<()> {
        record_outputs_fmi2(self, time, recorder)
    }
}
