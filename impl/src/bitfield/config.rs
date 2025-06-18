use super::{
    field_config::FieldConfig,
    variable::field_config_ext::VariableFieldConfigs,
};
use crate::errors::CombineError;
use core::any::TypeId;
use proc_macro2::Span;
use std::collections::{hash_map::Entry, HashMap};
use syn::parse::Result;

/// Unified bits configuration for both fixed and variable sizes
#[derive(Clone)]
pub enum BitsConfig {
    Fixed(usize),         // #[bitfield(bits = 32)]
    Variable(Vec<usize>), // #[bitfield(bits = (32, 64, 96))]
}

impl BitsConfig {
    /// Get the maximum size in bits
    pub fn max_bits(&self) -> usize {
        match self {
            Self::Fixed(size) => *size,
            Self::Variable(sizes) => *sizes.iter().max().unwrap_or(&0),
        }
    }

    /// Check if this is a variable-size configuration
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }

    /// Get the sizes for variable configuration
    pub fn sizes(&self) -> Option<&[usize]> {
        match self {
            Self::Fixed(_) => None,
            Self::Variable(sizes) => Some(sizes),
        }
    }
}

impl core::fmt::Debug for BitsConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Fixed(size) => write!(f, "{}", size),
            Self::Variable(sizes) => write!(f, "{:?}", sizes),
        }
    }
}

/// The configuration for the `#[bitfield]` macro.
#[derive(Default)]
pub struct Config {
    pub bytes: Option<ConfigValue<usize>>,
    pub bits: Option<ConfigValue<BitsConfig>>,
    pub filled: Option<ConfigValue<bool>>,
    pub repr: Option<ConfigValue<ReprKind>>,
    pub derive_debug: Option<ConfigValue<()>>,
    pub derive_specifier: Option<ConfigValue<()>>,
    pub deprecated_specifier: Option<Span>,
    pub retained_attributes: Vec<syn::Attribute>,
    pub field_configs: HashMap<usize, ConfigValue<FieldConfig>>,
    pub variable_field_configs: VariableFieldConfigs,
}

/// Kinds of `#[repr(uN)]` annotations for a `#[bitfield]` struct.
#[derive(Copy, Clone)]
pub enum ReprKind {
    /// Found a `#[repr(u8)]` annotation.
    U8,
    /// Found a `#[repr(u16)]` annotation.
    U16,
    /// Found a `#[repr(u32)]` annotation.
    U32,
    /// Found a `#[repr(u64)]` annotation.
    U64,
    /// Found a `#[repr(u128)]` annotation.
    U128,
}

impl ReprKind {
    /// Returns the amount of bits required to have for the bitfield to satisfy the `#[repr(uN)]`.
    pub fn bits(self) -> usize {
        match self {
            Self::U8 => 8,
            Self::U16 => 16,
            Self::U32 => 32,
            Self::U64 => 64,
            Self::U128 => 128,
        }
    }
}

impl core::fmt::Debug for ReprKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#[repr(u{})]", self.bits())
    }
}

/// A configuration value and its originating span.
#[derive(Clone)]
pub struct ConfigValue<T> {
    /// The actual value of the config.
    pub value: T,
    /// The originating span of the config.
    pub span: Span,
}

impl<T> ConfigValue<T> {
    /// Creates a new config value.
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

impl Config {
    /// Returns the value of the `filled` parameter if provided and otherwise `true`.
    pub fn filled_enabled(&self) -> bool {
        self.filled.as_ref().map_or(true, |config| config.value)
    }

    fn ensure_no_bits_and_repr_conflict(&self) -> Result<()> {
        if let (Some(bits), Some(repr)) = (self.bits.as_ref(), self.repr.as_ref()) {
            let bits_size = bits.value.max_bits();
            if bits_size != repr.value.bits() {
                return Err(format_err!(
                    Span::call_site(),
                    "encountered conflicting `bits` and {:?} parameters",
                    repr.value,
                )
                .into_combine(
                    format_err!(bits.span, "conflicting bits configuration here")
                        .into_combine(format_err!(repr.span, "conflicting {:?} here", repr.value)),
                ));
            }
        }
        Ok(())
    }

    fn ensure_no_bits_and_bytes_conflict(&self) -> Result<()> {
        if let (Some(bits), Some(bytes)) = (self.bits.as_ref(), self.bytes.as_ref()) {
            fn next_div_by_8(value: usize) -> usize {
                ((value.saturating_sub(1) / 8) + 1) * 8
            }
            let bits_size = bits.value.max_bits();
            if next_div_by_8(bits_size) / 8 != bytes.value {
                return Err(format_err!(
                    Span::call_site(),
                    "encountered conflicting bits and `bytes = {}` parameters",
                    bytes.value,
                )
                .into_combine(format_err!(
                    bits.span,
                    "conflicting bits configuration here"
                ))
                .into_combine(format_err!(
                    bytes.span,
                    "conflicting `bytes = {}` here",
                    bytes.value,
                )));
            }
        }
        Ok(())
    }

    pub fn ensure_no_repr_and_filled_conflict(&self) -> Result<()> {
        if let (Some(repr), Some(filled @ ConfigValue { value: false, .. })) =
            (self.repr.as_ref(), self.filled.as_ref())
        {
            return Err(format_err!(
                Span::call_site(),
                "encountered conflicting `{:?}` and `filled = {}` parameters",
                repr.value,
                filled.value,
            )
            .into_combine(format_err!(
                repr.span,
                "conflicting `{:?}` here",
                repr.value
            ))
            .into_combine(format_err!(
                filled.span,
                "conflicting `filled = {}` here",
                filled.value,
            )));
        }
        Ok(())
    }

