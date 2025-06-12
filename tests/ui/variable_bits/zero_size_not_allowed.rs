use modular_bitfield::prelude::*;

#[derive(Specifier)]
#[bits(0, 8)]  // Zero size not allowed
enum TestData {
    #[discriminant = 0]
    Empty,
    #[discriminant = 1]
    Small(u8),
}

fn main() {}