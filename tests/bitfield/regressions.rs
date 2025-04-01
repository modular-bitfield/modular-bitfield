//! Tests for regressions found in published versions

use modular_bitfield::prelude::*;

#[test]
fn deny_elided_lifetime() {
    #[bitfield]
    #[derive(Debug)]
    #[repr(u8)]
    struct KeyFlags {
        certify: bool,
        fill: B7,
    }
}

#[test]
fn regression_issue_8() {
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
        z: B4,
        #[bits = 2]
        mode: Mode,
    }

    let mut flag = StatFlag::new();

    assert!(!flag.x());
    assert!(!flag.y());
    assert_eq!(flag.z(), 0);
    assert_eq!(flag.mode(), Mode::A);

    let new_mode = Mode::B;

    flag.set_mode(new_mode);
    assert_eq!(flag.mode(), new_mode);

    flag.set_x(true);
    assert!(flag.x());
    assert_eq!(flag.mode(), new_mode);

    flag.set_y(true);
    assert!(flag.y());
    assert_eq!(flag.mode(), new_mode);

    flag.set_z(0b01);
    assert_eq!(flag.z(), 0b01);
    assert_eq!(flag.mode(), new_mode);

    flag.set_z(0b11);
    assert_eq!(flag.z(), 0b11);
    assert_eq!(flag.mode(), new_mode);
}

#[test]
fn regression_v0_11() {
    #[bitfield(bits = 8)]
    #[derive(Copy, Clone)]
    pub struct Flags {
        pub a: bool,
        #[skip]
        __: bool,
        pub b: bool,
        #[skip]
        __: B5,
    }

    let mut flags = Flags::new();
    assert!(!flags.a());
    assert!(!flags.b());
    assert_eq!(flags.into_bytes(), [0b0000_0000]);
    flags.set_a(true);
    flags.set_b(true);
    assert!(flags.a());
    assert!(flags.b());
    assert_eq!(flags.into_bytes(), [0b0000_0101]);
}
