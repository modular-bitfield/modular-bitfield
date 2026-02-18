use modular_bitfield::prelude::*;

fn get_value() -> u8 {
    42
}

#[bitfield]
pub struct NonConstDefault {
    #[default(get_value())]  // Function call not allowed in const context
    field: B8,
}

fn main() {}