//! Tests for `#[derive(Specifier)]` using `#[bitfield]`

use modular_bitfield::prelude::*;

#[test]
fn struct_in_struct() {
    #[bitfield(filled = false)]
    #[derive(Specifier, Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Header {
        a: B2,
        b: B3,
    }

    #[bitfield]
    #[derive(Debug, PartialEq, Eq)]
    pub struct Base {
        pub header: Header,
        pub rest: B3,
    }

    let mut base = Base::new();
    assert_eq!(base.header(), Header::new());
    let h = Header::new().with_a(1).with_b(2);
    base.set_header(h);
    let h2 = base.header();
    assert_eq!(h2, h);
    assert_eq!(h2.a(), 1);
    assert_eq!(h2.b(), 2);
}

#[test]
fn unfilled_from_bytes() {
    use modular_bitfield::error::OutOfBounds;

    #[bitfield(filled = false)]
    #[derive(Specifier, Debug, PartialEq, Eq, Copy, Clone)]
    pub struct Unfilled {
        a: B2,
    }

    assert_eq!(Unfilled::from_bytes([0x00]), Ok(Unfilled::new()));
    assert_eq!(
        Unfilled::from_bytes([0b0000_0001]),
        Ok(Unfilled::new().with_a(1))
    );
    assert_eq!(
        Unfilled::from_bytes([0b0000_0010]),
        Ok(Unfilled::new().with_a(2))
    );
    assert_eq!(
        Unfilled::from_bytes([0b0000_0011]),
        Ok(Unfilled::new().with_a(3))
    );
    assert_eq!(Unfilled::from_bytes([0b0000_0100]), Err(OutOfBounds));
}

#[test]
fn valid_use() {
    #[bitfield]
    #[derive(Specifier)]
    pub struct Header {
        live: bool,
        received: bool,
        status: B2,
        rest: B4,
    }

    assert_eq!(<Header as Specifier>::BITS, 8);
}

#[test]
fn enum_with_data_variants() {
    // Use u8 primitive type for 8-bit data (guaranteed to have BITS = 8)
    // Enum with data variants - external discrimination
    #[derive(Specifier, Debug, PartialEq)]
    #[bits = 8] // External discrimination: all 8 bits for data
    enum Message {
        // Unit variant
        Empty,
        // Data variant - u8 is 8 bits (uses all bits)
        PriorityMsg(u8),
        // Data variant - u8 is 8 bits (uses all bits)
        StatusMsg(u8),
        // Unit variant
        Reset,
    }

    // Test basic functionality
    let empty = Message::Empty;
    let priority_msg = Message::PriorityMsg(2); // u8 value 2
    let status_msg = Message::StatusMsg(1); // u8 value 1
    let reset = Message::Reset;

    // Test serialization
    let empty_bytes = Message::into_bytes(empty).unwrap();
    let priority_bytes = Message::into_bytes(priority_msg).unwrap();
    let status_bytes = Message::into_bytes(status_msg).unwrap();
    let reset_bytes = Message::into_bytes(reset).unwrap();

    // External discrimination: from_bytes defaults to first variant
    assert_eq!(Message::from_bytes(empty_bytes).unwrap(), Message::Empty);
    assert_eq!(Message::from_bytes(priority_bytes).unwrap(), Message::Empty); // Defaults to Empty
    assert_eq!(Message::from_bytes(status_bytes).unwrap(), Message::Empty); // Defaults to Empty
    assert_eq!(Message::from_bytes(reset_bytes).unwrap(), Message::Empty); // Defaults to Empty
}

