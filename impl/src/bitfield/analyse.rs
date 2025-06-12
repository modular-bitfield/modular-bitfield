use super::{
    config::{Config, ReprKind, VariableBitsConfig},
    field_config::{FieldConfig, SkipWhich},
    field_info::{FieldInfo, VariantRole},
    raise_skip_error,
    variable_bits_errors::VariableBitsError,
    BitfieldStruct,
};
use core::convert::TryFrom;
use quote::quote;
use syn::{self, parse::Result, spanned::Spanned as _};

impl TryFrom<(&mut Config, syn::ItemStruct)> for BitfieldStruct {
    type Error = syn::Error;

    fn try_from((config, item_struct): (&mut Config, syn::ItemStruct)) -> Result<Self> {
        Self::ensure_has_fields(&item_struct)?;
        Self::ensure_valid_generics(&item_struct)?;
        Self::extract_attributes(&item_struct.attrs, config)?;
        Self::analyse_config_for_fields(&item_struct, config)?;
        config.ensure_no_conflicts()?;
        Ok(Self { item_struct })
    }
}

impl BitfieldStruct {
    /// Returns an error if the input struct does not have any fields.
    fn ensure_has_fields(item_struct: &syn::ItemStruct) -> Result<()> {
        if matches!(&item_struct.fields, syn::Fields::Unit)
            || matches!(&item_struct.fields, syn::Fields::Unnamed(f) if f.unnamed.is_empty())
            || matches!(&item_struct.fields, syn::Fields::Named(f) if f.named.is_empty())
        {
            return Err(format_err_spanned!(
                item_struct,
                "encountered invalid bitfield struct without fields"
            ));
        }
        Ok(())
    }

    /// Returns an error if the input struct contains generics that cannot be
    /// used in a const expression.
    fn ensure_valid_generics(item_struct: &syn::ItemStruct) -> Result<()> {
        if item_struct.generics.type_params().next().is_some()
            || item_struct.generics.lifetimes().next().is_some()
        {
            return Err(format_err_spanned!(
                item_struct.generics,
                "bitfield structs can only use const generics"
            ));
        }
        Ok(())
    }

    /// Extracts the `#[repr(uN)]` annotations from the given `#[bitfield]` struct.
    fn extract_repr_attribute(attr: &syn::Attribute, config: &mut Config) -> Result<()> {
        let list = attr.meta.require_list()?;
        let mut retained_reprs = vec![];
        attr.parse_nested_meta(|meta| {
            let path = &meta.path;
            let repr_kind = if path.is_ident("u8") {
                Some(ReprKind::U8)
            } else if path.is_ident("u16") {
                Some(ReprKind::U16)
            } else if path.is_ident("u32") {
                Some(ReprKind::U32)
            } else if path.is_ident("u64") {
                Some(ReprKind::U64)
            } else if path.is_ident("u128") {
                Some(ReprKind::U128)
            } else {
                // If other repr such as `transparent` or `C` have been found we
                // are going to re-expand them into a new `#[repr(..)]` that is
                // ignored by the rest of this macro.
                retained_reprs.push(path.clone());
                None
            };
            if let Some(repr_kind) = repr_kind {
                config.repr(repr_kind, path.span())?;
            }
            Ok(())
        })?;
        if !retained_reprs.is_empty() {
            // We only push back another re-generated `#[repr(..)]` if its contents
            // contained some non-bitfield representations and thus is not empty.
            let retained_reprs_tokens = quote! {
                #( #retained_reprs ),*
            };
            config.push_retained_attribute(syn::Attribute {
                pound_token: attr.pound_token,
                style: attr.style,
                bracket_token: attr.bracket_token,
                meta: syn::Meta::List(syn::MetaList {
                    path: list.path.clone(),
                    delimiter: list.delimiter.clone(),
                    tokens: retained_reprs_tokens,
                }),
            });
        }
        Ok(())
    }

    /// Extracts the `#[derive(Debug)]` annotations from the given `#[bitfield]` struct.
    fn extract_derive_debug_attribute(attr: &syn::Attribute, config: &mut Config) -> Result<()> {
        let list = attr.meta.require_list()?;
        let mut retained_derives = vec![];
        attr.parse_nested_meta(|meta| {
            let path = &meta.path;
            if path.is_ident("Debug") {
                config.derive_debug(path.span())?;
            } else if path.is_ident("BitfieldSpecifier") {
                config.deprecated_specifier(path.span());
                config.derive_specifier(path.span())?;
            } else if path.is_ident("Specifier") {
                config.derive_specifier(path.span())?;
            } else {
                // Other derives are going to be re-expanded them into a new
                // `#[derive(..)]` that is ignored by the rest of this macro.
                retained_derives.push(path.clone());
            }
            Ok(())
        })?;
        if !retained_derives.is_empty() {
            // We only push back another re-generated `#[derive(..)]` if its contents
            // contain some remaining derives and thus is not empty.
            let retained_derives_tokens = quote! {
                #( #retained_derives ),*
            };
            config.push_retained_attribute(syn::Attribute {
                pound_token: attr.pound_token,
                style: attr.style,
                bracket_token: attr.bracket_token,
                meta: syn::Meta::List(syn::MetaList {
                    path: list.path.clone(),
                    delimiter: list.delimiter.clone(),
                    tokens: retained_derives_tokens,
                }),
            });
        }
        Ok(())
    }

