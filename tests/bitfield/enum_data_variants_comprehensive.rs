//! Comprehensive tests for enums with data variants using the unified bits syntax

use modular_bitfield::prelude::*;

#[test]
fn test_variable_enum_basic_functionality() {
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(8, 16, 32)]
    enum TestData {
        #[discriminant = 0]
        Small(u8),
        #[discriminant = 1] 
        Medium(u16),
        #[discriminant = 2]
        Large(u32),
    }

    // Test basic properties
    assert_eq!(<TestData as Specifier>::BITS, 32); // Max size
    
    // Test variant creation and properties
    let small = TestData::Small(42);
    assert_eq!(small.discriminant(), 0);
    assert_eq!(small.size(), 8);
    
    let medium = TestData::Medium(1234);
    assert_eq!(medium.discriminant(), 1);
    assert_eq!(medium.size(), 16);
    
    let large = TestData::Large(0x12345678);
    assert_eq!(large.discriminant(), 2);
    assert_eq!(large.size(), 32);
}

#[test]
fn test_variable_enum_serialization() {
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(8, 16)]
    enum SimpleData {
        #[discriminant = 0]
        Small(u8),
        #[discriminant = 1]
        Large(u16),
    }

    let small = SimpleData::Small(255);
    let large = SimpleData::Large(65535);

    // Test Specifier trait methods
    let small_bytes = <SimpleData as Specifier>::into_bytes(small).unwrap();
    let large_bytes = <SimpleData as Specifier>::into_bytes(large).unwrap();
    
    assert_eq!(small_bytes, 255);
    assert_eq!(large_bytes, 65535);

    // Test discriminant-based reconstruction
    let decoded_small = SimpleData::from_discriminant_and_bytes(0, small_bytes).unwrap();
    let decoded_large = SimpleData::from_discriminant_and_bytes(1, large_bytes).unwrap();
    
    assert_eq!(decoded_small, small);
    assert_eq!(decoded_large, large);

    // Test invalid discriminant
    assert!(SimpleData::from_discriminant_and_bytes(99, small_bytes).is_err());
}

#[test]
fn test_variable_enum_mixed_unit_and_data_variants() {
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(4, 16)]
    enum MixedData {
        #[discriminant = 0]
        Empty, // Unit variant
        #[discriminant = 1]
        Value(u16), // Data variant
    }

    assert_eq!(<MixedData as Specifier>::BITS, 16); // Max size

    // Test unit variant
    let empty = MixedData::Empty;
    assert_eq!(empty.discriminant(), 0);
    assert_eq!(empty.size(), 4);
    
    // Test data variant
    let value = MixedData::Value(1234);
    assert_eq!(value.discriminant(), 1);
    assert_eq!(value.size(), 16);

    // Test serialization of unit variant
    let empty_bytes = <MixedData as Specifier>::into_bytes(empty).unwrap();
    assert_eq!(empty_bytes, 0);
    
    let reconstructed_empty = MixedData::from_discriminant_and_bytes(0, empty_bytes).unwrap();
    assert_eq!(reconstructed_empty, empty);
    
    // Test serialization of data variant
    let value_bytes = <MixedData as Specifier>::into_bytes(value).unwrap();
    let reconstructed_value = MixedData::from_discriminant_and_bytes(1, value_bytes).unwrap();
    assert_eq!(reconstructed_value, value);
}

#[test]
fn test_variable_enum_validation() {
    #[derive(Specifier, Debug, Clone, Copy)]
    #[bits(7, 13)] // Odd bit sizes
    enum ValidatedData {
        #[discriminant = 0]
        Small(u8), // Can hold 7 bits (0-127)
        #[discriminant = 1]
        Medium(u16), // Can hold 13 bits (0-8191)
    }

    // Test values within limits
    let small_valid = ValidatedData::Small(127); // 2^7 - 1
    assert!(<ValidatedData as Specifier>::into_bytes(small_valid).is_ok());
    
    let medium_valid = ValidatedData::Medium(8191); // 2^13 - 1  
    assert!(<ValidatedData as Specifier>::into_bytes(medium_valid).is_ok());

    // Test values at the edge of limits
    let small_edge = ValidatedData::Small(128); // 2^7, too big for 7 bits
    assert!(<ValidatedData as Specifier>::into_bytes(small_edge).is_err());
    
    let medium_edge = ValidatedData::Medium(8192); // 2^13, too big for 13 bits
    assert!(<ValidatedData as Specifier>::into_bytes(medium_edge).is_err());
}

