//! Variable bits comprehensive benchmark comparing generated vs handwritten code.

#![allow(dead_code)]

mod utils;

use modular_bitfield::prelude::*;
use utils::*;

// Core enum implementations (large bit sizes)

// Generated implementation using large bit sizes
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[bits(32, 64, 128)]
pub enum GeneratedVariableEnum {
    #[discriminant = 0]
    Small(u32),
    #[discriminant = 1]
    Medium(u64),
    #[discriminant = 2]
    Large(u128),
}

// Handwritten implementation for comparison
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandwrittenVariableEnum {
    Small(u32),
    Medium(u64),
    Large(u128),
}

impl HandwrittenVariableEnum {
    #[inline]
    pub fn into_bytes(self) -> Result<u128, &'static str> {
        match self {
            Self::Small(val) => Ok(val as u128),
            Self::Medium(val) => Ok(val as u128),
            Self::Large(val) => Ok(val),
        }
    }

    #[inline]
    pub fn from_discriminant_and_bytes(
        discriminant: u16,
        bytes: u128,
    ) -> Result<Self, &'static str> {
        match discriminant {
            0 => Ok(Self::Small(bytes as u32)),
            1 => Ok(Self::Medium(bytes as u64)),
            2 => Ok(Self::Large(bytes)),
            _ => Err("Invalid discriminant value"),
        }
    }

    #[inline]
    pub fn discriminant(&self) -> u16 {
        match self {
            Self::Small(_) => 0,
            Self::Medium(_) => 1,
            Self::Large(_) => 2,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        match self {
            Self::Small(_) => 32,
            Self::Medium(_) => 64,
            Self::Large(_) => 128,
        }
    }

    #[inline]
    pub fn size_for_discriminant(discriminant: u16) -> Option<usize> {
        match discriminant {
            0 => Some(32),
            1 => Some(64),
            2 => Some(128),
            _ => None,
        }
    }
}

// Practical enum implementations (small bit sizes)

// Variable bits enum with smaller sizes for bitfield integration
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[bits(8, 16, 32)]
enum VariableData {
    #[discriminant = 0]
    Small(u8),
    #[discriminant = 1]
    Medium(u16),
    #[discriminant = 2]
    Large(u32),
}

// Handwritten reference implementation
#[derive(Debug, Clone, Copy, PartialEq)]
enum HandwrittenData {
    Small(u8),
    Medium(u16),
    Large(u32),
}

impl HandwrittenData {
    fn into_bytes(self) -> u32 {
        match self {
            Self::Small(val) => val as u32,
            Self::Medium(val) => val as u32,
            Self::Large(val) => val,
        }
    }

    fn discriminant(&self) -> u16 {
        match self {
            Self::Small(_) => 0,
            Self::Medium(_) => 1,
            Self::Large(_) => 2,
        }
    }

    fn size(&self) -> usize {
        match self {
            Self::Small(_) => 8,
            Self::Medium(_) => 16,
            Self::Large(_) => 32,
        }
    }

    fn size_for_discriminant(discriminant: u16) -> Option<usize> {
        match discriminant {
            0 => Some(8),
            1 => Some(16),
            2 => Some(32),
            _ => None,
        }
    }
}

// Variable enum with custom type names
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[bits(8, 16, 32)]
enum VariableDataCustomTypes {
    #[discriminant = 0]
    ByteValue(u8),
    #[discriminant = 1]
    WordValue(u16),
    #[discriminant = 2]
    DwordValue(u32),
}

// Traditional unit enum for baseline comparison
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[bits = 2]
enum SimpleEnum {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}

// Bitfield integration

// Bitfield struct using variable enum
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct VariableMessage {
    #[bits = 32]
    data: VariableData,
}

// Bitfield struct using simple enum for comparison
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct SimpleMessage {
    #[bits = 32]
    payload: u32,
}

// Bitfield struct using custom types variable enum
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct CustomTypesMessage {
    #[bits = 32]
    data: VariableDataCustomTypes,
}

// Handwritten struct for comparison
#[derive(Debug, Clone, Copy, PartialEq)]
struct HandwrittenMessage {
    msg_type: u8,
    data: HandwrittenData,
}

