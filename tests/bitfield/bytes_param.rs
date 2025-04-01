//! Tests for `bytes = N` #[bitfield] parameter

use modular_bitfield::prelude::*;

#[test]
fn valid_bitfield() {
    // Just requires exactly 32 bits (4 bytes) as expected.
    #[bitfield(bytes = 4)]
    pub struct Base {
        a: B2,
        b: B6,
        c: u8,
        d: u16,
    }

    assert_eq!(core::mem::size_of::<Base>(), 4)
}

#[test]
fn valid_specifier_bitfield() {
    // Is only 9 bits, so will be 2 bytes in size.
    #[bitfield(bytes = 2, filled = false)]
    #[derive(BitfieldSpecifier)]
    pub struct Header {
        a: B6,
        b: bool,
        c: bool,
        d: bool,
    }

    assert_eq!(core::mem::size_of::<Header>(), 2)
}
