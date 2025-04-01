use modular_bitfield::prelude::*;

#[bitfield(bits = 32, bits = 16)]
pub struct SignInteger {
    sign: bool,
    value: B31,
}

fn main() {}
