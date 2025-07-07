use modular_bitfield::prelude::*;

#[bitfield]
pub struct DuplicateDefault {
    #[default = true]
    #[default = false]
    pub field: bool,
}

fn main() {}