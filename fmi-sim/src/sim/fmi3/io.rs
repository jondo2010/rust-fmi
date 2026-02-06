//! FMI3-specific input and output implementation

use arrow::{
    array::{
        ArrayBuilder, ArrayRef, AsArray, BinaryBuilder, BooleanBuilder, FixedSizeListBuilder,
        Float32Array, Float32Builder, Float64Array, Float64Builder, Int8Builder, Int16Builder,
        Int32Builder, Int64Builder, ListBuilder, StringBuilder, UInt8Array, UInt8Builder,
        UInt16Array, UInt16Builder, UInt32Array, UInt32Builder, UInt64Array, UInt64Builder,
        downcast_array,
    },
    datatypes::{
        DataType, Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type, UInt8Type,
        UInt16Type, UInt32Type, UInt64Type,
    },
};

use crate::sim::{
    interpolation::{Interpolate, PreLookup},
    output::{OutputDimension, OutputKind, OutputRecorder},
    traits::{InstRecordValues, InstSetValues},
};

use fmi::{fmi3::GetSet, traits::FmiInstance};

use itertools::Itertools;

const DEFAULT_BINARY_BUFFER_SIZE: usize = 1024;

fn resolve_array_len<Inst>(
    inst: &mut Inst,
    dims: &[OutputDimension<<Inst as FmiInstance>::ValueRef>],
) -> anyhow::Result<usize>
where
    Inst: GetSet + FmiInstance,
    Inst::ValueRef: Copy + Into<u32>,
{
    if dims.is_empty() {
        return Ok(1);
    }
    let mut len = 1usize;
    for dim in dims {
        match dim {
            OutputDimension::Fixed(size) => {
                len = len.saturating_mul(*size);
            }
            OutputDimension::Variable(vr) => {
                let mut value = [0u64];
                let vr_u32: u32 = (*vr).into();
                inst.get_uint64(&[vr_u32], &mut value)?;
                len = len.saturating_mul(value[0] as usize);
            }
        }
    }
    Ok(len)
}

macro_rules! append_listable_primitive {
    ($fn_name:ident, $scalar_dt:pat, $builder_ty:ty, $values_ty:ty, $builder_label:expr, $dtype_label:expr) => {
        fn $fn_name(
            builder: &mut dyn ArrayBuilder,
            dtype: &DataType,
            values: &[$values_ty],
            len: usize,
        ) {
            match dtype {
                $scalar_dt => builder
                    .as_any_mut()
                    .downcast_mut::<$builder_ty>()
                    .expect(concat!("column is not ", $builder_label))
                    .append_value(values[0]),
                DataType::FixedSizeList(_, _) => {
                    let builder = builder
                        .as_any_mut()
                        .downcast_mut::<FixedSizeListBuilder<$builder_ty>>()
                        .expect(concat!("column is not FixedSizeList<", $builder_label, ">"));
                    for v in values.iter().take(len) {
                        builder.values().append_value(*v);
                    }
                    builder.append(true);
                }
                DataType::List(_) => {
                    let builder = builder
                        .as_any_mut()
                        .downcast_mut::<ListBuilder<$builder_ty>>()
                        .expect(concat!("column is not List<", $builder_label, ">"));
                    for v in values.iter().take(len) {
                        builder.values().append_value(*v);
                    }
                    builder.append(true);
                }
                _ => unimplemented!(concat!("Unsupported ", $dtype_label, " dtype {:?}"), dtype),
            }
        }
    };
}

