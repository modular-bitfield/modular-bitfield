use modular_bitfield::prelude::*;

#[derive_const(Specifier)]
pub struct InvalidStructSpecifier {
    a: bool,
    b: B7,
    c: u8,
}

fn main() {}
