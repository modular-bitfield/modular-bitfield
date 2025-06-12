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
    pub fn to_syn_error(self) -> syn::Error {
        match self {
            Self::TupleSizeMismatch {
                expected,
                found,
                span,
            } => {
                format_err!(
                    span,
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
                    span,
                    "variant {} data type {} must have exactly {} bits for variable_bits compatibility",
                    variant_name, actual_bits, expected_bits
                )
            }
            Self::MissingVariantField { field_type, span } => {
                format_err!(
                    span,
                    "variable_bits structs require exactly one #[variant_{}] field",
                    field_type
                )
            }
            Self::MultipleVariantFields {
                field_type,
                first_span,
                second_span,
            } => format_err!(
                second_span,
                "found multiple #[variant_{}] fields, only one allowed",
                field_type
            )
            .into_combine(format_err!(
                first_span,
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
                    span,
                    "discriminant value {} exceeds maximum {} for {}-bit discriminator field",
                    discriminant,
                    max_allowed,
                    discriminator_bits
                )
            }
            Self::ConflictingAttributes { attr1, attr2, span } => {
                format_err!(
                    span,
                    "cannot use both #[{}] and #[{}] on the same item",
                    attr1,
                    attr2
                )
            }
        }
    }
}
