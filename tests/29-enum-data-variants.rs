use modular_bitfield::prelude::*;

// Simple data types that derive Specifier
#[derive(Specifier, Debug, PartialEq)]
#[bits = 4]
enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

#[derive(Specifier, Debug, PartialEq)]
#[bits = 3]
enum Status {
    Idle = 0,
    Running = 1,
    Stopped = 2,
    Error = 3,
}

// Enum with data variants - all variants must have same size
#[derive(Specifier, Debug, PartialEq)]
#[bits = 8]  // 2 bits discriminant + 6 bits data
enum Message {
    // Unit variant (discriminant = 0)
    Empty,
    // Data variant (discriminant = 1) - Priority is 4 bits, fits in 6 data bits
    PriorityMsg(Priority),
    // Data variant (discriminant = 2) - Status is 3 bits, fits in 6 data bits  
    StatusMsg(Status),
    // Unit variant (discriminant = 3)
    Reset,
}

#[test]
fn test_enum_with_data_variants() {
    // Test basic functionality
    let empty = Message::Empty;
    let priority_msg = Message::PriorityMsg(Priority::High);
    let status_msg = Message::StatusMsg(Status::Running);
    let reset = Message::Reset;

    // Test serialization
    let empty_bytes = Message::into_bytes(empty).unwrap();
    let priority_bytes = Message::into_bytes(priority_msg).unwrap();
    let status_bytes = Message::into_bytes(status_msg).unwrap();  
    let reset_bytes = Message::into_bytes(reset).unwrap();

    println!("Empty bytes: {:#04x}", empty_bytes);
    println!("Priority bytes: {:#04x}", priority_bytes);
    println!("Status bytes: {:#04x}", status_bytes);
    println!("Reset bytes: {:#04x}", reset_bytes);

    // Test deserialization roundtrip
    assert_eq!(Message::from_bytes(empty_bytes).unwrap(), Message::Empty);
    assert_eq!(Message::from_bytes(priority_bytes).unwrap(), Message::PriorityMsg(Priority::High));
    assert_eq!(Message::from_bytes(status_bytes).unwrap(), Message::StatusMsg(Status::Running));
    assert_eq!(Message::from_bytes(reset_bytes).unwrap(), Message::Reset);

    println!("All tests passed!");
}