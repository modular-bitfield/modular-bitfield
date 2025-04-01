use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Debug)]
pub struct Sparse {
    #[skip(getters, setters)]
    unused_1: B7,
    a: bool,
}

fn main() {
    let sparse = Sparse::from_bytes([0xFF; 1]);
    assert_eq!(sparse.unused_1(), 0xFE); // ERROR!
}