    /// Ensures that there are no conflicting configuration parameters.
    pub fn ensure_no_conflicts(&self) -> Result<()> {
        self.ensure_no_bits_and_repr_conflict()?;
        self.ensure_no_bits_and_bytes_conflict()?;
        self.ensure_no_repr_and_filled_conflict()?;
        Ok(())
    }

    /// Returns an error showing both the duplicate as well as the previous parameters.
    fn raise_duplicate_error<T>(name: &str, span: Span, previous: &ConfigValue<T>) -> syn::Error
    where
        T: core::fmt::Debug + 'static,
    {
        if TypeId::of::<T>() == TypeId::of::<()>() {
            format_err!(span, "encountered duplicate `{}` parameter", name,)
        } else {
            format_err!(
                span,
                "encountered duplicate `{}` parameter: duplicate set to {:?}",
                name,
                previous.value
            )
        }
        .into_combine(format_err!(
            previous.span,
            "previous `{}` parameter here",
            name
        ))
    }

    /// Sets the `bytes: int` #[bitfield] parameter to the given value.
    ///
    /// # Errors
    ///
    /// If the specifier has already been set.
    pub fn bytes(&mut self, value: usize, span: Span) -> Result<()> {
        match &self.bytes {
            Some(previous) => return Err(Self::raise_duplicate_error("bytes", span, previous)),
            None => self.bytes = Some(ConfigValue::new(value, span)),
        }
        Ok(())
    }

    /// Sets the `bits` #[bitfield] parameter to the given value.
    ///
    /// # Errors
    ///
    /// If the specifier has already been set.
    pub fn bits(&mut self, value: BitsConfig, span: Span) -> Result<()> {
        match &self.bits {
            Some(previous) => return Err(Self::raise_duplicate_error("bits", span, previous)),
            None => self.bits = Some(ConfigValue::new(value, span)),
        }
        Ok(())
    }

    /// Sets the `filled: bool` #[bitfield] parameter to the given value.
    ///
    /// # Errors
    ///
    /// If the specifier has already been set.
    pub fn filled(&mut self, value: bool, span: Span) -> Result<()> {
        match &self.filled {
            Some(previous) => return Err(Self::raise_duplicate_error("filled", span, previous)),
            None => self.filled = Some(ConfigValue::new(value, span)),
        }
        Ok(())
    }

    /// Registers the `#[repr(uN)]` attribute for the #[bitfield] macro.
    ///
    /// # Errors
    ///
    /// If a `#[repr(uN)]` attribute has already been found.
    pub fn repr(&mut self, value: ReprKind, span: Span) -> Result<()> {
        match &self.repr {
            Some(previous) => {
                return Err(Self::raise_duplicate_error("#[repr(uN)]", span, previous))
            }
            None => self.repr = Some(ConfigValue::new(value, span)),
        }
        Ok(())
    }

    /// Registers the `#[derive(Debug)]` attribute for the #[bitfield] macro.
    ///
    /// # Errors
    ///
    /// If a `#[derive(Debug)]` attribute has already been found.
    pub fn derive_debug(&mut self, span: Span) -> Result<()> {
        match &self.derive_debug {
            Some(previous) => {
                return Err(Self::raise_duplicate_error(
                    "#[derive(Debug)]",
                    span,
                    previous,
                ))
            }
            None => self.derive_debug = Some(ConfigValue::new((), span)),
        }
        Ok(())
    }

    /// Registers the `#[derive(Specifier)]` attribute for the #[bitfield] macro.
    ///
    /// # Errors
    ///
    /// If a `#[derive(Specifier)]` attribute has already been found.
    pub fn derive_specifier(&mut self, span: Span) -> Result<()> {
        match &self.derive_specifier {
            Some(previous) => {
                return Err(Self::raise_duplicate_error(
                    "#[derive(Specifier)]",
                    span,
                    previous,
                ))
            }
            None => self.derive_specifier = Some(ConfigValue::new((), span)),
        }
        Ok(())
    }

    /// Tell codegen to raise a warning about use of the deprecated
    /// `#[derive(BitfieldSpecifier)]`.
    pub fn deprecated_specifier(&mut self, span: Span) {
        self.deprecated_specifier = Some(span);
    }

    /// Pushes another retained attribute that the #[bitfield] macro is going to re-expand and ignore.
    pub fn push_retained_attribute(&mut self, retained_attr: syn::Attribute) {
        self.retained_attributes.push(retained_attr);
    }

    /// Sets the field configuration and retained attributes for the given field.
    ///
    /// By convention we use the fields name to identify the field if existing.
    /// Otherwise we turn the fields discriminant into an appropriate string.
    ///
    /// # Errors
    ///
    /// If duplicate field configurations have been found for a field.
    pub fn field_config(&mut self, index: usize, span: Span, config: FieldConfig) -> Result<()> {
        match self.field_configs.entry(index) {
            Entry::Occupied(occupied) => {
                return Err(format_err!(span, "encountered duplicate config for field")
                    .into_combine(format_err!(occupied.get().span, "previous config here")))
            }
            Entry::Vacant(vacant) => {
                vacant.insert(ConfigValue::new(config, span));
            }
        }
        Ok(())
    }

    /// Returns true if this struct uses variable bits
    #[allow(dead_code)]
    pub fn is_variable_bits(&self) -> bool {
        self.bits.as_ref().map_or(false, |b| b.value.is_variable())
    }
}
