use crate::bitfield::field_config::FieldConfig;
use super::field_config::{VariableFieldConfig, VariantRole};
use std::collections::HashMap;
use proc_macro2::Span;

/// Storage for variable field configurations
#[derive(Default)]
pub struct VariableFieldConfigs {
    configs: HashMap<usize, VariableFieldConfig>,
}

impl VariableFieldConfigs {
    /// Get the variable config for a field index
    pub fn get(&self, index: usize) -> Option<&VariableFieldConfig> {
        self.configs.get(&index)
    }

    /// Set variant discriminator for a field
    pub fn set_variant_discriminator(&mut self, index: usize, span: Span) -> Result<(), syn::Error> {
        let config = self.configs.entry(index).or_default();
        config.variant_discriminator(span)
    }

    /// Set variant data for a field
    pub fn set_variant_data(&mut self, index: usize, span: Span) -> Result<(), syn::Error> {
        let config = self.configs.entry(index).or_default();
        config.variant_data(span)
    }
}

/// Extension trait for field configuration to work with variable bits
pub trait VariableFieldConfigExt {
    /// Get the variant role for this field from external variable config
    fn variant_role(&self, field_index: usize, variable_configs: &VariableFieldConfigs) -> Option<VariantRole>;

    /// Check if this field is a variant discriminator
    fn is_variant_discriminator(&self, field_index: usize, variable_configs: &VariableFieldConfigs) -> bool;

    /// Check if this field is variant data
    fn is_variant_data(&self, field_index: usize, variable_configs: &VariableFieldConfigs) -> bool;

    /// Check if this field is fixed (not part of variable bits)
    fn is_fixed_field(&self, field_index: usize, variable_configs: &VariableFieldConfigs) -> bool;
}

impl VariableFieldConfigExt for FieldConfig {
    fn variant_role(&self, field_index: usize, variable_configs: &VariableFieldConfigs) -> Option<VariantRole> {
        variable_configs.get(field_index)?.variant_role()
    }

    fn is_variant_discriminator(&self, field_index: usize, variable_configs: &VariableFieldConfigs) -> bool {
        variable_configs.get(field_index).map_or(false, |c| c.is_variant_discriminator())
    }

    fn is_variant_data(&self, field_index: usize, variable_configs: &VariableFieldConfigs) -> bool {
        variable_configs.get(field_index).map_or(false, |c| c.is_variant_data())
    }

    fn is_fixed_field(&self, field_index: usize, variable_configs: &VariableFieldConfigs) -> bool {
        variable_configs.get(field_index).map_or(true, |c| c.variant_role().is_none())
    }
}