use modular_bitfield::prelude::*;

#[bitfield(skip(new, new))]
struct A {
    f: u8,
}

#[bitfield(skip(from_bytes, new, from_bytes))]
struct B {
    f: u8,
}

#[bitfield(skip(new, into_bytes, into_bytes))]
struct C {
    f: u8,
}

#[bitfield(skip(invalid))]
struct D {
    f: u8,
}

#[bitfield(skip)]
struct E {
    f: u8,
}

#[bitfield(skip(all, convert))]
struct F {
    f: u8,
}

#[bitfield(skip(all, new))]
struct G {
    f: u8,
}

#[bitfield(skip(convert, from_bytes, into_bytes))]
struct H {
    f: u8,
}

#[bitfield]
struct I {
    _implicit_skip: u8,
}

fn main() {
    let i = I::new();
    i.implicit_skip();
    i._implicit_skip();
}
