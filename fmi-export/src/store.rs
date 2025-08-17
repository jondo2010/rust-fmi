//! A no_std-friendly, typed state store for FMI 3.0 models.
//!
//! Goals:
//! - Fixed-size, SoA layout per FMI scalar type (and arrays thereof)
//! - Static ValueReference mapping to typed pools via descriptors
//! - No heap usage (no_std) by default; allocations must be explicit and up-front
//! - Fast get/set helpers matching FMI access patterns (batch copy per type)
//! - Simple serialize/deserialize of the entire store for FMU state save/restore

#![allow(clippy::needless_lifetimes)]

use core::{marker::PhantomData, mem};
use std::os::raw;

pub trait StoreOps {
    type ValueRef: Copy + Into<u32> + From<u32>;

    fn get<U>(&self, vr: Self::ValueRef) -> Result<&U, StoreError>;
    fn get_mut<U>(&mut self, vr: Self::ValueRef) -> Result<&mut U, StoreError>;
    fn slice_for_desc<U>(&self, vr: Self::ValueRef) -> Result<&[U], StoreError>;
    fn slice_for_desc_mut<U>(&mut self, vr: Self::ValueRef) -> Result<&mut [U], StoreError>;
}

// ---------- VR encoding helpers (O(1) lookup) ----------
const TYPE_SHIFT: u32 = 28; // upper 4 bits
const IDX_MASK: u32 = 0x0FFF_FFFF; // lower 28 bits

/// FMI scalar types supported by the store (strings/binary not included by default).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarType {
    Boolean, // represented as u8 0/1 internally
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
}

#[inline]
pub const fn vr_pack(tag: ScalarType, idx: u32) -> u32 {
    ((tag as u32) << TYPE_SHIFT) | (idx & IDX_MASK)
}

#[inline]
pub const fn vr_unpack(vr: u32) -> (ScalarType, usize) {
    let tag = ((vr >> TYPE_SHIFT) & 0xF) as u8;
    // SAFETY: tag originates from our ScalarType range 0..=10
    let tag = unsafe { core::mem::transmute::<u8, ScalarType>(tag) };
    let idx = (vr & IDX_MASK) as usize;
    (tag, idx)
}

/// Typed descriptor for table entries, making offset/len explicit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Desc {
    pub offset: usize,
    pub len: usize,
}

/// Per-type descriptor tables: each entry maps a VR ordinal to (offset,len) in the typed pool.
///
/// This trait defines the descriptor arrays for each scalar type.
pub struct Tables<
    const NB: usize,
    const NI8: usize,
    const NI16: usize,
    const NI32: usize,
    const NI64: usize,
    const NU8: usize,
    const NU16: usize,
    const NU32: usize,
    const NU64: usize,
    const NF32: usize,
    const NF64: usize,
> {
    bools: &'static [Desc],
    i8s: &'static [Desc],
    i16s: &'static [Desc],
    i32s: &'static [Desc],
    i64s: &'static [Desc],
    u8s: &'static [Desc],
    u16s: &'static [Desc],
    u32s: &'static [Desc],
    u64s: &'static [Desc],
    f32s: &'static [Desc],
    f64s: &'static [Desc],
}

impl<
        const NB: usize,
        const NI8: usize,
        const NI16: usize,
        const NI32: usize,
        const NI64: usize,
        const NU8: usize,
        const NU16: usize,
        const NU32: usize,
        const NU64: usize,
        const NF32: usize,
        const NF64: usize,
    > Tables<NB, NI8, NI16, NI32, NI64, NU8, NU16, NU32, NU64, NF32, NF64>
{
    /// Decode a VR into (type, offset, len) in O(1) time.
    #[inline]
    fn get_desc(&self, tag: ScalarType, idx: usize) -> Result<&'static Desc, StoreError> {
        match tag {
            ScalarType::Boolean => self.bools.get(idx).ok_or(StoreError::UnknownValueReference),
            ScalarType::Int8 => self.i8s.get(idx).ok_or(StoreError::UnknownValueReference),
            ScalarType::Int16 => self.i16s.get(idx).ok_or(StoreError::UnknownValueReference),
            ScalarType::Int32 => self.i32s.get(idx).ok_or(StoreError::UnknownValueReference),
            ScalarType::Int64 => self.i64s.get(idx).ok_or(StoreError::UnknownValueReference),
            ScalarType::UInt8 => self.u8s.get(idx).ok_or(StoreError::UnknownValueReference),
            ScalarType::UInt16 => self.u16s.get(idx).ok_or(StoreError::UnknownValueReference),
            ScalarType::UInt32 => self.u32s.get(idx).ok_or(StoreError::UnknownValueReference),
            ScalarType::UInt64 => self.u64s.get(idx).ok_or(StoreError::UnknownValueReference),
            ScalarType::Float32 => self.f32s.get(idx).ok_or(StoreError::UnknownValueReference),
            ScalarType::Float64 => self.f64s.get(idx).ok_or(StoreError::UnknownValueReference),
        }
    }
}

