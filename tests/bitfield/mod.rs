mod bits_param;
mod bytes_param;
mod derive_bitfield_specifier;
mod derive_debug;
mod derive_specifier;
mod filled_param;
mod no_implicit_prelude;
mod regressions;
mod repr;
mod skip;

use modular_bitfield::prelude::*;

#[test]
fn accessors() {
    #[bitfield]
    pub struct MyFourBytes {
        a: B1,
        b: B3,
        c: B4,
        d: B24,
    }

    let mut bitfield = MyFourBytes::new();
    assert_eq!(0, bitfield.a());
    assert_eq!(0, bitfield.b());
    assert_eq!(0, bitfield.c());
    assert_eq!(0, bitfield.d());

    bitfield.set_c(14);
    assert_eq!(0, bitfield.a());
    assert_eq!(0, bitfield.b());
    assert_eq!(14, bitfield.c());
    assert_eq!(0, bitfield.d());
}

#[test]
fn accessor_signatures() {
    use core::mem::size_of_val;

    // For getters and setters, we would like for the signature to be in terms of
    // the narrowest unsigned integer type that can hold the right number of bits.
    // That means the accessors for B1 through B8 would use u8, B9 through B16 would
    // use u16 etc.

    type A = B1;
    type B = B3;
    type C = B4;
    type D = B24;

    #[bitfield]
    pub struct MyFourBytes {
        a: A,
        b: B,
        c: C,
        d: D,
    }

    let mut x = MyFourBytes::new();

    // I am testing the signatures in this roundabout way to avoid making it
    // possible to pass this test with a generic signature that is inconvenient
    // for callers, such as `fn a<T: From<u64>>(&self) -> T`.

    let a = 1;
    x.set_a(a); // expect fn(&mut MyFourBytes, u8)
    let b = 1;
    x.set_b(b);
    let c = 1;
    x.set_c(c);
    let d = 1;
    x.set_d(d); // expect fn(&mut MyFourBytes, u32)

    assert_eq!(size_of_val(&a), 1);
    assert_eq!(size_of_val(&b), 1);
    assert_eq!(size_of_val(&c), 1);
    assert_eq!(size_of_val(&d), 4);

    assert_eq!(size_of_val(&x.a()), 1); // expect fn(&MyFourBytes) -> u8
    assert_eq!(size_of_val(&x.b()), 1);
    assert_eq!(size_of_val(&x.c()), 1);
    assert_eq!(size_of_val(&x.d()), 4); // expect fn(&MyFourBytes) -> u32
}

// This test is equivalent to accessors but with some fields spanning across
// byte boundaries. This may or may not already work depending on how your
// implementation has been done so far.
//
//
//     ║  first byte   ║  second byte  ║  third byte   ║  fourth byte  ║
//     ╟───────────────╫───────────────╫───────────────╫───────────────╢
//     ║▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒║
//     ╟─────────────────╫───────────╫─────────────────────────╫───────╢
//     ║        a        ║     b     ║            c            ║   d   ║
#[test]
fn accessors_edge() {
    #[bitfield]
    pub struct EdgeCaseBytes {
        a: B9,
        b: B6,
        c: B13,
        d: B4,
    }

    let mut bitfield = EdgeCaseBytes::new();
    assert_eq!(0, bitfield.a());
    assert_eq!(0, bitfield.b());
    assert_eq!(0, bitfield.c());
    assert_eq!(0, bitfield.d());

    let a = 0b1_1000_0111;
    let b = 0b101_010;
    let c = 0x1675;
    let d = 0b1110;

    bitfield.set_a(a);
    bitfield.set_b(b);
    bitfield.set_c(c);
    bitfield.set_d(d);

    assert_eq!(a, bitfield.a());
    assert_eq!(b, bitfield.b());
    assert_eq!(c, bitfield.c());
    assert_eq!(d, bitfield.d());
}

// We also want to allow for tuple structs to be accepted by the `#[bitfield]` macro.
//
// For this we generate getters and setters in a way that refer to the corresponding
// number of the field within the annotated tuple struct.
#[test]
fn tuple_structs() {
    #[bitfield]
    struct MyTwoBytes(bool, B7, B8);

    let mut test = MyTwoBytes::new();

    assert!(!test.get_0());
    assert_eq!(test.get_1(), 0);
    assert_eq!(test.get_2(), 0);

    test.set_0(true);
    test.set_1(42);
    test.set_2(0xFF);

    assert!(test.get_0());
    assert_eq!(test.get_1(), 42);
    assert_eq!(test.get_2(), 0xFF);
}

