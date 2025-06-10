use modular_bitfield::prelude::*;

#[derive(Specifier)]
#[variable_bits = 42]  // Invalid: should be list format
enum TestData {
    #[discriminant = 0]
    Small(u8),
    #[discriminant = 1]
    Large(u16),
}

fn main() {}