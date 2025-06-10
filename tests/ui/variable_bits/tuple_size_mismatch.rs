use modular_bitfield::prelude::*;

#[derive(Specifier)]
#[variable_bits(8, 16)]  // Only 2 sizes but 3 variants
enum TestData {
    #[discriminant = 0]
    Small(u8),
    #[discriminant = 1]
    Medium(u16),
    #[discriminant = 2]
    Large(u32),
}

fn main() {}