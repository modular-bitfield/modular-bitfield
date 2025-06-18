//! Unit tests for variable bits expansion functionality

use super::super::expand::VariableStructExpander;
use super::super::analysis::{VariableBitsAnalysis, VariableStructAnalysis};
use crate::bitfield::{
    config::{Config, BitsConfig},
    BitfieldStruct,
};
use proc_macro2::Span;
use quote::quote;

#[test]
fn test_expand_variable_struct_not_variable() {
    // Test struct without variable_bits
    let input = quote! {
        struct Normal {
            field1: B8,
            field2: B16,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    let config = Config::default();
    
    let result = bitfield.expand_variable_struct(&config).unwrap();
    let generated = result.to_string();
    
    // Should return empty for non-variable structs
    assert!(generated.is_empty());
}

#[test]
fn test_expand_variable_struct_complete() {
    // Test complete expansion with all components
    let input = quote! {
        struct CompleteVariable {
            disc: B4,
            data: MyDataEnum,
            extra: B12,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    let mut config = Config::default();
    config.bits(
        BitsConfig::Variable(vec![32, 48, 64]),
        Span::call_site()
    ).unwrap();
    
    config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    config.variable_field_configs.set_variant_data(1, Span::call_site()).unwrap();
    
    let result = bitfield.expand_variable_struct(&config).unwrap();
    let generated = result.to_string();
    
    // Verify implementation block is generated
    assert!(generated.contains("impl CompleteVariable"));
}

#[test]
fn test_generate_variable_specifier_impl() {
    let input = quote! {
        struct TestStruct {
            discriminator: B2,
            data: DataEnum,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    // Create analysis with specific sizes
    let analysis = VariableStructAnalysis {
        discriminator_field_index: 0,
        _data_field_index: 1,
        _fixed_field_indices: vec![],
        sizes: vec![16, 32, 64],
        fixed_bits: 2,
        data_enum_type: syn::parse_quote! { DataEnum },
        discriminator_bits: 2,
    };
    
    let specifier_impl = bitfield.generate_variable_specifier_impl(&analysis);
    let generated = specifier_impl.to_string();
    
    
    // Verify Specifier trait implementation - check for the actual pattern
    assert!(generated.contains("impl") && generated.contains("Specifier") && generated.contains("TestStruct"));
    assert!(generated.contains("const BITS : usize = 64"));  // Max size (with spaces)
    assert!(generated.contains("type Bytes = [u8 ; 8"));  // 64 bits = 8 bytes (with spaces)
    assert!(generated.contains("type InOut = Self"));
    
    // Verify methods
    assert!(generated.contains("fn into_bytes"));
    assert!(generated.contains("fn from_bytes"));
    
    // Verify compile-time check
    assert!(generated.contains("CheckSpecifierHasAtMost128Bits"));
}

#[test]
fn test_generate_variable_size_extensions() {
    let input = quote! {
        struct TestStruct {
            discriminator: B2,
            data: DataEnum,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    let mut config = Config::default();
    config.bits(
        BitsConfig::Variable(vec![16, 32]),
        Span::call_site()
    ).unwrap();
    
    config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    config.variable_field_configs.set_variant_data(1, Span::call_site()).unwrap();
    
    let result = bitfield.generate_variable_size_extensions(&config).unwrap();
    assert!(result.is_some());
    
    let extensions = result.unwrap();
    let generated = extensions.to_string();
    
    // Verify implementation block
    assert!(generated.contains("impl TestStruct"));
}

#[test]
fn test_analyze_and_expand_integration() {
    // Test the full analysis and expansion flow
    let input = quote! {
        struct VariablePacket {
            variant_type: B3,
            payload: DataEnum,
            checksum: B5,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    let mut config = Config::default();
    config.bits(
        BitsConfig::Variable(vec![24, 32, 48]),
        Span::call_site()
    ).unwrap();
    
    config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    config.variable_field_configs.set_variant_data(1, Span::call_site()).unwrap();
    
    // Analyze
    let analysis = bitfield.analyze_variable_bits(&config).unwrap();
    assert!(analysis.is_some());
    
    let analysis = analysis.unwrap();
    assert_eq!(analysis.discriminator_field_index, 0);
    assert_eq!(analysis._data_field_index, 1);
    assert_eq!(analysis._fixed_field_indices, vec![2]);
    assert_eq!(analysis.sizes, vec![24, 32, 48]);
    assert_eq!(analysis.fixed_bits, 8); // 3 + 5
    assert_eq!(analysis.discriminator_bits, 3);
    
    // Expand
    let expanded = bitfield.expand_variable_struct(&config).unwrap();
    let generated = expanded.to_string();
    
    // Should have implementation
    assert!(generated.contains("impl VariablePacket"));
}

#[test]
fn test_max_size_calculation() {
    // Test with various size configurations
    let test_cases = vec![
        (vec![8, 16, 32], 32, 4),
        (vec![24, 48, 96], 96, 12),
        (vec![128], 128, 16),
        (vec![7, 15, 23], 23, 3),  // Non-byte-aligned sizes
    ];
    
    for (sizes, expected_bits, expected_bytes) in test_cases {
        let analysis = VariableStructAnalysis {
            discriminator_field_index: 0,
            _data_field_index: 1,
            _fixed_field_indices: vec![],
            sizes: sizes.clone(),
            fixed_bits: 4,
            data_enum_type: syn::parse_quote! { DataEnum },
            discriminator_bits: 4,
        };
        
        let max_bits = analysis.sizes.iter().max().unwrap_or(&0);
        let max_bytes = (max_bits + 7) / 8;
        
        assert_eq!(*max_bits, expected_bits, "Max bits mismatch for {:?}", sizes);
        assert_eq!(max_bytes, expected_bytes, "Max bytes mismatch for {:?}", sizes);
    }
}