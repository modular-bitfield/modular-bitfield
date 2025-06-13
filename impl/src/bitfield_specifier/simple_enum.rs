use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::spanned::Spanned as _;

/// Original simple enum implementation
/// This preserves the exact logic from the original bitfield_specifier.rs

pub struct Attributes {
    pub bits: Option<usize>,
}

pub fn parse_attrs(attrs: &[syn::Attribute]) -> syn::Result<Attributes> {
    let attributes = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("bits"))
        .try_fold(Attributes { bits: None }, |mut acc, attr| {
            if acc.bits.is_some() {
                return Err(format_err_spanned!(
                    attr,
                    "More than one 'bits' attribute is not permitted",
                ));
            }
            let meta = attr.meta.require_name_value()?;
            acc.bits = if let syn::Expr::Lit(syn::ExprLit {
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
            Ok(acc)
        })?;
    Ok(attributes)
}

pub fn generate_simple_enum(input: &syn::ItemEnum) -> syn::Result<TokenStream2> {
    let span = input.span();
    let attributes = parse_attrs(&input.attrs)?;
    let enum_ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let bits = if let Some(bits) = attributes.bits {
        bits
    } else {
        let count_variants = input.variants.iter().count();
        if !count_variants.is_power_of_two() {
            return Err(format_err!(
                span,
                "#[derive(Specifier)] expected a number of variants which is a power of 2, specify #[bits = {}] if that was your intent",
                count_variants.next_power_of_two().trailing_zeros(),
            ));
        }
        // We can take `trailing_zeros` returns type as the required amount of bits.
        if let Some(power_of_two) = count_variants.checked_next_power_of_two() {
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
        .filter_map(|variant| match &variant.fields {
            syn::Fields::Unit => Some(&variant.ident),
            _ => None,
        })
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
