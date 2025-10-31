use crate::{
    error::{InvalidBitPattern, OutOfBounds},
    Specifier,
};

use core::num::NonZero;

impl Specifier for bool {
    const BITS: usize = 1;
    type Bytes = u8;
    type InOut = bool;

    #[inline]
    fn into_bytes(input: Self::InOut) -> Result<Self::Bytes, OutOfBounds> {
        Ok(input.into())
    }

    #[inline]
    fn from_bytes(bytes: Self::Bytes) -> Result<Self::InOut, InvalidBitPattern<Self::Bytes>> {
        match bytes {
            0 => Ok(false),
            1 => Ok(true),
            invalid_bytes => Err(InvalidBitPattern::new(invalid_bytes)),
        }
    }
}

macro_rules! impl_specifier_for_primitive {
    ( $( ($prim:ty: $bits:literal) ),* $(,)? ) => {
        $(
            impl Specifier for $prim {
                const BITS: usize = $bits;
                type Bytes = $prim;
                type InOut = $prim;

                #[inline]
                fn into_bytes(input: Self::InOut) -> Result<Self::Bytes, OutOfBounds> {
                    Ok(input)
                }

                #[inline]
                fn from_bytes(bytes: Self::Bytes) -> Result<Self::InOut, InvalidBitPattern<Self::Bytes>> {
                    Ok(bytes)
                }
            }

            impl Specifier for NonZero<$prim> {
                const BITS: usize = $bits;
                type Bytes = $prim;
                type InOut = NonZero<$prim>;

                #[inline]
                fn into_bytes(input: Self::InOut) -> Result<Self::Bytes, OutOfBounds> {
                    Ok(input.get())
                }

                #[inline]
                fn from_bytes(bytes: Self::Bytes) -> Result<Self::InOut, InvalidBitPattern<Self::Bytes>> {
                    NonZero::<$prim>::new(bytes).ok_or_else(|| InvalidBitPattern { invalid_bytes: bytes })
                }
            }

            impl Specifier for Option<NonZero<$prim>> {
                const BITS: usize = $bits;
                type Bytes = $prim;
                type InOut = Option<NonZero<$prim>>;

                #[inline]
                fn into_bytes(input: Self::InOut) -> Result<Self::Bytes, OutOfBounds> {
                    match input {
                        Some(x) => Ok(x.get()),
                        None => Ok(0),
                    }
                }

                #[inline]
                fn from_bytes(bytes: Self::Bytes) -> Result<Self::InOut, InvalidBitPattern<Self::Bytes>> {
                    Ok(NonZero::<$prim>::new(bytes))
                }
            }
        )*
    };
}
impl_specifier_for_primitive!(
    (u8: 8),
    (u16: 16),
    (u32: 32),
    (u64: 64),
    (u128: 128),
);
