use modular_bitfield::prelude::*;

#[bitfield(bits)]
pub struct SignInteger {
    sign: bool,
    value: B31,
}

fn main() {}
