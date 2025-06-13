use crate::bitfield::VariableBitsError;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use std::collections::HashSet;
use syn::spanned::Spanned as _;

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_or_error(input) {
        Ok(output) => {
            // Wrap the output in an allow attribute to suppress the false positive
            // clippy warning about semicolon_if_nothing_returned on enum variants
            quote! {
                #[allow(clippy::semicolon_if_nothing_returned)]
                const _: () = {
                    #output
                };
            }
        },
        Err(err) => err.to_compile_error(),
    }
}

fn generate_or_error(input: TokenStream2) -> syn::Result<TokenStream2> {
    let input = syn::parse2::<syn::DeriveInput>(input)?;
    match input.data {
        syn::Data::Enum(data_enum) => generate_enum(&syn::ItemEnum {
            attrs: input.attrs,
            vis: input.vis,
            enum_token: data_enum.enum_token,
            ident: input.ident,
            generics: input.generics,
            brace_token: data_enum.brace_token,
            variants: data_enum.variants,
        }),
        syn::Data::Struct(_) => Err(format_err!(
            input,
            "structs are not supported as bitfield specifiers",
        )),
        syn::Data::Union(_) => Err(format_err!(
            input,
            "unions are not supported as bitfield specifiers",
        )),
    }
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

struct EnumAnalysis {
    variants: Vec<EnumVariant>,
    total_bits: usize,
    has_data_variants: bool, // True if any variant has data
}

#[derive(Debug, Clone)]
enum BitsConfig {
    Fixed(usize),         // #[bits = 32]
    Variable(Vec<usize>), // #[bits = (8, 16, 32)]
}

struct Attributes {
    bits: Option<BitsConfig>,
    discriminant_bits: Option<usize>, // For #[discriminant_bits = N]
}

struct VariableEnumAnalysis {
    variants: Vec<EnumVariant>,
    variant_sizes: Vec<usize>, // Sizes for each variant (parallel to variants)
    max_size: usize,           // Maximum variant size
    _has_discriminants: bool,  // True if any variant has explicit discriminant
    discriminant_bits: Option<usize>, // External discriminant bits (from parent struct)
}

fn parse_attrs(attrs: &[syn::Attribute]) -> syn::Result<Attributes> {
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
    _analysis: &VariableEnumAnalysis,
    enum_ident: &syn::Ident,
    discriminant_bits: usize,
) -> TokenStream2 {
    let span = enum_ident.span();

    // Generate methods specific to external discriminant handling
    quote_spanned!(span=>
        impl #enum_ident {
            /// Get the number of discriminant bits expected from parent
            pub const fn discriminant_bits() -> usize {
                #discriminant_bits
            }

            /// Check if this enum uses external discriminant
            pub const fn uses_external_discriminant() -> bool {
                true
            }
        }
    )
}

