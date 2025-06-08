use modular_bitfield::prelude::*;

#[bitfield]
pub struct DefaultWithSkipSetters {
    #[default(42)]
    #[skip(setters)]
    pub field: B6,
    pub other: B2,
}

fn main() {}