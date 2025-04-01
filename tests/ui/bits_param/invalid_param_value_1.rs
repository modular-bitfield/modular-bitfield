use modular_bitfield::prelude::*;

#[bitfield(bits = true)]
pub struct SignInteger {
    sign: bool,
    value: B31,
}

fn main() {}
