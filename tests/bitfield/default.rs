use modular_bitfield::prelude::*;

#[test]
fn basic_defaults() {
    #[bitfield]
    pub struct BasicDefaults {
        foo: bool,
        #[default = false]
        bar: bool,
        baz: B6,
        #[default = 42]
        qux: B8,
    }

    let bf = BasicDefaults::new();
    assert!(!bf.foo()); // no default
    assert!(!bf.bar()); // explicit false
    assert_eq!(bf.baz(), 0); // no default
    assert_eq!(bf.qux(), 42); // explicit default
}

#[test]
fn all_primitive_types() {
    #[bitfield]
    pub struct PrimitiveDefaults {
        #[default = true]
        flag: bool,
        #[default = 0b11]
        two_bits: B2,
        #[default = 0xFF]
        byte: B8,
        #[default = 0x1234]
        word: B16,
    }

    let bf = PrimitiveDefaults::new();
    assert!(bf.flag());
    assert_eq!(bf.two_bits(), 0b11);
    assert_eq!(bf.byte(), 0xFF);
    assert_eq!(bf.word(), 0x1234);
}

#[test]
fn enum_defaults() {
    #[derive(Debug, Clone, Copy, PartialEq, Specifier)]
    #[bits = 2]
    pub enum Mode {
        Off = 0,
        On = 1,
        Auto = 2,
    }

    #[bitfield]
    pub struct EnumDefaults {
        #[default = Mode::Auto]
        mode: Mode,
        #[default = true]
        enabled: bool,
        padding: B5,
    }

    let bf = EnumDefaults::new();
    assert_eq!(bf.mode(), Mode::Auto);
    assert!(bf.enabled());
    assert_eq!(bf.padding(), 0);
}

#[test]
fn nested_bitfield_defaults() {
    #[bitfield(bits = 4)]
    #[derive(Debug, Clone, Copy, PartialEq, Specifier)]
    pub struct Nibble {
        #[default = 0xF]
        value: B4,
    }

    #[bitfield]
    pub struct Parent {
        nibble: Nibble,
        #[default = 0xAB]
        data: B12,
    }

    let parent = Parent::new();
    assert_eq!(parent.nibble().value(), 0xF); // Uses Nibble's default
    assert_eq!(parent.data(), 0xAB);
}

#[test]
fn nested_specifier_defaults() {
    #[bitfield(bits = 8)]
    #[derive(Debug, Clone, Copy, PartialEq, Specifier)]
    pub struct Flags {
        #[default = true]
        enabled: bool,
        #[default = false]
        debug: bool,
        #[default = 0b11111]
        level: B6,
    }

    #[bitfield]
    pub struct Config {
        flags: Flags,
        #[default = 0x42]
        data: B8,
    }

    let config = Config::new();
    let flags = config.flags();
    assert!(flags.enabled());
    assert!(!flags.debug());
    assert_eq!(flags.level(), 0b11111);
    assert_eq!(config.data(), 0x42);
}

#[test]
fn const_expressions() {
    const DEFAULT_VALUE: u8 = 42;

    #[bitfield]
    pub struct ConstDefaults {
        #[default = 1 + 2]
        sum: B4,
        #[default = DEFAULT_VALUE]
        from_const: B8,
        #[default = 0xFF & 0x0F]
        masked: B4,
    }

    let bf = ConstDefaults::new();
    assert_eq!(bf.sum(), 3);
    assert_eq!(bf.from_const(), 42);
    assert_eq!(bf.masked(), 0x0F);
}

#[test]
fn derive_default_trait() {
    #[bitfield]
    #[derive(Default, PartialEq, Debug)]
    pub struct DeriveDefault {
        #[default = true]
        flag: bool,
        #[default = 123]
        value: B8,
        counter: B7, // no default
    }

    let default_bf = DeriveDefault::default();
    let new_bf = DeriveDefault::new();
    
    // Default trait should match new()
    assert_eq!(default_bf, new_bf);
    assert!(default_bf.flag());
    assert_eq!(default_bf.value(), 123);
    assert_eq!(default_bf.counter(), 0);
}

#[test]
fn defaults_with_skip() {
    #[bitfield]
    pub struct SkipDefaults {
        #[default = 42]
        #[skip(setters)]
        readonly: B8,
        #[default = true]
        #[skip]
        _reserved: bool,
        #[default = 0x7]
        data: B7,
    }

    let bf = SkipDefaults::new();
    assert_eq!(bf.readonly(), 42);
    assert_eq!(bf.data(), 0x7);
    
    // Verify readonly field has no setter
    let bf2 = bf.with_data(0x5);
    assert_eq!(bf2.readonly(), 42); // unchanged
    assert_eq!(bf2.data(), 0x5);
}

#[test]
fn new_zeroed_ignores_defaults() {
    #[bitfield]
    pub struct ZeroedTest {
        #[default = true]
        flag: bool,
        #[default = 0xFF]
        value: B8,
        padding: B7,
    }

    let zeroed = ZeroedTest::new_zeroed();
    assert!(!zeroed.flag());
    assert_eq!(zeroed.value(), 0);
    assert_eq!(zeroed.padding(), 0);
}

#[test]
fn specifier_default_matches_constructor() {
    #[bitfield(bits = 8)]
    #[derive(Debug, Clone, Copy, PartialEq, Specifier)]
    struct TestBitfield {
        #[default = true]
        flag: bool,
        #[default = 0x7]
        value: B7,
    }

    let instance = TestBitfield::new();
    let from_default = TestBitfield::from_bytes(TestBitfield::DEFAULT.to_le_bytes());
    
    assert_eq!(instance.into_bytes(), from_default.into_bytes());
}

#[test]
fn cross_byte_boundary() {
    #[bitfield]
    pub struct CrossByte {
        #[default = 0x3]
        prefix: B2,
        #[default = 0x3FF]
        middle: B10, // Crosses byte boundary
        #[default = 0xF]
        suffix: B4,
    }

    let bf = CrossByte::new();
    assert_eq!(bf.prefix(), 0x3);
    assert_eq!(bf.middle(), 0x3FF);
    assert_eq!(bf.suffix(), 0xF);
}

#[test]
fn tuple_struct_defaults() {
    #[bitfield]
    pub struct TupleDefaults(
        #[default = true] bool,
        #[default = 255] B8,
        B7, // no default
    );

    let bf = TupleDefaults::new();
    assert!(bf.get_0());
    assert_eq!(bf.get_1(), 255);
    assert_eq!(bf.get_2(), 0);
}

#[test]
fn const_context_usage() {
    #[bitfield]
    pub struct ConstBitfield {
        #[default = 0xFF]
        value: B8,
        #[default = true]
        flag: bool,
        padding: B7,
    }

    const STATIC_BF: ConstBitfield = ConstBitfield::new();
    assert_eq!(STATIC_BF.value(), 0xFF);
    assert!(STATIC_BF.flag());
}

#[test]
fn max_value_defaults() {
    #[bitfield]
    pub struct MaxValues {
        #[default = 0x7F]
        seven_bits: B7,
        #[default = 0xFFFF]
        sixteen_bits: B16,
        #[default = true]
        flag: bool,
        #[default = 0xFF]
        byte: B8,
    }

    let bf = MaxValues::new();
    assert_eq!(bf.seven_bits(), 0x7F);
    assert_eq!(bf.sixteen_bits(), 0xFFFF);
    assert!(bf.flag());
    assert_eq!(bf.byte(), 0xFF);
}