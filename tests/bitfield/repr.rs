//! Tests for `#[repr(uN)]` and `#[cfg_attr(cond, repr(uN))]`

use modular_bitfield::prelude::*;

#[test]
fn complex_use() {
    #[bitfield]
    #[repr(u32)]
    #[derive(Debug, PartialEq, Eq)]
    struct TtResp {
        mregion: u8,
        sregion: u8,
        mrvalid: bool,
        srvalid: bool,
        r: bool,
        rw: bool,
        nsr: bool,
        nsrw: bool,
        s: bool,
        irvalid: bool,
        iregion: u8,
    }

    let mut rsp = TtResp::new();
    rsp.set_mregion(u8::MAX);
    rsp.set_sregion(0xEE);
    rsp.set_mrvalid(true);
    rsp.set_srvalid(true);
    rsp.set_r(true);
    rsp.set_rw(true);
    rsp.set_nsr(true);
    rsp.set_nsrw(true);
    rsp.set_s(true);
    rsp.set_irvalid(true);
    rsp.set_iregion(0xDD);
    assert_eq!(rsp, TtResp::from(0xDDFF_EEFF_u32));
    assert_eq!(rsp.mregion(), u8::MAX);
    assert_eq!(rsp.sregion(), 0xEE);
    assert!(rsp.mrvalid());
    assert!(rsp.srvalid());
    assert!(rsp.r());
    assert!(rsp.rw());
    assert!(rsp.nsr());
    assert!(rsp.nsrw());
    assert!(rsp.s());
    assert!(rsp.irvalid());
    assert_eq!(rsp.iregion(), 0xDD);
    rsp.set_mregion(0);
    rsp.set_sregion(0);
    rsp.set_mrvalid(false);
    rsp.set_srvalid(false);
    rsp.set_r(false);
    rsp.set_rw(false);
    rsp.set_nsr(false);
    rsp.set_nsrw(false);
    rsp.set_s(false);
    rsp.set_irvalid(false);
    rsp.set_iregion(0x00);
    assert_eq!(rsp, TtResp::new());
}

#[test]
fn generic_repr() {
    #[bitfield]
    #[repr(u8)]
    struct Generic<const N: u8> {
        v: u8,
    }
}

#[test]
fn multiple_valid_reprs_1() {
    #[bitfield]
    #[repr(C, u32)] // The macro simply ignores `repr(C)`
    pub struct SignedInt {
        sign: bool,
        value: B31,
    }
}

#[test]
fn multiple_valid_reprs_2() {
    #[bitfield]
    #[repr(u32)]
    #[repr(C)] // The macro simply ignores `repr(C)`
    pub struct SignedInt {
        sign: bool,
        value: B31,
    }
}

#[test]
fn valid_cond_use() {
    #[bitfield]
    #[cfg_attr(all(), repr(u32))]
    #[derive(Debug, PartialEq, Eq)]
    pub struct SignedInt {
        sign: bool,
        value: B31,
    }

    let i1 = SignedInt::new().with_sign(true).with_value(0b1001_0011);
    let i2 = SignedInt::from(0b0000_0000_0000_0000_0000_0001_0010_0111_u32);
    assert_eq!(i1, i2);
    assert_eq!(i1.sign(), i2.sign());
    assert_eq!(i1.value(), i2.value());
}

macro_rules! valid_use {
    ($(($name:ident, $repr_ty:ty, $value_ty:ty)),*$(,)?) => {
        $(#[test]
        fn $name() {
            #[bitfield]
            #[repr($repr_ty)]
            #[derive(Debug, PartialEq, Eq)]
            pub struct SignedInt {
                value: $value_ty,
                sign: bool,
            }

            let value = 0b101_0011;
            let bits = (1 << (<$repr_ty>::BITS - 1)) | value;

            let i1 = SignedInt::new().with_sign(true).with_value(value);
            let i2 = SignedInt::from(bits);
            assert_eq!(i1, i2);
            assert_eq!(i1.sign(), i2.sign());
            assert_eq!(i1.value(), i2.value());
        })*
    }
}

valid_use!(
    (valid_use_u8, u8, B7),
    (valid_use_u16, u16, B15),
    (valid_use_u32, u32, B31),
    (valid_use_u64, u64, B63),
    (valid_use_u128, u128, B127)
);
