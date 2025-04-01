//! Tests for `#[bitfield(bits = N)]`

use modular_bitfield::prelude::*;

#[test]
fn bits_non_filled_1() {
    #[bitfield(bits = 32, filled = false)]
    #[derive(BitfieldSpecifier)]
    pub struct SignIntegerShort {
        sign: bool,
        value: B7,
    }

    assert_eq!(<SignIntegerShort as Specifier>::BITS, 32);
}

#[test]
fn bits_non_filled_2() {
    #[bitfield(bits = 32, filled = false)]
    #[derive(BitfieldSpecifier)]
    pub struct SignIntegerLong {
        sign: bool,
        value: B30,
    }

    assert_eq!(<SignIntegerLong as Specifier>::BITS, 32);
}

#[test]
fn complex_use_case() {
    #[derive(BitfieldSpecifier)]
    #[bits = 2]
    pub enum Status {
        Red,
        Green,
        Yellow,
    }

    #[bitfield(bits = 4)]
    #[derive(BitfieldSpecifier)]
    pub struct Header {
        is_compact: bool,
        is_secure: bool,
        #[bits = 2]
        pre_status: Status,
    }

    #[bitfield(bits = 16, bytes = 2, filled = false)]
    #[derive(BitfieldSpecifier)]
    pub struct PackedData {
        #[bits = 4]
        header: Header,
        body: B9,
        #[bits = 2]
        status: Status,
    }

    assert_eq!(<Status as Specifier>::BITS, 2);
    assert_eq!(<Header as Specifier>::BITS, 4);
    assert_eq!(<PackedData as Specifier>::BITS, 16);
}

#[test]
fn low_bits_filled() {
    #[bitfield(bits = 4)]
    #[derive(BitfieldSpecifier)]
    pub struct Header {
        is_compact: bool,
        is_secure: bool,
        #[bits = 2]
        pre_status: B2,
    }

    assert_eq!(<Header as Specifier>::BITS, 4);
}

#[test]
fn valid_use_1() {
    #[bitfield(bits = 32)]
    pub struct SignInteger {
        sign: bool,
        value: B31,
    }
}

#[test]
fn valid_use_2() {
    #[bitfield(bits = 32)]
    #[repr(u32)]
    pub struct SignInteger {
        sign: bool,
        value: B31,
    }
}

#[test]
fn valid_use_3() {
    #[bitfield(bits = 32, bytes = 4)]
    #[repr(u32)]
    pub struct SignInteger {
        sign: bool,
        value: B31,
    }
}

#[test]
fn valid_use_4() {
    #[bitfield(bits = 32, bytes = 4)]
    #[repr(u32)]
    #[derive(Debug, BitfieldSpecifier)]
    pub struct SignInteger {
        sign: bool,
        value: B31,
    }
}
