//! OWL2 Reasoner - Core Data Model
//! 
//! This module provides the foundational data structures for representing
//! OWL2 ontologies, entities, and axioms in a memory-efficient and
//! type-safe manner.

/// OWL2 Reasoner error types
pub mod error;

/// IRI management for OWL2 entities
pub mod iri;

/// OWL2 Entities - Classes, Properties, and Individuals
pub mod entities;

/// OWL2 Axioms - Logical statements about entities
pub mod axioms;

/// Ontology structure and management
pub mod ontology;

/// Storage backends for OWL2 ontologies
pub mod storage;

/// OWL2 syntax parsers
pub mod parser;

/// OWL2 reasoning engine
pub mod reasoning;

/// Comprehensive test suite
#[cfg(test)]
pub mod tests;

// Re-export common types for convenience
pub use error::{OwlError, OwlResult};
pub use iri::{IRI, IRIRef};
pub use entities::*;
pub use axioms::*;
pub use ontology::*;
pub use storage::*;
pub use parser::*;
pub use reasoning::*;