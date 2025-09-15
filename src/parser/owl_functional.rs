//! OWL Functional Syntax parser for OWL2 ontologies
//!
//! Implements parsing of the OWL2 Functional Syntax serialization format.

use crate::parser::{ParserConfig, OntologyParser};
use crate::ontology::Ontology;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::entities::*;
use crate::axioms::*;
use std::collections::HashMap;
use std::path::Path;

/// OWL Functional Syntax parser
pub struct OwlFunctionalSyntaxParser {
    config: ParserConfig,
    prefixes: HashMap<String, String>,
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
        prefixes.insert("owl".to_string(), "http://www.w3.org/2002/07/owl#".to_string());
        prefixes.insert("rdf".to_string(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string());
        prefixes.insert("rdfs".to_string(), "http://www.w3.org/2000/01/rdf-schema#".to_string());
        prefixes.insert("xsd".to_string(), "http://www.w3.org/2001/XMLSchema#".to_string());

        OwlFunctionalSyntaxParser { config, prefixes }
    }

    /// Parse OWL Functional Syntax content and build an ontology
    fn parse_content(&mut self, content: &str) -> OwlResult<Ontology> {
        let mut ontology = Ontology::new();

        // Parse prefixes first
        self.parse_prefixes(content, &mut ontology)?;

        // Parse ontology declaration
        if let Some(ontology_iri) = self.parse_ontology_declaration(content) {
            ontology.set_iri(IRI::new(&ontology_iri)?);
        }

        // Parse declarations
        self.parse_declarations(content, &mut ontology)?;

        // Parse axioms
        self.parse_axioms(content, &mut ontology)?;

        if self.config.strict_validation {
            self.validate_ontology(&ontology)?;
        }

        Ok(ontology)
    }

    /// Parse prefix declarations
    fn parse_prefixes(&mut self, content: &str, _ontology: &mut Ontology) -> OwlResult<()> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Prefix(") && line.ends_with(")") {
                // Extract prefix and namespace
                let prefix_content = &line[7..line.len()-1];
                if let Some((prefix_part, namespace_part)) = prefix_content.split_once('=') {
                    let prefix = prefix_part.trim().trim_matches('<').trim_matches('>');
                    let namespace = namespace_part.trim().trim_matches('<').trim_matches('>');
                    self.prefixes.insert(prefix.to_string(), namespace.to_string());
                }
            }
        }
        Ok(())
    }

    /// Parse ontology declaration
    fn parse_ontology_declaration(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Ontology(") && line.ends_with(")") {
                let ontology_content = &line[9..line.len()-1];
                let iri = ontology_content.trim().trim_matches('<').trim_matches('>');
                return Some(iri.to_string());
            }
        }
        None
    }

    /// Parse entity declarations
    fn parse_declarations(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Declaration(") && line.ends_with(")") {
                let declaration_content = &line[12..line.len()-1];
                self.parse_declaration(declaration_content, ontology)?;
            }
        }
        Ok(())
    }

    /// Parse individual declaration
    fn parse_declaration(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let content = content.trim();

        if content.starts_with("Class(") {
            let iri_str = &content[6..content.len()-1];
            let iri = self.resolve_iri(iri_str)?;
            let class = Class::new(iri);
            ontology.add_class(class)?;
        } else if content.starts_with("ObjectProperty(") {
            let iri_str = &content[15..content.len()-1];
            let iri = self.resolve_iri(iri_str)?;
            let prop = ObjectProperty::new(iri);
            ontology.add_object_property(prop)?;
        } else if content.starts_with("DataProperty(") {
            let iri_str = &content[13..content.len()-1];
            let iri = self.resolve_iri(iri_str)?;
            let prop = DataProperty::new(iri);
            ontology.add_data_property(prop)?;
        } else if content.starts_with("NamedIndividual(") {
            let iri_str = &content[16..content.len()-1];
            let iri = self.resolve_iri(iri_str)?;
            let individual = NamedIndividual::new(iri);
            ontology.add_named_individual(individual)?;
        }

        Ok(())
    }

    /// Parse axioms
    fn parse_axioms(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("SubClassOf(") && line.ends_with(")") {
                let axiom_content = &line[11..line.len()-1];
                self.parse_subclass_of(axiom_content, ontology)?;
            } else if line.starts_with("EquivalentClasses(") && line.ends_with(")") {
                // TODO: Implement EquivalentClasses
            } else if line.starts_with("DisjointClasses(") && line.ends_with(")") {
                // TODO: Implement DisjointClasses
            } else if line.starts_with("ClassAssertion(") && line.ends_with(")") {
                let axiom_content = &line[16..line.len()-1];
                self.parse_class_assertion(axiom_content, ontology)?;
            }
        }
        Ok(())
    }

    /// Parse SubClassOf axiom
    fn parse_subclass_of(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let sub_class_iri = self.resolve_iri(parts[0])?;
            let super_class_iri = self.resolve_iri(parts[1])?;

            let sub_class = Class::new(sub_class_iri);
            let super_class = Class::new(super_class_iri);

            let subclass_axiom = SubClassOfAxiom::new(
                crate::axioms::class_expressions::ClassExpression::Class(sub_class),
                crate::axioms::class_expressions::ClassExpression::Class(super_class),
            );
            ontology.add_subclass_axiom(subclass_axiom)?;
        }
        Ok(())
    }

    /// Parse ClassAssertion axiom
    fn parse_class_assertion(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let class_iri = self.resolve_iri(parts[0])?;
            let individual_iri = self.resolve_iri(parts[1])?;

            let class = Class::new(class_iri);
            let individual = NamedIndividual::new(individual_iri);

            // Add class assertion
            let class_assertion = ClassAssertionAxiom::new(
                individual.iri().clone(),
                crate::axioms::class_expressions::ClassExpression::Class(class),
            );
            ontology.add_class_assertion(class_assertion)?;
        }
        Ok(())
    }

    /// Resolve IRI from string, handling prefixed names and full IRIs
    fn resolve_iri(&self, iri_str: &str) -> OwlResult<IRI> {
        let iri_str = iri_str.trim();

        if iri_str.starts_with('<') && iri_str.ends_with('>') {
            // Full IRI
            let iri_content = &iri_str[1..iri_str.len()-1];
            IRI::new(iri_content)
        } else if iri_str.contains(':') {
            // Prefixed name
            let mut parts = iri_str.splitn(2, ':');
            let first_part = parts.next().unwrap_or("");
            let local_name = parts.next().unwrap_or("");

            // Handle the case where the prefix is empty (e.g., ":Person")
            let prefix = if first_part.is_empty() {
                ":"  // Use ":" as the prefix for names like ":Person"
            } else {
                first_part
            };

            if let Some(namespace) = self.prefixes.get(prefix) {
                let full_iri = format!("{}{}", namespace, local_name);
                IRI::new(&full_iri)
            } else {
                Err(crate::error::OwlError::ParseError(format!("Unknown prefix: {}", prefix)))
            }
        } else {
            // Assume it's a full IRI without angle brackets
            IRI::new(iri_str)
        }
    }

    /// Validate the parsed ontology
    fn validate_ontology(&self, ontology: &Ontology) -> OwlResult<()> {
        if ontology.classes().is_empty() && ontology.object_properties().is_empty()
            && ontology.data_properties().is_empty() && ontology.named_individuals().is_empty()
            && ontology.imports().is_empty() {
            return Err(crate::error::OwlError::ValidationError("Ontology contains no entities or imports".to_string()));
        }
        Ok(())
    }
}

