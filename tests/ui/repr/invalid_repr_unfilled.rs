use modular_bitfield::prelude::*;

#[bitfield(filled = false)]
#[repr(u32)]
pub struct SignedInt {
    sign: bool,
    value: B30,
}

fn main() {}
