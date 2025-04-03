//! Provides macros to support bitfield structs allowing for modular use of bit-enums.
//!
//! The mainly provided macros are [`#[bitfield]`](bitfield) for structs and
//! [`#[derive(BitfieldSpecifier)]`](BitfieldSpecifier) for enums that shall be usable
//! within bitfield structs.
//!
//! There are preset bitfield specifiers such as `B1`, `B2`,..,`B64`
//! that allow for easy bitfield usage in structs very similar to how
//! they work in C or C++.
//!
//! - Performance of the macro generated code is as fast as its hand-written
//!   alternative.
//! - Compile-time checks allow for safe usage of bitfield structs and enums.
//!
//!
//! ### Usage
//!
//! Annotate a Rust struct with the [`#[bitfield]`](bitfield) attribute in order to convert it into a bitfield,
//! with [optional parameters](bitfield#parameters) that control how the bitfield is generated.
//! The `B1`, `B2`, ... `B128` prelude types can be used as primitives to declare the number of bits per field.
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! #[bitfield]
//! pub struct PackedData {
//!     header: B4,
//!     body: B9,
//!     is_alive: B1,
//!     status: B2,
//! }
//! ```
//!
//! This produces a `new` constructor as well as a variety of getters and setters that
//! allows to interact with the bitfield in a safe fashion:
//!
//! #### Example: Constructors
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! # #[bitfield]
//! # pub struct PackedData {
//! #     header: B4,
//! #     body: B9,
//! #     is_alive: B1,
//! #     status: B2,
//! # }
//! let data = PackedData::new()
//!     .with_header(1)
//!     .with_body(2)
//!     .with_is_alive(0)
//!     .with_status(3);
//! assert_eq!(data.header(), 1);
//! assert_eq!(data.body(), 2);
//! assert_eq!(data.is_alive(), 0);
//! assert_eq!(data.status(), 3);
//! ```
//!
//! #### Example: Primitive Types
//!
//! Any type that implements the `Specifier` trait can be used as a bitfield field.
//! Besides the already mentioned `B1`, .. `B128` also the `bool`, `u8`, `u16`, `u32`,
//! `u64` or `u128` primitive types can be used from prelude.
//!
//! We can use this knowledge to encode our `is_alive` as `bool` type instead of `B1`:
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! #[bitfield]
//! pub struct PackedData {
//!     header: B4,
//!     body: B9,
//!     is_alive: bool,
//!     status: B2,
//! }
//!
//! let mut data = PackedData::new()
//!     .with_is_alive(true);
//! assert!(data.is_alive());
//! data.set_is_alive(false);
//! assert!(!data.is_alive());
//! ```
//!
//! #### Example: Enum Specifiers
//!
//! It is possible to derive the `Specifier` trait for `enum` types very easily to make
//! them also usable as a field within a bitfield type:
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! #[derive(BitfieldSpecifier)]
//! pub enum Status {
//!     Red, Green, Yellow, None,
//! }
//!
//! #[bitfield]
//! pub struct PackedData {
//!     header: B4,
//!     body: B9,
//!     is_alive: bool,
//!     status: Status,
//! }
//! ```
//!
//! #### Example: Extra Safety Guard
//!
//! In order to make sure that our `Status` enum still requires exatly 2 bit we can add
//! `#[bits = 2]` to its field:
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! # #[derive(BitfieldSpecifier)]
//! # pub enum Status {
//! #     Red, Green, Yellow, None,
//! # }
//! #
//! #[bitfield]
//! pub struct PackedData {
//!     header: B4,
//!     body: B9,
//!     is_alive: bool,
//!     #[bits = 2]
//!     status: Status,
//! }
//! ```
//!
//! Setting and getting our new `status` field is naturally as follows:
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! # #[derive(BitfieldSpecifier)]
//! # #[derive(Debug, PartialEq, Eq)]
//! # pub enum Status {
//! #     Red, Green, Yellow, None,
//! # }
//! #
//! # #[bitfield]
//! # pub struct PackedData {
//! #     header: B4,
//! #     body: B9,
//! #     is_alive: bool,
//! #     #[bits = 2]
//! #     status: Status,
//! # }
//! #
//! let mut data = PackedData::new()
//!     .with_status(Status::Green);
//! assert_eq!(data.status(), Status::Green);
//! data.set_status(Status::Red);
//! assert_eq!(data.status(), Status::Red);
//! ```
//!
//! #### Example: Skipping Fields
//!
//! It might make sense to only allow users to set or get information from a field or
//! even to entirely disallow interaction with a bitfield. For this the `#[skip]` attribute
//! can be used on a bitfield of a `#[bitfield]` annotated struct.
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! #[bitfield]
//! pub struct SomeBitsUndefined {
//!     #[skip(setters)]
//!     read_only: bool,
//!     #[skip(getters)]
//!     write_only: bool,
//!     #[skip]
//!     unused: B6,
//! }
//! ```
//!
//! It is possible to use `#[skip(getters, setters)]` or `#[skip(getters)]` followed by a `#[skip(setters)]`
//! attribute applied on the same bitfield. The effects are the same. When skipping both, getters and setters,
//! it is possible to completely avoid having to specify a name:
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! #[bitfield]
//! pub struct SomeBitsUndefined {
//!     #[skip] __: B2,
//!     is_activ: bool,
//!     #[skip] __: B2,
//!     is_received: bool,
//!     #[skip] __: B2,
//! }
//! ```
//!
//! #### Example: Unfilled Bitfields
//!
//! Sometimes it might be useful to not be required to construct a bitfield that defines
//! all bits and therefore is required to have a bit width divisible by 8. In this case
//! you can use the `filled: bool` parameter of the `#[bitfield]` macro in order to toggle
//! this for your respective bitfield:
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! #[bitfield(filled = false)]
//! pub struct SomeBitsUndefined {
//!     is_compact: bool,
//!     is_secure: bool,
//!     pre_status: B3,
//! }
//! ```
//!
//! In the above example `SomeBitsUndefined` only defines the first 5 bits and leaves the rest
//! 3 bits of its entire 8 bits undefined. The consequences are that its generated `from_bytes`
//! method is fallible since it must guard against those undefined bits.
//!
//! #### Example: Recursive Bitfields
//!
//! It is possible to use `#[bitfield]` structs as fields of `#[bitfield]` structs.
//! This is generally useful if there are some common fields for multiple bitfields
//! and is achieved by adding the `#[derive(BitfieldSpecifier)]` attribute to the struct
//! annotated with `#[bitfield]`:
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! # #[derive(BitfieldSpecifier)]
//! # pub enum Status {
//! #     Red, Green, Yellow, None,
//! # }
//! #
//! #[bitfield(filled = false)]
//! #[derive(BitfieldSpecifier)]
//! pub struct Header {
//!     is_compact: bool,
//!     is_secure: bool,
//!     pre_status: Status,
//! }
//!
//! #[bitfield]
//! pub struct PackedData {
//!     header: Header,
//!     body: B9,
//!     is_alive: bool,
//!     status: Status,
//! }
//! ```
//!
//! With the `bits: int` parameter of the `#[bitfield]` macro on the `Header` struct and the
//! `#[bits: int]` attribute of the `#[derive(BitfieldSpecifier)]` on the `Status` enum we
//! can have additional compile-time guarantees about the bit widths of the resulting entities:
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! #[derive(BitfieldSpecifier)]
//! #[bits = 2]
//! pub enum Status {
//!     Red, Green, Yellow, None,
//! }
//!
//! #[bitfield(bits = 4)]
//! #[derive(BitfieldSpecifier)]
//! pub struct Header {
//!     is_compact: bool,
//!     is_secure: bool,
//!     #[bits = 2]
//!     pre_status: Status,
//! }
//!
//! #[bitfield(bits = 16)]
//! pub struct PackedData {
//!     #[bits = 4]
//!     header: Header,
//!     body: B9,
//!     is_alive: bool,
//!     #[bits = 2]
//!     status: Status,
//! }
//! ```
//!
//! #### Example: Advanced Enum Specifiers
//!
//! For our `Status` enum we actually just need 3 status variants: `Green`, `Yellow` and `Red`.
//! We introduced the `None` status variants because `Specifier` enums by default are required
//! to have a number of variants that is a power of two. We can ship around this by specifying
//! `#[bits = 2]` on the top and get rid of our placeholder `None` variant while maintaining
//! the invariant of it requiring 2 bits:
//!
//! ```
//! # use modular_bitfield::prelude::*;
//!
//! #[derive(BitfieldSpecifier)]
//! #[bits = 2]
//! pub enum Status {
//!     Red, Green, Yellow,
//! }
//! ```
//!
//! However, having such enums now yields the possibility that a bitfield might contain invalid bit
//! patterns for such fields. We can safely access those fields with protected getters. For the sake
//! of demonstration we will use the generated `from_bytes` constructor with which we can easily
//! construct bitfields that may contain invalid bit patterns:
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! # use modular_bitfield::error::InvalidBitPattern;
//! #
//! # #[derive(BitfieldSpecifier)]
//! # #[derive(Debug, PartialEq, Eq)]
//! # #[bits = 2]
//! # pub enum Status {
//! #     Red, Green, Yellow,
//! # }
//! #
//! # #[bitfield(filled = false)]
//! # #[derive(BitfieldSpecifier)]
//! # pub struct Header {
//! #     is_compact: bool,
//! #     is_secure: bool,
//! #     pre_status: Status,
//! # }
//! #
//! # #[bitfield]
//! # pub struct PackedData {
//! #     header: Header,
//! #     body: B9,
//! #     is_alive: bool,
//! #     status: Status,
//! # }
//! #
//! let mut data = PackedData::from_bytes([0b0000_0000, 0b1100_0000]);
//! //           The 2 status field bits are invalid -----^^
//! //           as Red = 0x00, Green = 0x01 and Yellow = 0x10
//! assert_eq!(data.status_or_err(), Err(InvalidBitPattern { invalid_bytes: 0b11 }));
//! data.set_status(Status::Green);
//! assert_eq!(data.status_or_err(), Ok(Status::Green));
//! ```
//!
//! ## Generated Implementations
//!
//! For the example `#[bitfield]` struct the following implementations are going to be generated:
//!
//! ```
//! # use modular_bitfield::prelude::*;
//! #
//! #[bitfield]
//! pub struct Example {
//!     a: bool,
//!     b: B7,
//! }
//! ```
//!
//! | Signature | Description |
//! |:--|:--|
//! | `fn new() -> Self` | Creates a new instance of the bitfield with all bits initialized to 0. |
//! | `fn from_bytes([u8; 1]) -> Self` | Creates a new instance of the bitfield from the given raw bytes. |
//! | `fn into_bytes(self) -> [u8; 1]` | Returns the underlying bytes of the bitfield. |
//!
//! And below the generated signatures for field `a`:
//!
//! | Signature | Description |
//! |:--|:--|
//! | `fn a() -> bool` | Returns the value of `a` or panics if invalid. |
//! | `fn a_or_err() -> Result<bool, InvalidBitPattern<u8>>` | Returns the value of `a` of an error providing information about the invalid bits. |
//! | `fn set_a(&mut self, new_value: bool)` | Sets `a` to the new value or panics if `new_value` contains invalid bits. |
//! | `fn set_a_checked(&mut self, new_value: bool) -> Result<(), OutOfBounds>` | Sets `a` to the new value of returns an out of bounds error. |
//! | `fn with_a(self, new_value: bool) -> Self` | Similar to `set_a` but useful for method chaining. |
//! | `fn with_a_checked(self, new_value: bool) -> Result<Self, OutOfBounds>` | Similar to `set_a_checked` but useful for method chaining. |
//!
//! Getters for unnamed fields in tuple-like structs are prefixed with `get_`
//! (e.g. `get_0()`, `get_1_or_err()`, etc.).
//!
//! ## Generated Structure
//!
//! From David Tolnay's procedural macro workshop:
//!
//! The macro conceptualizes given structs as a sequence of bits 0..N.
//! The bits are grouped into fields in the order specified by the struct written by the user.
//!
//! The `#[bitfield]` attribute rewrites the caller's struct into a private byte array representation
//! with public getter and setter methods for each field.
//! The total number of bits N is required to be a multiple of 8: This is checked at compile time.
//!
//! ### Example
//!
//! The following invocation builds a struct with a total size of 32 bits or 4 bytes.
//! It places field `a` in the least significant bit of the first byte,
//! field `b` in the next three least significant bits,
//! field `c` in the remaining four most significant bits of the first byte,
//! and field `d` spanning the next three bytes.
//!
//! ```rust
//! use modular_bitfield::prelude::*;
//!
//! #[bitfield]
//! pub struct MyFourBytes {
//!     a: B1,
//!     b: B3,
//!     c: B4,
//!     d: B24,
//! }
//! ```
//! ```no_compile
//!                                least significant bit of third byte
//!                                  ┊           most significant
//!                                  ┊             ┊
//!                                  ┊             ┊
//! ║  first byte   ║  second byte  ║  third byte   ║  fourth byte  ║
//! ╟───────────────╫───────────────╫───────────────╫───────────────╢
//! ║▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒║
//! ╟─╫─────╫───────╫───────────────────────────────────────────────╢
//! ║a║  b  ║   c   ║                       d                       ║
//!                  ┊                                             ┊
//!                  ┊                                             ┊
//!                least significant bit of d         most significant
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs, rust_2018_idioms)]

