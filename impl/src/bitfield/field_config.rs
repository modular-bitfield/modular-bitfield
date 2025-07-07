use super::{config::ConfigValue, raise_skip_error};
use crate::errors::CombineError;
use proc_macro2::Span;

#[derive(Clone)]
pub struct FieldConfig {
    /// Attributes that are re-expanded and going to be ignored by the rest of the `#[bitfield]` invocation.
    pub retained_attrs: Vec<syn::Attribute>,
    /// An encountered `#[bits = N]` attribute on a field.
    pub bits: Option<ConfigValue<usize>>,
    /// An encountered `#[skip]` attribute on a field.
    pub skip: Option<ConfigValue<SkipWhich>>,
    /// An encountered `#[default(...)]` attribute on a field.
    pub default: Option<ConfigValue<syn::Expr>>,
}

/// Controls which parts of the code generation to skip.
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum SkipWhich {
    /// Skip code generation of getters and setters.
    All,
    /// Skip code generation of only getters.
    ///
    /// For field `f` these include:
    ///
    /// - `f`
    /// - `f_or_err`
    Getters,
    /// Skip code generation of only setters.
    ///
    /// For field `f` these include:
    ///
    /// - `set_f`
    /// - `set_f_checked`
    /// - `with_f`
    /// - `with_f_checked`
    Setters,
}

impl SkipWhich {
    /// Returns `true` if code generation of getters should be skipped.
    pub fn skip_getters(self) -> bool {
        matches!(self, Self::All | Self::Getters)
    }

    /// Returns `true` if code generation of setters should be skipped.
    pub fn skip_setters(self) -> bool {
        matches!(self, Self::All | Self::Setters)
    }
}

impl FieldConfig {
    pub fn new() -> Self {
        Self {
            retained_attrs: Vec::new(),
            bits: None,
            skip: None,
            default: None,
        }
    }

    /// Registers the given attribute to be re-expanded and further ignored.
    pub fn retain_attr(&mut self, attr: syn::Attribute) {
        self.retained_attrs.push(attr);
    }

    /// Generic helper for setting config values that ensures no duplicates.
    fn set_config<T>(
        name: &str,
        config: &mut Option<ConfigValue<T>>,
        value: T,
        span: Span,
    ) -> Result<(), syn::Error> {
        if let Some(ref previous) = config {
            Err(format_err!(
                span,
                "encountered duplicate `#[{} = ...]` attribute for field",
                name
            )
            .into_combine(format_err!(
                previous.span,
                "duplicate `#[{} = ...]` here",
                name
            )))
        } else {
            *config = Some(ConfigValue { value, span });
            Ok(())
        }
    }

    /// Sets the `#[bits = N]` if found for a `#[bitfield]` annotated field.
    ///
    /// # Errors
    ///
    /// If previously already registered a `#[bits = N]`.
    pub fn bits(&mut self, amount: usize, span: Span) -> Result<(), syn::Error> {
        Self::set_config("bits", &mut self.bits, amount, span)
    }

    /// Sets the `#[skip(which)]` if found for a `#[bitfield]` annotated field.
    ///
    /// # Syntax
    ///
    /// - `#[skip]` defaults to `SkipWhich::All`.
    /// - `#[skip(getters)]` is `SkipWhich::Getters`.
    /// - `#[skip(setters)]` is `SkipWhich::Setters`.
    /// - `#[skip(getters, setters)]` is the same as `#[skip]`.
    /// - `#[skip(getters)] #[skip(setters)]` is the same as `#[skip]`.
    ///
    /// # Errors
    ///
    /// If previously already registered a `#[skip]` that overlaps with the previous.
    /// E.g. when skipping getters or setters twice. Note that skipping getters followed
    /// by skipping setters is fine.
    pub fn skip(&mut self, which: SkipWhich, span: Span) -> Result<(), syn::Error> {
        match self.skip {
            Some(ref previous) => {
                match which {
                    SkipWhich::All => return raise_skip_error("", span, previous.span),
                    SkipWhich::Getters => {
                        if previous.value == SkipWhich::Getters || previous.value == SkipWhich::All
                        {
                            return raise_skip_error("(getters)", span, previous.span);
                        }
                    }
                    SkipWhich::Setters => {
                        if previous.value == SkipWhich::Setters || previous.value == SkipWhich::All
                        {
                            return raise_skip_error("(setters)", span, previous.span);
                        }
                    }
                }
                self.skip = Some(ConfigValue {
                    value: SkipWhich::All,
                    span: span.join(previous.span).unwrap_or(span),
                });
            }
            None => self.skip = Some(ConfigValue { value: which, span }),
        }
        Ok(())
    }

    /// Returns the span of the skip attribute if the config demands that code generation for setters should be skipped.
    pub fn skip_setters(&self) -> Option<&Span> {
        self.skip
            .as_ref()
            .filter(|config| SkipWhich::skip_setters(config.value))
            .map(|config| &config.span)
    }

    /// Returns the span of the skip attribute if the config demands that code generation for getters should be skipped.
    pub fn skip_getters(&self) -> Option<&Span> {
        self.skip
            .as_ref()
            .filter(|config| SkipWhich::skip_getters(config.value))
            .map(|config| &config.span)
    }

    /// Sets the `#[default = ...]` if found for a `#[bitfield]` annotated field.
    ///
    /// # Errors
    ///
    /// If previously already registered a `#[default = ...]`.
    pub fn default(&mut self, value: syn::Expr, span: Span) -> Result<(), syn::Error> {
        Self::set_config("default", &mut self.default, value, span)
    }
}
