#![allow(dead_code)]
#![feature(const_trait_impl)]
#![feature(const_try)]
#![feature(derive_const)]
#![feature(const_index)]

use modular_bitfield::prelude::*;

#[bitfield]
pub struct EdgeCaseBytes {
    a: B9,
    b: B6,
    c: B13,
    d: B4,
}

#[test]
#[should_panic(expected = "value out of bounds for field EdgeCaseBytes.a")]
fn invalid_access_a() {
    let mut bytes = EdgeCaseBytes::new();
    bytes.set_a(0b0010_0000_0000_u16);
}

#[test]
#[should_panic(expected = "value out of bounds for field EdgeCaseBytes.b")]
fn invalid_access_b() {
    let mut bytes = EdgeCaseBytes::new();
    bytes.set_b(0b0000_0100_0000_u8);
}

#[test]
#[should_panic(expected = "value out of bounds for field EdgeCaseBytes.c")]
fn invalid_access_c() {
    let mut bytes = EdgeCaseBytes::new();
    bytes.set_c(0x2000_u16);
}

#[test]
#[should_panic(expected = "value out of bounds for field EdgeCaseBytes.d")]
fn invalid_access_d() {
    let mut bytes = EdgeCaseBytes::new();
    bytes.set_d(0b0001_0000_u8);
}
