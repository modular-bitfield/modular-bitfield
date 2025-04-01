use modular_bitfield::prelude::*;

#[bitfield(bits = 32, bits = 32)]
pub struct SignInteger {
    sign: bool,
    value: B31,
}

fn main() {}
