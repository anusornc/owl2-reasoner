//! OWL2 EL Profile Implementation
//!
//! This module implements the Expressive Logic (EL++) profile validation
//! and optimization for OWL2 ontologies.

pub mod validator;
pub mod optimization; // TODO: Fix optimization module

// Re-export EL profile types and functions
pub use validator::*;
pub use optimization::*;
