use modular_bitfield::prelude::*;

#[bitfield(bits)]
pub struct Missing {
    sign: bool,
    value: B31,
}

#[bitfield(bits(32))]
pub struct WrongMetaType {
    sign: bool,
    value: B31,
}

fn main() {}
