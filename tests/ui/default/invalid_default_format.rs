use modular_bitfield::prelude::*;

#[bitfield]
pub struct InvalidDefaultFormat {
    #[default]
    pub field: bool,
    pub other: B7,
}

fn main() {}