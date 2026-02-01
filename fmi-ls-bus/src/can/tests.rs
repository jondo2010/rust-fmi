use crate::{
    FmiLsBus,
    can::{
        LsBusCanArbitrationLostBehavior, LsBusCanErrorCode, LsBusCanErrorFlag, LsBusCanOp,
        LsBusCanStatusKind,
    },
};

use std::borrow::Cow;

fn cow_bytes<'a>(data: &'a Cow<'a, [u8]>) -> &'a [u8] {
    AsRef::<[u8]>::as_ref(data)
}

#[test]
fn test_can_transmit_operation() {
    let mut buffer = vec![0u8; 2048];
    let mut bus = FmiLsBus::new();
    let test_data = b"test_data";

    // Test creating CAN transmit operation
    bus.write_operation(
        LsBusCanOp::Transmit {
            id: 0x123,
            ide: 1,
            rtr: 0,
            data: Cow::Borrowed(test_data),
        },
        &mut buffer,
    )
    .unwrap();

    // Read back the operation
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::Transmit { id, ide, rtr, data }) => {
            assert_eq!(id, 0x123);
            assert_eq!(ide, 1);
            assert_eq!(rtr, 0);
            assert_eq!(cow_bytes(&data), test_data);
        }
        _ => panic!("Expected Transmit operation"),
    }
}

#[test]
fn test_can_fd_transmit_operation() {
    let mut buffer = vec![0u8; 2048];
    let mut bus = FmiLsBus::new();
    let test_data = b"canfd_test";

    // Create CAN FD transmit operation
    bus.write_operation(
        LsBusCanOp::FdTransmit {
            id: 0x456,
            ide: 0,
            brs: 1,
            esi: 0,
            data: Cow::Borrowed(test_data),
        },
        &mut buffer,
    )
    .unwrap();

    // Read back the operation
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::FdTransmit {
            id,
            ide,
            brs,
            esi,
            data,
        }) => {
            assert_eq!(id, 0x456);
            assert_eq!(ide, 0);
            assert_eq!(brs, 1);
            assert_eq!(esi, 0);
            assert_eq!(cow_bytes(&data), test_data);
        }
        _ => panic!("Expected FdTransmit operation"),
    }
}

#[test]
fn test_can_xl_transmit_operation() {
    let mut buffer = vec![0u8; 2048];
    let mut bus = FmiLsBus::new();
    let test_data = b"canxl_test";

    // Create CAN XL transmit operation
    bus.write_operation(
        LsBusCanOp::XlTransmit {
            id: 0x789,
            ide: 1,
            sec: 1,
            sdt: 0,
            vcid: 2,
            af: 0xABC,
            data: Cow::Borrowed(test_data),
        },
        &mut buffer,
    )
    .unwrap();

    // Read back the operation
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::XlTransmit {
            id,
            ide,
            sec,
            sdt,
            vcid,
            af,
            data,
        }) => {
            assert_eq!(id, 0x789);
            assert_eq!(ide, 1);
            assert_eq!(sec, 1);
            assert_eq!(sdt, 0);
            assert_eq!(vcid, 2);
            assert_eq!(af, 0xABC);
            assert_eq!(cow_bytes(&data), test_data);
        }
        _ => panic!("Expected XlTransmit operation"),
    }
}

#[test]
fn test_can_confirm_operation() {
    let mut buffer = vec![0u8; 1024];
    let mut bus = FmiLsBus::new();

    // Create confirm operation
    bus.write_operation(LsBusCanOp::Confirm(0x555), &mut buffer)
        .unwrap();

    // Read back the operation
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::Confirm(id)) => {
            assert_eq!(id, 0x555);
        }
        _ => panic!("Expected Confirm operation"),
    }
}

