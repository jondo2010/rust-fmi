use crate::{FmiLsBusError, LsBusOperation};
use fmi::fmi3::binding;
use fmi_sys::ls_bus;
use std::borrow::Cow;

#[cfg(test)]
mod tests;

#[cfg(feature = "fmi-export")]
use fmi_export::{
    FmuModel,
    fmi3::{Binary, Clock},
};

/// MIME type for FMI-LS-BUS CAN payloads.
pub const CAN_MIME_TYPE: &str =
    "application/org.fmi-standard.fmi-ls-bus.can; version=\"1.0.0-beta.1\"";

/// Default maximum CAN bus buffer size.
pub const DEFAULT_CAN_BUFFER_SIZE: usize = 2048;

/// Reusable CAN bus component for FMU models.
#[cfg(feature = "fmi-export")]
#[derive(FmuModel, Debug, Default)]
pub struct CanBus {
    #[variable(
        name = "Rx_Data",
        causality = Input,
        variability = Discrete,
        initial = Exact,
        max_size = 2048,
        clocks = [rx_clock],
        mime_type = "application/org.fmi-standard.fmi-ls-bus.can; version=\"1.0.0-beta.1\"",
        start = b""
    )]
    pub rx_data: Binary,

    #[variable(
        name = "Tx_Data",
        causality = Output,
        variability = Discrete,
        initial = Calculated,
        max_size = 2048,
        clocks = [tx_clock],
        mime_type = "application/org.fmi-standard.fmi-ls-bus.can; version=\"1.0.0-beta.1\"",
    )]
    pub tx_data: Binary,

    #[variable(name = "Rx_Clock", causality = Input, interval_variability = Triggered)]
    pub rx_clock: Clock,

    #[variable(name = "Tx_Clock", causality = Output, interval_variability = Triggered)]
    pub tx_clock: Clock,

    #[variable(skip)]
    pub rx_bus: crate::FmiLsBus,

    #[variable(skip)]
    pub tx_bus: crate::FmiLsBus,
}

#[cfg(feature = "fmi-export")]
impl CanBus {
    /// Reset both RX/TX buffers and associated LS-BUS state.
    pub fn reset_buffers(&mut self) {
        self.rx_bus.reset();
        self.tx_bus.reset();
        self.rx_data.0.clear();
        self.tx_data.0.clear();
    }

    /// Returns true when the RX clock is triggered.
    pub fn rx_triggered(&self) -> bool {
        *self.rx_clock
    }

    /// Read the next CAN operation from the RX buffer (if any).
    pub fn read_next_operation<'a>(
        &'a mut self,
    ) -> Result<Option<LsBusCanOp<'a>>, FmiLsBusError> {
        self.rx_bus.read_next_operation(&mut self.rx_data)
    }

    /// Invoke `handler` for every available RX operation when the RX clock is triggered.
    ///
    /// This is a convenience wrapper for `rx_triggered` + `read_next_operation` loops.
    pub fn process_rx<F>(&mut self, mut handler: F) -> Result<(), FmiLsBusError>
    where
        F: for<'a> FnMut(LsBusCanOp<'a>),
    {
        if !self.rx_triggered() {
            return Ok(());
        }

        while let Some(op) = self.read_next_operation()? {
            handler(op);
        }

        Ok(())
    }

    /// Clear RX state, reset buffer, and lower the RX clock.
    pub fn clear_rx(&mut self) {
        *self.rx_clock = false;
        self.rx_bus.reset();
        self.rx_data.0.clear();
    }

    /// Clear TX state, reset buffer, and lower the TX clock.
    pub fn clear_tx(&mut self) {
        *self.tx_clock = false;
        self.tx_bus.reset();
        self.tx_data.0.clear();
    }

    /// Clear both RX and TX state after event handling.
    pub fn clear_after_event(&mut self) {
        self.clear_rx();
        self.clear_tx();
    }

    /// Ensure the TX buffer is large enough for upcoming writes.
    pub fn ensure_tx_buffer_capacity(&mut self) {
        let needed_len = DEFAULT_CAN_BUFFER_SIZE.max(self.tx_bus.write_pos);
        if self.tx_data.0.len() < needed_len {
            self.tx_data.0.resize(needed_len, 0);
        }
    }

    /// Prepare a TX batch and return the initial write position.
    pub fn begin_tx_batch(&mut self) -> usize {
        self.ensure_tx_buffer_capacity();
        self.tx_bus.write_pos
    }

    /// Write a CAN operation into the TX buffer.
    pub fn write_operation<'a>(&mut self, op: LsBusCanOp<'a>) -> Result<(), FmiLsBusError> {
        self.ensure_tx_buffer_capacity();
        self.tx_bus.write_operation(op, &mut self.tx_data)
    }

    /// Finalize the TX buffer by truncating to the write position.
    pub fn shrink_tx_buffer(&mut self) {
        self.tx_data.0.truncate(self.tx_bus.write_pos);
    }

    /// Finalize a TX batch and toggle the TX clock if data was written.
    ///
    /// Returns true if any data was written.
    pub fn finalize_tx_batch(&mut self, initial_write_pos: usize) -> bool {
        let wrote_data = self.tx_bus.write_pos > initial_write_pos;
        self.shrink_tx_buffer();
        if wrote_data {
            *self.tx_clock = true;
        }
        wrote_data
    }

    /// Convenience helper to run a TX batch and toggle the TX clock if data was written.
    ///
    /// Returns true if any data was written.
    pub fn tx_send_batch<F>(&mut self, mut writer: F) -> Result<bool, FmiLsBusError>
    where
        F: FnMut(&mut Self) -> Result<(), FmiLsBusError>,
    {
        let initial_write_pos = self.begin_tx_batch();
        writer(self)?;
        Ok(self.finalize_tx_batch(initial_write_pos))
    }
}

