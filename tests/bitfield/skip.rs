//! Tests for `#[skip(..)]`

extern crate alloc;
use alloc::format;
use modular_bitfield::prelude::*;

#[test]
fn double_wildcards_1() {
    #[bitfield]
    pub struct Sparse {
        #[skip]
        __: B10,
        a: bool,
        #[skip]
        __: B10,
        b: bool,
        #[skip]
        __: B10,
    }
}

#[test]
fn double_wildcards_2() {
    #[bitfield]
    pub struct Sparse {
        #[skip(getters, setters)]
        __: B10,
        a: bool,
        #[skip(getters, setters)]
        __: B10,
        b: bool,
        #[skip(getters, setters)]
        __: B10,
    }
}

#[deny(dead_code)]
#[test]
fn no_dead_code() {
    #[bitfield(skip(from_bytes, into_bytes))]
    struct A {
        #[skip(setters)]
        f: u8,
    }

    #[bitfield(skip(new, into_bytes))]
    struct B {
        #[skip(setters)]
        f: u8,
    }

    #[bitfield(skip(convert))]
    #[derive(Specifier)]
    struct C {
        #[skip(setters)]
        f: u8,
    }

    #[bitfield(skip(all))]
    #[repr(u8)]
    struct D {
        f: u8,
    }

    #[bitfield(skip(from_bytes, into_bytes))]
    struct E {
        f: u8,
        _g: u8,
    }

    let _ = A::new().f();
    let _ = B::from_bytes([0u8; 1]).f();
    let _ = C::new().f();
    let _ = D::from(0).f();
    let _ = E::new().f();
}

#[test]
fn skip_default() {
    #[bitfield]
    pub struct Sparse {
        #[skip]
        unused_1: B10,
        a: bool,
        #[skip]
        unused_2: B10,
        b: bool,
        #[skip]
        unused_3: B10,
    }

    let mut sparse = Sparse::new();
    assert!(!sparse.a());
    assert!(!sparse.b());
    sparse.set_a(true);
    sparse.set_b(true);
    assert!(sparse.a());
    assert!(sparse.b());
}

#[test]
fn skip_getters_and_setters_1() {
    #[bitfield]
    pub struct Sparse {
        #[skip(getters, setters)]
        unused_1: B10,
        a: bool,
        #[skip(getters, setters)]
        unused_2: B10,
        b: bool,
        #[skip(getters, setters)]
        unused_3: B10,
    }

    let mut sparse = Sparse::new();
    assert!(!sparse.a());
    assert!(!sparse.b());
    sparse.set_a(true);
    sparse.set_b(true);
    assert!(sparse.a());
    assert!(sparse.b());
}

#[test]
fn skip_getters_and_setters_2() {
    #[bitfield]
    pub struct Sparse {
        #[skip(getters)]
        #[skip(setters)]
        unused_1: B10,
        a: bool,
        #[skip(setters)]
        #[skip(getters)]
        unused_2: B10,
        b: bool,
        #[skip(getters)]
        #[skip(setters)]
        unused_3: B10,
    }

    let mut sparse = Sparse::new();
    assert!(!sparse.a());
    assert!(!sparse.b());
    sparse.set_a(true);
    sparse.set_b(true);
    assert!(sparse.a());
    assert!(sparse.b());
}

#[test]
fn skip_getters() {
    #[bitfield]
    #[derive(Debug)]
    pub struct Sparse {
        #[skip(getters)]
        unused_1: B10,
        a: bool,
        #[skip(getters)]
        unused_2: B10,
        b: bool,
        #[skip(getters)]
        unused_3: B10,
    }

    let mut sparse = Sparse::new();
    assert!(!sparse.a());
    assert!(!sparse.b());
    sparse.set_a(true);
    sparse.set_b(true);
    assert!(sparse.a());
    assert!(sparse.b());

    // Use setters of fields with skipped getters:
    sparse.set_unused_1(0b0011_1111_1111);
    sparse.set_unused_2(0b0011_1111_1111);
    sparse.set_unused_3(0b0011_1111_1111);

    assert_eq!(sparse.into_bytes(), [0xFF; 4]);
}

#[test]
fn skip_setters() {
    #[bitfield]
    #[derive(Debug)]
    pub struct Sparse {
        #[skip(setters)]
        unused_1: B10,
        a: bool,
        #[skip(setters)]
        unused_2: B10,
        b: bool,
        #[skip(setters)]
        unused_3: B10,
    }

    let mut sparse = Sparse::new();
    assert!(!sparse.a());
    assert!(!sparse.b());
    sparse.set_a(true);
    sparse.set_b(true);
    assert!(sparse.a());
    assert!(sparse.b());

    // Use setters of fields with skipped getters:
    assert_eq!(sparse.unused_1(), 0);
    assert_eq!(sparse.unused_2(), 0);
    assert_eq!(sparse.unused_3(), 0);
}

#[test]
fn skip_with_debug() {
    #[bitfield]
    #[derive(Debug)]
    pub struct Sparse {
        #[skip(getters)]
        no_getters: B10,
        a: bool,
        #[skip(setters)]
        no_setters: B10,
        b: bool,
        #[skip]
        skipped: B10,
    }

    let sparse = Sparse::new().with_a(true).with_b(false);
    assert_eq!(
        format!("{sparse:?}"),
        "Sparse { a: true, no_setters: 0, b: false }",
    );
    assert_eq!(
        format!("{sparse:#X?}"),
        "Sparse {\n    a: true,\n    no_setters: 0x0,\n    b: false,\n}",
    );
}
