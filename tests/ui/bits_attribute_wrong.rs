use modular_bitfield::prelude::*;

#[derive(Specifier, Debug)]
pub enum TriggerMode {
    Edge = 0,
    Level = 1,
}

#[bitfield]
pub struct MismatchedTypes {
    #[bits = 9]
    trigger_mode: TriggerMode,
    reserved: B7,
}

const NOT_A_LITERAL: u32 = 9;

#[bitfield]
pub struct InvalidValueType {
    #[bits = NOT_A_LITERAL]
    trigger_mode: TriggerMode,
    reserved: B7,
}

#[bitfield]
pub struct DuplicateAttribute {
    #[bits = 1]
    #[bits = 1]
    trigger_mode: TriggerMode,
    reserved: B7,
}

#[bitfield]
pub struct NotANameValue {
    #[bits(1)]
    trigger_mode: TriggerMode,
    reserved: B7,
}

fn main() {}