/// Helper function to peek at the next operation without requiring FmiLsBus instance
fn peek_next_operation_helper(
    buffer: &[u8],
    read_pos: usize,
) -> Option<(ls_bus::fmi3LsBusOperationCode, usize)> {
    let remaining = buffer.len() - read_pos;

    // Need at least header size
    if remaining < std::mem::size_of::<ls_bus::fmi3LsBusOperationHeader>() {
        return None;
    }

    // Read header
    let header_bytes =
        &buffer[read_pos..read_pos + std::mem::size_of::<ls_bus::fmi3LsBusOperationHeader>()];
    let header = unsafe {
        std::ptr::read_unaligned(header_bytes.as_ptr() as *const ls_bus::fmi3LsBusOperationHeader)
    };

    Some((header.opCode, header.length as usize))
}

/// Helper function to check if buffer has enough remaining capacity
fn check_buffer_capacity(buffer: &[u8], needed_size: usize) -> Result<(), FmiLsBusError> {
    if needed_size > buffer.len() {
        Err(FmiLsBusError::BufferOverflow)
    } else {
        Ok(())
    }
}

/// CAN bus operations that can be transmitted over FMI-LS-BUS.
///
/// This enum represents the different types of CAN operations that can be
/// serialized and transmitted between FMUs using the FMI-LS-BUS interface.
///
/// # Example
///
/// Creating and sending a basic CAN message:
///
/// ```rust
/// use fmi_ls_bus::{FmiLsBus, can::LsBusCanOp};
/// use std::borrow::Cow;
///
/// let mut bus = FmiLsBus::new();
/// let mut buffer = vec![0u8; 1024]; // Pre-allocate buffer
///
/// // Create a CAN transmit operation
/// bus.write_operation(LsBusCanOp::Transmit {
///     id: 0x123,
///     ide: 0,  // Standard ID
///     rtr: 0,  // Data frame
///     data: Cow::Borrowed(b"Hello"),
/// }, &mut buffer).unwrap();
///
/// // Read the operation back
/// let operation = bus.read_next_operation(&buffer[..bus.write_pos]).unwrap();
/// match operation {
///     Some(LsBusCanOp::Transmit { id, data, .. }) => {
///         println!("Received CAN message ID: 0x{:X}, Data: {:?}", id, data);
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug)]
pub enum LsBusCanOp<'a> {
    /// CAN transmit operation
    Transmit {
        id: ls_bus::fmi3LsBusCanId,
        ide: ls_bus::fmi3LsBusCanIde,
        rtr: ls_bus::fmi3LsBusCanRtr,
        data: Cow<'a, [u8]>,
    },
    /// CAN FD transmit operation
    FdTransmit {
        id: ls_bus::fmi3LsBusCanId,
        ide: ls_bus::fmi3LsBusCanIde,
        brs: ls_bus::fmi3LsBusCanBrs,
        esi: ls_bus::fmi3LsBusCanEsi,
        data: Cow<'a, [u8]>,
    },
    /// CAN XL transmit operation
    XlTransmit {
        id: ls_bus::fmi3LsBusCanId,
        ide: ls_bus::fmi3LsBusCanIde,
        sec: ls_bus::fmi3LsBusCanSec,
        sdt: ls_bus::fmi3LsBusCanSdt,
        vcid: ls_bus::fmi3LsBusCanVcId,
        af: ls_bus::fmi3LsBusCanAf,
        data: Cow<'a, [u8]>,
    },
    /// CAN confirm operation
    Confirm(ls_bus::fmi3LsBusCanId),
    /// CAN configuration operation baud rate setting
    ConfigBaudrate(ls_bus::fmi3LsBusCanBaudrate),
    /// CAN configuration operation FD baud rate setting
    ConfigFdBaudrate,
    /// CAN configuration operation XL baud rate setting
    ConfigXlBaudrate,
    /// CAN configuration operation for the arbitration lost behavior setting
    ConfigArbitrationLost(LsBusCanArbitrationLostBehavior),
    /// CAN arbitration lost operation
    ArbitrationLost { id: ls_bus::fmi3LsBusCanId },
    /// CAN bus error operation
    BusError {
        id: ls_bus::fmi3LsBusCanId,
        error_code: LsBusCanErrorCode,
        error_flags: LsBusCanErrorFlag,
        /// Whether the error occurred in response to a transmission of this FMU
        is_sender: bool,
    },
    /// CAN status operation
    Status(LsBusCanStatusKind),
    /// CAN wakeup operation
    Wakeup,
}

