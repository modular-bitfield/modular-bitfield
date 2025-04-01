//! In this benchmark we compare the macro generated code for
//! setters and getters to some hand-written code for the same
//! data structure.
//!
//! We do a performance analysis for the getter and setter of
//! all fields of both structs.
//!
//! Also we test here that our hand-written code and the macro
//! generated code actually are semantically equivalent.
//! This allows us to further enhance the hand-written code
//! and to eventually come up with new optimization tricks
//! for the macro generated code while staying correct.

use modular_bitfield::prelude::*;

/// This generates code by the macros that we are going to test.
///
/// For every field a getter `get_*` and a setter `set_*` is generated
/// where `*` is the name of the field.
///
/// Note that this tests the following cases:
///
/// - `a`: Spans 2 bytes where the first byte is used fully and the
///        second byte stores only one of its bits.
/// - `b`: Fits into one byte but doesn't reach the bounds on either side.
/// - `c`: Spans across 3 bytes in total and uses only 1 bit and 4 bits in
///        the respective first and last byte.
/// - `d`: Spans 3 whole bytes in total.
///
/// More cases could be missing and might be added in the future.
#[bitfield]
pub struct Generated {
    pub a: B9,
    pub b: B6,
    pub c: B13,
    pub d: B1,
    pub e: B3,
    pub f: B32,
}

/// This is the hand-written part that the macro generated getters
/// and setters are compared against.
///
/// We try to encode the handwritten setters and getters as good as
/// we can while trying to stay within reasonable bounds of readability.
///
/// This code should perform as good as the macro generated code and vice versa.
pub struct Handwritten {
    data: [u8; 8],
}

impl Handwritten {
    /// Creates a new hand-written struct initialized with all zeros.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { data: [0x00; 8] }
    }

    /// Returns the value of `a`.
    pub fn a(&self) -> u16 {
        u16::from_le_bytes([self.data[0], self.data[1] & 0x01])
    }

    /// Sets the value of `a`.
    pub fn set_a(&mut self, new_val: u16) {
        assert!(new_val < (0x01 << 9));
        let [ls, ms] = new_val.to_le_bytes();
        self.data[0] = ls;
        self.data[1] = (self.data[1] & (!0x01)) | (ms & 0x01);
    }

    /// Returns the value of `b`.
    pub fn b(&self) -> u8 {
        (self.data[1] >> 1) & 0b0011_1111
    }

    /// Sets the value of `b`.
    pub fn set_b(&mut self, new_val: u8) {
        assert!(new_val < (0x01 << 6));
        self.data[1] = (self.data[1] & 0x81) | (new_val << 1);
    }

    /// Returns the value of `c`.
    pub fn c(&self) -> u16 {
        let mut res = 0;
        res |= (self.data[1] >> 7) as u16;
        res |= (self.data[2] as u16) << 1;
        res |= ((self.data[3] & 0b0000_1111) as u16) << 9;
        res
    }

    /// Sets the value of `c`.
    pub fn set_c(&mut self, new_val: u16) {
        assert!(new_val < (0x01 << 13));
        self.data[1] = (self.data[1] & !0x80) | (((new_val & 0x01) << 7) as u8);
        self.data[2] = ((new_val >> 1) & 0xFF) as u8;
        self.data[3] = (self.data[3] & !0x0F) | (((new_val >> 9) & 0x0F) as u8);
    }

    /// Returns the value of `d`.
    pub fn d(&self) -> u8 {
        (self.data[3] >> 4) & 0b0000_0001
    }

    /// Sets the value of `d`.
    pub fn set_d(&mut self, new_val: u8) {
        self.data[3] = (self.data[3] & (!0b0001_0000)) | ((new_val & 0b0000_0001) << 4)
    }

    /// Returns the value of `e`.
    pub fn e(&self) -> u8 {
        (self.data[3] >> 5) & 0b0000_0111
    }

    /// Sets the value of `e`.
    pub fn set_e(&mut self, new_val: u8) {
        assert!(new_val < (0x01 << 3));
        self.data[3] = (self.data[3] & !0b1110_0000) | ((new_val & 0b0000_0111) << 5)
    }

    /// Returns the value of `f`.
    pub fn f(&self) -> u32 {
        u32::from_le_bytes([self.data[4], self.data[5], self.data[6], self.data[7]])
    }

    /// Sets the value of `e`.
    pub fn set_f(&mut self, new_val: u32) {
        assert!((new_val as u64) < (0x01_u64 << 32));
        let le_bytes = new_val.to_le_bytes();
        self.data[4..].copy_from_slice(&le_bytes[..]);
    }
}

macro_rules! impl_getter_setter_tests {
    ( $( ($name:ident, $getter:ident, $setter:ident, $n:expr), )* ) => {
        #[cfg(test)]
        mod generated_is_equal_to_handwritten {
            $(
                #[test]
                fn $name() {
                    let mut macro_struct = super::Generated::new();
                    let mut hand_struct = super::Handwritten::new();
                    assert_eq!(hand_struct.$getter(), macro_struct.$getter());
                    macro_struct.$setter($n);
                    hand_struct.$setter($n);
                    assert_eq!(hand_struct.$getter(), $n);
                    assert_eq!(macro_struct.$getter(), $n);
                    macro_struct.$setter(0);
                    hand_struct.$setter(0);
                    assert_eq!(hand_struct.$getter(), 0);
                    assert_eq!(macro_struct.$getter(), 0);
                }
            )*
        }
    }
}
impl_getter_setter_tests!(
    (get_set_a, a, set_a, 0b0001_1111_1111),
    (get_set_b, b, set_b, 0b0011_1111),
    (get_set_c, c, set_c, 0b0001_1111_1111_1111),
    (get_set_d, d, set_d, 0b0001),
    (get_set_e, e, set_e, 0b0111),
    (get_set_f, f, set_f, u32::MAX),
);