append_listable_primitive!(
    append_bool,
    DataType::Boolean,
    BooleanBuilder,
    bool,
    "BooleanBuilder",
    "boolean"
);
append_listable_primitive!(
    append_i8,
    DataType::Int8,
    Int8Builder,
    i8,
    "Int8Builder",
    "int8"
);
append_listable_primitive!(
    append_i16,
    DataType::Int16,
    Int16Builder,
    i16,
    "Int16Builder",
    "int16"
);
append_listable_primitive!(
    append_i32,
    DataType::Int32,
    Int32Builder,
    i32,
    "Int32Builder",
    "int32"
);
append_listable_primitive!(
    append_i64,
    DataType::Int64,
    Int64Builder,
    i64,
    "Int64Builder",
    "int64"
);
append_listable_primitive!(
    append_u8,
    DataType::UInt8,
    UInt8Builder,
    u8,
    "UInt8Builder",
    "uint8"
);
append_listable_primitive!(
    append_u16,
    DataType::UInt16,
    UInt16Builder,
    u16,
    "UInt16Builder",
    "uint16"
);
append_listable_primitive!(
    append_u32,
    DataType::UInt32,
    UInt32Builder,
    u32,
    "UInt32Builder",
    "uint32"
);
append_listable_primitive!(
    append_u64,
    DataType::UInt64,
    UInt64Builder,
    u64,
    "UInt64Builder",
    "uint64"
);
append_listable_primitive!(
    append_f32,
    DataType::Float32,
    Float32Builder,
    f32,
    "Float32Builder",
    "float32"
);
append_listable_primitive!(
    append_f64,
    DataType::Float64,
    Float64Builder,
    f64,
    "Float64Builder",
    "float64"
);
fn append_utf8(
    builder: &mut dyn ArrayBuilder,
    dtype: &DataType,
    values: &[std::ffi::CString],
    len: usize,
) {
    match dtype {
        DataType::Utf8 => builder
            .as_any_mut()
            .downcast_mut::<StringBuilder>()
            .expect("column is not StringBuilder")
            .append_value(values[0].to_string_lossy()),
        DataType::FixedSizeList(_, _) => {
            let builder = builder
                .as_any_mut()
                .downcast_mut::<FixedSizeListBuilder<StringBuilder>>()
                .expect("column is not FixedSizeList<StringBuilder>");
            for v in values.iter().take(len) {
                builder.values().append_value(v.to_string_lossy());
            }
            builder.append(true);
        }
        DataType::List(_) => {
            let builder = builder
                .as_any_mut()
                .downcast_mut::<ListBuilder<StringBuilder>>()
                .expect("column is not List<StringBuilder>");
            for v in values.iter().take(len) {
                builder.values().append_value(v.to_string_lossy());
            }
            builder.append(true);
        }
        _ => unimplemented!("Unsupported utf8 dtype {:?}", dtype),
    }
}

fn append_binary(builder: &mut dyn ArrayBuilder, dtype: &DataType, values: &[Vec<u8>], len: usize) {
    match dtype {
        DataType::Binary => builder
            .as_any_mut()
            .downcast_mut::<BinaryBuilder>()
            .expect("column is not BinaryBuilder")
            .append_value(values[0].as_slice()),
        DataType::FixedSizeList(_, _) => {
            let builder = builder
                .as_any_mut()
                .downcast_mut::<FixedSizeListBuilder<BinaryBuilder>>()
                .expect("column is not FixedSizeList<BinaryBuilder>");
            for v in values.iter().take(len) {
                builder.values().append_value(v.as_slice());
            }
            builder.append(true);
        }
        DataType::List(_) => {
            let builder = builder
                .as_any_mut()
                .downcast_mut::<ListBuilder<BinaryBuilder>>()
                .expect("column is not List<BinaryBuilder>");
            for v in values.iter().take(len) {
                builder.values().append_value(v.as_slice());
            }
            builder.append(true);
        }
        _ => unimplemented!("Unsupported binary dtype {:?}", dtype),
    }
}

