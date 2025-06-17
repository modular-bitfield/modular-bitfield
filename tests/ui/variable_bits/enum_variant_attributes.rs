use modular_bitfield::prelude::*;

// Test: Explicit bits on enum variants
#[derive(Specifier)]
#[bits = (8, 16, 24)]
pub enum ExplicitBitsOnVariants {
    #[bits = 8]  // Should match position 0
    A(u8),
    #[bits = 16] // Should match position 1
    B(u16),
    #[bits = 24] // Should match position 2
    C(B24),
}

// Test: Conflicting explicit bits
#[derive(Specifier)]
#[bits = (8, 16, 24)]
pub enum ConflictingBits {
    #[bits = 8]  // OK
    A(u8),
    #[bits = 32] // Conflicts with tuple position 1 which specifies 16
    B(u16),
    C(B24),
}

// Test: bits attribute on unit variant
#[derive(Specifier)]
#[bits = (8, 16)]
pub enum BitsOnUnitVariant {
    #[bits = 8]
    A,  // Unit variants shouldn't have bits attribute
    B(u16),
}

// Test: Invalid bits attribute format
#[derive(Specifier)]
pub enum InvalidBitsFormat {
    #[bits = "8"]  // Should be numeric literal
    A(u8),
    #[bits(8)]     // Should be #[bits = N]
    B(u16),
}

// Test: discriminant on data variant without discriminant_bits
#[derive(Specifier)]
#[bits = (8, 16)]
pub enum DiscriminantWithoutDiscriminantBits {
    #[discriminant = 0]  // discriminant without discriminant_bits might not make sense
    A(u8),
    #[discriminant = 1]
    B(u16),
}

fn main() {}