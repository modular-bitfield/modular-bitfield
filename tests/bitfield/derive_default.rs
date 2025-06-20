use modular_bitfield::prelude::*;

#[test]
fn basic_default_without_field_defaults() {
    #[bitfield]
    #[derive(Default, PartialEq, Debug)]
    pub struct BasicBitfield {
        flag: bool,
        value: B8,
        counter: B4,
        enabled: bool,
        padding: B2,
    }

    let default_bf = BasicBitfield::default();
    let new_bf = BasicBitfield::new();

    // Default should be identical to new()
    assert_eq!(default_bf, new_bf);

    // All fields should be zero-initialized
    assert!(!default_bf.flag());
    assert_eq!(default_bf.value(), 0);
    assert_eq!(default_bf.counter(), 0);
    assert!(!default_bf.enabled());
    assert_eq!(default_bf.padding(), 0);
}

#[test]
fn default_with_field_defaults() {
    #[bitfield]
    #[derive(Default, PartialEq, Debug)]
    pub struct WithFieldDefaults {
        #[default(true)]
        flag: bool,
        #[default(42)]
        value: B8,
        counter: B4, // no default
        #[default(false)]
        enabled: bool,
        #[default(3)]
        padding: B2,
    }

    let default_bf = WithFieldDefaults::default();
    let new_bf = WithFieldDefaults::new();

    // Default should be identical to new()
    assert_eq!(default_bf, new_bf);

    // Fields with defaults should use default values
    assert!(default_bf.flag());
    assert_eq!(default_bf.value(), 42);
    assert_eq!(default_bf.counter(), 0); // no default, so zero
    assert!(!default_bf.enabled());
    assert_eq!(default_bf.padding(), 3);
}

#[test]
fn default_with_enum_fields() {
    #[derive(Debug, Clone, Copy, PartialEq, Specifier)]
    #[bits = 2]
    pub enum Status {
        Idle = 0,
        Running = 1,
        Paused = 2,
        Error = 3,
    }

    #[bitfield]
    #[derive(Default, PartialEq, Debug)]
    pub struct WithEnumDefaults {
        #[default(Status::Running)]
        status: Status,
        #[default(true)]
        enabled: bool,
        counter: B5, // no default
    }

    let default_bf = WithEnumDefaults::default();
    let new_bf = WithEnumDefaults::new();

    // Default should be identical to new()
    assert_eq!(default_bf, new_bf);

    // Check field values
    assert_eq!(default_bf.status(), Status::Running);
    assert!(default_bf.enabled());
    assert_eq!(default_bf.counter(), 0); // no default, so zero
}

#[test]
fn default_tuple_struct() {
    #[bitfield]
    #[derive(Default, PartialEq, Debug)]
    pub struct TupleBitfield(
        #[default(true)] bool,
        #[default(255)] B8,
        B4, // no default
        #[default(false)] bool,
        B2, // padding to make it 16 bits total
    );

    let default_bf = TupleBitfield::default();
    let new_bf = TupleBitfield::new();

    // Default should be identical to new()
    assert_eq!(default_bf, new_bf);

    // Check field values
    assert!(default_bf.get_0());
    assert_eq!(default_bf.get_1(), 255);
    assert_eq!(default_bf.get_2(), 0); // no default, so zero
    assert!(!default_bf.get_3());
    assert_eq!(default_bf.get_4(), 0); // no default, so zero
}

#[test]
fn default_with_complex_expressions() {
    const DEFAULT_VALUE: u8 = 42;

    #[bitfield]
    #[derive(Default, PartialEq, Debug)]
    pub struct ComplexDefaults {
        #[default(1 + 2)]
        a: B4,
        #[default(DEFAULT_VALUE)]
        b: B8,
        #[default(0xFF & 0x0F)]
        c: B4,
    }

    let default_bf = ComplexDefaults::default();
    let new_bf = ComplexDefaults::new();

    // Default should be identical to new()
    assert_eq!(default_bf, new_bf);

    // Check field values
    assert_eq!(default_bf.a(), 3); // 1 + 2
    assert_eq!(default_bf.b(), DEFAULT_VALUE);
    assert_eq!(default_bf.c(), 0x0F); // 0xFF & 0x0F
}

#[test]
fn default_preserves_other_derives() {
    extern crate alloc;
    use alloc::format;

    #[bitfield]
    #[derive(Default, Debug, Clone, PartialEq, Eq)]
    pub struct MultiDerive {
        #[default(true)]
        flag: bool,
        #[default(123)]
        value: B8,
        #[default(127)]
        padding: B7,
    }

    let default_bf = MultiDerive::default();
    let cloned_bf = default_bf.clone();

    // Test that all derives work
    assert_eq!(default_bf, cloned_bf);
    // Test Debug implementation works
    let _debug_str = format!("{:?}", default_bf);

    // Check values
    assert!(default_bf.flag());
    assert_eq!(default_bf.value(), 123);
    assert_eq!(default_bf.padding(), 127);
}
