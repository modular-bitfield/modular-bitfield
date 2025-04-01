//! Tests for `#[derive(BitfieldSpecifier)]` using `#[bitfield]`

use modular_bitfield::prelude::*;

#[test]
fn struct_in_struct() {
    #[bitfield(filled = false)]
    #[derive(BitfieldSpecifier, Debug, PartialEq, Eq, Copy, Clone)]
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
    #[derive(BitfieldSpecifier, Debug, PartialEq, Eq, Copy, Clone)]
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
    #[derive(BitfieldSpecifier)]
    pub struct Header {
        live: bool,
        received: bool,
        status: B2,
        rest: B4,
    }

    assert_eq!(<Header as Specifier>::BITS, 8);
}
