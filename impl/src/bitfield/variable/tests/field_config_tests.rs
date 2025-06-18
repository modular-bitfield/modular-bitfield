//! Unit tests for variable field configuration

use super::super::field_config::{VariableFieldConfig, VariantRole};
use super::super::field_config_ext::{VariableFieldConfigs, VariableFieldConfigExt};
use crate::bitfield::field_config::FieldConfig;
use proc_macro2::Span;

#[test]
fn test_variable_field_config_default() {
    let config = VariableFieldConfig::default();
    assert!(!config.is_variant_discriminator());
    assert!(!config.is_variant_data());
    assert_eq!(config.variant_role(), None);
}

#[test]
fn test_set_variant_discriminator() {
    let mut config = VariableFieldConfig::default();
    
    // Set discriminator
    let result = config.variant_discriminator(Span::call_site());
    assert!(result.is_ok());
    
    assert!(config.is_variant_discriminator());
    assert!(!config.is_variant_data());
    assert_eq!(config.variant_role(), Some(VariantRole::Discriminator));
}

#[test]
fn test_set_variant_data() {
    let mut config = VariableFieldConfig::default();
    
    // Set data
    let result = config.variant_data(Span::call_site());
    assert!(result.is_ok());
    
    assert!(!config.is_variant_discriminator());
    assert!(config.is_variant_data());
    assert_eq!(config.variant_role(), Some(VariantRole::Data));
}

#[test]
fn test_duplicate_variant_discriminator() {
    let mut config = VariableFieldConfig::default();
    
    // Set discriminator once
    let result1 = config.variant_discriminator(Span::call_site());
    assert!(result1.is_ok());
    
    // Try to set again
    let result2 = config.variant_discriminator(Span::call_site());
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("duplicate"));
}

#[test]
fn test_duplicate_variant_data() {
    let mut config = VariableFieldConfig::default();
    
    // Set data once
    let result1 = config.variant_data(Span::call_site());
    assert!(result1.is_ok());
    
    // Try to set again
    let result2 = config.variant_data(Span::call_site());
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("duplicate"));
}

#[test]
fn test_conflicting_discriminator_data() {
    let mut config = VariableFieldConfig::default();
    
    // Set discriminator
    let result1 = config.variant_discriminator(Span::call_site());
    assert!(result1.is_ok());
    
    // Try to set data on same field
    let result2 = config.variant_data(Span::call_site());
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("both"));
}

#[test]
fn test_conflicting_data_discriminator() {
    let mut config = VariableFieldConfig::default();
    
    // Set data
    let result1 = config.variant_data(Span::call_site());
    assert!(result1.is_ok());
    
    // Try to set discriminator on same field
    let result2 = config.variant_discriminator(Span::call_site());
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("both"));
}

#[test]
fn test_variable_field_configs_default() {
    let configs = VariableFieldConfigs::default();
    
    // Should return None for any index
    assert!(configs.get(0).is_none());
    assert!(configs.get(100).is_none());
}

#[test]
fn test_variable_field_configs_set_discriminator() {
    let mut configs = VariableFieldConfigs::default();
    
    // Set discriminator for field index 0
    let result = configs.set_variant_discriminator(0, Span::call_site());
    assert!(result.is_ok());
    
    // Check it was set
    let field_config = configs.get(0).unwrap();
    assert!(field_config.is_variant_discriminator());
    assert!(!field_config.is_variant_data());
}

#[test]
fn test_variable_field_configs_set_data() {
    let mut configs = VariableFieldConfigs::default();
    
    // Set data for field index 1
    let result = configs.set_variant_data(1, Span::call_site());
    assert!(result.is_ok());
    
    // Check it was set
    let field_config = configs.get(1).unwrap();
    assert!(!field_config.is_variant_discriminator());
    assert!(field_config.is_variant_data());
}

