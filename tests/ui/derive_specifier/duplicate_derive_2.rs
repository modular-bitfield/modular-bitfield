use modular_bitfield::prelude::*;

#[bitfield]
#[derive_const(Specifier)] #[derive_const(Specifier)]
pub struct SignedInt {
    sign: bool,
    value: B31,
}

fn main() {}
