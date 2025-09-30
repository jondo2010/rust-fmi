//! Rust bindings for FMI-LS-BUS interface
//!
//! This module provides a safe, ergonomic Rust interface for FMI-LS-BUS operations
//! that is binary compatible with the C implementation.

use bytes::BytesMut;
use std::mem;

use crate::fmi3::binding;
use fmi_sys::ls_bus;

#[cfg(feature = "ls-bus-can")]
pub mod can;
#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum FmiLsBusError {
    #[error("Buffer overflow: not enough space in buffer")]
    BufferOverflow,
    #[error("Invalid variant code: {0}")]
    InvalidVariant(u32),
    #[error("Invalid operation code or size mismatch: {0}")]
    InvalidOperation(ls_bus::fmi3LsBusOperationCode),
}

pub trait LsBusOperation<'a>: Sized {
    /// Transmit the operation by writing it into the provided LS-BUS buffer.
    fn transmit(self, bus: &mut FmiLsBus) -> Result<(), FmiLsBusError>;

    fn read_next_operation(bus: &'a mut FmiLsBus) -> Result<Option<Self>, FmiLsBusError>;
}

/// A Rust wrapper around FMI-LS-BUS buffer operations using the `bytes` crate.
///
/// This provides a safe, ergonomic interface that's binary compatible with
/// the C `fmi3LsBusUtilBufferInfo` structure and associated macros.
#[derive(Debug)]
pub struct FmiLsBus {
    buffer: BytesMut,
    read_pos: usize,
}

impl FmiLsBus {
    /// Create a new LS-BUS buffer with the specified capacity.
    ///
    /// This is equivalent to initializing `fmi3LsBusUtilBufferInfo` with
    /// `FMI3_LS_BUS_BUFFER_INFO_INIT`.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            read_pos: 0,
        }
    }

    /// Reset the buffer to empty state.
    ///
    /// Equivalent to `FMI3_LS_BUS_BUFFER_INFO_RESET`.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.read_pos = 0;
    }

    /// Check if the buffer is empty.
    ///
    /// Equivalent to `FMI3_LS_BUS_BUFFER_IS_EMPTY`.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get the start address of the buffer data.
    ///
    /// Equivalent to `FMI3_LS_BUS_BUFFER_START`.
    pub fn start(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    /// Get the current length of data in the buffer.
    ///
    /// Equivalent to `FMI3_LS_BUS_BUFFER_LENGTH`.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Get a reference to the internal buffer as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    pub fn write_operation<'a, OP: LsBusOperation<'a>>(
        &mut self,
        op: OP,
    ) -> Result<(), FmiLsBusError> {
        op.transmit(self)
    }

    /// Write raw data to the buffer, replacing existing content.
    ///
    /// Equivalent to `FMI3_LS_BUS_BUFFER_WRITE`.
    pub fn write(&mut self, data: &[u8]) -> Result<(), FmiLsBusError> {
        if data.len() <= self.buffer.capacity() {
            self.buffer.clear();
            self.buffer.extend_from_slice(data);
            self.read_pos = 0;
            Ok(())
        } else {
            Err(FmiLsBusError::BufferOverflow)
        }
    }

    /// Peek at the next operation's opcode and length without advancing the read position.
    pub fn peek_next_operation(&self) -> Option<(ls_bus::fmi3LsBusOperationCode, usize)> {
        let remaining = self.buffer.len() - self.read_pos;

        // Need at least header size
        if remaining < mem::size_of::<ls_bus::fmi3LsBusOperationHeader>() {
            return None;
        }

        // Read header
        let header_bytes = &self.buffer
            [self.read_pos..self.read_pos + mem::size_of::<ls_bus::fmi3LsBusOperationHeader>()];
        let header = unsafe {
            std::ptr::read_unaligned(
                header_bytes.as_ptr() as *const ls_bus::fmi3LsBusOperationHeader
            )
        };

        Some((header.opCode, header.length as usize))
    }

    /// Read the next operation from the buffer.
    ///
    /// Returns `Some((op_code, data))` if an operation is available,
    /// or `None` if no complete operation is available.
    ///
    /// This is equivalent to `FMI3_LS_BUS_READ_NEXT_OPERATION`.
    pub fn read_next_operation<'a, OP: LsBusOperation<'a>>(
        &'a mut self,
    ) -> Result<Option<OP>, FmiLsBusError> {
        OP::read_next_operation(self)
    }

    /// Get a direct reference to the raw bytes for FFI compatibility.
    ///
    /// This allows the buffer to be used directly with C functions that expect
    /// `fmi3Binary` (const fmi3UInt8*).
    pub fn as_fmi_binary(&self) -> (*const binding::fmi3UInt8, usize) {
        (self.buffer.as_ptr(), self.buffer.len())
    }

    /// Create from existing raw buffer data (for FFI compatibility).
    ///
    /// # Safety
    /// The caller must ensure the buffer pointer is valid and contains valid LS-BUS data.
    pub unsafe fn from_raw_buffer(data: *const binding::fmi3UInt8, len: usize) -> Self {
        let slice = unsafe { std::slice::from_raw_parts(data, len) };
        let mut buffer = BytesMut::with_capacity(len * 2); // Some extra capacity
        buffer.extend_from_slice(slice);

        Self {
            buffer,
            read_pos: 0,
        }
    }
}