// Tests to check for correct execution of checked setters.
#[test]
fn checked_setters() {
    use modular_bitfield::error::OutOfBounds;

    #[bitfield]
    #[derive(Debug, PartialEq)]
    pub struct MyTwoBytes {
        a: B1,
        b: B2,
        c: B13,
    }

    let mut bitfield = MyTwoBytes::new();

    // Everything is initialized to zero.
    assert_eq!(bitfield.a(), 0);
    assert_eq!(bitfield.b(), 0);
    assert_eq!(bitfield.c(), 0);

    // Do some invalid manipulations.
    assert_eq!(bitfield.set_a_checked(2), Err(OutOfBounds));
    assert_eq!(bitfield.set_b_checked(4), Err(OutOfBounds));
    assert_eq!(bitfield.set_c_checked(12345), Err(OutOfBounds));

    // Asserts that nothing has changed.
    assert_eq!(bitfield.a(), 0);
    assert_eq!(bitfield.b(), 0);
    assert_eq!(bitfield.c(), 0);

    // Do some valid manipulations.
    assert_eq!(bitfield.set_a_checked(1), Ok(()));
    assert_eq!(bitfield.set_b_checked(3), Ok(()));
    assert_eq!(bitfield.set_c_checked(42), Ok(()));

    // Asserts that the valid manipulation has had effect.
    assert_eq!(bitfield.a(), 1);
    assert_eq!(bitfield.b(), 3);
    assert_eq!(bitfield.c(), 42);

    // Check the checked with statement throws error
    assert_eq!(MyTwoBytes::new().with_a_checked(2), Err(OutOfBounds));
    assert_eq!(
        MyTwoBytes::new()
            .with_a_checked(1)
            .unwrap()
            .with_b_checked(4),
        Err(OutOfBounds)
    );

    // Check that with_checked populates values without touching other fields
    let bitfield = bitfield
        .with_a_checked(0)
        .unwrap()
        .with_b_checked(2)
        .unwrap();

    assert_eq!(bitfield.a(), 0);
    assert_eq!(bitfield.b(), 2);
    assert_eq!(bitfield.c(), 42);
}

// Tests if it is possible to manually reset the bitfields again.
#[test]
fn manual_reset() {
    #[bitfield]
    pub struct MyTwoBytes {
        a: B1,
        b: B2,
        c: B13,
    }

    let mut bitfield = MyTwoBytes::new();

    // Everything is initialized to zero.
    assert_eq!(bitfield.a(), 0);
    assert_eq!(bitfield.b(), 0);
    assert_eq!(bitfield.c(), 0);

    // Manipulate bitfield.
    bitfield.set_a(1);
    bitfield.set_b(3);
    bitfield.set_c(42);

    // Check that manipulation was successful.
    assert_eq!(bitfield.a(), 1);
    assert_eq!(bitfield.b(), 3);
    assert_eq!(bitfield.c(), 42);

    // Manually reset the bitfield.
    bitfield.set_a(0);
    bitfield.set_b(0);
    bitfield.set_c(0);

    // Check if reset was successful.
    assert_eq!(bitfield.a(), 0);
    assert_eq!(bitfield.b(), 0);
    assert_eq!(bitfield.c(), 0);
}

// Tests bitfield specifiers of more than 64 bit.
#[test]
fn u128_specifier() {
    #[bitfield]
    pub struct SomeMoreBytes {
        a: B47,
        b: B65,
        c: B128,
    }

    let mut bitfield = SomeMoreBytes::new();

    // Everything is initialized to zero.
    assert_eq!(bitfield.a(), 0);
    assert_eq!(bitfield.b(), 0);
    assert_eq!(bitfield.c(), 0);

    // Manipulate bitfield.
    assert_eq!(bitfield.set_a_checked(1), Ok(()));
    assert_eq!(bitfield.set_b_checked(3), Ok(()));
    assert_eq!(bitfield.set_c_checked(42), Ok(()));

    // Check that manipulation was successful.
    assert_eq!(bitfield.a(), 1);
    assert_eq!(bitfield.b(), 3);
    assert_eq!(bitfield.c(), 42);

    // // Manually reset the bitfield.
    bitfield.set_a(0);
    bitfield.set_b(0);
    bitfield.set_c(0);

    // // Check if reset was successful.
    assert_eq!(bitfield.a(), 0);
    assert_eq!(bitfield.b(), 0);
    assert_eq!(bitfield.c(), 0);
}

// These tests check the conversions from and to bytes.
#[test]
fn byte_conversions() {
    #[bitfield]
    #[derive(PartialEq, Eq, Debug)]
    pub struct MyFourBytes {
        a: bool,
        b: B2,
        c: B13,
        d: B16,
    }

    let mut bitfield_1 = MyFourBytes::new();

    bitfield_1.set_a(true);
    bitfield_1.set_b(0b11);
    bitfield_1.set_c(444);
    bitfield_1.set_d(1337);

    assert!(bitfield_1.a());
    assert_eq!(bitfield_1.b(), 3);
    assert_eq!(bitfield_1.c(), 444);
    assert_eq!(bitfield_1.d(), 1337);

    let bytes = bitfield_1.into_bytes();
    assert_eq!(bytes, [231, 13, 57, 5]);

    let bitfield2 = MyFourBytes::from_bytes(bytes);

    assert!(bitfield2.a());
    assert_eq!(bitfield2.b(), 3);
    assert_eq!(bitfield2.c(), 444);
    assert_eq!(bitfield2.d(), 1337);
}

