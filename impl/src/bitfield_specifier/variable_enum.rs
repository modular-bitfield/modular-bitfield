use crate::bitfield::VariableBitsError;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use std::collections::HashSet;
use syn::spanned::Spanned as _;

/// Variable enum implementation
/// This contains all the new logic for variable-size enums with data

#[derive(Debug, Clone)]
pub enum BitsConfig {
    Fixed(usize),         // #[bits = 32]
    Variable(Vec<usize>), // #[bits = (8, 16, 32)]
}

pub struct Attributes {
    pub bits: Option<BitsConfig>,
    pub discriminant_bits: Option<usize>, // For #[discriminant_bits = N]
}

enum VariantType {
    Unit,                 // No data
    Data(Box<syn::Type>), // Has data of specified type
}

struct EnumVariant {
    name: syn::Ident,
    variant_type: VariantType,
    discriminant: Option<usize>,  // From #[discriminant = N]
    explicit_bits: Option<usize>, // From #[bits = N] on variant
    span: proc_macro2::Span,
}

// Removed EnumAnalysis as it's not used in the refactored code

struct VariableEnumAnalysis {
    variants: Vec<EnumVariant>,
    variant_sizes: Vec<usize>, // Sizes for each variant (parallel to variants)
    max_size: usize,           // Maximum variant size
    _has_discriminants: bool,  // True if any variant has explicit discriminant
    discriminant_bits: Option<usize>, // External discriminant bits (from parent struct)
}

pub fn parse_attrs(attrs: &[syn::Attribute]) -> syn::Result<Attributes> {
    let mut attributes = Attributes {
        bits: None,
        discriminant_bits: None,
    };

    for attr in attrs {
        if attr.path().is_ident("bits") {
            if attributes.bits.is_some() {
                return Err(format_err_spanned!(
                    attr,
                    "More than one 'bits' attribute is not permitted",
                ));
            }

            match &attr.meta {
                syn::Meta::NameValue(meta) => {
                    match &meta.value {
                        // #[bits = 32]
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Int(lit),
                            ..
                        }) => {
                            attributes.bits = Some(BitsConfig::Fixed(lit.base10_parse::<usize>()?));
                        }
                        // #[bits = (8, 16, 32)]
                        syn::Expr::Tuple(tuple) => {
                            let mut sizes = Vec::new();
                            for elem in &tuple.elems {
                                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit), .. }) = elem {
                                    let size = lit.base10_parse::<usize>()?;
                                    
                                    // Validate size constraints
                                    if size == 0 {
                                        return Err(format_err_spanned!(lit, "bits sizes must be greater than 0"));
                                    }
                                    if size > 128 {
                                        return Err(format_err_spanned!(lit, "bits sizes cannot exceed 128 bits"));
                                    }
                                    
                                    sizes.push(size);
                                } else {
                                    return Err(format_err_spanned!(elem, "expected integer literal in bits tuple"));
                                }
                            }
                            if sizes.is_empty() {
                                return Err(format_err_spanned!(tuple, "bits tuple cannot be empty"));
                            }
                            attributes.bits = Some(BitsConfig::Variable(sizes));
                        }
                        _ => {
                            return Err(format_err_spanned!(
                                meta,
                                "bits must be integer literal for fixed size: #[bits = N]"
                            ));
                        }
                    }
                }
                syn::Meta::List(meta_list) => {
                    // #[bits(8, 16, 32)]
                    let content = &meta_list.tokens;

                    // Custom parser for the comma-separated list of integers
                    #[allow(clippy::items_after_statements)]
                    struct IntList(Vec<syn::LitInt>);

                    #[allow(clippy::items_after_statements)]
                    impl syn::parse::Parse for IntList {
                        fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
                            let parsed = syn::punctuated::Punctuated::<syn::LitInt, syn::Token![,]>::parse_terminated(input)?;
                            Ok(IntList(parsed.into_iter().collect()))
                        }
                    }

                    let parsed_content: IntList = syn::parse2(content.clone())?;

                    let mut sizes = Vec::new();
                    for lit in parsed_content.0 {
                        let size = lit.base10_parse::<usize>()?;
                        
                        // Validate size constraints
                        if size == 0 {
                            return Err(format_err_spanned!(lit, "bits sizes must be greater than 0"));
                        }
                        if size > 128 {
                            return Err(format_err_spanned!(lit, "bits sizes cannot exceed 128 bits"));
                        }
                        
                        sizes.push(size);
                    }

                    if sizes.is_empty() {
                        return Err(format_err_spanned!(meta_list, "bits list cannot be empty"));
                    }
                    attributes.bits = Some(BitsConfig::Variable(sizes));
                }
                syn::Meta::Path(_) => {
                    return Err(format_err_spanned!(
                        attr,
                        "bits attribute requires a value: #[bits = N], #[bits = (N, M, ...)], or #[bits(N, M, ...)]"
                    ));
                }
            }
        } else if attr.path().is_ident("discriminant_bits") {
            if attributes.discriminant_bits.is_some() {
                return Err(format_err_spanned!(
                    attr,
                    "More than one 'discriminant_bits' attribute is not permitted",
                ));
            }
            let meta = attr.meta.require_name_value()?;
            attributes.discriminant_bits = if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit),
                ..
            }) = &meta.value
            {
                let bits = lit.base10_parse::<usize>()?;
                if bits == 0 || bits > 64 {
                    return Err(format_err_spanned!(
                        lit,
                        "discriminant_bits must be between 1 and 64",
                    ));
                }
                Some(bits)
            } else {
                return Err(format_err_spanned!(
                    attr,
                    "discriminant_bits must be in form #[discriminant_bits = N]",
                ));
            };
        }
    }
    Ok(attributes)
}

