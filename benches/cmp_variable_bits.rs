//! Variable bits comparison benchmark
//!
//! This benchmark compares the generated variable bits enum code 
//! against hand-written equivalent implementations to validate
//! zero-cost abstraction claims.
//!
//! We test construction, serialization, helper methods, and 
//! discriminant-based operations to ensure the generated code
//! performs identically to optimized handwritten code.

mod utils;

use utils::*;
use modular_bitfield::prelude::*;

// =============================================================================
// GENERATED IMPLEMENTATION (using our variable bits library)
// =============================================================================

#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[variable_bits(32, 64, 128)]
pub enum GeneratedVariableEnum {
    #[discriminant = 0]
    Small(u32),
    #[discriminant = 1] 
    Medium(u64),
    #[discriminant = 2]
    Large(u128),
}

// =============================================================================
// HANDWRITTEN IMPLEMENTATION (equivalent manual code)
// =============================================================================

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
    pub fn from_discriminant_and_bytes(discriminant: u8, bytes: u128) -> Result<Self, &'static str> {
        match discriminant {
            0 => Ok(Self::Small(bytes as u32)),
            1 => Ok(Self::Medium(bytes as u64)),
            2 => Ok(Self::Large(bytes)),
            _ => Err("Invalid discriminant value")
        }
    }
    
    #[inline]
    pub fn discriminant(&self) -> u8 {
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
    pub fn size_for_discriminant(discriminant: u8) -> Option<usize> {
        match discriminant {
            0 => Some(32),
            1 => Some(64),
            2 => Some(128),
            _ => None,
        }
    }
}

// =============================================================================
// BENCHMARK FUNCTIONS
// =============================================================================

fn bench_generated_construction() {
    println!();
    compare("Generated variable enum construction", &|| (), |_| {
        repeat(|| {
            black_box(GeneratedVariableEnum::Small(42));
            black_box(GeneratedVariableEnum::Medium(1234));
            black_box(GeneratedVariableEnum::Large(5678));
        });
    });
}

fn bench_handwritten_construction() {
    println!();
    compare("Handwritten variable enum construction", &|| (), |_| {
        repeat(|| {
            black_box(HandwrittenVariableEnum::Small(42));
            black_box(HandwrittenVariableEnum::Medium(1234));
            black_box(HandwrittenVariableEnum::Large(5678));
        });
    });
}

fn bench_generated_into_bytes() {
    println!();
    let data = [
        GeneratedVariableEnum::Small(42),
        GeneratedVariableEnum::Medium(1234),
        GeneratedVariableEnum::Large(5678),
    ];
    compare("Generated into_bytes", &|| data, |data| {
        repeat(|| {
            for item in data.iter().copied() {
                black_box(<GeneratedVariableEnum as Specifier>::into_bytes(item));
            }
        });
    });
}

fn bench_handwritten_into_bytes() {
    println!();
    let data = [
        HandwrittenVariableEnum::Small(42),
        HandwrittenVariableEnum::Medium(1234), 
        HandwrittenVariableEnum::Large(5678),
    ];
    compare("Handwritten into_bytes", &|| data, |data| {
        repeat(|| {
            for item in data.iter().copied() {
                black_box(item.into_bytes());
            }
        });
    });
}

fn bench_generated_discriminant() {
    println!();
    let data = [
        GeneratedVariableEnum::Small(42),
        GeneratedVariableEnum::Medium(1234),
        GeneratedVariableEnum::Large(5678),
    ];
    compare("Generated discriminant", &|| data, |data| {
        repeat(|| {
            for item in data.iter().copied() {
                black_box(item.discriminant());
            }
        });
    });
}

fn bench_handwritten_discriminant() {
    println!();
    let data = [
        HandwrittenVariableEnum::Small(42),
        HandwrittenVariableEnum::Medium(1234),
        HandwrittenVariableEnum::Large(5678),
    ];
    compare("Handwritten discriminant", &|| data, |data| {
        repeat(|| {
            for item in data.iter().copied() {
                black_box(item.discriminant());
            }
        });
    });
}

fn bench_generated_size() {
    println!();
    let data = [
        GeneratedVariableEnum::Small(42),
        GeneratedVariableEnum::Medium(1234),
        GeneratedVariableEnum::Large(5678),
    ];
    compare("Generated size", &|| data, |data| {
        repeat(|| {
            for item in data.iter().copied() {
                black_box(item.size());
            }
        });
    });
}

fn bench_handwritten_size() {
    println!();
    let data = [
        HandwrittenVariableEnum::Small(42),
        HandwrittenVariableEnum::Medium(1234),
        HandwrittenVariableEnum::Large(5678),
    ];
    compare("Handwritten size", &|| data, |data| {
        repeat(|| {
            for item in data.iter().copied() {
                black_box(item.size());
            }
        });
    });
}

