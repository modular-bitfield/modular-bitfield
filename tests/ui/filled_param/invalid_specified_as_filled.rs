use modular_bitfield::prelude::*;

// The bitfield has exactly 15 bits and therefore is unfilled but not specified as such.
#[bitfield(filled = true)]
pub struct UnfilledBitfield {
    a: B7,
    b: u8,
}

fn main() {}
