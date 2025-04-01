use modular_bitfield::prelude::*;

// The bitfield has exactly 16 bits and therefore is filled but not specified as such.
#[bitfield(filled = false)]
pub struct UnfilledBitfield {
    a: B8,
    b: u8,
}

fn main() {}