#[allow(clippy::too_many_lines)]
fn analyze_enum(input: &syn::ItemEnum, attributes: &Attributes) -> syn::Result<EnumAnalysis> {
    let span = input.span();

    // Check if any variants have data
    let has_data_variants = input
        .variants
        .iter()
        .any(|v| !matches!(v.fields, syn::Fields::Unit));

    if has_data_variants {
        // Data variants require explicit bits specification
        if let Some(BitsConfig::Variable(_)) = &attributes.bits {
            // Variable bits enum - use different analysis path
            return Err(format_err!(
                span,
                "variable bits enum detected - should use analyze_variable_enum"
            ));
        }

        let total_bits = match &attributes.bits {
            Some(BitsConfig::Fixed(size)) => *size,
            Some(BitsConfig::Variable(_)) => {
                // This should have been caught above
                unreachable!("variable bits should be handled separately")
            }
            None => {
                return Err(format_err!(
                    span,
                    "enums with data variants must specify #[bits = N] or #[bits = (sizes...)]"
                ));
            }
        };

        // Classify each variant
        let variants = input
            .variants
            .iter()
            .map(|variant| {
                let variant_type = match &variant.fields {
                    syn::Fields::Unit => VariantType::Unit,
                    syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                        VariantType::Data(Box::new(fields.unnamed.first().unwrap().ty.clone()))
                    }
                    syn::Fields::Named(_) => {
                        return Err(format_err_spanned!(
                            variant,
                            "named fields in enum variants are not supported for data variants"
                        ));
                    }
                    syn::Fields::Unnamed(_) => {
                        return Err(format_err_spanned!(
                            variant,
                            "multiple fields in enum variants are not supported"
                        ));
                    }
                };

                let (discriminant, explicit_bits) = parse_variant_attrs(variant)?;

                Ok(EnumVariant {
                    name: variant.ident.clone(),
                    variant_type,
                    discriminant,
                    explicit_bits,
                    span: variant.span(),
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(EnumAnalysis {
            variants,
            total_bits,
            has_data_variants: true,
        })
    } else {
        // Unit-only enums (existing logic)
        let variant_count = input.variants.len();
        let total_bits = if let Some(bits_config) = &attributes.bits {
            match bits_config {
                BitsConfig::Fixed(size) => *size,
                BitsConfig::Variable(_) => {
                    return Err(format_err!(
                        span,
                        "unit-only enums cannot use variable bits"
                    ));
                }
            }
        } else {
            if !variant_count.is_power_of_two() {
                return Err(format_err!(
                    span,
                    "#[derive(Specifier)] expected a number of variants which is a power of 2, specify #[bits = {}] if that was your intent",
                    variant_count.next_power_of_two().trailing_zeros(),
                ));
            }
            // We can take `trailing_zeros` returns type as the required amount of bits.
            if let Some(power_of_two) = variant_count.checked_next_power_of_two() {
                power_of_two.trailing_zeros() as usize
            } else {
                return Err(format_err!(
                    span,
                    "#[derive(Specifier)] has too many variants to pack into a bitfield",
                ));
            }
        };

        let variants = input
            .variants
            .iter()
            .map(|variant| {
                let (discriminant, explicit_bits) = parse_variant_attrs(variant)?;
                Ok(EnumVariant {
                    name: variant.ident.clone(),
                    variant_type: VariantType::Unit,
                    discriminant,
                    explicit_bits,
                    span: variant.span(),
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(EnumAnalysis {
            variants,
            total_bits,
            has_data_variants: false,
        })
    }
}

fn generate_enum(input: &syn::ItemEnum) -> syn::Result<TokenStream2> {
    let attributes = parse_attrs(&input.attrs)?;
    let enum_ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Check if this is a variable-size enum
    if let Some(BitsConfig::Variable(_)) = &attributes.bits {
        // Use variable enum analysis and generation
        let variable_analysis = analyze_variable_enum(input, &attributes)?;
        generate_variable_enum_specifier_impl(
            &variable_analysis,
            enum_ident,
            &impl_generics,
            &ty_generics,
            where_clause,
        )
    } else {
        // Use the analyze_enum function to handle both unit and data variants
        let analysis = analyze_enum(input, &attributes)?;

        if analysis.has_data_variants {
            // Generate code for enums with data variants - external discrimination
            generate_enum_with_data_variants(
                &analysis,
                enum_ident,
                &impl_generics,
                &ty_generics,
                where_clause,
            )
        } else {
            // Generate code for unit-only enums (existing logic)
            Ok(generate_unit_enum(
                input,
                &analysis,
                enum_ident,
                &impl_generics,
                &ty_generics,
                where_clause,
            ))
        }
    }
}

fn generate_unit_enum(
    input: &syn::ItemEnum,
    analysis: &EnumAnalysis,
    enum_ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
) -> TokenStream2 {
    let span = input.span();
    let bits = analysis.total_bits;

    let variants = analysis
        .variants
        .iter()
        .map(|variant| &variant.name)
        .collect::<Vec<_>>();

    let check_discriminants = variants.iter().map(|ident| {
        let span = ident.span();
        quote_spanned!(span =>
            impl #impl_generics ::modular_bitfield::private::checks::CheckDiscriminantInRange<[(); Self::#ident as usize]> for #enum_ident #ty_generics #where_clause {
                type CheckType = [(); ((Self::#ident as usize) < (0x01_usize << #bits)) as usize ];
            }
        )
    });
    let from_bytes_arms = variants.iter().map(|ident| {
        let span = ident.span();
        quote_spanned!(span=>
            __bitfield_binding if __bitfield_binding == Self::#ident as <Self as ::modular_bitfield::Specifier>::Bytes => {
                ::core::result::Result::Ok(Self::#ident)
            }
        )
    });

    quote_spanned!(span=>
        #( #check_discriminants )*

        impl #impl_generics ::modular_bitfield::Specifier for #enum_ident #ty_generics #where_clause {
            const BITS: usize = #bits;
            type Bytes = <[(); #bits] as ::modular_bitfield::private::SpecifierBytes>::Bytes;
            type InOut = Self;

            #[inline]
            fn into_bytes(input: <Self as ::modular_bitfield::Specifier>::InOut) -> ::core::result::Result<<Self as ::modular_bitfield::Specifier>::Bytes, ::modular_bitfield::error::OutOfBounds> {
                ::core::result::Result::Ok(input as <Self as ::modular_bitfield::Specifier>::Bytes)
            }

            #[inline]
            fn from_bytes(bytes: <Self as ::modular_bitfield::Specifier>::Bytes) -> ::core::result::Result<<Self as ::modular_bitfield::Specifier>::InOut, ::modular_bitfield::error::InvalidBitPattern<<Self as ::modular_bitfield::Specifier>::Bytes>> {
                match bytes {
                    #( #from_bytes_arms ),*
                    invalid_bytes => {
                        ::core::result::Result::Err(
                            <::modular_bitfield::error::InvalidBitPattern<<Self as ::modular_bitfield::Specifier>::Bytes>>::new(invalid_bytes)
                        )
                    }
                }
            }
        }
    )
}

#[allow(clippy::too_many_lines)]
fn generate_enum_with_data_variants(
    analysis: &EnumAnalysis,
    enum_ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
) -> syn::Result<TokenStream2> {
    let span = enum_ident.span();
    let total_bits = analysis.total_bits;

    // Determine the Bytes type based on total bits
    let bytes_type = match total_bits {
        1..=8 => quote! { u8 },
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

    // For external discrimination, we just convert directly between each variant
    // and the underlying bytes - no internal discriminant needed
    let into_bytes_arms = analysis.variants.iter().map(|variant| {
        let variant_name = &variant.name;
        let variant_span = variant_name.span();

        match &variant.variant_type {
            VariantType::Unit => {
                // Unit variant: all bits zero
                quote_spanned!(variant_span=>
                    Self::#variant_name => {
                        ::core::result::Result::Ok(0 as #bytes_type)
                    }
                )
            },
            VariantType::Data(data_type) => {
                let data_type = &**data_type;
                quote_spanned!(variant_span=>
                    Self::#variant_name(data) => {
                        // Convert data directly to bytes using its Specifier impl
                        let data_bytes = <#data_type as ::modular_bitfield::Specifier>::into_bytes(data)?;
                        ::core::result::Result::Ok(data_bytes as #bytes_type)
                    }
                )
            }
        }
    });

    // For external discrimination, from_bytes always constructs the first variant
    // User is responsible for using the correct constructor based on external information

    // Generate const assertions to validate all data types have the same BITS as the enum
    let data_types: Vec<_> = analysis
        .variants
        .iter()
        .filter_map(|variant| match &variant.variant_type {
            VariantType::Data(data_type) => Some(&**data_type),
            VariantType::Unit => None,
        })
        .collect();

    let const_assertions = if data_types.is_empty() {
        vec![]
    } else {
        let mut assertions = vec![];

        // All data types must have the same BITS as the enum
        for data_type in &data_types {
            assertions.push(quote! {
                const _: () = {
                    // Debug: let's see what the actual values are
                    const DATA_TYPE_BITS: usize = <#data_type as ::modular_bitfield::Specifier>::BITS;
                    const TOTAL_BITS: usize = #total_bits;

                    assert!(
                        DATA_TYPE_BITS == TOTAL_BITS,
                        "All data variant types must have the same BITS as the enum total"
                    );
                };
            });
        }
        assertions
    };

    // Note: from_bytes is not a typical match - it will only use the first arm that matches.
    // For external discrimination, the user is responsible for constructing the correct variant
    // based on external information. The from_bytes here is mainly for completeness.
    let first_arm = if let Some(first_variant) = analysis.variants.first() {
        let variant_name = &first_variant.name;
        let variant_span = variant_name.span();

        match &first_variant.variant_type {
            VariantType::Unit => {
                quote_spanned!(variant_span=>
                    _ => ::core::result::Result::Ok(Self::#variant_name)
                )
            }
            VariantType::Data(data_type) => {
                let data_type = &**data_type;
                quote_spanned!(variant_span=>
                    bytes => {
                        let data = <#data_type as ::modular_bitfield::Specifier>::from_bytes(bytes as <#data_type as ::modular_bitfield::Specifier>::Bytes)?;
                        ::core::result::Result::Ok(Self::#variant_name(data))
                    }
                )
            }
        }
    } else {
        return Err(format_err!(span, "enum must have at least one variant"));
    };

    Ok(quote_spanned!(span=>
        #( #const_assertions )*

        impl #impl_generics ::modular_bitfield::Specifier for #enum_ident #ty_generics #where_clause {
            const BITS: usize = #total_bits;
            type Bytes = #bytes_type;
            type InOut = Self;

            #[inline]
            fn into_bytes(input: <Self as ::modular_bitfield::Specifier>::InOut) -> ::core::result::Result<<Self as ::modular_bitfield::Specifier>::Bytes, ::modular_bitfield::error::OutOfBounds> {
                match input {
                    #( #into_bytes_arms ),*
                }
            }

            #[inline]
            fn from_bytes(bytes: <Self as ::modular_bitfield::Specifier>::Bytes) -> ::core::result::Result<<Self as ::modular_bitfield::Specifier>::InOut, ::modular_bitfield::error::InvalidBitPattern<<Self as ::modular_bitfield::Specifier>::Bytes>> {
                // For external discrimination, we default to the first variant
                // User is responsible for using the correct constructor
                match bytes {
                    #first_arm
                }
            }
        }
    ))
}