#[test]
fn test_can_baudrate_config() {
    let mut buffer = vec![0u8; 1024];
    let mut bus = FmiLsBus::new();

    // Create baudrate configuration
    bus.write_operation(LsBusCanOp::ConfigBaudrate(500000), &mut buffer)
        .unwrap();

    // Read back the operation
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::ConfigBaudrate(baudrate)) => {
            assert_eq!(baudrate, 500000);
        }
        _ => panic!("Expected ConfigBaudrate operation"),
    }
}

#[test]
fn test_can_config_arbitration_lost_operation() {
    let mut buffer = vec![0u8; 1024];
    let mut bus = FmiLsBus::new();

    // Test buffer and retransmit behavior
    bus.write_operation(
        LsBusCanOp::ConfigArbitrationLost(LsBusCanArbitrationLostBehavior::BufferAndRetransmit),
        &mut buffer,
    )
    .unwrap();

    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::ConfigArbitrationLost(behavior)) => {
            assert!(matches!(
                behavior,
                LsBusCanArbitrationLostBehavior::BufferAndRetransmit
            ));
        }
        _ => panic!("Expected ConfigArbitrationLost operation"),
    }

    bus.reset();
    buffer.clear();
    buffer.resize(1024, 0);

    // Test discard and notify behavior
    bus.write_operation(
        LsBusCanOp::ConfigArbitrationLost(LsBusCanArbitrationLostBehavior::DiscardAndNotify),
        &mut buffer,
    )
    .unwrap();

    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::ConfigArbitrationLost(behavior)) => {
            assert!(matches!(
                behavior,
                LsBusCanArbitrationLostBehavior::DiscardAndNotify
            ));
        }
        _ => panic!("Expected ConfigArbitrationLost operation"),
    }
}

#[test]
fn test_can_arbitration_lost_operation() {
    let mut buffer = vec![0u8; 1024];
    let mut bus = FmiLsBus::new();
    let test_id = 0x456;

    bus.write_operation(LsBusCanOp::ArbitrationLost { id: test_id }, &mut buffer)
        .unwrap();

    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::ArbitrationLost { id }) => {
            assert_eq!(id, test_id);
        }
        _ => panic!("Expected ArbitrationLost operation"),
    }
}

#[test]
fn test_can_bus_error_operation() {
    let mut buffer = vec![0u8; 1024];
    let mut bus = FmiLsBus::new();
    let test_id = 0x789;

    bus.write_operation(
        LsBusCanOp::BusError {
            id: test_id,
            error_code: LsBusCanErrorCode::BitError,
            error_flags: LsBusCanErrorFlag::Primary,
            is_sender: true,
        },
        &mut buffer,
    )
    .unwrap();

    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::BusError {
            id,
            error_code,
            error_flags,
            is_sender,
        }) => {
            assert_eq!(id, test_id);
            assert!(matches!(error_code, LsBusCanErrorCode::BitError));
            assert!(matches!(error_flags, LsBusCanErrorFlag::Primary));
            assert!(is_sender);
        }
        _ => panic!("Expected BusError operation"),
    }
}

#[test]
fn test_can_status_operation() {
    let mut buffer = vec![0u8; 1024];
    let mut bus = FmiLsBus::new();

    // Test ErrorActive status
    bus.write_operation(
        LsBusCanOp::Status(LsBusCanStatusKind::ErrorActive),
        &mut buffer,
    )
    .unwrap();
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::Status(LsBusCanStatusKind::ErrorActive)) => {}
        _ => panic!("Expected Status(ErrorActive) operation"),
    }

    bus.reset();
    buffer.clear();
    buffer.resize(1024, 0);

    // Test ErrorPassive status
    bus.write_operation(
        LsBusCanOp::Status(LsBusCanStatusKind::ErrorPassive),
        &mut buffer,
    )
    .unwrap();
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::Status(LsBusCanStatusKind::ErrorPassive)) => {}
        _ => panic!("Expected Status(ErrorPassive) operation"),
    }

    bus.reset();
    buffer.clear();
    buffer.resize(1024, 0);

    // Test BusOff status
    bus.write_operation(LsBusCanOp::Status(LsBusCanStatusKind::BusOff), &mut buffer)
        .unwrap();
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::Status(LsBusCanStatusKind::BusOff)) => {}
        _ => panic!("Expected Status(BusOff) operation"),
    }
}

