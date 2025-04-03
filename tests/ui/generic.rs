use modular_bitfield::prelude::*;

#[bitfield]
struct Generic<T> {
    t: core::marker::PhantomData<T>,
}

fn main() {}
