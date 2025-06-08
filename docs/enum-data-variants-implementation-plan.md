# Implementation Plan: Enum Data Variants for BitfieldSpecifier

## Overview

This document outlines the implementation plan for adding support for enums with data variants to the `#[derive(Specifier)]` macro, as requested in [GitHub Issue #122](https://github.com/modular-bitfield/modular-bitfield/issues/122).

## Feature Summary

Enable enums with data variants to be used as bitfield specifiers by automatically packing discriminant + data into a fixed bit width.

### Before (Current)
```rust
#[derive(Specifier)]
#[bits = 4]
enum SimpleEnum {
    A = 0,
    B = 1,
    C = 2,
}
```

### After (Proposed)
```rust
#[derive(Specifier)]
#[bits = 16]
enum MessageType {
    Header(HeaderData),   // HeaderData: 14-bit Specifier
    Data(PayloadData),    // PayloadData: 14-bit Specifier  
    Control(ControlData), // ControlData: 14-bit Specifier
}
// Auto-layout: 2 discriminant bits + 14 data bits = 16 total
```

## Design Constraints

1. **All data variants must have identical bit size** - Simplifies layout and code generation
2. **All data types must implement `Specifier`** - Leverages existing infrastructure
3. **Automatic discriminant calculation** - `ceil(log2(variant_count))` bits for discriminant
4. **Fixed total bit width** - Specified via `#[bits = N]` attribute
5. **Compile-time validation** - All size checks at compile time using const assertions

## Implementation Plan

### Phase 1: Infrastructure Setup (Week 1)

#### 1.1 Const Math Utilities
**File**: `impl/src/utils.rs` (new file)
```rust
/// Calculate ceiling of log2 for discriminant bit calculation
pub const fn const_log2_ceil(n: usize) -> usize {
    if n <= 1 { 0 }
    else { (usize::BITS - (n - 1).leading_zeros()) as usize }
}

/// Pack discriminant and data bits into bytes
pub fn pack_discriminant_and_data(
    discriminant: u8,
    data_bytes: &[u8], 
    discriminant_bits: usize,
    data_bits: usize
) -> Vec<u8> {
    // Implementation for bit packing
}

/// Unpack discriminant and data bits from bytes  
pub fn unpack_discriminant_and_data(
    bytes: &[u8],
    discriminant_bits: usize,
    data_bits: usize
) -> (u8, Vec<u8>) {
    // Implementation for bit unpacking
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_const_log2_ceil() {
        assert_eq!(const_log2_ceil(1), 0);  // 1 variant = 0 discriminant bits
        assert_eq!(const_log2_ceil(2), 1);  // 2 variants = 1 discriminant bit
        assert_eq!(const_log2_ceil(3), 2);  // 3 variants = 2 discriminant bits
        assert_eq!(const_log2_ceil(4), 2);  // 4 variants = 2 discriminant bits
        assert_eq!(const_log2_ceil(5), 3);  // 5 variants = 3 discriminant bits
    }
}
```

#### 1.2 Enhanced Attribute Parsing
**File**: `impl/src/bitfield_specifier.rs`

Extend existing attribute parsing to handle optional `#[discriminant_bits = N]`:

```rust
struct SpecifierConfig {
    bits: Option<usize>,
    discriminant_bits: Option<usize>, // New field
}

fn parse_specifier_attributes(attrs: &[syn::Attribute]) -> Result<SpecifierConfig> {
    // Extend existing parsing logic
}
```

### Phase 2: Enum Analysis and Validation (Week 1-2)

#### 2.1 Enum Variant Classification
**File**: `impl/src/bitfield_specifier.rs`

```rust
#[derive(Debug)]
enum VariantType {
    Unit,                           // No data
    Data(syn::Type),               // Has data of specified type
}

struct EnumVariant {
    name: syn::Ident,
    variant_type: VariantType,
    discriminant: usize,           // Auto-assigned
}

struct EnumAnalysis {
    variants: Vec<EnumVariant>,
    total_bits: usize,
    discriminant_bits: usize,      // Auto-calculated or manual
    data_bits: usize,              // total_bits - discriminant_bits
}

fn analyze_enum(item: &syn::ItemEnum, config: &SpecifierConfig) -> Result<EnumAnalysis> {
    let total_bits = config.bits.ok_or("missing #[bits = N] attribute")?;
    let variant_count = item.variants.len();
    
    let discriminant_bits = config.discriminant_bits
        .unwrap_or_else(|| const_log2_ceil(variant_count));
    
    let data_bits = total_bits.checked_sub(discriminant_bits)
        .ok_or("not enough bits for discriminant")?;
    
    // Classify each variant
    let variants = item.variants.iter().enumerate()
        .map(|(i, variant)| analyze_variant(variant, i))
        .collect::<Result<Vec<_>>>()?;
    
    Ok(EnumAnalysis {
        variants,
        total_bits,
        discriminant_bits,
        data_bits,
    })
}
```

#### 2.2 Compile-Time Validation Generation
**File**: `impl/src/bitfield_specifier.rs`

```rust
fn generate_validation_assertions(analysis: &EnumAnalysis) -> proc_macro2::TokenStream {
    let data_bits = analysis.data_bits;
    let mut assertions = Vec::new();
    
    for variant in &analysis.variants {
        if let VariantType::Data(data_type) = &variant.variant_type {
            assertions.push(quote! {
                const _: () = {
                    // Validate data type implements Specifier and has correct size
                    fn assert_specifier<T: ::modular_bitfield::Specifier>() {}
                    assert_specifier::<#data_type>();
                    
                    // Validate data type has exactly the required bit size
                    assert!(
                        <#data_type as ::modular_bitfield::Specifier>::BITS == #data_bits,
                        "Data type bit size mismatch"
                    );
                };
            });
        }
    }
    
    quote! { #(#assertions)* }
}
```

### Phase 3: Code Generation (Week 2)

#### 3.1 Specifier Implementation Generation
**File**: `impl/src/bitfield_specifier.rs`

```rust
fn generate_specifier_impl(
    enum_name: &syn::Ident,
    analysis: &EnumAnalysis
) -> proc_macro2::TokenStream {
    let total_bits = analysis.total_bits;
    let byte_count = (total_bits + 7) / 8;
    let byte_array_type = quote! { [u8; #byte_count] };
    
    let into_bytes_arms = generate_into_bytes_arms(analysis);
    let from_bytes_arms = generate_from_bytes_arms(analysis);
    let validation = generate_validation_assertions(analysis);
    
    quote! {
        #validation
        
        impl ::modular_bitfield::Specifier for #enum_name {
            const BITS: usize = #total_bits;
            type Bytes = #byte_array_type;
            type InOut = Self;
            
            fn into_bytes(input: Self::InOut) -> Result<Self::Bytes, ::modular_bitfield::error::OutOfBounds> {
                use ::modular_bitfield::private::utils::pack_discriminant_and_data;
                
                let (discriminant, data_bytes) = match input {
                    #(#into_bytes_arms)*
                };
                
                pack_discriminant_and_data(
                    discriminant,
                    &data_bytes,
                    #discriminant_bits,
                    #data_bits
                )
            }
            
            fn from_bytes(bytes: Self::Bytes) -> Result<Self::InOut, ::modular_bitfield::error::InvalidBitPattern<Self::Bytes>> {
                use ::modular_bitfield::private::utils::unpack_discriminant_and_data;
                
                let (discriminant, data_bytes) = unpack_discriminant_and_data(
                    &bytes,
                    #discriminant_bits,
                    #data_bits
                );
                
                match discriminant {
                    #(#from_bytes_arms)*
                    _ => Err(::modular_bitfield::error::InvalidBitPattern::new(bytes))
                }
            }
        }
    }
}
```

#### 3.2 Match Arm Generation
**File**: `impl/src/bitfield_specifier.rs`

```rust
fn generate_into_bytes_arms(analysis: &EnumAnalysis) -> Vec<proc_macro2::TokenStream> {
    analysis.variants.iter().map(|variant| {
        let variant_name = &variant.name;
        let discriminant = variant.discriminant;
        
        match &variant.variant_type {
            VariantType::Unit => {
                quote! {
                    Self::#variant_name => (#discriminant, vec![0u8; (#data_bits + 7) / 8]),
                }
            }
            VariantType::Data(data_type) => {
                quote! {
                    Self::#variant_name(inner) => {
                        let data_bytes = <#data_type as ::modular_bitfield::Specifier>::into_bytes(inner)?;
                        (#discriminant, data_bytes.to_vec())
                    },
                }
            }
        }
    }).collect()
}

fn generate_from_bytes_arms(analysis: &EnumAnalysis) -> Vec<proc_macro2::TokenStream> {
    analysis.variants.iter().map(|variant| {
        let variant_name = &variant.name;
        let discriminant = variant.discriminant;
        
        match &variant.variant_type {
            VariantType::Unit => {
                quote! {
                    #discriminant => Ok(Self::#variant_name),
                }
            }
            VariantType::Data(data_type) => {
                quote! {
                    #discriminant => {
                        let data_array: <#data_type as ::modular_bitfield::Specifier>::Bytes = 
                            data_bytes.try_into().map_err(|_| ::modular_bitfield::error::InvalidBitPattern::new(bytes))?;
                        let inner = <#data_type as ::modular_bitfield::Specifier>::from_bytes(data_array)
                            .map_err(|_| ::modular_bitfield::error::InvalidBitPattern::new(bytes))?;
                        Ok(Self::#variant_name(inner))
                    },
                }
            }
        }
    }).collect()
}
```

### Phase 4: Integration and Testing (Week 3)

#### 4.1 Main Macro Integration
**File**: `impl/src/bitfield_specifier.rs`

Update the main `derive_bitfield_specifier` function:

```rust
pub fn derive_bitfield_specifier(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    match &input.data {
        syn::Data::Enum(data_enum) => {
            let config = parse_specifier_attributes(&input.attrs)?;
            
            // Check if this is a simple enum (existing) or data enum (new)
            let has_data_variants = data_enum.variants.iter()
                .any(|v| !matches!(v.fields, syn::Fields::Unit));
            
            if has_data_variants {
                // New: Generate data variant specifier
                let analysis = analyze_enum(&syn::ItemEnum { /* ... */ }, &config)?;
                generate_specifier_impl(&input.ident, &analysis)
            } else {
                // Existing: Generate simple enum specifier
                generate_simple_enum_specifier(&input, &config)
            }
        }
        _ => return syn::Error::new_spanned(input, "Specifier can only be derived for enums").to_compile_error(),
    }
}
```

#### 4.2 Comprehensive Test Suite
**File**: `tests/bitfield/enum_data_variants.rs`

```rust
use modular_bitfield::prelude::*;

#[test]
fn basic_data_variant_functionality() {
    #[derive(Specifier, Debug, PartialEq)]
    #[bits = 8]
    struct Header {
        protocol: B4,
        flags: B4,
    }
    
    #[derive(Specifier, Debug, PartialEq)]
    #[bits = 8]
    struct Payload {
        sequence: B4,
        data: B4,
    }
    
    #[derive(Specifier, Debug, PartialEq)]
    #[bits = 12]  // 2 discriminant + 8 data + 2 padding = 12
    enum Message {
        Header(Header),   // discriminant = 0
        Data(Payload),    // discriminant = 1
        Heartbeat,        // discriminant = 2, no data
    }
    
    // Test construction and roundtrip
    let header = Header::new().with_protocol(5).with_flags(10);
    let msg = Message::Header(header);
    
    let bytes = msg.into_bytes();
    let recovered = Message::from_bytes(bytes).unwrap();
    
    assert_eq!(msg, recovered);
    
    if let Message::Header(h) = recovered {
        assert_eq!(h.protocol(), 5);
        assert_eq!(h.flags(), 10);
    } else {
        panic!("Expected Header variant");
    }
}

#[test]
fn manual_discriminant_bits() {
    #[derive(Specifier)]
    #[bits = 8]
    struct Data {
        value: B4,
    }
    
    #[derive(Specifier)]
    #[bits = 8]
    #[discriminant_bits = 4]  // Force 4 bits even though 2 would suffice
    enum Message {
        Data(Data),      // discriminant = 0, data gets 4 bits now
        Heartbeat,       // discriminant = 1
    }
    
    // Should compile and work correctly
}

#[test] 
fn unit_variants_mixed() {
    #[derive(Specifier)]
    #[bits = 6]
    struct SmallData {
        value: B4,
    }
    
    #[derive(Specifier)]
    #[bits = 8]  // 2 discriminant + 4 data + 2 padding = 8
    enum Mixed {
        Data(SmallData),  // Has data
        Reset,            // Unit variant
        Error,            // Unit variant
    }
    
    let reset = Mixed::Reset;
    let bytes = reset.into_bytes();
    let recovered = Mixed::from_bytes(bytes).unwrap();
    assert!(matches!(recovered, Mixed::Reset));
}
```

#### 4.3 UI Error Tests
**File**: `tests/ui/enum_data_variants/`

```rust
// size_mismatch.rs
use modular_bitfield::prelude::*;

#[derive(Specifier)]
#[bits = 8]
struct Data8 { value: B8 }

#[derive(Specifier)]
#[bits = 4]
struct Data4 { value: B4 }

#[derive(Specifier)]
#[bits = 16]  // 2 discriminant + 14 data = 16
enum Message {
    Small(Data4),  // ERROR: Data4 is 4 bits, expected 14
    Large(Data8),  // ERROR: Data8 is 8 bits, expected 14
}

fn main() {}

// size_mismatch.stderr
error: Data type bit size mismatch
 --> tests/ui/enum_data_variants/size_mismatch.rs:XX:XX
  |
  | assert!(<Data4 as Specifier>::BITS == 14);
  |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

### Phase 5: Documentation and Examples (Week 3)

#### 5.1 Documentation Updates
**File**: `docs/bitfield_specifier.md`

Add section explaining enum data variants:

```markdown
## Enum Data Variants

The `#[derive(Specifier)]` macro supports enums with data variants, automatically 
packing discriminant and data into a fixed bit width.

### Basic Usage

```rust
#[derive(Specifier)]
#[bits = 8]
struct Header { protocol: B4, flags: B4 }

#[derive(Specifier)]
#[bits = 8]  
struct Payload { sequence: B4, data: B4 }

#[derive(Specifier)]
#[bits = 12]  // Total size: 2 discriminant + 8 data + 2 padding
enum Message {
    Header(Header),   // Auto-assigned discriminant = 0
    Data(Payload),    // Auto-assigned discriminant = 1  
    Heartbeat,        // Auto-assigned discriminant = 2, no data
}
```

### Requirements

1. **All data types must implement `Specifier`**
2. **All data types must have identical bit size**
3. **Total enum size specified with `#[bits = N]`**

### Advanced Usage

```rust
#[derive(Specifier)]
#[bits = 16]
#[discriminant_bits = 4]  // Manual control for future expansion
enum ProtocolMessage {
    Header(Header),    // 12 bits of data (16 - 4)
    Data(Data),        // 12 bits of data
    Control(Control),  // 12 bits of data
}
```
```

#### 5.2 Example Applications
**File**: `examples/network_protocol.rs`

```rust
//! Example: Network protocol with different message types
use modular_bitfield::prelude::*;

#[derive(Specifier, Debug, PartialEq)]
#[bits = 14]
struct NetworkHeader {
    version: B4,
    packet_type: B4,
    flags: B6,
}

#[derive(Specifier, Debug, PartialEq)]
#[bits = 14]
struct NetworkData {
    sequence: B8,
    fragment: B6,
}

#[derive(Specifier, Debug, PartialEq)]
#[bits = 14]
struct NetworkControl {
    command: B8,
    parameter: B6,
}

#[derive(Specifier, Debug, PartialEq)]
#[bits = 16]  // 2 discriminant + 14 data = 16 total
enum NetworkMessage {
    Header(NetworkHeader),
    Data(NetworkData),
    Control(NetworkControl),
    Keepalive,  // No data
}

fn main() {
    // Create messages
    let header = NetworkHeader::new()
        .with_version(1)
        .with_packet_type(2)
        .with_flags(0x3F);
    
    let msg = NetworkMessage::Header(header);
    
    // Serialize to bytes
    let bytes = msg.into_bytes();
    println!("Serialized: {:02X?}", bytes);
    
    // Deserialize and verify
    let recovered = NetworkMessage::from_bytes(bytes).unwrap();
    assert_eq!(msg, recovered);
    
    match recovered {
        NetworkMessage::Header(h) => {
            println!("Received header: version={}, type={}, flags=0x{:02X}",
                h.version(), h.packet_type(), h.flags());
        }
        _ => unreachable!(),
    }
}
```

## Implementation Checklist

### Week 1
- [ ] Implement const math utilities (`const_log2_ceil`, bit packing functions)
- [ ] Extend attribute parsing for `#[discriminant_bits = N]`
- [ ] Implement enum variant analysis and classification
- [ ] Create compile-time validation assertion generation

### Week 2  
- [ ] Implement Specifier trait code generation for data variants
- [ ] Create match arm generation for `into_bytes` and `from_bytes`
- [ ] Integrate with existing macro infrastructure
- [ ] Handle edge cases (unit variants, error propagation)

### Week 3
- [ ] Write comprehensive test suite (basic functionality, edge cases, error cases)
- [ ] Create UI tests for compile-time error validation  
- [ ] Update documentation with examples and usage patterns
- [ ] Create example applications demonstrating real-world usage

## Success Criteria

1. ✅ **Functionality**: Enums with data variants can be used as Specifiers
2. ✅ **Type Safety**: Compile-time validation prevents size mismatches
3. ✅ **Performance**: Zero runtime overhead, const-friendly where possible
4. ✅ **Ergonomics**: Minimal syntax, automatic layout calculation
5. ✅ **Compatibility**: No breaking changes to existing simple enum support
6. ✅ **Documentation**: Clear examples and comprehensive API documentation

## Risk Mitigation

### Potential Issues
1. **Complex bit manipulation bugs** → Extensive unit testing of packing/unpacking
2. **Compile-time error message clarity** → Custom error types and helpful diagnostics  
3. **Performance concerns** → Benchmarking against manual implementations
4. **API design complexity** → Start with minimal viable API, iterate based on feedback

### Fallback Plan
If implementation proves more complex than estimated, fall back to a more constrained initial version:
- Require power-of-2 total bit sizes only
- Limit to specific data bit sizes (8, 16, 32)
- Manual discriminant assignment only

This plan provides a structured approach to implementing a significant but well-scoped enhancement to the modular-bitfield library.