#[test]
fn test_can_wakeup_operation() {
    let mut buffer = vec![0u8; 1024];
    let mut bus = FmiLsBus::new();

    bus.write_operation(LsBusCanOp::Wakeup, &mut buffer)
        .unwrap();

    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::Wakeup) => {
            // Success
        }
        _ => panic!("Expected Wakeup operation"),
    }
}

#[test]
fn test_multiple_operations_in_sequence() {
    let mut buffer = vec![0u8; 4096];
    let mut bus = FmiLsBus::new();
    let data1 = b"first";
    let data2 = b"second";

    // Create multiple operations
    bus.write_operation(
        LsBusCanOp::Transmit {
            id: 0x100,
            ide: 0,
            rtr: 0,
            data: Cow::Borrowed(data1),
        },
        &mut buffer,
    )
    .unwrap();
    bus.write_operation(LsBusCanOp::Confirm(0x100), &mut buffer)
        .unwrap();
    bus.write_operation(
        LsBusCanOp::Transmit {
            id: 0x200,
            ide: 1,
            rtr: 1,
            data: Cow::Borrowed(data2),
        },
        &mut buffer,
    )
    .unwrap();

    // Read back operations in order
    let op1: Option<LsBusCanOp> = bus.read_next_operation(&buffer[..bus.write_pos]).unwrap();
    match op1 {
        Some(LsBusCanOp::Transmit { id, data, .. }) => {
            assert_eq!(id, 0x100);
            assert_eq!(cow_bytes(&data), data1);
        }
        _ => panic!("Expected first Transmit operation"),
    }

    let op2: Option<LsBusCanOp> = bus.read_next_operation(&buffer[..bus.write_pos]).unwrap();
    match op2 {
        Some(LsBusCanOp::Confirm(id)) => {
            assert_eq!(id, 0x100);
        }
        _ => panic!("Expected Confirm operation"),
    }

    let op3: Option<LsBusCanOp> = bus.read_next_operation(&buffer[..bus.write_pos]).unwrap();
    match op3 {
        Some(LsBusCanOp::Transmit { id, data, .. }) => {
            assert_eq!(id, 0x200);
            assert_eq!(cow_bytes(&data), data2);
        }
        _ => panic!("Expected second Transmit operation"),
    }

    // No more operations
    let op4: Option<LsBusCanOp> = bus.read_next_operation(&buffer[..bus.write_pos]).unwrap();
    assert!(op4.is_none());
}

#[test]
fn test_buffer_serialization() {
    let mut buf1 = vec![0u8; 1024];
    let mut buf2 = vec![0u8; 1024];
    let mut bus1 = FmiLsBus::new();
    let mut bus2 = FmiLsBus::new();
    let test_data = b"serialize_test";

    // Create operation in first buffer
    bus1.write_operation(
        LsBusCanOp::Transmit {
            id: 0x999,
            ide: 1,
            rtr: 1,
            data: Cow::Borrowed(test_data),
        },
        &mut buf1,
    )
    .unwrap();

    // Serialize to second buffer
    bus2.write(&mut buf2, &buf1).unwrap();

    // Read from second buffer
    let operation: Option<LsBusCanOp> = bus2.read_next_operation(&buf2).unwrap();
    match operation {
        Some(LsBusCanOp::Transmit { id, ide, rtr, data }) => {
            assert_eq!(id, 0x999);
            assert_eq!(ide, 1);
            assert_eq!(rtr, 1);
            assert_eq!(cow_bytes(&data), test_data);
        }
        _ => panic!("Expected Transmit operation"),
    }
}

