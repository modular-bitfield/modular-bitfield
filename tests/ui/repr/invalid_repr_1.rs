use modular_bitfield::prelude::*;

#[bitfield]
#[repr(invalid)]
pub struct SignedInt {
    sign: bool,
    value: B31,
}

fn main() {}