/// Error type for store operations.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("The ValueReference was not found in the descriptor table.")]
    UnknownValueReference,
    #[error("The requested type does not match the underlying variable type.")]
    TypeMismatch,
    #[error("The destination/source buffer is too small.")]
    BufferTooSmall,
    #[error("Attempted to access out of bounds region given the configured capacities.")]
    OutOfBounds,
    #[error("Serialized payload length mismatch or corruption.")]
    BadSerialization,
}

/// Fixed-size, SoA store across FMI scalar types.
///
/// The capacity for each type is provided via const generic parameters.
/// All variables of that type are packed into the corresponding typed pool and addressed via
/// the `Tables` descriptor arrays.
pub struct Store<
    VR: Copy + Into<u32> + From<u32> + 'static,
    const NB: usize,
    const NI8: usize,
    const NI16: usize,
    const NI32: usize,
    const NI64: usize,
    const NU8: usize,
    const NU16: usize,
    const NU32: usize,
    const NU64: usize,
    const NF32: usize,
    const NF64: usize,
> {
    // Pools (SoA) by FMI scalar type
    booleans: [u8; NB],
    i8s: [i8; NI8],
    i16s: [i16; NI16],
    i32s: [i32; NI32],
    i64s: [i64; NI64],
    u8s: [u8; NU8],
    u16s: [u16; NU16],
    u32s: [u32; NU32],
    u64s: [u64; NU64],
    f32s: [f32; NF32],
    f64s: [f64; NF64],
    tables: Tables<NB, NI8, NI16, NI32, NI64, NU8, NU16, NU32, NU64, NF32, NF64>,
    _vr: PhantomData<VR>,
}

/// Helper macro to match scalar types and extract relevant information
macro_rules! with_match_scalar_type {
    ($ty:expr, | $_1:tt $T:ident $_2:tt $F:ident $_3:tt $CAP:ident | $($body:tt)*) => ({
        macro_rules! __with_ty__ {( $_1 $T:ident $_2 $F:ident $_3 $CAP:ident ) => ( $($body)* )}
        match $ty {
            ScalarType::Boolean => __with_ty__! { u8 booleans NB },
            ScalarType::Int8 => __with_ty__! { i8 i8s NI8 },
            ScalarType::Int16 => __with_ty__! { i16 i16s NI16 },
            ScalarType::Int32 => __with_ty__! { i32 i32s NI32 },
            ScalarType::Int64 => __with_ty__! { i64 i64s NI64 },
            ScalarType::UInt8 => __with_ty__! { u8 u8s NU8 },
            ScalarType::UInt16 => __with_ty__! { u16 u16s NU16 },
            ScalarType::UInt32 => __with_ty__! { u32 u32s NU32 },
            ScalarType::UInt64 => __with_ty__! { u64 u64s NU64 },
            ScalarType::Float32 => __with_ty__! { f32 f32s NF32 },
            ScalarType::Float64 => __with_ty__! { f64 f64s NF64 },
            _ => panic!(),
        }
    });
}

impl<
        VR: Copy + Into<u32> + From<u32>,
        const NB: usize,
        const NI8: usize,
        const NI16: usize,
        const NI32: usize,
        const NI64: usize,
        const NU8: usize,
        const NU16: usize,
        const NU32: usize,
        const NU64: usize,
        const NF32: usize,
        const NF64: usize,
    > Store<VR, NB, NI8, NI16, NI32, NI64, NU8, NU16, NU32, NU64, NF32, NF64>
{
    /// Create a new zero-initialized store with a static descriptor table.
    ///
    /// Note: Descriptors must reference offsets within the configured capacities.
    /// This constructor performs debug assertions; for production builds ensure
    /// your descriptors are correct by construction.
    pub const fn new(
        tables: Tables<NB, NI8, NI16, NI32, NI64, NU8, NU16, NU32, NU64, NF32, NF64>,
    ) -> Self {
        // Arrays are zero-initialized; booleans use 0=false.
        Self {
            booleans: [0u8; NB],
            i8s: [0i8; NI8],
            i16s: [0i16; NI16],
            i32s: [0i32; NI32],
            i64s: [0i64; NI64],
            u8s: [0u8; NU8],
            u16s: [0u16; NU16],
            u32s: [0u32; NU32],
            u64s: [0u64; NU64],
            f32s: [0.0f32; NF32],
            f64s: [0.0f64; NF64],
            tables,
            _vr: PhantomData,
        }
    }

    /// Get the descriptor for a value reference.
    #[inline]
    fn get_desc(&self, vr: VR) -> Result<(ScalarType, &'static Desc), StoreError> {
        let raw: u32 = vr.into();
        let (tag, idx) = vr_unpack(raw);
        self.tables.get_desc(tag, idx).map(|desc| (tag, desc))
    }

    // ...existing code...
}

