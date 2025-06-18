//! Unit tests for field configuration extensions

use super::super::field_config_ext::{VariableFieldConfigs, VariableFieldConfigExt};
use super::super::field_config::{VariantRole};
use crate::bitfield::field_config::FieldConfig;
use proc_macro2::Span;

#[test]
fn test_variable_field_configs_default() {
    let configs = VariableFieldConfigs::default();
    
    // Should return None for any index
    assert!(configs.get(0).is_none());
    assert!(configs.get(100).is_none());
}

#[test]
fn test_set_variant_discriminator() {
    let mut configs = VariableFieldConfigs::default();
    
    // Set discriminator for field 0
    let result = configs.set_variant_discriminator(0, Span::call_site());
    assert!(result.is_ok());
    
    // Verify it was set
    let config = configs.get(0).unwrap();
    assert!(config.is_variant_discriminator());
    assert!(!config.is_variant_data());
    assert_eq!(config.variant_role(), Some(VariantRole::Discriminator));
}

#[test]
fn test_set_variant_data() {
    let mut configs = VariableFieldConfigs::default();
    
    // Set data for field 1
    let result = configs.set_variant_data(1, Span::call_site());
    assert!(result.is_ok());
    
    // Verify it was set
    let config = configs.get(1).unwrap();
    assert!(!config.is_variant_discriminator());
    assert!(config.is_variant_data());
    assert_eq!(config.variant_role(), Some(VariantRole::Data));
}

#[test]
fn test_duplicate_variant_discriminator() {
    let mut configs = VariableFieldConfigs::default();
    
    // Set discriminator once
    let result1 = configs.set_variant_discriminator(0, Span::call_site());
    assert!(result1.is_ok());
    
    // Try to set it again
    let result2 = configs.set_variant_discriminator(0, Span::call_site());
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("duplicate"));
}

#[test]
fn test_duplicate_variant_data() {
    let mut configs = VariableFieldConfigs::default();
    
    // Set data once
    let result1 = configs.set_variant_data(0, Span::call_site());
    assert!(result1.is_ok());
    
    // Try to set it again
    let result2 = configs.set_variant_data(0, Span::call_site());
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("duplicate"));
}

#[test]
fn test_conflicting_attributes() {
    let mut configs = VariableFieldConfigs::default();
    
    // Set discriminator first
    let result1 = configs.set_variant_discriminator(0, Span::call_site());
    assert!(result1.is_ok());
    
    // Try to set data on same field
    let result2 = configs.set_variant_data(0, Span::call_site());
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("cannot be both"));
    
    // Test opposite order
    let mut configs2 = VariableFieldConfigs::default();
    
    // Set data first
    let result3 = configs2.set_variant_data(1, Span::call_site());
    assert!(result3.is_ok());
    
    // Try to set discriminator on same field
    let result4 = configs2.set_variant_discriminator(1, Span::call_site());
    assert!(result4.is_err());
    assert!(result4.unwrap_err().to_string().contains("cannot be both"));
}

#[test]
fn test_multiple_fields() {
    let mut configs = VariableFieldConfigs::default();
    
    // Set different roles for different fields
    configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    configs.set_variant_data(1, Span::call_site()).unwrap();
    configs.set_variant_discriminator(2, Span::call_site()).unwrap();
    configs.set_variant_data(3, Span::call_site()).unwrap();
    
    // Verify each field
    assert_eq!(configs.get(0).unwrap().variant_role(), Some(VariantRole::Discriminator));
    assert_eq!(configs.get(1).unwrap().variant_role(), Some(VariantRole::Data));
    assert_eq!(configs.get(2).unwrap().variant_role(), Some(VariantRole::Discriminator));
    assert_eq!(configs.get(3).unwrap().variant_role(), Some(VariantRole::Data));
    assert!(configs.get(4).is_none());
}

// Tests for VariableFieldConfigExt trait implementation