pub mod error;
#[doc(hidden)]
pub mod private;

use self::error::{InvalidBitPattern, OutOfBounds};

/// Applicable to structs to turn their fields into compact bitfields.
///
/// # Generated API
///
/// By default this generates the following API:
///
/// - **Constructors:**
///
///     1. `new()`: Initializes all bits to 0 even if 0 bits may be invalid.
///        Note that invalid bit patterns are supported in that getters and setters will
///        be protecting accesses.
///
/// - **Getters:**
///
///     For every field `f` we generate the following getters:
///
///     1. `f()`: Returns the value of `f` and might panic
///        if the value contains an invalid bit pattern.
///     2. `f_or_err()`: Returns the value of `f` or an error
///        if the value contains an invalid bit pattern.
///
/// - **Setters:**
///
///     For every field `f` we generate the following setters:
///
///     1. `set_f(new_value)`: Sets the value of `f` to `new_value` and might panic
///        if `new_value` is out of bounds for the bit width of `f`.
///     2. `set_f_checked(new_value)`: Sets the value of `f` to `new` or returns an error
///        if `new_value` if out of bounds for the bit width of `f`.
///     3. `with_f(new_value)`: Similar to `set_f` but consumes and returns `Self`.
///        Primarily useful for method chaining.
///     4. `with_f_checked(new_value)`: Similar to `set_f_checked` but consumes and returns `Self`.
///        Primarily useful for method chaining.
///
/// - **Conversions:**
///
///     - `from_bytes(bytes)`: Allows to constructor the bitfield type from a fixed array of bytes.
///     - `into_bytes()`: Allows to convert the bitfield into its underlying byte representation.
///
/// # Parameters
///
/// The following parameters for the `#[bitfield]` macro are supported:
///
/// ## Parameter: `bytes = N`
///
/// This ensures at compilation time that the resulting `#[bitfield]` struct consists of
/// exactly `N` bytes. Yield a compilation error if this does not hold true.
///
/// ### Example
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #[bitfield(bytes = 2)]
/// pub struct SingedInt {
///     sign: bool, //  1 bit
///     value: B15, // 15 bits
/// }
/// ```
///
/// ## Parameter: `filled: bool`
///
/// If `filled` is `true` ensures that the `#[bitfield]` struct defines all bits and
/// therefore has a bitwidth that is divisible by 8. If `filled` is `false` ensures the
/// exact opposite.
///
/// The default value is: `true`
///
/// ### Example
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #[bitfield(filled = false)]
/// pub struct Package {
///     is_received: bool, // 1 bit
///     is_alive: bool,    // 1 bit
///     status: B2,        // 2 bits
/// }
/// ```
///
/// ## Parameter: `bits = N`
///
/// With the `bits: int` parameter it is possible to control the targeted bit width of
/// a `#[bitfield]` annoated struct. Using `bits = N` guarantees that the resulting bitfield
/// struct will have a bit width of exactly `N`.
///
/// ### Example 1
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #[bitfield(bits = 16)]
/// pub struct Package {
///     is_received: bool, // 1 bit
///     is_alive: bool,    // 1 bit
///     status: B14,       // 14 bits
/// }
/// ```
///
/// ### Example 2
///
/// The `bits: int` parameter is especially useful when using this in conjunction with
/// `#[derive(BitfieldSpecifier)]` and `filled = false` as shown in the below example.
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #[bitfield(bits = 5)]
/// #[derive(BitfieldSpecifier)]
/// pub struct Package {
///     is_received: bool, // 1 bit
///     is_alive: bool,    // 1 bit
///     status: B3,        // 3 bits
/// }
/// ```
///
/// ## Field Parameter: `#[bits = N]`
///
/// To ensure at compile time that a field of a `#[bitfield]` struct has a bit width of exactly
/// `N` a user may add `#[bits = N]` to the field in question.
///
/// ### Example
///
/// ```
/// # use modular_bitfield::prelude::*;
/// # #[bitfield(filled = false)]
/// # #[derive(BitfieldSpecifier)]
/// # pub struct Header {
/// #     is_received: bool, // 1 bit
/// #     is_alive: bool,    // 1 bit
/// #     status: B2,        // 2 bits
/// # }
/// #[bitfield]
/// pub struct Base {
///     #[bits = 4]
///     header: Header, //  4 bits
///     content: B28,   // 28 bits
/// }
/// ```
///
/// ## Field Parameter: `#[skip(..)]`
///
/// It is possible to skip the entire code generation for getters or setters with the `#[skip]`
/// field attribute.
/// This is useful if a field just needs to be read or written exclusively. Skipping both
/// setters and getters is useful if you want to have undefined blocks within your bitfields.
///
/// ### Example
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #[bitfield]
/// pub struct Sparse {
///     #[skip(getters)]
///     no_getters: B4,
///     #[skip(setters)]
///     no_setters: B4,
///     #[skip]
///     skipped_entirely: B4,
///     #[skip(getters, setters)]
///     skipped_entirely_2: B2,
///     #[skip(getters)] #[skip(setters)]
///     skipped_entirely_2: B2,
/// }
/// ```
///
/// ### Trick: Wildcards
///
/// If you are completely uninterested in a field of a bitfield, for example when specifying
/// some undefined bits in your bitfield you can use double wildcards as their names:
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #[bitfield]
/// pub struct Sparse {
///     #[skip] __: B10,
///     a: bool,
///     #[skip] __: B10,
///     b: bool,
///     #[skip] __: B10,
/// }
/// ```
///
/// # Features
///
/// ## Support: `#[derive(BitfieldSpecifier)]`
///
/// If a `#[bitfield]` struct is annotated with a `#[derive(BitfieldSpecifier)]` attribute
/// an implementation of the `Specifier` trait will be generated for it. This has the effect
/// that the bitfield struct itself can be used as the type of a field of another bitfield type.
///
/// This feature is limited to bitfield types that have a total bit width of 128 bit or fewer.
/// This restriction is ensured at compile time.
///
/// ### Example
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #[bitfield(filled = false)]
/// #[derive(BitfieldSpecifier)]
/// pub struct Header {
///     is_received: bool, // 1 bit
///     is_alive: bool,    // 1 bit
///     status: B2,        // 2 bits
/// }
/// ```
///
/// Now the above `Header` bitfield type can be used in yet another `#[bitfield]` annotated type:
///
/// ```
/// # use modular_bitfield::prelude::*;
/// # #[bitfield(filled = false)]
/// # #[derive(BitfieldSpecifier)]
/// # pub struct Header {
/// #     is_received: bool, // 1 bit
/// #     is_alive: bool,    // 1 bit
/// #     status: B2,        // 2 bits
/// # }
/// #[bitfield]
/// pub struct Base {
///     header: Header, //  4 bits
///     content: B28,   // 28 bits
/// }
/// ```
///
/// ## Support: `#[derive(Debug)]`
///
/// If a `#[derive(Debug)]` is found by the `#[bitfield]` a naturally formatting implementation
/// is going to be generated that clearly displays all the fields and their values as the user
/// would expect.
/// Also invalid bit patterns for fields are clearly displayed under this implementation.
///
/// ### Example
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #[bitfield]
/// #[derive(Debug)]
/// pub struct Package {
///     is_received: bool, // 1 bit
///     is_alive: bool,    // 1 bit
///     status: B6,        // 6 bits
/// }
///
/// let package = Package::new()
///     .with_is_received(false)
///     .with_is_alive(true)
///     .with_status(3);
/// println!("{:?}", package);
/// assert_eq!(
///     format!("{:?}", package),
///     "Package { is_received: false, is_alive: true, status: 3 }",
/// );
/// ```
///
/// ## Support: `#[repr(uN)]`
///
/// It is possible to additionally annotate a `#[bitfield]` annotated struct with `#[repr(uN)]`
/// where `uN` is one of `u8`, `u16`, `u32`, `u64` or `u128` in order to make it conveniently
/// interchangeable with such an unsigned integer value.
///
/// As an effect to the user this implements `From` implementations between the chosen primitive
/// and the bitfield as well as ensuring at compile time that the bit width of the bitfield struct
/// matches the bit width of the primitive.
///
/// ### Example
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #[bitfield]
/// #[repr(u16)]
/// pub struct SignedU16 {
///     sign: bool,     //  1 bit
///     abs_value: B15, // 15 bits
/// }
///
/// let sint = SignedU16::from(0b0111_0001);
/// assert_eq!(sint.sign(), true);
/// assert_eq!(sint.abs_value(), 0b0011_1000);
/// assert_eq!(u16::from(sint), 0b0111_0001_u16);
/// ```
pub use modular_bitfield_impl::bitfield;