#[test]
fn test_data_borrowing_optimization() {
    let mut buffer = vec![0u8; 2048];
    let mut bus = FmiLsBus::new();
    let test_data = b"borrowed_data";

    // Create operation with borrowed data
    bus.write_operation(
        LsBusCanOp::Transmit {
            id: 42,
            ide: 0,
            rtr: 0,
            data: Cow::Borrowed(test_data),
        },
        &mut buffer,
    )
    .unwrap();

    // Read back the operation - data should be borrowed from buffer
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::Transmit { data, .. }) => {
            assert_eq!(cow_bytes(&data), test_data);
            assert!(matches!(data, Cow::Borrowed(_)));
        }
        _ => panic!("Expected Transmit operation"),
    }
}

#[test]
fn test_comprehensive_operation_sequence() {
    let mut buffer = vec![0u8; 8192];
    let mut bus = FmiLsBus::new();
    let test_data = b"comprehensive";

    // Create one of each operation type
    bus.write_operation(
        LsBusCanOp::Transmit {
            id: 0x100,
            ide: 0,
            rtr: 0,
            data: Cow::Borrowed(test_data),
        },
        &mut buffer,
    )
    .unwrap();

    bus.write_operation(
        LsBusCanOp::FdTransmit {
            id: 0x200,
            ide: 1,
            brs: 1,
            esi: 0,
            data: Cow::Borrowed(test_data),
        },
        &mut buffer,
    )
    .unwrap();

    bus.write_operation(
        LsBusCanOp::XlTransmit {
            id: 0x300,
            ide: 0,
            sec: 1,
            sdt: 0,
            vcid: 1,
            af: 0x123,
            data: Cow::Borrowed(test_data),
        },
        &mut buffer,
    )
    .unwrap();

    bus.write_operation(LsBusCanOp::Confirm(0x400), &mut buffer)
        .unwrap();
    bus.write_operation(LsBusCanOp::ConfigBaudrate(1000000), &mut buffer)
        .unwrap();
    bus.write_operation(
        LsBusCanOp::ConfigArbitrationLost(LsBusCanArbitrationLostBehavior::BufferAndRetransmit),
        &mut buffer,
    )
    .unwrap();
    bus.write_operation(LsBusCanOp::ArbitrationLost { id: 0x500 }, &mut buffer)
        .unwrap();
    bus.write_operation(
        LsBusCanOp::BusError {
            id: 0x600,
            error_code: LsBusCanErrorCode::CrcError,
            error_flags: LsBusCanErrorFlag::Secondary,
            is_sender: false,
        },
        &mut buffer,
    )
    .unwrap();
    bus.write_operation(
        LsBusCanOp::Status(LsBusCanStatusKind::ErrorActive),
        &mut buffer,
    )
    .unwrap();
    bus.write_operation(LsBusCanOp::Wakeup, &mut buffer)
        .unwrap();

    // Read all operations back and verify
    let operations = [
        "Transmit",
        "FdTransmit",
        "XlTransmit",
        "Confirm",
        "ConfigBaudrate",
        "ConfigArbitrationLost",
        "ArbitrationLost",
        "BusError",
        "Status",
        "Wakeup",
    ];

    for expected_op in operations {
        let operation: Option<LsBusCanOp> =
            bus.read_next_operation(&buffer[..bus.write_pos]).unwrap();
        match (&operation, expected_op) {
            (
                Some(LsBusCanOp::Transmit {
                    id: 0x100, data, ..
                }),
                "Transmit",
            ) => {
                assert_eq!(cow_bytes(&data), test_data);
            }
            (
                Some(LsBusCanOp::FdTransmit {
                    id: 0x200, data, ..
                }),
                "FdTransmit",
            ) => {
                assert_eq!(cow_bytes(&data), test_data);
            }
            (
                Some(LsBusCanOp::XlTransmit {
                    id: 0x300, data, ..
                }),
                "XlTransmit",
            ) => {
                assert_eq!(cow_bytes(&data), test_data);
            }
            (Some(LsBusCanOp::Confirm(0x400)), "Confirm") => {}
            (Some(LsBusCanOp::ConfigBaudrate(1000000)), "ConfigBaudrate") => {}
            (Some(LsBusCanOp::ConfigArbitrationLost(_)), "ConfigArbitrationLost") => {}
            (Some(LsBusCanOp::ArbitrationLost { id: 0x500 }), "ArbitrationLost") => {}
            (Some(LsBusCanOp::BusError { id: 0x600, .. }), "BusError") => {}
            (Some(LsBusCanOp::Status(_)), "Status") => {}
            (Some(LsBusCanOp::Wakeup), "Wakeup") => {}
            _ => panic!("Expected {} operation, got {:?}", expected_op, operation),
        }
    }

    // Verify no more operations
    assert!(
        bus.read_next_operation::<LsBusCanOp>(&buffer[..bus.write_pos])
            .unwrap()
            .is_none()
    );
}

