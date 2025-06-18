//! Test for BITS_UNIFICATION_PLAN.md implementation
//! 
//! This test demonstrates the unified bits syntax and variable struct support
//! as described in the plan document.

use modular_bitfield::prelude::*;

#[test]
fn test_unified_bits_enum_syntax() {
    // Test positional tuple syntax for variable bits enum
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(8, 16, 32)]
    enum PositionalEnum {
        Small(u8),    // Must be ≤ 8 bits, discriminant = 0
        Medium(u16),  // Must be ≤ 16 bits, discriminant = 1
        Large(u32),   // Must be ≤ 32 bits, discriminant = 2
    }
    
    // Test basic properties
    assert_eq!(<PositionalEnum as Specifier>::BITS, 32); // Max size
    
    let small = PositionalEnum::Small(42);
    assert_eq!(small.discriminant(), 0);
    assert_eq!(small.size(), 8);
    
    let medium = PositionalEnum::Medium(1234);
    assert_eq!(medium.discriminant(), 1);
    assert_eq!(medium.size(), 16);
    
    let large = PositionalEnum::Large(0x12345678);
    assert_eq!(large.discriminant(), 2);
    assert_eq!(large.size(), 32);
}

#[test]
fn test_variable_struct_with_enum_data() {
    // Define the variable data enum
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(28, 60, 124)]  // Sizes after 4-bit discriminator
    enum MessageData {
        #[discriminant = 0]
        Small(u32),    // 28 bits
        #[discriminant = 1]
        Medium(u64),   // 60 bits
        #[discriminant = 2]
        Large(u128),   // 124 bits
    }
    
    // Variable-size container struct with validation
    #[bitfield(bits = (32, 64, 128))]  // Total sizes
    #[derive(Debug, Clone, Copy)]
    struct Message {
        #[variant_discriminator]
        msg_type: B4,
        #[variant_data]
        data: MessageData,
    }
    
    // Test constructors
    let _msg32 = Message::new_32bit();
    let _msg64 = Message::new_64bit();
    let _msg128 = Message::new_128bit();
    
    // Test supported sizes
    assert_eq!(Message::supported_sizes(), &[32, 64, 128]);
}

#[test]
fn test_explicit_bits_on_variants() {
    // Test explicit bits with cross-validation
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(8, 16, 32)]
    enum ExplicitEnum {
        #[discriminant = 0]
        #[bits = 8]
        Small(u8),
        #[discriminant = 1]
        #[bits = 16]
        Medium(u16),
        #[discriminant = 2]
        #[bits = 32]
        Large(u32),
    }
    
    // Note: The BITS constant is the maximum size
    assert_eq!(<ExplicitEnum as Specifier>::BITS, 32);
    
    // Test discriminant values
    let small = ExplicitEnum::Small(42);
    assert_eq!(small.discriminant(), 0);
    
    let medium = ExplicitEnum::Medium(1234);
    assert_eq!(medium.discriminant(), 1);
    
    let large = ExplicitEnum::Large(0x12345678);
    assert_eq!(large.discriminant(), 2);
}

#[test]
fn test_unit_and_data_variants_mixed() {
    // Test enum with both unit and data variants
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(8, 16)]
    enum MixedEnum {
        #[discriminant = 0]
        Empty,        // Unit variant
        #[discriminant = 1]
        WithData(u16), // Data variant
    }
    
    assert_eq!(<MixedEnum as Specifier>::BITS, 16);
    
    let empty = MixedEnum::Empty;
    assert_eq!(empty.discriminant(), 0);
    assert_eq!(empty.size(), 8);
    
    let with_data = MixedEnum::WithData(42);
    assert_eq!(with_data.discriminant(), 1);
    assert_eq!(with_data.size(), 16);
    
    // Test serialization roundtrip
    let empty_bytes = <MixedEnum as Specifier>::into_bytes(empty).unwrap();
    let reconstructed = MixedEnum::from_discriminant_and_bytes(0, empty_bytes).unwrap();
    assert_eq!(empty, reconstructed);
}

// Note: Some features from BITS_UNIFICATION_PLAN.md are not yet implemented:
// - Auto-detection of variable struct without explicit sizes
// - Name-based constructors (new_small, new_medium, new_large)
// - Error cases for mismatched configurations