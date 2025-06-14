use crate::bitfield::config::{Config, VariableBitsConfig};
use syn::{parse::Result, spanned::Spanned};

/// Extension trait for parsing variable-bits specific parameters
pub trait VariableParamsExt {
    /// Parse variable_bits parameter
    fn parse_variable_bits(&mut self, meta: &syn::Meta) -> Result<bool>;
}

impl VariableParamsExt for Config {
    fn parse_variable_bits(&mut self, meta: &syn::Meta) -> Result<bool> {
        match meta {
            syn::Meta::Path(path) if path.is_ident("variable_bits") => {
                // #[variable_bits] - inferred from variant data enum
                self.variable_bits(VariableBitsConfig::Inferred, path.span())?;
                Ok(true)
            }
            syn::Meta::NameValue(name_value) if name_value.path.is_ident("variable_bits") => {
                // #[variable_bits = (32, 64, 96)] - explicit tuple
                parse_variable_bits_tuple(name_value, self)?;
                Ok(true)
            }
            _ => Ok(false), // Not a variable_bits parameter
        }
    }
}

/// Parse the variable_bits tuple value
fn parse_variable_bits_tuple(name_value: &syn::MetaNameValue, config: &mut Config) -> Result<()> {
    match &name_value.value {
        syn::Expr::Tuple(tuple) => {
            let mut sizes = Vec::new();
            for element in &tuple.elems {
                match element {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(lit_int),
                        ..
                    }) => {
                        let size = lit_int.base10_parse::<usize>().map_err(|err| {
                            format_err!(
                                lit_int.span(),
                                "encountered malformatted integer value in variable_bits tuple: {}",
                                err
                            )
                        })?;
                        sizes.push(size);
                    }
                    invalid => {
                        return Err(format_err!(
                            invalid,
                            "encountered invalid element in variable_bits tuple, expected integer literal"
                        ));
                    }
                }
            }
            if sizes.is_empty() {
                return Err(format_err!(
                    tuple.span(),
                    "variable_bits tuple cannot be empty"
                ));
            }

            // Validate sizes are in non-decreasing order (optional constraint for performance)
            for window in sizes.windows(2) {
                if window[1] < window[0] {
                    return Err(format_err!(
                        tuple.span(),
                        "variable_bits sizes should be in non-decreasing order for optimal performance"
                    ));
                }
            }

            config.variable_bits(VariableBitsConfig::Explicit(sizes), name_value.span())
        }
        invalid => {
            Err(format_err!(
                invalid,
                "encountered invalid value argument for #[bitfield] `variable_bits` parameter, expected tuple like (32, 64, 96)"
            ))
        }
    }
}