#[test]
fn test_edge_cases() {
    let mut buffer = vec![0u8; 1024];
    let mut bus = FmiLsBus::new();

    // Test with empty data
    bus.write_operation(
        LsBusCanOp::Transmit {
            id: 0,
            ide: 0,
            rtr: 0,
            data: Cow::Borrowed(&[]),
        },
        &mut buffer,
    )
    .unwrap();
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::Transmit { data, .. }) => {
            assert!(data.is_empty());
        }
        _ => panic!("Expected Transmit operation"),
    }

    bus.reset();
    buffer.clear();
    buffer.resize(1024, 0);

    // Test with maximum values
    let max_data = vec![0xFF; 64]; // Reasonable max size for CAN data
    bus.write_operation(
        LsBusCanOp::Transmit {
            id: u32::MAX,
            ide: u8::MAX,
            rtr: u8::MAX,
            data: Cow::Borrowed(&max_data),
        },
        &mut buffer,
    )
    .unwrap();
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer).unwrap();
    match operation {
        Some(LsBusCanOp::Transmit { id, ide, rtr, data }) => {
            assert_eq!(id, u32::MAX);
            assert_eq!(ide, u8::MAX);
            assert_eq!(rtr, u8::MAX);
            assert_eq!(cow_bytes(&data), &max_data);
        }
        _ => panic!("Expected Transmit operation"),
    }
}

#[test]
fn test_buffer_overflow_error() {
    let mut buffer = vec![0u8; 16]; // Very small buffer
    let mut bus = FmiLsBus::new();
    let large_data = vec![0xFF; 64]; // Large data that won't fit

    // Attempt to write operation that's too large for buffer
    let result = bus.write_operation(
        LsBusCanOp::Transmit {
            id: 0x123,
            ide: 0,
            rtr: 0,
            data: Cow::Borrowed(&large_data),
        },
        &mut buffer,
    );

    // Should return BufferOverflow error
    assert!(matches!(result, Err(crate::FmiLsBusError::BufferOverflow)));

    // Buffer write position should remain unchanged
    assert_eq!(bus.write_pos, 0);
}

#[test]
fn test_simplified_transmit_interface() {
    let mut buffer = vec![0u8; 1024];
    let mut bus = FmiLsBus::new();
    let test_data = b"simple_test";

    // Demonstrate the cleaner API - no need to pass write_pos around
    bus.write_operation(
        LsBusCanOp::Transmit {
            id: 0x456,
            ide: 0,
            rtr: 0,
            data: Cow::Borrowed(test_data),
        },
        &mut buffer,
    )
    .unwrap();

    // Write position should be updated automatically
    assert!(bus.write_pos > 0);

    // Read back the operation
    let operation: Option<LsBusCanOp> = bus.read_next_operation(&buffer[..bus.write_pos]).unwrap();
    match operation {
        Some(LsBusCanOp::Transmit { id, data, .. }) => {
            assert_eq!(id, 0x456);
            assert_eq!(cow_bytes(&data), test_data);
        }
        _ => panic!("Expected Transmit operation"),
    }
}
