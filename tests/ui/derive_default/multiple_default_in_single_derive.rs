use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Default, Default)]
pub struct MultipleDefaultInDerive {
    flag: bool,
    value: B7,
}

fn main() {}