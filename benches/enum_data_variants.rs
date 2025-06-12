#![allow(dead_code)]

mod utils;

use modular_bitfield::{
    bitfield,
    specifiers::{B4, B6, B8},
    Specifier,
};
use utils::*;

// Baseline: Simple unit enum (existing functionality)
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[bits = 2]
enum SimpleEnum {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}

// Data variants: 8-bit enum with primitive data
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[bits = 8]
enum DataEnum8 {
    Empty,
    Value(u8),
    Flag,
}

// Data variants: 16-bit enum with complex data
#[bitfield]
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
struct Header {
    protocol: B4,
    flags: B4,
}

#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[bits = 8]
enum DataEnum16 {
    Empty,
    Header(Header),
    Value(u8),
    Control,
}

// Data variants: 16-bit enum with u16
#[derive(Specifier, Debug, Clone, Copy, PartialEq)]
#[bits = 16]
enum DataEnum32 {
    Empty,
    Value(u16),
    Extended,
}

// Handwritten equivalent for comparison
#[derive(Debug, Clone, Copy, PartialEq)]
struct HandwrittenEnum {
    value: u8,
}

impl HandwrittenEnum {
    fn new_empty() -> Self {
        Self { value: 0 }
    }
    fn new_value(v: u8) -> Self {
        Self { value: v }
    }
    fn new_flag() -> Self {
        Self { value: 0 }
    }

    fn into_bytes(self) -> u8 {
        self.value
    }
    fn from_bytes(bytes: u8) -> Self {
        Self { value: bytes }
    }
}

fn bench_enum_into_bytes() {
    // Baseline: Simple unit enum
    one_shot("into_bytes - SimpleEnum", &|| SimpleEnum::A, |input| {
        repeat(|| {
            black_box(SimpleEnum::into_bytes(*black_box(&input)).unwrap());
        });
    });

    // Data variants: 8-bit
    one_shot(
        "into_bytes - DataEnum8::Empty",
        &|| DataEnum8::Empty,
        |input| {
            repeat(|| {
                black_box(DataEnum8::into_bytes(*black_box(&input)).unwrap());
            });
        },
    );

    one_shot(
        "into_bytes - DataEnum8::Value",
        &|| DataEnum8::Value(42),
        |input| {
            repeat(|| {
                black_box(DataEnum8::into_bytes(*black_box(&input)).unwrap());
            });
        },
    );

    // Data variants: 8-bit complex
    let header = Header::new().with_protocol(5).with_flags(10);
    one_shot(
        "into_bytes - DataEnum16::Header",
        &|| DataEnum16::Header(header),
        |input| {
            repeat(|| {
                black_box(DataEnum16::into_bytes(*black_box(&input)).unwrap());
            });
        },
    );

    one_shot(
        "into_bytes - DataEnum16::Value",
        &|| DataEnum16::Value(123),
        |input| {
            repeat(|| {
                black_box(DataEnum16::into_bytes(*black_box(&input)).unwrap());
            });
        },
    );

    // Data variants: 16-bit
    one_shot(
        "into_bytes - DataEnum32::Value",
        &|| DataEnum32::Value(1234),
        |input| {
            repeat(|| {
                black_box(DataEnum32::into_bytes(*black_box(&input)).unwrap());
            });
        },
    );

    // Handwritten comparison
    one_shot(
        "into_bytes - Handwritten",
        &|| HandwrittenEnum::new_value(42),
        |input| {
            repeat(|| {
                black_box(black_box(&input).into_bytes());
            });
        },
    );
}

fn bench_enum_from_bytes() {
    // Baseline: Simple unit enum
    one_shot("from_bytes - SimpleEnum", &|| 1u8, |input| {
        repeat(|| {
            black_box(SimpleEnum::from_bytes(*black_box(&input)).unwrap());
        });
    });

    // Data variants: 8-bit
    one_shot("from_bytes - DataEnum8", &|| 42u8, |input| {
        repeat(|| {
            black_box(DataEnum8::from_bytes(*black_box(&input)).unwrap());
        });
    });

    // Data variants: 8-bit complex
    one_shot("from_bytes - DataEnum16", &|| 123u8, |input| {
        repeat(|| {
            black_box(DataEnum16::from_bytes(*black_box(&input)).unwrap());
        });
    });

    // Data variants: 16-bit
    one_shot("from_bytes - DataEnum32", &|| 1234u16, |input| {
        repeat(|| {
            black_box(DataEnum32::from_bytes(*black_box(&input)).unwrap());
        });
    });

    // Handwritten comparison
    one_shot("from_bytes - Handwritten", &|| 42u8, |input| {
        repeat(|| {
            black_box(HandwrittenEnum::from_bytes(*black_box(&input)));
        });
    });
}

