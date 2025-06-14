use crate::bitfield::config::ConfigValue;
use proc_macro2::Span;

/// Variable-specific field configuration
#[derive(Default, Clone)]
pub struct VariableFieldConfig {
    /// An encountered `#[variant_discriminator]` attribute on a field.
    pub variant_discriminator: Option<ConfigValue<()>>,
    /// An encountered `#[variant_data]` attribute on a field.
    pub variant_data: Option<ConfigValue<()>>,
}

/// Role of a field in variable-size structs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantRole {
    Discriminator, // Field that determines which variant is active
    Data,          // Field that contains variable-size data
}

impl VariableFieldConfig {
    /// Sets the `#[variant_discriminator]` attribute for a field
    ///
    /// # Errors
    ///
    /// If previously already registered a `#[variant_discriminator]` or if the field
    /// is already marked as `#[variant_data]`.
    pub fn variant_discriminator(&mut self, span: Span) -> Result<(), syn::Error> {
        if self.variant_discriminator.is_some() {
            return Err(format_err!(
                span,
                "duplicate #[variant_discriminator] attribute"
            ));
        }
        if self.variant_data.is_some() {
            return Err(format_err!(
                span,
                "field cannot be both variant_discriminator and variant_data"
            ));
        }
        self.variant_discriminator = Some(ConfigValue::new((), span));
        Ok(())
    }

    /// Sets the `#[variant_data]` attribute for a field
    ///
    /// # Errors
    ///
    /// If previously already registered a `#[variant_data]` or if the field
    /// is already marked as `#[variant_discriminator]`.
    pub fn variant_data(&mut self, span: Span) -> Result<(), syn::Error> {
        if self.variant_data.is_some() {
            return Err(format_err!(span, "duplicate #[variant_data] attribute"));
        }
        if self.variant_discriminator.is_some() {
            return Err(format_err!(
                span,
                "field cannot be both variant_discriminator and variant_data"
            ));
        }
        self.variant_data = Some(ConfigValue::new((), span));
        Ok(())
    }

    /// Returns true if this field is marked as a variant discriminator
    pub fn is_variant_discriminator(&self) -> bool {
        self.variant_discriminator.is_some()
    }

    /// Returns true if this field is marked as variant data
    pub fn is_variant_data(&self) -> bool {
        self.variant_data.is_some()
    }

    /// Returns the variant role if any
    pub fn variant_role(&self) -> Option<VariantRole> {
        if self.is_variant_discriminator() {
            Some(VariantRole::Discriminator)
        } else if self.is_variant_data() {
            Some(VariantRole::Data)
        } else {
            None
        }
    }
}