    /// Analyses and extracts the `#[repr(uN)]` or other annotations from the given struct.
    fn extract_attributes(attributes: &[syn::Attribute], config: &mut Config) -> Result<()> {
        for attr in attributes {
            if attr.path().is_ident("repr") {
                Self::extract_repr_attribute(attr, config)?;
            } else if attr.path().is_ident("derive") {
                Self::extract_derive_debug_attribute(attr, config)?;
            } else {
                config.push_retained_attribute(attr.clone());
            }
        }
        Ok(())
    }

    /// Analyses and extracts the configuration for all bitfield fields.
    fn analyse_config_for_fields(item_struct: &syn::ItemStruct, config: &mut Config) -> Result<()> {
        for (index, field) in Self::fields(item_struct) {
            let span = field.span();
            let field_config = Self::extract_field_config(field)?;
            config.field_config(index, span, field_config)?;
        }
        Ok(())
    }

    /// Extracts the `#[bits = N]` and `#[skip(..)]` attributes for a given field.
    fn extract_field_config(field: &syn::Field) -> Result<FieldConfig> {
        let mut config = FieldConfig::default();
        for attr in &field.attrs {
            if attr.path().is_ident("bits") {
                let name_value = attr.meta.require_name_value()?;
                let span = name_value.span();
                match &name_value.value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(lit_int),
                        ..
                    }) => {
                        config.bits(lit_int.base10_parse::<usize>()?, span)?;
                    }
                    value => {
                        return Err(format_err!(
                            value.span(),
                            "encountered invalid value type for #[bits = N]"
                        ))
                    }
                }
            } else if attr.path().is_ident("skip") {
                match &attr.meta {
                    syn::Meta::Path(path) => {
                        assert!(path.is_ident("skip"));
                        config.skip(SkipWhich::All, path.span())?;
                    }
                    syn::Meta::List(meta_list) => {
                        let (mut getters, mut setters) = (None, None);
                        meta_list.parse_nested_meta(|meta| {
                            let path = &meta.path;
                            if path.is_ident("getters") {
                                if let Some(previous) = getters {
                                    return raise_skip_error("(getters)", path.span(), previous);
                                }
                                getters = Some(path.span());
                            } else if path.is_ident("setters") {
                                if let Some(previous) = setters {
                                    return raise_skip_error("(setters)", path.span(), previous);
                                }
                                setters = Some(path.span());
                            } else {
                                return Err(meta.error(
                                    "encountered unknown or unsupported #[skip(..)] specifier",
                                ));
                            }
                            Ok(())
                        })?;
                        if getters.is_some() == setters.is_some() {
                            config.skip(SkipWhich::All, meta_list.path.span())?;
                        } else if getters.is_some() {
                            config.skip(SkipWhich::Getters, meta_list.path.span())?;
                        } else {
                            config.skip(SkipWhich::Setters, meta_list.path.span())?;
                        }
                    }
                    meta @ syn::Meta::NameValue(..) => {
                        return Err(format_err!(
                            meta.span(),
                            "encountered invalid format for #[skip] field attribute"
                        ))
                    }
                }
            } else if attr.path().is_ident("variant_discriminator") {
                match &attr.meta {
                    syn::Meta::Path(path) => {
                        config.variant_discriminator(path.span())?;
                    }
                    _ => {
                        return Err(format_err!(
                            attr.span(),
                            "encountered invalid format for #[variant_discriminator] field attribute, expected #[variant_discriminator]"
                        ))
                    }
                }
            } else if attr.path().is_ident("variant_data") {
                match &attr.meta {
                    syn::Meta::Path(path) => {
                        config.variant_data(path.span())?;
                    }
                    _ => {
                        return Err(format_err!(
                            attr.span(),
                            "encountered invalid format for #[variant_data] field attribute, expected #[variant_data]"
                        ))
                    }
                }
            } else {
                config.retain_attr(attr.clone());
            }
        }
        Ok(config)
    }
}

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

impl BitfieldStruct {
    /// Analyze variable-size struct configuration
    pub fn analyze_variable_bits(
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
            match field_info.variant_role {
                Some(VariantRole::Discriminator) => {
                    if discriminator_field.is_some() {
                        return Err(VariableBitsError::MultipleVariantFields {
                            field_type: "discriminator",
                            first_span: discriminator_field.unwrap().1.field.span(),
                            second_span: field_info.field.span(),
                        }
                        .to_syn_error());
                    }
                    discriminator_field = Some((index, field_info));
                }
                Some(VariantRole::Data) => {
                    if data_field.is_some() {
                        return Err(VariableBitsError::MultipleVariantFields {
                            field_type: "data",
                            first_span: data_field.unwrap().1.field.span(),
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
