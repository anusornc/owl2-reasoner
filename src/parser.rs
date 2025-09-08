//! OWL2 Syntax Parsers
//! 
//! This module provides parsers for different OWL2 serialization formats including:
//! - RDF/XML
//! - Turtle/TTL
//! - OWL/XML
//! - Manchester Syntax (future)

pub mod turtle;
pub mod rdf_xml;
pub mod owl_xml;
pub mod common;

pub use turtle::*;
pub use rdf_xml::*;
pub use owl_xml::*;
pub use common::*;

// Re-export common parsing utilities
pub use common::{parse_literal, parse_curie, validate_iri, get_local_name, get_namespace};

use crate::ontology::Ontology;
use crate::error::OwlResult;

/// Trait for OWL2 ontology parsers
pub trait OwlParser {
    /// Parse an ontology from the given input
    fn parse(&mut self, input: &str) -> OwlResult<Ontology>;
    
    /// Parse an ontology from a file
    fn parse_file(&mut self, path: &std::path::Path) -> OwlResult<Ontology>;
    
    /// Get the parser format name
    fn format(&self) -> &str;
}

/// Parser configuration options
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Whether to validate the ontology during parsing
    pub validate: bool,
    /// Whether to ignore parse errors and continue
    pub strict: bool,
    /// Maximum file size to parse (in bytes)
    pub max_file_size: Option<u64>,
    /// Custom namespace prefixes
    pub namespaces: Vec<(String, String)>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        ParserConfig {
            validate: true,
            strict: false,
            max_file_size: Some(100 * 1024 * 1024), // 100MB default
            namespaces: vec![
                ("owl".to_string(), "http://www.w3.org/2002/07/owl#".to_string()),
                ("rdf".to_string(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string()),
                ("rdfs".to_string(), "http://www.w3.org/2000/01/rdf-schema#".to_string()),
                ("xsd".to_string(), "http://www.w3.org/2001/XMLSchema#".to_string()),
            ],
        }
    }
}

/// Create a parser for the given format
pub fn create_parser(format: &str) -> Option<Box<dyn OwlParser>> {
    match format.to_lowercase().as_str() {
        "turtle" | "ttl" => Some(Box::new(TurtleParser::new())),
        "rdf/xml" | "rdf" | "owl" => Some(Box::new(RdfXmlParser::new())),
        "owl/xml" => Some(Box::new(OwlXmlParser::new())),
        _ => None,
    }
}

/// Create a parser with custom configuration
pub fn create_parser_with_config(format: &str, config: ParserConfig) -> Option<Box<dyn OwlParser>> {
    match format.to_lowercase().as_str() {
        "turtle" | "ttl" => Some(Box::new(TurtleParser::with_config(config))),
        "rdf/xml" | "rdf" | "owl" => Some(Box::new(RdfXmlParser::with_config(config))),
        "owl/xml" => Some(Box::new(OwlXmlParser::with_config(config))),
        _ => None,
    }
}

/// Auto-detect format from file extension or content
pub fn detect_format(path: &std::path::Path, content: Option<&str>) -> Option<&'static str> {
    // First try file extension
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        match ext.to_lowercase().as_str() {
            "ttl" | "turtle" => return Some("turtle"),
            "rdf" | "xml" | "owl" => return Some("rdf/xml"),
            "owx" => return Some("owl/xml"),
            _ => {}
        }
    }
    
    // Then try content detection
    if let Some(content) = content {
        if content.contains("@prefix") || content.contains("PREFIX") {
            return Some("turtle");
        }
        if content.contains("<rdf:RDF") || content.contains("<owl:Ontology") {
            return Some("rdf/xml");
        }
        if content.contains("<Ontology") && content.contains("xmlns=") {
            return Some("owl/xml");
        }
    }
    
    None
}