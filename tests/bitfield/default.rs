use modular_bitfield::prelude::*;

#[test]
fn basic_functionality() {
    #[bitfield]
    pub struct WithDefaults {
        pub foo: bool,
        #[default(false)]
        pub bar: bool,
        pub baz: B6,
        #[default(42)]
        pub qux: B8,
    }

    // Test that new_zeroed() creates zero-initialized data
    let bf_new = WithDefaults::new_zeroed();
    assert!(!bf_new.foo());
    assert!(!bf_new.bar());
    assert_eq!(bf_new.baz(), 0);
    assert_eq!(bf_new.qux(), 0);

    // Test that new() applies default values
    let bf_defaults = WithDefaults::new();
    assert!(!bf_defaults.foo());  // no default specified
    assert!(!bf_defaults.bar());  // default specified as false
    assert_eq!(bf_defaults.baz(), 0);      // no default specified
    assert_eq!(bf_defaults.qux(), 42);     // default specified as 42
}

#[test]
fn comprehensive_defaults() {
    #[bitfield]
    pub struct Foo {
        pub foo: bool,
        #[default(true)]
        pub flag: bool,
        pub bar: bool,
        #[skip]
        __: B5,
    }

    // Test that new_zeroed() creates zero-initialized data
    let bf_new = Foo::new_zeroed();
    assert!(!bf_new.foo());
    assert!(!bf_new.flag());
    assert!(!bf_new.bar());

    // Test that new() applies default values
    let bf_defaults = Foo::new();
    assert!(!bf_defaults.foo());  // no default specified
    assert!(bf_defaults.flag());  // default specified as true
    assert!(!bf_defaults.bar());  // no default specified
    
    // Test that manually setting works the same as defaults
    let bf_manual = Foo::new().with_flag(true);
    assert_eq!(bf_defaults.into_bytes(), bf_manual.into_bytes());
}

#[test]
fn all_fields_with_defaults() {
    #[bitfield]
    pub struct AllDefaults {
        #[default(true)]
        a: bool,
        #[default(false)]
        b: bool,
        #[default(3)]
        c: B2,
        #[default(15)]
        d: B4,
    }

    let bf = AllDefaults::new();
    assert!(bf.a());
    assert!(!bf.b());
    assert_eq!(bf.c(), 3);
    assert_eq!(bf.d(), 15);
}

#[test]
fn complex_expressions() {
    #[bitfield]
    pub struct ComplexDefaults {
        #[default(1 + 2)]
        a: B4,
        #[default(0xFF & 0x0F)]
        b: B4,
        #[default(true)]
        c: bool,
        #[default(7)]
        #[allow(non_snake_case)]
        __padding: B7,
    }

    let bf = ComplexDefaults::new();
    assert_eq!(bf.a(), 3); // 1 + 2
    assert_eq!(bf.b(), 0x0F); // 0xFF & 0x0F
    assert!(bf.c());
}

#[test]
fn enum_defaults() {
    #[derive(Debug, Clone, Copy, PartialEq, Specifier)]
    #[bits = 2]
    pub enum Mode {
        Off = 0,
        On = 1,
        Auto = 2,
        Manual = 3,
    }

    #[bitfield]
    pub struct EnumDefaults {
        #[default(Mode::Auto)]
        mode: Mode,
        #[default(true)]
        enabled: bool,
        #[allow(non_snake_case)]
        __padding: B5,
    }

    let bf = EnumDefaults::new();
    assert_eq!(bf.mode(), Mode::Auto);
    assert!(bf.enabled());
}

#[test]
fn nested_bitfield_defaults() {
    #[bitfield(bits = 4)]
    #[derive(Debug, Clone, Copy, PartialEq, Specifier)]
    pub struct Nibble {
        #[default(0xF)]
        value: B4,
    }

    #[bitfield]
    pub struct NestedDefaults {
        // TODO: Nibble::new() is not const-evaluable yet
        // #[default(Nibble::new())]
        nibble: Nibble,
        #[default(0xAB)]
        data: B12,
    }

    let bf = NestedDefaults::new();
    assert_eq!(bf.nibble().value(), 0);  // No default, so zero-initialized
    assert_eq!(bf.data(), 0xAB);
}

#[test]
fn defaults_vs_manual_construction() {
    #[bitfield]
    pub struct TestDefaults {
        #[default(true)]
        a: bool,
        #[default(false)]
        b: bool,
        #[default(3)]
        c: B2,
        #[default(15)]
        d: B4,
    }

    // Verify that defaults produce the same result as manual construction
    let defaults = TestDefaults::new();
    let manual = TestDefaults::new()
        .with_a(true)
        .with_b(false)
        .with_c(3)
        .with_d(15);
    
    assert_eq!(defaults.into_bytes(), manual.into_bytes());
}

