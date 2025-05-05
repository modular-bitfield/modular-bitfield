use modular_bitfield::prelude::*;

const NOT_LITERAL: u32 = 32;

#[bitfield(bits = NOT_LITERAL)]
pub struct SignInteger {
    sign: bool,
    value: B31,
}

fn main() {}
