/// Variable bits functionality for bitfield structs
/// This module contains all logic related to variable-size structs
pub mod analysis;
pub mod errors;
pub mod expand;
pub mod field_config;
pub mod field_config_ext;

pub use analysis::VariableBitsAnalysis;
pub use errors::VariableBitsError;
pub use expand::VariableStructExpander;

#[cfg(test)]
mod tests;