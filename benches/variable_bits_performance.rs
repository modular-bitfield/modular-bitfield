use tiny_bench::bench;
use modular_bitfield::prelude::*;

// Variable bits enum for testing
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[variable_bits(8, 16, 32)]
enum VariableData {
    #[discriminant = 0]
    Small(u8),
    #[discriminant = 1]
    Medium(u16),
    #[discriminant = 2]
    Large(u32),
}

// Traditional unit enum for comparison
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[bits = 2]
enum SimpleEnum {
    A = 0,
    B = 1,
    C = 2,
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
    
    fn discriminant(&self) -> u8 {
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
}

fn main() {
    println!("Variable Bits Performance Benchmarks");
    println!("=====================================");
    
    // Test data
    let variable_data = [
        VariableData::Small(42),
        VariableData::Medium(1234),
        VariableData::Large(0x12345678),
    ];
    
    let handwritten_data = [
        HandwrittenData::Small(42),
        HandwrittenData::Medium(1234),
        HandwrittenData::Large(0x12345678),
    ];
    
    let simple_enum_data = [SimpleEnum::A, SimpleEnum::B, SimpleEnum::C];
    
    // Benchmark variable enum construction
    println!("\\nBenchmarking construction...");
    bench(|| {
        for _ in 0..1000 {
            let _small = VariableData::Small(42);
            let _medium = VariableData::Medium(1234);  
            let _large = VariableData::Large(0x12345678);
        }
    });
    println!("Variable enum construction complete");
    
    bench(|| {
        for _ in 0..1000 {
            let _small = HandwrittenData::Small(42);
            let _medium = HandwrittenData::Medium(1234);
            let _large = HandwrittenData::Large(0x12345678);
        }
    });
    println!("Handwritten construction complete");
    
    bench(|| {
        for _ in 0..1000 {
            let _a = SimpleEnum::A;
            let _b = SimpleEnum::B;
            let _c = SimpleEnum::C;
        }
    });
    println!("Simple enum construction complete");
    
    // Benchmark into_bytes
    println!("\\nBenchmarking into_bytes...");
    bench(|| {
        for _ in 0..1000 {
            for &item in &variable_data {
                let _ = <VariableData as Specifier>::into_bytes(item);
            }
        }
    });
    println!("Variable enum into_bytes complete");
    
    bench(|| {
        for _ in 0..1000 {
            for &item in &handwritten_data {
                let _ = item.into_bytes();
            }
        }
    });
    println!("Handwritten into_bytes complete");
    
    bench(|| {
        for _ in 0..1000 {
            for &item in &simple_enum_data {
                let _ = <SimpleEnum as Specifier>::into_bytes(item);
            }
        }
    });
    println!("Simple enum into_bytes complete");
    
    // Benchmark helper methods
    println!("\\nBenchmarking helper methods...");
    bench(|| {
        for _ in 0..1000 {
            for &item in &variable_data {
                let _ = item.discriminant();
                let _ = item.size();
            }
        }
    });
    println!("Variable enum helpers complete");
    
    bench(|| {
        for _ in 0..1000 {
            for &item in &handwritten_data {
                let _ = item.discriminant();
                let _ = item.size();
            }
        }
    });
    println!("Handwritten helpers complete");
    
    // Benchmark size lookup
    println!("\\nBenchmarking size lookup...");
    bench(|| {
        for _ in 0..1000 {
            for disc in 0u8..=3u8 {
                let _ = VariableData::size_for_discriminant(disc);
            }
        }
    });
    println!("Variable enum size lookup complete");
    
    // Static analysis comparison  
    println!("\\nStatic Analysis:");
    println!("===============");
    println!("VariableData::BITS: {}", <VariableData as Specifier>::BITS);
    println!("SimpleEnum::BITS: {}", <SimpleEnum as Specifier>::BITS);
    
    // Test supported sizes
    println!("VariableData supported sizes: {:?}", VariableData::supported_sizes());
    println!("VariableData supported discriminants: {:?}", VariableData::supported_discriminants());
    
    // Memory layout analysis
    println!("\\nMemory sizes:");
    println!("  VariableData: {} bytes", std::mem::size_of::<VariableData>());
    println!("  SimpleEnum: {} bytes", std::mem::size_of::<SimpleEnum>());
    println!("  HandwrittenData: {} bytes", std::mem::size_of::<HandwrittenData>());
    
    // Zero-cost validation
    println!("\\nZero-cost validation:");
    let small = VariableData::Small(42);
    println!("  small.discriminant() == 0: {}", small.discriminant() == 0);
    println!("  small.size() == 8: {}", small.size() == 8);
    println!("  VariableData::size_for_discriminant(0) == Some(8): {:?}", VariableData::size_for_discriminant(0));
    
    // Correctness validation
    println!("\\nCorrectness validation:");
    for &data in &variable_data {
        let bytes = <VariableData as Specifier>::into_bytes(data).unwrap();
        let discriminant = data.discriminant();
        let reconstructed = VariableData::from_discriminant_and_bytes(discriminant, bytes).unwrap();
        println!("  {:?} -> bytes: {}, disc: {} -> {:?} ✓", data, bytes, discriminant, reconstructed);
        assert_eq!(data, reconstructed);
    }
    
    println!("\\n🎉 All performance tests and validations complete!");
}