use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Debug)]
pub struct Sparse {
    #[skip(setters)]
    unused_1: B7,
    a: bool,
}

fn main() {
    let sparse = Sparse::new();
    assert_eq!(sparse.unused_1(), 0);
    sparse.set_unused_1(0b11_1111_1111); // ERROR!
}
