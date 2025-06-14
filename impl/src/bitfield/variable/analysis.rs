use crate::bitfield::{
    config::{Config, VariableBitsConfig},
    field_info::FieldInfo,
    BitfieldStruct,
};
use super::{
    errors::VariableBitsError,
    field_config::VariantRole,
    field_config_ext::VariableFieldConfigExt,
};
use syn::{self, spanned::Spanned as _};

/// Analysis result for variable-size structs
pub struct VariableStructAnalysis {
    pub discriminator_field_index: usize, // Index of field marked with #[variant_discriminator]
    pub _data_field_index: usize,         // Index of field marked with #[variant_data]
    pub _fixed_field_indices: Vec<usize>, // Indices of all other fields
    pub sizes: Vec<usize>,                // Total struct sizes for each configuration
    pub fixed_bits: usize,                // Total bits used by non-variant fields
    pub data_enum_type: syn::Type,        // Type of the variant data field
    pub discriminator_bits: usize,        // Number of bits in discriminator field
}

/// Extension trait for variable bits analysis
pub trait VariableBitsAnalysis {
    /// Analyze variable-size struct configuration
    fn analyze_variable_bits(
        &self,
        config: &Config,
    ) -> syn::Result<Option<VariableStructAnalysis>>;
}

impl VariableBitsAnalysis for BitfieldStruct {
    fn analyze_variable_bits(
        &self,
        config: &Config,
    ) -> syn::Result<Option<VariableStructAnalysis>> {
        let variable_config = match &config.variable_bits {
            Some(config_value) => &config_value.value,
            None => return Ok(None), // Not a variable-size struct
        };

        // Collect all field infos
        let field_infos: Vec<_> = self.field_infos(config).collect();

        // Find discriminator and data fields
        let mut discriminator_field: Option<(usize, &FieldInfo<'_>)> = None;
        let mut data_field: Option<(usize, &FieldInfo<'_>)> = None;
        let mut fixed_fields = Vec::new();

        for (index, field_info) in field_infos.iter().enumerate() {
            let variant_role = field_info.config.variant_role(field_info.index, &config.variable_field_configs);
            match variant_role {
                Some(VariantRole::Discriminator) => {
                    if let Some((_, existing_field)) = discriminator_field {
                        return Err(VariableBitsError::MultipleVariantFields {
                            field_type: "discriminator",
                            first_span: existing_field.field.span(),
                            second_span: field_info.field.span(),
                        }
                        .to_syn_error());
                    }
                    discriminator_field = Some((index, field_info));
                }
                Some(VariantRole::Data) => {
                    if let Some((_, existing_field)) = data_field {
                        return Err(VariableBitsError::MultipleVariantFields {
                            field_type: "data",
                            first_span: existing_field.field.span(),
                            second_span: field_info.field.span(),
                        }
                        .to_syn_error());
                    }
                    data_field = Some((index, field_info));
                }
                None => {
                    fixed_fields.push((index, field_info));
                }
            }
        }

        let (discriminator_field_index, discriminator_field) =
            discriminator_field.ok_or_else(|| {
                VariableBitsError::MissingVariantField {
                    field_type: "discriminator",
                    span: self.item_struct.span(),
                }
                .to_syn_error()
            })?;

        let (data_field_index, data_field) = data_field.ok_or_else(|| {
            VariableBitsError::MissingVariantField {
                field_type: "data",
                span: self.item_struct.span(),
            }
            .to_syn_error()
        })?;

        // Calculate discriminator bits
        let discriminator_bits = if let Some(bits_config) = &discriminator_field.config.bits {
            bits_config.value
        } else {
            extract_bits_from_type(&discriminator_field.field.ty)?
        };

        // Calculate fixed field bits
        let fixed_bits = calculate_fixed_field_bits(
            &fixed_fields,
            &(discriminator_field_index, discriminator_field),
        )?;

        // Determine struct sizes
        let sizes = match variable_config {
            VariableBitsConfig::Explicit(sizes) => {
                // Validate sizes are achievable with fixed fields
                for &total_size in sizes {
                    if total_size <= fixed_bits {
                        return Err(format_err!(
                            data_field.field.span(),
                            "total size {} too small for fixed fields requiring {} bits",
                            total_size,
                            fixed_bits
                        ));
                    }
                }
                sizes.clone()
            }
            VariableBitsConfig::Inferred => {
                // For struct inference, we generate sizes that accommodate all possible
                // data variants. This requires the data enum to have variable_bits too.
                // For now, require explicit specification for structs.
                return Err(format_err!(
                    self.item_struct.span(),
                    "inferred variable_bits requires explicit tuple for structs, e.g., #[variable_bits = (32, 64, 96)]"
                ));
            }
        };

        // Validate discriminator field can hold enough values
        validate_discriminator_capacity(
            &(discriminator_field_index, discriminator_field),
            sizes.len(),
        )?;

        // Validate data field type compatibility (will be checked at compile-time too)
        let data_enum_type = data_field.field.ty.clone();

        Ok(Some(VariableStructAnalysis {
            discriminator_field_index,
            _data_field_index: data_field_index,
            _fixed_field_indices: fixed_fields.iter().map(|(index, _)| *index).collect(),
            sizes,
            fixed_bits,
            data_enum_type,
            discriminator_bits,
        }))
    }
}

// Helper functions

fn calculate_fixed_field_bits(
    fixed_fields: &[(usize, &FieldInfo<'_>)],
    discriminator_field: &(usize, &FieldInfo<'_>),
) -> syn::Result<usize> {
    let mut total_bits = 0;

    // Add discriminator field bits
    let (_, discriminator_field_info) = discriminator_field;
    if let Some(bits_config) = &discriminator_field_info.config.bits {
        total_bits += bits_config.value;
    } else {
        // Try to extract from type (e.g., B4 -> 4 bits)
        total_bits += extract_bits_from_type(&discriminator_field_info.field.ty)?;
    }

    // Add fixed field bits
    for (_, field_info) in fixed_fields {
        if let Some(bits_config) = &field_info.config.bits {
            total_bits += bits_config.value;
        } else {
            // Try to extract from type
            total_bits += extract_bits_from_type(&field_info.field.ty)?;
        }
    }

    Ok(total_bits)
}

fn validate_discriminator_capacity(
    discriminator_field: &(usize, &FieldInfo<'_>),
    num_variants: usize,
) -> syn::Result<()> {
    let (_, discriminator_field_info) = discriminator_field;
    let discriminator_bits = if let Some(bits_config) = &discriminator_field_info.config.bits {
        bits_config.value
    } else {
        extract_bits_from_type(&discriminator_field_info.field.ty)?
    };

    let max_variants = 1 << discriminator_bits;
    if num_variants > max_variants {
        return Err(VariableBitsError::InvalidDiscriminantRange {
            discriminant: num_variants - 1,
            max_allowed: max_variants - 1,
            discriminator_bits,
            span: discriminator_field_info.field.span(),
        }
        .to_syn_error());
    }

    Ok(())
}

/// Extract bit count from types like B4, B8, etc.
fn extract_bits_from_type(ty: &syn::Type) -> syn::Result<usize> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = &segment.ident;
            let ident_str = ident.to_string();

            // Check if it's a B<N> type
            if ident_str.starts_with('B') && ident_str.len() > 1 {
                if let Ok(bits) = ident_str[1..].parse::<usize>() {
                    if bits > 0 && bits <= 128 {
                        return Ok(bits);
                    }
                }
            }
        }
    }

    Err(format_err!(
        ty.span(),
        "could not extract bit count from type - expected B<N> type like B4, B8, etc."
    ))
}