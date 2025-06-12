use modular_bitfield::prelude::*;

// Simple variable-size enum for testing
#[derive(Debug, Clone, Copy, Specifier)]
#[variable_bits(4, 12)] // 8-4=4, 16-4=12
#[discriminant_bits = 4]
pub enum SimpleData {
    #[discriminant = 0]
    Small(u8),
    #[discriminant = 1]
    Large(u16),
}

// Variable struct using the enum
#[bitfield(variable_bits = (8, 16))]
#[derive(Debug, Clone, Copy)]
pub struct VariableMessage {
    #[variant_discriminator]
    msg_type: B4,
    #[variant_data]
    data: SimpleData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_struct_creation() {
        // Test that we can create instances
        let msg = VariableMessage::new();
        assert_eq!(msg.msg_type(), 0);

        // Test 8-bit constructor
        let msg_8bit = VariableMessage::new_8bit();
        assert_eq!(msg_8bit.msg_type(), 0);

        // Test 16-bit constructor
        let msg_16bit = VariableMessage::new_16bit();
        assert_eq!(msg_16bit.msg_type(), 0);
    }

    #[test]
    fn test_variable_struct_accessors() {
        let mut msg = VariableMessage::new();

        // Test setting discriminator
        msg.set_msg_type(1);
        assert_eq!(msg.msg_type(), 1);

        // Test conversion to bytes
        let bytes = msg.into_bytes();
        assert_eq!(bytes.len(), 2); // 16 bits = 2 bytes
    }

    #[test]
    fn test_variable_struct_sizes() {
        // Test that the struct reports correct sizes
        let msg = VariableMessage::new();

        // The struct should use max size (16 bits = 2 bytes)
        assert_eq!(std::mem::size_of::<VariableMessage>(), 2);

        // Test byte conversion maintains size
        let bytes = msg.into_bytes();
        assert_eq!(bytes.len(), 2);

        // Test from_bytes works
        let reconstructed = VariableMessage::from_bytes(bytes);
        assert_eq!(reconstructed.msg_type(), msg.msg_type());
    }
}

