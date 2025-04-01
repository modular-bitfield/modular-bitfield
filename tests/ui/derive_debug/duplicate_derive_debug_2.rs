use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Debug, Debug)]
pub struct SignedInt {
    sign: bool,
    value: B31,
}

fn main() {}
