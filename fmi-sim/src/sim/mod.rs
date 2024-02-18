use std::{io::Seek, path::Path, sync::Arc};

use arrow::{
    array::{
        self, ArrayRef, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array,
        UInt16Array, UInt32Array, UInt64Array, UInt8Array,
    },
    csv::{reader::Format, ReaderBuilder},
    datatypes::DataType,
    record_batch::RecordBatch,
};
use fmi::fmi3::{
    binding,
    instance::{Common, Instance},
};

#[cfg(feature = "cs")]
pub mod fmi3_cs;
#[cfg(feature = "me")]
pub mod fmi3_me;
mod input_state;
mod interpolation;
pub mod options;
mod output_state;
pub mod params;
mod schema_builder;

pub use input_state::InputState;
pub use output_state::OutputState;

use self::interpolation::{Interpolate, PreLookup};

pub trait InstanceSetValues {
    type ValueReference;
    fn set_array(&mut self, vrs: &[Self::ValueReference], values: &arrow::array::ArrayRef);
    fn set_interpolated<I: Interpolate>(
        &mut self,
        vr: Self::ValueReference,
        pl: &PreLookup,
        array: &ArrayRef,
    ) -> anyhow::Result<()>;
}

impl<Tag> InstanceSetValues for Instance<'_, Tag> {
    type ValueReference = binding::fmi3ValueReference;
    fn set_array(&mut self, vrs: &[binding::fmi3ValueReference], values: &arrow::array::ArrayRef) {
        match values.data_type() {
            DataType::Boolean => {
                let values: arrow::array::BooleanArray =
                    arrow::compute::cast(values, &DataType::Boolean)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                let values = values.into_iter().filter_map(|v| v).collect::<Vec<_>>();
                self.set_boolean(vrs, &values);
            }
            DataType::Int8 => {
                let values: arrow::array::Int8Array = arrow::compute::cast(values, &DataType::Int8)
                    .map(|a| arrow::array::downcast_array(&a))
                    .expect("Error casting");
                self.set_int8(vrs, values.values());
            }
            DataType::Int16 => {
                let values: arrow::array::Int16Array =
                    arrow::compute::cast(values, &DataType::Int16)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_int16(vrs, values.values());
            }
            DataType::Int32 => {
                let values: arrow::array::Int32Array =
                    arrow::compute::cast(values, &DataType::Int32)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_int32(vrs, values.values());
            }
            DataType::Int64 => {
                let values: arrow::array::Int64Array =
                    arrow::compute::cast(values, &DataType::Int64)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_int64(vrs, values.values());
            }
            DataType::UInt8 => {
                let values: arrow::array::UInt8Array =
                    arrow::compute::cast(values, &DataType::UInt8)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_uint8(vrs, values.values());
            }
            DataType::UInt16 => {
                let values: arrow::array::UInt16Array =
                    arrow::compute::cast(values, &DataType::UInt16)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_uint16(vrs, values.values());
            }
            DataType::UInt32 => {
                let values: arrow::array::UInt32Array =
                    arrow::compute::cast(values, &DataType::UInt32)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_uint32(vrs, values.values());
            }
            DataType::UInt64 => {
                let values: arrow::array::UInt64Array =
                    arrow::compute::cast(values, &DataType::UInt64)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_uint64(vrs, values.values());
            }
            DataType::Float16 => {
                todo!()
            }
            DataType::Float32 => {
                let values: arrow::array::Float32Array =
                    arrow::compute::cast(values, &DataType::Float32)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_float32(vrs, values.values());
            }
            DataType::Float64 => {
                let values: arrow::array::Float64Array =
                    arrow::compute::cast(values, &DataType::Float64)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_float64(vrs, values.values());
            }
            DataType::Binary => {
                let values: arrow::array::BinaryArray =
                    arrow::compute::cast(values, &DataType::Binary)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_binary(vrs, values.iter().filter_map(|x| x));
            }
            DataType::FixedSizeBinary(_) => todo!(),
            DataType::LargeBinary => todo!(),
            DataType::Utf8 => {
                let values: arrow::array::StringArray =
                    arrow::compute::cast(values, &DataType::Utf8)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_string(vrs, values.iter().filter_map(|x| x));
            }
            DataType::LargeUtf8 => todo!(),
            _ => unimplemented!("Unsupported data type"),
        }
    }

    fn set_interpolated<I: Interpolate>(
        &mut self,
        vr: Self::ValueReference,
        pl: &PreLookup,
        array: &ArrayRef,
    ) -> anyhow::Result<()> {
        match array.data_type() {
            DataType::Boolean => todo!(),
            DataType::Int8 => {
                let array: Int8Array = array::downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_int8(&[vr], &[value]).ok()?;
            }
            DataType::Int16 => {
                let array: Int16Array = array::downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_int16(&[vr], &[value]).ok()?;
            }
            DataType::Int32 => {
                let array: Int32Array = array::downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_int32(&[vr], &[value]).ok()?;
            }
            DataType::Int64 => {
                let array: Int64Array = array::downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_int64(&[vr], &[value]).ok()?;
            }
            DataType::UInt8 => {
                let array: UInt8Array = array::downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_uint8(&[vr], &[value]).ok()?;
            }
            DataType::UInt16 => {
                let array: UInt16Array = array::downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_uint16(&[vr], &[value]).ok()?;
            }
            DataType::UInt32 => {
                let array: UInt32Array = array::downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_uint32(&[vr], &[value]).ok()?;
            }
            DataType::UInt64 => {
                let array: UInt64Array = array::downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_uint64(&[vr], &[value]).ok()?;
            }
            DataType::Float32 => {
                let array: Float32Array = array::downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_float32(&[vr], &[value]).ok()?;
            }
            DataType::Float64 => {
                let array: Float64Array = array::downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_float64(&[vr], &[value]).ok()?;
            }
            DataType::Binary => todo!(),
            DataType::Utf8 => todo!(),
            _ => unimplemented!("Unsupported data type: {:?}", array.data_type()),
        }
        Ok(())
    }
}

