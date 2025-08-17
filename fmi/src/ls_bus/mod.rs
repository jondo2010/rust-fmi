use std::ptr;

use fmi_sys::ls_bus::{
    fmi3LsBusCanData, fmi3LsBusCanId, fmi3LsBusCanIde, fmi3LsBusCanRtr, fmi3LsBusOperationHeader, FMI3_LS_BUS_CAN_OP_CANFD_TRANSMIT, FMI3_LS_BUS_CAN_OP_CAN_TRANSMIT, FMI3_LS_BUS_OP_FORMAT_ERROR
};
use zerocopy::{AsBytes, FromBytes, FromZeroes, Ref};

/// Safe wrapper for CAN ID with validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanId(u32);

impl CanId {
    pub const STANDARD_MAX: u32 = 0x7FF;
    pub const EXTENDED_MAX: u32 = 0x1FFFFFFF;

    pub fn standard(id: u16) -> Result<Self, &'static str> {
        if id as u32 <= Self::STANDARD_MAX {
            Ok(Self(id as u32))
        } else {
            Err("Standard CAN ID too large")
        }
    }

    pub fn extended(id: u32) -> Result<Self, &'static str> {
        if id <= Self::EXTENDED_MAX {
            Ok(Self(id))
        } else {
            Err("Extended CAN ID too large")
        }
    }

    pub fn raw(&self) -> u32 {
        self.0
    }

    pub fn is_extended(&self) -> bool {
        self.0 > Self::STANDARD_MAX
    }
}

impl From<CanId> for fmi3LsBusCanId {
    fn from(id: CanId) -> Self {
        id.0
    }
}

impl TryFrom<fmi3LsBusCanId> for CanId {
    type Error = &'static str;

    fn try_from(id: fmi3LsBusCanId) -> Result<Self, Self::Error> {
        if id <= Self::EXTENDED_MAX {
            Ok(Self(id))
        } else {
            Err("Invalid CAN ID")
        }
    }
}

/// Safe wrapper for CAN data with length validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanData {
    data: Vec<u8>,
    max_len: usize,
}

impl CanData {
    pub const CLASSIC_MAX: usize = 8;
    pub const FD_MAX: usize = 64;
    pub const XL_MAX: usize = 2048;

    pub fn classic(data: Vec<u8>) -> Result<Self, &'static str> {
        if data.len() <= Self::CLASSIC_MAX {
            Ok(Self { data, max_len: Self::CLASSIC_MAX })
        } else {
            Err("CAN data too long for classic CAN")
        }
    }

    pub fn fd(data: Vec<u8>) -> Result<Self, &'static str> {
        if data.len() <= Self::FD_MAX {
            Ok(Self { data, max_len: Self::FD_MAX })
        } else {
            Err("CAN data too long for CAN FD")
        }
    }

    pub fn xl(data: Vec<u8>) -> Result<Self, &'static str> {
        if data.len() <= Self::XL_MAX {
            Ok(Self { data, max_len: Self::XL_MAX })
        } else {
            Err("CAN data too long for CAN XL")
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Safe CAN transmit operation that can be zero-copy converted to C struct
#[derive(Debug, Clone)]
pub struct CanTransmitOperation {
    pub id: CanId,
    pub is_extended: bool,
    pub is_rtr: bool,
    pub data: CanData,
}

impl CanTransmitOperation {
    pub fn new(id: CanId, is_rtr: bool, data: Vec<u8>) -> Result<Self, &'static str> {
        Ok(Self {
            is_extended: id.is_extended(),
            id,
            is_rtr,
            data: CanData::classic(data)?,
        })
    }

    /// Serialize to bytes that match the C struct layout
    pub fn to_bytes(&self) -> Vec<u8> {
        let header_size = std::mem::size_of::<fmi3LsBusOperationHeader>();
        let fixed_size = header_size + 4 + 1 + 1 + 2; // id + ide + rtr + dataLength
        let total_size = fixed_size + self.data.len();

        let mut bytes = Vec::with_capacity(total_size);

        // Header
        let header = fmi3LsBusOperationHeader {
            opCode: FMI3_LS_BUS_CAN_OP_CAN_TRANSMIT,
            length: total_size as u32,
        };
        bytes.extend_from_slice(header.as_bytes());

        // Fixed fields
        bytes.extend_from_slice(&self.id.raw().to_le_bytes());
        bytes.push(if self.is_extended { 1 } else { 0 });
        bytes.push(if self.is_rtr { 1 } else { 0 });
        bytes.extend_from_slice(&(self.data.len() as u16).to_le_bytes());

        // Variable data
        bytes.extend_from_slice(self.data.as_slice());

        bytes
    }

    /// Parse from bytes that match the C struct layout
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < std::mem::size_of::<fmi3LsBusOperationHeader>() + 8 {
            return Err("Buffer too small for CAN transmit operation");
        }

        let header_size = std::mem::size_of::<fmi3LsBusOperationHeader>();
        
        // Parse header
        let header_bytes = &bytes[0..header_size];
        let header = Ref::<_, fmi3LsBusOperationHeader>::new(header_bytes)
            .ok_or("Invalid header alignment")?;

        if header.opCode != FMI3_LS_BUS_CAN_OP_CAN_TRANSMIT {
            return Err("Invalid operation code");
        }

        if bytes.len() < header.length as usize {
            return Err("Buffer too small for complete operation");
        }

        // Parse fixed fields
        let mut offset = header_size;
        let id_raw = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]);
        offset += 4;

        let is_extended = bytes[offset] != 0;
        offset += 1;

        let is_rtr = bytes[offset] != 0;
        offset += 1;

        let data_length = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
        offset += 2;

        if bytes.len() < offset + data_length {
            return Err("Buffer too small for data");
        }

        // Validate and create types
        let id = CanId::try_from(id_raw)?;
        let data = CanData::classic(bytes[offset..offset + data_length].to_vec())?;

        Ok(Self {
            id,
            is_extended,
            is_rtr,
            data,
        })
    }

    /// Get a reference to the underlying C struct (zero-copy)
    pub fn as_c_ref(&self) -> Option<Ref<&[u8], fmi3LsBusCanOperationCanTransmit>> {
        let bytes = self.to_bytes();
        Ref::new(&bytes[..])
    }
}

