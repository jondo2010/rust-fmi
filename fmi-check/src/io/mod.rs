use std::{io::Seek, path::Path, sync::Arc};

use arrow::{
    array::{
        self, ArrayRef, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array,
        UInt16Array, UInt32Array, UInt64Array, UInt8Array,
    },
    csv::{reader::Format, ReaderBuilder},
    datatypes::{DataType, Schema},
    record_batch::RecordBatch,
};
use fmi::fmi3::{
    binding,
    instance::{Common, Instance},
};

mod input_state;
mod interpolation;
mod output_state;
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
                dbg!(&value);
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
fn csv_recordbatch<P>(path: P, input_schema: &Schema) -> anyhow::Result<RecordBatch>
where
    P: AsRef<Path>,
{
    let mut file = std::fs::File::open(path)?;

    // Infer the schema with the first 100 records
    let (file_schema, _) = Format::default()
        .with_header(true)
        .infer_schema(&file, Some(100))?;
    file.rewind()?;

    let time = Arc::new(arrow::datatypes::Field::new(
        "time",
        arrow::datatypes::DataType::Float64,
        false,
    ));

    // Build a projection based on the input schema and the file schema.
    // Input fields that are not in the file schema are ignored.
    let input_projection = input_schema
        .fields()
        .iter()
        .chain(std::iter::once(&time))
        .filter_map(|input_field| {
            file_schema.index_of(input_field.name()).ok().map(|idx| {
                let file_dt = file_schema.field(idx).data_type();
                let input_dt = input_field.data_type();

                // Check if the file data type is compatible with the input data type.
                let dt_match = file_dt == input_dt
                    || file_dt.primitive_width() >= input_dt.primitive_width()
                        //&& file_dt.is_signed_integer() == input_dt.is_signed_integer()
                        //&& file_dt.is_unsigned_integer() == input_dt.is_unsigned_integer()
                        && file_dt.is_floating() == input_dt.is_floating();

                dt_match.then(|| idx).ok_or(anyhow::anyhow!(
                    "Input field {} has type {:?} but file field {} has type {:?}",
                    input_field.name(),
                    input_field.data_type(),
                    file_schema.field(idx).name(),
                    file_schema.field(idx).data_type()
                ))
            })
        })
        .collect::<Result<Vec<usize>, _>>()?;

    let reader = ReaderBuilder::new(Arc::new(file_schema))
        .with_header(true)
        .with_projection(input_projection)
        .build(file)?;

    let batches = reader.collect::<Result<Vec<_>, _>>()?;

    Ok(arrow::compute::concat_batches(
        &batches[0].schema(),
        &batches,
    )?)
}
