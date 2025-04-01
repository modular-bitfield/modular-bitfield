use modular_bitfield::prelude::*;

#[bitfield]
#[repr(C, transparent, u32)] // The macro simply ignores `repr(C)` and `repr(transparent)`
pub struct SignedInt {
    sign: bool,
    value: B31,
}

fn main() {}
