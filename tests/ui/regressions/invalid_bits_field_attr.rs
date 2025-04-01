
use modular_bitfield::prelude::*;

#[bitfield]
pub struct SignInt {
    #[bits = 1] // This one is valid.
    sign: bool,
    #[whoat_bits = 31] // Should error!
    value: B31,
}

fn main() {}
