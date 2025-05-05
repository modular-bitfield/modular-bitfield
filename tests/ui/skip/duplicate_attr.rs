use modular_bitfield::prelude::*;

#[bitfield]
pub struct TwoExplicitAll {
    #[skip]
    #[skip]
    unused_1: B15,
    a: bool,
}

#[bitfield]
pub struct TwoExplicitListAll {
    #[skip()]
    #[skip()]
    unused_1: B15,
    a: bool,
}

#[bitfield]
pub struct TwoExplicitGetters {
    #[skip(getters)]
    #[skip(getters)]
    unused_1: B15,
    a: bool,
}

#[bitfield]
pub struct TwoExplicitListGetters {
    #[skip(getters, getters)]
    unused_1: B15,
    a: bool,
}

#[bitfield]
pub struct ImplicitExplicitGetters {
    #[skip]
    #[skip(getters)]
    unused_1: B15,
    a: bool,
}

#[bitfield]
pub struct ExplicitImplicitGetters {
    #[skip(getters)]
    #[skip]
    unused_1: B15,
    a: bool,
}

#[bitfield]
pub struct TwoExplicitSetters {
    #[skip(setters)]
    #[skip(setters)]
    unused_1: B15,
    a: bool,
}

#[bitfield]
pub struct TwoExplicitListSetters {
    #[skip(setters, setters)]
    unused_1: B15,
    a: bool,
}

#[bitfield]
pub struct ImplicitExplicitSetters {
    #[skip]
    #[skip(setters)]
    unused_1: B15,
    a: bool,
}

#[bitfield]
pub struct ExplicitImplicitSetters {
    #[skip(setters)]
    #[skip]
    unused_1: B15,
    a: bool,
}

fn main() {}