fn record_outputs_fmi3<T>(
    inst: &mut T,
    time: f64,
    recorder: &mut OutputRecorder<T>,
) -> anyhow::Result<()>
where
    T: GetSet + FmiInstance,
    T::ValueRef: Copy + Into<u32>,
{
    log::trace!("Recording variables at time {}", time);

    recorder.time_builder.append_value(time);

    let mut record_kind = |kind: OutputKind| -> anyhow::Result<()> {
        let mut indices = Vec::new();
        for (idx, (column, _)) in recorder.columns.iter().enumerate() {
            if column.kind == kind {
                indices.push(idx);
            }
        }
        if indices.is_empty() {
            return Ok(());
        }

        let mut vrs = Vec::with_capacity(indices.len());
        let mut vrs_u32 = Vec::with_capacity(indices.len());
        let mut lengths = Vec::with_capacity(indices.len());
        let mut total_len = 0usize;

        for idx in &indices {
            let column = &recorder.columns[*idx].0;
            let len = resolve_array_len(inst, &column.array_dims)?;
            vrs.push(column.vr);
            vrs_u32.push(column.vr.into());
            lengths.push(len);
            total_len = total_len.saturating_add(len);
        }

        match kind {
            OutputKind::Boolean | OutputKind::Clock => {
                let mut values = vec![false; total_len];
                if kind == OutputKind::Clock {
                    inst.get_clock(&vrs_u32, &mut values)?;
                } else {
                    inst.get_boolean(&vrs_u32, &mut values)?;
                }
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_bool(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::Int8 => {
                let mut values = vec![0i8; total_len];
                inst.get_int8(&vrs_u32, &mut values)?;
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_i8(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::Int16 => {
                let mut values = vec![0i16; total_len];
                inst.get_int16(&vrs_u32, &mut values)?;
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_i16(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::Int32 => {
                let mut values = vec![0i32; total_len];
                inst.get_int32(&vrs_u32, &mut values)?;
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_i32(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::Int64 => {
                let mut values = vec![0i64; total_len];
                inst.get_int64(&vrs_u32, &mut values)?;
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_i64(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::UInt8 => {
                let mut values = vec![0u8; total_len];
                inst.get_uint8(&vrs_u32, &mut values)?;
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_u8(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::UInt16 => {
                let mut values = vec![0u16; total_len];
                inst.get_uint16(&vrs_u32, &mut values)?;
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_u16(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::UInt32 => {
                let mut values = vec![0u32; total_len];
                inst.get_uint32(&vrs_u32, &mut values)?;
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_u32(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::UInt64 => {
                let mut values = vec![0u64; total_len];
                inst.get_uint64(&vrs_u32, &mut values)?;
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_u64(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::Float32 => {
                let mut values = vec![0f32; total_len];
                inst.get_float32(&vrs_u32, &mut values)?;
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_f32(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::Float64 => {
                let mut values = vec![0f64; total_len];
                inst.get_float64(&vrs_u32, &mut values)?;
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_f64(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::Utf8 => {
                let mut values = vec![std::ffi::CString::new("").unwrap(); total_len];
                inst.get_string(&vrs_u32, &mut values)?;
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &values[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_utf8(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
            OutputKind::Binary => {
                let mut buffers: Vec<Vec<u8>> = Vec::with_capacity(total_len);
                let mut buffer_slices: Vec<&mut [u8]> = Vec::with_capacity(total_len);
                for (i, idx) in indices.iter().enumerate() {
                    let column = &recorder.columns[*idx].0;
                    let len = lengths[i];
                    let buffer_len = column.binary_max_size.unwrap_or(DEFAULT_BINARY_BUFFER_SIZE);
                    for _ in 0..len {
                        buffers.push(vec![0u8; buffer_len]);
                    }
                }
                for buf in &mut buffers {
                    buffer_slices.push(buf.as_mut_slice());
                }
                let sizes = inst.get_binary(&vrs_u32, &mut buffer_slices)?;
                for (buf, size) in buffers.iter_mut().zip(sizes.iter()) {
                    buf.truncate(*size);
                }
                let mut offset = 0;
                for (i, idx) in indices.iter().enumerate() {
                    let len = lengths[i];
                    let slice = &buffers[offset..offset + len];
                    let (column, state) = &mut recorder.columns[*idx];
                    append_binary(state.builder.as_mut(), column.field.data_type(), slice, len);
                    offset += len;
                }
            }
        }

        Ok(())
    };

    record_kind(OutputKind::Boolean)?;
    record_kind(OutputKind::Clock)?;
    record_kind(OutputKind::Int8)?;
    record_kind(OutputKind::Int16)?;
    record_kind(OutputKind::Int32)?;
    record_kind(OutputKind::Int64)?;
    record_kind(OutputKind::UInt8)?;
    record_kind(OutputKind::UInt16)?;
    record_kind(OutputKind::UInt32)?;
    record_kind(OutputKind::UInt64)?;
    record_kind(OutputKind::Float32)?;
    record_kind(OutputKind::Float64)?;
    record_kind(OutputKind::Utf8)?;
    record_kind(OutputKind::Binary)?;
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
impl_set_values!(fmi::fmi3::instance::InstanceCS);

#[cfg(feature = "me")]
impl_set_values!(fmi::fmi3::instance::InstanceME);

#[cfg(feature = "cs")]
impl InstRecordValues for fmi::fmi3::instance::InstanceCS {
    fn record_outputs(
        &mut self,
        time: f64,
        recorder: &mut OutputRecorder<Self>,
    ) -> anyhow::Result<()> {
        record_outputs_fmi3(self, time, recorder)
    }
}

#[cfg(feature = "me")]
impl InstRecordValues for fmi::fmi3::instance::InstanceME {
    fn record_outputs(
        &mut self,
        time: f64,
        recorder: &mut OutputRecorder<Self>,
    ) -> anyhow::Result<()> {
        record_outputs_fmi3(self, time, recorder)
    }
}