impl OntologyParser for OwlFunctionalSyntaxParser {
    fn parse_str(&self, content: &str) -> OwlResult<Ontology> {
        // Create a mutable copy for parsing
        let mut parser_copy = OwlFunctionalSyntaxParser::with_config(self.config.clone());
        parser_copy.parse_content(content)
    }

    fn parse_file(&self, path: &Path) -> OwlResult<Ontology> {
        use std::fs;
        use std::io::Read;

        // Check file size
        if self.config.max_file_size > 0 {
            let metadata = fs::metadata(path)?;
            if metadata.len() > self.config.max_file_size as u64 {
                return Err(crate::error::OwlError::ParseError(format!("File size exceeds maximum allowed size: {} bytes", self.config.max_file_size)));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owl_functional_parser_initialization() {
        let parser = OwlFunctionalSyntaxParser::new();
        assert_eq!(parser.format_name(), "OWL Functional Syntax");
    }

    #[test]
    fn test_simple_owl_functional_parsing() {
        let simple_owl = r#"
Prefix(:=<http://example.org/test#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)
Prefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)

Ontology(<http://example.org/test>

Declaration(Class(:Person))
Declaration(Class(:Student))
Declaration(Class(:Professor))

SubClassOf(:Student :Person)
SubClassOf(:Professor :Person)
)
"#;

        let parser = OwlFunctionalSyntaxParser::new();
        let result = parser.parse_str(simple_owl);

        assert!(result.is_ok(), "Parsing failed: {:?}", result);

        if let Ok(ontology) = result {
            assert!(ontology.classes().len() >= 3, "Should have parsed at least 3 classes");
        }
    }

    #[test]
    fn test_iri_resolution() {
        let parser = OwlFunctionalSyntaxParser::new();

        // Test full IRI
        assert!(parser.resolve_iri("<http://example.org/Person>").is_ok());

        // Test unknown prefix
        assert!(parser.resolve_iri("unknown:Person").is_err());
    }

    #[test]
    fn test_prefix_parsing() {
        let test_content = r#"
Prefix(:=<http://example.org/test#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)

Declaration(Class(:Person))
"#;

        let mut parser = OwlFunctionalSyntaxParser::new();
        let mut ontology = Ontology::new();

        // Parse prefixes first
        parser.parse_prefixes(test_content, &mut ontology).unwrap();

        println!("Available prefixes after parsing: {:?}", parser.prefixes);

        // Now test prefixed name resolution
        assert!(parser.resolve_iri(":Person").is_ok());
        assert!(parser.resolve_iri("owl:Class").is_ok());
    }

    #[test]
    fn test_with_config() {
        let config = ParserConfig {
            max_file_size: 1000,
            strict_validation: false,
            resolve_base_iri: false,
            prefixes: std::collections::HashMap::new(),
        };

        let parser = OwlFunctionalSyntaxParser::with_config(config);
        assert_eq!(parser.format_name(), "OWL Functional Syntax");
    }
}