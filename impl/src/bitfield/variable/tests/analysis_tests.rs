//! Unit tests for variable bits analysis functionality

use super::super::analysis::VariableBitsAnalysis;
use crate::bitfield::{
    config::{Config, ConfigValue, BitsConfig},
    field_config::FieldConfig,
    BitfieldStruct,
};
use proc_macro2::Span;
use quote::quote;

// Note: extract_bits_from_type is a private function, so we can't test it directly
// These tests have been removed as they test implementation details

#[test]
fn test_analyze_variable_bits_not_variable() {
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
    
    let result = bitfield.analyze_variable_bits(&config).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_analyze_variable_bits_missing_discriminator() {
    // Test struct with variable_bits but missing discriminator field
    let input = quote! {
        struct MissingDiscriminator {
            data: DataEnum,
            other: B8,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    let mut config = Config::default();
    config.bits(
        BitsConfig::Variable(vec![32, 64]),
        Span::call_site()
    ).unwrap();
    
    // Set up field configs - only mark data field
    config.variable_field_configs.set_variant_data(0, Span::call_site()).unwrap();
    
    let result = bitfield.analyze_variable_bits(&config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("discriminator"));
}

#[test]
fn test_analyze_variable_bits_missing_data() {
    // Test struct with variable_bits but missing data field
    let input = quote! {
        struct MissingData {
            discriminator: B4,
            other: B8,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    let mut config = Config::default();
    config.bits(
        BitsConfig::Variable(vec![32, 64]),
        Span::call_site()
    ).unwrap();
    
    // Set up field configs - only mark discriminator field
    config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    
    let result = bitfield.analyze_variable_bits(&config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("data"));
}

#[test]
fn test_analyze_variable_bits_duplicate_discriminator() {
    // Test struct with multiple discriminator fields
    let input = quote! {
        struct DuplicateDiscriminator {
            disc1: B4,
            disc2: B4,
            data: DataEnum,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    let mut config = Config::default();
    config.bits(
        BitsConfig::Variable(vec![32, 64]),
        Span::call_site()
    ).unwrap();
    
    // Mark two fields as discriminator
    config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    config.variable_field_configs.set_variant_discriminator(1, Span::call_site()).unwrap();
    config.variable_field_configs.set_variant_data(2, Span::call_site()).unwrap();
    
    let result = bitfield.analyze_variable_bits(&config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("multiple"));
}

#[test]
fn test_analyze_variable_bits_duplicate_data() {
    // Test struct with multiple data fields
    let input = quote! {
        struct DuplicateData {
            discriminator: B4,
            data1: DataEnum,
            data2: DataEnum,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    let mut config = Config::default();
    config.bits(
        BitsConfig::Variable(vec![32, 64]),
        Span::call_site()
    ).unwrap();
    
    // Mark two fields as data
    config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    config.variable_field_configs.set_variant_data(1, Span::call_site()).unwrap();
    config.variable_field_configs.set_variant_data(2, Span::call_site()).unwrap();
    
    let result = bitfield.analyze_variable_bits(&config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("multiple"));
}

#[test]
fn test_analyze_variable_bits_size_validation() {
    // Test struct where fixed fields exceed total size
    let input = quote! {
        struct TooManyFixedBits {
            discriminator: B4,
            data: DataEnum,
            fixed1: B16,
            fixed2: B16,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    let mut config = Config::default();
    // Total size 32, but fixed fields = 4 + 16 + 16 = 36
    config.bits(
        BitsConfig::Variable(vec![32]),
        Span::call_site()
    ).unwrap();
    
    config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    config.variable_field_configs.set_variant_data(1, Span::call_site()).unwrap();
    
    let result = bitfield.analyze_variable_bits(&config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("too small"));
}

#[test]
fn test_analyze_variable_bits_discriminator_capacity() {
    // Test struct where discriminator can't hold enough values
    let input = quote! {
        struct SmallDiscriminator {
            discriminator: B2,  // Can only hold 4 values (0-3)
            data: DataEnum,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    let mut config = Config::default();
    // 5 configurations but discriminator can only hold 4 values
    config.bits(
        BitsConfig::Variable(vec![32, 40, 48, 56, 64]),
        Span::call_site()
    ).unwrap();
    
    config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    config.variable_field_configs.set_variant_data(1, Span::call_site()).unwrap();
    
    let result = bitfield.analyze_variable_bits(&config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("discriminant"));
}

#[test]
fn test_analyze_variable_bits_valid_configuration() {
    // Test a valid variable bits configuration
    let input = quote! {
        struct ValidVariable {
            discriminator: B4,
            data: DataEnum,
            fixed1: B8,
            fixed2: B4,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    let mut config = Config::default();
    config.bits(
        BitsConfig::Variable(vec![32, 64, 128]),
        Span::call_site()
    ).unwrap();
    
    config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    config.variable_field_configs.set_variant_data(1, Span::call_site()).unwrap();
    
    let result = bitfield.analyze_variable_bits(&config).unwrap();
    assert!(result.is_some());
    
    let analysis = result.unwrap();
    assert_eq!(analysis.discriminator_field_index, 0);
    assert_eq!(analysis._data_field_index, 1);
    assert_eq!(analysis.sizes, vec![32, 64, 128]);
    assert_eq!(analysis.fixed_bits, 16); // 4 + 8 + 4
    assert_eq!(analysis.discriminator_bits, 4);
}

#[test]
fn test_analyze_variable_bits_with_bits_attribute() {
    // Test discriminator field with explicit #[bits = N] attribute
    let input = quote! {
        struct WithBitsAttribute {
            #[bits = 6]
            discriminator: SomeType,  // Not a B type
            data: DataEnum,
        }
    };
    
    let item_struct: syn::ItemStruct = syn::parse2(input).unwrap();
    let bitfield = BitfieldStruct { item_struct };
    
    let mut config = Config::default();
    config.bits(
        BitsConfig::Variable(vec![32, 64]),
        Span::call_site()
    ).unwrap();
    
    // Set bits attribute on discriminator field
    let mut field_config = FieldConfig::default();
    field_config.bits = Some(ConfigValue::new(6, Span::call_site()));
    config.field_configs.insert(0, ConfigValue::new(field_config, Span::call_site()));
    
    config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    config.variable_field_configs.set_variant_data(1, Span::call_site()).unwrap();
    
    let result = bitfield.analyze_variable_bits(&config).unwrap();
    assert!(result.is_some());
    
    let analysis = result.unwrap();
    assert_eq!(analysis.discriminator_bits, 6);
}

// Inferred mode test removed since we no longer support inferred mode

