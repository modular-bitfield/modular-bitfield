use modular_bitfield::prelude::*;

#[bitfield]
pub struct InvalidSpecifier {
    #[skip(invalid_specifier)]
    unused_1: B10,
    a: bool,
    #[skip]
    unused_2: B10,
    b: bool,
    #[skip]
    unused_3: B10,
}

#[bitfield]
pub struct InvalidFormat {
    #[skip = 1]
    unused_1: B10,
    a: bool,
    #[skip]
    unused_2: B10,
    b: bool,
    #[skip]
    unused_3: B10,
}

fn main() {}
