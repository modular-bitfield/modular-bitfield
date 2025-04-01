use modular_bitfield::prelude::*;

// The integer value cannot be parsed into a `usize` since it is negative.
#[bitfield(bytes = -1)]
pub struct Base {
    a: B2,
    b: B6,
    c: u8,
    d: u32,
}

fn main() {}
