use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned as _;
use crate::utils::calculate_discriminant_bits;

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_or_error(input) {
        Ok(output) => output,
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
    Unit,                           // No data
    Data(syn::Type),               // Has data of specified type
}

struct EnumVariant {
    name: syn::Ident,
    variant_type: VariantType,
    discriminant: usize,           // Auto-assigned index
}

struct EnumAnalysis {
    variants: Vec<EnumVariant>,
    total_bits: usize,
    discriminant_bits: usize,      // Auto-calculated or manual
    data_bits: usize,              // total_bits - discriminant_bits
    has_data_variants: bool,       // True if any variant has data
}

struct Attributes {
    bits: Option<usize>,
    discriminant_bits: Option<usize>,
}

fn parse_attrs(attrs: &[syn::Attribute]) -> syn::Result<Attributes> {
    let mut attributes = Attributes { 
        bits: None, 
        discriminant_bits: None 
    };
    
    for attr in attrs {
        if attr.path().is_ident("bits") {
            if attributes.bits.is_some() {
                return Err(format_err_spanned!(
                    attr,
                    "More than one 'bits' attribute is not permitted",
                ));
            }
            let meta = attr.meta.require_name_value()?;
            attributes.bits = if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit),
                ..
            }) = &meta.value
            {
                Some(lit.base10_parse::<usize>()?)
            } else {
                return Err(format_err_spanned!(
                    attr,
                    "could not parse 'bits' attribute",
                ));
            };
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
                Some(lit.base10_parse::<usize>()?)
            } else {
                return Err(format_err_spanned!(
                    attr,
                    "could not parse 'discriminant_bits' attribute",
                ));
            };
        }
    }
    Ok(attributes)
}

