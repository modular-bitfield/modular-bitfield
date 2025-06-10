use modular_bitfield::prelude::*;

#[derive(Specifier)]
#[variable_bits]  // Inferred but missing #[bits] on data variant
enum TestData {
    #[discriminant = 0]
    #[bits = 8]
    Small(u8),
    #[discriminant = 1]
    // Missing #[bits = N] for inferred mode
    Medium(u16),
}

fn main() {}