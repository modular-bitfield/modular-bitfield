use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

pub fn generate(_input: TokenStream2) -> TokenStream2 {
    let specifiers = (1usize..=128).map(generate_specifier_for);
    quote! {
        #( #specifiers )*
    }
}

fn generate_specifier_for(bits: usize) -> TokenStream2 {
    let in_out = match bits {
        1..=8 => quote! { ::core::primitive::u8 },
        9..=16 => quote! { ::core::primitive::u16 },
        17..=32 => quote! { ::core::primitive::u32 },
        33..=64 => quote! { ::core::primitive::u64 },
        65..=128 => quote! { ::core::primitive::u128 },
        _ => unreachable!(),
    };
    let ident = format_ident!("B{bits}");
    let doc_comment = if bits == 1 {
        "Specifier for a single bit.".to_string()
    } else {
        format!("Specifier for {bits} bits.")
    };
    let max_value = if bits.is_power_of_two() && bits >= 8 {
        // The compiler can eliminate a check against `x > MAX` entirely
        // so this will yield a no-op in release mode builds.
        quote! {{ <#in_out>::MAX }}
    } else {
        quote! {{ ((0x01 as #in_out) << #bits) - 1 }}
    };
    quote! {
        #[doc = #doc_comment]
        #[derive(Copy, Clone)]
        pub enum #ident {}

        impl crate::Specifier for #ident {
            const BITS: usize = #bits;
            type Bytes = #in_out;
            type InOut = #in_out;

            #[inline]
            fn into_bytes(input: Self::InOut) -> Result<Self::Bytes, crate::OutOfBounds> {
                if input > #max_value {
                    return Err(crate::OutOfBounds)
                }
                Ok(input)
            }

            #[inline]
            fn from_bytes(bytes: Self::Bytes) -> Result<Self::InOut, crate::InvalidBitPattern<Self::Bytes>> {
                if bytes > #max_value {
                    return Err(crate::InvalidBitPattern { invalid_bytes: bytes })
                }
                Ok(bytes)
            }
        }

        impl crate::private::SpecifierBytes for [(); #bits] {
            type Bytes = #in_out;
        }

        impl crate::private::checks::private::Sealed for [(); #bits] {}
    }
}
