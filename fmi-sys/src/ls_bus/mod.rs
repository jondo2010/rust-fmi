#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]
include!(concat!(env!("OUT_DIR"), "/ls_bus_bindings.rs"));

pub const FMI3_LS_BUS_OP_FORMAT_ERROR: fmi3LsBusOperationCode = 0x0001;

// CAN bus-specific operation codes
pub const FMI3_LS_BUS_CAN_OP_CAN_TRANSMIT: fmi3LsBusOperationCode = 0x0010;
pub const FMI3_LS_BUS_CAN_OP_CANFD_TRANSMIT: fmi3LsBusOperationCode = 0x0011;
pub const FMI3_LS_BUS_CAN_OP_CANXL_TRANSMIT: fmi3LsBusOperationCode = 0x0012;
pub const FMI3_LS_BUS_CAN_OP_CONFIRM: fmi3LsBusOperationCode = 0x0020;
pub const FMI3_LS_BUS_CAN_OP_ARBITRATION_LOST: fmi3LsBusOperationCode = 0x0030;
pub const FMI3_LS_BUS_CAN_OP_BUS_ERROR: fmi3LsBusOperationCode = 0x0031;
pub const FMI3_LS_BUS_CAN_OP_CONFIGURATION: fmi3LsBusOperationCode = 0x0040;
pub const FMI3_LS_BUS_CAN_OP_STATUS: fmi3LsBusOperationCode = 0x0041;
pub const FMI3_LS_BUS_CAN_OP_WAKEUP: fmi3LsBusOperationCode = 0x0042;

// CAN bus error codes
pub const FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_BIT_ERROR: fmi3LsBusCanErrorCode = 0x1;
pub const FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_BIT_STUFFING_ERROR: fmi3LsBusCanErrorCode = 0x2;
pub const FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_FORM_ERROR: fmi3LsBusCanErrorCode = 0x3;
pub const FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_CRC_ERROR: fmi3LsBusCanErrorCode = 0x4;
pub const FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_ACK_ERROR: fmi3LsBusCanErrorCode = 0x5;
pub const FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_CODE_BROKEN_ERROR_FRAME: fmi3LsBusCanErrorCode = 0x6;

// CAN bus error flags
pub const FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_FLAG_PRIMARY_ERROR_FLAG: fmi3LsBusCanErrorFlag = 0x1;
pub const FMI3_LS_BUS_CAN_BUSERROR_PARAM_ERROR_FLAG_SECONDARY_ERROR_FLAG: fmi3LsBusCanErrorFlag =
    0x2;

// CAN status kinds
pub const FMI3_LS_BUS_CAN_STATUS_PARAM_STATUS_KIND_ERROR_ACTIVE: fmi3LsBusCanStatusKind = 0x1;
pub const FMI3_LS_BUS_CAN_STATUS_PARAM_STATUS_KIND_ERROR_PASSIVE: fmi3LsBusCanStatusKind = 0x2;
pub const FMI3_LS_BUS_CAN_STATUS_PARAM_STATUS_KIND_BUS_OFF: fmi3LsBusCanStatusKind = 0x3;

// CAN configuration parameter types
pub const FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_CAN_BAUDRATE: fmi3LsBusCanConfigParameterType = 0x1;
pub const FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_CANFD_BAUDRATE: fmi3LsBusCanConfigParameterType = 0x2;
pub const FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_CANXL_BAUDRATE: fmi3LsBusCanConfigParameterType = 0x3;
pub const FMI3_LS_BUS_CAN_CONFIG_PARAM_TYPE_ARBITRATION_LOST_BEHAVIOR:
    fmi3LsBusCanConfigParameterType = 0x4;

// CAN arbitration lost behavior
pub const FMI3_LS_BUS_CAN_CONFIG_PARAM_ARBITRATION_LOST_BEHAVIOR_BUFFER_AND_RETRANSMIT:
    fmi3LsBusCanArbitrationLostBehavior = 0x1;
pub const FMI3_LS_BUS_CAN_CONFIG_PARAM_ARBITRATION_LOST_BEHAVIOR_DISCARD_AND_NOTIFY:
    fmi3LsBusCanArbitrationLostBehavior = 0x2;
