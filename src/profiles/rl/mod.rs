//! OWL2 RL Profile Implementation
//!
//! This module implements the Rule Language (OWL2 RL) profile validation
//! and optimization for OWL2 ontologies.

pub mod validator;
// pub mod optimization; // TODO: Fix optimization module

// Re-export RL profile types and functions
pub use validator::*;
// pub use optimization::*;
