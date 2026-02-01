#![doc = include_str!("../README.md")]

use std::mem;

use fmi_sys::ls_bus;

#[cfg(feature = "can")]
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
    /// Transmit the operation by writing it into the provided LS-BUS buffer slice.
    /// Returns the number of bytes written.
    fn transmit(self, buffer: &mut [u8]) -> Result<usize, FmiLsBusError>;

    fn read_next_operation(
        buffer: &'a [u8],
        read_pos: &mut usize,
    ) -> Result<Option<Self>, FmiLsBusError>;
}

/// A Rust wrapper around FMI-LS-BUS buffer operations.
///
/// This provides a safe, ergonomic interface that operates on external buffers
/// and is binary compatible with the C `fmi3LsBusUtilBufferInfo` structure.
///
/// The struct tracks read and write positions while the actual buffer data
/// is owned externally.
#[derive(Debug, Default)]
pub struct FmiLsBus {
    pub read_pos: usize,
    pub write_pos: usize,
}

impl FmiLsBus {
    /// Create a new LS-BUS helper. The buffer itself is provided externally.
    pub fn new() -> Self {
        Self {
            read_pos: 0,
            write_pos: 0,
        }
    }

    /// Reset the buffer to empty state by setting length and read position to 0.
    ///
    /// Equivalent to `FMI3_LS_BUS_BUFFER_INFO_RESET`.
    pub fn reset(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
    }

    /// Get the start address of the buffer data.
    ///
    /// Equivalent to `FMI3_LS_BUS_BUFFER_START`.
    pub fn start(buffer: &[u8]) -> *const u8 {
        buffer.as_ptr()
    }

    pub fn write_operation<'a, OP: LsBusOperation<'a>>(
        &mut self,
        op: OP,
        buffer: &mut [u8],
    ) -> Result<(), FmiLsBusError> {
        // Get the slice starting at the current write position
        let available_slice = &mut buffer[self.write_pos..];
        let bytes_written = op.transmit(available_slice)?;
        self.write_pos += bytes_written;
        Ok(())
    }

    /// Write raw data to the buffer, replacing existing content.
    ///
    /// Equivalent to `FMI3_LS_BUS_BUFFER_WRITE`.
    pub fn write(&mut self, buffer: &mut [u8], data: &[u8]) -> Result<(), FmiLsBusError> {
        if data.len() <= buffer.len() {
            let copy_len = data.len();
            buffer[..copy_len].copy_from_slice(data);
            self.read_pos = 0;
            self.write_pos = copy_len;
            Ok(())
        } else {
            Err(FmiLsBusError::BufferOverflow)
        }
    }

    /// Peek at the next operation's opcode and length without advancing the read position.
    pub fn peek_next_operation(
        &self,
        buffer: &[u8],
    ) -> Option<(ls_bus::fmi3LsBusOperationCode, usize)> {
        let remaining = buffer.len() - self.read_pos;

        // Need at least header size
        if remaining < mem::size_of::<ls_bus::fmi3LsBusOperationHeader>() {
            return None;
        }

        // Read header
        let header_bytes = &buffer
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
    /// Returns `Some(op)` if an operation is available,
    /// or `None` if no complete operation is available.
    ///
    /// This is equivalent to `FMI3_LS_BUS_READ_NEXT_OPERATION`.
    pub fn read_next_operation<'a, OP: LsBusOperation<'a>>(
        &mut self,
        buffer: &'a [u8],
    ) -> Result<Option<OP>, FmiLsBusError> {
        OP::read_next_operation(buffer, &mut self.read_pos)
    }
}
