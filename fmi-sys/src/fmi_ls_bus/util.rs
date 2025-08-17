use crate::fmi_ls_bus::*;
use std::ptr;

/// Creates a Format Error operation and writes it to the buffer.
///
/// Returns `Ok(())` if successful, `Err(())` if there's insufficient buffer space.
pub fn create_op_format_error(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    data: &[fmi3LsBusData],
) -> Result<(), ()> {
    let data_length = data.len() as fmi3LsBusDataLength;
    let total_length = std::mem::size_of::<fmi3LsBusOperationHeader>()
        + std::mem::size_of::<fmi3LsBusDataLength>()
        + data.len();

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if total_length <= available_space {
        unsafe {
            // Write header
            let header = fmi3LsBusOperationHeader {
                opCode: FMI3_LS_BUS_OP_FORMAT_ERROR,
                length: total_length as fmi3LsBusOperationLength,
            };

            ptr::copy_nonoverlapping(
                &header as *const _ as *const u8,
                buffer_info.writePos,
                std::mem::size_of::<fmi3LsBusOperationHeader>(),
            );
            buffer_info.writePos = buffer_info
                .writePos
                .add(std::mem::size_of::<fmi3LsBusOperationHeader>());

            // Write data length
            ptr::copy_nonoverlapping(
                &data_length as *const _ as *const u8,
                buffer_info.writePos,
                std::mem::size_of::<fmi3LsBusDataLength>(),
            );
            buffer_info.writePos = buffer_info
                .writePos
                .add(std::mem::size_of::<fmi3LsBusDataLength>());

            // Write data
            ptr::copy_nonoverlapping(data.as_ptr(), buffer_info.writePos, data.len());
            buffer_info.writePos = buffer_info.writePos.add(data.len());
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Initializes a buffer info structure.
pub fn buffer_info_init(buffer_info: &mut fmi3LsBusUtilBufferInfo, buffer: &mut [fmi3UInt8]) {
    buffer_info.start = buffer.as_mut_ptr();
    buffer_info.size = buffer.len();
    buffer_info.end = unsafe { buffer.as_mut_ptr().add(buffer.len()) };
    buffer_info.writePos = buffer_info.start;
    buffer_info.readPos = buffer_info.start;
    buffer_info.status = true;
}

/// Resets a buffer info structure.
pub fn buffer_info_reset(buffer_info: &mut fmi3LsBusUtilBufferInfo) {
    buffer_info.writePos = buffer_info.start;
    buffer_info.readPos = buffer_info.start;
    buffer_info.status = true;
}

/// Checks whether the buffer is empty.
pub fn buffer_is_empty(buffer_info: &fmi3LsBusUtilBufferInfo) -> bool {
    buffer_info.writePos == buffer_info.start
}

/// Returns the start address of the buffer.
pub fn buffer_start(buffer_info: &fmi3LsBusUtilBufferInfo) -> *mut fmi3UInt8 {
    buffer_info.start
}

/// Returns the actual length of the buffer from the start address.
pub fn buffer_length(buffer_info: &fmi3LsBusUtilBufferInfo) -> usize {
    unsafe { buffer_info.writePos.offset_from(buffer_info.start) as usize }
}

/// Writes data to a buffer, overwriting existing data.
///
/// Returns `Ok(())` if successful, `Err(())` if there's insufficient buffer space.
pub fn buffer_write(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    data: &[fmi3UInt8],
) -> Result<(), ()> {
    if data.len() <= buffer_info.size {
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), buffer_info.start, data.len());
            buffer_info.writePos = buffer_info.start.add(data.len());
            buffer_info.readPos = buffer_info.start;
        }
        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Iterator for reading bus operations from a buffer.
pub struct OperationIterator<'a> {
    buffer_info: &'a mut fmi3LsBusUtilBufferInfo,
}

impl<'a> OperationIterator<'a> {
    pub fn new(buffer_info: &'a mut fmi3LsBusUtilBufferInfo) -> Self {
        Self { buffer_info }
    }
}

impl<'a> Iterator for OperationIterator<'a> {
    type Item = *const fmi3LsBusOperationHeader;

    fn next(&mut self) -> Option<Self::Item> {
        let available_bytes = unsafe {
            self.buffer_info
                .writePos
                .offset_from(self.buffer_info.readPos) as usize
        };

        if available_bytes >= std::mem::size_of::<fmi3LsBusOperationHeader>() {
            let header = unsafe { &*(self.buffer_info.readPos as *const fmi3LsBusOperationHeader) };

            if available_bytes >= header.length as usize {
                let operation = self.buffer_info.readPos as *const fmi3LsBusOperationHeader;
                unsafe {
                    self.buffer_info.readPos = self.buffer_info.readPos.add(header.length as usize);
                }
                Some(operation)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Reads the next bus operation from a buffer.
///
/// Returns `Some(operation_ptr)` if successful, `None` if no more operations.
pub fn read_next_operation(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
) -> Option<*const fmi3LsBusOperationHeader> {
    let available_bytes = unsafe { buffer_info.writePos.offset_from(buffer_info.readPos) as usize };

    if available_bytes >= std::mem::size_of::<fmi3LsBusOperationHeader>() {
        let header = unsafe { &*(buffer_info.readPos as *const fmi3LsBusOperationHeader) };

        if available_bytes >= header.length as usize {
            let operation = buffer_info.readPos as *const fmi3LsBusOperationHeader;
            unsafe {
                buffer_info.readPos = buffer_info.readPos.add(header.length as usize);
            }
            Some(operation)
        } else {
            None
        }
    } else {
        None
    }
}

/// Reads the next bus operation directly from a raw buffer.
///
/// Returns `Some(operation_ptr)` if successful, `None` if no more operations.
pub fn read_next_operation_direct(
    buffer: &[fmi3UInt8],
    read_pos: &mut usize,
) -> Option<*const fmi3LsBusOperationHeader> {
    let available_bytes = buffer.len() - *read_pos;

    if available_bytes >= std::mem::size_of::<fmi3LsBusOperationHeader>() {
        let header_ptr =
            unsafe { buffer.as_ptr().add(*read_pos) as *const fmi3LsBusOperationHeader };
        let header = unsafe { &*header_ptr };

        if available_bytes >= header.length as usize {
            *read_pos += header.length as usize;
            Some(header_ptr)
        } else {
            None
        }
    } else {
        None
    }
}

/// Submits a bus operation to the specified buffer.
///
/// Returns `Ok(())` if successful, `Err(())` if there's insufficient buffer space.
pub fn submit_operation(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    operation: &[fmi3UInt8],
    additional_data: Option<&[fmi3UInt8]>,
) -> Result<(), ()> {
    let additional_data_len = additional_data.map_or(0, |d| d.len());
    let total_length = operation.len() + additional_data_len;

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) as usize };

    if total_length <= available_space {
        unsafe {
            // Copy operation structure
            ptr::copy_nonoverlapping(operation.as_ptr(), buffer_info.writePos, operation.len());
            buffer_info.writePos = buffer_info.writePos.add(operation.len());

            // Copy additional data if present
            if let Some(data) = additional_data {
                ptr::copy_nonoverlapping(data.as_ptr(), buffer_info.writePos, data.len());
                buffer_info.writePos = buffer_info.writePos.add(data.len());
            }
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN transmit operation and writes it to the buffer.
pub fn create_can_transmit(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    id: fmi3LsBusCanId,
    ide: fmi3LsBusCanIde,
    rtr: fmi3LsBusCanRtr,
    data: &[fmi3LsBusCanData],
) -> Result<(), ()> {
    let data_length = data.len() as fmi3LsBusCanDataLength;
    let total_length = std::mem::size_of::<fmi3LsBusOperationHeader>()
        + std::mem::size_of::<fmi3LsBusCanId>()
        + std::mem::size_of::<fmi3LsBusCanIde>()
        + std::mem::size_of::<fmi3LsBusCanRtr>()
        + std::mem::size_of::<fmi3LsBusCanDataLength>()
        + data.len();

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if total_length <= available_space {
        unsafe {
            // Create and write the operation structure (without variable data)
            let op = fmi3LsBusCanOperationCanTransmit {
                header: fmi3LsBusOperationHeader {
                    opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_CAN_TRANSMIT,
                    length: total_length as fmi3LsBusOperationLength,
                },
                id,
                ide,
                rtr,
                dataLength: data_length,
                data: std::mem::zeroed(), // Flexible array member
            };

            let op_size = total_length - data.len();
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op_size,
            );
            buffer_info.writePos = buffer_info.writePos.add(op_size);

            // Write data
            ptr::copy_nonoverlapping(data.as_ptr(), buffer_info.writePos, data.len());
            buffer_info.writePos = buffer_info.writePos.add(data.len());
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN FD transmit operation and writes it to the buffer.
pub fn create_can_fd_transmit(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    id: fmi3LsBusCanId,
    ide: fmi3LsBusCanIde,
    brs: fmi3LsBusCanBrs,
    esi: fmi3LsBusCanEsi,
    data: &[fmi3LsBusCanData],
) -> Result<(), ()> {
    let data_length = data.len() as fmi3LsBusCanDataLength;
    let total_length = std::mem::size_of::<fmi3LsBusOperationHeader>()
        + std::mem::size_of::<fmi3LsBusCanId>()
        + std::mem::size_of::<fmi3LsBusCanIde>()
        + std::mem::size_of::<fmi3LsBusCanBrs>()
        + std::mem::size_of::<fmi3LsBusCanEsi>()
        + std::mem::size_of::<fmi3LsBusCanDataLength>()
        + data.len();

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if total_length <= available_space {
        unsafe {
            let op = fmi3LsBusCanOperationCanFdTransmit {
                header: fmi3LsBusOperationHeader {
                    opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_CANFD_TRANSMIT,
                    length: total_length as fmi3LsBusOperationLength,
                },
                id,
                ide,
                brs,
                esi,
                dataLength: data_length,
                data: std::mem::zeroed(),
            };

            let op_size = total_length - data.len();
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op_size,
            );
            buffer_info.writePos = buffer_info.writePos.add(op_size);

            ptr::copy_nonoverlapping(data.as_ptr(), buffer_info.writePos, data.len());
            buffer_info.writePos = buffer_info.writePos.add(data.len());
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN XL transmit operation and writes it to the buffer.
pub fn create_can_xl_transmit(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    id: fmi3LsBusCanId,
    ide: fmi3LsBusCanIde,
    sec: fmi3LsBusCanSec,
    sdt: fmi3LsBusCanSdt,
    vcid: fmi3LsBusCanVcId,
    af: fmi3LsBusCanAf,
    data: &[fmi3LsBusCanData],
) -> Result<(), ()> {
    let data_length = data.len() as fmi3LsBusCanDataLength;
    let total_length = std::mem::size_of::<fmi3LsBusOperationHeader>()
        + std::mem::size_of::<fmi3LsBusCanId>()
        + std::mem::size_of::<fmi3LsBusCanIde>()
        + std::mem::size_of::<fmi3LsBusCanSec>()
        + std::mem::size_of::<fmi3LsBusCanSdt>()
        + std::mem::size_of::<fmi3LsBusCanVcId>()
        + std::mem::size_of::<fmi3LsBusCanAf>()
        + std::mem::size_of::<fmi3LsBusCanDataLength>()
        + data.len();

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if total_length <= available_space {
        unsafe {
            let op = fmi3LsBusCanOperationCanXlTransmit {
                header: fmi3LsBusOperationHeader {
                    opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_CANXL_TRANSMIT,
                    length: total_length as fmi3LsBusOperationLength,
                },
                id,
                ide,
                sec,
                sdt,
                vcid,
                af,
                dataLength: data_length,
                data: std::mem::zeroed(),
            };

            let op_size = total_length - data.len();
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op_size,
            );
            buffer_info.writePos = buffer_info.writePos.add(op_size);

            ptr::copy_nonoverlapping(data.as_ptr(), buffer_info.writePos, data.len());
            buffer_info.writePos = buffer_info.writePos.add(data.len());
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN confirm operation and writes it to the buffer.
pub fn create_can_confirm(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    id: fmi3LsBusCanId,
) -> Result<(), ()> {
    let op = fmi3LsBusCanOperationConfirm {
        header: fmi3LsBusOperationHeader {
            opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_CONFIRM,
            length: std::mem::size_of::<fmi3LsBusCanOperationConfirm>() as fmi3LsBusOperationLength,
        },
        id,
    };

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if op.header.length as usize <= available_space {
        unsafe {
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op.header.length as usize,
            );
            buffer_info.writePos = buffer_info.writePos.add(op.header.length as usize);
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN arbitration lost operation and writes it to the buffer.
pub fn create_can_arbitration_lost(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    id: fmi3LsBusCanId,
) -> Result<(), ()> {
    let op = fmi3LsBusCanOperationArbitrationLost {
        header: fmi3LsBusOperationHeader {
            opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_ARBITRATION_LOST,
            length: std::mem::size_of::<fmi3LsBusCanOperationArbitrationLost>() as fmi3LsBusOperationLength,
        },
        id,
    };

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if op.header.length as usize <= available_space {
        unsafe {
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op.header.length as usize,
            );
            buffer_info.writePos = buffer_info.writePos.add(op.header.length as usize);
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN bus error operation and writes it to the buffer.
pub fn create_can_bus_error(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    id: fmi3LsBusCanId,
    error_code: fmi3LsBusCanErrorCode,
    error_flag: fmi3LsBusCanErrorFlag,
    is_sender: fmi3LsBusCanIsSender,
) -> Result<(), ()> {
    let op = fmi3LsBusCanOperationBusError {
        header: fmi3LsBusOperationHeader {
            opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_BUS_ERROR,
            length: std::mem::size_of::<fmi3LsBusCanOperationBusError>() as fmi3LsBusOperationLength,
        },
        id,
        errorCode: error_code,
        errorFlag: error_flag,
        isSender: is_sender,
    };

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if op.header.length as usize <= available_space {
        unsafe {
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op.header.length as usize,
            );
            buffer_info.writePos = buffer_info.writePos.add(op.header.length as usize);
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN status operation and writes it to the buffer.
pub fn create_can_status(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    status: fmi3LsBusCanStatusKind,
) -> Result<(), ()> {
    let op = fmi3LsBusCanOperationStatus {
        header: fmi3LsBusOperationHeader {
            opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_STATUS,
            length: std::mem::size_of::<fmi3LsBusCanOperationStatus>() as fmi3LsBusOperationLength,
        },
        status,
    };

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if op.header.length as usize <= available_space {
        unsafe {
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op.header.length as usize,
            );
            buffer_info.writePos = buffer_info.writePos.add(op.header.length as usize);
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN wakeup operation and writes it to the buffer.
pub fn create_can_wakeup(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
) -> Result<(), ()> {
    let op = fmi3LsBusCanOperationWakeup {
        header: fmi3LsBusOperationHeader {
            opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_WAKEUP,
            length: std::mem::size_of::<fmi3LsBusCanOperationWakeup>() as fmi3LsBusOperationLength,
        },
    };

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if op.header.length as usize <= available_space {
        unsafe {
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op.header.length as usize,
            );
            buffer_info.writePos = buffer_info.writePos.add(op.header.length as usize);
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN configuration operation for CAN baudrate.
pub fn create_can_config_baudrate(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    baudrate: fmi3LsBusCanBaudrate,
) -> Result<(), ()> {
    let op = fmi3LsBusCanOperationConfiguration {
        header: fmi3LsBusOperationHeader {
            opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_CONFIGURATION,
            length: (std::mem::size_of::<fmi3LsBusOperationHeader>()
                + std::mem::size_of::<fmi3LsBusCanConfigParameterType>()
                + std::mem::size_of::<fmi3LsBusCanBaudrate>()) as fmi3LsBusOperationLength,
        },
        parameterType: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_CAN_BAUDRATE,
        __bindgen_anon_1: fmi3LsBusCanOperationConfiguration__bindgen_ty_1 { baudrate },
    };

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if op.header.length as usize <= available_space {
        unsafe {
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op.header.length as usize,
            );
            buffer_info.writePos = buffer_info.writePos.add(op.header.length as usize);
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN configuration operation for CAN FD baudrate.
pub fn create_can_config_fd_baudrate(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    baudrate: fmi3LsBusCanBaudrate,
) -> Result<(), ()> {
    let op = fmi3LsBusCanOperationConfiguration {
        header: fmi3LsBusOperationHeader {
            opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_CONFIGURATION,
            length: (std::mem::size_of::<fmi3LsBusOperationHeader>()
                + std::mem::size_of::<fmi3LsBusCanConfigParameterType>()
                + std::mem::size_of::<fmi3LsBusCanBaudrate>()) as fmi3LsBusOperationLength,
        },
        parameterType: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_CANFD_BAUDRATE,
        __bindgen_anon_1: fmi3LsBusCanOperationConfiguration__bindgen_ty_1 { baudrate },
    };

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if op.header.length as usize <= available_space {
        unsafe {
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op.header.length as usize,
            );
            buffer_info.writePos = buffer_info.writePos.add(op.header.length as usize);
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN configuration operation for CAN XL baudrate.
pub fn create_can_config_xl_baudrate(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    baudrate: fmi3LsBusCanBaudrate,
) -> Result<(), ()> {
    let op = fmi3LsBusCanOperationConfiguration {
        header: fmi3LsBusOperationHeader {
            opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_CONFIGURATION,
            length: (std::mem::size_of::<fmi3LsBusOperationHeader>()
                + std::mem::size_of::<fmi3LsBusCanConfigParameterType>()
                + std::mem::size_of::<fmi3LsBusCanBaudrate>()) as fmi3LsBusOperationLength,
        },
        parameterType: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_CANXL_BAUDRATE,
        __bindgen_anon_1: fmi3LsBusCanOperationConfiguration__bindgen_ty_1 { baudrate },
    };

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if op.header.length as usize <= available_space {
        unsafe {
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op.header.length as usize,
            );
            buffer_info.writePos = buffer_info.writePos.add(op.header.length as usize);
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Creates a CAN configuration operation for arbitration lost behavior.
pub fn create_can_config_arbitration_lost_behavior(
    buffer_info: &mut fmi3LsBusUtilBufferInfo,
    behavior: fmi3LsBusCanArbitrationLostBehavior,
) -> Result<(), ()> {
    let op = fmi3LsBusCanOperationConfiguration {
        header: fmi3LsBusOperationHeader {
            opCode: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_OP_CONFIGURATION,
            length: (std::mem::size_of::<fmi3LsBusOperationHeader>()
                + std::mem::size_of::<fmi3LsBusCanConfigParameterType>()
                + std::mem::size_of::<fmi3LsBusCanArbitrationLostBehavior>()) as fmi3LsBusOperationLength,
        },
        parameterType: crate::fmi_ls_bus::FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_ARBITRATION_LOST_BEHAVIOR,
        __bindgen_anon_1: fmi3LsBusCanOperationConfiguration__bindgen_ty_1 { 
            arbitrationLostBehavior: behavior 
        },
    };

    let available_space = unsafe { buffer_info.end.offset_from(buffer_info.writePos) } as usize;

    if op.header.length as usize <= available_space {
        unsafe {
            ptr::copy_nonoverlapping(
                &op as *const _ as *const u8,
                buffer_info.writePos,
                op.header.length as usize,
            );
            buffer_info.writePos = buffer_info.writePos.add(op.header.length as usize);
        }

        buffer_info.status = true;
        Ok(())
    } else {
        buffer_info.status = false;
        Err(())
    }
}

/// Helper macro for creating operations with proper length calculation.
#[macro_export]
macro_rules! create_operation {
    ($op_type:ty, $op_code:expr, $init_fn:expr) => {{
        let mut operation: $op_type = unsafe { std::mem::zeroed() };
        operation.header.opCode = $op_code;
        operation.header.length = std::mem::size_of::<$op_type>() as fmi3LsBusOperationLength;
        $init_fn(&mut operation);
        operation
    }};
}
