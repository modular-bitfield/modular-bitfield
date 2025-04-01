use modular_bitfield::prelude::*;

// Requires 6 bytes in total instead of 4.
#[bitfield(bytes = 4)]
pub struct Base {
    a: B2,
    b: B6,
    c: u8,
    d: u32,
}

fn main() {
    assert_eq!(core::mem::size_of::<Base>(), 6)
}
