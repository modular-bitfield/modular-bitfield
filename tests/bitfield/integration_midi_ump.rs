//! MIDI UMP (Universal MIDI Packet) integration test
//!
//! This demonstrates real-world usage of variable_bits functionality for implementing
//! the MIDI 2.0 Universal MIDI Packet format, which has variable message sizes:
//! - 32-bit: Type 0 (Utility), Type 1 (System Real Time), Type 2 (MIDI 1.0 Channel Voice)
//! - 64-bit: Type 3 (Data messages including System Exclusive)
//! - 128-bit: Type 5 (Data messages)
//!
//! NOTE: The current modular-bitfield API supports variable bits only for enums,
//! not for bitfield structs. The approach shown here uses a variable bits enum
//! directly to represent UMP messages.

// Clippy's semicolon_if_nothing_returned lint incorrectly triggers on enum variants with doc comments

use modular_bitfield::prelude::*;

// =============================================================================
// VARIABLE UMP MESSAGE ENUM
// =============================================================================

/// MIDI UMP message with variable size based on message type
///
/// This enum represents the complete UMP message including the message type
/// discriminator. The different variants have different sizes:
/// - Utility32 and System32: 32 bits total
/// - `SysEx64`: 64 bits total
/// - `SysEx8_128`: 128 bits total
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[bits(32, 32, 64, 128)]
enum UmpMessage {
    #[discriminant = 0]
    Utility32(u32),
    #[discriminant = 1]
    System32(u32),
    #[discriminant = 3]
    SysEx64(u64),
    #[discriminant = 5]
    SysEx8_128(u128),
}

// =============================================================================
// MESSAGE STRUCTURE DEFINITIONS
// =============================================================================

/// Utility messages (Type 0) - 32 bits total
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UtilityMessage {
    message_type: B4, // Always 0 for utility
    group: B4,        // MIDI group (0-15)
    status: B16,      // Utility status
    data: B8,         // Utility data/timestamp
}

/// System messages (Type 1) - 32 bits total
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemMessage {
    message_type: B4, // Always 1 for system
    group: B4,        // MIDI group (0-15)
    status: B8,       // System status byte
    data1: B8,        // First data byte
    data2: B8,        // Second data byte
}

/// System Exclusive messages (Type 3) - 64 bits total
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SysExMessage {
    message_type: B4, // Always 3 for SysEx
    group: B4,        // MIDI group (0-15)
    status: B8,       // SysEx status (0x00=complete, 0x01=start, 0x02=continue, 0x03=end)
    num_bytes: B8,    // Number of SysEx bytes (1-6)
    sysex_data: B40,  // Up to 5 bytes of SysEx data
}

/// System Exclusive 8 messages (Type 5) - 128 bits total
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SysEx8Message {
    message_type: B4, // Always 5 for SysEx8
    group: B4,        // MIDI group (0-15)
    status: B8,       // SysEx8 status
    num_bytes: B8,    // Number of SysEx bytes (1-14)
    stream_id: B8,    // Stream ID
    sysex_data: B96,  // Up to 12 bytes of SysEx data
}

// =============================================================================
// UMP MESSAGE CONSTRUCTION HELPERS
// =============================================================================

impl UmpMessage {
    /// Create a NOOP utility message (Type 0x0, Status 0x0000)
    fn noop(group: u8) -> Self {
        let msg = UtilityMessage::new()
            .with_message_type(0)
            .with_group(group)
            .with_status(0x0000)
            .with_data(0x00);

        UmpMessage::Utility32(u32::from_le_bytes(msg.into_bytes()))
    }

    /// Create a JR Clock utility message (Type 0x0, Status 0x0001)
    fn jr_clock(group: u8, timestamp: u8) -> Self {
        let msg = UtilityMessage::new()
            .with_message_type(0)
            .with_group(group)
            .with_status(0x0001)
            .with_data(timestamp);

        UmpMessage::Utility32(u32::from_le_bytes(msg.into_bytes()))
    }

