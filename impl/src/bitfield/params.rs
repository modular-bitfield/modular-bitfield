use super::config::{Config, VariableBitsConfig};
use proc_macro2::Span;
use syn::{parse::Result, spanned::Spanned};

/// The parameters given to the `#[bitfield]` proc. macro.
pub struct ParamArgs {
    args: Vec<syn::Meta>,
}

impl syn::parse::Parse for ParamArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> Result<Self> {
        // Back to original implementation
        let punctuated =
            <syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>>::parse_terminated(input)?;
        Ok(Self {
            args: punctuated.into_iter().collect(),
        })
    }
}

impl IntoIterator for ParamArgs {
    type Item = syn::Meta;
    type IntoIter = std::vec::IntoIter<syn::Meta>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}

impl Config {
    /// Feeds a parameter that takes an integer value to the `#[bitfield]` configuration.
    fn feed_int_param<F>(name_value: &syn::MetaNameValue, name: &str, on_success: F) -> Result<()>
    where
        F: FnOnce(usize, Span) -> Result<()>,
    {
        assert!(name_value.path.is_ident(name));
        match &name_value.value {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) => {
                let span = lit_int.span();
                let value = lit_int.base10_parse::<usize>().map_err(|err| {
                    format_err!(
                        span,
                        "encountered malformatted integer value for `{}` parameter: {}",
                        name,
                        err
                    )
                })?;
                on_success(value, name_value.span())?;
            }
            invalid => {
                return Err(format_err!(
                    invalid,
                    "encountered invalid value argument for #[bitfield] `{}` parameter",
                    name
                ))
            }
        }
        Ok(())
    }

    /// Feeds a `bytes: int` parameter to the `#[bitfield]` configuration.
    fn feed_bytes_param(&mut self, name_value: &syn::MetaNameValue) -> Result<()> {
        Self::feed_int_param(name_value, "bytes", |value, span| self.bytes(value, span))
    }

    /// Feeds a `bytes: int` parameter to the `#[bitfield]` configuration.
    fn feed_bits_param(&mut self, name_value: &syn::MetaNameValue) -> Result<()> {
        Self::feed_int_param(name_value, "bits", |value, span| self.bits(value, span))
    }

    /// Feeds a `filled: bool` parameter to the `#[bitfield]` configuration.
    fn feed_filled_param(&mut self, name_value: &syn::MetaNameValue) -> Result<()> {
        assert!(name_value.path.is_ident("filled"));
        match &name_value.value {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(lit_bool),
                ..
            }) => {
                self.filled(lit_bool.value, name_value.span())?;
            }
            invalid => {
                return Err(format_err!(
                    invalid,
                    "encountered invalid value argument for #[bitfield] `filled` parameter",
                ))
            }
        }
        Ok(())
    }

    /// Feeds a `variable_bits` parameter to the `#[bitfield]` configuration.
    fn feed_variable_bits_param(&mut self, meta: &syn::Meta) -> Result<()> {
        match meta {
            syn::Meta::Path(path) if path.is_ident("variable_bits") => {
                // #[variable_bits] - inferred from variant data enum
                self.variable_bits(VariableBitsConfig::Inferred, path.span())
            }
            syn::Meta::NameValue(name_value) if name_value.path.is_ident("variable_bits") => {
                // #[variable_bits = (32, 64, 96)] - explicit tuple
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

                        self.variable_bits(VariableBitsConfig::Explicit(sizes), name_value.span())
                    }
                    invalid => {
                        return Err(format_err!(
                            invalid,
                            "encountered invalid value argument for #[bitfield] `variable_bits` parameter, expected tuple like (32, 64, 96)"
                        ));
                    }
                }
            }
            _ => {
                return Err(format_err!(
                    meta,
                    "encountered invalid format for variable_bits parameter"
                ));
            }
        }
    }

    /// Feeds the given parameters to the `#[bitfield]` configuration.
    ///
    /// # Errors
    ///
    /// If a parameter is malformatted, unexpected, duplicate or in conflict.
    pub fn feed_params<'a, P>(&mut self, params: P) -> Result<()>
    where
        P: IntoIterator<Item = syn::Meta> + 'a,
    {
        for meta in params {
            match &meta {
                syn::Meta::NameValue(name_value) => {
                    if name_value.path.is_ident("bytes") {
                        self.feed_bytes_param(name_value)?;
                    } else if name_value.path.is_ident("bits") {
                        self.feed_bits_param(name_value)?;
                    } else if name_value.path.is_ident("filled") {
                        self.feed_filled_param(name_value)?;
                    } else if name_value.path.is_ident("variable_bits") {
                        self.feed_variable_bits_param(&meta)?;
                    } else {
                        return Err(format_err!(
                            name_value,
                            "encountered unsupported #[bitfield] attribute"
                        ));
                    }
                }
                syn::Meta::Path(path) => {
                    if path.is_ident("variable_bits") {
                        self.feed_variable_bits_param(&meta)?;
                    } else {
                        return Err(format_err!(
                            path,
                            "encountered unsupported #[bitfield] attribute"
                        ));
                    }
                }
                syn::Meta::List(_) => {
                    return Err(format_err!(
                        meta,
                        "encountered unsupported #[bitfield] attribute format"
                    ));
                }
            }
        }
        Ok(())
    }
}
