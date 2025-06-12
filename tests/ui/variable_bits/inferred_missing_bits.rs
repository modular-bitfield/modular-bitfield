use modular_bitfield::prelude::*;

#[derive(Specifier)]
// Missing #[bits] attribute entirely - should fail
enum TestData {
    #[discriminant = 0]
    Small(u8),
    #[discriminant = 1]
    Medium(u16),
}

fn main() {}