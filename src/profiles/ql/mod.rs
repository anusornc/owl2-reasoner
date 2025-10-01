//! OWL2 QL Profile Implementation
//!
//! This module implements the Query Language (OWL2 QL) profile validation
//! and optimization for OWL2 ontologies.

pub mod validator;
// pub mod optimization; // TODO: Fix optimization module

// Re-export QL profile types and functions
pub use validator::*;
// pub use optimization::*;
