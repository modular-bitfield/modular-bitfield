use modular_bitfield::prelude::*;

extern crate alloc;
use alloc::format;

/// Tests for const evaluation edge cases
#[cfg(test)]
mod const_evaluation_edge_cases {
    use super::*;

    #[test]
    fn max_values_for_types() {
        #[bitfield]
        pub struct MaxValues {
            #[default(255)] // u8::MAX for B8
            max_b8: B8,
            #[default(127)] // Max for B7
            max_b7: B7,
            #[default(1)] // Max for B1
            max_b1: B1,
        }

        const BF: MaxValues = MaxValues::new();
        assert_eq!(BF.max_b8(), 255);
        assert_eq!(BF.max_b7(), 127);
        assert_eq!(BF.max_b1(), 1);
    }

    #[test]
    fn const_expressions() {
        #[bitfield]
        pub struct ConstExpressions {
            #[default(1 << 3)] // Shift operation
            shifted: B4,
            #[default(0xFF & 0x0F)] // Bitwise AND
            masked: B4,
            #[default(7 + 1)] // Addition
            added: B4,
            #[default(15 - 3)] // Subtraction
            subtracted: B4,
        }

        const BF: ConstExpressions = ConstExpressions::new();
        assert_eq!(BF.shifted(), 8); // 1 << 3
        assert_eq!(BF.masked(), 15); // 0xFF & 0x0F
        assert_eq!(BF.added(), 8); // 7 + 1
        assert_eq!(BF.subtracted(), 12); // 15 - 3
    }

    #[test]
    fn const_variables_as_defaults() {
        const DEFAULT_VALUE: u8 = 42;
        const ANOTHER_VALUE: u16 = 0x1234;

        #[bitfield]
        pub struct ConstVariables {
            #[default(DEFAULT_VALUE)]
            field1: B8,
            #[default(ANOTHER_VALUE)]
            field2: B16,
        }

        const BF: ConstVariables = ConstVariables::new();
        assert_eq!(BF.field1(), 42);
        assert_eq!(BF.field2(), 0x1234);
    }
}

/// Tests for bit manipulation edge cases
#[cfg(test)]
mod bit_manipulation_edge_cases {
    use super::*;

    #[test]
    fn cross_byte_boundaries() {
        #[bitfield]
        pub struct CrossByte {
            prefix: B3, // Takes first 3 bits of byte 0
            #[default(0xFF)] // B8 field starting at bit 3, spans byte 0-1
            cross_field: B8,
            #[default(0x0F)] // B4 field in middle of byte 1
            middle: B4,
            suffix: B1, // Last bit of byte 1
        }

        const BF: CrossByte = CrossByte::new();
        assert_eq!(BF.cross_field(), 0xFF);
        assert_eq!(BF.middle(), 0x0F);
        assert_eq!(BF.prefix(), 0); // No default, should be 0
        assert_eq!(BF.suffix(), 0); // No default, should be 0

        // Verify total is 16 bits (2 bytes)
        let bytes = BF.into_bytes();
        assert_eq!(bytes.len(), 2);
    }

    #[test]
    fn large_multi_byte_fields() {
        #[bitfield]
        pub struct LargeFields {
            #[default(0x12345678)] // 32-bit default
            large32: B32,
            #[default(0x9ABC)] // 16-bit default
            medium16: B16,
            #[default(0xDEF)] // 12-bit default
            medium12: B12,
            padding: B4,
        }

        const BF: LargeFields = LargeFields::new();
        assert_eq!(BF.large32(), 0x12345678);
        assert_eq!(BF.medium16(), 0x9ABC);
        assert_eq!(BF.medium12(), 0xDEF);
        assert_eq!(BF.padding(), 0);
    }

    #[test]
    fn unaligned_fields() {
        #[bitfield]
        pub struct UnalignedFields {
            offset1: B1,
            #[default(0x7F)] // B7 field offset by 1 bit
            field7: B7,
            offset2: B2,
            #[default(0x3F)] // B6 field offset by 2 bits from byte boundary
            field6: B6,
        }

        const BF: UnalignedFields = UnalignedFields::new();
        assert_eq!(BF.field7(), 0x7F);
        assert_eq!(BF.field6(), 0x3F);
        assert_eq!(BF.offset1(), 0);
        assert_eq!(BF.offset2(), 0);
    }