#[test]
fn test_field_config_variant_role() {
    let mut configs = VariableFieldConfigs::default();
    configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    configs.set_variant_data(1, Span::call_site()).unwrap();
    
    let field_config = FieldConfig::default();
    
    // Test variant_role method
    assert_eq!(field_config.variant_role(0, &configs), Some(VariantRole::Discriminator));
    assert_eq!(field_config.variant_role(1, &configs), Some(VariantRole::Data));
    assert_eq!(field_config.variant_role(2, &configs), None);
}

#[test]
fn test_field_config_is_variant_discriminator() {
    let mut configs = VariableFieldConfigs::default();
    configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    configs.set_variant_data(1, Span::call_site()).unwrap();
    
    let field_config = FieldConfig::default();
    
    assert!(field_config.is_variant_discriminator(0, &configs));
    assert!(!field_config.is_variant_discriminator(1, &configs));
    assert!(!field_config.is_variant_discriminator(2, &configs));
}

#[test]
fn test_field_config_is_variant_data() {
    let mut configs = VariableFieldConfigs::default();
    configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    configs.set_variant_data(1, Span::call_site()).unwrap();
    
    let field_config = FieldConfig::default();
    
    assert!(!field_config.is_variant_data(0, &configs));
    assert!(field_config.is_variant_data(1, &configs));
    assert!(!field_config.is_variant_data(2, &configs));
}

#[test]
fn test_field_config_is_fixed_field() {
    let mut configs = VariableFieldConfigs::default();
    configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    configs.set_variant_data(1, Span::call_site()).unwrap();
    
    let field_config = FieldConfig::default();
    
    // Fields with variant roles are not fixed
    assert!(!field_config.is_fixed_field(0, &configs));
    assert!(!field_config.is_fixed_field(1, &configs));
    
    // Fields without variant roles are fixed
    assert!(field_config.is_fixed_field(2, &configs));
    assert!(field_config.is_fixed_field(99, &configs));
}

#[test]
fn test_empty_configs_all_fields_fixed() {
    let configs = VariableFieldConfigs::default();
    let field_config = FieldConfig::default();
    
    // With empty configs, all fields should be considered fixed
    assert!(field_config.is_fixed_field(0, &configs));
    assert!(field_config.is_fixed_field(1, &configs));
    assert!(field_config.is_fixed_field(100, &configs));
}

#[test]
fn test_sparse_field_indices() {
    let mut configs = VariableFieldConfigs::default();
    
    // Set sparse indices
    configs.set_variant_discriminator(10, Span::call_site()).unwrap();
    configs.set_variant_data(50, Span::call_site()).unwrap();
    configs.set_variant_discriminator(100, Span::call_site()).unwrap();
    
    let field_config = FieldConfig::default();
    
    // Check sparse indices work correctly
    assert_eq!(field_config.variant_role(10, &configs), Some(VariantRole::Discriminator));
    assert_eq!(field_config.variant_role(50, &configs), Some(VariantRole::Data));
    assert_eq!(field_config.variant_role(100, &configs), Some(VariantRole::Discriminator));
    
    // Check gaps
    assert_eq!(field_config.variant_role(9, &configs), None);
    assert_eq!(field_config.variant_role(11, &configs), None);
    assert_eq!(field_config.variant_role(49, &configs), None);
    assert_eq!(field_config.variant_role(51, &configs), None);
}

#[test]
fn test_get_returns_reference() {
    let mut configs = VariableFieldConfigs::default();
    configs.set_variant_discriminator(0, Span::call_site()).unwrap();
    
    // Get should return a reference that can be used multiple times
    let config_ref1 = configs.get(0);
    let config_ref2 = configs.get(0);
    
    assert!(config_ref1.is_some());
    assert!(config_ref2.is_some());
    assert!(config_ref1.unwrap().is_variant_discriminator());
    assert!(config_ref2.unwrap().is_variant_discriminator());
}