#[test]
fn partial_defaults_byte_representation() {
    #[bitfield]
    pub struct PartialDefaults {
        #[default(3)]
        a: B4,
        #[default(15)]
        b: B4,
        #[default(true)]
        c: bool,
        #[default(7)]
        #[allow(non_snake_case)]
        __padding: B7,
    }

    // Only some fields have defaults
    let bf = PartialDefaults::new();
    let bytes = bf.into_bytes();
    
    // Verify the exact byte pattern
    // a=3 (0011), b=15 (1111), c=true (1), padding=7 (0000111)
    // Layout: pppppppc bbbbaaaa
    assert_eq!(bytes[0], 0b11110011); // bbbbaaaa
    assert_eq!(bytes[1], 0b00001111); // pppppppc
}

#[test]
fn primitive_specifier_defaults() {
    #[bitfield]
    pub struct PrimitiveSpecifierDefaults {
        #[default(1)]
        flag: B1,
        #[default(0b11)]
        two_bits: B2,
        #[default(0x1F)]
        five_bits: B5,
        #[default(0xFF)]
        byte: B8,
        #[default(0x1234)]
        word: B16,
        #[default(0xDEADBEEF)]
        dword: B32,
    }

    let bf = PrimitiveSpecifierDefaults::new();
    assert_eq!(bf.flag(), 1);
    assert_eq!(bf.two_bits(), 0b11);
    assert_eq!(bf.five_bits(), 0x1F);
    assert_eq!(bf.byte(), 0xFF);
    assert_eq!(bf.word(), 0x1234);
    assert_eq!(bf.dword(), 0xDEADBEEF);
}

#[test]
fn bool_specifier_defaults() {
    #[bitfield]
    pub struct BoolSpecifierDefaults {
        #[default(true)]
        a: bool,
        #[default(false)]
        b: bool,
        #[default(true)]
        c: bool,
        #[default(false)]
        d: bool,
        #[default(0xF)]
        padding: B4,
    }

    let bf = BoolSpecifierDefaults::new();
    assert!(bf.a());
    assert!(!bf.b());
    assert!(bf.c());
    assert!(!bf.d());
    assert_eq!(bf.padding(), 0xF);
    
    // Verify byte representation
    let bytes = bf.into_bytes();
    // Expected layout: ppppddcba (LSB first)
    // a=1, b=0, c=1, d=0, padding=1111
    // Binary: 11110101 = 0xF5
    assert_eq!(bytes[0], 0b11110101);
}

#[test]
fn complex_specifier_defaults() {
    #[derive(Debug, Clone, Copy, PartialEq, Specifier)]
    #[bits = 3]
    pub enum Status {
        Idle = 0,
        Running = 1,
        Paused = 2,
        Stopped = 3,
        Error = 4,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Specifier)]
    #[bits = 4]
    pub enum Level {
        Low = 0,
        Medium = 5,
        High = 10,
        Critical = 15,
    }

    const DEFAULT_LEVEL: Level = Level::Medium;
    const DEFAULT_FLAGS: u8 = 0b1010;

    #[bitfield]
    pub struct ComplexSpecifierDefaults {
        #[default(DEFAULT_LEVEL)]
        level: Level,
        #[default(DEFAULT_FLAGS)]
        flags: B8,
        #[default(Status::Idle)]
        status: Status,
        #[default(true)]
        active: bool,
    }

    let bf = ComplexSpecifierDefaults::new();
    assert_eq!(bf.level(), Level::Medium);
    assert_eq!(bf.flags(), DEFAULT_FLAGS);
    assert_eq!(bf.status(), Status::Idle);
    assert!(bf.active());
}

#[test]
fn nested_specifier_construction() {
    #[bitfield(bits = 8)]
    #[derive(Debug, Clone, Copy, PartialEq, Specifier)]
    pub struct Flags {
        #[default(true)]
        enabled: bool,
        #[default(false)]
        debug: bool,
        #[default(true)]
        verbose: bool,
        #[default(0b11111)]
        level: B5,
    }

    #[bitfield]
    pub struct NestedSpecifierDefaults {
        // Note: Using Flags::new() in default won't work yet as it's not const-evaluable
        // For now, we'll comment this out and test it manually
        // #[default(Flags::new())]
        flags: Flags,
        #[default(0x42)]
        data: B8,
    }
    
    // Test that we can construct nested specifiers manually
    let flags = Flags::new();
    let bf_manual = NestedSpecifierDefaults::new_zeroed()
        .with_flags(flags)
        .with_data(0x42);
    
    // Test that new() applies defaults (only data has default now)
    let bf_defaults = NestedSpecifierDefaults::new();
    assert_eq!(bf_defaults.data(), 0x42);
    
    // Verify the flags field is zero-initialized (no default specified for flags field)
    let flags = bf_defaults.flags();
    assert!(!flags.enabled());   // zero-initialized
    assert!(!flags.debug());     // zero-initialized
    assert!(!flags.verbose());   // zero-initialized
    assert_eq!(flags.level(), 0); // zero-initialized
}