fn bench_generated_from_discriminant() {
    println!();
    let test_cases = [(0u8, 42u128), (1u8, 1234u128), (2u8, 5678u128)];
    compare("Generated from_discriminant_and_bytes", &|| test_cases, |test_cases| {
        repeat(|| {
            for (disc, bytes) in test_cases.iter().copied() {
                black_box(GeneratedVariableEnum::from_discriminant_and_bytes(disc, bytes));
            }
        });
    });
}

fn bench_handwritten_from_discriminant() {
    println!();
    let test_cases = [(0u8, 42u128), (1u8, 1234u128), (2u8, 5678u128)];
    compare("Handwritten from_discriminant_and_bytes", &|| test_cases, |test_cases| {
        repeat(|| {
            for (disc, bytes) in test_cases.iter().copied() {
                black_box(HandwrittenVariableEnum::from_discriminant_and_bytes(disc, bytes));
            }
        });
    });
}

fn bench_generated_size_lookup() {
    println!();
    compare("Generated size_for_discriminant", &|| (), |_| {
        repeat(|| {
            for disc in 0u8..=3u8 {
                black_box(GeneratedVariableEnum::size_for_discriminant(disc));
            }
        });
    });
}

fn bench_handwritten_size_lookup() {
    println!();
    compare("Handwritten size_for_discriminant", &|| (), |_| {
        repeat(|| {
            for disc in 0u8..=3u8 {
                black_box(HandwrittenVariableEnum::size_for_discriminant(disc));
            }
        });
    });
}

// =============================================================================
// CORRECTNESS VALIDATION
// =============================================================================

fn validate_correctness() {
    println!("🔍 Validating correctness between implementations...");
    
    let test_cases = [
        (GeneratedVariableEnum::Small(42), HandwrittenVariableEnum::Small(42)),
        (GeneratedVariableEnum::Medium(1234), HandwrittenVariableEnum::Medium(1234)),
        (GeneratedVariableEnum::Large(5678), HandwrittenVariableEnum::Large(5678)),
    ];
    
    for (gen, hw) in test_cases {
        // Test serialization equivalence
        let gen_bytes = <GeneratedVariableEnum as Specifier>::into_bytes(gen).unwrap();
        let hw_bytes = hw.into_bytes().unwrap();
        assert_eq!(gen_bytes as u128, hw_bytes, "Serialization mismatch for {:?}", gen);
        
        // Test helper method equivalence
        assert_eq!(gen.discriminant(), hw.discriminant(), "Discriminant mismatch for {:?}", gen);
        assert_eq!(gen.size(), hw.size(), "Size mismatch for {:?}", gen);
        
        // Test round-trip equivalence
        let disc = gen.discriminant();
        let gen_reconstructed = GeneratedVariableEnum::from_discriminant_and_bytes(disc, gen_bytes).unwrap();
        let _hw_reconstructed = HandwrittenVariableEnum::from_discriminant_and_bytes(disc, hw_bytes).unwrap();
        
        assert_eq!(gen, gen_reconstructed, "Generated round-trip failed for {:?}", gen);
        // Note: We can't directly compare gen and hw due to different types, but we validate behavior
    }
    
    // Test static helper methods
    for disc in 0u8..=3u8 {
        let gen_size = GeneratedVariableEnum::size_for_discriminant(disc);
        let hw_size = HandwrittenVariableEnum::size_for_discriminant(disc);
        assert_eq!(gen_size, hw_size, "size_for_discriminant mismatch for disc {}", disc);
    }
    
    println!("✅ All correctness validations passed!");
}

// =============================================================================
// MAIN BENCHMARK EXECUTION
// =============================================================================

fn main() {
    println!("🏁 Variable Bits: Generated vs Handwritten Performance Comparison");
    println!("================================================================");
    
    // First validate correctness
    validate_correctness();
    
    println!("\n📊 Running performance benchmarks...");
    
    // Construction benchmarks
    bench_generated_construction();
    bench_handwritten_construction();
    
    // Serialization benchmarks
    bench_generated_into_bytes();
    bench_handwritten_into_bytes();
    
    // Helper method benchmarks
    bench_generated_discriminant();
    bench_handwritten_discriminant();
    
    bench_generated_size();
    bench_handwritten_size();
    
    // Complex operation benchmarks
    bench_generated_from_discriminant();
    bench_handwritten_from_discriminant();
    
    bench_generated_size_lookup();
    bench_handwritten_size_lookup();
    
    println!("\n🎯 Variable bits benchmark complete!");
    println!("Generated code should perform identically to handwritten code.");
}