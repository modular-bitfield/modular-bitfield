use modular_bitfield::prelude::*;

#[derive(Specifier)]
#[discriminant_bits = 2]  // Only 2 bits allows values 0-3
#[bits = (8, 16, 24, 32)]  // 4 variants means discriminant 3 is max
pub enum ValidEnum {
    A(u8),
    B(u16),
    C(B24),
    D(u32),  // This has implicit discriminant 3, which fits in 2 bits
}

#[derive(Specifier)]
#[discriminant_bits = 2]  // Only 2 bits allows values 0-3
#[bits = (8, 16, 24, 32, 40)]  // 5 variants means discriminant 4 is needed
pub enum TooManyVariants {
    A(u8),
    B(u16),
    C(B24),
    D(u32),
    E(B40),  // This has implicit discriminant 4, which doesn't fit in 2 bits
}

#[derive(Specifier)]
#[discriminant_bits = 3]  // 3 bits allows values 0-7
#[bits = (8, 16)]
pub enum ExplicitDiscriminantTooLarge {
    A(u8),
    #[discriminant = 8]  // 8 doesn't fit in 3 bits
    B(u16),
}

#[derive(Specifier)]
#[discriminant_bits = 1]  // Only 1 bit allows values 0-1
#[bits = (8, 16, 24)]
pub enum MixedDiscriminants {
    A(u8),  // implicit discriminant 0
    #[discriminant = 1]
    B(u16),  // explicit discriminant 1
    C(B24),  // implicit discriminant 2, doesn't fit in 1 bit
}

fn main() {}