#[test]
fn test_variable_field_configs_multiple_fields() {
    let mut configs = VariableFieldConfigs::default();
    
    // Set different roles for different fields
    configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    configs.set_variant_data(2, Span::call_site()).unwrap();
    
    // Check field 0
    let field0 = configs.get(0).unwrap();
    assert!(field0.is_variant_discriminator());
    
    // Check field 1 (not set)
    assert!(configs.get(1).is_none());
    
    // Check field 2
    let field2 = configs.get(2).unwrap();
    assert!(field2.is_variant_data());
}

#[test]
fn test_variable_field_config_ext_variant_role() {
    let field_config = FieldConfig::default();
    let mut variable_configs = VariableFieldConfigs::default();
    
    // Initially no role
    assert_eq!(field_config.variant_role(0, &variable_configs), None);
    
    // Set discriminator role
    variable_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    assert_eq!(field_config.variant_role(0, &variable_configs), Some(VariantRole::Discriminator));
    
    // Different field has no role
    assert_eq!(field_config.variant_role(1, &variable_configs), None);
}

#[test]
fn test_variable_field_config_ext_is_variant_discriminator() {
    let field_config = FieldConfig::default();
    let mut variable_configs = VariableFieldConfigs::default();
    
    // Initially false
    assert!(!field_config.is_variant_discriminator(0, &variable_configs));
    
    // Set discriminator
    variable_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    assert!(field_config.is_variant_discriminator(0, &variable_configs));
    
    // Other fields are false
    assert!(!field_config.is_variant_discriminator(1, &variable_configs));
}

#[test]
fn test_variable_field_config_ext_is_variant_data() {
    let field_config = FieldConfig::default();
    let mut variable_configs = VariableFieldConfigs::default();
    
    // Initially false
    assert!(!field_config.is_variant_data(0, &variable_configs));
    
    // Set data
    variable_configs.set_variant_data(0, Span::call_site()).unwrap();
    assert!(field_config.is_variant_data(0, &variable_configs));
    
    // Other fields are false
    assert!(!field_config.is_variant_data(1, &variable_configs));
}

#[test]
fn test_variable_field_config_ext_is_fixed_field() {
    let field_config = FieldConfig::default();
    let mut variable_configs = VariableFieldConfigs::default();
    
    // Initially all fields are fixed
    assert!(field_config.is_fixed_field(0, &variable_configs));
    assert!(field_config.is_fixed_field(1, &variable_configs));
    assert!(field_config.is_fixed_field(2, &variable_configs));
    
    // Set field 0 as discriminator
    variable_configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    assert!(!field_config.is_fixed_field(0, &variable_configs));
    assert!(field_config.is_fixed_field(1, &variable_configs));
    
    // Set field 2 as data
    variable_configs.set_variant_data(2, Span::call_site()).unwrap();
    assert!(!field_config.is_fixed_field(0, &variable_configs));
    assert!(field_config.is_fixed_field(1, &variable_configs));
    assert!(!field_config.is_fixed_field(2, &variable_configs));
}

#[test]
fn test_variant_role_equality() {
    assert_eq!(VariantRole::Discriminator, VariantRole::Discriminator);
    assert_eq!(VariantRole::Data, VariantRole::Data);
    assert_ne!(VariantRole::Discriminator, VariantRole::Data);
}

#[test]
fn test_variable_field_configs_sparse_indices() {
    let mut configs = VariableFieldConfigs::default();
    
    // Set fields with sparse indices
    configs.set_variant_discriminator(5, Span::call_site()).unwrap();
    configs.set_variant_data(100, Span::call_site()).unwrap();
    
    // Check they're set correctly
    assert!(configs.get(5).unwrap().is_variant_discriminator());
    assert!(configs.get(100).unwrap().is_variant_data());
    
    // Intermediate indices are still None
    assert!(configs.get(0).is_none());
    assert!(configs.get(50).is_none());
    assert!(configs.get(99).is_none());
}