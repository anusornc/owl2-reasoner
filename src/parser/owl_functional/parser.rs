//! Main OWL Functional Syntax parser
//!
//! This module provides the main parser implementation that coordinates
//! tokenization, grammar parsing, and validation to produce OWL ontologies.

use crate::axioms::*;
use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::parser::owl_functional::grammar::GrammarParser;
use crate::parser::owl_functional::syntax::{EntityDeclaration, FunctionalSyntaxAST};
use crate::parser::owl_functional::tokenizer::Tokenizer;
use crate::parser::owl_functional::validator::FunctionalSyntaxValidator;
use crate::parser::{OntologyParser, ParserArenaBuilder, ParserArenaTrait, ParserConfig};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// OWL Functional Syntax parser
pub struct OwlFunctionalSyntaxParser {
    /// Parser configuration
    config: ParserConfig,
    /// Prefix mappings
    prefixes: HashMap<String, String>,
    /// Arena allocator for efficient string and object allocation
    #[allow(dead_code)]
    arena: Option<Box<dyn ParserArenaTrait>>,
    /// Semantic validator
    validator: FunctionalSyntaxValidator,
}

impl OwlFunctionalSyntaxParser {
    /// Create a new OWL Functional Syntax parser with default configuration
    pub fn new() -> Self {
        Self::with_config(ParserConfig::default())
    }

    /// Create a new OWL Functional Syntax parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        let mut prefixes = HashMap::new();
        for (prefix, namespace) in &config.prefixes {
            prefixes.insert(prefix.clone(), namespace.clone());
        }

        // Add default OWL2 prefixes
        prefixes.insert(
            "owl".to_string(),
            "http://www.w3.org/2002/07/owl#".to_string(),
        );
        prefixes.insert(
            "rdf".to_string(),
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
        );
        prefixes.insert(
            "rdfs".to_string(),
            "http://www.w3.org/2000/01/rdf-schema#".to_string(),
        );
        prefixes.insert(
            "xsd".to_string(),
            "http://www.w3.org/2001/XMLSchema#".to_string(),
        );

        // Initialize arena allocator if enabled
        let arena = if config.use_arena_allocation {
            Some(
                ParserArenaBuilder::new()
                    .with_capacity(config.arena_capacity)
                    .build(),
            )
        } else {
            None
        };

        // Initialize validator
        let validator = FunctionalSyntaxValidator::with_strict_mode(config.strict_validation);

        OwlFunctionalSyntaxParser {
            config,
            prefixes,
            arena,
            validator,
        }
    }

    /// Parse OWL Functional Syntax content and build an ontology
    fn parse_content(&mut self, content: &str) -> OwlResult<Ontology> {
        if self.config.strict_validation && content.trim().is_empty() {
            return Err(OwlError::ValidationError(
                "Ontology contains no content".to_string(),
            ));
        }

        // Tokenize the input
        let tokenizer = Tokenizer::new(content);
        let tokens = tokenizer
            .tokenize()
            .map_err(|e| OwlError::ParseError(e.to_string()))?;

        // Parse grammar
        let mut grammar_parser = GrammarParser::new(tokens);
        let ast = grammar_parser
            .parse_document()
            .map_err(|e| OwlError::ParseError(e.to_string()))?;

        // Validate semantic structure
        self.validator
            .validate_document(&ast)
            .map_err(|e| OwlError::ValidationError(e.to_string()))?;

        // Convert AST to ontology
        let ontology = self.ast_to_ontology(&ast)?;

        // Prefixes are handled internally by the parser for IRI resolution

        // Final validation
        if self.config.strict_validation {
            self.validator.validate_ontology(&ontology)?;
        }

        Ok(ontology)
    }

    /// Convert an AST to an ontology
    fn ast_to_ontology(&self, ast: &FunctionalSyntaxAST) -> OwlResult<Ontology> {
        let mut ontology = Ontology::new();

        // Set ontology IRI if present
        let FunctionalSyntaxAST::OntologyDocument { ontology_iri, .. } = ast;
        if let Some(iri_str) = ontology_iri {
            let iri = IRI::new_optimized(iri_str).map_err(|e| {
                OwlError::ParseError(format!("Invalid ontology IRI '{}': {}", iri_str, e))
            })?;
            ontology.set_iri((*iri).clone());
        }

        // Process all content
        for content in ast.content() {
            self.process_ontology_content(content, &mut ontology)?;
        }

        Ok(ontology)
    }

    /// Process a single piece of ontology content
    fn process_ontology_content(
        &self,
        content: &crate::parser::owl_functional::syntax::OntologyContent,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        use crate::parser::owl_functional::syntax::*;

        match content {
            OntologyContent::Declaration(declaration) => {
                self.process_declaration(declaration, ontology)?;
            }
            OntologyContent::Axiom(axiom) => {
                ontology.add_axiom(axiom.clone())?;
            }
            OntologyContent::Import(import) => {
                let import_axiom = ImportAxiom::new(Arc::new(import.import_iri.clone()));
                ontology.add_axiom(Axiom::Import(import_axiom))?;
            }
        }

        Ok(())
    }

    /// Process an entity declaration
    fn process_declaration(
        &self,
        declaration: &EntityDeclaration,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        match declaration {
            EntityDeclaration::Class(class) => {
                ontology.add_class(class.clone())?;
            }
            EntityDeclaration::ObjectProperty(prop) => {
                ontology.add_object_property(prop.clone())?;
            }
            EntityDeclaration::DataProperty(prop) => {
                ontology.add_data_property(prop.clone())?;
            }
            EntityDeclaration::NamedIndividual(individual) => {
                ontology.add_named_individual(individual.clone())?;
            }
            EntityDeclaration::AnonymousIndividual(individual) => {
                ontology.add_anonymous_individual(individual.clone())?;
            }
            EntityDeclaration::AnnotationProperty(prop) => {
                ontology.add_annotation_property(prop.clone())?;
            }
        }

        Ok(())
    }

    /// Add a prefix mapping
    pub fn add_prefix(&mut self, prefix: String, namespace: String) {
        self.prefixes.insert(prefix, namespace);
    }

    /// Get all prefix mappings
    pub fn prefixes(&self) -> &HashMap<String, String> {
        &self.prefixes
    }

    /// Get the parser configuration
    pub fn config(&self) -> &ParserConfig {
        &self.config
    }

    /// Set the strict validation mode
    pub fn set_strict_validation(&mut self, strict: bool) {
        self.validator = FunctionalSyntaxValidator::with_strict_mode(strict);
        self.config.strict_validation = strict;
    }

    /// Validate an existing ontology with this parser's settings
    pub fn validate_ontology(&self, ontology: &Ontology) -> OwlResult<()> {
        self.validator.validate_ontology(ontology)?;
        Ok(())
    }
}

