use modular_bitfield::prelude::*;

#[bitfield]
pub struct DefaultWithSkip {
    #[default = true]
    #[skip]
    pub field: bool,
    pub other: B7,
}

fn main() {}