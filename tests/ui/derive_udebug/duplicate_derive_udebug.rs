use modular_bitfield::prelude::*;

#[bitfield]
#[derive(uDebug)] #[derive(uDebug)]
pub struct SignedInt {
    sign: bool,
    value: B31,
}

fn main() {}