#[test]
fn test_variable_enum_large_discriminants() {
    #[derive(Specifier, Debug, Clone, Copy)]
    #[bits(8, 16)]
    enum LargeDiscriminant {
        #[discriminant = 300] // > 255, tests u16 discriminant support
        First(u8),
        #[discriminant = 1000] // Large discriminant value
        Second(u16),
    }

    let first = LargeDiscriminant::First(42);
    assert_eq!(first.discriminant(), 300);
    assert_eq!(first.size(), 8);
    
    let second = LargeDiscriminant::Second(1234);
    assert_eq!(second.discriminant(), 1000);
    assert_eq!(second.size(), 16);

    // Test helper methods
    assert_eq!(LargeDiscriminant::size_for_discriminant(300), Some(8));
    assert_eq!(LargeDiscriminant::size_for_discriminant(1000), Some(16));
    assert_eq!(LargeDiscriminant::size_for_discriminant(500), None);
    
    assert_eq!(LargeDiscriminant::supported_discriminants(), &[300, 1000]);
    assert_eq!(LargeDiscriminant::supported_sizes(), &[8, 16]);
}

#[test]
fn test_variable_enum_helper_methods() {
    #[derive(Specifier, Debug, Clone, Copy)]
    #[bits(4, 8, 12)]
    enum HelperTest {
        #[discriminant = 10]
        Tiny(u8),
        #[discriminant = 20]
        Small(u8),
        #[discriminant = 30]
        Medium(u16),
    }

    // Test static helper methods
    assert_eq!(HelperTest::size_for_discriminant(10), Some(4));
    assert_eq!(HelperTest::size_for_discriminant(20), Some(8));
    assert_eq!(HelperTest::size_for_discriminant(30), Some(12));
    assert_eq!(HelperTest::size_for_discriminant(99), None);
    
    assert_eq!(HelperTest::supported_discriminants(), &[10, 20, 30]);
    assert_eq!(HelperTest::supported_sizes(), &[4, 8, 12]);

    // Test instance methods
    let tiny = HelperTest::Tiny(15);
    assert_eq!(tiny.discriminant(), 10);
    assert_eq!(tiny.size(), 4);
    
    let small = HelperTest::Small(255);
    assert_eq!(small.discriminant(), 20);
    assert_eq!(small.size(), 8);
    
    let medium = HelperTest::Medium(4095);
    assert_eq!(medium.discriminant(), 30);
    assert_eq!(medium.size(), 12);
}

#[test]
fn test_variable_enum_bytes_type_selection() {
    // Test that the correct Bytes type is chosen based on max size
    
    // 8-bit max should use u8
    #[derive(Specifier)]
    #[bits(4, 8)]
    enum ByteTypeTest8 {
        Small(u8),
        Large(u8),
    }
    
    // 16-bit max should use u16
    #[derive(Specifier)]
    #[bits(8, 16)]
    enum ByteTypeTest16 {
        Small(u8),
        Large(u16),
    }
    
    // 32-bit max should use u32
    #[derive(Specifier)]
    #[bits(16, 32)]
    enum ByteTypeTest32 {
        Small(u16),
        Large(u32),
    }

    // We can't directly test the type, but we can verify the BITS constant
    assert_eq!(<ByteTypeTest8 as Specifier>::BITS, 8);
    assert_eq!(<ByteTypeTest16 as Specifier>::BITS, 16);
    assert_eq!(<ByteTypeTest32 as Specifier>::BITS, 32);
}

