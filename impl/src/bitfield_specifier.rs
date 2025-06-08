use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned as _;

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
    Data(Box<syn::Type>),          // Has data of specified type
}

struct EnumVariant {
    name: syn::Ident,
    variant_type: VariantType,
}

struct EnumAnalysis {
    variants: Vec<EnumVariant>,
    total_bits: usize,
    has_data_variants: bool,       // True if any variant has data
}

struct Attributes {
    bits: Option<usize>,
}

fn parse_attrs(attrs: &[syn::Attribute]) -> syn::Result<Attributes> {
    let mut attributes = Attributes { 
        bits: None,
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
        }
    }
    Ok(attributes)
}

fn analyze_enum(input: &syn::ItemEnum, attributes: &Attributes) -> syn::Result<EnumAnalysis> {
    let span = input.span();
    
    // Check if any variants have data
    let has_data_variants = input.variants.iter()
        .any(|v| !matches!(v.fields, syn::Fields::Unit));
    
    if has_data_variants {
        // Data variants require explicit bits specification
        let total_bits = attributes.bits.ok_or_else(|| {
            format_err!(span, "enums with data variants must specify #[bits = N]")
        })?;
        
        // Classify each variant
        let variants = input.variants.iter()
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
                
                Ok(EnumVariant {
                    name: variant.ident.clone(),
                    variant_type,
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
        
        let variants = input.variants.iter()
            .map(|variant| {
                Ok(EnumVariant {
                    name: variant.ident.clone(),
                    variant_type: VariantType::Unit,
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

    // Use the analyze_enum function to handle both unit and data variants
    let analysis = analyze_enum(input, &attributes)?;

    if analysis.has_data_variants {
        // Generate code for enums with data variants - external discrimination
        generate_enum_with_data_variants(&analysis, enum_ident, &impl_generics, &ty_generics, where_clause)
    } else {
        // Generate code for unit-only enums (existing logic)
        Ok(generate_unit_enum(input, &analysis, enum_ident, &impl_generics, &ty_generics, where_clause))
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
        _ => return Err(format_err!(span, "enum requires more than 128 bits, which is not supported")),
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
    let data_types: Vec<_> = analysis.variants.iter().filter_map(|variant| {
        match &variant.variant_type {
            VariantType::Data(data_type) => Some(&**data_type),
            VariantType::Unit => None,
        }
    }).collect();
    
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
            },
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