    /// Create a MIDI Time Code system message (Type 0x1, Status 0xF1)
    fn midi_time_code(group: u8, time_code: u8) -> Self {
        let msg = SystemMessage::new()
            .with_message_type(1)
            .with_group(group)
            .with_status(0xF1)
            .with_data1(time_code)
            .with_data2(0x00);

        UmpMessage::System32(u32::from_le_bytes(msg.into_bytes()))
    }

    /// Create a Song Position Pointer system message (Type 0x1, Status 0xF2)
    fn song_position(group: u8, position: u16) -> Self {
        let lsb = (position & 0x7F) as u8;
        let msb = ((position >> 7) & 0x7F) as u8;
        let msg = SystemMessage::new()
            .with_message_type(1)
            .with_group(group)
            .with_status(0xF2)
            .with_data1(lsb)
            .with_data2(msb);

        UmpMessage::System32(u32::from_le_bytes(msg.into_bytes()))
    }

    /// Create a System Exclusive in 1 UMP message (Type 0x3)
    fn sysex_in_1_ump(group: u8, sysex_data: &[u8]) -> Self {
        let num_bytes = sysex_data.len().min(5) as u8;
        let mut data_u64 = 0u64;
        for (i, &byte) in sysex_data.iter().take(5).enumerate() {
            data_u64 |= (byte as u64) << (i * 8);
        }

        let msg = SysExMessage::new()
            .with_message_type(3)
            .with_group(group)
            .with_status(0x00) // Complete in 1 UMP
            .with_num_bytes(num_bytes)
            .with_sysex_data(data_u64);

        UmpMessage::SysEx64(u64::from_le_bytes(msg.into_bytes()))
    }

    /// Create a System Exclusive Start message (Type 0x3)
    fn sysex_start(group: u8, sysex_data: &[u8]) -> Self {
        let num_bytes = sysex_data.len().min(5) as u8;
        let mut data_u64 = 0u64;
        for (i, &byte) in sysex_data.iter().take(5).enumerate() {
            data_u64 |= (byte as u64) << (i * 8);
        }

        let msg = SysExMessage::new()
            .with_message_type(3)
            .with_group(group)
            .with_status(0x01) // Start
            .with_num_bytes(num_bytes)
            .with_sysex_data(data_u64);

        UmpMessage::SysEx64(u64::from_le_bytes(msg.into_bytes()))
    }

    /// Create a System Exclusive 8 in 1 UMP message (Type 0x5)
    fn sysex8_in_1_ump(group: u8, stream_id: u8, sysex_data: &[u8]) -> Self {
        let num_bytes = sysex_data.len().min(12) as u8;
        let mut data_u128 = 0u128;
        for (i, &byte) in sysex_data.iter().take(12).enumerate() {
            data_u128 |= (byte as u128) << (i * 8);
        }

        let msg = SysEx8Message::new()
            .with_message_type(5)
            .with_group(group)
            .with_status(0x00) // Complete in 1 UMP
            .with_num_bytes(num_bytes)
            .with_stream_id(stream_id)
            .with_sysex_data(data_u128);

        UmpMessage::SysEx8_128(u128::from_le_bytes(msg.into_bytes()))
    }

    /// Create a System Exclusive 8 Start message (Type 0x5)
    fn sysex8_start(group: u8, stream_id: u8, sysex_data: &[u8]) -> Self {
        let num_bytes = sysex_data.len().min(12) as u8;
        let mut data_u128 = 0u128;
        for (i, &byte) in sysex_data.iter().take(12).enumerate() {
            data_u128 |= (byte as u128) << (i * 8);
        }

        let msg = SysEx8Message::new()
            .with_message_type(5)
            .with_group(group)
            .with_status(0x01) // Start
            .with_num_bytes(num_bytes)
            .with_stream_id(stream_id)
            .with_sysex_data(data_u128);

        UmpMessage::SysEx8_128(u128::from_le_bytes(msg.into_bytes()))
    }

