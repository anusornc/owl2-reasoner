//! Error types for the OWL2 reasoner

use thiserror::Error;

/// OWL2 Reasoner error type
#[derive(Error, Debug)]
pub enum OwlError {
    /// IRI-related errors
    #[error("Invalid IRI: {0}")]
    InvalidIRI(String),

    /// Unknown namespace prefix
    #[error("Unknown prefix: {0}")]
    UnknownPrefix(String),

    /// Parse errors
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Reasoning errors
    #[error("Reasoning error: {0}")]
    ReasoningError(String),

    /// Query errors
    #[error("Query error: {0}")]
    QueryError(String),

    /// Storage errors
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// OWL2 specification violations
    #[error("OWL2 specification violation: {0}")]
    OwlViolation(String),

    /// Inconsistent ontology
    #[error("Inconsistent ontology: {0}")]
    InconsistentOntology(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for OWL2 operations
pub type OwlResult<T> = Result<T, OwlError>;
