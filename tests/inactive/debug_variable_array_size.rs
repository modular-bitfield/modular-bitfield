#[test]
fn test_variable_array_size() {
    use modular_bitfield::prelude::*;

    #[bitfield(variable_bits = (8, 16))]
    struct Test8_16 {
        #[variant_discriminator]
        disc: B4,
        fixed: B4,  // 8 bits total for fixed fields
        #[variant_data]
        data: B8,   // This will be variable
    }

    // The struct should have a 2-byte array to accommodate the 16-bit max size
    // Let's check the generated code
    let test = Test8_16::new();
    let bytes = test.into_bytes();
    
    println!("Test8_16 bytes array length: {}", bytes.len());
    assert_eq!(bytes.len(), 2, "Expected 2 bytes for 16-bit max size");
    
    // Test another size
    #[bitfield(variable_bits = (32, 64))]
    struct Test32_64 {
        #[variant_discriminator]
        disc: B4,
        fixed: B28,  // 32 bits total for fixed fields
        #[variant_data]
        data: B32,   // This will be variable
    }
    
    let test2 = Test32_64::new();
    let bytes2 = test2.into_bytes();
    
    println!("Test32_64 bytes array length: {}", bytes2.len());
    assert_eq!(bytes2.len(), 8, "Expected 8 bytes for 64-bit max size");
}