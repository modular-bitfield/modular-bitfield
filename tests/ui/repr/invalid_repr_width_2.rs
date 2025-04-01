use modular_bitfield::prelude::*;

#[bitfield]
#[repr(u64)] // Too many bits!
pub struct SignedInt {
    sign: bool,
    value: B31,
}

fn main() {}
