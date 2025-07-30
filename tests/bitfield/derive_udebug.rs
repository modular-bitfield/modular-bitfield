#![cfg(feature = "ufmt")]

//! Tests for `#[derive(uDebug)]`

extern crate alloc;
use core::fmt::Write;
use ufmt::derive::uDebug;
use ufmt::{uWrite, uwrite};
use modular_bitfield::prelude::*;

#[derive(Debug)]
pub struct AllocString(alloc::string::String);

impl uWrite for AllocString {
    type Error = ();

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.0.write_str(s).map_err(|_|())
    }
}

impl AllocString {
    pub fn new() -> Self {
        Self(alloc::string::String::new())
    }
}

impl<'s> PartialEq<&'s str> for AllocString {
    fn eq(&self, other: &&'s str) -> bool {
        self.0 == *other
    }
}

/// Wrapper for ufmt's uwrite!() macro to return an AllocString, analogous to the format!() macro.
macro_rules! ufmt_format_wrapper {
    ($($tt:tt)*) => {{
        let mut s = AllocString::new();
        uwrite!(s, $($tt)*).unwrap();
        s
    }};
}

#[test]
fn print_invalid_bits() {
    #[derive(Specifier, uDebug)]
    #[bits = 2]
    pub enum Status {
        Green = 0,
        Yellow = 1,
        Red = 2, // 0x11 (= 3) is undefined here for Status!
    }

    #[bitfield]
    #[derive(uDebug)]
    pub struct DataPackage {
        status: Status,
        contents: B4,
        is_alive: bool,
        is_received: bool,
    }

    let package = DataPackage::from_bytes([0b0101_1011]);
    assert_eq!(
        ufmt_format_wrapper!("{:?}", package),
        "DataPackage { status: InvalidBitPattern { invalid_bytes: 3 }, contents: 6, is_alive: true, is_received: false }",
    );
    assert_eq!(
        ufmt_format_wrapper!("{:#?}", package),
        "DataPackage {\n    \
            status: InvalidBitPattern {\n        \
                invalid_bytes: 3,\n    \
            },\n    \
            contents: 6,\n    \
            is_alive: true,\n    \
            is_received: false,\n\
        }",
    );
}

#[test]
fn respects_other_derives() {
    #[bitfield]
    #[derive(Debug, uDebug, Clone, PartialEq, Eq)]
    pub struct Color {
        r: B6,
        g: B6,
        b: B6,
        a: B6,
    }

    let color1 = Color::new().with_r(63).with_g(32).with_b(16).with_a(8);
    let color2 = color1.clone();
    assert_eq!(color1, color2);
    assert_eq!(ufmt_format_wrapper!("{:?}", color1), "Color { r: 63, g: 32, b: 16, a: 8 }",);
    assert_eq!(
        ufmt_format_wrapper!("{:#?}", color2),
        "Color {\n    r: 63,\n    g: 32,\n    b: 16,\n    a: 8,\n}",
    );
}

#[test]
fn valid_use_2() {
    #[derive(Specifier, uDebug)]
    pub enum Status {
        Green,
        Yellow,
        Red,
        None,
    }

    #[bitfield]
    #[derive(uDebug)]
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
        ufmt_format_wrapper!("{:?}", package),
        "DataPackage { status: Green, contents: 3235826430, is_alive: true, is_received: false }",
    );
    assert_eq!(
        ufmt_format_wrapper!("{:#?}", package),
        "DataPackage {\n    status: Green,\n    contents: 3235826430,\n    is_alive: true,\n    is_received: false,\n}",
    );
}

#[test]
fn valid_use_specifier() {
    #[bitfield(filled = false)] // Requires just 4 bits!
    #[derive(Specifier, uDebug)]
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
        ufmt_format_wrapper!("{:?}", header),
        "Header { status: 1, is_alive: true, is_received: false }",
    );
    assert_eq!(
        ufmt_format_wrapper!("{:#?}", header),
        "Header {\n    status: 1,\n    is_alive: true,\n    is_received: false,\n}",
    );
}

#[test]
fn valid_use() {
    #[bitfield]
    #[derive(uDebug)]
    pub struct Color {
        r: B6,
        g: B6,
        b: B6,
        a: B6,
    }

    let color = Color::new().with_r(63).with_g(32).with_b(16).with_a(8);
    assert_eq!(ufmt_format_wrapper!("{:?}", color), "Color { r: 63, g: 32, b: 16, a: 8 }",);
    assert_eq!(
        ufmt_format_wrapper!("{:#?}", color),
        "Color {\n    r: 63,\n    g: 32,\n    b: 16,\n    a: 8,\n}",
    );
}

#[test]
fn valid_use_tuple() {
    #[bitfield]
    #[derive(uDebug)]
    pub struct Color(B6, B6, B6, B6);

    let color = Color::new().with_0(63).with_1(32).with_2(16).with_3(8);
    assert_eq!(ufmt_format_wrapper!("{:?}", color), "Color(63, 32, 16, 8)",);
    assert_eq!(
        ufmt_format_wrapper!("{:#?}", color),
        "Color(\n    63,\n    32,\n    16,\n    8,\n)",
    );
}
