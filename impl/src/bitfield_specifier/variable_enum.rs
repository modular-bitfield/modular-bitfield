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
    discriminant_bits: Option<usize>, // External discriminant bits (from parent struct)
}

pub fn parse_attrs(attrs: &[syn::Attribute]) -> syn::Result<Attributes> {
    let mut attributes = Attributes {
        bits: None,
        discriminant_bits: None,
    };

    for attr in attrs {
        if attr.path().is_ident("bits") {
            parse_bits_attribute(attr, &mut attributes)?;
        } else if attr.path().is_ident("discriminant_bits") {
            parse_discriminant_bits_attribute(attr, &mut attributes)?;
        }
    }
    Ok(attributes)
}

/// Parse the `#[bits = ...]` attribute
fn parse_bits_attribute(attr: &syn::Attribute, attributes: &mut Attributes) -> syn::Result<()> {
    if attributes.bits.is_some() {
        return Err(format_err_spanned!(
            attr,
            "More than one 'bits' attribute is not permitted",
        ));
    }

    match &attr.meta {
        syn::Meta::NameValue(meta) => parse_bits_name_value(meta, attributes),
        syn::Meta::List(meta_list) => parse_bits_list(meta_list, attributes),
        syn::Meta::Path(_) => Err(format_err_spanned!(
            attr,
            "bits attribute requires a value: #[bits = N], #[bits = (N, M, ...)], or #[bits(N, M, ...)]"
        )),
    }
}

/// Parse `#[bits = N]` or `#[bits = (N, M, ...)]`
fn parse_bits_name_value(meta: &syn::MetaNameValue, attributes: &mut Attributes) -> syn::Result<()> {
    match &meta.value {
        // #[bits = 32]
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit),
            ..
        }) => {
            let size = parse_and_validate_bit_size(lit)?;
            attributes.bits = Some(BitsConfig::Fixed(size));
            Ok(())
        }
        // #[bits = (8, 16, 32)]
        syn::Expr::Tuple(tuple) => {
            let sizes = parse_bit_sizes_from_tuple(tuple)?;
            attributes.bits = Some(BitsConfig::Variable(sizes));
            Ok(())
        }
        _ => Err(format_err_spanned!(
            meta,
            "bits must be integer literal for fixed size: #[bits = N]"
        )),
    }
}

/// Parse `#[bits(8, 16, 32)]`
fn parse_bits_list(meta_list: &syn::MetaList, attributes: &mut Attributes) -> syn::Result<()> {
    // Custom parser for the comma-separated list of integers
    struct IntList(Vec<syn::LitInt>);

    impl syn::parse::Parse for IntList {
        fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
            let parsed = syn::punctuated::Punctuated::<syn::LitInt, syn::Token![,]>::parse_terminated(input)?;
            Ok(IntList(parsed.into_iter().collect()))
        }
    }

    let content = &meta_list.tokens;
    let parsed_content: IntList = syn::parse2(content.clone())?;
    let mut sizes = Vec::new();
    
    for lit in parsed_content.0 {
        let size = parse_and_validate_bit_size(&lit)?;
        sizes.push(size);
    }

    if sizes.is_empty() {
        return Err(format_err_spanned!(meta_list, "bits list cannot be empty"));
    }
    
    attributes.bits = Some(BitsConfig::Variable(sizes));
    Ok(())
}

/// Parse bit sizes from a tuple expression
fn parse_bit_sizes_from_tuple(tuple: &syn::ExprTuple) -> syn::Result<Vec<usize>> {
    let mut sizes = Vec::new();
    
    for elem in &tuple.elems {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit),
            ..
        }) = elem
        {
            let size = parse_and_validate_bit_size(lit)?;
            sizes.push(size);
        } else {
            return Err(format_err_spanned!(
                elem,
                "expected integer literal in bits tuple"
            ));
        }
    }
    
    if sizes.is_empty() {
        return Err(format_err_spanned!(
            tuple,
            "bits tuple cannot be empty"
        ));
    }
    
    Ok(sizes)
}

/// Parse and validate a single bit size value
fn parse_and_validate_bit_size(lit: &syn::LitInt) -> syn::Result<usize> {
    let size = lit.base10_parse::<usize>()?;
    
    if size == 0 {
        return Err(format_err_spanned!(
            lit,
            "bits sizes must be greater than 0"
        ));
    }
    if size > 128 {
        return Err(format_err_spanned!(
            lit,
            "bits sizes cannot exceed 128 bits"
        ));
    }
    
    Ok(size)
}