impl HandwrittenMessage {
    fn new() -> Self {
        Self {
            msg_type: 0,
            data: HandwrittenData::Small(0),
        }
    }

    fn data(&self) -> HandwrittenData {
        self.data
    }

    fn set_data(&mut self, value: HandwrittenData) {
        self.data = value;
    }

    fn into_bytes(self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0] = self.msg_type;
        let data_bytes = self.data.into_bytes();
        bytes[1..5].copy_from_slice(&data_bytes.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: [u8; 8]) -> Self {
        let msg_type = bytes[0];
        let data_bytes = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        let data = HandwrittenData::Large(data_bytes);
        Self { msg_type, data }
    }
}

// Core enum benchmarks (large sizes)

fn bench_core_enum_construction() {
    println!("\n=== Core Enum Construction (32/64/128 bits) ===");
    
    compare("Generated enum construction", &|| (), |_| {
        repeat(|| {
            black_box(GeneratedVariableEnum::Small(42));
            black_box(GeneratedVariableEnum::Medium(1234));
            black_box(GeneratedVariableEnum::Large(5678));
        });
    });
    
    compare("Handwritten enum construction", &|| (), |_| {
        repeat(|| {
            black_box(HandwrittenVariableEnum::Small(42));
            black_box(HandwrittenVariableEnum::Medium(1234));
            black_box(HandwrittenVariableEnum::Large(5678));
        });
    });
}

fn bench_core_enum_into_bytes() {
    println!("\n=== Core Enum Serialization (32/64/128 bits) ===");
    
    let gen_data = [
        GeneratedVariableEnum::Small(42),
        GeneratedVariableEnum::Medium(1234),
        GeneratedVariableEnum::Large(5678),
    ];
    
    compare("Generated into_bytes", &|| gen_data, |data| {
        repeat(|| {
            for item in data.iter().copied() {
                black_box(<GeneratedVariableEnum as Specifier>::into_bytes(item).unwrap());
            }
        });
    });
    
    let hw_data = [
        HandwrittenVariableEnum::Small(42),
        HandwrittenVariableEnum::Medium(1234),
        HandwrittenVariableEnum::Large(5678),
    ];
    
    compare("Handwritten into_bytes", &|| hw_data, |data| {
        repeat(|| {
            for item in data.iter().copied() {
                black_box(item.into_bytes().unwrap());
            }
        });
    });
}

fn bench_core_enum_helpers() {
    println!("\n=== Core Enum Helper Methods (32/64/128 bits) ===");
    
    let gen_data = GeneratedVariableEnum::Medium(1234);
    compare("Generated discriminant", &|| gen_data, |data| {
        repeat(|| {
            black_box(data.discriminant());
        });
    });
    
    let hw_data = HandwrittenVariableEnum::Medium(1234);
    compare("Handwritten discriminant", &|| hw_data, |data| {
        repeat(|| {
            black_box(data.discriminant());
        });
    });
    
    compare("Generated size", &|| gen_data, |data| {
        repeat(|| {
            black_box(data.size());
        });
    });
    
    compare("Handwritten size", &|| hw_data, |data| {
        repeat(|| {
            black_box(data.size());
        });
    });
}

fn bench_core_enum_deserialization() {
    println!("\n=== Core Enum Deserialization (32/64/128 bits) ===");
    
    let test_cases = [(0u16, 42u128), (1u16, 1234u128), (2u16, 5678u128)];
    
    compare("Generated from_discriminant_and_bytes", &|| test_cases, |test_cases| {
        repeat(|| {
            for (disc, bytes) in test_cases.iter().copied() {
                black_box(GeneratedVariableEnum::from_discriminant_and_bytes(disc, bytes).unwrap());
            }
        });
    });
    
    compare("Handwritten from_discriminant_and_bytes", &|| test_cases, |test_cases| {
        repeat(|| {
            for (disc, bytes) in test_cases.iter().copied() {
                black_box(HandwrittenVariableEnum::from_discriminant_and_bytes(disc, bytes).unwrap());
            }
        });
    });
}

