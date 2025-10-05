use crate::{
    private::{PopBits, PopBuffer, PushBits, PushBuffer},
    Specifier,
};

/// Creates a new push buffer with all bits initialized to 0.
#[inline]
const fn push_buffer<T>() -> PushBuffer<<T as Specifier>::Bytes>
where
    T: Specifier,
    PushBuffer<T::Bytes>: const Default,
{
    <PushBuffer<<T as Specifier>::Bytes> as Default>::default()
}

#[doc(hidden)]
#[inline]
#[must_use]
pub const fn read_specifier<T>(bytes: &[u8], offset: usize) -> <T as Specifier>::Bytes
where
    T: const Specifier,
    PushBuffer<T::Bytes>: const Default + const PushBits,
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
        let mut bindex = ms_byte;
        while bindex >= ls_byte {
            buffer.push_bits(8, bytes[bindex]);
            if bindex == 0 { break; }
            bindex -= 1;
        }
    } else {
        if ls_byte != ms_byte {
            // Most-significant byte
            buffer.push_bits(msb_offset, bytes[ms_byte]);
        }
        if ms_byte - ls_byte >= 2 {
            // Middle bytes
            let mut bindex = ms_byte - 1;
            while bindex > ls_byte {
                buffer.push_bits(8, bytes[bindex]);
                bindex -= 1;
            }
        }
        if ls_byte == ms_byte {
            buffer.push_bits(
                <T as Specifier>::BITS as u32,
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
pub const fn write_specifier<T>(bytes: &mut [u8], offset: usize, new_val: <T as Specifier>::Bytes)
where
    T: const Specifier,
    PopBuffer<T::Bytes>: const PopBits,
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
        let mut bindex = ls_byte;
        while bindex <= ms_byte {
            bytes[bindex] = buffer.pop_bits(8);
            bindex += 1;
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
            let mut bindex = ls_byte + 1;
            while bindex < ms_byte {
                bytes[bindex] = buffer.pop_bits(8);
                bindex += 1;
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

    // TODO: Consider this further.
    core::mem::forget(buffer);
}