fn analyze_enum(input: &syn::ItemEnum, attributes: &Attributes) -> syn::Result<EnumAnalysis> {
    let span = input.span();
    let variant_count = input.variants.len();
    
    // Check if any variants have data
    let has_data_variants = input.variants.iter()
        .any(|v| !matches!(v.fields, syn::Fields::Unit));
    
    if has_data_variants {
        // New logic for data variants - require explicit bits
        let total_bits = attributes.bits.ok_or_else(|| {
            format_err!(span, "enums with data variants must specify #[bits = N]")
        })?;
        
        let discriminant_bits = attributes.discriminant_bits
            .unwrap_or_else(|| calculate_discriminant_bits(variant_count));
        
        let data_bits = total_bits.checked_sub(discriminant_bits)
            .ok_or_else(|| {
                format_err!(span, "not enough bits for discriminant: need {} bits for {} variants, but only {} total bits specified", 
                    discriminant_bits, variant_count, total_bits)
            })?;
        
        // Classify each variant and assign discriminants
        let variants = input.variants.iter().enumerate()
            .map(|(i, variant)| {
                let variant_type = match &variant.fields {
                    syn::Fields::Unit => VariantType::Unit,
                    syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                        VariantType::Data(fields.unnamed.first().unwrap().ty.clone())
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
                
                Ok(EnumVariant {
                    name: variant.ident.clone(),
                    variant_type,
                    discriminant: i,
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;
            
        Ok(EnumAnalysis {
            variants,
            total_bits,
            discriminant_bits,
            data_bits,
            has_data_variants: true,
        })
    } else {
        // Existing logic for unit-only enums
        let total_bits = if let Some(bits) = attributes.bits {
            bits
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
        
        let variants = input.variants.iter().enumerate()
            .map(|(i, variant)| {
                Ok(EnumVariant {
                    name: variant.ident.clone(),
                    variant_type: VariantType::Unit,
                    discriminant: i,
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;
            
        Ok(EnumAnalysis {
            variants,
            total_bits,
            discriminant_bits: total_bits, // For unit enums, all bits are discriminant
            data_bits: 0,
            has_data_variants: false,
        })
    }
}

fn generate_enum(input: &syn::ItemEnum) -> syn::Result<TokenStream2> {
    let span = input.span();
    let attributes = parse_attrs(&input.attrs)?;
    let enum_ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Use the analyze_enum function to handle both unit and data variants
    let analysis = analyze_enum(input, &attributes)?;

    if analysis.has_data_variants {
        // Generate code for enums with data variants
        generate_enum_with_data_variants(input, &analysis, enum_ident, impl_generics, ty_generics, where_clause)
    } else {
        // Generate code for unit-only enums (existing logic)
        generate_unit_enum(input, &analysis, enum_ident, impl_generics, ty_generics, where_clause)
    }
}

fn generate_unit_enum(
    input: &syn::ItemEnum,
    analysis: &EnumAnalysis,
    enum_ident: &syn::Ident,
    impl_generics: syn::ImplGenerics,
    ty_generics: syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
) -> syn::Result<TokenStream2> {
    let span = input.span();
    let bits = analysis.total_bits;

    let variants = analysis.variants.iter()
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

    Ok(quote_spanned!(span=>
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
    ))
}

fn generate_enum_with_data_variants(
    input: &syn::ItemEnum,
    analysis: &EnumAnalysis,
    enum_ident: &syn::Ident,
    impl_generics: syn::ImplGenerics,
    ty_generics: syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
) -> syn::Result<TokenStream2> {
    let span = input.span();
    let total_bits = analysis.total_bits;
    let discriminant_bits = analysis.discriminant_bits;
    let data_bits = analysis.data_bits;

    // Determine the Bytes type based on total bits
    let bytes_type = match total_bits {
        1..=8 => quote! { u8 },
        9..=16 => quote! { u16 },
        17..=32 => quote! { u32 },
        33..=64 => quote! { u64 },
        65..=128 => quote! { u128 },
        _ => return Err(format_err!(span, "enum requires more than 128 bits, which is not supported")),
    };

    // Generate into_bytes match arms
    let into_bytes_arms = analysis.variants.iter().map(|variant| {
        let variant_name = &variant.name;
        let discriminant = variant.discriminant;
        let variant_span = variant_name.span();
        
        match &variant.variant_type {
            VariantType::Unit => {
                // Unit variant: just encode discriminant in data_bits position
                quote_spanned!(variant_span=>
                    Self::#variant_name => {
                        let discriminant = #discriminant as #bytes_type;
                        let result = discriminant << #data_bits;
                        ::core::result::Result::Ok(result)
                    }
                )
            },
            VariantType::Data(data_type) => {
                quote_spanned!(variant_span=>
                    Self::#variant_name(data) => {
                        // Convert data to bytes using its Specifier impl
                        let data_bytes = <#data_type as ::modular_bitfield::Specifier>::into_bytes(data)?;
                        
                        // Convert to target bytes type for packing 
                        let data_value = data_bytes as #bytes_type;
                        
                        // Pack discriminant and data
                        let discriminant = #discriminant as #bytes_type;
                        let packed = discriminant << #data_bits;
                        let data_mask = if #data_bits >= 64 { (#bytes_type::MAX) } else { ((1 as #bytes_type) << #data_bits) - 1 };
                        let result = packed | (data_value & data_mask);
                        
                        ::core::result::Result::Ok(result)
                    }
                )
            }
        }
    });

    // Generate from_bytes match arms
    let from_bytes_arms = analysis.variants.iter().map(|variant| {
        let variant_name = &variant.name;
        let discriminant = variant.discriminant;
        let variant_span = variant_name.span();
        
        match &variant.variant_type {
            VariantType::Unit => {
                quote_spanned!(variant_span=>
                    bytes if bytes >> #data_bits == (#discriminant as #bytes_type) => {
                        ::core::result::Result::Ok(Self::#variant_name)
                    }
                )
            },
            VariantType::Data(data_type) => {
                quote_spanned!(variant_span=>
                    bytes if bytes >> #data_bits == (#discriminant as #bytes_type) => {
                        // Extract data bits
                        let data_mask = if #data_bits >= 64 { (#bytes_type::MAX) } else { ((1 as #bytes_type) << #data_bits) - 1 };
                        let data_value = bytes & data_mask;
                        
                        // Convert back to data type using its Specifier impl
                        let data = <#data_type as ::modular_bitfield::Specifier>::from_bytes(data_value as <#data_type as ::modular_bitfield::Specifier>::Bytes)?;
                        
                        ::core::result::Result::Ok(Self::#variant_name(data))
                    }
                )
            }
        }
    });

    Ok(quote_spanned!(span=>
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
    ))
}