/// Parse the `#[discriminant_bits = N]` attribute
fn parse_discriminant_bits_attribute(attr: &syn::Attribute, attributes: &mut Attributes) -> syn::Result<()> {
    if attributes.discriminant_bits.is_some() {
        return Err(format_err_spanned!(
            attr,
            "More than one 'discriminant_bits' attribute is not permitted",
        ));
    }
    
    let meta = attr.meta.require_name_value()?;
    
    if let syn::Expr::Lit(syn::ExprLit {
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
        attributes.discriminant_bits = Some(bits);
        Ok(())
    } else {
        Err(format_err_spanned!(
            attr,
            "discriminant_bits must be in form #[discriminant_bits = N]",
        ))
    }
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
    }

    Ok(())
}

/// Validate that all discriminants fit within the specified number of bits
fn validate_discriminants_fit_in_bits(variants: &[EnumVariant], discriminant_bits: usize) -> syn::Result<()> {
    let max_value = (1usize << discriminant_bits) - 1;
    
    for (index, variant) in variants.iter().enumerate() {
        let discriminant = variant.discriminant.unwrap_or(index);
        
        if discriminant > max_value {
            return Err(format_err!(
                variant.span,
                "discriminant value {} for variant {} exceeds maximum value {} for {} discriminant bits",
                discriminant,
                variant.name,
                max_value,
                discriminant_bits
            ));
        }
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
    
    // If using external discriminant bits, validate all discriminants fit
    if let Some(discriminant_bits) = attributes.discriminant_bits {
        validate_discriminants_fit_in_bits(&variants, discriminant_bits)?;
    }

    Ok(VariableEnumAnalysis {
        variants,
        variant_sizes,
        max_size,
        discriminant_bits: attributes.discriminant_bits,
    })
}

pub fn generate_variable_enum(
    input: &syn::ItemEnum,
    attributes: &Attributes,
) -> syn::Result<TokenStream2> {
    let enum_ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Check if this enum has variable bits configuration
    if let Some(BitsConfig::Variable(_)) = &attributes.bits {
        // Analyze as variable enum with explicit sizes
        let analysis = analyze_variable_enum(input, attributes)?;
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
    generate_fixed_size_enum_with_data(
        input,
        attributes,
        enum_ident,
        &impl_generics,
        &ty_generics,
        where_clause,
    )
}

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
    let bytes_type = determine_bytes_type(max_size, span)?;

    // Generate compile-time assertions for data type sizes
    let size_assertions = generate_size_assertions(analysis);

    // Generate into_bytes match arms with bit validation
    let into_bytes_arms = generate_into_bytes_arms(analysis, &bytes_type);

    // Generate from_bytes - defaults to first variant for external discrimination
    let first_variant_construction = generate_first_variant_construction(analysis, span)?;

    // Generate discriminant helper methods
    let discriminant_helpers = generate_enum_discriminant_helpers(analysis, enum_ident);

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

/// Determine the appropriate Bytes type based on the maximum size needed
fn determine_bytes_type(max_size: usize, span: proc_macro2::Span) -> syn::Result<TokenStream2> {
    match max_size {
        0..=8 => Ok(quote! { u8 }), // Handle all-unit enums and 1-8 bits
        9..=16 => Ok(quote! { u16 }),
        17..=32 => Ok(quote! { u32 }),
        33..=64 => Ok(quote! { u64 }),
        65..=128 => Ok(quote! { u128 }),
        _ => Err(format_err!(
            span,
            "enum requires more than 128 bits, which is not supported"
        ))
    }
}

/// Generate compile-time assertions for data type sizes
fn generate_size_assertions(analysis: &VariableEnumAnalysis) -> impl Iterator<Item = TokenStream2> + '_ {
    analysis.variants.iter().enumerate()
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
        })
}

/// Generate `into_bytes` match arms for all variants
fn generate_into_bytes_arms(analysis: &VariableEnumAnalysis, bytes_type: &TokenStream2) -> Vec<TokenStream2> {
    analysis.variants.iter().enumerate().map(|(index, variant)| {
        let variant_name = &variant.name;
        let variant_span = variant.span;
        let variant_size = analysis.variant_sizes[index];

        match &variant.variant_type {
            VariantType::Unit => generate_unit_variant_into_bytes(variant_name, variant_span, bytes_type),
            VariantType::Data(data_type) => generate_data_variant_into_bytes(
                variant_name, 
                variant_span, 
                data_type, 
                variant_size, 
                bytes_type
            ),
        }
    }).collect()
}

