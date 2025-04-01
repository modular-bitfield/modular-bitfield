use super::config::ConfigValue;
use crate::errors::CombineError;
use proc_macro2::Span;

#[derive(Default, Clone)]
pub struct FieldConfig {
    /// Attributes that are re-expanded and going to be ignored by the rest of the `#[bitfield]` invocation.
    pub retained_attrs: Vec<syn::Attribute>,
    /// An encountered `#[bits = N]` attribute on a field.
    pub bits: Option<ConfigValue<usize>>,
    /// An encountered `#[skip]` attribute on a field.
    pub skip: Option<ConfigValue<SkipWhich>>,
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
    /// Registers the given attribute to be re-expanded and further ignored.
    pub fn retain_attr(&mut self, attr: syn::Attribute) {
        self.retained_attrs.push(attr);
    }

    /// Sets the `#[bits = N]` if found for a `#[bitfield]` annotated field.
    ///
    /// # Errors
    ///
    /// If previously already registered a `#[bits = M]`.
    pub fn bits(&mut self, amount: usize, span: Span) -> Result<(), syn::Error> {
        match self.bits {
            Some(ref previous) => {
                return Err(format_err!(
                    span,
                    "encountered duplicate `#[bits = N]` attribute for field"
                )
                .into_combine(format_err!(previous.span, "duplicate `#[bits = M]` here")))
            }
            None => {
                self.bits = Some(ConfigValue {
                    value: amount,
                    span,
                });
            }
        }
        Ok(())
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
        fn raise_skip_error(
            skip_params: &str,
            span: Span,
            previous: &ConfigValue<SkipWhich>,
        ) -> syn::Error {
            format_err!(
                span,
                "encountered duplicate `#[skip{}]` attribute for field",
                skip_params
            )
            .into_combine(format_err!(
                previous.span,
                "duplicate `#[skip{}]` here",
                skip_params
            ))
        }
        match self.skip {
            Some(ref previous) => {
                match which {
                    SkipWhich::All => return Err(raise_skip_error("", span, previous)),
                    SkipWhich::Getters => {
                        if previous.value == SkipWhich::Getters || previous.value == SkipWhich::All
                        {
                            return Err(raise_skip_error("(getters)", span, previous));
                        }
                    }
                    SkipWhich::Setters => {
                        if previous.value == SkipWhich::Setters || previous.value == SkipWhich::All
                        {
                            return Err(raise_skip_error("(setters)", span, previous));
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

    /// Returns `true` if the config demands that code generation for setters should be skipped.
    pub fn skip_setters(&self) -> bool {
        self.skip
            .as_ref()
            .is_some_and(|config| SkipWhich::skip_setters(config.value))
    }

    /// Returns `true` if the config demands that code generation for getters should be skipped.
    pub fn skip_getters(&self) -> bool {
        self.skip
            .as_ref()
            .is_some_and(|config| SkipWhich::skip_getters(config.value))
    }
}