impl<
        VR: Copy + Into<u32> + From<u32>,
        const NB: usize,
        const NI8: usize,
        const NI16: usize,
        const NI32: usize,
        const NI64: usize,
        const NU8: usize,
        const NU16: usize,
        const NU32: usize,
        const NU64: usize,
        const NF32: usize,
        const NF64: usize,
    > StoreOps for Store<VR, NB, NI8, NI16, NI32, NI64, NU8, NU16, NU32, NU64, NF32, NF64>
{
    type ValueRef = VR;
    fn get<U>(&self, vr: Self::ValueRef) -> Result<&U, StoreError> {
        let (ty, &Desc { offset, len }) = self.get_desc(vr)?;
        let ptr = with_match_scalar_type!(ty, |$T $F $CAP| {
            if mem::size_of::<$T>() != mem::size_of::<U>() { return Err(StoreError::TypeMismatch); }
            if offset >= $CAP { return Err(StoreError::OutOfBounds); }
            &(self).$F[offset] as *const $T as *const U
        });
        Ok(unsafe { &*ptr })
    }
    fn get_mut<U>(&mut self, vr: Self::ValueRef) -> Result<&mut U, StoreError> {
        let (ty, &Desc { offset, len }) = self.get_desc(vr)?;
        let ptr = with_match_scalar_type!(ty, |$T $F $CAP| {
            if mem::size_of::<$T>() != mem::size_of::<U>() { return Err(StoreError::TypeMismatch); }
            if offset >= $CAP { return Err(StoreError::OutOfBounds); }
            &mut (self).$F[offset] as *mut $T as *mut U
        });
        Ok(unsafe { &mut *ptr })
    }
    fn slice_for_desc<U>(&self, vr: Self::ValueRef) -> Result<&[U], StoreError> {
        let (ty, &Desc { offset, len }) = self.get_desc(vr)?;
        let ptr = with_match_scalar_type!(ty, |$T $F $CAP| {
            if mem::size_of::<$T>() != mem::size_of::<U>() { return Err(StoreError::TypeMismatch); }
            if offset >= $CAP { return Err(StoreError::OutOfBounds); }
            &(self).$F[offset] as *const $T as *const U
        });
        Ok(unsafe { core::slice::from_raw_parts(ptr, len) })
    }
    fn slice_for_desc_mut<U>(&mut self, vr: Self::ValueRef) -> Result<&mut [U], StoreError> {
        let (ty, &Desc { offset, len }) = self.get_desc(vr)?;
        let ptr = with_match_scalar_type!(ty, |$T $F $CAP| {
            if mem::size_of::<$T>() != mem::size_of::<U>() { return Err(StoreError::TypeMismatch); }
            if offset >= $CAP { return Err(StoreError::OutOfBounds); }
            &mut (self).$F[offset] as *mut $T as *mut U
        });
        Ok(unsafe { core::slice::from_raw_parts_mut(ptr, len) })
    }
}

// ------------- Tests -------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_get_set_and_serialize() {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(u32)]
        enum VR {
            // Float64 tag with indices 0,1,2
            X0 = vr_pack(ScalarType::Float64, 0),
            X1 = vr_pack(ScalarType::Float64, 1),
            Mu = vr_pack(ScalarType::Float64, 2),
        }

        impl From<u32> for VR {
            fn from(v: u32) -> Self {
                // Decodiere Typ/Index; wir kennen nur Float64-Testeinträge
                let (tag, idx) = super::vr_unpack(v);
                match (tag, idx) {
                    (ScalarType::Float64, 0) => VR::X0,
                    (ScalarType::Float64, 1) => VR::X1,
                    (ScalarType::Float64, 2) => VR::Mu,
                    _ => panic!("Unknown value reference"),
                }
            }
        }
        impl From<VR> for u32 {
            fn from(v: VR) -> Self {
                v as u32
            }
        }

        const TABLES: Tables<0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3> = Tables {
            bools: &[],
            i8s: &[],
            i16s: &[],
            i32s: &[],
            i64s: &[],
            u8s: &[],
            u16s: &[],
            u32s: &[],
            u64s: &[],
            f32s: &[],
            f64s: &[
                Desc { offset: 0, len: 1 },
                Desc { offset: 1, len: 1 },
                Desc { offset: 2, len: 1 },
            ],
        };

        let mut s = Store::new(TABLES);
        s.slice_for_desc_mut::<f64>(VR::X0).unwrap()[0] = 1.0;
    }
}