/// Generate `into_bytes` for unit variants
fn generate_unit_variant_into_bytes(
    variant_name: &syn::Ident,
    variant_span: proc_macro2::Span,
    bytes_type: &TokenStream2,
) -> TokenStream2 {
    quote_spanned!(variant_span=>
        Self::#variant_name => ::core::result::Result::Ok(0 as #bytes_type)
    )
}

/// Generate `into_bytes` for data variants with validation
fn generate_data_variant_into_bytes(
    variant_name: &syn::Ident,
    variant_span: proc_macro2::Span,
    data_type: &syn::Type,
    variant_size: usize,
    bytes_type: &TokenStream2,
) -> TokenStream2 {
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

/// Generate `from_bytes` construction for the first variant (default)
fn generate_first_variant_construction(
    analysis: &VariableEnumAnalysis,
    span: proc_macro2::Span,
) -> syn::Result<TokenStream2> {
    let first_variant = analysis.variants.first()
        .ok_or_else(|| format_err!(span, "enum must have at least one variant"))?;
    
    let variant_name = &first_variant.name;
    let variant_span = first_variant.span;

    match &first_variant.variant_type {
        VariantType::Unit => Ok(quote_spanned!(variant_span=>
            _ => ::core::result::Result::Ok(Self::#variant_name)
        )),
        VariantType::Data(data_type) => {
            // Get the size for the first variant
            let first_variant_size = analysis.variant_sizes[0];
            let data_bytes_cast = get_bytes_cast_expression(first_variant_size);

            Ok(quote_spanned!(variant_span=>
                bytes => {
                    match <#data_type as ::modular_bitfield::Specifier>::from_bytes(#data_bytes_cast) {
                        ::core::result::Result::Ok(data) => ::core::result::Result::Ok(Self::#variant_name(data)),
                        ::core::result::Result::Err(_) => ::core::result::Result::Err(::modular_bitfield::error::InvalidBitPattern::new(bytes)),
                    }
                }
            ))
        }
    }
}

/// Get the appropriate bytes cast expression based on size
fn get_bytes_cast_expression(size: usize) -> TokenStream2 {
    match size {
        0..=8 => quote! { bytes as u8 },
        9..=16 => quote! { bytes as u16 },
        17..=32 => quote! { bytes as u32 },
        33..=64 => quote! { bytes as u64 },
        65..=128 => quote! { bytes as u128 },
        _ => quote! { bytes },
    }
}

fn generate_enum_discriminant_helpers(
    analysis: &VariableEnumAnalysis,
    enum_ident: &syn::Ident,
) -> TokenStream2 {
    // Check if this enum uses external discriminant bits
    if let Some(discriminant_bits) = analysis.discriminant_bits {
        // Generate methods for external discriminant handling
        return generate_external_discriminant_helpers(
            analysis,
            enum_ident,
            discriminant_bits,
        );
    }

    // Generate all the match arms we need
    let size_match_arms = generate_size_lookup_arms(analysis);
    let discriminant_match_arms = generate_discriminant_lookup_arms(analysis);
    let size_by_variant_arms = generate_size_by_variant_arms(analysis);
    let from_discriminant_arms = generate_from_discriminant_arms(analysis);
    let supported_discriminants = generate_supported_discriminants(analysis);
    let supported_sizes = generate_supported_sizes(analysis);

    quote! {
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
    }
}

/// Generate size lookup match arms by discriminant
fn generate_size_lookup_arms(analysis: &VariableEnumAnalysis) -> Vec<TokenStream2> {
    analysis
        .variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let discriminant = variant.discriminant.unwrap_or(index);
            let size = analysis.variant_sizes[index];
            quote! { #discriminant => ::core::option::Option::Some(#size) }
        })
        .collect()
}

/// Generate discriminant lookup match arms by variant
fn generate_discriminant_lookup_arms(analysis: &VariableEnumAnalysis) -> Vec<TokenStream2> {
    analysis
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
        .collect()
}

/// Generate size lookup match arms by variant
fn generate_size_by_variant_arms(analysis: &VariableEnumAnalysis) -> Vec<TokenStream2> {
    analysis
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
        .collect()
}

/// Generate `from_discriminant_and_bytes` match arms
fn generate_from_discriminant_arms(analysis: &VariableEnumAnalysis) -> Vec<TokenStream2> {
    analysis.variants.iter().enumerate().map(|(index, variant)| {
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
                let data_bytes_cast = get_bytes_cast_expression(variant_size);

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
    }).collect()
}

/// Generate supported discriminants list
fn generate_supported_discriminants(analysis: &VariableEnumAnalysis) -> Vec<TokenStream2> {
    analysis
        .variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let discriminant = variant.discriminant.unwrap_or(index);
            quote! { #discriminant }
        })
        .collect()
}