fn bench_core_static_methods() {
    println!("\n=== Core Static Methods (32/64/128 bits) ===");
    
    compare("Generated size_for_discriminant", &|| (), |_| {
        repeat(|| {
            for disc in 0u16..=3u16 {
                black_box(GeneratedVariableEnum::size_for_discriminant(disc));
            }
        });
    });
    
    compare("Handwritten size_for_discriminant", &|| (), |_| {
        repeat(|| {
            for disc in 0u16..=3u16 {
                black_box(HandwrittenVariableEnum::size_for_discriminant(disc));
            }
        });
    });
}

// Practical enum benchmarks (small sizes)

fn bench_practical_enum_construction() {
    println!("\n=== Practical Enum Construction (8/16/32 bits) ===");
    
    compare("Variable enum construction (all sizes)", &|| (), |_| {
        repeat(|| {
            black_box(VariableData::Small(42));
            black_box(VariableData::Medium(1234));
            black_box(VariableData::Large(0x12345678));
        });
    });
    
    compare("Handwritten enum construction (all sizes)", &|| (), |_| {
        repeat(|| {
            black_box(HandwrittenData::Small(42));
            black_box(HandwrittenData::Medium(1234));
            black_box(HandwrittenData::Large(0x12345678));
        });
    });
}

fn bench_practical_enum_serialization() {
    println!("\n=== Practical Enum Serialization (8/16/32 bits) ===");
    
    let var_data = [
        VariableData::Small(42),
        VariableData::Medium(1234),
        VariableData::Large(0x12345678),
    ];
    
    compare("Variable enum into_bytes", &|| var_data, |data| {
        repeat(|| {
            for item in data.iter() {
                black_box(<VariableData as Specifier>::into_bytes(*item).unwrap());
            }
        });
    });
    
    let hw_data = [
        HandwrittenData::Small(42),
        HandwrittenData::Medium(1234),
        HandwrittenData::Large(0x12345678),
    ];
    
    compare("Handwritten enum into_bytes", &|| hw_data, |data| {
        repeat(|| {
            for item in data.iter() {
                black_box(item.into_bytes());
            }
        });
    });
}

// Bitfield integration benchmarks

fn bench_bitfield_construction() {
    println!("\n=== Bitfield Construction ===");
    
    compare("Variable bitfield construction", &|| (), |_| {
        repeat(|| {
            black_box(VariableMessage::new().with_data(VariableData::Large(0x12345678)));
        });
    });
    
    compare("Handwritten struct construction", &|| (), |_| {
        repeat(|| {
            black_box(HandwrittenMessage {
                msg_type: 3,
                data: HandwrittenData::Large(0x12345678),
            });
        });
    });
    
    // Baseline comparison
    compare("Simple bitfield construction (baseline)", &|| (), |_| {
        repeat(|| {
            black_box(SimpleMessage::new().with_payload(0x12345678));
        });
    });
}

fn bench_bitfield_accessors() {
    println!("\n=== Bitfield Accessors ===");
    
    compare("Variable bitfield getter", 
        &|| VariableMessage::new().with_data(VariableData::Large(0x12345678)), 
        |input| {
            repeat(|| {
                black_box(black_box(&input).data());
            });
        }
    );
    
    compare("Handwritten struct getter", 
        &|| HandwrittenMessage {
            msg_type: 3,
            data: HandwrittenData::Large(0x12345678),
        }, 
        |input| {
            repeat(|| {
                black_box(black_box(&input).data());
            });
        }
    );
}

fn bench_bitfield_setters() {
    println!("\n=== Bitfield Setters ===");
    
    compare("Variable bitfield setter", 
        &|| VariableMessage::new(), 
        |mut input| {
            repeat(|| {
                black_box(&mut input).set_data(VariableData::Small(99));
            });
        }
    );
    
    compare("Handwritten struct setter", 
        &|| HandwrittenMessage::new(), 
        |mut input| {
            repeat(|| {
                black_box(&mut input).set_data(HandwrittenData::Small(99));
            });
        }
    );
}

