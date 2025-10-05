use modular_bitfield::prelude::*;

#[derive_const(Specifier)]
#[bits = 1]
#[bits = 1]
enum TooManyAttrs {
    Zero = 0,
    One = 1,
}

#[derive_const(Specifier)]
#[bits = 1.0]
enum NotAnInt {
    Zero = 0,
    One = 1,
}

fn main() {}
