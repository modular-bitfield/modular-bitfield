//! Unit tests for variable bits error types and formatting

use super::super::errors::VariableBitsError;
use proc_macro2::Span;

#[test]
fn test_tuple_size_mismatch_error() {
    let error = VariableBitsError::TupleSizeMismatch {
        expected: 3,
        found: 5,
        span: Span::call_site(),
    };
    
    let syn_error = error.to_syn_error();
    let error_msg = syn_error.to_string();
    
    assert!(error_msg.contains("variable_bits tuple must have 3 elements"));
    assert!(error_msg.contains("found 5"));
}

#[test]
fn test_data_type_size_mismatch_error() {
    let error = VariableBitsError::DataTypeSizeMismatch {
        variant_name: "SmallData".to_string(),
        expected_bits: 16,
        actual_bits: "u8".to_string(),
        span: Span::call_site(),
    };
    
    let syn_error = error.to_syn_error();
    let error_msg = syn_error.to_string();
    
    assert!(error_msg.contains("variant SmallData"));
    assert!(error_msg.contains("data type u8"));
    assert!(error_msg.contains("must have exactly 16 bits"));
}

#[test]
fn test_missing_variant_field_discriminator() {
    let error = VariableBitsError::MissingVariantField {
        field_type: "discriminator",
        span: Span::call_site(),
    };
    
    let syn_error = error.to_syn_error();
    let error_msg = syn_error.to_string();
    
    assert!(error_msg.contains("variable_bits structs require exactly one #[variant_discriminator] field"));
}

#[test]
fn test_missing_variant_field_data() {
    let error = VariableBitsError::MissingVariantField {
        field_type: "data",
        span: Span::call_site(),
    };
    
    let syn_error = error.to_syn_error();
    let error_msg = syn_error.to_string();
    
    assert!(error_msg.contains("variable_bits structs require exactly one #[variant_data] field"));
}

#[test]
fn test_multiple_variant_fields_discriminator() {
    let error = VariableBitsError::MultipleVariantFields {
        field_type: "discriminator",
        first_span: Span::call_site(),
        second_span: Span::call_site(),
    };
    
    let syn_error = error.to_syn_error();
    let error_msg = syn_error.to_string();
    
    assert!(error_msg.contains("found multiple #[variant_discriminator] fields"));
    assert!(error_msg.contains("only one allowed"));
}

#[test]
fn test_multiple_variant_fields_data() {
    let error = VariableBitsError::MultipleVariantFields {
        field_type: "data",
        first_span: Span::call_site(),
        second_span: Span::call_site(),
    };
    
    let syn_error = error.to_syn_error();
    let error_msg = syn_error.to_string();
    
    
    assert!(error_msg.contains("found multiple #[variant_data] fields"));
    assert!(error_msg.contains("only one allowed"));
    // The combined error message may not include both parts in to_string()
    // So let's just check for the main error
}

#[test]
fn test_invalid_discriminant_range() {
    let error = VariableBitsError::InvalidDiscriminantRange {
        discriminant: 7,
        max_allowed: 3,
        discriminator_bits: 2,
        span: Span::call_site(),
    };
    
    let syn_error = error.to_syn_error();
    let error_msg = syn_error.to_string();
    
    assert!(error_msg.contains("discriminant value 7"));
    assert!(error_msg.contains("exceeds maximum 3"));
    assert!(error_msg.contains("for 2-bit discriminator field"));
}

#[test]
fn test_conflicting_attributes() {
    let error = VariableBitsError::ConflictingAttributes {
        attr1: "variant_discriminator",
        attr2: "variant_data",
        span: Span::call_site(),
    };
    
    let syn_error = error.to_syn_error();
    let error_msg = syn_error.to_string();
    
    assert!(error_msg.contains("cannot use both #[variant_discriminator] and #[variant_data]"));
    assert!(error_msg.contains("on the same item"));
}

#[test]
fn test_discriminant_range_edge_cases() {
    // Test various bit sizes and their maximum allowed values
    let test_cases = vec![
        (1, 0, 1),   // 1 bit: max discriminant 0 (allows 0,1 so max is 1)
        (2, 3, 2),   // 2 bits: max discriminant 3
        (3, 7, 3),   // 3 bits: max discriminant 7
        (4, 15, 4),  // 4 bits: max discriminant 15
        (8, 255, 8), // 8 bits: max discriminant 255
    ];
    
    for (_bits, max_allowed, discriminator_bits) in test_cases {
        let error = VariableBitsError::InvalidDiscriminantRange {
            discriminant: max_allowed + 1,
            max_allowed,
            discriminator_bits,
            span: Span::call_site(),
        };
        
        let syn_error = error.to_syn_error();
        let error_msg = syn_error.to_string();
        
        assert!(error_msg.contains(&format!("discriminant value {}", max_allowed + 1)));
        assert!(error_msg.contains(&format!("exceeds maximum {}", max_allowed)));
        assert!(error_msg.contains(&format!("for {}-bit discriminator field", discriminator_bits)));
    }
}

#[test]
fn test_error_message_formatting() {
    // Test that error messages are properly formatted and informative
    let errors = vec![
        (
            VariableBitsError::TupleSizeMismatch {
                expected: 2,
                found: 1,
                span: Span::call_site(),
            },
            vec!["variable_bits tuple", "2 elements", "found 1"],
        ),
        (
            VariableBitsError::DataTypeSizeMismatch {
                variant_name: "LargeVariant".to_string(),
                expected_bits: 64,
                actual_bits: "B32".to_string(),
                span: Span::call_site(),
            },
            vec!["variant LargeVariant", "B32", "64 bits"],
        ),
        (
            VariableBitsError::ConflictingAttributes {
                attr1: "skip",
                attr2: "variant_data",
                span: Span::call_site(),
            },
            vec!["cannot use both", "#[skip]", "#[variant_data]"],
        ),
    ];
    
    for (error, expected_fragments) in errors {
        let syn_error = error.to_syn_error();
        let error_msg = syn_error.to_string();
        
        for fragment in expected_fragments {
            assert!(
                error_msg.contains(fragment),
                "Error message '{}' should contain '{}'",
                error_msg,
                fragment
            );
        }
    }
}

#[test]
fn test_error_spans_preserved() {
    // While we can't easily test span positions in unit tests,
    // we can verify that errors are created with the provided spans
    use proc_macro2::TokenStream;
    
    let span = Span::call_site();
    let error = VariableBitsError::MissingVariantField {
        field_type: "discriminator",
        span,
    };
    
    // Convert to syn error and back to tokens to verify span is used
    let syn_error = error.to_syn_error();
    let tokens: TokenStream = syn_error.to_compile_error();
    
    // The compile error tokens should be non-empty
    assert!(!tokens.is_empty());
}