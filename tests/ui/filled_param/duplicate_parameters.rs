use modular_bitfield::prelude::*;

// There are 2 duplicate `filled` parameters.
#[bitfield(filled = true, filled = true)]
pub struct Base {
    a: B2,
    b: B6,
    c: u8,
    d: u16,
}

fn main() {}
