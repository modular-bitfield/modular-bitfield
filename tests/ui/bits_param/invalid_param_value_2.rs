use modular_bitfield::prelude::*;

#[bitfield(bits = -1)]
pub struct SignInteger {
    sign: bool,
    value: B31,
}

fn main() {}