impl OntologyParser for OwlFunctionalSyntaxParser {
    fn parse_str(&self, content: &str) -> OwlResult<Ontology> {
        // Create a mutable copy for parsing
        let mut parser_copy = OwlFunctionalSyntaxParser::with_config(self.config.clone());
        parser_copy.prefixes = self.prefixes.clone();
        parser_copy.parse_content(content)
    }

    fn parse_file(&self, path: &Path) -> OwlResult<Ontology> {
        use std::fs;
        use std::io::Read;

        // Check file size
        if self.config.max_file_size > 0 {
            let metadata = fs::metadata(path)?;
            if metadata.len() > self.config.max_file_size as u64 {
                return Err(OwlError::ParseError(format!(
                    "File size exceeds maximum allowed size: {} bytes",
                    self.config.max_file_size
                )));
            }
        }

        let mut file = fs::File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        self.parse_str(&content)
    }

    fn format_name(&self) -> &'static str {
        "OWL Functional Syntax"
    }
}

impl Default for OwlFunctionalSyntaxParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_initialization() {
        let parser = OwlFunctionalSyntaxParser::new();
        assert_eq!(parser.format_name(), "OWL Functional Syntax");
        assert!(!parser.prefixes().is_empty());
        assert!(parser.prefixes().contains_key("owl"));
        assert!(parser.prefixes().contains_key("rdf"));
        assert!(parser.prefixes().contains_key("rdfs"));
        assert!(parser.prefixes().contains_key("xsd"));
    }

    #[test]
    fn test_parser_with_config() {
        let config = ParserConfig {
            max_file_size: 1000,
            strict_validation: false,
            resolve_base_iri: false,
            prefixes: HashMap::new(),
            use_arena_allocation: false,
            arena_capacity: 0,
            max_arena_size: 0,
            resolve_imports: false,
            ignore_import_errors: true,
        };

        let parser = OwlFunctionalSyntaxParser::with_config(config);
        assert_eq!(parser.format_name(), "OWL Functional Syntax");
        assert!(!parser.config().strict_validation);
    }

    #[test]
    fn test_add_prefix() {
        let mut parser = OwlFunctionalSyntaxParser::new();
        parser.add_prefix("test".to_string(), "http://example.org/test#".to_string());

        assert!(parser.prefixes().contains_key("test"));
        assert_eq!(
            parser.prefixes().get("test"),
            Some(&"http://example.org/test#".to_string())
        );
    }

    #[test]
    fn test_set_strict_validation() {
        let mut parser = OwlFunctionalSyntaxParser::new();
        parser.set_strict_validation(false);

        assert!(!parser.config().strict_validation);
    }

    #[test]
    fn test_empty_content_strict() {
        let mut parser = OwlFunctionalSyntaxParser::new();
        parser.set_strict_validation(true);

        let result = parser.parse_content("");
        assert!(result.is_err());

        match result.unwrap_err() {
            OwlError::ValidationError(msg) => {
                assert!(msg.contains("no content"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_empty_content_non_strict() {
        let mut parser = OwlFunctionalSyntaxParser::new();
        parser.set_strict_validation(false);

        let result = parser.parse_content("");
        assert!(result.is_ok());

        let ontology = result.unwrap();
        assert!(ontology.classes().is_empty());
        assert!(ontology.axioms().is_empty());
    }

    #[test]
    fn test_simple_ontology_parsing() {
        let simple_owl = r#"
Prefix(:=<http://example.org/test#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)

Ontology(<http://example.org/test>

Declaration(Class(:Person))
Declaration(Class(:Student))

SubClassOf(:Student :Person)
)
"#;

        let parser = OwlFunctionalSyntaxParser::new();
        let result = parser.parse_str(simple_owl);

        assert!(result.is_ok(), "Parsing failed: {:?}", result);

        if let Ok(ontology) = result {
            assert!(
                ontology.classes().len() >= 2,
                "Should have parsed at least 2 classes"
            );
            assert!(!ontology.axioms().is_empty(), "Should have parsed axioms");
        }
    }
}