    #[test]
    fn boundary_values() {
        #[bitfield]
        pub struct BoundaryValues {
            #[default(1)] // Minimum non-zero
            min_b1: B1,
            #[default(3)] // Maximum for B2
            max_b2: B2,
            #[default(7)] // Maximum for B3
            max_b3: B3,
            #[default(15)] // Maximum for B4
            max_b4: B4,
            #[default(31)] // Maximum for B5
            max_b5: B5,
            #[default(63)] // Maximum for B6
            max_b6: B6,
            padding: B3, // Add padding to make 24 bits (3 bytes)
        }

        const BF: BoundaryValues = BoundaryValues::new();
        assert_eq!(BF.min_b1(), 1);
        assert_eq!(BF.max_b2(), 3);
        assert_eq!(BF.max_b3(), 7);
        assert_eq!(BF.max_b4(), 15);
        assert_eq!(BF.max_b5(), 31);
        assert_eq!(BF.max_b6(), 63);
        assert_eq!(BF.padding(), 0);
    }
}

/// Tests for enum edge cases
#[cfg(test)]
mod enum_edge_cases {
    use super::*;

    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits = 8]
    pub enum MaxEnum {
        First = 0,
        Last = 255, // Maximum u8 value
    }

    #[derive(Specifier, Debug, Clone, Copy, PartialEq)]
    #[bits = 3]
    pub enum SmallEnum {
        A = 0,
        B = 7, // Maximum for 3 bits
    }

    #[test]
    fn enum_max_discriminants() {
        #[bitfield]
        pub struct EnumMax {
            #[default(MaxEnum::Last)]
            max_enum: MaxEnum,
            #[default(SmallEnum::B)]
            small_enum: SmallEnum,
            padding: B5, // Add padding to make 16 bits (2 bytes)
        }

        const BF: EnumMax = EnumMax::new();
        assert_eq!(BF.max_enum(), MaxEnum::Last);
        assert_eq!(BF.small_enum(), SmallEnum::B);
        assert_eq!(BF.padding(), 0);
    }

    #[test]
    fn enum_zero_discriminants() {
        #[bitfield]
        pub struct EnumZero {
            #[default(MaxEnum::First)]
            zero_enum: MaxEnum,
            #[default(SmallEnum::A)]
            small_zero: SmallEnum,
            padding: B5, // Add padding to make 16 bits (2 bytes)
        }

        const BF: EnumZero = EnumZero::new();
        assert_eq!(BF.zero_enum(), MaxEnum::First);
        assert_eq!(BF.small_zero(), SmallEnum::A);
        assert_eq!(BF.padding(), 0);
    }
}

/// Tests for const context verification
#[cfg(test)]
mod const_context_tests {
    use super::*;

    #[test]
    fn const_construction_in_various_contexts() {
        #[bitfield]
        pub struct ConstTest {
            #[default(true)]
            flag: bool,
            #[default(42)]
            value: B7, // Changed to B7 to make total 8 bits (1 byte)
        }

        // Test in const context
        const CONST_BF: ConstTest = ConstTest::new();
        assert!(CONST_BF.flag());
        assert_eq!(CONST_BF.value(), 42);

        // Test in static context
        static STATIC_BF: ConstTest = ConstTest::new();
        assert!(STATIC_BF.flag());
        assert_eq!(STATIC_BF.value(), 42);

        // Test new_zeroed in const context
        const ZERO_BF: ConstTest = ConstTest::new_zeroed();
        assert!(!ZERO_BF.flag());
        assert_eq!(ZERO_BF.value(), 0);
    }

    #[test]
    fn array_of_const_bitfields() {
        #[bitfield]
        pub struct ArrayTest {
            #[default(0xAB)]
            data: B8,
        }

        const ARRAY: [ArrayTest; 3] = [ArrayTest::new(), ArrayTest::new_zeroed(), ArrayTest::new()];

        assert_eq!(ARRAY[0].data(), 0xAB);
        assert_eq!(ARRAY[1].data(), 0);
        assert_eq!(ARRAY[2].data(), 0xAB);
    }
}