    /// Get the message type from a UMP message
    fn message_type(&self) -> u8 {
        match self {
            UmpMessage::Utility32(data) => (data & 0xF) as u8,
            UmpMessage::System32(data) => (data & 0xF) as u8,
            UmpMessage::SysEx64(data) => (data & 0xF) as u8,
            UmpMessage::SysEx8_128(data) => (data & 0xF) as u8,
        }
    }
}

// =============================================================================
// TESTS: 32-BIT MESSAGES
// =============================================================================

#[test]
fn test_32bit_noop_utility_message() {
    // Type 0: NOOP utility message
    let noop = UmpMessage::noop(5);

    // Verify basic properties
    assert_eq!(noop.message_type(), 0);
    assert_eq!(noop.discriminant(), 0);
    assert_eq!(noop.size(), 32);

    // Serialize and verify format
    let bytes = <UmpMessage as Specifier>::into_bytes(noop).unwrap();
    let word = bytes;
    let mt = word & 0xF;
    let gr = (word >> 4) & 0xF;
    let status = (word >> 8) & 0xFFFF;
    let data = (word >> 24) & 0xFF;

    assert_eq!(mt, 0); // Message Type 0
    assert_eq!(gr, 5); // Group 5
    assert_eq!(status, 0x0000); // NOOP status
    assert_eq!(data, 0x00); // Zero data

    // Test round-trip serialization
    let reconstructed = UmpMessage::from_discriminant_and_bytes(0, bytes).unwrap();
    assert_eq!(noop, reconstructed);

    // Validate using helper methods
    assert_eq!(UmpMessage::size_for_discriminant(0), Some(32));
}

#[test]
fn test_32bit_jr_clock_utility_message() {
    // Type 0: JR Clock utility message with timestamp
    let jr_clock = UmpMessage::jr_clock(2, 0x34);

    // Verify basic properties
    assert_eq!(jr_clock.discriminant(), 0);
    assert_eq!(jr_clock.size(), 32);

    // Serialize and verify format
    let bytes = <UmpMessage as Specifier>::into_bytes(jr_clock).unwrap();
    let word = bytes;
    let mt = word & 0xF;
    let gr = (word >> 4) & 0xF;
    let status = (word >> 8) & 0xFFFF;
    let timestamp = (word >> 24) & 0xFF;

    assert_eq!(mt, 0); // Message Type 0
    assert_eq!(gr, 2); // Group 2
    assert_eq!(status, 0x0001); // JR Clock status
    assert_eq!(timestamp, 0x34); // Timestamp

    // Test round-trip serialization
    let reconstructed = UmpMessage::from_discriminant_and_bytes(0, bytes).unwrap();
    assert_eq!(jr_clock, reconstructed);
}

#[test]
fn test_32bit_midi_time_code_system_message() {
    // Type 1: MIDI Time Code system message
    let mtc = UmpMessage::midi_time_code(1, 0x25);

    // Verify basic properties
    assert_eq!(mtc.discriminant(), 1);
    assert_eq!(mtc.size(), 32);

    // Serialize and verify format
    let bytes = <UmpMessage as Specifier>::into_bytes(mtc).unwrap();
    let word = bytes;
    let mt = word & 0xF;
    let gr = (word >> 4) & 0xF;
    let status = (word >> 8) & 0xFF;
    let time_code = (word >> 16) & 0xFF;
    let reserved = (word >> 24) & 0xFF;

    assert_eq!(mt, 1); // Message Type 1
    assert_eq!(gr, 1); // Group 1
    assert_eq!(status, 0xF1); // MIDI Time Code status
    assert_eq!(time_code, 0x25); // Time code
    assert_eq!(reserved, 0x00); // Reserved

    // Test round-trip serialization
    let reconstructed = UmpMessage::from_discriminant_and_bytes(1, bytes).unwrap();
    assert_eq!(mtc, reconstructed);
}

