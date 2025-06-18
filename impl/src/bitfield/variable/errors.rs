use crate::errors::CombineError;
use proc_macro2::Span;

/// Errors specific to variable bits functionality
#[allow(dead_code)]
pub enum VariableBitsError {
    TupleSizeMismatch {
        expected: usize,
        found: usize,
        span: Span,
    },
    DataTypeSizeMismatch {
        variant_name: String,
        expected_bits: usize,
        actual_bits: String, // Type name since we can't evaluate BITS at parse time
        span: Span,
    },
    MissingVariantField {
        field_type: &'static str, // "discriminator" or "data"
        span: Span,
    },
    MultipleVariantFields {
        field_type: &'static str,
        first_span: Span,
        second_span: Span,
    },
    InvalidDiscriminantRange {
        discriminant: usize,
        max_allowed: usize,
        discriminator_bits: usize,
        span: Span,
    },
    ConflictingAttributes {
        attr1: &'static str,
        attr2: &'static str,
        span: Span,
    },
}

impl VariableBitsError {
    pub fn to_syn_error(&self) -> syn::Error {
        match self {
            Self::TupleSizeMismatch {
                expected,
                found,
                span,
            } => {
                format_err!(
                    *span,
                    "variable_bits tuple must have {} elements (one per variant), found {}",
                    expected,
                    found
                )
            }
            Self::DataTypeSizeMismatch {
                variant_name,
                expected_bits,
                actual_bits,
                span,
            } => {
                format_err!(
                    *span,
                    "variant {} data type {} must have exactly {} bits for variable_bits compatibility",
                    variant_name, actual_bits, expected_bits
                )
            }
            Self::MissingVariantField { field_type, span } => {
                format_err!(
                    *span,
                    "variable_bits structs require exactly one #[variant_{}] field",
                    field_type
                )
            }
            Self::MultipleVariantFields {
                field_type,
                first_span,
                second_span,
            } => format_err!(
                *second_span,
                "found multiple #[variant_{}] fields, only one allowed",
                field_type
            )
            .into_combine(format_err!(
                *first_span,
                "first #[variant_{}] field declared here",
                field_type
            )),
            Self::InvalidDiscriminantRange {
                discriminant,
                max_allowed,
                discriminator_bits,
                span,
            } => {
                format_err!(
                    *span,
                    "discriminant value {} exceeds maximum {} for {}-bit discriminator field",
                    discriminant,
                    max_allowed,
                    discriminator_bits
                )
            }
            Self::ConflictingAttributes { attr1, attr2, span } => {
                format_err!(
                    *span,
                    "cannot use both #[{}] and #[{}] on the same item",
                    attr1,
                    attr2
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
