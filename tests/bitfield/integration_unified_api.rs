//! Integration test demonstrating the ideal unified API
//! as proposed in BITS_UNIFICATION_PLAN.md

use modular_bitfield::prelude::*;

#[test]
fn test_ideal_midi_ump_api() {
    // Variable-size data enum
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
    
    // Variable-size container struct with explicit validation
    #[bitfield(bits = (32, 64, 128))]  // Total sizes
    #[derive(Debug, Clone, Copy)]
    struct UmpMessage {
        #[variant_discriminator]
        message_type: B4,
        #[variant_data]
        data: UmpData,
    }
    
    // Test size-based constructors
    let msg32 = UmpMessage::new_32bit();
    let msg64 = UmpMessage::new_64bit();
    let msg128 = UmpMessage::new_128bit();
    
    // These constructors exist and compile
    assert_eq!(core::mem::size_of_val(&msg32), core::mem::size_of_val(&msg64));
    assert_eq!(core::mem::size_of_val(&msg64), core::mem::size_of_val(&msg128));
    
    // Test that supported sizes are correct
    assert_eq!(UmpMessage::supported_sizes(), &[32, 64, 128]);
    
    // Note: The following features from BITS_UNIFICATION_PLAN.md are not yet implemented:
    // - Name-based constructors (new_small, new_medium, new_large)
    // - Automatic size inference from enum
    // - Proper integration between discriminator and data variant selection
}

#[test]
fn test_comprehensive_variable_enum() {
    // Test all variable enum features
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(8, 16, 32, 64)]
    enum ComprehensiveData {
        #[discriminant = 0]
        Tiny(u8),       // 8 bits
        #[discriminant = 1]
        Small(u16),     // 16 bits
        #[discriminant = 2]
        Medium(u32),    // 32 bits
        #[discriminant = 3]
        Large(u64),     // 64 bits
    }
    
    // Test all helper methods
    assert_eq!(ComprehensiveData::supported_discriminants(), &[0, 1, 2, 3]);
    assert_eq!(ComprehensiveData::supported_sizes(), &[8, 16, 32, 64]);
    
    // Test size lookups
    assert_eq!(ComprehensiveData::size_for_discriminant(0), Some(8));
    assert_eq!(ComprehensiveData::size_for_discriminant(1), Some(16));
    assert_eq!(ComprehensiveData::size_for_discriminant(2), Some(32));
    assert_eq!(ComprehensiveData::size_for_discriminant(3), Some(64));
    assert_eq!(ComprehensiveData::size_for_discriminant(99), None);
    
    // Test variant operations
    let tiny = ComprehensiveData::Tiny(255);
    assert_eq!(tiny.discriminant(), 0);
    assert_eq!(tiny.size(), 8);
    
    let small = ComprehensiveData::Small(65535);
    assert_eq!(small.discriminant(), 1);
    assert_eq!(small.size(), 16);
    
    let medium = ComprehensiveData::Medium(0xFFFFFFFF);
    assert_eq!(medium.discriminant(), 2);
    assert_eq!(medium.size(), 32);
    
    let large = ComprehensiveData::Large(0xFFFFFFFF_FFFFFFFF);
    assert_eq!(large.discriminant(), 3);
    assert_eq!(large.size(), 64);
    
    // Test serialization
    let tiny_bytes = <ComprehensiveData as Specifier>::into_bytes(tiny).unwrap();
    assert_eq!(tiny_bytes, 255u64);
    
    let reconstructed = ComprehensiveData::from_discriminant_and_bytes(0, tiny_bytes).unwrap();
    assert_eq!(reconstructed, tiny);
}

#[test]
fn test_variable_struct_validation() {
    // Test struct with multiple fields including fixed ones
    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits(24, 56)]  // 8 + 24 = 32, 8 + 56 = 64
    enum PayloadData {
        #[discriminant = 0]
        Short(u32),   // 24 bits
        #[discriminant = 1]
        Long(u64),    // 56 bits
    }
    
    #[bitfield(bits = (32, 64))]
    #[derive(Debug, Clone, Copy)]
    struct VariablePacket {
        header: B8,  // Fixed field
        #[variant_discriminator]
        packet_type: B2,  // Only 2 bits for discriminator
        reserved: B6,     // Fixed padding
        #[variant_data]
        payload: PayloadData,
    }
    
    // Test constructors
    let _packet32 = VariablePacket::new_32bit();
    let _packet64 = VariablePacket::new_64bit();
    
    // Test field access
    let mut packet = VariablePacket::new();
    packet.set_header(0xFF);
    assert_eq!(packet.header(), 0xFF);
    
    packet.set_reserved(0x3F);
    assert_eq!(packet.reserved(), 0x3F);
    
    // Note: Full integration between packet_type and payload selection
    // is not yet implemented as described in BITS_UNIFICATION_PLAN.md
}