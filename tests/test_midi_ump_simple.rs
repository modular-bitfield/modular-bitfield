// Simplified MIDI UMP example showing variable struct usage
use modular_bitfield::prelude::*;

// Variable-size message types
#[derive(Debug, Clone, Copy, Specifier)]
#[variable_bits(28, 60, 124)] // 32-4=28, 64-4=60, 128-4=124
#[discriminant_bits = 4]
enum UmpData {
    #[discriminant = 0]
    Utility(u32), // 32-bit message
    #[discriminant = 3]
    SysEx(u64), // 64-bit message
    #[discriminant = 5]
    Extended(u128), // 128-bit message
}

// MIDI UMP message with variable size
#[bitfield(variable_bits = (32, 64, 128))]
#[derive(Debug, Clone, Copy)]
struct UmpMessage {
    #[variant_discriminator]
    message_type: B4,
    #[variant_data]
    data: UmpData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ump_message_sizes() {
        // Create messages of different sizes
        let msg_32 = UmpMessage::new_32bit();
        let msg_64 = UmpMessage::new_64bit();
        let msg_128 = UmpMessage::new_128bit();

        // Verify sizes
        assert_eq!(std::mem::size_of_val(&msg_32), 16); // Max size
        assert_eq!(std::mem::size_of_val(&msg_64), 16); // Max size
        assert_eq!(std::mem::size_of_val(&msg_128), 16); // Max size

        // All use the same struct size (128 bits = 16 bytes)
        assert_eq!(std::mem::size_of::<UmpMessage>(), 16);
    }

    #[test]
    fn test_ump_message_creation() {
        // Create a 32-bit utility message
        let mut msg = UmpMessage::new_32bit();
        msg.set_message_type(0); // Utility type

        // Convert to bytes
        let bytes = msg.into_bytes();
        assert_eq!(bytes.len(), 16); // Always max size

        // Test discriminator
        assert_eq!(msg.message_type(), 0);
    }
}