fn bench_bitfield_serialization() {
    println!("\n=== Bitfield Serialization/Deserialization ===");
    
    compare("Variable bitfield into_bytes", 
        &|| VariableMessage::new().with_data(VariableData::Large(0x12345678)), 
        |input| {
            repeat(|| {
                black_box(black_box(&input).into_bytes());
            });
        }
    );
    
    compare("Handwritten struct into_bytes", 
        &|| HandwrittenMessage {
            msg_type: 3,
            data: HandwrittenData::Large(0x12345678),
        }, 
        |input| {
            repeat(|| {
                black_box(black_box(&input).into_bytes());
            });
        }
    );
    
    let variable_bytes = VariableMessage::new().with_data(VariableData::Large(0x12345678)).into_bytes();
    compare("Variable bitfield from_bytes", 
        &|| variable_bytes, 
        |input| {
            repeat(|| {
                black_box(VariableMessage::from_bytes(*black_box(&input)));
            });
        }
    );
    
    let handwritten_bytes = HandwrittenMessage {
        msg_type: 3,
        data: HandwrittenData::Large(0x12345678),
    }.into_bytes();
    compare("Handwritten struct from_bytes", 
        &|| handwritten_bytes, 
        |input| {
            repeat(|| {
                black_box(HandwrittenMessage::from_bytes(*black_box(&input)));
            });
        }
    );
}

// Custom types benchmarks

fn bench_custom_types() {
    println!("\n=== Custom Type Names ===");
    
    compare("Custom types construction", &|| (), |_| {
        repeat(|| {
            black_box(VariableDataCustomTypes::ByteValue(42));
            black_box(VariableDataCustomTypes::WordValue(1234));
            black_box(VariableDataCustomTypes::DwordValue(0x12345678));
        });
    });
    
    compare("Custom types into_bytes", 
        &|| VariableDataCustomTypes::DwordValue(0x12345678), 
        |input| {
            repeat(|| {
                black_box(<VariableDataCustomTypes as Specifier>::into_bytes(*black_box(&input)).unwrap());
            });
        }
    );
    
    compare("Custom types bitfield roundtrip", 
        &|| CustomTypesMessage::new().with_data(VariableDataCustomTypes::DwordValue(0x12345678)), 
        |input| {
            repeat(|| {
                let bytes = black_box(&input).into_bytes();
                black_box(CustomTypesMessage::from_bytes(bytes));
            });
        }
    );
}

// Baseline comparisons

fn bench_baseline_comparisons() {
    println!("\n=== Baseline Comparisons ===");
    
    compare("Simple enum construction", &|| (), |_| {
        repeat(|| {
            black_box(SimpleEnum::A);
        });
    });
    
    compare("Simple enum into_bytes", &|| SimpleEnum::A, |input| {
        repeat(|| {
            black_box(<SimpleEnum as Specifier>::into_bytes(*black_box(&input)).unwrap());
        });
    });
}

// Correctness validation