#[test]
fn within_single_byte() {
    #[derive(BitfieldSpecifier, Debug, PartialEq, Copy, Clone)]
    pub enum Mode {
        A = 0b00,
        B = 0b01,
        C = 0b10,
        D = 0b11,
    }

    #[bitfield]
    #[derive(Debug)]
    pub struct StatFlag {
        x: bool,
        y: bool,
        z: B2,
        #[bits = 2]
        mode: Mode,
        w: B2,
    }

    let mut flag = StatFlag::new();

    assert!(!flag.x());
    assert!(!flag.y());
    assert_eq!(flag.z(), 0);
    assert_eq!(flag.w(), 0);
    assert_eq!(flag.mode(), Mode::A);

    let new_mode = Mode::B;

    flag.set_mode(new_mode);
    assert!(!flag.x());
    assert!(!flag.y());
    assert_eq!(flag.z(), 0);
    assert_eq!(flag.w(), 0);
    assert_eq!(flag.mode(), new_mode);

    flag.set_x(true);
    assert!(flag.x());
    assert!(!flag.y());
    assert_eq!(flag.z(), 0);
    assert_eq!(flag.w(), 0);
    assert_eq!(flag.mode(), new_mode);

    flag.set_y(true);
    assert!(flag.x());
    assert!(flag.y());
    assert_eq!(flag.z(), 0);
    assert_eq!(flag.w(), 0);
    assert_eq!(flag.mode(), new_mode);

    flag.set_z(0b11);
    assert!(flag.x());
    assert!(flag.y());
    assert_eq!(flag.z(), 0b11);
    assert_eq!(flag.w(), 0);
    assert_eq!(flag.mode(), new_mode);

    flag.set_w(0b01);
    assert!(flag.x());
    assert!(flag.y());
    assert_eq!(flag.z(), 0b11);
    assert_eq!(flag.w(), 0b01);
    assert_eq!(flag.mode(), new_mode);
}

#[test]
fn get_spanning_data() {
    #[bitfield]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct ColorEntry {
        r: B5,
        g: B5,
        b: B5,
        unused: B1,
    }

    for i in 0..=u16::MAX {
        let entry = ColorEntry::from_bytes(i.to_le_bytes());
        let mut new = ColorEntry::new();
        new.set_r(entry.r());
        assert_eq!(new.r(), entry.r());
        new.set_g(entry.g());
        assert_eq!(new.g(), entry.g());
        new.set_b(entry.b());
        assert_eq!(new.b(), entry.b());
        new.set_unused(entry.unused());

        assert_eq!(new.r(), entry.r());
        assert_eq!(new.g(), entry.g());
        assert_eq!(new.b(), entry.b());
        assert_eq!(new.unused(), entry.unused());
    }
}

#[test]
fn raw_identifiers() {
    #[bitfield]
    struct RawIdentifiers {
        r#struct: B5,
        r#bool: B3,
    }

    let r = RawIdentifiers::new();
    let _ = r.r#struct();
    let _ = r.r#bool();
}

// Generate getters and.with() setters that manipulate the right range of bits
// corresponding to each field.
//
//
//     ║  first byte   ║  second byte  ║  third byte   ║  fourth byte  ║
//     ╟───────────────╫───────────────╫───────────────╫───────────────╢
//     ║▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒║
//     ╟─╫─────╫───────╫───────────────────────────────────────────────╢
//     ║a║  b  ║   c   ║                       d                       ║
#[test]
fn with_setter() {
    #[bitfield]
    pub struct MyFourBytes {
        a: bool,
        b: B3,
        c: B4,
        d: B24,
    }

    let bitfield = MyFourBytes::new()
        .with_a(true)
        .with_b(2)
        .with_c(14)
        .with_d(1_000_000);

    assert!(bitfield.a());
    assert_eq!(bitfield.b(), 2);
    assert_eq!(bitfield.c(), 14);
    assert_eq!(bitfield.d(), 1_000_000);
}

// Checks that no implicit paths are generated by the `#[bitfield]` proc. macro
// and `#[derive(BitfieldSpecifier)]` derive macro.
#[test]
fn primitives_as_specifiers() {
    #[bitfield]
    pub struct PrimitivesBitfield {
        a: bool,
        b: u8,
        c: u16,
        d: u32,
        e: u64,
        f: u128,
        rest: B7,
    }
}

// Validates that in a degenerate case with a single bit, non-power-of-two enums
// behave as expected.
#[test]
fn single_bit_enum() {
    use modular_bitfield::error::InvalidBitPattern;

    #[bitfield]
    pub struct UselessStruct {
        reserved: B3,
        field: ForciblyTrue,
        reserved2: B4,
    }

    #[derive(BitfieldSpecifier, Debug, PartialEq)]
    #[bits = 1]
    pub enum ForciblyTrue {
        True = 1,
    }

    assert_eq!(core::mem::size_of::<UselessStruct>(), 1);

    // Initialized to all 0 bits.
    let entry = UselessStruct::new();
    assert_eq!(
        entry.field_or_err(),
        Err(InvalidBitPattern { invalid_bytes: 0 })
    );

    let entry = UselessStruct::new().with_field(ForciblyTrue::True);
    assert_eq!(entry.field_or_err(), Ok(ForciblyTrue::True));
}
