use modular_bitfield::prelude::*;

#[bitfield]
pub struct OverflowTest {
    #[default(256)]  // Too large for B8 (max 255)
    overflow_b8: B8,
}

fn main() {}