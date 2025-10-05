//! # OWL2 Reasoner
//!
//! A high-performance, feature-complete OWL2 reasoning engine implemented in Rust.
//!
//! ## Features
//!
//! - **Complete OWL2 DL support** with SROIQ(D) description logic
//! - **High-performance reasoning** with optimized data structures and algorithms
//! - **Multi-format parsing** for Turtle, RDF/XML, OWL/XML, and N-Triples
//! - **SPARQL-like query engine** with pattern matching and optimization
//! - **Memory-efficient storage** with indexed axiom access and caching
//! - **Type-safe API** leveraging Rust's type system for correctness
//!
//! ## Quick Start
//!
//! ```rust
//! use owl2_reasoner::{Ontology, Class, SimpleReasoner, SubClassOfAxiom, ClassExpression};
//!
//! // Create a new ontology
//! let mut ontology = Ontology::new();
//!
//! // Add classes
//! let person = Class::new("http://example.org/Person");
//! let parent = Class::new("http://example.org/Parent");
//! ontology.add_class(person.clone())?;
//! ontology.add_class(parent.clone())?;
//!
//! // Add subclass relationship
//! let subclass_axiom = SubClassOfAxiom::new(
//!     ClassExpression::Class(parent.clone()),
//!     ClassExpression::Class(person.clone()),
//! );
//! ontology.add_subclass_axiom(subclass_axiom)?;
//!
//! // Create reasoner and perform inference
//! let reasoner = SimpleReasoner::new(ontology);
//! let is_consistent = reasoner.is_consistent()?;
//! let is_subclass = reasoner.is_subclass_of(&parent.iri(), &person.iri())?;
//!
//! println!("Ontology consistent: {}", is_consistent);
//! println!("Parent ⊑ Person: {}", is_subclass);
//! # Ok::<(), owl2_reasoner::OwlError>(())
//! ```
//!
//! ## Architecture
//!
//! The library is organized into several key modules:
//!
//! - [`ontology`] - Ontology management and indexed storage
//! - [`entities`] - OWL2 entities (classes, properties, individuals)
//! - [`axioms`] - Logical statements and relationships
//! - [`reasoning`] - Reasoning algorithms and inference
//! - [`parser`] - Multi-format parsing and serialization
//! - [`iri`] - IRI management with caching
//! - [`cache`] - Configurable caching system with eviction strategies
//! - [`memory`] - Memory leak prevention and monitoring system
//! - [`error`] - Comprehensive error handling
//!
//! ## Performance
//!
//! - **Memory Efficiency**: String interning and Arc-based sharing
//! - **Fast Access**: Indexed axiom storage with O(1) access patterns
//! - **Intelligent Caching**: Multi-layered caching with TTL expiration
//! - **Query Optimization**: Hash join algorithms and pattern reordering
//!
//! ## Examples
//!
//! See the [examples] directory for comprehensive usage patterns including:
//!
//! - Family relationship ontologies
//! - Biomedical knowledge graphs
//! - Performance benchmarking
//! - Complex class expressions
//!
//! [examples]: https://github.com/your-org/owl2-reasoner/tree/main/examples

/// OWL2 Reasoner error types and result handling
pub mod error;

/// IRI management for OWL2 entities with caching and namespace support
pub mod iri;

/// Constants for commonly used IRIs and configuration values
pub mod constants;

/// OWL2 Entities - Classes, Properties, and Individuals with characteristics
pub mod entities;

/// OWL2 Axioms - Logical statements about entities with full OWL2 support
pub mod axioms;

/// Ontology structure and management with indexed storage and performance optimization
pub mod ontology;

/// Storage backends for OWL2 ontologies (for future extensibility)
pub mod storage;

/// OWL2 syntax parsers supporting Turtle, RDF/XML, OWL/XML, and N-Triples
pub mod parser;

/// OWL2 reasoning engine with tableaux algorithm and rule-based inference
pub mod reasoning;

/// OWL2 Profile validation (EL, QL, RL) with comprehensive checking
pub mod profiles;

// /// Empirical validation and benchmarking system for performance claims
// pub mod validation; // Temporarily disabled for compilation

/// GS1 EPCIS ontology implementation for supply chain traceability
pub mod epcis;

/// EPCIS document parser for XML and JSON formats
pub mod epcis_parser;

/// EPCIS test data generator for different scales
pub mod epcis_test_generator;

/// Configurable caching system with eviction strategies
pub mod cache;

/// Web service API for OWL2 reasoning and EPCIS processing
#[cfg(feature = "web-service")]
pub mod web_service;

/// Global cache management with encapsulated synchronization
pub mod cache_manager;

/// Memory leak prevention and monitoring system
pub mod memory;

/// Test memory guard for preventing OOM issues during testing
pub mod test_memory_guard;

/// Test helpers for memory-safe testing patterns
pub mod test_helpers;

/// Performance utilities and optimizations
pub mod utils;

pub mod test_suite_advanced;
/// OWL2 Test Suite integration for W3C compliance validation
pub mod test_suite_simple;

/// Comprehensive test suite with regression tests and performance benchmarks
#[cfg(test)]
pub mod tests;

// Re-export common types for convenience
pub use axioms::*;
pub use cache::*;
pub use constants::*;
pub use entities::*;
pub use epcis::*;
pub use epcis_parser::{EPCISDocumentParser, EPCISDocumentWriter, EPCISParserConfig};
pub use epcis_test_generator::*;
pub use error::{OwlError, OwlResult};
pub use iri::{IRIRef, IRI};
pub use memory::{LeakDetectionReport, MemoryMonitor, MemoryMonitorConfig, MemoryStats};
pub use ontology::*;
// Re-export parser modules with specific naming to avoid conflicts
pub use parser::arena::*;
pub use parser::common as parser_common;
pub use parser::import_resolver::*;
pub use parser::manchester::{ManchesterAST, ManchesterParser};
pub use parser::owl_functional::OwlFunctionalSyntaxParser;
pub use parser::owl_xml::*;
pub use parser::rdf_xml::*;
pub use parser::turtle::*;
pub use parser::{OntologyParser, ParserConfig, ParserFactory};

// Re-export profile modules with specific naming to avoid conflicts
pub use profiles::common::{
    OntologyStats, OptimizationHint, OptimizationType, Owl2Profile, Owl2ProfileValidator,
    ProfileAnalysisReport, ProfileValidationResult, ProfileValidator, ProfileViolation,
    ProfileViolationType, ValidationStatistics, ViolationSeverity,
};
// Re-export profile-specific validators with explicit naming
pub use profiles::el::validator::ElValidator;
pub use profiles::ql::validator::QlValidator;
pub use profiles::rl::*;
// cache types are already available through profiles::common above
pub use reasoning::*;
pub use storage::*;
pub use test_suite_advanced::*;
pub use test_suite_simple::*;

#[cfg(feature = "web-service")]
pub use web_service::start_web_service;

/// Start the web service - dummy function when feature is disabled
#[cfg(not(feature = "web-service"))]
pub async fn start_web_service(_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    Err("Web service feature not enabled. Compile with --features web-service".into())
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Library description
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Repository URL
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Homepage URL
pub const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