fn parse_variant_attrs(variant: &syn::Variant) -> syn::Result<(Option<usize>, Option<usize>)> {
    let mut discriminant = None;
    let mut explicit_bits = None;

    for attr in &variant.attrs {
        if attr.path().is_ident("discriminant") {
            if discriminant.is_some() {
                return Err(format_err_spanned!(
                    attr,
                    "duplicate #[discriminant] attribute on variant",
                ));
            }

            let meta = attr.meta.require_name_value()?;
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit),
                ..
            }) = &meta.value
            {
                discriminant = Some(lit.base10_parse::<usize>()?);
            } else {
                return Err(format_err_spanned!(
                    attr,
                    "discriminant value must be an integer literal",
                ));
            }
        } else if attr.path().is_ident("bits") {
            if explicit_bits.is_some() {
                return Err(format_err_spanned!(
                    attr,
                    "duplicate #[bits] attribute on variant",
                ));
            }

            let meta = attr.meta.require_name_value()?;
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit),
                ..
            }) = &meta.value
            {
                explicit_bits = Some(lit.base10_parse::<usize>()?);
            } else {
                return Err(format_err_spanned!(
                    attr,
                    "bits value must be an integer literal",
                ));
            }
        }
    }

    Ok((discriminant, explicit_bits))
}

fn validate_discriminant_values(variants: &[EnumVariant]) -> syn::Result<()> {
    let mut used_discriminants = HashSet::new();

    for (index, variant) in variants.iter().enumerate() {
        let discriminant = variant.discriminant.unwrap_or(index);

        // Check for duplicates
        if !used_discriminants.insert(discriminant) {
            return Err(format_err!(
                variant.span,
                "duplicate discriminant value {} (variant {} conflicts with previous variant)",
                discriminant,
                variant.name
            ));
        }

        // No need to validate discriminant size since we're using usize
    }

    Ok(())
}

