use modular_bitfield::prelude::*;

#[test]
fn test_simple_variable_bits() {
    #[bitfield(variable_bits = (32, 64))]
    struct SimpleMessage {
        #[variant_discriminator]
        msg_type: B4,
        #[variant_data]
        data: MessageData,
    }

    #[derive(Specifier)]
    #[variable_bits = (32, 64)]
    #[discriminant_bits = 4]
    enum MessageData {
        #[discriminant = 0]
        Short(B28),
        #[discriminant = 1]
        Long(B60),
    }

    // Test basic compilation
    let msg = SimpleMessage::new();
    assert_eq!(msg.msg_type(), 0);
}

