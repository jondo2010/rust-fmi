use super::{
    interpolation::{Interpolate, PreLookup},
    traits::InstanceSetValues,
};
use arrow::{
    array::{
        downcast_array, ArrayRef, BinaryArray, BooleanArray, Float32Array, Float64Array,
        Int16Array, Int32Array, Int64Array, Int8Array, StringArray, UInt16Array, UInt32Array,
        UInt64Array, UInt8Array,
    },
    compute::cast,
    datatypes::DataType,
};
use fmi::{fmi3::instance::Common, traits::FmiInstance};

impl<Inst> InstanceSetValues for Inst
where
    Inst: Common,
{
    fn set_array(&mut self, vrs: &[Inst::ValueReference], values: &arrow::array::ArrayRef) {
        match values.data_type() {
            DataType::Boolean => {
                let values: BooleanArray = cast(values, &DataType::Boolean)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                let values = values.into_iter().filter_map(|v| v).collect::<Vec<_>>();
                self.set_boolean(vrs, &values);
            }
            DataType::Int8 => {
                let values: Int8Array = cast(values, &DataType::Int8)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_int8(vrs, values.values());
            }
            DataType::Int16 => {
                let values: Int16Array = cast(values, &DataType::Int16)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_int16(vrs, values.values());
            }
            DataType::Int32 => {
                let values: Int32Array = cast(values, &DataType::Int32)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_int32(vrs, values.values());
            }
            DataType::Int64 => {
                let values: Int64Array = cast(values, &DataType::Int64)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_int64(vrs, values.values());
            }
            DataType::UInt8 => {
                let values: UInt8Array = cast(values, &DataType::UInt8)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_uint8(vrs, values.values());
            }
            DataType::UInt16 => {
                let values: UInt16Array = cast(values, &DataType::UInt16)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_uint16(vrs, values.values());
            }
            DataType::UInt32 => {
                let values: UInt32Array = cast(values, &DataType::UInt32)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_uint32(vrs, values.values());
            }
            DataType::UInt64 => {
                let values: UInt64Array = cast(values, &DataType::UInt64)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_uint64(vrs, values.values());
            }
            DataType::Float16 => {
                unimplemented!()
            }
            DataType::Float32 => {
                let values: Float32Array = cast(values, &DataType::Float32)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_float32(vrs, values.values());
            }
            DataType::Float64 => {
                let values: Float64Array = cast(values, &DataType::Float64)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_float64(vrs, values.values());
            }
            DataType::Binary => {
                let values: BinaryArray = cast(values, &DataType::Binary)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_binary(vrs, values.iter().filter_map(|x| x));
            }
            DataType::FixedSizeBinary(_) => todo!(),
            DataType::LargeBinary => todo!(),
            DataType::Utf8 => {
                let values: StringArray = cast(values, &DataType::Utf8)
                    .map(|a| downcast_array(&a))
                    .expect("Error casting");
                self.set_string(vrs, values.iter().filter_map(|x| x));
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
                let array: Int8Array = downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_int8(&[vr], &[value]).ok()?;
            }
            DataType::Int16 => {
                let array: Int16Array = downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_int16(&[vr], &[value]).ok()?;
            }
            DataType::Int32 => {
                let array: Int32Array = downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_int32(&[vr], &[value]).ok()?;
            }
            DataType::Int64 => {
                let array: Int64Array = downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_int64(&[vr], &[value]).ok()?;
            }
            DataType::UInt8 => {
                let array: UInt8Array = downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_uint8(&[vr], &[value]).ok()?;
            }
            DataType::UInt16 => {
                let array: UInt16Array = downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_uint16(&[vr], &[value]).ok()?;
            }
            DataType::UInt32 => {
                let array: UInt32Array = downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_uint32(&[vr], &[value]).ok()?;
            }
            DataType::UInt64 => {
                let array: UInt64Array = downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_uint64(&[vr], &[value]).ok()?;
            }
            DataType::Float32 => {
                let array: Float32Array = downcast_array(&array);
                let value = I::interpolate(&pl, &array);
                self.set_float32(&[vr], &[value]).ok()?;
            }
            DataType::Float64 => {
                let array: Float64Array = downcast_array(&array);
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