/// Tests for integration with other bitfield features
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn defaults_with_bytes_parameter() {
        #[bitfield(bytes = 4)]
        pub struct WithBytes {
            #[default(0xDEAD)]
            field1: B16,
            #[default(0xBEEF)]
            field2: B16,
        }

        const BF: WithBytes = WithBytes::new();
        assert_eq!(BF.field1(), 0xDEAD);
        assert_eq!(BF.field2(), 0xBEEF);

        let bytes = BF.into_bytes();
        assert_eq!(bytes.len(), 4);
    }

    #[test]
    fn defaults_with_unfilled_bitfield() {
        #[bitfield(filled = false)]
        pub struct Unfilled {
            #[default(0x7)]
            field1: B3,
            #[default(true)]
            field2: bool,
            // Only 4 bits used, 4 bits unused in the byte
        }

        const BF: Unfilled = Unfilled::new();
        assert_eq!(BF.field1(), 0x7);
        assert!(BF.field2());
    }

    #[test]
    fn defaults_with_repr() {
        #[repr(u32)]
        #[bitfield]
        pub struct WithRepr {
            #[default(0x12345678)]
            field1: B32,
        }

        const BF: WithRepr = WithRepr::new();
        assert_eq!(BF.field1(), 0x12345678);

        // Test repr conversion
        let as_u32: u32 = BF.into();
        assert_eq!(as_u32, 0x12345678);

        let from_u32 = WithRepr::from(0x87654321);
        assert_eq!(from_u32.field1(), 0x87654321);
    }

    #[test]
    fn defaults_with_debug_derive() {
        #[bitfield]
        #[derive(Debug)]
        pub struct WithDebug {
            #[default(true)]
            flag: bool,
            #[default(42)]
            value: B7,
        }

        const BF: WithDebug = WithDebug::new();
        let debug_str = format!("{:?}", BF);
        assert!(debug_str.contains("flag: true"));
        assert!(debug_str.contains("value: 42"));
    }
}

/// Tests for complex scenarios
#[cfg(test)]
mod complex_scenarios {
    use super::*;

    #[test]
    fn all_primitive_types_with_defaults() {
        #[bitfield]
        pub struct AllPrimitives {
            #[default(true)]
            bool_field: bool,
            #[default(255)]
            u8_field: u8,
            #[default(0x1234)]
            u16_field: u16,
            #[default(0x12345678)]
            u32_field: u32,
            padding: B7, // Add padding to make 64 bits (8 bytes)
        }

        const BF: AllPrimitives = AllPrimitives::new();
        assert!(BF.bool_field());
        assert_eq!(BF.u8_field(), 255);
        assert_eq!(BF.u16_field(), 0x1234);
        assert_eq!(BF.u32_field(), 0x12345678);
        assert_eq!(BF.padding(), 0);
    }

    #[test]
    fn mixed_defaults_and_skipped_fields() {
        #[bitfield]
        pub struct MixedFields {
            #[default(0xAB)]
            normal: B8,
            #[skip(getters)]
            write_only: B8,
            #[skip(setters)]
            read_only: B8,
            #[default(0xCD)]
            normal2: B8,
        }

        const BF: MixedFields = MixedFields::new();
        assert_eq!(BF.normal(), 0xAB);
        assert_eq!(BF.normal2(), 0xCD);
        assert_eq!(BF.read_only(), 0); // No default applied to read-only field
    }

    #[test]
    fn nested_specifier_with_defaults() {
        #[bitfield]
        #[derive(Specifier)]
        pub struct NestedBitfield {
            #[default(true)]
            inner_flag: bool,
            #[default(0x7)]
            inner_value: B3,
            padding: B4,
        }

        #[bitfield]
        pub struct OuterBitfield {
            prefix: B8,
            // Note: Using NestedBitfield::new() in default won't work yet
            // as it's not const-evaluable, so we leave this without default
            nested: NestedBitfield,
            #[default(0xFF)]
            suffix: B8,
        }

        const OUTER: OuterBitfield = OuterBitfield::new();
        assert_eq!(OUTER.prefix(), 0);
        assert_eq!(OUTER.suffix(), 0xFF);

        // The nested field should be zero-initialized since no default is applied
        let nested = OUTER.nested();
        assert!(!nested.inner_flag());
        assert_eq!(nested.inner_value(), 0);
    }
}