/// Safe CAN FD transmit operation
#[derive(Debug, Clone)]
pub struct CanFdTransmitOperation {
    pub id: CanId,
    pub is_extended: bool,
    pub bit_rate_switch: bool,
    pub error_state_indicator: bool,
    pub data: CanData,
}

impl CanFdTransmitOperation {
    pub fn new(
        id: CanId,
        bit_rate_switch: bool,
        error_state_indicator: bool,
        data: Vec<u8>,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            is_extended: id.is_extended(),
            id,
            bit_rate_switch,
            error_state_indicator,
            data: CanData::fd(data)?,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let header_size = std::mem::size_of::<fmi3LsBusOperationHeader>();
        let fixed_size = header_size + 4 + 1 + 1 + 1 + 2; // id + ide + brs + esi + dataLength
        let total_size = fixed_size + self.data.len();

        let mut bytes = Vec::with_capacity(total_size);

        // Header
        let header = fmi3LsBusOperationHeader {
            opCode: FMI3_LS_BUS_CAN_OP_CANFD_TRANSMIT,
            length: total_size as u32,
        };
        bytes.extend_from_slice(header.as_bytes());

        // Fixed fields
        bytes.extend_from_slice(&self.id.raw().to_le_bytes());
        bytes.push(if self.is_extended { 1 } else { 0 });
        bytes.push(if self.bit_rate_switch { 1 } else { 0 });
        bytes.push(if self.error_state_indicator { 1 } else { 0 });
        bytes.extend_from_slice(&(self.data.len() as u16).to_le_bytes());

        // Variable data
        bytes.extend_from_slice(self.data.as_slice());

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < std::mem::size_of::<fmi3LsBusOperationHeader>() + 9 {
            return Err("Buffer too small for CAN FD transmit operation");
        }

        let header_size = std::mem::size_of::<fmi3LsBusOperationHeader>();
        
        // Parse header
        let header_bytes = &bytes[0..header_size];
        let header = Ref::<_, fmi3LsBusOperationHeader>::new(header_bytes)
            .ok_or("Invalid header alignment")?;

        if header.opCode != FMI3_LS_BUS_CAN_OP_CANFD_TRANSMIT {
            return Err("Invalid operation code");
        }

        // Parse fixed fields
        let mut offset = header_size;
        let id_raw = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]);
        offset += 4;

        let is_extended = bytes[offset] != 0;
        offset += 1;
        let bit_rate_switch = bytes[offset] != 0;
        offset += 1;
        let error_state_indicator = bytes[offset] != 0;
        offset += 1;

        let data_length = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
        offset += 2;

        if bytes.len() < offset + data_length {
            return Err("Buffer too small for data");
        }

        let id = CanId::try_from(id_raw)?;
        let data = CanData::fd(bytes[offset..offset + data_length].to_vec())?;

        Ok(Self {
            id,
            is_extended,
            bit_rate_switch,
            error_state_indicator,
            data,
        })
    }
}

