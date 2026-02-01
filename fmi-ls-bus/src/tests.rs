use super::*;

#[test]
fn test_buffer_creation_and_basic_operations() {
    let mut buffer = vec![0u8; 1024];
    let mut bus = FmiLsBus::new();

    // Test initial state
    assert_eq!(bus.read_pos, 0);
    assert_eq!(bus.write_pos, 0);

    // Test writing some data
    let test_data = b"test";
    bus.write(&mut buffer, test_data).unwrap();
    assert_eq!(bus.write_pos, test_data.len());
    assert_eq!(&buffer[..test_data.len()], test_data);

    // Test reset - positions reset but buffer content remains
    bus.reset();
    assert_eq!(bus.read_pos, 0);
    assert_eq!(bus.write_pos, 0);
    assert_eq!(&buffer[..test_data.len()], test_data); // Buffer content unchanged
}

#[test]
fn test_position_tracking() {
    let mut buffer = vec![0u8; 1024];
    let mut bus = FmiLsBus::new();

    // Initial positions should be 0
    assert_eq!(bus.read_pos, 0);
    assert_eq!(bus.write_pos, 0);

    // Test writing some data
    let test_data = b"Hello, World!";
    bus.write(&mut buffer, test_data).unwrap();

    // Write position should be updated, read position stays at 0
    assert_eq!(bus.read_pos, 0);
    assert_eq!(bus.write_pos, test_data.len());
    assert_eq!(&buffer[..test_data.len()], test_data);

    // Test reset
    bus.reset();
    assert_eq!(bus.read_pos, 0);
    assert_eq!(bus.write_pos, 0);
    assert_eq!(&buffer[..test_data.len()], test_data); // reset doesn't clear buffer, just positions
}
