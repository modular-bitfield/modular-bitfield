use modular_bitfield::prelude::*;

#[bitfield(filled = false)]
#[derive_const(Specifier)]
#[derive( Debug)]
pub struct Header {
    a: B1,
    b: B128,
}

fn main() {}
