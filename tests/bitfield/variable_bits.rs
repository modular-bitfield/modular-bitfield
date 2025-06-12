//! Basic variable_bits functionality tests

use modular_bitfield::prelude::*;

#[test]
fn basic_variable_enum() {
    #[derive(Specifier, Debug, Clone, Copy)]
    #[bits(8, 16, 32)]
    enum SimpleData {
        #[discriminant = 0]
        Small(u8), // For 8 bits: use u8 (can hold 0-255)
        #[discriminant = 1]
        Medium(u16), // For 16 bits: use u16 (can hold 0-65535)
        #[discriminant = 2]
        Large(u32), // For 32 bits: use u32 (can hold 0-4294967295)
    }

    // Test basic properties
    assert_eq!(<SimpleData as Specifier>::BITS, 32); // Max size

    // Test discriminant and size methods
    let small = SimpleData::Small(42);
    assert_eq!(small.discriminant(), 0);
    assert_eq!(small.size(), 8);

    let medium = SimpleData::Medium(1234);
    assert_eq!(medium.discriminant(), 1);
    assert_eq!(medium.size(), 16);

    let large = SimpleData::Large(0x12345678);
    assert_eq!(large.discriminant(), 2);
    assert_eq!(large.size(), 32);

    // Test helper functions
    assert_eq!(SimpleData::size_for_discriminant(0), Some(8));
    assert_eq!(SimpleData::size_for_discriminant(1), Some(16));
    assert_eq!(SimpleData::size_for_discriminant(2), Some(32));
    assert_eq!(SimpleData::size_for_discriminant(99), None);

    assert_eq!(SimpleData::supported_discriminants(), &[0, 1, 2]);
    assert_eq!(SimpleData::supported_sizes(), &[8, 16, 32]);
}

#[test]
fn variable_enum_serialization() {
    #[derive(Specifier, Debug, Clone, Copy)]
    #[bits(8, 16)]
    enum TestData {
        #[discriminant = 0]
        Small(u8),
        #[discriminant = 1]
        Large(u16),
    }

    let small = TestData::Small(255);
    let large = TestData::Large(65535);

    // Test serialization
    let small_bytes = TestData::into_bytes(small).unwrap();
    let large_bytes = TestData::into_bytes(large).unwrap();

    // Test deserialization with discriminant
    let decoded_small = TestData::from_discriminant_and_bytes(0, small_bytes).unwrap();
    let decoded_large = TestData::from_discriminant_and_bytes(1, large_bytes).unwrap();

    assert_eq!(decoded_small.discriminant(), 0);
    assert_eq!(decoded_large.discriminant(), 1);

    // Test invalid discriminant
    assert!(TestData::from_discriminant_and_bytes(99, small_bytes).is_err());
}

#[test]
fn variable_enum_inferred() {
    #[derive(Specifier, Debug, Clone, Copy)]
    #[bits(8, 16)]
    enum InferredData {
        #[discriminant = 0]
        #[bits = 8]
        Small(u8),
        #[discriminant = 1]
        #[bits = 16]
        Medium(u16),
    }

    // Same tests should work for inferred variant
    assert_eq!(<InferredData as Specifier>::BITS, 16);
    assert_eq!(InferredData::Small(42).size(), 8);
    assert_eq!(InferredData::Medium(1234).size(), 16);
}

#[test]
fn variable_enum_unit_variants() {
    #[derive(Specifier, Debug, Clone, Copy)]
    #[bits(8, 16)]
    enum MixedData {
        #[discriminant = 0]
        Empty, // Unit variant
        #[discriminant = 1]
        Data(u16),
    }

    assert_eq!(<MixedData as Specifier>::BITS, 16);
    assert_eq!(MixedData::Empty.size(), 8);
    assert_eq!(MixedData::Data(1234).size(), 16);

    // Test unit variant serialization
    let empty_bytes = MixedData::into_bytes(MixedData::Empty).unwrap();
    let reconstructed = MixedData::from_discriminant_and_bytes(0, empty_bytes).unwrap();
    assert_eq!(reconstructed.discriminant(), 0);
}

