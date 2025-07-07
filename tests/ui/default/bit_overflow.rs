use modular_bitfield::prelude::*;

#[bitfield]
pub struct BitOverflowTest {
    #[default = -1]    // Negative value should fail
    invalid: B8,
}

fn main() {}