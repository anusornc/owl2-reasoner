//! OWL2 Axioms - Logical statements about entities
//! 
//! This module defines all OWL2 axiom types that express logical relationships
//! between classes, properties, and individuals.

pub mod class_expressions;
pub mod property_expressions;

pub use class_expressions::*;
pub use property_expressions::*;