/// Safe CAN confirm operation
#[derive(Debug, Clone, Copy)]
pub struct CanConfirmOperation {
    pub id: CanId,
}

impl CanConfirmOperation {
    pub fn new(id: CanId) -> Self {
        Self { id }
    }

    pub fn to_bytes(&self) -> [u8; 12] {
        let mut bytes = [0u8; 12];
        
        // Header
        let header = fmi3LsBusOperationHeader {
            opCode: FMI3_LS_BUS_CAN_OP_CONFIRM,
            length: 12,
        };
        bytes[0..8].copy_from_slice(header.as_bytes());
        
        // ID
        bytes[8..12].copy_from_slice(&self.id.raw().to_le_bytes());
        
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 12 {
            return Err("Buffer too small for CAN confirm operation");
        }

        let header_bytes = &bytes[0..8];
        let header = Ref::<_, fmi3LsBusOperationHeader>::new(header_bytes)
            .ok_or("Invalid header alignment")?;

        if header.opCode != FMI3_LS_BUS_CAN_OP_CONFIRM {
            return Err("Invalid operation code");
        }

        let id_raw = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let id = CanId::try_from(id_raw)?;

        Ok(Self { id })
    }

    /// Get a reference to the underlying C struct (zero-copy)
    pub fn as_c_ref(&self) -> Ref<&[u8], fmi3LsBusCanOperationConfirm> {
        let bytes = self.to_bytes();
        Ref::new(&bytes[..]).expect("Static size should always work")
    }
}

/// Unified operation type for type-safe parsing
#[derive(Debug, Clone)]
pub enum CanOperation {
    Transmit(CanTransmitOperation),
    FdTransmit(CanFdTransmitOperation),
    Confirm(CanConfirmOperation),
    FormatError { data: Vec<u8> },
}

impl CanOperation {
    /// Parse any CAN operation from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < std::mem::size_of::<fmi3LsBusOperationHeader>() {
            return Err("Buffer too small for operation header");
        }

        let header_bytes = &bytes[0..std::mem::size_of::<fmi3LsBusOperationHeader>()];
        let header = Ref::<_, fmi3LsBusOperationHeader>::new(header_bytes)
            .ok_or("Invalid header alignment")?;

        match header.opCode {
            FMI3_LS_BUS_CAN_OP_CAN_TRANSMIT => {
                Ok(CanOperation::Transmit(CanTransmitOperation::from_bytes(bytes)?))
            }
            FMI3_LS_BUS_CAN_OP_CANFD_TRANSMIT => {
                Ok(CanOperation::FdTransmit(CanFdTransmitOperation::from_bytes(bytes)?))
            }
            FMI3_LS_BUS_CAN_OP_CONFIRM => {
                Ok(CanOperation::Confirm(CanConfirmOperation::from_bytes(bytes)?))
            }
            FMI3_LS_BUS_OP_FORMAT_ERROR => {
                // Parse format error
                if bytes.len() < 10 {
                    return Err("Buffer too small for format error");
                }
                let data_length = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
                if bytes.len() < 10 + data_length {
                    return Err("Buffer too small for format error data");
                }
                let data = bytes[10..10 + data_length].to_vec();
                Ok(CanOperation::FormatError { data })
            }
            _ => Err("Unknown operation code"),
        }
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            CanOperation::Transmit(op) => op.to_bytes(),
            CanOperation::FdTransmit(op) => op.to_bytes(),
            CanOperation::Confirm(op) => op.to_bytes().to_vec(),
            CanOperation::FormatError { data } => {
                let mut bytes = Vec::with_capacity(10 + data.len());
                let header = fmi3LsBusOperationHeader {
                    opCode: FMI3_LS_BUS_OP_FORMAT_ERROR,
                    length: (10 + data.len()) as u32,
                };
                bytes.extend_from_slice(header.as_bytes());
                bytes.extend_from_slice(&(data.len() as u16).to_le_bytes());
                bytes.extend_from_slice(data);
                bytes
            }
        }
    }
}

