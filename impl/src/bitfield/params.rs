use super::config::{Config, SkipMethod};
use proc_macro2::Span;
use syn::{parse::Result, spanned::Spanned};

/// The parameters given to the `#[bitfield]` proc. macro.
pub struct ParamArgs {
    args: Vec<syn::Meta>,
}

impl syn::parse::Parse for ParamArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> Result<Self> {
        let punctuated = <syn::punctuated::Punctuated<_, syn::Token![,]>>::parse_terminated(input)?;
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

    /// Feeds a `skip(...)` parameter to the `#[bitfield]` configuration.
    fn feed_skip_param(&mut self, meta_list: &syn::MetaList) -> Result<()> {
        meta_list.parse_nested_meta(|meta| {
            let path = &meta.path;
            if path.is_ident("all") {
                self.skip(SkipMethod::New, path.span())?;
                self.skip(SkipMethod::FromBytes, path.span())?;
                self.skip(SkipMethod::IntoBytes, path.span())
            } else if path.is_ident("convert") {
                self.skip(SkipMethod::FromBytes, path.span())?;
                self.skip(SkipMethod::IntoBytes, path.span())
            } else if path.is_ident("new") {
                self.skip(SkipMethod::New, path.span())
            } else if path.is_ident("from_bytes") {
                self.skip(SkipMethod::FromBytes, path.span())
            } else if path.is_ident("into_bytes") {
                self.skip(SkipMethod::IntoBytes, path.span())
            } else {
                Err(meta.error("encountered unknown or unsupported #[skip(..)] specifier"))
            }
        })
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
        for param in params {
            if param.path().is_ident("bytes") {
                self.feed_bytes_param(param.require_name_value()?)?;
            } else if param.path().is_ident("bits") {
                self.feed_bits_param(param.require_name_value()?)?;
            } else if param.path().is_ident("filled") {
                self.feed_filled_param(param.require_name_value()?)?;
            } else if param.path().is_ident("skip") {
                self.feed_skip_param(param.require_list()?)?;
            } else {
                return Err(format_err!(
                    param,
                    "encountered unsupported #[bitfield] attribute"
                ));
            }
        }
        Ok(())
    }
}