#[derive(Debug)]
#[repr(u8)]
pub enum LsBusCanArbitrationLostBehavior {
    /// On arbitration lost, buffer the message and retransmit later.
    BufferAndRetransmit =
        ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_ARBITRATION_LOST_BEHAVIOR_BUFFER_AND_RETRANSMIT,
    /// On arbitration lost, discard the message and notify the user.
    DiscardAndNotify =
        ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_ARBITRATION_LOST_BEHAVIOR_DISCARD_AND_NOTIFY,
}

#[derive(Debug)]
#[repr(u8)]
pub enum LsBusCanErrorCode {
    /// Represents a CAN bus error of type 'BIT_ERROR'.
    BitError = ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_BIT_ERROR,
    /// Represents a CAN bus error of type 'BIT_STUFFING_ERROR'.
    BitStuffingError = ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_BIT_STUFFING_ERROR,
    /// Represents a CAN bus error of type 'FORM_ERROR'.
    FormError = ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_FORM_ERROR,
    /// Represents a CAN bus error of type 'CRC_ERROR'.
    CrcError = ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_CRC_ERROR,
    /// Represents a CAN bus error of type 'ACK_ERROR'.
    AckError = ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_ACK_ERROR,
    /// Represents a CAN bus error of type 'BROKEN_ERROR_FRAME'.
    BrokenErrorFrame = ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_BROKEN_ERROR_FRAME,
}