#[test]
fn test_variable_enum_round_trip_all_variants() {
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(8, 16, 24, 32)]
    enum RoundTripTest {
        #[discriminant = 0]
        A(u8),
        #[discriminant = 1]
        B(u16),
        #[discriminant = 2]
        C(u32), // Will be truncated to 24 bits
        #[discriminant = 3]
        D(u32),
    }

    let test_cases = [
        (RoundTripTest::A(255), 0u16),
        (RoundTripTest::B(65535), 1u16),
        (RoundTripTest::C(16777215), 2u16), // 2^24 - 1, max for 24 bits
        (RoundTripTest::D(0x12345678), 3u16),
    ];

    for (original, expected_discriminant) in test_cases {
        // Test discriminant
        assert_eq!(original.discriminant(), expected_discriminant);
        
        // Test serialization
        let bytes = <RoundTripTest as Specifier>::into_bytes(original).unwrap();
        
        // Test deserialization with correct discriminant
        let reconstructed = RoundTripTest::from_discriminant_and_bytes(expected_discriminant, bytes).unwrap();
        assert_eq!(reconstructed, original);
    }
}

#[test]
fn test_variable_enum_error_cases() {
    #[derive(Specifier, Debug, Clone, Copy)]
    #[bits(8, 16)]
    enum ErrorTest {
        #[discriminant = 0]
        Small(u8),
        #[discriminant = 1]
        Large(u16),
    }

    // Test invalid discriminant in from_discriminant_and_bytes
    assert!(ErrorTest::from_discriminant_and_bytes(99, 0).is_err());
    
    // Test that values too large for their bit allocation fail
    let oversized_small = ErrorTest::Small(255); // This should be fine for 8 bits
    assert!(<ErrorTest as Specifier>::into_bytes(oversized_small).is_ok());
    
    // Note: We can't easily test oversized values since Rust's type system 
    // prevents us from creating u8 > 255, but the validation is in place
    // for when the data comes from external sources
}

#[test]
fn test_variable_enum_with_different_data_types() {
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(1, 4, 8)]
    enum TypeTest {
        #[discriminant = 0]
        Flag(bool), // 1 bit
        #[discriminant = 1]
        Nibble(u8), // 4 bits - will validate that value fits
        #[discriminant = 2]
        Byte(u8), // 8 bits
    }

    let flag = TypeTest::Flag(true);
    assert_eq!(flag.discriminant(), 0);
    assert_eq!(flag.size(), 1);
    
    let nibble = TypeTest::Nibble(15); // Max value for 4 bits
    assert_eq!(nibble.discriminant(), 1);
    assert_eq!(nibble.size(), 4);
    
    let byte = TypeTest::Byte(255); // Max value for 8 bits
    assert_eq!(byte.discriminant(), 2);
    assert_eq!(byte.size(), 8);

    // Test serialization and round-trip
    for variant in [flag, nibble, byte] {
        let bytes = <TypeTest as Specifier>::into_bytes(variant).unwrap();
        let reconstructed = TypeTest::from_discriminant_and_bytes(variant.discriminant(), bytes).unwrap();
        assert_eq!(reconstructed, variant);
    }
    
    // Test that oversized values for 4-bit nibble fail validation
    let oversized_nibble = TypeTest::Nibble(16); // 16 = 2^4, too big for 4 bits
    assert!(<TypeTest as Specifier>::into_bytes(oversized_nibble).is_err());
}

#[test]
fn test_variable_enum_consistent_with_fixed_enum() {
    // Test that a variable enum with one size behaves like a fixed enum
    
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits = 16] // Fixed size
    enum FixedEnum {
        Value(u16),
    }

    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(16)] // Variable size with one option
    enum VariableEnum {
        #[discriminant = 0]
        Value(u16),
    }

    // Both should have the same BITS
    assert_eq!(<FixedEnum as Specifier>::BITS, <VariableEnum as Specifier>::BITS);
    
    // Test serialization compatibility
    let fixed_val = FixedEnum::Value(1234);
    let variable_val = VariableEnum::Value(1234);
    
    let fixed_bytes = <FixedEnum as Specifier>::into_bytes(fixed_val).unwrap();
    let variable_bytes = <VariableEnum as Specifier>::into_bytes(variable_val).unwrap();
    
    assert_eq!(fixed_bytes, variable_bytes);
}