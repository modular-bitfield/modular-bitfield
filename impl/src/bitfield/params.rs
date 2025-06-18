use super::config::Config;
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

    /// Feeds a `bits` parameter to the `#[bitfield]` configuration.
    /// Supports both single values and tuples.
    fn feed_bits_param(&mut self, name_value: &syn::MetaNameValue) -> Result<()> {
        assert!(name_value.path.is_ident("bits"));
        match &name_value.value {
            // Single value: #[bitfield(bits = 32)]
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) => {
                let value = lit_int.base10_parse::<usize>()
                    .map_err(|err| {
                        format_err!(
                            lit_int.span(),
                            "invalid integer value for `bits` parameter: {}",
                            err
                        )
                    })?;
                self.bits(super::config::BitsConfig::Fixed(value), name_value.span())?;
            }
            // Tuple: #[bitfield(bits = (32, 64, 128))]
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
                                    "encountered malformatted integer value in bits tuple: {}",
                                    err
                                )
                            })?;
                            sizes.push(size);
                        }
                        invalid => {
                            return Err(format_err!(
                                invalid,
                                "encountered invalid element in bits tuple, expected integer literal"
                            ));
                        }
                    }
                }
                if sizes.is_empty() {
                    return Err(format_err!(
                        tuple.span(),
                        "bits tuple cannot be empty"
                    ));
                }
                
                // Validate sizes are in non-decreasing order (optional constraint for performance)
                for window in sizes.windows(2) {
                    if window[1] < window[0] {
                        return Err(format_err!(
                            tuple.span(),
                            "bits sizes should be in non-decreasing order for optimal performance"
                        ));
                    }
                }
                
                self.bits(super::config::BitsConfig::Variable(sizes), name_value.span())?;
            }
            invalid => {
                return Err(format_err!(
                    invalid,
                    "encountered invalid value argument for #[bitfield] `bits` parameter, expected integer or tuple"
                ))
            }
        }
        Ok(())
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
            // Handle standard parameters
            match &meta {
                syn::Meta::NameValue(name_value) => {
                    if name_value.path.is_ident("bytes") {
                        self.feed_bytes_param(name_value)?;
                    } else if name_value.path.is_ident("bits") {
                        self.feed_bits_param(name_value)?;
                    } else if name_value.path.is_ident("filled") {
                        self.feed_filled_param(name_value)?;
                    } else {
                        return Err(format_err!(
                            name_value,
                            "encountered unsupported #[bitfield] attribute"
                        ));
                    }
                }
                syn::Meta::Path(path) => {
                    return Err(format_err!(
                        path,
                        "encountered unsupported #[bitfield] attribute"
                    ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    
    #[test]
    fn test_param_args_parse() {
        // Test parsing empty parameters
        let empty: ParamArgs = syn::parse2(quote! {}).unwrap();
        assert_eq!(empty.args.len(), 0);
        
        // Test parsing single parameter
        let single: ParamArgs = syn::parse2(quote! { bits = 32 }).unwrap();
        assert_eq!(single.args.len(), 1);
        
        // Test parsing multiple parameters
        let multiple: ParamArgs = syn::parse2(quote! { bits = 32, filled = true, bytes = 4 }).unwrap();
        assert_eq!(multiple.args.len(), 3);
        
        // Test parsing with trailing comma
        let trailing: ParamArgs = syn::parse2(quote! { bits = 32, }).unwrap();
        assert_eq!(trailing.args.len(), 1);
    }
    
    #[test]
    fn test_param_args_into_iterator() {
        let params: ParamArgs = syn::parse2(quote! { bits = 32, filled = true }).unwrap();
        let args: Vec<_> = params.into_iter().collect();
        assert_eq!(args.len(), 2);
    }
    
    #[test]
    fn test_feed_int_param_valid() {
        let meta: syn::Meta = syn::parse_quote! { bytes = 4 };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let mut called = false;
        let result = Config::feed_int_param(&name_value, "bytes", |value, _span| {
            assert_eq!(value, 4);
            called = true;
            Ok(())
        });
        
        assert!(result.is_ok());
        assert!(called);
    }
    
    #[test]
    fn test_feed_int_param_invalid_value() {
        let meta: syn::Meta = syn::parse_quote! { bytes = "string" };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = Config::feed_int_param(&name_value, "bytes", |_value, _span| Ok(()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid value argument"));
    }
    
    #[test]
    fn test_feed_int_param_malformed_integer() {
        // Create a meta with an integer that's too large
        let meta: syn::Meta = syn::parse_quote! { bytes = 99999999999999999999999999999999 };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = Config::feed_int_param(&name_value, "bytes", |_value, _span| Ok(()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("malformatted integer value"));
    }
    
    #[test]
    fn test_feed_bytes_param() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { bytes = 8 };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_bytes_param(&name_value);
        assert!(result.is_ok());
        assert!(config.bytes.is_some());
        assert_eq!(config.bytes.unwrap().value, 8);
    }
    
    #[test]
    fn test_feed_bits_param_single_value() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { bits = 32 };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_bits_param(&name_value);
        assert!(result.is_ok());
        assert!(config.bits.is_some());
        assert!(matches!(config.bits.unwrap().value, super::super::config::BitsConfig::Fixed(32)));
    }
    
    #[test]
    fn test_feed_bits_param_tuple() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { bits = (32, 64, 128) };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_bits_param(&name_value);
        assert!(result.is_ok());
        assert!(config.bits.is_some());
        match &config.bits.unwrap().value {
            super::super::config::BitsConfig::Variable(sizes) => {
                assert_eq!(sizes, &vec![32, 64, 128]);
            }
            _ => panic!("Expected Variable bits config"),
        }
    }
    
    #[test]
    fn test_feed_bits_param_empty_tuple() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { bits = () };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_bits_param(&name_value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bits tuple cannot be empty"));
    }
    
    #[test]
    fn test_feed_bits_param_decreasing_order() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { bits = (64, 32, 128) };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_bits_param(&name_value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-decreasing order"));
    }
    
    #[test]
    fn test_feed_bits_param_invalid_tuple_element() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { bits = (32, "invalid", 128) };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_bits_param(&name_value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid element in bits tuple"));
    }
    
    #[test]
    fn test_feed_bits_param_malformed_int_in_tuple() {
        let mut config = Config::default();
        // Create a tuple with an integer that's too large
        let meta: syn::Meta = syn::parse_quote! { bits = (32, 99999999999999999999999999999999, 128) };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_bits_param(&name_value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("malformatted integer value"));
    }
    
    #[test]
    fn test_feed_bits_param_invalid_type() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { bits = true };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_bits_param(&name_value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid value argument"));
    }
    
    #[test]
    fn test_feed_filled_param_true() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { filled = true };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_filled_param(&name_value);
        assert!(result.is_ok());
        assert!(config.filled.is_some());
        assert_eq!(config.filled.unwrap().value, true);
    }
    
    #[test]
    fn test_feed_filled_param_false() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { filled = false };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_filled_param(&name_value);
        assert!(result.is_ok());
        assert!(config.filled.is_some());
        assert_eq!(config.filled.unwrap().value, false);
    }
    
    #[test]
    fn test_feed_filled_param_invalid() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { filled = 123 };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_filled_param(&name_value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid value argument"));
    }
    
    #[test]
    fn test_feed_params_all_types() {
        let mut config = Config::default();
        let params: ParamArgs = syn::parse2(quote! {
            bits = 32,
            bytes = 4,
            filled = true
        }).unwrap();
        
        let result = config.feed_params(params);
        assert!(result.is_ok());
        assert!(config.bits.is_some());
        assert!(config.bytes.is_some());
        assert!(config.filled.is_some());
    }
    
    #[test]
    fn test_feed_params_unsupported_name_value() {
        let mut config = Config::default();
        let params: ParamArgs = syn::parse2(quote! {
            unknown = 42
        }).unwrap();
        
        let result = config.feed_params(params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unsupported #[bitfield] attribute"));
    }
    
    #[test]
    fn test_feed_params_path_attribute() {
        let mut config = Config::default();
        let params: ParamArgs = syn::parse2(quote! {
            some_flag
        }).unwrap();
        
        let result = config.feed_params(params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unsupported #[bitfield] attribute"));
    }
    
    #[test]
    fn test_feed_params_list_attribute() {
        let mut config = Config::default();
        let params: ParamArgs = syn::parse2(quote! {
            derive(Debug)
        }).unwrap();
        
        let result = config.feed_params(params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unsupported #[bitfield] attribute format"));
    }
    
    #[test]
    fn test_feed_params_multiple_errors_stops_on_first() {
        let mut config = Config::default();
        let params: ParamArgs = syn::parse2(quote! {
            unknown1 = 42,
            unknown2 = 84
        }).unwrap();
        
        let result = config.feed_params(params);
        assert!(result.is_err());
        // Should stop on first error
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("unsupported"));
        // The error should mention one of the unknown attributes
        // (we can't predict which one comes first in iteration order)
    }
    
    #[test]
    fn test_feed_int_param_callback_error() {
        let meta: syn::Meta = syn::parse_quote! { bytes = 4 };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = Config::feed_int_param(&name_value, "bytes", |_value, span| {
            Err(format_err!(span, "callback error"))
        });
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("callback error"));
    }
    
    #[test]
    #[should_panic(expected = "assertion")]
    fn test_feed_int_param_wrong_name() {
        let meta: syn::Meta = syn::parse_quote! { other = 4 };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        // This should panic because name doesn't match
        let _ = Config::feed_int_param(&name_value, "bytes", |_value, _span| Ok(()));
    }
    
    #[test]
    #[should_panic(expected = "assertion")]
    fn test_feed_bits_param_wrong_name() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { other = 32 };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        // This should panic because name doesn't match
        let _ = config.feed_bits_param(&name_value);
    }
    
    #[test]
    #[should_panic(expected = "assertion")]
    fn test_feed_filled_param_wrong_name() {
        let mut config = Config::default();
        let meta: syn::Meta = syn::parse_quote! { other = true };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        // This should panic because name doesn't match
        let _ = config.feed_filled_param(&name_value);
    }
    
    #[test]
    fn test_feed_bits_param_malformed_single_value() {
        let mut config = Config::default();
        // Test with an integer that can't be parsed
        let meta: syn::Meta = syn::parse_quote! { bits = 18446744073709551616 }; // u64::MAX + 1
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_bits_param(&name_value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid integer value"));
    }
    
    #[test]
    fn test_feed_bits_param_non_decreasing_equal_values() {
        let mut config = Config::default();
        // Equal values should be allowed (non-decreasing, not strictly increasing)
        let meta: syn::Meta = syn::parse_quote! { bits = (32, 32, 64, 64) };
        let name_value = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => panic!("Expected NameValue"),
        };
        
        let result = config.feed_bits_param(&name_value);
        assert!(result.is_ok());
        match &config.bits.unwrap().value {
            super::super::config::BitsConfig::Variable(sizes) => {
                assert_eq!(sizes, &vec![32, 32, 64, 64]);
            }
            _ => panic!("Expected Variable bits config"),
        }
    }
    
    #[test]
    fn test_param_args_complex_parsing() {
        // Test with various whitespace and formatting
        let params: ParamArgs = syn::parse2(quote! {
            bits = 32 ,
            filled = true ,
            bytes = 4
        }).unwrap();
        assert_eq!(params.args.len(), 3);
    }
}
