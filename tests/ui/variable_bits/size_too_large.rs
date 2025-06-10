use modular_bitfield::prelude::*;

#[derive(Specifier)]
#[variable_bits(8, 256)]  // 256 > 128 bits max
enum TestData {
    #[discriminant = 0]
    Small(u8),
    #[discriminant = 1]
    TooLarge,
}

fn main() {}