pub enum LsBusOperation {}

pub struct BufferInfo<'a> {
    buffer: &'a mut [u8],
    write_pos: usize,
    read_pos: usize,
}

impl<'a> BufferInfo<'a> {
    /// Creates a new BufferInfo from a mutable buffer slice.
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            write_pos: 0,
            read_pos: 0,
        }
    }

    /// Returns the remaining unread bytes in the buffer.
    pub fn remaining(&self) -> usize {
        self.write_pos.saturating_sub(self.read_pos)
    }

    /// Returns true if there are no unread bytes.
    pub fn is_empty(&self) -> bool {
        self.read_pos >= self.write_pos
    }

    /// Returns the unread portion of the buffer as a slice.
    pub fn unread_slice(&self) -> &[u8] {
        if self.read_pos < self.write_pos {
            &self.buffer[self.read_pos..self.write_pos]
        } else {
            &[]
        }
    }

    /// Reads the next bus operation from the buffer safely.
    /// 
    /// Returns `Some((header, data_slice))` if a complete operation is available,
    /// `None` if there's insufficient data for a complete operation.
    pub fn read_next_operation(&mut self) -> Option<(OperationHeader, &[u8])> {
        let unread = self.unread_slice();
        
        // Check if we have enough bytes for a header
        if unread.len() < std::mem::size_of::<OperationHeader>() {
            return None;
        }

        // Safely read the header using slice operations
        let header = self.read_operation_header(unread)?;
        
        // Check if we have the complete operation
        if unread.len() < header.length as usize {
            return None;
        }

        // Calculate the data portion (everything after the header)
        let header_size = std::mem::size_of::<OperationHeader>();
        let data_size = header.length as usize - header_size;
        let data_slice = &unread[header_size..header_size + data_size];

        // Advance the read position
        self.read_pos += header.length as usize;

        Some((header, data_slice))
    }

    /// Safely reads an operation header from a byte slice.
    fn read_operation_header(&self, data: &[u8]) -> Option<OperationHeader> {
        if data.len() < 8 {
            return None;
        }

        // Read op_code and length using safe slice operations
        let op_code = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let length = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

        Some(OperationHeader { op_code, length })
    }

    /// Iterator over operations in the buffer.
    pub fn operations(&mut self) -> OperationIterator<'_> {
        OperationIterator::new(self)
    }

    /// Writes operation data to the buffer.
    /// 
    /// Returns `Ok(())` if successful, `Err(())` if insufficient space.
    pub fn write_operation(&mut self, header: OperationHeader, data: &[u8]) -> Result<(), ()> {
        let total_size = std::mem::size_of::<OperationHeader>() + data.len();
        
        if self.write_pos + total_size > self.buffer.len() {
            return Err(());
        }

        // Write header
        let header_bytes = self.serialize_header(header);
        self.buffer[self.write_pos..self.write_pos + 8].copy_from_slice(&header_bytes);
        self.write_pos += 8;

        // Write data
        if !data.is_empty() {
            self.buffer[self.write_pos..self.write_pos + data.len()].copy_from_slice(data);
            self.write_pos += data.len();
        }

        Ok(())
    }

    /// Serializes an operation header to bytes.
    fn serialize_header(&self, header: OperationHeader) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0..4].copy_from_slice(&header.op_code.to_le_bytes());
        bytes[4..8].copy_from_slice(&header.length.to_le_bytes());
        bytes
    }

    /// Resets the buffer to empty state.
    pub fn reset(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
    }
}

/// Represents an operation header with proper validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationHeader {
    pub op_code: u32,
    pub length: u32,
}

impl OperationHeader {
    pub const SIZE: usize = 8;

    pub fn new(op_code: u32, length: u32) -> Self {
        Self { op_code, length }
    }

    /// Validates that the header has reasonable values.
    pub fn is_valid(&self) -> bool {
        // Length should at least include the header itself
        self.length >= Self::SIZE as u32 && 
        // Reasonable upper bound to prevent huge allocations
        self.length <= 1024 * 1024 // 1MB max
    }
}

