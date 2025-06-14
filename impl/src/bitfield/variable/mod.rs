/// Variable bits functionality for bitfield structs
/// This module contains all logic related to variable-size structs

pub mod analysis;
pub mod errors;
pub mod expand;

pub use analysis::{VariableBitsAnalysis, VariableStructAnalysis};
pub use errors::VariableBitsError;
pub use expand::VariableStructExpander;