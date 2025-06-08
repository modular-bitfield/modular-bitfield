//! Utility functions for enum data variants implementation

/// Calculate discriminant bits needed for a given number of variants
/// 
/// This determines the minimum number of bits needed to represent `n` distinct values.
/// Uses the same logic as existing bitfield_specifier.rs but works for non-power-of-2 counts.
/// 
/// For example:
/// - 1 variant needs 0 bits (but we enforce at least 1 for consistency)
/// - 2 variants need 1 bit
/// - 3-4 variants need 2 bits
/// - 5-8 variants need 3 bits
pub fn calculate_discriminant_bits(variant_count: usize) -> usize {
    if variant_count <= 1 {
        // Even with 1 variant, use 1 bit for consistency with existing patterns
        1
    } else {
        // Use the same approach as existing code: next_power_of_two().trailing_zeros()
        // This automatically handles non-power-of-2 counts correctly
        variant_count.next_power_of_two().trailing_zeros() as usize
    }
}

/// Pack discriminant and data into a combined value for enum variants
/// 
/// Layout: [discriminant_bits][data_bits] where discriminant goes in high bits
/// This uses a simple approach suitable for small to medium bit counts.
/// 
/// # Arguments
/// * `discriminant` - The discriminant value (must fit in discriminant_bits)  
/// * `data_value` - The data value as integer (must fit in data_bits)
/// * `discriminant_bits` - Number of bits for discriminant
/// * `data_bits` - Number of bits for data
/// 
/// # Returns
/// Combined value with discriminant in high bits, data in low bits
pub fn pack_discriminant_and_data_simple(
    discriminant: usize,
    data_value: u64,
    _discriminant_bits: usize,
    data_bits: usize,
) -> u64 {
    // Mask data to ensure it fits in data_bits
    let data_mask = if data_bits >= 64 { u64::MAX } else { (1u64 << data_bits) - 1 };
    let masked_data = data_value & data_mask;
    
    // Combine: discriminant in high bits, data in low bits  
    ((discriminant as u64) << data_bits) | masked_data
}

/// Unpack discriminant and data from a combined value
/// 
/// Returns (discriminant, data_value)
/// 
/// # Arguments
/// * `combined` - The packed value
/// * `discriminant_bits` - Number of bits for discriminant
/// * `data_bits` - Number of bits for data
pub fn unpack_discriminant_and_data_simple(
    combined: u64,
    discriminant_bits: usize,
    data_bits: usize,
) -> (usize, u64) {
    // Extract data (lower bits)
    let data_mask = if data_bits >= 64 { u64::MAX } else { (1u64 << data_bits) - 1 };
    let data_value = combined & data_mask;
    
    // Extract discriminant (upper bits)
    let discriminant_mask = if discriminant_bits >= 64 { u64::MAX } else { (1u64 << discriminant_bits) - 1 };
    let discriminant = ((combined >> data_bits) & discriminant_mask) as usize;
    
    (discriminant, data_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_discriminant_bits() {
        assert_eq!(calculate_discriminant_bits(1), 1);  // 1 variant = 1 discriminant bit (forced)
        assert_eq!(calculate_discriminant_bits(2), 1);  // 2 variants = 1 discriminant bit
        assert_eq!(calculate_discriminant_bits(3), 2);  // 3 variants = 2 discriminant bits
        assert_eq!(calculate_discriminant_bits(4), 2);  // 4 variants = 2 discriminant bits
        assert_eq!(calculate_discriminant_bits(5), 3);  // 5 variants = 3 discriminant bits
        assert_eq!(calculate_discriminant_bits(8), 3);  // 8 variants = 3 discriminant bits
        assert_eq!(calculate_discriminant_bits(9), 4);  // 9 variants = 4 discriminant bits
        assert_eq!(calculate_discriminant_bits(16), 4); // 16 variants = 4 discriminant bits
    }

    #[test]
    fn test_pack_unpack_simple() {
        // Test case: 2 discriminant bits + 6 data bits = 8 total bits
        let discriminant = 2; // 0b10
        let data_value = 0b111100u64; // 6 bits of data
        let discriminant_bits = 2;
        let data_bits = 6;
        
        let packed = pack_discriminant_and_data_simple(
            discriminant, 
            data_value, 
            discriminant_bits, 
            data_bits
        );
        
        // Expected: discriminant (10) in upper 2 bits, data (111100) in lower 6 bits
        // Result: 0b10111100 = 0xBC
        assert_eq!(packed, 0b10111100);
        
        let (unpacked_discriminant, unpacked_data) = unpack_discriminant_and_data_simple(
            packed, 
            discriminant_bits, 
            data_bits
        );
        
        assert_eq!(unpacked_discriminant, discriminant);
        assert_eq!(unpacked_data, data_value);
    }

    #[test]
    fn test_pack_unpack_larger() {
        // Test case: 3 discriminant bits + 13 data bits = 16 total bits
        let discriminant = 5; // 0b101
        let data_value = 0x1234u64; // 13 bits of data (will be masked)
        let discriminant_bits = 3;
        let data_bits = 13;
        
        let packed = pack_discriminant_and_data_simple(
            discriminant, 
            data_value, 
            discriminant_bits, 
            data_bits
        );
        
        let (unpacked_discriminant, unpacked_data) = unpack_discriminant_and_data_simple(
            packed, 
            discriminant_bits, 
            data_bits
        );
        
        assert_eq!(unpacked_discriminant, discriminant);
        
        // Data should be masked to 13 bits
        let expected_data = data_value & ((1u64 << data_bits) - 1);
        assert_eq!(unpacked_data, expected_data);
    }

    #[test]
    fn test_data_masking() {
        // Test that data gets properly masked when it's too large
        let discriminant = 1;
        let data_value = 0xFFFFu64; // 16 bits, but we only want 8
        let discriminant_bits = 2;
        let data_bits = 8;
        
        let packed = pack_discriminant_and_data_simple(
            discriminant, 
            data_value, 
            discriminant_bits, 
            data_bits
        );
        
        let (unpacked_discriminant, unpacked_data) = unpack_discriminant_and_data_simple(
            packed, 
            discriminant_bits, 
            data_bits
        );
        
        assert_eq!(unpacked_discriminant, discriminant);
        assert_eq!(unpacked_data, 0xFF); // Should be masked to 8 bits
    }
}