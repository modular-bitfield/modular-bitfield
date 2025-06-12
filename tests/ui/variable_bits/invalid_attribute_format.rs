use modular_bitfield::prelude::*;

#[derive(Specifier)]
#[bits = hello]  // Invalid: should be an integer
enum TestData {
    #[discriminant = 0]
    Small(u8),
    #[discriminant = 1]
    Large(u16),
}

fn main() {}