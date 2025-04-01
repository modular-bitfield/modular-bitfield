use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier)]
pub union InvalidUnionSpecifier {
    a: bool,
    b: B7,
    c: u8,
}

fn main() {}
