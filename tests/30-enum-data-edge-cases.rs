use modular_bitfield::prelude::*;

// Simple 4-bit enum for testing
#[derive(Specifier, Debug, PartialEq, Clone, Copy)]
#[bits = 4]
enum SimpleData {
    Value0 = 0,
    Value1 = 1,
    Value2 = 2,
    Value3 = 3,
}

// Test with mixed unit and data variants
#[derive(Specifier, Debug, PartialEq)]
#[bits = 8]  // 2 bits discriminant + 6 bits data  
enum MixedEnum {
    Empty,                    // Unit variant
    Data(SimpleData),        // Data variant 
    Flag,                    // Another unit variant  
    Value(u8),               // Data variant with primitive 
}

#[test]
fn test_mixed_enum() {
    let empty = MixedEnum::Empty;
    let data = MixedEnum::Data(SimpleData::Value2);
    let flag = MixedEnum::Flag;
    let value = MixedEnum::Value(42u8);
    
    // Test serialization
    let empty_bytes = MixedEnum::into_bytes(empty).unwrap();
    let data_bytes = MixedEnum::into_bytes(data).unwrap();
    let flag_bytes = MixedEnum::into_bytes(flag).unwrap();
    let value_bytes = MixedEnum::into_bytes(value).unwrap();
    
    println!("Mixed enum bytes - Empty: {:#04x}, Data: {:#04x}, Flag: {:#04x}, Value: {:#04x}", 
             empty_bytes, data_bytes, flag_bytes, value_bytes);
    
    // Test roundtrip  
    assert_eq!(MixedEnum::from_bytes(empty_bytes).unwrap(), MixedEnum::Empty);
    assert_eq!(MixedEnum::from_bytes(data_bytes).unwrap(), MixedEnum::Data(SimpleData::Value2));
    assert_eq!(MixedEnum::from_bytes(flag_bytes).unwrap(), MixedEnum::Flag);
    assert_eq!(MixedEnum::from_bytes(value_bytes).unwrap(), MixedEnum::Value(42u8));
}

#[test]
fn test_bit_layouts() {
    // Test that discriminants are in high bits as expected
    
    // MixedEnum: 2 discriminant bits + 6 data bits = 8 total
    // Empty (disc=0): 0b0000_0000 = 0x00
    // Data(Value2) (disc=1): 0b0100_0010 = 0x42  (SimpleData::Value2 = 2)
    // Flag (disc=2): 0b1000_0000 = 0x80
    // Value(42) (disc=3): 0b1110_1010 = 0xEA (42 + (3 << 6))
    
    assert_eq!(MixedEnum::into_bytes(MixedEnum::Empty).unwrap(), 0x00);
    assert_eq!(MixedEnum::into_bytes(MixedEnum::Data(SimpleData::Value2)).unwrap(), 0x42); 
    assert_eq!(MixedEnum::into_bytes(MixedEnum::Flag).unwrap(), 0x80);
    assert_eq!(MixedEnum::into_bytes(MixedEnum::Value(42u8)).unwrap(), 0xEA);
}