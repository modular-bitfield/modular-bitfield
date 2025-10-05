use super::checks;

/// Helper trait for underlying primitives handling of bitfields.
///
/// # Note
///
/// Must not and cannot be implemented by dependencies.
#[doc(hidden)]
pub const trait PushBits: checks::private::Sealed {
    fn push_bits(&mut self, amount: u32, bits: u8);
}

/// Helper trait for underlying primitives handling of bitfields.
///
/// # Note
///
/// Must not and cannot be implemented by dependencies.
#[doc(hidden)]
pub const trait PopBits: checks::private::Sealed {
    fn pop_bits(&mut self, amount: u32) -> u8;
}

/// Trait implemented by primitives that drive bitfield manipulations generically.
#[doc(hidden)]
pub trait SpecifierBytes: checks::private::Sealed {
    /// The base type that the specifier is operating on.
    type Bytes;
}

pub trait IsU8Compatible: checks::private::Sealed {}
pub trait IsU16Compatible: checks::private::Sealed {}
pub trait IsU32Compatible: checks::private::Sealed {}
pub trait IsU64Compatible: checks::private::Sealed {}
pub trait IsU128Compatible: checks::private::Sealed {}

impl IsU8Compatible for [(); 8] {}
impl IsU16Compatible for [(); 16] {}
impl IsU32Compatible for [(); 32] {}
impl IsU64Compatible for [(); 64] {}
impl IsU128Compatible for [(); 128] {}
