/// Variable bits functionality for bitfield structs
/// This module contains all logic related to variable-size structs

pub mod analysis;
pub mod errors;
pub mod expand;
pub mod field_config;
pub mod field_config_ext;
pub mod params;

pub use analysis::VariableBitsAnalysis;
pub use errors::VariableBitsError;
pub use expand::VariableStructExpander;
pub use field_config::VariantRole;
pub use field_config_ext::VariableFieldConfigExt;
pub use params::VariableParamsExt;