/// Read a CSV file into a single RecordBatch.
fn read_csv<P>(path: P) -> anyhow::Result<RecordBatch>
where
    P: AsRef<Path>,
{
    let mut file = std::fs::File::open(&path)?;

    // Infer the schema with the first 100 records
    let (file_schema, _) = Format::default()
        .with_header(true)
        .infer_schema(&file, Some(100))?;
    file.rewind()?;

    log::debug!(
        "Read CSV file {:?}, with schema: {:?}",
        path.as_ref(),
        file_schema
            .fields()
            .iter()
            .map(|f| f.name())
            .collect::<Vec<_>>()
    );

    let _time = Arc::new(arrow::datatypes::Field::new(
        "time",
        arrow::datatypes::DataType::Float64,
        false,
    ));

    let reader = ReaderBuilder::new(Arc::new(file_schema))
        .with_header(true)
        //.with_projection(input_projection)
        .build(file)?;

    let batches = reader.collect::<Result<Vec<_>, _>>()?;

    Ok(arrow::compute::concat_batches(
        &batches[0].schema(),
        &batches,
    )?)
}

#[cfg(test)]
mod tests {
    #[test_log::test]
    fn test_read_csv() {
        //let import = fmi::Import::new("../data/reference_fmus/3.0/Feedthrough.fmu")
        //    .unwrap()
        //    .as_fmi3()
        //    .unwrap();

        //let schema = import.inputs_schema();

        let data = crate::sim::read_csv("../tests/data/feedthrough_in.csv").unwrap();

        println!(
            "{}",
            arrow::util::pretty::pretty_format_batches(&[data]).unwrap()
        );

        // let time_array: Float64Array =
        // arrow::array::downcast_array(data[0].column_by_name("time").unwrap());
    }
}
