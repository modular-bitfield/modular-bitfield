#![doc = include_str!("../docs/index.md")]
#![no_std]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs, rust_2018_idioms)]

pub mod error;
#[doc(hidden)]
pub mod private;

use self::error::{InvalidBitPattern, OutOfBounds};

#[doc = include_str!("../docs/bitfield.md")]
pub use modular_bitfield_impl::bitfield;

#[doc = include_str!("../docs/bitfield_specifier.md")]
pub use modular_bitfield_impl::Specifier;

/// The prelude: `use modular_bitfield::prelude::*;`
pub mod prelude {
    pub use super::{bitfield, specifiers::*, Specifier};
}

/// The `Specifier` trait describes a sequence of bits stored in an integer
/// primitive (the [`Bytes`](Self::Bytes) type) and how to convert them to/from
/// a more convenient higher-level interface type (the [`InOut`](Self::InOut)
/// type).
///
/// For example:
///
/// * The specifier for `bool` converts between a `u8` with a 1 or 0 bit in
///   the lowest bit position and a native `bool`.
/// * The specifier for a unit enum with variants `{0, 1, 14}` converts
///   between a `u16` matching those variants and the enum type.
/// * The specifier for a 20-bit struct converts between a `u32` and the
///   struct type.
///
/// All types used in a `#[bitfield]` struct must implement this trait, and it
/// should usually only be implemented with
/// [`#[derive(Specifier)]`](macro@crate::Specifier).
pub trait Specifier {
    /// The number of bits used by the `Specifier`.
    const BITS: usize;

    /// The storage type. This is typically the smallest integer primitive that
    /// can store all possible values of the [`InOut`](Self::InOut) type.
    type Bytes;

    /// The interface type. This type is used by getters and setters. For
    /// integers, this is the same as the [`Bytes`](Self::Bytes) type; for other
    /// types with more logical representations, like an enum or struct, this is
    /// the enum or struct.
    type InOut;

    /// Converts an interface type into its storage type.
    ///
    /// # Errors
    ///
    /// If the input value is out of bounds, an error will be returned. For
    /// example, the value `b100_u8` cannot be converted with `B2` because it is
    /// three bits wide.
    fn into_bytes(input: Self::InOut) -> Result<Self::Bytes, OutOfBounds>;

    /// Converts a storage type into its interface type.
    ///
    /// # Errors
    ///
    /// If the given bit pattern is invalid for the interface type, an error
    /// will be returned. For example, `3_u8` cannot be converted to an enum
    /// which only has variants `{0, 1, 2}`.
    fn from_bytes(bytes: Self::Bytes) -> Result<Self::InOut, InvalidBitPattern<Self::Bytes>>;
}

/// The default set of predefined specifiers.
pub mod specifiers {
    ::modular_bitfield_impl::define_specifiers!();
}
