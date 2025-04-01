use crate::{
    private::{PopBits, PopBuffer, PushBits, PushBuffer},
    Specifier,
};

/// Creates a new push buffer with all bits initialized to 0.
#[inline]
fn push_buffer<T>() -> PushBuffer<<T as Specifier>::Bytes>
where
    T: Specifier,
    PushBuffer<T::Bytes>: Default,
{
    <PushBuffer<<T as Specifier>::Bytes> as Default>::default()
}

#[doc(hidden)]
#[inline]
#[must_use]
pub fn read_specifier<T>(bytes: &[u8], offset: usize) -> <T as Specifier>::Bytes
where
    T: Specifier,
    PushBuffer<T::Bytes>: Default + PushBits,
{
    let end = offset + <T as Specifier>::BITS;
    let ls_byte = offset / 8; // compile-time
    let ms_byte = (end - 1) / 8; // compile-time

    // Truncation is always valid due to mod 8 value range
    #[allow(clippy::cast_possible_truncation)]
    let lsb_offset = (offset % 8) as u32; // compile-time
    #[allow(clippy::cast_possible_truncation)]
    let msb_offset = (end % 8) as u32; // compile-time
    let msb_offset = if msb_offset == 0 { 8 } else { msb_offset };

    let mut buffer = push_buffer::<T>();

    if lsb_offset == 0 && msb_offset == 8 {
        // Edge-case for whole bytes manipulation.
        for byte in bytes[ls_byte..=ms_byte].iter().rev() {
            buffer.push_bits(8, *byte);
        }
    } else {
        if ls_byte != ms_byte {
            // Most-significant byte
            buffer.push_bits(msb_offset, bytes[ms_byte]);
        }
        if ms_byte - ls_byte >= 2 {
            // Middle bytes
            for byte in bytes[(ls_byte + 1)..ms_byte].iter().rev() {
                buffer.push_bits(8, *byte);
            }
        }
        if ls_byte == ms_byte {
            buffer.push_bits(
                u32::try_from(<T as Specifier>::BITS).unwrap(),
                bytes[ls_byte] >> lsb_offset,
            );
        } else {
            buffer.push_bits(8 - lsb_offset, bytes[ls_byte] >> lsb_offset);
        }
    }
    buffer.into_bytes()
}

#[doc(hidden)]
#[inline]
pub fn write_specifier<T>(bytes: &mut [u8], offset: usize, new_val: <T as Specifier>::Bytes)
where
    T: Specifier,
    PopBuffer<T::Bytes>: PopBits,
{
    let end = offset + <T as Specifier>::BITS;
    let ls_byte = offset / 8; // compile-time
    let ms_byte = (end - 1) / 8; // compile-time

    // Truncation is always valid due to mod 8 value range
    #[allow(clippy::cast_possible_truncation)]
    let lsb_offset = (offset % 8) as u32; // compile-time
    #[allow(clippy::cast_possible_truncation)]
    let msb_offset = (end % 8) as u32; // compile-time
    let msb_offset = if msb_offset == 0 { 8 } else { msb_offset };

    let mut buffer = <PopBuffer<T::Bytes>>::from_bytes(new_val);

    if lsb_offset == 0 && msb_offset == 8 {
        // Edge-case for whole bytes manipulation.
        for byte in &mut bytes[ls_byte..=ms_byte] {
            *byte = buffer.pop_bits(8);
        }
    } else {
        // Least-significant byte
        let stays_same = bytes[ls_byte]
            & (if ls_byte == ms_byte && msb_offset != 8 {
                !((0x01 << msb_offset) - 1)
            } else {
                0u8
            } | ((0x01 << lsb_offset) - 1));
        let overwrite = buffer.pop_bits(8 - lsb_offset);
        bytes[ls_byte] = stays_same | (overwrite << lsb_offset);
        if ms_byte - ls_byte >= 2 {
            // Middle bytes
            for byte in &mut bytes[(ls_byte + 1)..ms_byte] {
                *byte = buffer.pop_bits(8);
            }
        }
        if ls_byte != ms_byte {
            // Most-significant byte
            if msb_offset == 8 {
                // We don't need to respect what was formerly stored in the byte.
                bytes[ms_byte] = buffer.pop_bits(msb_offset);
            } else {
                // All bits that do not belong to this field should be preserved.
                let stays_same = bytes[ms_byte] & !((0x01 << msb_offset) - 1);
                let overwrite = buffer.pop_bits(msb_offset);
                bytes[ms_byte] = stays_same | overwrite;
            }
        }
    }
}