/// Safe iterator over operations in a buffer.
pub struct OperationIterator<'a> {
    buffer_info: &'a mut BufferInfo<'a>,
}

impl<'a> OperationIterator<'a> {
    fn new(buffer_info: &'a mut BufferInfo<'a>) -> Self {
        Self { buffer_info }
    }
}

impl<'a> Iterator for OperationIterator<'a> {
    type Item = (OperationHeader, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let (header, data_slice) = self.buffer_info.read_next_operation()?;
        
        // Validate the header before returning
        if !header.is_valid() {
            return None;
        }

        Some((header, data_slice.to_vec()))
    }
}

/// Convenience functions for creating specific operation types.
impl<'a> BufferInfo<'a> {
    /// Creates a Format Error operation safely.
    pub fn create_format_error(&mut self, error_data: &[u8]) -> Result<(), ()> {
        let data_length = error_data.len() as u16;
        let total_length = OperationHeader::SIZE + 2 + error_data.len();
        
        let header = OperationHeader::new(
            FMI3_LS_BUS_OP_FORMAT_ERROR,
            total_length as u32
        );

        // Prepare the operation data (length + actual data)
        let mut op_data = Vec::with_capacity(2 + error_data.len());
        op_data.extend_from_slice(&data_length.to_le_bytes());
        op_data.extend_from_slice(error_data);

        self.write_operation(header, &op_data)
    }
}

impl LsBusCanExt for BufferInfo<'_> {
    fn create_op_can_transmit(
        &mut self,
        id: fmi3LsBusCanId,
        ide: fmi3LsBusCanIde,
        rtr: fmi3LsBusCanRtr,
        data: &[fmi3LsBusCanData],
    ) -> Result<(), ()> {
        let data_length = data.len() as u16;
        let total_length = OperationHeader::SIZE + 4 + 1 + 1 + 2 + data.len();
        
        let header = OperationHeader::new(
            fmi_sys::ls_bus::FMI3_LS_BUS_CAN_OP_CAN_TRANSMIT,
            total_length as u32
        );

        // Serialize the CAN transmit operation data
        let mut op_data = Vec::with_capacity(4 + 1 + 1 + 2 + data.len());
        op_data.extend_from_slice(&id.to_le_bytes());
        op_data.push(ide);
        op_data.push(rtr);
        op_data.extend_from_slice(&data_length.to_le_bytes());
        op_data.extend_from_slice(data);

        self.write_operation(header, &op_data)
    }

    fn create_op_can_confirm(&mut self, id: fmi3LsBusCanId) -> Result<(), ()> {
        let total_length = OperationHeader::SIZE + 4;
        
        let header = OperationHeader::new(
            fmi_sys::ls_bus::FMI3_LS_BUS_CAN_OP_CONFIRM,
            total_length as u32
        );

        let op_data = id.to_le_bytes();
        self.write_operation(header, &op_data)
    }
}

/// Type-safe wrapper for parsing specific operation types.
pub enum ParsedOperation {
    FormatError { data: Vec<u8> },
    CanTransmit { 
        id: u32,
        ide: u8,
        rtr: u8,
        data: Vec<u8>,
    },
    CanConfirm { id: u32 },
    Unknown { op_code: u32, data: Vec<u8> },
}

impl ParsedOperation {
    /// Safely parse an operation from header and data.
    pub fn parse(header: OperationHeader, data: &[u8]) -> Option<Self> {
        if !header.is_valid() {
            return None;
        }

        match header.op_code {
            FMI3_LS_BUS_OP_FORMAT_ERROR => Self::parse_format_error(data),
            fmi_sys::ls_bus::FMI3_LS_BUS_CAN_OP_CAN_TRANSMIT => Self::parse_can_transmit(data),
            fmi_sys::ls_bus::FMI3_LS_BUS_CAN_OP_CONFIRM => Self::parse_can_confirm(data),
            _ => Some(ParsedOperation::Unknown {
                op_code: header.op_code,
                data: data.to_vec(),
            }),
        }
    }

    fn parse_format_error(data: &[u8]) -> Option<Self> {
        if data.len() < 2 {
            return None;
        }

        let data_length = u16::from_le_bytes([data[0], data[1]]) as usize;
        if data.len() < 2 + data_length {
            return None;
        }

        let error_data = data[2..2 + data_length].to_vec();
        Some(ParsedOperation::FormatError { data: error_data })
    }

