#![doc = include_str!("../docs/index.md")]
#![no_std]
// #![forbid(unsafe_code)] TODO: See issue with PushBits
#![warn(clippy::pedantic, missing_docs, rust_2018_idioms)]

#![feature(const_trait_impl)]
#![feature(const_try)]
#![feature(const_option_ops)]
#![feature(const_index)]
#![feature(const_slice_make_iter)]
#![feature(const_default)]

pub mod error;
#[doc(hidden)]
pub mod private;

use self::error::{InvalidBitPattern, OutOfBounds};

#[doc = include_str!("../docs/bitfield.md")]
pub use modular_bitfield_impl::bitfield;

#[doc = include_str!("../docs/bitfield_specifier.md")]
pub use modular_bitfield_impl::Specifier;

#[doc(hidden)]
pub use modular_bitfield_impl::BitfieldSpecifier;

/// The prelude: `use modular_bitfield::prelude::*;`
pub mod prelude {
    #[doc(hidden)]
    pub use super::BitfieldSpecifier;
    pub use super::{bitfield, specifiers::*, Specifier};
}

/// Trait implemented by all bitfield specifiers.
///
/// Should generally not be implemented directly by users
/// but through the macros provided by the crate.
///
/// # Note
///
/// These can be all unsigned fixed-size primitives,
/// represented by `B1, B2, ... B64` and enums that
/// derive from `Specifier`.
#[const_trait]
pub trait Specifier {
    /// The amount of bits used by the specifier.
    const BITS: usize;

    /// The base type of the specifier.
    ///
    /// # Note
    ///
    /// This is the type that is used internally for computations.
    type Bytes;

    /// The interface type of the specifier.
    ///
    /// # Note
    ///
    /// This is the type that is used for the getters and setters.
    type InOut;

    /// Converts the in-out type into bytes.
    ///
    /// # Errors
    ///
    /// If the in-out type is out of bounds. This can for example happen if your
    /// in-out type is `u8` (for `B7`) but you specified a value that is bigger
    /// or equal to 128 which exceeds the 7 bits.
    fn into_bytes(input: Self::InOut) -> Result<Self::Bytes, OutOfBounds>;

    /// Converts the given bytes into the in-out type.
    ///
    /// # Errors
    ///
    /// If the given byte pattern is invalid for the in-out type.
    /// This can happen for example for enums that have a number of variants which
    /// is not equal to the power of two and therefore yields some invalid bit
    /// patterns.
    fn from_bytes(bytes: Self::Bytes) -> Result<Self::InOut, InvalidBitPattern<Self::Bytes>>;
}

/// The default set of predefined specifiers.
pub mod specifiers {
    ::modular_bitfield_impl::define_specifiers!();
}