fn bench_enum_roundtrip() {
    // Baseline: Simple unit enum
    one_shot("roundtrip - SimpleEnum", &|| SimpleEnum::B, |input| {
        repeat(|| {
            let bytes = SimpleEnum::into_bytes(*black_box(&input)).unwrap();
            black_box(SimpleEnum::from_bytes(bytes).unwrap());
        });
    });

    // Data variants: 8-bit
    one_shot(
        "roundtrip - DataEnum8::Value",
        &|| DataEnum8::Value(42),
        |input| {
            repeat(|| {
                let bytes = DataEnum8::into_bytes(*black_box(&input)).unwrap();
                black_box(DataEnum8::from_bytes(bytes).unwrap());
            });
        },
    );

    // Data variants: 8-bit complex
    let header = Header::new().with_protocol(5).with_flags(10);
    one_shot(
        "roundtrip - DataEnum16::Header",
        &|| DataEnum16::Header(header),
        |input| {
            repeat(|| {
                let bytes = DataEnum16::into_bytes(*black_box(&input)).unwrap();
                black_box(DataEnum16::from_bytes(bytes).unwrap());
            });
        },
    );

    // Data variants: 16-bit
    one_shot(
        "roundtrip - DataEnum32::Value",
        &|| DataEnum32::Value(1234),
        |input| {
            repeat(|| {
                let bytes = DataEnum32::into_bytes(*black_box(&input)).unwrap();
                black_box(DataEnum32::from_bytes(bytes).unwrap());
            });
        },
    );

    // Handwritten comparison
    one_shot(
        "roundtrip - Handwritten",
        &|| HandwrittenEnum::new_value(42),
        |input| {
            repeat(|| {
                let bytes = black_box(&input).into_bytes();
                black_box(HandwrittenEnum::from_bytes(bytes));
            });
        },
    );
}

fn bench_bitfield_usage() {
    #[bitfield]
    struct SimplePacket {
        header: B8,
        simple_enum: SimpleEnum,
        padding: B6,
    }

    #[bitfield]
    struct DataPacket {
        header: B8,
        #[bits = 8]
        data_enum: DataEnum8,
        padding: B8,
    }

    one_shot(
        "bitfield - SimplePacket get",
        &|| SimplePacket::new().with_simple_enum(SimpleEnum::C),
        |input| {
            repeat(|| {
                black_box(black_box(&input).simple_enum());
            });
        },
    );

    one_shot(
        "bitfield - SimplePacket set",
        &SimplePacket::new,
        |mut input| {
            repeat(|| {
                black_box(&mut input).set_simple_enum(SimpleEnum::A);
            });
        },
    );

    one_shot(
        "bitfield - DataPacket get",
        &|| DataPacket::new().with_data_enum(DataEnum8::Value(42)),
        |input| {
            repeat(|| {
                black_box(black_box(&input).data_enum());
            });
        },
    );

    one_shot(
        "bitfield - DataPacket set",
        &DataPacket::new,
        |mut input| {
            repeat(|| {
                black_box(&mut input).set_data_enum(DataEnum8::Value(123));
            });
        },
    );
}

fn bench_simple_enum() {
    println!("\n=== SimpleEnum (baseline unit enum) ===");
    one_shot("simple_enum_into_bytes", &|| SimpleEnum::A, |input| {
        repeat(|| {
            black_box(SimpleEnum::into_bytes(*black_box(&input)).unwrap());
        });
    });

    one_shot("simple_enum_from_bytes", &|| 1u8, |input| {
        repeat(|| {
            black_box(SimpleEnum::from_bytes(*black_box(&input)).unwrap());
        });
    });

    one_shot("simple_enum_roundtrip", &|| SimpleEnum::B, |input| {
        repeat(|| {
            let bytes = SimpleEnum::into_bytes(*black_box(&input)).unwrap();
            black_box(SimpleEnum::from_bytes(bytes).unwrap());
        });
    });
}

fn bench_data_enum8() {
    println!("\n=== DataEnum8 (8-bit data variants) ===");
    one_shot(
        "data_enum8_into_bytes_empty",
        &|| DataEnum8::Empty,
        |input| {
            repeat(|| {
                black_box(DataEnum8::into_bytes(*black_box(&input)).unwrap());
            });
        },
    );

    one_shot(
        "data_enum8_into_bytes_value",
        &|| DataEnum8::Value(42),
        |input| {
            repeat(|| {
                black_box(DataEnum8::into_bytes(*black_box(&input)).unwrap());
            });
        },
    );

    one_shot("data_enum8_from_bytes", &|| 42u8, |input| {
        repeat(|| {
            black_box(DataEnum8::from_bytes(*black_box(&input)).unwrap());
        });
    });

    one_shot("data_enum8_roundtrip", &|| DataEnum8::Value(42), |input| {
        repeat(|| {
            let bytes = DataEnum8::into_bytes(*black_box(&input)).unwrap();
            black_box(DataEnum8::from_bytes(bytes).unwrap());
        });
    });
}

fn bench_handwritten() {
    println!("\n=== HandwrittenEnum (comparison) ===");
    one_shot(
        "handwritten_into_bytes",
        &|| HandwrittenEnum::new_value(42),
        |input| {
            repeat(|| {
                black_box(black_box(&input).into_bytes());
            });
        },
    );

    one_shot("handwritten_from_bytes", &|| 42u8, |input| {
        repeat(|| {
            black_box(HandwrittenEnum::from_bytes(*black_box(&input)));
        });
    });

    one_shot(
        "handwritten_roundtrip",
        &|| HandwrittenEnum::new_value(42),
        |input| {
            repeat(|| {
                let bytes = black_box(&input).into_bytes();
                black_box(HandwrittenEnum::from_bytes(bytes));
            });
        },
    );
}

fn main() {
    println!("=== Enum Data Variants Performance Benchmarks ===");
    println!("Rust version: {}", env!("CARGO_PKG_VERSION"));
    println!("Comparing enum data variants vs unit enums vs handwritten code");
    println!();

    bench_simple_enum();
    bench_data_enum8();
    bench_handwritten();

    println!();
    println!("=== BENCHMARKS COMPLETE ===");
}