/// Derive macro for Rust `enums` to implement `Specifier` trait.
///
/// This allows such an enum to be used as a field of a `#[bitfield]` struct.
/// The annotated enum must not have any variants with associated data and
/// by default must have a number of variants that is equal to the power of 2.
///
/// If a user wants to circumvent the latter restriction they can add
/// `#[bits = N]` below the `#[derive(BitfieldSpecifier)]` line in order to
/// signal to the code generation that the enum may have a relaxed number
/// of variants.
///
/// # Example
///
/// ## Example: Basic Usage
///
/// In the following we define a `MaybeWeekday` enum that lists all weekdays
/// as well as an invalid day so that we have a power-of-two number of variants.
///
/// ```
/// use modular_bitfield::prelude::*;
///
/// #[derive(BitfieldSpecifier)]
/// pub enum Weekday {
///     Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday, None
/// }
/// ```
///
/// ## Example: `#[bits = N]`
///
/// If we want to get rid of the `None` variant we need to add `#[bits = 3]`:
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #
/// #[derive(BitfieldSpecifier)]
/// #[bits = 3]
/// pub enum Weekday {
///     Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday
/// }
/// ```
///
/// ## Example: Discriminants
///
/// It is possible to explicitly assign discriminants to some of the days.
/// In our case this is useful since our week starts at sunday:
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #
/// #[derive(BitfieldSpecifier)]
/// #[bits = 3]
/// pub enum Weekday {
///     Monday = 1,
///     Tuesday = 2,
///     Wednesday = 3,
///     Thursday = 4,
///     Friday = 5,
///     Saturday = 6,
///     Sunday = 0,
/// }
/// ```
///
/// ## Example: Use in `#[bitfield]`
///
/// Given the above `Weekday` enum that starts at `Sunday` and uses 3 bits in total
/// we can now use it in a `#[bitfield]` annotated struct as follows:
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #
/// # #[derive(BitfieldSpecifier)]
/// # #[bits = 3]
/// # pub enum Weekday {
/// #     Monday = 1,
/// #     Tuesday = 2,
/// #     Wednesday = 3,
/// #     Thursday = 4,
/// #     Friday = 5,
/// #     Saturday = 6,
/// #     Sunday = 0,
/// # }
/// #[bitfield]
/// pub struct MeetingTimeSlot {
///     day: Weekday,
///     from: B6,
///     to: B6,
///     expired: bool,
/// }
/// ```
///
/// The above `MeetingTimeSlot` uses exactly 16 bits and defines our `Weekday` enum as
/// compact `day` bitfield. The `from` and `to` require 6 bits each and finally the
/// `expired` flag requires a single bit.
///
/// ## Example: Interacting
///
/// A user can interact with the above `MeetingTimeSlot` and `Weekday` definitions in
/// the following ways:
///
/// ```
/// # use modular_bitfield::prelude::*;
/// #
/// # #[derive(BitfieldSpecifier, Debug, PartialEq)]
/// # #[bits = 3]
/// # pub enum Weekday {
/// #     Monday = 1,
/// #     Tuesday = 2,
/// #     Wednesday = 3,
/// #     Thursday = 4,
/// #     Friday = 5,
/// #     Saturday = 6,
/// #     Sunday = 0,
/// # }
/// # #[bitfield]
/// # pub struct MeetingTimeSlot {
/// #     day: Weekday,
/// #     from: B6,
/// #     to: B6,
/// #     expired: bool,
/// # }
/// #
/// let mut slot = MeetingTimeSlot::new()
///     .with_day(Weekday::Friday)
///     .with_from(14) // 14:00 CEST
///     .with_to(15); // 15:00 CEST
/// assert_eq!(slot.day(), Weekday::Friday);
/// assert_eq!(slot.from(), 14);
/// assert_eq!(slot.to(), 15);
/// assert!(!slot.expired());
/// ```
pub use modular_bitfield_impl::BitfieldSpecifier;

/// The prelude: `use modular_bitfield::prelude::*;`
pub mod prelude {
    pub use super::{bitfield, specifiers::*, BitfieldSpecifier, Specifier};
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
/// derive from `BitfieldSpecifier`.
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
