use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier)]
pub struct InvalidStructSpecifier {
    a: bool,
    b: B7,
    c: u8,
}

fn main() {}