fn validate_correctness() {
    println!("\n🔍 Correctness Validation");
    println!("========================");
    
    // Validate large enum sizes
    println!("\n--- Large Enums (32/64/128 bits) ---");
    let test_cases_large = [
        (GeneratedVariableEnum::Small(42), HandwrittenVariableEnum::Small(42)),
        (GeneratedVariableEnum::Medium(1234), HandwrittenVariableEnum::Medium(1234)),
        (GeneratedVariableEnum::Large(5678), HandwrittenVariableEnum::Large(5678)),
    ];
    
    for (gen, hw) in test_cases_large {
        let gen_bytes = <GeneratedVariableEnum as Specifier>::into_bytes(gen).unwrap();
        let hw_bytes = hw.into_bytes().unwrap();
        assert_eq!(gen_bytes, hw_bytes, "Serialization mismatch for {:?}", gen);
        assert_eq!(gen.discriminant(), hw.discriminant(), "Discriminant mismatch for {:?}", gen);
        assert_eq!(gen.size(), hw.size(), "Size mismatch for {:?}", gen);
        println!("  {:?} ✓", gen);
    }
    
    // Validate small enum sizes
    println!("\n--- Small Enums (8/16/32 bits) ---");
    let test_data = [
        VariableData::Small(42),
        VariableData::Medium(1234),
        VariableData::Large(0x12345678),
    ];
    
    for &data in &test_data {
        let bytes = <VariableData as Specifier>::into_bytes(data).unwrap();
        let discriminant = data.discriminant();
        let reconstructed = VariableData::from_discriminant_and_bytes(discriminant, bytes).unwrap();
        assert_eq!(data, reconstructed);
        println!("  {:?} -> bytes: {}, disc: {} -> {:?} ✓", data, bytes, discriminant, reconstructed);
    }
    
    // Validate custom types
    println!("\n--- Custom Type Names ---");
    let custom_data = [
        VariableDataCustomTypes::ByteValue(42),
        VariableDataCustomTypes::WordValue(1234),
        VariableDataCustomTypes::DwordValue(0x12345678),
    ];
    
    for &data in &custom_data {
        let bytes = <VariableDataCustomTypes as Specifier>::into_bytes(data).unwrap();
        let discriminant = data.discriminant();
        let reconstructed = VariableDataCustomTypes::from_discriminant_and_bytes(discriminant, bytes).unwrap();
        assert_eq!(data, reconstructed);
        println!("  {:?} ✓", data);
    }
    
    // Validate handwritten vs generated behavior
    println!("\n--- Behavior Validation ---");
    let var_data = VariableData::Medium(1234);
    let hw_data = HandwrittenData::Medium(1234);
    
    assert_eq!(var_data.discriminant(), hw_data.discriminant(), "Discriminant mismatch");
    assert_eq!(var_data.size(), hw_data.size(), "Size mismatch");
    
    let var_bytes = <VariableData as Specifier>::into_bytes(var_data).unwrap();
    let hw_bytes = hw_data.into_bytes();
    assert_eq!(var_bytes, hw_bytes, "Serialization mismatch");
    
    println!("✅ All validations passed!");
}

// Static analysis

fn static_analysis() {
    println!("\n📊 Static Analysis");
    println!("==================");
    
    println!("\n--- Bits Constants ---");
    println!("GeneratedVariableEnum::BITS: {}", <GeneratedVariableEnum as Specifier>::BITS);
    println!("VariableData::BITS: {}", <VariableData as Specifier>::BITS);
    println!("VariableDataCustomTypes::BITS: {}", <VariableDataCustomTypes as Specifier>::BITS);
    println!("SimpleEnum::BITS: {}", <SimpleEnum as Specifier>::BITS);
    
    println!("\n--- Memory Layout ---");
    println!("GeneratedVariableEnum: {} bytes", std::mem::size_of::<GeneratedVariableEnum>());
    println!("HandwrittenVariableEnum: {} bytes", std::mem::size_of::<HandwrittenVariableEnum>());
    println!("VariableData: {} bytes", std::mem::size_of::<VariableData>());
    println!("HandwrittenData: {} bytes", std::mem::size_of::<HandwrittenData>());
    println!("SimpleEnum: {} bytes", std::mem::size_of::<SimpleEnum>());
    
    println!("\n--- Supported Sizes ---");
    println!("GeneratedVariableEnum: {:?}", GeneratedVariableEnum::supported_sizes());
    println!("VariableData: {:?}", VariableData::supported_sizes());
}

// Main execution

fn main() {
    println!("🏁 Variable Bits Comprehensive Benchmark");
    println!("=======================================");
    println!("Testing generated vs handwritten code across all scenarios");

    // First validate correctness
    validate_correctness();
    
    println!("\n📊 Running performance benchmarks...\n");
    
    // Core enum benchmarks (large sizes)
    bench_core_enum_construction();
    bench_core_enum_into_bytes();
    bench_core_enum_helpers();
    bench_core_enum_deserialization();
    bench_core_static_methods();
    
    // Practical enum benchmarks (small sizes)
    bench_practical_enum_construction();
    bench_practical_enum_serialization();
    
    // Bitfield integration
    bench_bitfield_construction();
    bench_bitfield_accessors();
    bench_bitfield_setters();
    bench_bitfield_serialization();
    
    // Additional tests
    bench_custom_types();
    bench_baseline_comparisons();
    
    // Analysis
    static_analysis();
    
    println!("\n🎯 Benchmark complete!");
    println!("Generated code should perform identically to handwritten code.");
    println!("Results are saved to disk for comparison across runs.");
}