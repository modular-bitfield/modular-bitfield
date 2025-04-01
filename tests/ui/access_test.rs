mod inner {
    use modular_bitfield::prelude::*;
    #[bitfield]
    #[derive(Copy, Clone, Eq, PartialEq, Default)]
    pub struct ColorEntry {
        a: B5,
        pub(crate) b: B3,
    }
}
use inner::*;

fn main() {
    let c = ColorEntry::new();
    let _ = c.a();
    // Notice no error for calling b
    let _ = c.b();
    // Also no error for using default
    let c = ColorEntry::default();
}
