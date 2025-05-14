pub(crate) mod private {
    /// Prevents internal traits from being implemented from dependencies.
    pub trait Sealed {}
}

macro_rules! impl_sealed_for {
    ( $($primitive:ty),* ) => {
        $(
            impl private::Sealed for $primitive {}
        )*
    }
}
impl_sealed_for!(bool, u8, u16, u32, u64, u128);

/// Helper trait to check whether the size of bitfield structs
/// is a multiple of 8 to form complete bytes.
pub trait TotalSizeIsMultipleOfEightBits: private::Sealed {}

/// Helper trait used to check whether a bitfield struct does not
/// fill its entire value space, e.g. has undefined bits.
pub trait TotalSizeIsNotMultipleOfEightBits: private::Sealed {}

/// Helper trait to improve compile error messages.
pub trait RenameSizeType: private::Sealed {
    type CheckType;
}

/// Helper type to sum up bit size of a bitfield at compile time.
pub struct TotalSize<T>(::core::marker::PhantomData<T>);

macro_rules! impl_total_size_for {
    ( $(($n:expr, $name:ident)),* ) => {
        $(
            pub enum $name {}
            impl private::Sealed for TotalSize<BitCount<$n>> {}
            impl private::Sealed for $name {}
            impl RenameSizeType for TotalSize<BitCount<$n>> {
                type CheckType = $name;
            }
        )*
    }
}

impl_total_size_for!(
    (0, ZeroMod8),
    (1, OneMod8),
    (2, TwoMod8),
    (3, ThreeMod8),
    (4, FourMod8),
    (5, FiveMod8),
    (6, SixMod8),
    (7, SevenMod8)
);

impl TotalSizeIsMultipleOfEightBits for ZeroMod8 {}

impl TotalSizeIsNotMultipleOfEightBits for OneMod8 {}
impl TotalSizeIsNotMultipleOfEightBits for TwoMod8 {}
impl TotalSizeIsNotMultipleOfEightBits for ThreeMod8 {}
impl TotalSizeIsNotMultipleOfEightBits for FourMod8 {}
impl TotalSizeIsNotMultipleOfEightBits for FiveMod8 {}
impl TotalSizeIsNotMultipleOfEightBits for SixMod8 {}
impl TotalSizeIsNotMultipleOfEightBits for SevenMod8 {}

/// Public facing trait implemented by bitfield structs in order to let the compiler
/// check if their sizes match a multiple of 8.
pub trait CheckTotalSizeMultipleOf8
where
    <Self::Size as RenameSizeType>::CheckType: TotalSizeIsMultipleOfEightBits,
{
    type Size: RenameSizeType;
}

/// Public facing trait implemented by bitfield structs in order to let the compiler
/// check if their sizes does not match a multiple of 8.
pub trait CheckTotalSizeIsNotMultipleOf8
where
    <Self::Size as RenameSizeType>::CheckType: TotalSizeIsNotMultipleOfEightBits,
{
    type Size: RenameSizeType;
}

/// Helper trait to check if an enum discriminant of a bitfield specifier
/// is within valid bounds.
pub trait DiscriminantInRange: private::Sealed {}

/// Helper trait to check if a `#[derive(Specifier)]` flagged bitfield
/// requires
/// at most 128 bits.
pub trait SpecifierHasAtMost128Bits: private::Sealed {}

/// Helper type to state that something is `true`.
///
/// # Note
///
/// Used for some compile time evaluation contexts.
pub enum True {}

/// Helper type to state that something is `false`.
///
/// # Note
///
/// Used for some compile time evaluation contexts.
pub enum False {}

impl private::Sealed for True {}
impl DiscriminantInRange for True {}
impl SpecifierHasAtMost128Bits for True {}
impl FillsUnalignedBits for True {}
impl DoesNotFillUnalignedBits for True {}

/// Helper trait to improve compile time error messages.
pub trait DispatchTrueFalse: private::Sealed {
    type Out;
}

/// Helper type for compile time evaluation of the number of bits.
pub struct BitCount<const N: usize>;

impl private::Sealed for BitCount<0> {}
impl DispatchTrueFalse for BitCount<0> {
    type Out = False;
}

// impl private::Sealed for [(); 1] {} // <-- Already implemented by `define_specifiers` macro!
impl DispatchTrueFalse for BitCount<1> {
    type Out = True;
}

/// Public facing trait that is implemented by bitfield specifiers to
/// let the compiler check if all its variant discriminants are within
/// valid bounds.
pub trait CheckDiscriminantInRange<A>
where
    <Self::CheckType as DispatchTrueFalse>::Out: DiscriminantInRange,
{
    type CheckType: DispatchTrueFalse;
}

/// Traits to check at compile-time if a `#[derive(Specifier)]` type requires
/// no more than 128 bits.
pub trait CheckSpecifierHasAtMost128Bits
where
    <Self::CheckType as DispatchTrueFalse>::Out: SpecifierHasAtMost128Bits,
{
    type CheckType: DispatchTrueFalse;
}

pub trait CheckFillsUnalignedBits
where
    <Self::CheckType as DispatchTrueFalse>::Out: FillsUnalignedBits,
{
    type CheckType: DispatchTrueFalse;
}

pub trait FillsUnalignedBits {}

pub trait CheckDoesNotFillUnalignedBits
where
    <Self::CheckType as DispatchTrueFalse>::Out: DoesNotFillUnalignedBits,
{
    type CheckType: DispatchTrueFalse;
}

pub trait DoesNotFillUnalignedBits {}
