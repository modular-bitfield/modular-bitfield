//! Tests for `#[derive(Debug)]`

extern crate alloc;
use alloc::format;
use modular_bitfield::prelude::*;

#[test]
fn print_invalid_bits() {
    #[derive(BitfieldSpecifier, Debug)]
    #[bits = 2]
    pub enum Status {
        Green = 0,
        Yellow = 1,
        Red = 2, // 0x11 (= 3) is undefined here for Status!
    }

    #[bitfield]
    #[derive(Debug)]
    pub struct DataPackage {
        status: Status,
        contents: B4,
        is_alive: bool,
        is_received: bool,
    }

    let package = DataPackage::from_bytes([0b01011011]);
    assert_eq!(
        format!("{:?}", package),
        "DataPackage { status: InvalidBitPattern { invalid_bytes: 3 }, contents: 6, is_alive: true, is_received: false }",
    );
    assert_eq!(
        format!("{:#X?}", package),
        "DataPackage {\n    \
            status: InvalidBitPattern {\n        \
                invalid_bytes: 0x3,\n    \
            },\n    \
            contents: 0x6,\n    \
            is_alive: true,\n    \
            is_received: false,\n\
        }",
    );
}

#[test]
fn respects_other_derives() {
    #[bitfield]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Color {
        r: B6,
        g: B6,
        b: B6,
        a: B6,
    }

    let color1 = Color::new().with_r(63).with_g(32).with_b(16).with_a(8);
    let color2 = color1.clone();
    assert_eq!(color1, color2);
    assert_eq!(
        format!("{:?}", color1),
        "Color { r: 63, g: 32, b: 16, a: 8 }",
    );
    assert_eq!(
        format!("{:#x?}", color2),
        "Color {\n    r: 0x3f,\n    g: 0x20,\n    b: 0x10,\n    a: 0x8,\n}",
    );
}

#[test]
fn valid_use_2() {
    #[derive(BitfieldSpecifier, Debug)]
    pub enum Status {
        Green,
        Yellow,
        Red,
        None,
    }

    #[bitfield]
    #[derive(Debug)]
    pub struct DataPackage {
        status: Status,
        contents: B60,
        is_alive: bool,
        is_received: bool,
    }

    let package = DataPackage::new()
        .with_status(Status::Green)
        .with_contents(0xC0DE_CAFE)
        .with_is_alive(true)
        .with_is_received(false);
    assert_eq!(
        format!("{:?}", package),
        "DataPackage { status: Green, contents: 3235826430, is_alive: true, is_received: false }",
    );
    assert_eq!(
        format!("{:#X?}", package),
        "DataPackage {\n    status: Green,\n    contents: 0xC0DECAFE,\n    is_alive: true,\n    is_received: false,\n}",
    );
}

#[test]
fn valid_use_specifier() {
    #[bitfield(filled = false)] // Requires just 4 bits!
    #[derive(BitfieldSpecifier, Debug)]
    pub struct Header {
        status: B2,
        is_alive: bool,
        is_received: bool,
    }

    let header = Header::new()
        .with_status(1)
        .with_is_alive(true)
        .with_is_received(false);
    assert_eq!(
        format!("{:?}", header),
        "Header { status: 1, is_alive: true, is_received: false }",
    );
    assert_eq!(
        format!("{:#X?}", header),
        "Header {\n    status: 0x1,\n    is_alive: true,\n    is_received: false,\n}",
    );
}

#[test]
fn valid_use() {
    #[bitfield]
    #[derive(Debug)]
    pub struct Color {
        r: B6,
        g: B6,
        b: B6,
        a: B6,
    }

    let color = Color::new().with_r(63).with_g(32).with_b(16).with_a(8);
    assert_eq!(
        format!("{:?}", color),
        "Color { r: 63, g: 32, b: 16, a: 8 }",
    );
    assert_eq!(
        format!("{:#x?}", color),
        "Color {\n    r: 0x3f,\n    g: 0x20,\n    b: 0x10,\n    a: 0x8,\n}",
    );
}
