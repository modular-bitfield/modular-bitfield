#![deny(deprecated)]

use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier)]
enum Foo {
    Zero = 0,
    One = 1,
}

#[bitfield]
#[derive(BitfieldSpecifier)]
struct Bar {
    f: u8,
}

#[bitfield]
#[derive(BitfieldSpecifier)]
struct Baz {
    f: u8,
}

fn main() {}
