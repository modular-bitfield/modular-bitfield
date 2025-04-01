use modular_bitfield::prelude::*;

// The bytes parameter is required to have an integer value.
#[bitfield(bytes = true)]
pub struct Base {
    a: B2,
    b: B6,
    c: u8,
    d: u32,
}

fn main() {}