#[test]
fn test_32bit_song_position_system_message() {
    // Type 1: Song Position Pointer system message
    let spp = UmpMessage::song_position(3, 0x1ABC); // Position = 6844

    // Verify basic properties
    assert_eq!(spp.discriminant(), 1);
    assert_eq!(spp.size(), 32);

    // Serialize and verify format
    let bytes = <UmpMessage as Specifier>::into_bytes(spp).unwrap();
    let word = bytes;
    let mt = word & 0xF;
    let gr = (word >> 4) & 0xF;
    let status = (word >> 8) & 0xFF;
    let lsb = (word >> 16) & 0xFF;
    let msb = (word >> 24) & 0xFF;

    assert_eq!(mt, 1); // Message Type 1
    assert_eq!(gr, 3); // Group 3
    assert_eq!(status, 0xF2); // Song Position status
    assert_eq!(lsb, 0x3C); // Position LSB (6844 & 0x7F)
    assert_eq!(msb, 0x35); // Position MSB ((6844 >> 7) & 0x7F)

    // Test round-trip serialization
    let reconstructed = UmpMessage::from_discriminant_and_bytes(1, bytes).unwrap();
    assert_eq!(spp, reconstructed);
}

// =============================================================================
// TESTS: 64-BIT MESSAGES
// =============================================================================

#[test]
fn test_64bit_sysex_in_1_ump_message() {
    // Type 3: System Exclusive in 1 UMP
    let sysex_data = [0xF0, 0x43, 0x12, 0x15, 0xF7]; // Yamaha SysEx
    let sysex = UmpMessage::sysex_in_1_ump(1, &sysex_data);

    // Verify basic properties
    assert_eq!(sysex.discriminant(), 3);
    assert_eq!(sysex.size(), 64);

    // Serialize and verify format
    let bytes = <UmpMessage as Specifier>::into_bytes(sysex).unwrap();
    let word = bytes;
    let mt = word & 0xF;
    let gr = (word >> 4) & 0xF;
    let status = (word >> 8) & 0xFF;
    let num_bytes = (word >> 16) & 0xFF;
    let sysex_byte0 = (word >> 24) & 0xFF;
    let sysex_byte1 = (word >> 32) & 0xFF;

    assert_eq!(mt, 3); // Message Type 3
    assert_eq!(gr, 1); // Group 1
    assert_eq!(status, 0x00); // Complete in 1 UMP
    assert_eq!(num_bytes, 5); // 5 bytes of SysEx data
    assert_eq!(sysex_byte0, 0xF0); // First SysEx byte
    assert_eq!(sysex_byte1, 0x43); // Second SysEx byte

    // Test round-trip serialization
    let reconstructed = UmpMessage::from_discriminant_and_bytes(3, bytes).unwrap();
    assert_eq!(sysex, reconstructed);

    // Validate using helper methods
    assert_eq!(UmpMessage::size_for_discriminant(3), Some(64));
}

#[test]
fn test_64bit_sysex_start_message() {
    // Type 3: System Exclusive Start message
    let sysex_data = [0xF0, 0x7F, 0x00, 0x04, 0x01]; // Universal Real Time
    let sysex_start = UmpMessage::sysex_start(2, &sysex_data);

    // Verify basic properties
    assert_eq!(sysex_start.discriminant(), 3);
    assert_eq!(sysex_start.size(), 64);

    // Serialize and verify format
    let bytes = <UmpMessage as Specifier>::into_bytes(sysex_start).unwrap();
    let word = bytes;
    let mt = word & 0xF;
    let gr = (word >> 4) & 0xF;
    let status = (word >> 8) & 0xFF;
    let num_bytes = (word >> 16) & 0xFF;

    assert_eq!(mt, 3); // Message Type 3
    assert_eq!(gr, 2); // Group 2
    assert_eq!(status, 0x01); // Start
    assert_eq!(num_bytes, 5); // 5 bytes of SysEx data

    // Test round-trip serialization
    let reconstructed = UmpMessage::from_discriminant_and_bytes(3, bytes).unwrap();
    assert_eq!(sysex_start, reconstructed);
}

