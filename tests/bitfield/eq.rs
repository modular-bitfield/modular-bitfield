use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;

#[test]
fn eq_specified_fields() {
    #[bitfield]
    #[derive(Debug, PartialEq, Eq)]
    pub struct DataPackage {
        is_ok: bool,
        contents: B4,
        is_received: bool,
        dummy: B2,
    }

    let package_1 = DataPackage::from_bytes([0b0101_1011]);
    let package_1_eq = DataPackage::from_bytes([0b0101_1011]);
    let package_1_ne = DataPackage::from_bytes([0b0100_1011]);

    assert_eq!(package_1, package_1_eq);

    assert_ne!(package_1, package_1_ne);
}

#[test]
fn eq_skips_ignored_field() {
    #[bitfield]
    #[derive(PartialEq, Debug)]
    pub struct DataPackage {
        is_ok: bool,
        #[skip]
        __: B4,
        is_received: bool,
        more_data: B2,
    }

    let package_1 = DataPackage::from_bytes([0b1001_1111]);
    let package_1_should_eq = DataPackage::from_bytes([0b1000_0001]);
    assert_eq!(package_1, package_1_should_eq);
}

#[test]
fn eq_skips_multi_ignored_field() {
    #[bitfield]
    #[derive(PartialEq, Debug)]
    pub struct DataPackage {
        is_ok: bool,
        #[skip]
        __: B4,
        is_received: bool,
        more_data: B2,
        #[skip]
        byte: u8,
    }

    let package_1 = DataPackage::from_bytes([0b1001_1111, 0b1010_1010]);
    let package_1_should_eq = DataPackage::from_bytes([0b1000_0001, 0b1111_1111]);
    assert_eq!(package_1, package_1_should_eq);
}

#[test]
fn eq_skips_enum_field() {
    #[derive(Specifier, PartialEq, Eq, Debug)]
    #[bits = 2]
    pub enum DataType {
        Control = 0,
        Data = 1,
        Ack = 2,
    }
    #[bitfield]
    #[derive(PartialEq, Debug)]
    pub struct DataPackage {
        is_ok: bool,
        contents: B5,
        #[skip]
        is_received: DataType,
    }

    let package_1 = DataPackage::from_bytes([0b0100_1011]);
    let package_1_should_eq = DataPackage::from_bytes([0b1100_1011]);
    assert_eq!(package_1, package_1_should_eq);
}

#[test]
fn eq_considers_only_setters_field() {
    #[derive(Specifier, PartialEq, Eq, Debug)]
    #[bits = 2]
    pub enum DataType {
        Control = 0,
        Data = 1,
        Ack = 2,
    }
    #[bitfield]
    #[derive(PartialEq, Debug)]
    pub struct DataPackage {
        is_ok: bool,
        contents: B5,
        #[skip(getters)]
        is_received: DataType,
    }

    let package_1 = DataPackage::from_bytes([0b0100_1011]);
    let package_1_should_eq = DataPackage::from_bytes([0b1000_1011]);
    assert_ne!(package_1, package_1_should_eq);
}

#[test]
fn eq_considers_only_getters_field() {
    #[derive(Specifier, PartialEq, Eq, Debug)]
    #[bits = 2]
    pub enum DataType {
        Control = 0,
        Data = 1,
        Ack = 2,
    }
    #[bitfield]
    #[derive(Debug, PartialEq)]
    pub struct DataPackage {
        is_ok: bool,
        contents: B5,
        #[skip(setters)]
        is_received: DataType,
    }

    let package_1 = DataPackage::from_bytes([0b0100_1011]);
    let package_1_should_eq = DataPackage::from_bytes([0b1100_1011]);
    assert_ne!(package_1, package_1_should_eq);
}

#[test]
fn eq_all_fields_skipped() {
    #[derive(Specifier, PartialEq, Eq, Debug)]
    #[bits = 2]
    pub enum DataType {
        Control = 0,
        Data = 1,
        Ack = 2,
    }
    #[bitfield]
    #[derive(Debug, PartialEq)]
    pub struct DataPackage {
        #[skip]
        is_ok: bool,
        #[skip]
        contents: B5,
        #[skip]
        is_received: DataType,
    }

    let package_1 = DataPackage::from_bytes([0b0100_1011]);
    let package_1_should_eq = DataPackage::from_bytes([0b1000_1110]);
    assert_eq!(package_1, package_1_should_eq);
}
