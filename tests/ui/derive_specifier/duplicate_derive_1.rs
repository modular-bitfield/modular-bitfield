use modular_bitfield::prelude::*;

#[bitfield]
#[derive(BitfieldSpecifier, BitfieldSpecifier)]
pub struct SignedInt {
    sign: bool,
    value: B31,
}

fn main() {}