// =============================================================================
// TESTS: 128-BIT MESSAGES
// =============================================================================

#[test]
fn test_128bit_sysex8_in_1_ump_message() {
    // Type 5: System Exclusive 8 in 1 UMP
    let sysex_data = [
        0xF0, 0x43, 0x12, 0x15, 0x20, 0x25, 0x30, 0x35, 0x40, 0x45, 0x50, 0x55,
    ];
    let sysex8 = UmpMessage::sysex8_in_1_ump(0, 0x42, &sysex_data);

    // Verify basic properties
    assert_eq!(sysex8.discriminant(), 5);
    assert_eq!(sysex8.size(), 128);

    // Serialize and verify format
    let bytes = <UmpMessage as Specifier>::into_bytes(sysex8).unwrap();
    let word = bytes;
    let mt = word & 0xF;
    let gr = (word >> 4) & 0xF;
    let status = (word >> 8) & 0xFF;
    let num_bytes = (word >> 16) & 0xFF;
    let stream_id = (word >> 24) & 0xFF;

    assert_eq!(mt, 5); // Message Type 5
    assert_eq!(gr, 0); // Group 0
    assert_eq!(status, 0x00); // Complete in 1 UMP
    assert_eq!(num_bytes, 12); // 12 bytes of SysEx data
    assert_eq!(stream_id, 0x42); // Stream ID

    // Test round-trip serialization
    let reconstructed = UmpMessage::from_discriminant_and_bytes(5, bytes).unwrap();
    assert_eq!(sysex8, reconstructed);

    // Validate using helper methods
    assert_eq!(UmpMessage::size_for_discriminant(5), Some(128));
}

#[test]
fn test_128bit_sysex8_start_message() {
    // Type 5: System Exclusive 8 Start message
    let sysex_data = [
        0xF0, 0x7E, 0x00, 0x06, 0x02, 0x41, 0x56, 0x00, 0x19, 0x00, 0x01, 0x00,
    ];
    let sysex8_start = UmpMessage::sysex8_start(3, 0x7F, &sysex_data);

    // Verify basic properties
    assert_eq!(sysex8_start.discriminant(), 5);
    assert_eq!(sysex8_start.size(), 128);

    // Serialize and verify format
    let bytes = <UmpMessage as Specifier>::into_bytes(sysex8_start).unwrap();
    let word = bytes;
    let mt = word & 0xF;
    let gr = (word >> 4) & 0xF;
    let status = (word >> 8) & 0xFF;
    let num_bytes = (word >> 16) & 0xFF;
    let stream_id = (word >> 24) & 0xFF;

    assert_eq!(mt, 5); // Message Type 5
    assert_eq!(gr, 3); // Group 3
    assert_eq!(status, 0x01); // Start
    assert_eq!(num_bytes, 12); // 12 bytes of SysEx data
    assert_eq!(stream_id, 0x7F); // Stream ID

    // Test round-trip serialization
    let reconstructed = UmpMessage::from_discriminant_and_bytes(5, bytes).unwrap();
    assert_eq!(sysex8_start, reconstructed);
}

// =============================================================================
// TESTS: VARIABLE BITS FUNCTIONALITY
// =============================================================================

