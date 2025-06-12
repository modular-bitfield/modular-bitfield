use modular_bitfield::prelude::*;

#[derive(Specifier)]
#[bits(8, 16)]
enum TestData {
    #[discriminant = 0]
    Small(u8),
    #[discriminant = 0]  // Duplicate discriminant
    Medium(u16),
}

fn main() {}