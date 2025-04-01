//! Tests for `filled: bool` #[bitfield] parameter

use modular_bitfield::prelude::*;

#[test]
fn valid_bitfield_1() {
    // The bitfield has only 7 bits and therefore is unfilled.
    #[bitfield(filled = false)]
    pub struct UnfilledBitfield {
        a: B7,
    }
}

#[test]
fn valid_bitfield_2() {
    // The bitfield has exactly 8 bits and therefore is filled.
    #[bitfield(filled = true)]
    pub struct UnfilledBitfield {
        a: B8,
    }
}

#[test]
fn valid_bitfield_specifier_1() {
    // The bitfield only has 23 bits and therefore is unfilled.
    #[bitfield(filled = false)]
    #[derive(BitfieldSpecifier)]
    pub struct UnfilledSpecifier {
        a: B7,
        b: u16,
    }
}

#[test]
fn valid_bitfield_specifier_2() {
    // The bitfield has 24 bits and therefore is filled.
    #[bitfield(filled = true)]
    #[derive(BitfieldSpecifier)]
    pub struct FilledSpecifier {
        a: B8,
        b: u16,
    }
}