fn analyze_variable_enum(
    input: &syn::ItemEnum,
    attributes: &Attributes,
) -> syn::Result<VariableEnumAnalysis> {
    let bits_config = attributes
        .bits
        .as_ref()
        .ok_or_else(|| format_err!(input, "variable enum requires #[bits] attribute"))?;

    // Parse all variants with their attributes
    let mut variants = Vec::new();
    for variant in &input.variants {
        let (discriminant, explicit_bits) = parse_variant_attrs(variant)?;

        let variant_type = match &variant.fields {
            syn::Fields::Unit => VariantType::Unit,
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                VariantType::Data(Box::new(fields.unnamed.first().unwrap().ty.clone()))
            }
            syn::Fields::Named(_) => {
                return Err(format_err_spanned!(
                    variant,
                    "named fields in enum variants are not supported for variable_bits"
                ));
            }
            syn::Fields::Unnamed(_) => {
                return Err(format_err_spanned!(
                    variant,
                    "multiple fields in enum variants are not supported"
                ));
            }
        };

        variants.push(EnumVariant {
            name: variant.ident.clone(),
            variant_type,
            discriminant,
            explicit_bits,
            span: variant.span(),
        });
    }

    // Determine variant sizes based on configuration
    let variant_sizes = match bits_config {
        BitsConfig::Variable(tuple_sizes) => {
            // Validate tuple size matches variant count
            if tuple_sizes.len() != variants.len() {
                return Err(VariableBitsError::TupleSizeMismatch {
                    expected: variants.len(),
                    found: tuple_sizes.len(),
                    span: input.span(),
                }
                .to_syn_error());
            }

            // Cross-validate with any explicit #[bits = N] on variants
            for (index, variant) in variants.iter().enumerate() {
                if let Some(explicit_bits) = variant.explicit_bits {
                    let tuple_bits = tuple_sizes[index];
                    if explicit_bits != tuple_bits {
                        return Err(format_err!(
                            variant.span,
                            "variant #[bits = {}] conflicts with tuple position {} size {}",
                            explicit_bits,
                            index,
                            tuple_bits
                        ));
                    }
                }
            }

            tuple_sizes.clone()
        }
        BitsConfig::Fixed(_size) => {
            // For fixed size enums, all variants should use the same size
            // This is handled by the regular fixed-size enum logic, not variable enum logic
            return Err(format_err!(
                input.span(),
                "fixed size enum should not use variable enum analysis"
            ));
        }
    };

    let max_size = *variant_sizes.iter().max().unwrap_or(&0);
    let has_discriminants = variants.iter().any(|v| v.discriminant.is_some());

    // Validate discriminant values if present
    if has_discriminants {
        validate_discriminant_values(&variants)?;
    }

    Ok(VariableEnumAnalysis {
        variants,
        variant_sizes,
        max_size,
        _has_discriminants: has_discriminants,
        discriminant_bits: attributes.discriminant_bits,
    })
}

pub fn generate_variable_enum(input: &syn::ItemEnum, attributes: Attributes) -> syn::Result<TokenStream2> {
    let enum_ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Check if this enum has variable bits configuration
    if let Some(BitsConfig::Variable(_)) = &attributes.bits {
        // Analyze as variable enum with explicit sizes
        let analysis = analyze_variable_enum(input, &attributes)?;
        return generate_variable_enum_specifier_impl(
            &analysis,
            enum_ident,
            &impl_generics,
            &ty_generics,
            where_clause,
        );
    }

    // If we're here, it's an enum with data variants but fixed size
    // We need to generate code similar to the original but handling data variants
    generate_fixed_size_enum_with_data(input, attributes, enum_ident, &impl_generics, &ty_generics, where_clause)
}