    fn parse_can_transmit(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }

        let id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let ide = data[4];
        let rtr = data[5];
        let data_length = u16::from_le_bytes([data[6], data[7]]) as usize;

        if data.len() < 8 + data_length {
            return None;
        }

        let can_data = data[8..8 + data_length].to_vec();
        Some(ParsedOperation::CanTransmit { id, ide, rtr, data: can_data })
    }

    fn parse_can_confirm(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        let id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        Some(ParsedOperation::CanConfirm { id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_buffer_operations() {
        let mut buffer = [0u8; 1024];
        let mut buffer_info = BufferInfo::new(&mut buffer);

        // Create a format error
        let error_data = vec![1, 2, 3, 4];
        buffer_info.create_format_error(&error_data).unwrap();

        // Read it back safely
        let (header, data) = buffer_info.read_next_operation().unwrap();
        assert_eq!(header.op_code, FMI3_LS_BUS_OP_FORMAT_ERROR);
        
        let parsed = ParsedOperation::parse(header, data).unwrap();
        match parsed {
            ParsedOperation::FormatError { data: parsed_data } => {
                assert_eq!(parsed_data, error_data);
            }
            _ => panic!("Expected FormatError"),
        }
    }

    #[test]
    fn test_insufficient_data() {
        let mut buffer = [0u8; 1024];
        let mut buffer_info = BufferInfo::new(&mut buffer);

        // Only write partial header
        buffer_info.buffer[0..4].copy_from_slice(&42u32.to_le_bytes());
        buffer_info.write_pos = 4;

        // Should return None due to insufficient data
        assert!(buffer_info.read_next_operation().is_none());
    }

    #[test]
    fn test_invalid_operation_length() {
        let mut buffer = [0u8; 1024];
        let mut buffer_info = BufferInfo::new(&mut buffer);

        // Write header with impossible length
        let header = OperationHeader::new(42, 2); // Length less than header size
        assert!(!header.is_valid());
    }

    #[test]
    fn test_can_id_validation() {
        assert!(CanId::standard(0x7FF).is_ok());
        assert!(CanId::standard(0x800).is_err());
        assert!(CanId::extended(0x1FFFFFFF).is_ok());
        assert!(CanId::extended(0x20000000).is_err());
    }

    #[test]
    fn test_can_transmit_roundtrip() {
        let id = CanId::standard(0x123).unwrap();
        let data = vec![1, 2, 3, 4];
        let op = CanTransmitOperation::new(id, false, data.clone()).unwrap();
        
        let bytes = op.to_bytes();
        let parsed = CanTransmitOperation::from_bytes(&bytes).unwrap();
        
        assert_eq!(parsed.id, id);
        assert!(!parsed.is_rtr);
        assert_eq!(parsed.data.as_slice(), &data);
    }

    #[test]
    fn test_zerocopy_safety() {
        let id = CanId::standard(0x456).unwrap();
        let confirm = CanConfirmOperation::new(id);
        
        let bytes = confirm.to_bytes();
        let parsed = CanConfirmOperation::from_bytes(&bytes).unwrap();
        
        assert_eq!(parsed.id, id);
    }

    #[test]
    fn test_buffer_operations() {
        let mut buffer = [0u8; 1024];
        let mut buffer_info = BufferInfo::new(&mut buffer);

        let id = CanId::standard(0x123).unwrap();
        let transmit = CanTransmitOperation::new(id, false, vec![1, 2, 3]).unwrap();
        let confirm = CanConfirmOperation::new(id);

        // Write operations
        buffer_info.write_can_operation(&CanOperation::Transmit(transmit.clone())).unwrap();
        buffer_info.write_can_operation(&CanOperation::Confirm(confirm)).unwrap();

        // Read them back
        let op1 = buffer_info.read_can_operation().unwrap();
        let op2 = buffer_info.read_can_operation().unwrap();

        match (op1, op2) {
            (CanOperation::Transmit(t), CanOperation::Confirm(c)) => {
                assert_eq!(t.id, id);
                assert_eq!(c.id, id);
            }
            _ => panic!("Unexpected operation types"),
        }
    }
}