/// Generate supported sizes list
fn generate_supported_sizes(analysis: &VariableEnumAnalysis) -> Vec<TokenStream2> {
    analysis
        .variant_sizes
        .iter()
        .map(|&size| {
            quote! { #size }
        })
        .collect()
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
    attributes: &Attributes,
    enum_ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
) -> syn::Result<TokenStream2> {
    let span = input.span();

    // Extract and validate bits configuration
    let bits = extract_fixed_bits(attributes, span)?;
    let bytes_type = determine_bytes_type(bits, span)?;

    // Analyze enum variants
    let variants = analyze_fixed_enum_variants(input)?;

    // Generate into_bytes match arms
    let into_bytes_arms = generate_fixed_enum_into_bytes_arms(&variants, &bytes_type);

    // Generate default from_bytes for first variant
    let default_from_bytes = generate_fixed_enum_default_from_bytes(&variants, span)?;

    // Generate discriminant helper methods
    let discriminant_arms = generate_fixed_enum_discriminant_arms(&variants);
    let from_discriminant_arms = generate_fixed_enum_from_discriminant_arms(&variants);

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

/// Extract fixed bits configuration from attributes
fn extract_fixed_bits(attributes: &Attributes, span: proc_macro2::Span) -> syn::Result<usize> {
    match &attributes.bits {
        Some(BitsConfig::Fixed(bits)) => Ok(*bits),
        None => Err(format_err!(
            span,
            "enums with data variants require explicit #[bits = N] attribute"
        )),
        _ => Err(format_err!(
            span,
            "internal error: unexpected bits configuration"
        )),
    }
}

/// Analyze variants for fixed-size enum
fn analyze_fixed_enum_variants(input: &syn::ItemEnum) -> syn::Result<Vec<EnumVariant>> {
    let mut variants = Vec::new();
    
    for variant in &input.variants {
        let (discriminant, _) = parse_variant_attrs(variant)?;
        let variant_type = analyze_variant_type(variant)?;

        variants.push(EnumVariant {
            name: variant.ident.clone(),
            variant_type,
            discriminant,
            explicit_bits: None,
            span: variant.span(),
        });
    }
    
    Ok(variants)
}

/// Analyze variant type (unit or data)
fn analyze_variant_type(variant: &syn::Variant) -> syn::Result<VariantType> {
    match &variant.fields {
        syn::Fields::Unit => Ok(VariantType::Unit),
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            Ok(VariantType::Data(Box::new(fields.unnamed.first().unwrap().ty.clone())))
        }
        syn::Fields::Named(_) => {
            Err(format_err_spanned!(
                variant,
                "named fields in enum variants are not supported"
            ))
        }
        syn::Fields::Unnamed(_) => {
            Err(format_err_spanned!(
                variant,
                "multiple fields in enum variants are not supported"
            ))
        }
    }
}

/// Generate `into_bytes` arms for fixed enum
fn generate_fixed_enum_into_bytes_arms(variants: &[EnumVariant], bytes_type: &TokenStream2) -> Vec<TokenStream2> {
    variants.iter().map(|variant| {
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
    }).collect()
}

/// Generate default `from_bytes` for fixed enum
fn generate_fixed_enum_default_from_bytes(variants: &[EnumVariant], span: proc_macro2::Span) -> syn::Result<TokenStream2> {
    let first_variant = variants
        .first()
        .ok_or_else(|| format_err!(span, "enum must have at least one variant"))?;

    match &first_variant.variant_type {
        VariantType::Unit => {
            let variant_name = &first_variant.name;
            Ok(quote! {
                _ => ::core::result::Result::Ok(Self::#variant_name)
            })
        }
        VariantType::Data(data_type) => {
            let variant_name = &first_variant.name;
            Ok(quote! {
                bytes => {
                    match <#data_type as ::modular_bitfield::Specifier>::from_bytes(bytes) {
                        ::core::result::Result::Ok(data) => ::core::result::Result::Ok(Self::#variant_name(data)),
                        ::core::result::Result::Err(_) => ::core::result::Result::Err(::modular_bitfield::error::InvalidBitPattern::new(bytes)),
                    }
                }
            })
        }
    }
}

/// Generate discriminant arms for fixed enum
fn generate_fixed_enum_discriminant_arms(variants: &[EnumVariant]) -> Vec<TokenStream2> {
    variants
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
        .collect()
}

/// Generate `from_discriminant_and_bytes` arms for fixed enum
fn generate_fixed_enum_from_discriminant_arms(variants: &[EnumVariant]) -> Vec<TokenStream2> {
    variants
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
        .collect()
}

