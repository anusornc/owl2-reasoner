//! Parser module for OWL2 ontology formats
//!
//! Provides parsers for various RDF/OWL serialization formats including:
//! - Turtle (TTL)
//! - RDF/XML
//! - OWL/XML
//! - N-Triples

pub mod arena;
pub mod common;
pub mod owl_functional;
pub mod owl_xml;
pub mod rdf_xml;
pub mod turtle;

pub use arena::*;
pub use common::*;
pub use owl_functional::*;
pub use owl_xml::*;
pub use rdf_xml::*;
pub use turtle::*;

use crate::entities::Class;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;

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
            "rdf" | "rdfs" => Some(Box::new(RdfXmlParser::new())),
            "owl" | "ofn" => Some(Box::new(OwlFunctionalSyntaxParser::new())), // OWL Functional Syntax files
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
        let content_trimmed = content.trim();

        // Check for OWL Functional Syntax (highest priority for .owl files)
        if content_trimmed.starts_with("Prefix(")
            || content_trimmed.contains("Ontology(")
            || (content_trimmed.starts_with("Document(") && content_trimmed.contains("Prefix("))
        {
            Some(Box::new(OwlFunctionalSyntaxParser::new()))
        } else if content_trimmed.starts_with("@prefix") || content_trimmed.starts_with("PREFIX") {
            Some(Box::new(TurtleParser::new()))
        } else if content_trimmed.starts_with("<rdf:RDF") || content.contains("<rdf:Description") {
            Some(Box::new(RdfXmlParser::new()))
        } else if content_trimmed.starts_with("<?xml") && content.contains("Ontology") {
            Some(Box::new(OwlXmlParser::new()))
        } else if content
            .lines()
            .next()
            .is_some_and(|line| line.contains("> <") && line.contains(" ."))
        {
            Some(Box::new(NtriplesParser::new()))
        } else {
            None
        }
    }
}

/// N-Triples parser implementing W3C N-Triples specification
pub struct NtriplesParser {
    #[allow(dead_code)]
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
        let mut ontology = Ontology::new();
        let mut line_num = 0;