impl TryFrom<ls_bus::fmi3LsBusCanErrorCode> for LsBusCanErrorCode {
    type Error = super::FmiLsBusError;
    fn try_from(value: ls_bus::fmi3LsBusCanErrorCode) -> Result<Self, Self::Error> {
        match value {
            ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_BIT_ERROR => {
                Ok(LsBusCanErrorCode::BitError)
            }
            ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_BIT_STUFFING_ERROR => {
                Ok(LsBusCanErrorCode::BitStuffingError)
            }
            ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_FORM_ERROR => {
                Ok(LsBusCanErrorCode::FormError)
            }
            ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_CRC_ERROR => {
                Ok(LsBusCanErrorCode::CrcError)
            }
            ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_ACK_ERROR => {
                Ok(LsBusCanErrorCode::AckError)
            }
            ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_BROKEN_ERROR_FRAME => {
                Ok(LsBusCanErrorCode::BrokenErrorFrame)
            }
            _ => Err(super::FmiLsBusError::InvalidVariant(value as u32)),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum LsBusCanErrorFlag {
    /// Indicates that a specified Network FMU is detecting the given Bus Error first.
    Primary = ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_FLAG_PRIMARY_ERROR_FLAG,

    /// Indicates that a specified Network FMU is reacting on a Bus Error and does not detect it.
    Secondary = ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_FLAG_SECONDARY_ERROR_FLAG,
}

impl TryFrom<ls_bus::fmi3LsBusCanErrorFlag> for LsBusCanErrorFlag {
    type Error = super::FmiLsBusError;
    fn try_from(value: ls_bus::fmi3LsBusCanErrorFlag) -> Result<Self, Self::Error> {
        match value {
            ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_FLAG_PRIMARY_ERROR_FLAG => {
                Ok(LsBusCanErrorFlag::Primary)
            }
            ls_bus::FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_FLAG_SECONDARY_ERROR_FLAG => {
                Ok(LsBusCanErrorFlag::Secondary)
            }
            _ => Err(super::FmiLsBusError::InvalidVariant(value as u32)),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum LsBusCanStatusKind {
    /// Indicates that the CAN node is in state 'ERROR_ACTIVE'.
    ErrorActive = ls_bus::FMI3_LS_BUS_CAN_STATUS_PARAM_STATUS_KIND_ERROR_ACTIVE,
    /// Indicates that the CAN node is in state 'ERROR_PASSIVE'.
    ErrorPassive = ls_bus::FMI3_LS_BUS_CAN_STATUS_PARAM_STATUS_KIND_ERROR_PASSIVE,
    /// Indicates that the CAN node is in state 'BUS_OFF'.
    BusOff = ls_bus::FMI3_LS_BUS_CAN_STATUS_PARAM_STATUS_KIND_BUS_OFF,
}

impl TryFrom<ls_bus::fmi3LsBusCanStatusKind> for LsBusCanStatusKind {
    type Error = super::FmiLsBusError;
    fn try_from(value: ls_bus::fmi3LsBusCanStatusKind) -> Result<Self, Self::Error> {
        match value {
            ls_bus::FMI3_LS_BUS_CAN_STATUS_PARAM_STATUS_KIND_ERROR_ACTIVE => {
                Ok(LsBusCanStatusKind::ErrorActive)
            }
            ls_bus::FMI3_LS_BUS_CAN_STATUS_PARAM_STATUS_KIND_ERROR_PASSIVE => {
                Ok(LsBusCanStatusKind::ErrorPassive)
            }
            ls_bus::FMI3_LS_BUS_CAN_STATUS_PARAM_STATUS_KIND_BUS_OFF => {
                Ok(LsBusCanStatusKind::BusOff)
            }
            _ => Err(super::FmiLsBusError::InvalidVariant(value as u32)),
        }
    }
}

impl<'a> LsBusOperation<'a> for LsBusCanOp<'a> {
    fn transmit(self, buffer: &mut [u8]) -> Result<usize, FmiLsBusError> {
        match self {
            LsBusCanOp::Transmit { id, ide, rtr, data } => {
                let op_size =
                    std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanTransmit>() + data.len();

                // Check if buffer has enough capacity
                check_buffer_capacity(buffer, op_size)?;

                let op = ls_bus::fmi3LsBusCanOperationCanTransmit {
                    header: ls_bus::fmi3LsBusOperationHeader {
                        opCode: ls_bus::FMI3_LS_BUS_CAN_OP_CAN_TRANSMIT,
                        length: op_size as ls_bus::fmi3LsBusOperationLength,
                    },
                    id,
                    ide,
                    rtr,
                    dataLength: data.len() as ls_bus::fmi3LsBusDataLength,
                    data: Default::default(),
                };

                // Write only the fixed part of the struct (without the flexible array member 'data')
                let fixed_size = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanTransmit>();
                let op_bytes =
                    unsafe { std::slice::from_raw_parts(&op as *const _ as *const u8, fixed_size) };

                buffer[0..fixed_size].copy_from_slice(op_bytes);

                // Append actual data immediately after the struct
                buffer[fixed_size..fixed_size + data.len()].copy_from_slice(data.as_ref());

                Ok(op_size)
            }
            LsBusCanOp::FdTransmit {
                id,
                ide,
                brs,
                esi,
                data,
            } => {
                let op_size =
                    std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanFdTransmit>() + data.len();

                check_buffer_capacity(buffer, op_size)?;

                let op = ls_bus::fmi3LsBusCanOperationCanFdTransmit {
                    header: ls_bus::fmi3LsBusOperationHeader {
                        opCode: ls_bus::FMI3_LS_BUS_CAN_OP_CANFD_TRANSMIT,
                        length: op_size as ls_bus::fmi3LsBusOperationLength,
                    },
                    id,
                    ide,
                    brs,
                    esi,
                    dataLength: data.len() as ls_bus::fmi3LsBusCanDataLength,
                    data: Default::default(),
                };

                let fixed_size = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanFdTransmit>();
                let op_bytes =
                    unsafe { std::slice::from_raw_parts(&op as *const _ as *const u8, fixed_size) };

                buffer[0..fixed_size].copy_from_slice(op_bytes);
                buffer[fixed_size..fixed_size + data.len()].copy_from_slice(data.as_ref());

                Ok(op_size)
            }
            LsBusCanOp::XlTransmit {
                id,
                ide,
                sec,
                sdt,
                vcid,
                af,
                data,
            } => {
                let op_size =
                    std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanXlTransmit>() + data.len();

                check_buffer_capacity(buffer, op_size)?;

                let op = ls_bus::fmi3LsBusCanOperationCanXlTransmit {
                    header: ls_bus::fmi3LsBusOperationHeader {
                        opCode: ls_bus::FMI3_LS_BUS_CAN_OP_CANXL_TRANSMIT,
                        length: op_size as ls_bus::fmi3LsBusOperationLength,
                    },
                    id,
                    ide,
                    sec,
                    sdt,
                    vcid,
                    af,
                    dataLength: data.len() as ls_bus::fmi3LsBusCanDataLength,
                    data: Default::default(),
                };

                let fixed_size = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanXlTransmit>();
                let op_bytes =
                    unsafe { std::slice::from_raw_parts(&op as *const _ as *const u8, fixed_size) };

                buffer[0..fixed_size].copy_from_slice(op_bytes);
                buffer[fixed_size..fixed_size + data.len()].copy_from_slice(data.as_ref());

                Ok(op_size)
            }
            LsBusCanOp::Confirm(id) => {
                let op_size = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationConfirm>();

                check_buffer_capacity(buffer, op_size)?;

                let op = ls_bus::fmi3LsBusCanOperationConfirm {
                    header: ls_bus::fmi3LsBusOperationHeader {
                        opCode: ls_bus::FMI3_LS_BUS_CAN_OP_CONFIRM,
                        length: op_size as binding::fmi3UInt32,
                    },
                    id,
                };

                let op_bytes =
                    unsafe { std::slice::from_raw_parts(&op as *const _ as *const u8, op_size) };

                buffer[0..op_size].copy_from_slice(op_bytes);

                Ok(op_size)
            }
            LsBusCanOp::ConfigBaudrate(baud_rate) => {
                let op_size = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationConfiguration>();

                check_buffer_capacity(buffer, op_size)?;

                let op = ls_bus::fmi3LsBusCanOperationConfiguration {
                    header: ls_bus::fmi3LsBusOperationHeader {
                        opCode: ls_bus::FMI3_LS_BUS_CAN_OP_CONFIGURATION,
                        length: op_size as binding::fmi3UInt32,
                    },
                    parameterType: ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_CAN_BAUDRATE,
                    __bindgen_anon_1: ls_bus::fmi3LsBusCanOperationConfiguration__bindgen_ty_1 {
                        baudrate: baud_rate,
                    },
                };

                let op_bytes =
                    unsafe { std::slice::from_raw_parts(&op as *const _ as *const u8, op_size) };

                buffer[0..op_size].copy_from_slice(op_bytes);

                Ok(op_size)
            }
            LsBusCanOp::ConfigFdBaudrate => todo!(),
            LsBusCanOp::ConfigXlBaudrate => todo!(),
            LsBusCanOp::ConfigArbitrationLost(behavior) => {
                let op_size = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationConfiguration>();

                check_buffer_capacity(buffer, op_size)?;

                let op = ls_bus::fmi3LsBusCanOperationConfiguration {
                    header: ls_bus::fmi3LsBusOperationHeader {
                        opCode: ls_bus::FMI3_LS_BUS_CAN_OP_CONFIGURATION,
                        length: op_size as binding::fmi3UInt32,
                    },
                    parameterType:
                        ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_ARBITRATION_LOST_BEHAVIOR,
                    __bindgen_anon_1: ls_bus::fmi3LsBusCanOperationConfiguration__bindgen_ty_1 {
                        arbitrationLostBehavior: behavior as binding::fmi3UInt8,
                    },
                };

                let op_bytes =
                    unsafe { std::slice::from_raw_parts(&op as *const _ as *const u8, op_size) };

                buffer[0..op_size].copy_from_slice(op_bytes);

                Ok(op_size)
            }
            LsBusCanOp::ArbitrationLost { id } => {
                let op_size = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationArbitrationLost>();

                check_buffer_capacity(buffer, op_size)?;

                let op = ls_bus::fmi3LsBusCanOperationArbitrationLost {
                    header: ls_bus::fmi3LsBusOperationHeader {
                        opCode: ls_bus::FMI3_LS_BUS_CAN_OP_ARBITRATION_LOST,
                        length: op_size as binding::fmi3UInt32,
                    },
                    id,
                };

                let op_bytes =
                    unsafe { std::slice::from_raw_parts(&op as *const _ as *const u8, op_size) };

                buffer[0..op_size].copy_from_slice(op_bytes);

                Ok(op_size)
            }
            LsBusCanOp::BusError {
                id,
                error_code,
                error_flags,
                is_sender,
            } => {
                let op_size = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationBusError>();

                check_buffer_capacity(buffer, op_size)?;

                let op = ls_bus::fmi3LsBusCanOperationBusError {
                    header: ls_bus::fmi3LsBusOperationHeader {
                        opCode: ls_bus::FMI3_LS_BUS_CAN_OP_BUS_ERROR,
                        length: op_size as binding::fmi3UInt32,
                    },
                    id,
                    errorCode: error_code as ls_bus::fmi3LsBusCanErrorCode,
                    errorFlag: error_flags as ls_bus::fmi3LsBusCanErrorFlag,
                    isSender: if is_sender { 1 } else { 0 },
                };

                let op_bytes =
                    unsafe { std::slice::from_raw_parts(&op as *const _ as *const u8, op_size) };

                buffer[0..op_size].copy_from_slice(op_bytes);

                Ok(op_size)
            }
            LsBusCanOp::Status(kind) => {
                let op_size = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationStatus>();

                check_buffer_capacity(buffer, op_size)?;

                let op = ls_bus::fmi3LsBusCanOperationStatus {
                    header: ls_bus::fmi3LsBusOperationHeader {
                        opCode: ls_bus::FMI3_LS_BUS_CAN_OP_STATUS,
                        length: op_size as binding::fmi3UInt32,
                    },
                    status: kind as ls_bus::fmi3LsBusCanStatusKind,
                };

                let op_bytes =
                    unsafe { std::slice::from_raw_parts(&op as *const _ as *const u8, op_size) };

                buffer[0..op_size].copy_from_slice(op_bytes);

                Ok(op_size)
            }
            LsBusCanOp::Wakeup => {
                let op_size = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationWakeup>();

                check_buffer_capacity(buffer, op_size)?;

                let op = ls_bus::fmi3LsBusCanOperationWakeup {
                    header: ls_bus::fmi3LsBusOperationHeader {
                        opCode: ls_bus::FMI3_LS_BUS_CAN_OP_WAKEUP,
                        length: op_size as binding::fmi3UInt32,
                    },
                };

                let op_bytes =
                    unsafe { std::slice::from_raw_parts(&op as *const _ as *const u8, op_size) };

                buffer[0..op_size].copy_from_slice(op_bytes);

                Ok(op_size)
            }
        }
    }

    fn read_next_operation(
        buffer: &'a [u8],
        read_pos: &mut usize,
    ) -> Result<Option<LsBusCanOp<'a>>, FmiLsBusError> {
        // peek the next operation, and return Ok(None) if there is None
        let (op, size) = match peek_next_operation_helper(buffer, *read_pos) {
            Some(v) => v,
            None => return Ok(None),
        };

        match op {
            ls_bus::FMI3_LS_BUS_CAN_OP_CAN_TRANSMIT
                if size >= std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanTransmit>() =>
            {
                let op_bytes = &buffer[*read_pos..*read_pos + size];
                let operation: &ls_bus::fmi3LsBusCanOperationCanTransmit = unsafe {
                    &*(op_bytes.as_ptr() as *const ls_bus::fmi3LsBusCanOperationCanTransmit)
                };
                let data_start = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanTransmit>();
                let data_end = data_start + operation.dataLength as usize;
                if data_end <= op_bytes.len() {
                    let data = &op_bytes[data_start..data_end];
                    *read_pos += size;
                    Ok(Some(LsBusCanOp::Transmit {
                        id: operation.id,
                        ide: operation.ide,
                        rtr: operation.rtr,
                        data: Cow::Borrowed(data),
                    }))
                } else {
                    Err(FmiLsBusError::BufferOverflow)
                }
            }

            ls_bus::FMI3_LS_BUS_CAN_OP_CANFD_TRANSMIT
                if size >= std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanFdTransmit>() =>
            {
                let op_bytes = &buffer[*read_pos..*read_pos + size];
                let operation: &ls_bus::fmi3LsBusCanOperationCanFdTransmit = unsafe {
                    &*(op_bytes.as_ptr() as *const ls_bus::fmi3LsBusCanOperationCanFdTransmit)
                };
                let data_start = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanFdTransmit>();
                let data_end = data_start + operation.dataLength as usize;
                if data_end <= op_bytes.len() {
                    let data = &op_bytes[data_start..data_end];
                    *read_pos += size;
                    Ok(Some(LsBusCanOp::FdTransmit {
                        id: operation.id,
                        ide: operation.ide,
                        brs: operation.brs,
                        esi: operation.esi,
                        data: Cow::Borrowed(data),
                    }))
                } else {
                    Err(FmiLsBusError::BufferOverflow)
                }
            }

            ls_bus::FMI3_LS_BUS_CAN_OP_CANXL_TRANSMIT
                if size >= std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanXlTransmit>() =>
            {
                let op_bytes = &buffer[*read_pos..*read_pos + size];
                let operation: &ls_bus::fmi3LsBusCanOperationCanXlTransmit = unsafe {
                    &*(op_bytes.as_ptr() as *const ls_bus::fmi3LsBusCanOperationCanXlTransmit)
                };
                let data_start = std::mem::size_of::<ls_bus::fmi3LsBusCanOperationCanXlTransmit>();
                let data_end = data_start + operation.dataLength as usize;
                if data_end <= op_bytes.len() {
                    let data = &op_bytes[data_start..data_end];
                    *read_pos += size;
                    Ok(Some(LsBusCanOp::XlTransmit {
                        id: operation.id,
                        ide: operation.ide,
                        sec: operation.sec,
                        sdt: operation.sdt,
                        vcid: operation.vcid,
                        af: operation.af,
                        data: Cow::Borrowed(data),
                    }))
                } else {
                    Err(FmiLsBusError::BufferOverflow)
                }
            }
            ls_bus::FMI3_LS_BUS_CAN_OP_CONFIRM
                if size >= std::mem::size_of::<ls_bus::fmi3LsBusCanOperationConfirm>() =>
            {
                let op_bytes = &buffer[*read_pos..*read_pos + size];
                let operation: &ls_bus::fmi3LsBusCanOperationConfirm =
                    unsafe { &*(op_bytes.as_ptr() as *const ls_bus::fmi3LsBusCanOperationConfirm) };
                *read_pos += size;
                Ok(Some(LsBusCanOp::Confirm(operation.id)))
            }

            ls_bus::FMI3_LS_BUS_CAN_OP_ARBITRATION_LOST
                if size >= std::mem::size_of::<ls_bus::fmi3LsBusCanOperationArbitrationLost>() =>
            {
                let op_bytes = &buffer[*read_pos..*read_pos + size];
                let operation: &ls_bus::fmi3LsBusCanOperationArbitrationLost = unsafe {
                    &*(op_bytes.as_ptr() as *const ls_bus::fmi3LsBusCanOperationArbitrationLost)
                };
                *read_pos += size;
                Ok(Some(LsBusCanOp::ArbitrationLost { id: operation.id }))
            }
            ls_bus::FMI3_LS_BUS_CAN_OP_BUS_ERROR
                if size >= std::mem::size_of::<ls_bus::fmi3LsBusCanOperationBusError>() =>
            {
                let op_bytes = &buffer[*read_pos..*read_pos + size];
                let operation: &ls_bus::fmi3LsBusCanOperationBusError = unsafe {
                    &*(op_bytes.as_ptr() as *const ls_bus::fmi3LsBusCanOperationBusError)
                };
                *read_pos += size;
                Ok(Some(LsBusCanOp::BusError {
                    id: operation.id,
                    error_code: operation.errorCode.try_into()?,
                    error_flags: operation.errorFlag.try_into()?,
                    is_sender: operation.isSender != 0,
                }))
            }
            ls_bus::FMI3_LS_BUS_CAN_OP_STATUS
                if size >= std::mem::size_of::<ls_bus::fmi3LsBusCanOperationStatus>() =>
            {
                let op_bytes = &buffer[*read_pos..*read_pos + size];
                let operation: &ls_bus::fmi3LsBusCanOperationStatus =
                    unsafe { &*(op_bytes.as_ptr() as *const ls_bus::fmi3LsBusCanOperationStatus) };
                *read_pos += size;
                Ok(Some(LsBusCanOp::Status(operation.status.try_into()?)))
            }
            ls_bus::FMI3_LS_BUS_CAN_OP_WAKEUP
                if size >= std::mem::size_of::<ls_bus::fmi3LsBusCanOperationWakeup>() =>
            {
                *read_pos += size;
                Ok(Some(LsBusCanOp::Wakeup))
            }

            ls_bus::FMI3_LS_BUS_CAN_OP_CONFIGURATION
                if size >= std::mem::size_of::<ls_bus::fmi3LsBusCanOperationConfiguration>() =>
            {
                let op_bytes = &buffer[*read_pos..*read_pos + size];
                let operation: &ls_bus::fmi3LsBusCanOperationConfiguration = unsafe {
                    &*(op_bytes.as_ptr() as *const ls_bus::fmi3LsBusCanOperationConfiguration)
                };
                *read_pos += size;

                match operation.parameterType {
                    ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_CAN_BAUDRATE => {
                        Ok(Some(LsBusCanOp::ConfigBaudrate(unsafe {
                            operation.__bindgen_anon_1.baudrate
                        })))
                    }
                    ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_ARBITRATION_LOST_BEHAVIOR => {
                        let behavior =
                            unsafe { operation.__bindgen_anon_1.arbitrationLostBehavior };
                        match behavior {
                            ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_ARBITRATION_LOST_BEHAVIOR_BUFFER_AND_RETRANSMIT => {
                                Ok(Some(LsBusCanOp::ConfigArbitrationLost(crate::can::LsBusCanArbitrationLostBehavior::BufferAndRetransmit)))
                            }
                            ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_ARBITRATION_LOST_BEHAVIOR_DISCARD_AND_NOTIFY => {
                                Ok(Some(LsBusCanOp::ConfigArbitrationLost(crate::can::LsBusCanArbitrationLostBehavior::DiscardAndNotify)))
                            }
                            _ => Err(FmiLsBusError::InvalidVariant(behavior as u32))
                        }
                    }
                    _ => Err(FmiLsBusError::InvalidOperation(
                        operation.parameterType as u32,
                    )),
                }
            }

            _ => {
                // Unknown operation or size too small
                Err(FmiLsBusError::InvalidOperation(op))
            }
        }
    }
}