#[test]
fn variable_enum_odd_sizes() {
    // For odd sizes, we specify the data types that can hold those bits
    #[derive(Specifier, Debug, Clone, Copy)]
    #[bits(7, 13, 17)]
    enum OddSizedData {
        #[discriminant = 0]
        Small(u8), // Can hold 7 bits (up to 2^7-1 = 127)
        #[discriminant = 1]
        Medium(u16), // Can hold 13 bits (up to 2^13-1 = 8191)
        #[discriminant = 2]
        Large(u32), // Can hold 17 bits (up to 2^17-1 = 131071)
    }

    // Test basic properties
    assert_eq!(<OddSizedData as Specifier>::BITS, 17); // Max size

    // Test discriminant and size methods
    let small = OddSizedData::Small(127); // Max value for 7 bits (2^7 - 1)
    assert_eq!(small.discriminant(), 0);
    assert_eq!(small.size(), 7);

    let medium = OddSizedData::Medium(8191); // Max value for 13 bits (2^13 - 1)
    assert_eq!(medium.discriminant(), 1);
    assert_eq!(medium.size(), 13);

    let large = OddSizedData::Large(131071); // Max value for 17 bits (2^17 - 1)
    assert_eq!(large.discriminant(), 2);
    assert_eq!(large.size(), 17);

    // Test helper functions
    assert_eq!(OddSizedData::size_for_discriminant(0), Some(7));
    assert_eq!(OddSizedData::size_for_discriminant(1), Some(13));
    assert_eq!(OddSizedData::size_for_discriminant(2), Some(17));
    assert_eq!(OddSizedData::size_for_discriminant(99), None);

    assert_eq!(OddSizedData::supported_discriminants(), &[0, 1, 2]);
    assert_eq!(OddSizedData::supported_sizes(), &[7, 13, 17]);

    // Test validation: values that are too large should fail
    let too_large_small = OddSizedData::Small(128); // 128 = 2^7, too big for 7 bits
    assert!(OddSizedData::into_bytes(too_large_small).is_err());

    let too_large_medium = OddSizedData::Medium(8192); // 8192 = 2^13, too big for 13 bits
    assert!(OddSizedData::into_bytes(too_large_medium).is_err());

    // Values at the limit should work
    let max_small = OddSizedData::Small(127); // 127 = 2^7-1, max for 7 bits
    assert!(OddSizedData::into_bytes(max_small).is_ok());

    let max_medium = OddSizedData::Medium(8191); // 8191 = 2^13-1, max for 13 bits
    assert!(OddSizedData::into_bytes(max_medium).is_ok());
}

#[test]
fn variable_enum_large_discriminants() {
    // Test that discriminants can now use u16 range (beyond 255)
    #[derive(Specifier, Debug, Clone, Copy)]
    #[bits(8, 16)]
    enum LargeDiscriminantData {
        #[discriminant = 300] // > 255, would fail with old u8 limit
        First(u8),
        #[discriminant = 1000] // Much larger discriminant
        Second(u16),
    }

    // Test basic functionality
    assert_eq!(<LargeDiscriminantData as Specifier>::BITS, 16);

    let first = LargeDiscriminantData::First(42);
    assert_eq!(first.discriminant(), 300);
    assert_eq!(first.size(), 8);

    let second = LargeDiscriminantData::Second(1234);
    assert_eq!(second.discriminant(), 1000);
    assert_eq!(second.size(), 16);

    // Test helper functions work with large discriminants
    assert_eq!(LargeDiscriminantData::size_for_discriminant(300), Some(8));
    assert_eq!(LargeDiscriminantData::size_for_discriminant(1000), Some(16));
    assert_eq!(LargeDiscriminantData::size_for_discriminant(999), None);

    assert_eq!(
        LargeDiscriminantData::supported_discriminants(),
        &[300, 1000]
    );
    assert_eq!(LargeDiscriminantData::supported_sizes(), &[8, 16]);

    // Test serialization with large discriminants
    let first_bytes = LargeDiscriminantData::into_bytes(first).unwrap();
    let decoded_first =
        LargeDiscriminantData::from_discriminant_and_bytes(300, first_bytes).unwrap();
    assert_eq!(decoded_first.discriminant(), 300);
}