        for line in content.lines() {
            line_num += 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            match self.parse_ntriples_line(line) {
                Ok(triple) => {
                    if let Err(e) = self.add_triple_to_ontology(&mut ontology, &triple) {
                        return Err(crate::error::OwlError::ParseError(format!(
                            "Error at line {}: {}",
                            line_num, e
                        )));
                    }
                }
                Err(e) => {
                    return Err(crate::error::OwlError::ParseError(format!(
                        "Parse error at line {}: {}",
                        line_num, e
                    )));
                }
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
    fn parse_ntriples_line(&self, line: &str) -> OwlResult<NtriplesTriple> {
        let mut chars = line.char_indices();

        // Parse subject
        let subject = self.parse_ntriples_term(&mut chars)?;

        // Skip whitespace
        self.skip_whitespace(&mut chars);

        // Parse predicate
        let predicate = self.parse_ntriples_term(&mut chars)?;

        // Skip whitespace
        self.skip_whitespace(&mut chars);

        // Parse object
        let object = self.parse_ntriples_term(&mut chars)?;

        // Skip whitespace
        self.skip_whitespace(&mut chars);

        // Expect trailing '.'
        if let Some((_, c)) = chars.next() {
            if c != '.' {
                return Err(crate::error::OwlError::ParseError(
                    "Expected '.' at end of triple".to_string(),
                ));
            }
        }

        Ok(NtriplesTriple {
            subject,
            predicate,
            object,
        })
    }

    fn parse_ntriples_term(
        &self,
        chars: &mut std::str::CharIndices<'_>,
    ) -> OwlResult<NtriplesTerm> {
        self.skip_whitespace(chars);

        if let Some((_, c)) = chars.next() {
            match c {
                '<' => {
                    // IRI
                    let mut iri_str = String::new();
                    while let Some((_, next_c)) = chars.next() {
                        if next_c == '>' {
                            break;
                        }
                        iri_str.push(next_c);
                    }

                    if iri_str.is_empty() {
                        return Err(crate::error::OwlError::ParseError("Empty IRI".to_string()));
                    }

                    // Validate and create IRI
                    let iri = IRI::new(&iri_str).map_err(|e| {
                        crate::error::OwlError::ParseError(format!(
                            "Invalid IRI '{}': {}",
                            iri_str, e
                        ))
                    })?;

                    Ok(NtriplesTerm::IRI(iri))
                }
                '"' => {
                    // Literal
                    let mut literal_str = String::new();
                    let mut lang_tag = None;
                    let mut datatype = None;

                    // Parse literal content
                    while let Some((_, next_c)) = chars.next() {
                        if next_c == '"' {
                            break;
                        }
                        if next_c == '\\' {
                            if let Some((_, esc_c)) = chars.next() {
                                match esc_c {
                                    't' => literal_str.push('\t'),
                                    'b' => literal_str.push('\x08'),
                                    'n' => literal_str.push('\n'),
                                    'r' => literal_str.push('\r'),
                                    'f' => literal_str.push('\x0c'),
                                    '"' => literal_str.push('"'),
                                    '\'' => literal_str.push('\''),
                                    '\\' => literal_str.push('\\'),
                                    'u' => {
                                        // Unicode escape \uXXXX
                                        let mut hex = String::new();
                                        for _ in 0..4 {
                                            if let Some((_, h)) = chars.next() {
                                                hex.push(h);
                                            }
                                        }
                                        if let Ok(code) = u16::from_str_radix(&hex, 16) {
                                            literal_str
                                                .push(char::from_u32(code as u32).unwrap_or('?'));
                                        }
                                    }
                                    'U' => {
                                        // Unicode escape \UXXXXXXXX
                                        let mut hex = String::new();
                                        for _ in 0..8 {
                                            if let Some((_, h)) = chars.next() {
                                                hex.push(h);
                                            }
                                        }
                                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                            literal_str.push(char::from_u32(code).unwrap_or('?'));
                                        }
                                    }
                                    _ => literal_str.push(esc_c),
                                }
                            }
                        } else {
                            literal_str.push(next_c);
                        }
                    }

                    // Check for language tag or datatype
                    self.skip_whitespace(chars);
                    if let Some((_, next_c)) = chars.clone().next() {
                        if next_c == '@' {
                            // Language tag
                            chars.next(); // consume '@'
                            let mut tag = String::new();
                            while let Some((_, c)) = chars.clone().next() {
                                if c.is_alphanumeric() || c == '-' {
                                    tag.push(c);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            if !tag.is_empty() {
                                lang_tag = Some(tag);
                            }
                        } else if next_c == '^' {
                            // Datatype
                            chars.next(); // consume '^'
                            if let Some((_, c)) = chars.next() {
                                if c == '^' {
                                    chars.next(); // consume second '^'
                                    if let Some((_, c2)) = chars.next() {
                                        if c2 == '<' {
                                            let mut dt_iri = String::new();
                                            while let Some((_, dt_c)) = chars.next() {
                                                if dt_c == '>' {
                                                    break;
                                                }
                                                dt_iri.push(dt_c);
                                            }
                                            if !dt_iri.is_empty() {
                                                datatype =
                                                    Some(IRI::new(&dt_iri).map_err(|e| {
                                                        crate::error::OwlError::ParseError(format!(
                                                            "Invalid datatype IRI '{}': {}",
                                                            dt_iri, e
                                                        ))
                                                    })?);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Ok(NtriplesTerm::Literal {
                        value: literal_str,
                        language: lang_tag,
                        datatype,
                    })
                }
                '_' => {
                    // Blank node
                    if let Some((_, c)) = chars.next() {
                        if c == ':' {
                            let mut bnode_id = String::new();
                            while let Some((_, next_c)) = chars.clone().next() {
                                if next_c.is_alphanumeric() || next_c == '-' || next_c == '_' {
                                    bnode_id.push(next_c);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            if bnode_id.is_empty() {
                                return Err(crate::error::OwlError::ParseError(
                                    "Empty blank node ID".to_string(),
                                ));
                            }
                            Ok(NtriplesTerm::BlankNode(bnode_id))
                        } else {
                            return Err(crate::error::OwlError::ParseError(
                                "Expected ':' after '_' for blank node".to_string(),
                            ));
                        }
                    } else {
                        return Err(crate::error::OwlError::ParseError(
                            "Incomplete blank node".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(crate::error::OwlError::ParseError(format!(
                        "Unexpected character '{}' at start of term",
                        c
                    )));
                }
            }
        } else {
            return Err(crate::error::OwlError::ParseError(
                "Unexpected end of input while parsing term".to_string(),
            ));
        }
    }

    fn skip_whitespace(&self, chars: &mut std::str::CharIndices<'_>) {
        while let Some((_, c)) = chars.clone().next() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
    }

    fn add_triple_to_ontology(
        &self,
        ontology: &mut Ontology,
        triple: &NtriplesTriple,
    ) -> OwlResult<()> {
        use crate::parser::common::*;

        // Convert N-Triples triple to OWL axioms based on common patterns
        match (&triple.subject, &triple.predicate, &triple.object) {
            (
                NtriplesTerm::IRI(subject_iri),
                NtriplesTerm::IRI(predicate_iri),
                NtriplesTerm::IRI(object_iri),
            ) => {
                // Handle common RDF/RDFS/OWL patterns
                if predicate_iri.as_str() == RDF_TYPE {
                    // Class assertion: subject rdf:type object
                    let subject_class = Class::new(subject_iri.clone());
                    let object_class = Class::new(object_iri.clone());

                    ontology.add_class(subject_class.clone())?;
                    ontology.add_class(object_class.clone())?;

                    let class_assertion = crate::axioms::ClassAssertionAxiom::new(
                        subject_iri.clone(),
                        crate::axioms::ClassExpression::Class(subject_class),
                    );
                    ontology.add_class_assertion(class_assertion)?;
                } else if predicate_iri.as_str() == RDFS_SUBCLASSOF {
                    // Subclass axiom: subject rdfs:subClassOf object
                    let subject_class = Class::new(subject_iri.clone());
                    let object_class = Class::new(object_iri.clone());

                    ontology.add_class(subject_class.clone())?;
                    ontology.add_class(object_class.clone())?;

                    let subclass_axiom = crate::axioms::SubClassOfAxiom::new(
                        crate::axioms::ClassExpression::Class(subject_class),
                        crate::axioms::ClassExpression::Class(object_class),
                    );
                    ontology.add_subclass_axiom(subclass_axiom)?;
                } else {
                    // Generic property assertion
                    let subject_individual =
                        crate::entities::NamedIndividual::new(subject_iri.clone());
                    ontology.add_named_individual(subject_individual)?;

                    // Create object property if it looks like one
                    if predicate_iri.as_str().starts_with("http://")
                        && !predicate_iri.as_str().contains("#")
                    {
                        let obj_prop = crate::entities::ObjectProperty::new(predicate_iri.clone());
                        ontology.add_object_property(obj_prop)?;
                    }
                }
            }
            (
                NtriplesTerm::IRI(subject_iri),
                NtriplesTerm::IRI(predicate_iri),
                NtriplesTerm::Literal {
                    value,
                    language: _,
                    datatype: _,
                },
            ) => {
                // Literal property assertion
                let subject_individual = crate::entities::NamedIndividual::new(subject_iri.clone());
                ontology.add_named_individual(subject_individual)?;

                // Could add data property assertion here in the future
                // For now, we'll just note that we've seen this pattern
                log::debug!(
                    "Skipping literal property assertion: {} {} {}",
                    subject_iri,
                    predicate_iri,
                    value
                );
            }
            _ => {
                // Other patterns (blank nodes, etc.) not yet implemented
                log::debug!(
                    "Skipping triple with unsupported pattern: {:?} {:?} {:?}",
                    triple.subject,
                    triple.predicate,
                    triple.object
                );
            }
        }

        Ok(())
    }
}

/// N-Triples term types
#[derive(Debug, Clone, PartialEq)]
enum NtriplesTerm {
    IRI(IRI),
    Literal {
        value: String,
        language: Option<String>,
        datatype: Option<IRI>,
    },
    BlankNode(String),
}

/// N-Triples triple
#[derive(Debug, Clone, PartialEq)]
struct NtriplesTriple {
    subject: NtriplesTerm,
    predicate: NtriplesTerm,
    object: NtriplesTerm,
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
    /// Whether to use arena allocation for parsing
    pub use_arena_allocation: bool,
    /// Initial arena capacity in bytes (if arena allocation is enabled)
    pub arena_capacity: usize,
    /// Maximum arena size in bytes (if arena allocation is enabled)
    pub max_arena_size: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            // Default to strict validation to satisfy tests and ensure predictable parsing
            strict_validation: true,
            resolve_base_iri: false,
            prefixes: std::collections::HashMap::new(),
            // Enable arena allocation by default for better performance
            use_arena_allocation: true,
            // Start with 1MB arena capacity
            arena_capacity: 1024 * 1024,
            // Maximum arena size of 10MB
            max_arena_size: 10 * 1024 * 1024,
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
