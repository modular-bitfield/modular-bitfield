use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod simple_enum;
mod variable_enum;

/// Main entry point for bitfield specifier generation
/// This dispatcher determines whether to use simple or variable enum logic
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
        syn::Data::Enum(data_enum) => {
            let item_enum = syn::ItemEnum {
                attrs: input.attrs,
                vis: input.vis,
                enum_token: data_enum.enum_token,
                ident: input.ident,
                generics: input.generics,
                brace_token: data_enum.brace_token,
                variants: data_enum.variants,
            };
            
            // Check if this enum has data variants
            let has_data_variants = item_enum.variants.iter().any(|v| !matches!(v.fields, syn::Fields::Unit));
            
            if has_data_variants {
                // Parse attributes for variable enum
                let attrs_result = variable_enum::parse_attrs(&item_enum.attrs);
                
                match attrs_result {
                    Ok(attrs) => {
                        // Use variable enum logic for enums with data
                        variable_enum::generate_variable_enum(&item_enum, attrs)
                    }
                    Err(e) => Err(e),
                }
            } else {
                // Simple unit-only enum
                simple_enum::generate_simple_enum(&item_enum)
            }
        },
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