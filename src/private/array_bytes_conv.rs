use crate::private::SpecifierBytes;

pub const trait ArrayBytesConversion {
    type Array;
    type Bytes;

    fn bytes_into_array(bytes: Self::Bytes) -> Self::Array;
    fn array_into_bytes(bytes: Self::Array) -> Self::Bytes;
}

macro_rules! impl_array_bytes_conversion_for_prim {
    ( $( $prim:ty ),* ) => {
        $(
            impl const ArrayBytesConversion for [(); ::core::mem::size_of::<$prim>() * 8] {
                type Array = [u8; ::core::mem::size_of::<$prim>()];
                type Bytes = <Self as SpecifierBytes>::Bytes;

                fn bytes_into_array(bytes: Self::Bytes) -> Self::Array {
                    bytes.to_le_bytes()
                }

                fn array_into_bytes(bytes: Self::Array) -> Self::Bytes {
                    <[(); ::core::mem::size_of::<$prim>() * 8] as SpecifierBytes>::Bytes::from_le_bytes(bytes)
                }
            }
        )*
    };
}
impl_array_bytes_conversion_for_prim!(u8, u16, u32, u64, u128);

macro_rules! impl_array_bytes_conversion_for_size {
    ( $( $size:literal ),* ) => {
        $(
            impl const ArrayBytesConversion for [(); $size] {
                type Array = [u8; $size / 8];
                type Bytes = <Self as SpecifierBytes>::Bytes;

                #[inline]
                fn bytes_into_array(bytes: Self::Bytes) -> Self::Array {
                    let array = bytes.to_le_bytes();
                    // debug_assert!(array[($size / 8)..].iter().all(|&byte| byte == 0)); TODO: enable once supported, or convert to a loop.
                    let mut result = [0; $size / 8];
                    result.copy_from_slice(&array[0..($size / 8)]);
                    result
                }

                #[inline]
                fn array_into_bytes(bytes: Self::Array) -> Self::Bytes {
                    let array: Self::Array = bytes;
                    let mut result = [0; ::core::mem::size_of::<Self::Bytes>()];
                    result[0..($size / 8)].copy_from_slice(&array[..]);
                    <Self::Bytes>::from_le_bytes(result)
                }
            }
        )*
    };
}
impl_array_bytes_conversion_for_size!(24, 40, 48, 56, 72, 80, 88, 96, 104, 112, 120);
