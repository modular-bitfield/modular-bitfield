use crate::bitfield::{
    config::Config,
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
#[allow(dead_code)]
pub struct VariableStructAnalysis {
    pub discriminator_field_index: usize, // Index of field marked with #[variant_discriminator]
    pub _data_field_index: usize,         // Index of field marked with #[variant_data]
    pub _fixed_field_indices: Vec<usize>, // Indices of all other fields
    pub sizes: Vec<usize>,                // Total struct sizes for each configuration
    pub fixed_bits: usize,                // Total bits used by non-variant fields
    pub data_enum_type: syn::Type,        // Type of the variant data field
    pub discriminator_bits: usize,        // Number of bits in discriminator field
}

// Manual Debug implementation that skips the syn::Type field
impl core::fmt::Debug for VariableStructAnalysis {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VariableStructAnalysis")
            .field("discriminator_field_index", &self.discriminator_field_index)
            .field("_data_field_index", &self._data_field_index)
            .field("_fixed_field_indices", &self._fixed_field_indices)
            .field("sizes", &self.sizes)
            .field("fixed_bits", &self.fixed_bits)
            .field("data_enum_type", &"<syn::Type>")
            .field("discriminator_bits", &self.discriminator_bits)
            .finish()
    }
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
        // Check if bits config is variable
        let sizes = match &config.bits {
            Some(config_value) => match config_value.value.sizes() {
                Some(sizes) => sizes,
                None => return Ok(None), // Fixed-size struct
            },
            None => return Ok(None), // No bits config
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
            sizes: sizes.to_vec(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitfield::{BitfieldStruct, config::{Config, ConfigValue}};
    use syn::parse_quote;
    use proc_macro2::Span;

    #[test]
    fn test_extract_bits_from_type_valid() {
        // Test valid B<N> types
        let test_cases = vec![
            ("B1", 1),
            ("B4", 4),
            ("B8", 8),
            ("B16", 16),
            ("B32", 32),
            ("B64", 64),
            ("B128", 128),
            ("B17", 17),  // Odd size
            ("B99", 99),  // Large odd size
        ];

        for (type_name, expected_bits) in test_cases {
            let ty: syn::Type = match type_name {
                "B1" => syn::parse_quote! { B1 },
                "B4" => syn::parse_quote! { B4 },
                "B8" => syn::parse_quote! { B8 },
                "B16" => syn::parse_quote! { B16 },
                "B32" => syn::parse_quote! { B32 },
                "B64" => syn::parse_quote! { B64 },
                "B128" => syn::parse_quote! { B128 },
                "B17" => syn::parse_quote! { B17 },
                "B99" => syn::parse_quote! { B99 },
                _ => panic!("Unknown type"),
            };
            let result = extract_bits_from_type(&ty).unwrap();
            assert_eq!(result, expected_bits, "Failed for type {}", type_name);
        }
    }

    #[test]
    fn test_extract_bits_from_type_invalid() {
        // Test invalid types
        let invalid_types = vec![
            "u8",       // Not a B type
            "B",        // No number
            "B0",       // Zero bits
            "B129",     // Too many bits
            "B1000",    // Way too many bits
            "Bits8",    // Wrong prefix
            "8B",       // Wrong order
            "B8u",      // Extra suffix
            "Option<B8>", // Generic type
        ];

        for type_name in invalid_types {
            let ty: syn::Type = match type_name {
                "u8" => syn::parse_quote! { u8 },
                "B" => syn::parse_quote! { B },
                "B0" => syn::parse_quote! { B0 },
                "B129" => syn::parse_quote! { B129 },
                "B1000" => syn::parse_quote! { B1000 },
                "Bits8" => syn::parse_quote! { Bits8 },
                "8B" => continue, // Can't parse this as valid Rust
                "B8u" => continue, // Can't parse this as valid Rust
                "Option<B8>" => syn::parse_quote! { Option<B8> },
                _ => panic!("Unknown type"),
            };
            let result = extract_bits_from_type(&ty);
            assert!(result.is_err(), "Expected error for type {}", type_name);
        }
    }

    #[test]
    fn test_extract_bits_from_path_type() {
        // Test with qualified paths - it actually looks at the last segment which is B16
        let ty: syn::Type = syn::parse_quote! { crate::B16 };
        let result = extract_bits_from_type(&ty);
        // This will succeed because the last segment is B16
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 16);

        // Test simple path
        let ty: syn::Type = syn::parse_quote! { B16 };
        let result = extract_bits_from_type(&ty).unwrap();
        assert_eq!(result, 16);
    }

    #[test]
    fn test_analyze_variable_bits_no_config() {
        // Test when no variable_bits config is present
        let input: syn::ItemStruct = parse_quote! {
            struct TestStruct {
                field1: B8,
                field2: B16,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input.clone() };
        let config = Config::default();
        
        let result = bitfield.analyze_variable_bits(&config).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_analyze_variable_bits_simple() {
        // Test basic variable bits analysis
        let input: syn::ItemStruct = parse_quote! {
            #[bitfield(variable_bits = (32, 64))]
            struct TestStruct {
                #[variant_discriminator]
                discriminator: B2,
                #[variant_data]
                data: DataEnum,
                fixed_field: B6,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input.clone() };
        let mut config = Config::default();
        config.bits = Some(ConfigValue {
            span: Span::call_site(),
            value: crate::bitfield::config::BitsConfig::Variable(vec![32, 64]),
        });
        
        // Add field configs
        config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
        config.variable_field_configs.set_variant_data(1, Span::call_site()).unwrap();
        
        let result = bitfield.analyze_variable_bits(&config);
        assert!(result.is_ok());
        let analysis = result.unwrap().unwrap();
        
        assert_eq!(analysis.discriminator_field_index, 0);
        assert_eq!(analysis._data_field_index, 1);
        assert_eq!(analysis._fixed_field_indices, vec![2]);
        assert_eq!(analysis.sizes, vec![32, 64]);
        assert_eq!(analysis.discriminator_bits, 2);
        assert_eq!(analysis.fixed_bits, 8); // 2 bits discriminator + 6 bits fixed field
    }

    #[test]
    fn test_analyze_variable_bits_missing_discriminator() {
        // Test error when discriminator field is missing
        let input: syn::ItemStruct = parse_quote! {
            #[bitfield(variable_bits = (32, 64))]
            struct TestStruct {
                #[variant_data]
                data: DataEnum,
                fixed_field: B6,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input.clone() };
        let mut config = Config::default();
        config.bits = Some(ConfigValue {
            span: Span::call_site(),
            value: crate::bitfield::config::BitsConfig::Variable(vec![32, 64]),
        });
        
        // Only add data field config
        config.variable_field_configs.set_variant_data(0, Span::call_site()).unwrap();
        
        let result = bitfield.analyze_variable_bits(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("discriminator"));
    }

    #[test]
    fn test_analyze_variable_bits_missing_data() {
        // Test error when data field is missing
        let input: syn::ItemStruct = parse_quote! {
            #[bitfield(variable_bits = (32, 64))]
            struct TestStruct {
                #[variant_discriminator]
                discriminator: B2,
                fixed_field: B6,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input.clone() };
        let mut config = Config::default();
        config.bits = Some(ConfigValue {
            span: Span::call_site(),
            value: crate::bitfield::config::BitsConfig::Variable(vec![32, 64]),
        });
        
        // Only add discriminator field config
        config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
        
        let result = bitfield.analyze_variable_bits(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("data"));
    }

    #[test]
    fn test_analyze_variable_bits_multiple_discriminators() {
        // Test error when multiple discriminator fields exist
        let input: syn::ItemStruct = parse_quote! {
            #[bitfield(variable_bits = (32, 64))]
            struct TestStruct {
                #[variant_discriminator]
                discriminator1: B2,
                #[variant_discriminator]
                discriminator2: B2,
                #[variant_data]
                data: DataEnum,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input.clone() };
        let mut config = Config::default();
        config.bits = Some(ConfigValue {
            span: Span::call_site(),
            value: crate::bitfield::config::BitsConfig::Variable(vec![32, 64]),
        });
        
        // Add multiple discriminator configs
        config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
        config.variable_field_configs.set_variant_discriminator(1, Span::call_site()).unwrap();
        config.variable_field_configs.set_variant_data(2, Span::call_site()).unwrap();
        
        let result = bitfield.analyze_variable_bits(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("discriminator"));
    }

    #[test]
    fn test_analyze_variable_bits_size_too_small() {
        // Test error when total size is too small for fixed fields
        let input: syn::ItemStruct = parse_quote! {
            #[bitfield(variable_bits = (8, 16))]
            struct TestStruct {
                #[variant_discriminator]
                discriminator: B4,
                #[variant_data]
                data: DataEnum,
                fixed_field: B8,
            }
        };
        
        let bitfield = BitfieldStruct { item_struct: input.clone() };
        let mut config = Config::default();
        config.bits = Some(ConfigValue {
            span: Span::call_site(),
            value: crate::bitfield::config::BitsConfig::Variable(vec![8, 16]),
        });
        
        config.variable_field_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
        config.variable_field_configs.set_variant_data(1, Span::call_site()).unwrap();
        
        let result = bitfield.analyze_variable_bits(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("too small"));
    }

    #[test]
    fn test_validate_discriminator_capacity() {
        // Test discriminator capacity validation
        let input: syn::ItemStruct = parse_quote! {
            struct TestStruct {
                discriminator: B1,
            }
        };
        
        let field = &input.fields.iter().next().unwrap();
        let field_info = crate::bitfield::field_info::FieldInfo {
            field,
            index: 0,
            config: crate::bitfield::field_config::FieldConfig::default(),
        };
        
        // Test with 2 variants (fits in 1 bit)
        let result = validate_discriminator_capacity(&(0, &field_info), 2);
        assert!(result.is_ok());
        
        // Test with 3 variants (doesn't fit in 1 bit)
        let result = validate_discriminator_capacity(&(0, &field_info), 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_fixed_field_bits() {
        // Test fixed field bits calculation
        let input: syn::ItemStruct = parse_quote! {
            struct TestStruct {
                discriminator: B2,
                field1: B8,
                field2: B16,
            }
        };
        
        let fields: Vec<_> = input.fields.iter().collect();
        
        let discriminator_info = crate::bitfield::field_info::FieldInfo {
            field: fields[0],
            index: 0,
            config: crate::bitfield::field_config::FieldConfig::default(),
        };
        
        let fixed_infos: Vec<_> = fields[1..].iter().enumerate().map(|(i, field)| {
            crate::bitfield::field_info::FieldInfo {
                field: *field,
                index: i + 1,
                config: crate::bitfield::field_config::FieldConfig::default(),
            }
        }).collect();
        
        let fixed_refs: Vec<_> = fixed_infos.iter().enumerate().map(|(i, info)| (i + 1, info)).collect();
        let result = calculate_fixed_field_bits(&fixed_refs, &(0, &discriminator_info));
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 26); // 2 + 8 + 16
    }

    #[test]
    fn test_debug_impl() {
        // Test Debug implementation for VariableStructAnalysis
        let analysis = VariableStructAnalysis {
            discriminator_field_index: 0,
            _data_field_index: 1,
            _fixed_field_indices: vec![2, 3],
            sizes: vec![32, 64],
            fixed_bits: 8,
            data_enum_type: parse_quote! { DataEnum },
            discriminator_bits: 2,
        };
        
        let debug_str = format!("{:?}", analysis);
        assert!(debug_str.contains("VariableStructAnalysis"));
        assert!(debug_str.contains("discriminator_field_index: 0"));
        assert!(debug_str.contains("sizes: [32, 64]"));
        assert!(debug_str.contains("<syn::Type>"));
    }
}