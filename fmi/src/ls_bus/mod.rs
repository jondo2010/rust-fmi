//! Rust bindings for FMI-LS-BUS interface
//!
//! This module provides a safe, ergonomic Rust interface for FMI-LS-BUS operations
//! that is binary compatible with the C implementation.

use bytes::{BufMut, BytesMut};
use std::mem;

use crate::fmi3::binding;
use fmi_sys::ls_bus;

/// A Rust wrapper around FMI-LS-BUS buffer operations using the `bytes` crate.
///
/// This provides a safe, ergonomic interface that's binary compatible with
/// the C `fmi3LsBusUtilBufferInfo` structure and associated macros.
#[derive(Debug)]
pub struct LsBusBuffer {
    buffer: BytesMut,
    read_pos: usize,
    status: bool,
}

impl LsBusBuffer {
    /// Create a new LS-BUS buffer with the specified capacity.
    ///
    /// This is equivalent to initializing `fmi3LsBusUtilBufferInfo` with
    /// `FMI3_LS_BUS_BUFFER_INFO_INIT`.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            read_pos: 0,
            status: true,
        }
    }

    /// Reset the buffer to empty state.
    ///
    /// Equivalent to `FMI3_LS_BUS_BUFFER_INFO_RESET`.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.read_pos = 0;
        self.status = true;
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

    /// Get the status of the last operation.
    pub fn status(&self) -> bool {
        self.status
    }

    /// Write raw data to the buffer, replacing existing content.
    ///
    /// Equivalent to `FMI3_LS_BUS_BUFFER_WRITE`.
    pub fn write(&mut self, data: &[u8]) {
        if data.len() <= self.buffer.capacity() {
            self.buffer.clear();
            self.buffer.extend_from_slice(data);
            self.read_pos = 0;
            self.status = true;
        } else {
            self.status = false;
        }
    }

    /// Create a CAN baudrate configuration operation.
    ///
    /// This is the Rust equivalent of the C macro:
    /// `FMI3_LS_BUS_CAN_CREATE_OP_CONFIGURATION_CAN_BAUDRATE`
    pub fn create_can_baudrate_config(&mut self, baudrate: binding::fmi3UInt32) {
        let op = ls_bus::fmi3LsBusCanOperationConfiguration {
            header: ls_bus::fmi3LsBusOperationHeader {
                opCode: ls_bus::FMI3_LS_BUS_CAN_OP_CONFIGURATION,
                length: std::mem::size_of::<ls_bus::fmi3LsBusCanOperationConfiguration>()
                    as binding::fmi3UInt32,
            },
            parameterType: ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_CAN_BAUDRATE,
            __bindgen_anon_1: ls_bus::fmi3LsBusCanOperationConfiguration__bindgen_ty_1 { baudrate },
        };

        // Calculate total size needed
        let op_size = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationConfiguration>();

        // Check if we have enough space
        if self.buffer.remaining_mut() >= op_size {
            // Safety: We're transmuting to bytes for a packed, repr(C) struct
            // This is equivalent to the memcpy operations in the C macro
            let op_bytes =
                unsafe { std::slice::from_raw_parts(&op as *const _ as *const u8, op_size) };

            self.buffer.extend_from_slice(op_bytes);
            self.status = true;
        } else {
            self.status = false;
        }
    }

    /// Read the next operation from the buffer.
    ///
    /// Returns `Some((op_code, data))` if an operation is available,
    /// or `None` if no complete operation is available.
    ///
    /// This is equivalent to `FMI3_LS_BUS_READ_NEXT_OPERATION`.
    pub fn read_next_operation(&mut self) -> Option<(binding::fmi3UInt32, &[u8])> {
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

        // Check if we have the complete operation
        if remaining < header.length as usize {
            return None;
        }

        // Extract operation data
        let op_data = &self.buffer[self.read_pos..self.read_pos + header.length as usize];
        self.read_pos += header.length as usize;

        Some((header.opCode, op_data))
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
            status: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = LsBusBuffer::new(1024);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert!(buffer.status());
    }

    #[test]
    fn test_can_baudrate_config() {
        let mut buffer = LsBusBuffer::new(1024);
        let baudrate = 500000; // 500 kbps

        buffer.create_can_baudrate_config(baudrate);
        assert!(buffer.status());
        assert!(!buffer.is_empty());

        // Read back the operation
        let (op_code, _data) = buffer.read_next_operation().unwrap();
        assert_eq!(op_code, ls_bus::FMI3_LS_BUS_CAN_OP_CONFIGURATION);
    }

    #[test]
    fn test_binary_compatibility() {
        // Test that our structures have the same size as the C equivalents
        assert_eq!(mem::size_of::<ls_bus::fmi3LsBusOperationHeader>(), 8); // As per C static_assert

        // The minimum size from the C static_assert
        assert!(mem::size_of::<ls_bus::fmi3LsBusCanOperationConfiguration>() >= 8 + 4 + 1);
    }

    #[test]
    fn test_buffer_reset() {
        let mut buffer = LsBusBuffer::new(1024);
        buffer.create_can_baudrate_config(125000);
        assert!(!buffer.is_empty());

        buffer.reset();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_ffi_compatibility() {
        let mut buffer = LsBusBuffer::new(1024);
        buffer.create_can_baudrate_config(250000);

        let (ptr, len) = buffer.as_fmi_binary();
        assert!(!ptr.is_null());
        assert!(len > 0);

        // Test round-trip through FFI
        let buffer2 = unsafe { LsBusBuffer::from_raw_buffer(ptr, len) };
        assert_eq!(buffer2.len(), len);
    }
}