#[allow(clippy::too_many_lines)]
fn generate_variable_enum_specifier_impl(
    analysis: &VariableEnumAnalysis,
    enum_ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
) -> syn::Result<TokenStream2> {
    let span = enum_ident.span();
    let max_size = analysis.max_size;

    // Determine the Bytes type based on max size
    let bytes_type = match max_size {
        0..=8 => quote! { u8 }, // Handle all-unit enums and 1-8 bits
        9..=16 => quote! { u16 },
        17..=32 => quote! { u32 },
        33..=64 => quote! { u64 },
        65..=128 => quote! { u128 },
        _ => {
            return Err(format_err!(
                span,
                "enum requires more than 128 bits, which is not supported"
            ))
        }
    };

    // Generate compile-time assertions for data type sizes
    let size_assertions = analysis.variants.iter().enumerate()
        .filter_map(|(index, variant)| {
            match &variant.variant_type {
                VariantType::Data(data_type) => {
                    let expected_size = analysis.variant_sizes[index];
                    let variant_name = &variant.name;
                    Some(quote! {
                        const _: () = {
                            const VARIANT_SIZE: usize = #expected_size;
                            const DATA_TYPE_SIZE: usize = <#data_type as ::modular_bitfield::Specifier>::BITS;
                            assert!(
                                DATA_TYPE_SIZE >= VARIANT_SIZE,
                                concat!(
                                    "Data type for variant ",
                                    stringify!(#variant_name),
                                    " has ", stringify!(DATA_TYPE_SIZE), " bits, but needs at least ",
                                    stringify!(VARIANT_SIZE), " bits to hold the data"
                                )
                            );
                        };
                    })
                }
                VariantType::Unit => None,
            }
        });

    // Generate into_bytes match arms with bit validation
    let into_bytes_arms: Vec<_> = analysis.variants.iter().enumerate().map(|(index, variant)| {
        let variant_name = &variant.name;
        let variant_span = variant.span;
        let variant_size = analysis.variant_sizes[index];

        match &variant.variant_type {
            VariantType::Unit => {
                quote_spanned!(variant_span=>
                    Self::#variant_name => ::core::result::Result::Ok(0 as #bytes_type)
                )
            },
            VariantType::Data(data_type) => {
                quote_spanned!(variant_span=>
                    Self::#variant_name(data) => {
                        // Convert using the Specifier trait implementation
                        let data_bytes = <#data_type as ::modular_bitfield::Specifier>::into_bytes(data)?;

                        // Validate that the data fits within the variant's bit size
                        // Only validate for sizes less than 64 bits where we can compute max value
                        if #variant_size > 0 && #variant_size < 64 {
                            // We need to cast to u128 for comparison, suppress clippy warnings
                            // since this is generated code and we know the casts are safe
                            #[allow(clippy::cast_lossless)]
                            #[allow(clippy::unnecessary_cast)]
                            let data_value = data_bytes as u128;
                            let variant_max_value = (1u128 << #variant_size) - 1;
                            if data_value > variant_max_value {
                                return ::core::result::Result::Err(::modular_bitfield::error::OutOfBounds);
                            }
                        }

                        ::core::result::Result::Ok(data_bytes as #bytes_type)
                    }
                )
            }
        }
    }).collect();

    // Generate from_bytes - defaults to first variant for external discrimination
    let first_variant_construction = if let Some(first_variant) = analysis.variants.first() {
        let variant_name = &first_variant.name;
        let variant_span = first_variant.span;

        match &first_variant.variant_type {
            VariantType::Unit => {
                quote_spanned!(variant_span=>
                    _ => ::core::result::Result::Ok(Self::#variant_name)
                )
            }
            VariantType::Data(data_type) => {
                // Get the size for the first variant
                let first_variant_size = analysis.variant_sizes[0];
                let data_bytes_cast = match first_variant_size {
                    0..=8 => quote! { bytes as u8 },
                    9..=16 => quote! { bytes as u16 },
                    17..=32 => quote! { bytes as u32 },
                    33..=64 => quote! { bytes as u64 },
                    65..=128 => quote! { bytes as u128 },
                    _ => quote! { bytes },
                };

                quote_spanned!(variant_span=>
                    bytes => {
                        match <#data_type as ::modular_bitfield::Specifier>::from_bytes(#data_bytes_cast) {
                            ::core::result::Result::Ok(data) => ::core::result::Result::Ok(Self::#variant_name(data)),
                            ::core::result::Result::Err(_) => ::core::result::Result::Err(::modular_bitfield::error::InvalidBitPattern::new(bytes)),
                        }
                    }
                )
            }
        }
    } else {
        return Err(format_err!(span, "enum must have at least one variant"));
    };

    // Generate discriminant helper methods
    let discriminant_helpers = generate_enum_discriminant_helpers(analysis, enum_ident)?;

    Ok(quote_spanned!(span=>
        #( #size_assertions )*

        impl #impl_generics ::modular_bitfield::Specifier for #enum_ident #ty_generics #where_clause {
            const BITS: usize = #max_size;
            type Bytes = #bytes_type;
            type InOut = Self;

            #[inline]
            fn into_bytes(input: <Self as ::modular_bitfield::Specifier>::InOut) -> ::core::result::Result<<Self as ::modular_bitfield::Specifier>::Bytes, ::modular_bitfield::error::OutOfBounds> {
                match input {
                    #( #into_bytes_arms, )*
                }
            }

            #[inline]
            fn from_bytes(bytes: <Self as ::modular_bitfield::Specifier>::Bytes) -> ::core::result::Result<<Self as ::modular_bitfield::Specifier>::InOut, ::modular_bitfield::error::InvalidBitPattern<<Self as ::modular_bitfield::Specifier>::Bytes>> {
                // For external discrimination, we default to the first variant
                // User is responsible for using correct constructor based on external discriminant
                match bytes {
                    #first_variant_construction
                }
            }
        }

        #discriminant_helpers
    ))
}

#[allow(clippy::too_many_lines)]
fn generate_enum_discriminant_helpers(
    analysis: &VariableEnumAnalysis,
    enum_ident: &syn::Ident,
) -> syn::Result<TokenStream2> {
    // Check if this enum uses external discriminant bits
    if let Some(discriminant_bits) = analysis.discriminant_bits {
        // Generate methods for external discriminant handling
        return Ok(generate_external_discriminant_helpers(
            analysis,
            enum_ident,
            discriminant_bits,
        ));
    }

    // Generate size lookup by discriminant
    let size_match_arms: Vec<_> = analysis
        .variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let discriminant = variant.discriminant.unwrap_or(index);
            let size = analysis.variant_sizes[index];
            quote! { #discriminant => ::core::option::Option::Some(#size) }
        })
        .collect();

    // Generate discriminant lookup by variant
    let discriminant_match_arms: Vec<_> = analysis
        .variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let variant_name = &variant.name;
            let discriminant = variant.discriminant.unwrap_or(index);
            match &variant.variant_type {
                VariantType::Unit => quote! { Self::#variant_name => #discriminant },
                VariantType::Data(_) => quote! { Self::#variant_name(_) => #discriminant },
            }
        })
        .collect();

    // Generate size lookup by variant
    let size_by_variant_arms: Vec<_> = analysis
        .variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let variant_name = &variant.name;
            let size = analysis.variant_sizes[index];
            match &variant.variant_type {
                VariantType::Unit => quote! { Self::#variant_name => #size },
                VariantType::Data(_) => quote! { Self::#variant_name(_) => #size },
            }
        })
        .collect();

    // Generate from_discriminant_and_bytes
    let from_discriminant_arms: Vec<_> = analysis.variants.iter().enumerate().map(|(index, variant)| {
        let variant_name = &variant.name;
        let discriminant = variant.discriminant.unwrap_or(index);

        match &variant.variant_type {
            VariantType::Unit => {
                quote! {
                    #discriminant => ::core::result::Result::Ok(Self::#variant_name)
                }
            }
            VariantType::Data(data_type) => {
                // Get the size for this variant
                let variant_size = analysis.variant_sizes[index];
                let data_bytes_cast = match variant_size {
                    0..=8 => quote! { bytes as u8 },
                    9..=16 => quote! { bytes as u16 },
                    17..=32 => quote! { bytes as u32 },
                    33..=64 => quote! { bytes as u64 },
                    65..=128 => quote! { bytes as u128 },
                    _ => quote! { bytes },
                };

                quote! {
                    #discriminant => {
                        match <#data_type as ::modular_bitfield::Specifier>::from_bytes(#data_bytes_cast) {
                            ::core::result::Result::Ok(data) => ::core::result::Result::Ok(Self::#variant_name(data)),
                            ::core::result::Result::Err(_) => ::core::result::Result::Err(::modular_bitfield::error::InvalidBitPattern::new(bytes)),
                        }
                    }
                }
            }
        }
    }).collect();

    // Generate the supported discriminants and sizes arrays
    let supported_discriminants: Vec<_> = analysis
        .variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let discriminant = variant.discriminant.unwrap_or(index);
            quote! { #discriminant }
        })
        .collect();

    let supported_sizes: Vec<_> = analysis
        .variant_sizes
        .iter()
        .map(|&size| {
            quote! { #size }
        })
        .collect();

    Ok(quote! {
        impl #enum_ident {
            /// Get the expected size in bits for a given discriminant value
            pub const fn size_for_discriminant(discriminant: usize) -> ::core::option::Option<usize> {
                match discriminant {
                    #( #size_match_arms, )*
                    _ => ::core::option::Option::None,
                }
            }

            /// Get the discriminant value for this variant
            pub const fn discriminant(&self) -> usize {
                match self {
                    #( #discriminant_match_arms, )*
                }
            }

            /// Get the actual size in bits of this variant's data
            pub const fn size(&self) -> usize {
                match self {
                    #( #size_by_variant_arms, )*
                }
            }

            /// Construct a variant from discriminant value and bytes
            ///
            /// # Arguments
            /// * `discriminant` - The discriminant value (from external source)
            /// * `bytes` - The data bytes for the variant
            ///
            /// # Returns
            /// The constructed enum variant, or an error if the discriminant is invalid
            /// or the bytes cannot be parsed for the target variant type.
            pub fn from_discriminant_and_bytes(
                discriminant: usize,
                bytes: <Self as ::modular_bitfield::Specifier>::Bytes
            ) -> ::core::result::Result<Self, ::modular_bitfield::error::InvalidBitPattern<<Self as ::modular_bitfield::Specifier>::Bytes>> {
                match discriminant {
                    #( #from_discriminant_arms, )*
                    _ => ::core::result::Result::Err(::modular_bitfield::error::InvalidBitPattern::new(bytes))
                }
            }

            /// Get all supported discriminant values for this enum
            pub const fn supported_discriminants() -> &'static [usize] {
                &[#( #supported_discriminants ),*]
            }

            /// Get all supported sizes for this enum
            pub const fn supported_sizes() -> &'static [usize] {
                &[#( #supported_sizes ),*]
            }
        }
    })
}