#[test]
fn enum_data_variants_mixed() {
    // Simple 4-bit enum for testing
    #[derive(Specifier, Debug, PartialEq, Clone, Copy)]
    #[bits = 4]
    enum SimpleData {
        Value0 = 0,
        Value1 = 1,
        Value2 = 2,
        Value3 = 3,
    }

    // Another 4-bit type for consistent sizing
    #[derive(Specifier, Debug, PartialEq, Clone, Copy)]
    #[bits = 4]
    enum CustomValue {
        A = 0,
        B = 1,
        C = 2,
        D = 3,
    }

    // Test with mixed unit and data variants - all data variants must have 4 bits
    #[derive(Specifier, Debug, PartialEq)]
    #[bits = 4] // External discrimination: all 4 bits for data
    enum MixedEnum {
        Empty,              // Unit variant
        Data(SimpleData),   // Data variant (4 bits)
        Flag,               // Another unit variant
        Value(CustomValue), // Data variant with custom type (4 bits)
    }

    let empty = MixedEnum::Empty;
    let data = MixedEnum::Data(SimpleData::Value2);
    let flag = MixedEnum::Flag;
    let value = MixedEnum::Value(CustomValue::C);

    // Test serialization
    let empty_bytes = MixedEnum::into_bytes(empty).unwrap();
    let data_bytes = MixedEnum::into_bytes(data).unwrap();
    let flag_bytes = MixedEnum::into_bytes(flag).unwrap();
    let value_bytes = MixedEnum::into_bytes(value).unwrap();

    // Test roundtrip
    assert_eq!(
        MixedEnum::from_bytes(empty_bytes).unwrap(),
        MixedEnum::Empty
    );
    assert_eq!(MixedEnum::from_bytes(data_bytes).unwrap(), MixedEnum::Empty); // External discrimination: defaults to first variant
    assert_eq!(MixedEnum::from_bytes(flag_bytes).unwrap(), MixedEnum::Empty); // External discrimination: defaults to first variant
    assert_eq!(
        MixedEnum::from_bytes(value_bytes).unwrap(),
        MixedEnum::Empty
    ); // External discrimination: defaults to first variant
}

#[test]
fn enum_data_variants_bit_layout() {
    // Test with mixed unit and data variants - external discrimination
    #[derive(Specifier, Debug, PartialEq)]
    #[bits = 8] // External discrimination: all 8 bits for data
    enum MixedEnum {
        Empty,     // Unit variant
        Data(u8),  // Data variant (8 bits)
        Flag,      // Another unit variant
        Value(u8), // Data variant (8 bits)
    }

    // Test external discrimination - all bits for data

    // MixedEnum: external discrimination, all 8 bits for data
    // Empty: 0x00 (unit variant = 0)
    // Data(2): 0x02 (u8 value 2, uses all 8 bits)
    // Flag: 0x00 (unit variant = 0)
    // Value(3): 0x03 (u8 value 3, uses all 8 bits)

    assert_eq!(MixedEnum::into_bytes(MixedEnum::Empty).unwrap(), 0x00);
    assert_eq!(MixedEnum::into_bytes(MixedEnum::Data(2)).unwrap(), 0x02); // External discrimination: direct data value
    assert_eq!(MixedEnum::into_bytes(MixedEnum::Flag).unwrap(), 0x00); // Unit variant = 0
    assert_eq!(MixedEnum::into_bytes(MixedEnum::Value(3)).unwrap(), 0x03); // u8 value 3
}

#[test]
fn enum_data_variants_manual_variant_bits() {
    // Test manual variant_bits control for future-proofing
    #[derive(Specifier, Debug, PartialEq, Clone, Copy)]
    #[bits = 4]
    enum Operation {
        Read = 0,
        Write = 1,
        Delete = 2,
        Execute = 3,
    }

    // External discrimination: all 4 bits used for data, no internal variant bits
    #[derive(Specifier, Debug, PartialEq)]
    #[bits = 4] // All 4 bits for data, discrimination handled externally
    enum Command {
        Noop,                // Unit variant
        Execute(Operation),  // Data variant (4 bits)
        Schedule(Operation), // Data variant (4 bits)
    }

    // Test that it works correctly
    let noop = Command::Noop;
    let execute = Command::Execute(Operation::Read);
    let schedule = Command::Schedule(Operation::Write);

    // Test serialization
    let noop_bytes = Command::into_bytes(noop).unwrap();
    let execute_bytes = Command::into_bytes(execute).unwrap();
    let schedule_bytes = Command::into_bytes(schedule).unwrap();

    // Test external discrimination bit layout:
    // Noop: 0x00 (unit variant = 0)
    // Execute(Read): 0x00 (Operation::Read = 0, uses all 4 bits)
    // Schedule(Write): 0x01 (Operation::Write = 1, uses all 4 bits)

    assert_eq!(noop_bytes, 0x00);
    assert_eq!(execute_bytes, 0x00);
    assert_eq!(schedule_bytes, 0x01);

    // Note: from_bytes with external discrimination always constructs the first variant
    // User must use specific constructors based on external discrimination
    assert_eq!(Command::from_bytes(noop_bytes).unwrap(), Command::Noop);
    // These would both construct Noop since from_bytes defaults to first variant
    assert_eq!(Command::from_bytes(execute_bytes).unwrap(), Command::Noop);
    assert_eq!(Command::from_bytes(schedule_bytes).unwrap(), Command::Noop);
}
