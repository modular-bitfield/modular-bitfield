#![deny(unused_must_use)]

use modular_bitfield::prelude::*;

#[bitfield]
struct Foo {
    a: B8,
}

fn main() {
    Foo::new();
    Foo::new().with_a(0);
    Foo::new().a();
    Foo::from_bytes([0]);
}