#[test]
fn test_ump_variable_bits_helpers() {
    // Test discriminant and size helper methods
    assert_eq!(UmpMessage::supported_discriminants(), &[0, 1, 3, 5]);
    assert_eq!(UmpMessage::supported_sizes(), &[32, 32, 64, 128]);

    // Test size lookup by discriminant
    assert_eq!(UmpMessage::size_for_discriminant(0), Some(32));
    assert_eq!(UmpMessage::size_for_discriminant(1), Some(32));
    assert_eq!(UmpMessage::size_for_discriminant(3), Some(64));
    assert_eq!(UmpMessage::size_for_discriminant(5), Some(128));
    assert_eq!(UmpMessage::size_for_discriminant(2), None);
    assert_eq!(UmpMessage::size_for_discriminant(4), None);

    // Test size method on instances
    let noop = UmpMessage::noop(0);
    let sysex = UmpMessage::sysex_in_1_ump(0, &[0xF0, 0x43, 0x12, 0x15, 0xF7]);
    let sysex8 = UmpMessage::sysex8_in_1_ump(0, 0x42, &[0xF0; 12]);

    assert_eq!(noop.size(), 32);
    assert_eq!(sysex.size(), 64);
    assert_eq!(sysex8.size(), 128);
}

#[test]
fn test_ump_message_type_extraction() {
    // Test extracting message type from different UMP messages

    // 32-bit utility message
    let noop = UmpMessage::noop(1);
    assert_eq!(noop.message_type(), 0); // Message Type 0

    // 32-bit system message
    let mtc = UmpMessage::midi_time_code(2, 0x25);
    assert_eq!(mtc.message_type(), 1); // Message Type 1

    // 64-bit SysEx message
    let sysex = UmpMessage::sysex_in_1_ump(2, &[0xF0, 0x43, 0x12]);
    assert_eq!(sysex.message_type(), 3); // Message Type 3

    // 128-bit SysEx8 message
    let sysex8 = UmpMessage::sysex8_in_1_ump(3, 0x42, &[0xF0, 0x7E]);
    assert_eq!(sysex8.message_type(), 5); // Message Type 5
}

#[test]
fn test_ump_bitfield_structure() {
    // Test that our bitfield structures work correctly

    // Test UtilityMessage
    let util = UtilityMessage::new()
        .with_message_type(0)
        .with_group(5)
        .with_status(0x1234)
        .with_data(0x56);

    assert_eq!(util.message_type(), 0);
    assert_eq!(util.group(), 5);
    assert_eq!(util.status(), 0x1234);
    assert_eq!(util.data(), 0x56);

    // Test SysExMessage
    let sysex = SysExMessage::new()
        .with_message_type(3)
        .with_group(2)
        .with_status(0x01)
        .with_num_bytes(5)
        .with_sysex_data(0x1234567890);

    assert_eq!(sysex.message_type(), 3);
    assert_eq!(sysex.group(), 2);
    assert_eq!(sysex.status(), 0x01);
    assert_eq!(sysex.num_bytes(), 5);
    assert_eq!(sysex.sysex_data(), 0x1234567890);
}

// =============================================================================
// DEMONSTRATION OF IDEAL API (NOT CURRENTLY SUPPORTED)
// =============================================================================

// The following is what an ideal API might look like if modular-bitfield
// supported variable-sized bitfield structs:
//
// ```rust
// #[bitfield(variable_bits = (32, 64, 128))]
// struct UmpMessage {
//     #[variant_discriminator]
//     message_type: B4,
//     #[variant_data]
//     data: UmpData,
// }
//
// #[derive(Specifier)]
// #[variable_bits(28, 60, 124)]  // Remaining bits after 4-bit discriminator
// enum UmpData {
//     #[discriminant = 0]
//     Utility(UtilityPayload),  // 28 bits
//     #[discriminant = 3]
//     SysEx(SysExPayload),      // 60 bits
//     #[discriminant = 5]
//     SysEx8(SysEx8Payload),    // 124 bits
// }
// ```
//
// This would allow the bitfield struct to automatically handle the variable
// sizing based on the discriminator value, similar to how Rust enums work
// at the language level.
