use modular_bitfield::prelude::*;

#[bitfield]
#[derive(uDebug, uDebug)]
pub struct SignedInt {
    sign: bool,
    value: B31,
}

fn main() {}
