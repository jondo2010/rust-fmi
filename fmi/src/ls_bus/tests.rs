use super::*;

#[test]
fn test_buffer_creation_and_basic_operations() {
    let mut buffer = FmiLsBus::new(1024);

    // Test initial state
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);

    // Test reset
    buffer.reset();
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
}
