//! Parser module for OWL2 ontology formats
//! 
//! Provides parsers for various RDF/OWL serialization formats including:
//! - Turtle (TTL)
//! - RDF/XML 
//! - OWL/XML
//! - N-Triples

pub mod common;
pub mod turtle;
pub mod rdf_xml;
pub mod owl_xml;

pub use common::*;
pub use turtle::*;
pub use rdf_xml::*;
pub use owl_xml::*;

use crate::ontology::Ontology;
use crate::error::OwlResult;

/// Parser trait for different serialization formats
pub trait OntologyParser {
    /// Parse an ontology from a string
    fn parse_str(&self, content: &str) -> OwlResult<Ontology>;
    
    /// Parse an ontology from a file
    fn parse_file(&self, path: &std::path::Path) -> OwlResult<Ontology>;
    
    /// Get the supported format name
    fn format_name(&self) -> &'static str;
}

/// Factory for creating parsers based on file extension or content type
pub struct ParserFactory;

impl ParserFactory {
    /// Create a parser based on file extension
    pub fn for_file_extension(ext: &str) -> Option<Box<dyn OntologyParser>> {
        match ext.to_lowercase().as_str() {
            "ttl" | "turtle" => Some(Box::new(TurtleParser::new())),
            "rdf" | "rdfs" | "owl" => Some(Box::new(RdfXmlParser::new())),
            "owx" | "xml" => Some(Box::new(OwlXmlParser::new())),
            "nt" => Some(Box::new(NtriplesParser::new())),
            _ => None,
        }
    }
    
    /// Create a parser based on content type
    pub fn for_content_type(content_type: &str) -> Option<Box<dyn OntologyParser>> {
        match content_type {
            "text/turtle" | "application/x-turtle" => Some(Box::new(TurtleParser::new())),
            "application/rdf+xml" => Some(Box::new(RdfXmlParser::new())),
            "application/owl+xml" => Some(Box::new(OwlXmlParser::new())),
            "application/n-triples" | "text/plain" => Some(Box::new(NtriplesParser::new())),
            _ => None,
        }
    }
    
    /// Auto-detect format and create appropriate parser
    pub fn auto_detect(content: &str) -> Option<Box<dyn OntologyParser>> {
        if content.trim().starts_with("@prefix") || content.trim().starts_with("PREFIX") {
            Some(Box::new(TurtleParser::new()))
        } else if content.trim().starts_with("<rdf:RDF") || content.contains("<rdf:Description") {
            Some(Box::new(RdfXmlParser::new()))
        } else if content.trim().starts_with("<?xml") && content.contains("Ontology") {
            Some(Box::new(OwlXmlParser::new()))
        } else if content.lines().next().map_or(false, |line| line.contains("> <") && line.contains(" .")) {
            Some(Box::new(NtriplesParser::new()))
        } else {
            None
        }
    }
}

/// Simple N-Triples parser (placeholder implementation)
pub struct NtriplesParser {
    config: ParserConfig,
}

impl NtriplesParser {
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }
    
    pub fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }
}

impl OntologyParser for NtriplesParser {
    fn parse_str(&self, content: &str) -> OwlResult<Ontology> {
        // Placeholder implementation
        let ontology = Ontology::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Basic N-Triples parsing
            if let Ok(_triple) = self.parse_ntriples_line(line) {
                // Add to ontology (simplified)
                // In a real implementation, this would properly parse and add axioms
            }
        }
        
        Ok(ontology)
    }
    
    fn parse_file(&self, path: &std::path::Path) -> OwlResult<Ontology> {
        use std::fs::File;
        use std::io::Read;
        
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        self.parse_str(&content)
    }
    
    fn format_name(&self) -> &'static str {
        "N-Triples"
    }
}

impl NtriplesParser {
    fn parse_ntriples_line(&self, line: &str) -> OwlResult<(String, String, String)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(crate::error::OwlError::ParseError("Invalid N-Triples line".to_string()));
        }
        
        let subject = parts[0].trim_matches('<').trim_matches('>');
        let predicate = parts[1].trim_matches('<').trim_matches('>');
        let object = parts[2].trim_matches('<').trim_matches('>').trim_matches('"');
        
        Ok((subject.to_string(), predicate.to_string(), object.to_string()))
    }
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum file size to parse (in bytes)
    pub max_file_size: usize,
    /// Whether to validate strict syntax
    pub strict_validation: bool,
    /// Whether to resolve base IRIs
    pub resolve_base_iri: bool,
    /// Custom prefix mappings
    pub prefixes: std::collections::HashMap<String, String>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            strict_validation: true,
            resolve_base_iri: false,
            prefixes: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_factory_file_extension() {
        assert!(ParserFactory::for_file_extension("ttl").is_some());
        assert!(ParserFactory::for_file_extension("turtle").is_some());
        assert!(ParserFactory::for_file_extension("rdf").is_some());
        assert!(ParserFactory::for_file_extension("owl").is_some());
        assert!(ParserFactory::for_file_extension("owx").is_some());
        assert!(ParserFactory::for_file_extension("nt").is_some());
        assert!(ParserFactory::for_file_extension("unknown").is_none());
    }
    
    #[test]
    fn test_parser_factory_content_type() {
        assert!(ParserFactory::for_content_type("text/turtle").is_some());
        assert!(ParserFactory::for_content_type("application/rdf+xml").is_some());
        assert!(ParserFactory::for_content_type("application/owl+xml").is_some());
        assert!(ParserFactory::for_content_type("application/n-triples").is_some());
        assert!(ParserFactory::for_content_type("unknown/type").is_none());
    }
    
    #[test]
    fn test_auto_detect_turtle() {
        let turtle_content = r#"
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
        "#;
        
        let parser = ParserFactory::auto_detect(turtle_content);
        assert!(parser.is_some());
        assert_eq!(parser.unwrap().format_name(), "Turtle");
    }
    
    #[test]
    fn test_auto_detect_rdf_xml() {
        let rdf_content = r#"
            <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
                <rdf:Description rdf:about="http://example.org/test"/>
            </rdf:RDF>
        "#;
        
        let parser = ParserFactory::auto_detect(rdf_content);
        assert!(parser.is_some());
        assert_eq!(parser.unwrap().format_name(), "RDF/XML");
    }
    
    #[test]
    fn test_ntriples_parser() {
        let parser = NtriplesParser::new();
        let ntriples_content = r#"
            <http://example.org/subject> <http://example.org/predicate> <http://example.org/object> .
        "#;
        
        let result = parser.parse_str(ntriples_content);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_parser_config() {
        let config = ParserConfig::default();
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
        assert!(config.strict_validation);
        assert!(!config.resolve_base_iri);
        assert!(config.prefixes.is_empty());
    }
}