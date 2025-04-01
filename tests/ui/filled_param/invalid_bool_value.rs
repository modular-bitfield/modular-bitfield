use modular_bitfield::prelude::*;

// The boolean value cannot be parsed from a string.
#[bitfield(filled = "yes")]
pub struct Base {
    a: B2,
    b: B6,
    c: u8,
    d: u32,
}

fn main() {}