fn generate_external_discriminant_helpers(
    analysis: &VariableEnumAnalysis,
    enum_ident: &syn::Ident,
    discriminant_bits: usize,
) -> TokenStream2 {
    // For external discriminant, we don't generate the standard helper methods
    // since the discriminant is managed externally by the containing struct
    let _ = (analysis, enum_ident, discriminant_bits);
    quote! {
        // External discriminant - helper methods are managed by the containing struct
    }
}

/// Generate code for fixed-size enums with data variants
fn generate_fixed_size_enum_with_data(
    input: &syn::ItemEnum,
    attributes: Attributes,
    enum_ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
) -> syn::Result<TokenStream2> {
    let span = input.span();
    
    // Parse bits from attributes or error
    let bits = if let Some(BitsConfig::Fixed(bits)) = attributes.bits {
        bits
    } else if attributes.bits.is_none() {
        // No bits specified for enum with data variants
        return Err(format_err!(
            span,
            "enums with data variants require explicit #[bits = N] attribute"
        ));
    } else {
        // This shouldn't happen as we checked for Variable earlier
        return Err(format_err!(
            span,
            "internal error: unexpected bits configuration"
        ));
    };

    // Determine bytes type based on bits
    let bytes_type = match bits {
        0..=8 => quote! { u8 },
        9..=16 => quote! { u16 },
        17..=32 => quote! { u32 },
        33..=64 => quote! { u64 },
        65..=128 => quote! { u128 },
        _ => {
            return Err(format_err!(
                span,
                "enum requires more than 128 bits, which is not supported"
            ))
        }
    };

    // Analyze enum variants
    let mut variants = Vec::new();
    for (_index, variant) in input.variants.iter().enumerate() {
        let (discriminant, _) = parse_variant_attrs(variant)?;
        
        let variant_type = match &variant.fields {
            syn::Fields::Unit => VariantType::Unit,
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                VariantType::Data(Box::new(fields.unnamed.first().unwrap().ty.clone()))
            }
            syn::Fields::Named(_) => {
                return Err(format_err_spanned!(
                    variant,
                    "named fields in enum variants are not supported"
                ));
            }
            syn::Fields::Unnamed(_) => {
                return Err(format_err_spanned!(
                    variant,
                    "multiple fields in enum variants are not supported"
                ));
            }
        };

        variants.push(EnumVariant {
            name: variant.ident.clone(),
            variant_type,
            discriminant,
            explicit_bits: None,
            span: variant.span(),
        });
    }

    // Generate into_bytes match arms
    let into_bytes_arms: Vec<_> = variants.iter().enumerate().map(|(_index, variant)| {
        let variant_name = &variant.name;
        let variant_span = variant.span;

        match &variant.variant_type {
            VariantType::Unit => {
                // For external discrimination, unit variants always encode as 0
                quote_spanned!(variant_span=>
                    Self::#variant_name => ::core::result::Result::Ok(0 as #bytes_type)
                )
            },
            VariantType::Data(data_type) => {
                // For external discrimination, data variants encode their data directly
                quote_spanned!(variant_span=>
                    Self::#variant_name(data) => {
                        let data_bytes = <#data_type as ::modular_bitfield::Specifier>::into_bytes(data)?;
                        ::core::result::Result::Ok(data_bytes as #bytes_type)
                    }
                )
            }
        }
    }).collect();

    // For fixed-size enums with data variants, from_bytes defaults to first variant
    // This is because external discrimination is required
    let first_variant = variants.first().ok_or_else(|| {
        format_err!(span, "enum must have at least one variant")
    })?;
    
    let default_from_bytes = match &first_variant.variant_type {
        VariantType::Unit => {
            let variant_name = &first_variant.name;
            quote! {
                _ => ::core::result::Result::Ok(Self::#variant_name)
            }
        }
        VariantType::Data(data_type) => {
            let variant_name = &first_variant.name;
            quote! {
                bytes => {
                    match <#data_type as ::modular_bitfield::Specifier>::from_bytes(bytes) {
                        ::core::result::Result::Ok(data) => ::core::result::Result::Ok(Self::#variant_name(data)),
                        ::core::result::Result::Err(_) => ::core::result::Result::Err(::modular_bitfield::error::InvalidBitPattern::new(bytes)),
                    }
                }
            }
        }
    };

    // Generate discriminant helper methods
    let discriminant_arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let variant_name = &variant.name;
            let discriminant = variant.discriminant.unwrap_or(index);
            match &variant.variant_type {
                VariantType::Unit => quote! { Self::#variant_name => #discriminant },
                VariantType::Data(_) => quote! { Self::#variant_name(_) => #discriminant },
            }
        })
        .collect();

    // Generate from_discriminant_and_bytes match arms
    let from_discriminant_arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let variant_name = &variant.name;
            let discriminant = variant.discriminant.unwrap_or(index);
            
            match &variant.variant_type {
                VariantType::Unit => {
                    quote! {
                        #discriminant => ::core::result::Result::Ok(Self::#variant_name)
                    }
                }
                VariantType::Data(data_type) => {
                    quote! {
                        #discriminant => {
                            match <#data_type as ::modular_bitfield::Specifier>::from_bytes(bytes) {
                                ::core::result::Result::Ok(data) => ::core::result::Result::Ok(Self::#variant_name(data)),
                                ::core::result::Result::Err(_) => ::core::result::Result::Err(::modular_bitfield::error::InvalidBitPattern::new(bytes)),
                            }
                        }
                    }
                }
            }
        })
        .collect();

    Ok(quote_spanned!(span=>
        impl #impl_generics ::modular_bitfield::Specifier for #enum_ident #ty_generics #where_clause {
            const BITS: usize = #bits;
            type Bytes = #bytes_type;
            type InOut = Self;

            #[inline]
            fn into_bytes(input: <Self as ::modular_bitfield::Specifier>::InOut) -> ::core::result::Result<<Self as ::modular_bitfield::Specifier>::Bytes, ::modular_bitfield::error::OutOfBounds> {
                match input {
                    #( #into_bytes_arms, )*
                }
            }

            #[inline]
            fn from_bytes(bytes: <Self as ::modular_bitfield::Specifier>::Bytes) -> ::core::result::Result<<Self as ::modular_bitfield::Specifier>::InOut, ::modular_bitfield::error::InvalidBitPattern<<Self as ::modular_bitfield::Specifier>::Bytes>> {
                // For enums with data variants, external discrimination is required
                // This defaults to the first variant
                match bytes {
                    #default_from_bytes
                }
            }
        }

        impl #enum_ident {
            /// Get the discriminant value for this variant
            pub const fn discriminant(&self) -> usize {
                match self {
                    #( #discriminant_arms, )*
                }
            }

            /// Construct a variant from discriminant value and bytes
            pub fn from_discriminant_and_bytes(
                discriminant: usize,
                bytes: <Self as ::modular_bitfield::Specifier>::Bytes
            ) -> ::core::result::Result<Self, ::modular_bitfield::error::InvalidBitPattern<<Self as ::modular_bitfield::Specifier>::Bytes>> {
                match discriminant {
                    #( #from_discriminant_arms, )*
                    _ => ::core::result::Result::Err(::modular_bitfield::error::InvalidBitPattern::new(bytes))
                }
            }
        }
    ))
}