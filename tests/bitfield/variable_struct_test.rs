//! Test variable-size bitfield structs with variant_discriminator and variant_data

use modular_bitfield::prelude::*;

// First, define the variable bits enum
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[bits(28, 60, 124)]  // Remaining bits after 4-bit discriminator
enum UmpData {
    #[discriminant = 0]
    Utility(u32),    // 28 bits
    #[discriminant = 3]
    SysEx(u64),      // 60 bits
    #[discriminant = 5]
    Extended(u128),  // 124 bits
}

#[test]
fn test_variable_struct_parsing() {
    // Test that variant_discriminator and variant_data attributes are parsed
    #[bitfield(bits = (32, 64, 128))]
    #[derive(Debug, Clone, Copy)]
    struct UmpMessage {
        #[variant_discriminator]
        message_type: B4,
        #[variant_data]
        data: UmpData,
    }
    
    // Basic test - just see if it compiles
    let _msg = UmpMessage::new();
    
    // Test that generated constructors exist
    let _msg32 = UmpMessage::new_32bit();
    let _msg64 = UmpMessage::new_64bit();
    let _msg128 = UmpMessage::new_128bit();
    
    // Test supported_sizes method exists
    assert_eq!(UmpMessage::supported_sizes(), &[32, 64, 128]);
    
    // Test basic new() constructor
    let msg = UmpMessage::new();
    
    // The message_type field should work as normal
    assert_eq!(msg.message_type(), 0);
    
    // Test that data field accessor works
    let _data = msg.data();
}