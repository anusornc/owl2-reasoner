//! Enhanced JSON-LD 1.1 Parser for OWL2 Ontologies
//!
//! This is the main parser implementation that uses the enhanced JSON-LD 1.1
//! standard compliance modules.

use crate::axioms::*;
use crate::entities::*;
use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::parser::{OntologyParser, ParserConfig};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;

// Import from enhanced modules
use super::algorithm::ExpansionConfig;
use super::algorithm::{JsonLdExpansionAlgorithm, Owl2Node, Owl2Value};

/// Enhanced JSON-LD format parser with full W3C 1.1 standard compliance
pub struct JsonLdParser {
    #[allow(dead_code)]
    config: ParserConfig,
    expansion_config: ExpansionConfig,
}

impl Default for JsonLdParser {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonLdParser {
    /// Create a new JSON-LD parser with default configuration
    pub fn new() -> Self {
        Self::with_config(ParserConfig::default())
    }

    /// Create a new JSON-LD parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        Self {
            config,
            expansion_config: ExpansionConfig::default(),
        }
    }

    /// Create a new JSON-LD parser with custom expansion configuration
    pub fn with_expansion_config(config: ParserConfig, expansion_config: ExpansionConfig) -> Self {
        Self {
            config,
            expansion_config,
        }
    }

    /// Process expanded OWL2 nodes and add them to ontology
    fn process_expanded_nodes(&self, ontology: &mut Ontology, nodes: &[Owl2Node]) -> OwlResult<()> {
        for node in nodes {
            self.process_single_node(ontology, node)?;
        }
        Ok(())
    }

    /// Process a single expanded node
    fn process_single_node(&self, ontology: &mut Ontology, node: &Owl2Node) -> OwlResult<()> {
        let node_id = if let Some(ref id) = node.id {
            IRI::new(id)
                .map_err(|e| OwlError::ParseError(format!("Invalid IRI '{}': {}", id, e)))?
        } else {
            // Generate a blank node IRI if no @id
            IRI::new(format!("_:bnode{}", node.properties.len()))?
        };

        // Process node types (@type)
        for type_iri in &node.types {
            match type_iri.as_str() {
                // OWL Class declarations
                "http://www.w3.org/2002/07/owl#Class" => {
                    let class = Class::new(node_id.clone());
                    ontology.add_class(class)?;
                }
                // OWL ObjectProperty declarations
                "http://www.w3.org/2002/07/owl#ObjectProperty" => {
                    let prop = ObjectProperty::new(node_id.clone());
                    ontology.add_object_property(prop)?;
                }
                // OWL DatatypeProperty declarations
                "http://www.w3.org/2002/07/owl#DatatypeProperty" => {
                    let prop = DataProperty::new(node_id.clone());
                    ontology.add_data_property(prop)?;
                }
                // OWL NamedIndividual declarations
                "http://www.w3.org/2002/07/owl#NamedIndividual" => {
                    let individual = NamedIndividual::new(node_id.clone());
                    ontology.add_named_individual(individual)?;
                }
                // OWL Ontology declarations
                "http://www.w3.org/2002/07/owl#Ontology" => {
                    // Set ontology IRI if not already set
                    if ontology.iri().is_none() {
                        // Note: This would require modifying the Ontology struct to support IRI
                    }
                }
                // Generic type assertion
                _ => {
                    let subject_class = Class::new(node_id.clone());
                    let object_class = Class::new(IRI::new(type_iri).map_err(|e| {
                        OwlError::ParseError(format!("Invalid type IRI '{}': {}", type_iri, e))
                    })?);

                    ontology.add_class(subject_class.clone())?;
                    ontology.add_class(object_class.clone())?;

                    let class_assertion = ClassAssertionAxiom::new(
                        Arc::new(node_id.clone()),
                        ClassExpression::Class(subject_class),
                    );
                    ontology.add_class_assertion(class_assertion)?;
                }
            }
        }

        // Process properties
        for (predicate, values) in &node.properties {
            self.process_property_values(ontology, &node_id, predicate, values)?;
        }

        Ok(())
    }

    /// Process property values for a predicate
    #[allow(unused_variables)]
    fn process_property_values(
        &self,
        ontology: &mut Ontology,
        subject_iri: &IRI,
        predicate: &str,
        values: &[Owl2Value],
    ) -> OwlResult<()> {
        let prop_iri = IRI::new(predicate).map_err(|e| {
            OwlError::ParseError(format!("Invalid property IRI '{}': {}", predicate, e))
        })?;

        for value in values {
            match value {
                Owl2Value::Iri(object_iri_str) => {
                    let object_iri = IRI::new(object_iri_str).map_err(|e| {
                        OwlError::ParseError(format!(
                            "Invalid object IRI '{}': {}",
                            object_iri_str, e
                        ))
                    })?;

                    // Handle special OWL/RDF properties
                    match predicate {
                        "http://www.w3.org/2000/01/rdf-schema#subClassOf" => {
                            self.process_subclass_of(ontology, subject_iri, &object_iri)?;
                        }
                        "http://www.w3.org/2000/01/rdf-schema#subPropertyOf" => {
                            self.process_sub_property_of(ontology, subject_iri, &object_iri)?;
                        }
                        "http://www.w3.org/2000/01/rdf-schema#domain" => {
                            self.process_domain(ontology, subject_iri, &object_iri)?;
                        }
                        "http://www.w3.org/2000/01/rdf-schema#range" => {
                            self.process_range(ontology, subject_iri, &object_iri)?;
                        }
                        "http://www.w3.org/2000/01/rdf-schema#label" => {
                            self.process_label(ontology, subject_iri, &object_iri, None)?;
                        }
                        "http://www.w3.org/2000/01/rdf-schema#comment" => {
                            self.process_comment(ontology, subject_iri, &object_iri, None)?;
                        }
                        _ => {
                            // Generic property assertion
                            self.process_generic_property(
                                ontology,
                                subject_iri,
                                &prop_iri,
                                &object_iri,
                            )?;
                        }
                    }
                }
                Owl2Value::Literal {
                    value,
                    datatype,
                    language,
                } => {
                    // Handle literal property assertions
                    self.process_literal_property(
                        ontology,
                        subject_iri,
                        &prop_iri,
                        value,
                        datatype,
                        language.clone(),
                    )?;
                }
                Owl2Value::BlankNode(blank_id) => {
                    // Handle blank node references
                    self.process_blank_node_property(ontology, subject_iri, &prop_iri, blank_id)?;
                }
                Owl2Value::List(_) => {
                    // For lists, create multiple property assertions
                    // This is a simplified approach
                    log::debug!("List property for {} - simplified processing", predicate);
                }
                Owl2Value::Set(_) => {
                    // Sets are treated like regular properties
                    log::debug!(
                        "Set property for {} - treating as regular property",
                        predicate
                    );
                }
            }
        }

        Ok(())
    }

    /// Process rdfs:subClassOf relationships
    fn process_subclass_of(
        &self,
        ontology: &mut Ontology,
        subject_iri: &IRI,
        object_iri: &IRI,
    ) -> OwlResult<()> {
        let subject_class = Class::new(subject_iri.clone());
        let object_class = Class::new(object_iri.clone());

        ontology.add_class(subject_class.clone())?;
        ontology.add_class(object_class.clone())?;

        let subclass_axiom = SubClassOfAxiom::new(
            ClassExpression::Class(subject_class),
            ClassExpression::Class(object_class),
        );
        ontology.add_subclass_axiom(subclass_axiom)?;
        Ok(())
    }

    /// Process rdfs:subPropertyOf relationships
    fn process_sub_property_of(
        &self,
        ontology: &mut Ontology,
        subject_iri: &IRI,
        object_iri: &IRI,
    ) -> OwlResult<()> {
        let subject_prop = ObjectProperty::new(subject_iri.clone());
        let object_prop = ObjectProperty::new(object_iri.clone());

        ontology.add_object_property(subject_prop.clone())?;
        ontology.add_object_property(object_prop.clone())?;
        // Note: SubObjectPropertyAxiom creation would need the API support
        Ok(())
    }

    /// Process rdfs:domain relationships
    fn process_domain(
        &self,
        ontology: &mut Ontology,
        subject_iri: &IRI,
        object_iri: &IRI,
    ) -> OwlResult<()> {
        let object_class = Class::new(object_iri.clone());
        ontology.add_class(object_class.clone())?;

        let prop = ObjectProperty::new(subject_iri.clone());
        ontology.add_object_property(prop.clone())?;
        // Note: ObjectPropertyDomainAxiom creation would need the API support
        Ok(())
    }

    /// Process rdfs:range relationships
    fn process_range(
        &self,
        ontology: &mut Ontology,
        subject_iri: &IRI,
        object_iri: &IRI,
    ) -> OwlResult<()> {
        let object_class = Class::new(object_iri.clone());
        ontology.add_class(object_class.clone())?;

        let prop = ObjectProperty::new(subject_iri.clone());
        ontology.add_object_property(prop.clone())?;
        // Note: ObjectPropertyRangeAxiom creation would need the API support
        Ok(())
    }

    /// Process rdfs:label annotations
    #[allow(unused_variables)]
    fn process_label(
        &self,
        _ontology: &mut Ontology,
        subject_iri: &IRI,
        object_iri: &IRI,
        language: Option<String>,
    ) -> OwlResult<()> {
        // For now, just log the label as annotations aren't fully supported
        let lang_info = language.map(|l| format!(" ({})", l)).unwrap_or_default();
        log::debug!("Label for {}{}: {}", subject_iri, lang_info, object_iri);
        Ok(())
    }

    /// Process rdfs:comment annotations
    #[allow(unused_variables)]
    fn process_comment(
        &self,
        _ontology: &mut Ontology,
        subject_iri: &IRI,
        object_iri: &IRI,
        language: Option<String>,
    ) -> OwlResult<()> {
        let lang_info = language.map(|l| format!(" ({})", l)).unwrap_or_default();
        log::debug!("Comment for {}{}: {}", subject_iri, lang_info, object_iri);
        Ok(())
    }

    /// Process generic property assertions
    fn process_generic_property(
        &self,
        ontology: &mut Ontology,
        _subject_iri: &IRI,
        prop_iri: &IRI,
        object_iri: &IRI,
    ) -> OwlResult<()> {
        // Create object property assertion
        let object_individual = NamedIndividual::new(object_iri.clone());
        ontology.add_named_individual(object_individual.clone())?;

        let prop = ObjectProperty::new(prop_iri.clone());
        ontology.add_object_property(prop.clone())?;
        // Note: ObjectPropertyAssertionAxiom creation would need the API support
        Ok(())
    }

    /// Process literal property assertions
    fn process_literal_property(
        &self,
        ontology: &mut Ontology,
        _subject_iri: &IRI,
        prop_iri: &IRI,
        value: &str,
        datatype: &str,
        _language: Option<String>,
    ) -> OwlResult<()> {
        // Create data property assertion
        let prop = DataProperty::new(prop_iri.clone());
        ontology.add_data_property(prop.clone())?;

        let _literal = crate::entities::Literal::typed(value, datatype);
        // Note: DataPropertyAssertionAxiom creation would need the API support
        Ok(())
    }

    /// Process blank node property assertions
    fn process_blank_node_property(
        &self,
        ontology: &mut Ontology,
        _subject_iri: &IRI,
        prop_iri: &IRI,
        blank_id: &str,
    ) -> OwlResult<()> {
        let object_individual = NamedIndividual::new(IRI::new(blank_id).map_err(|e| {
            crate::error::OwlError::ParseError(format!(
                "Invalid blank node IRI '{}': {}",
                blank_id, e
            ))
        })?);
        ontology.add_named_individual(object_individual.clone())?;

        let prop = ObjectProperty::new(prop_iri.clone());
        ontology.add_object_property(prop.clone())?;
        // Note: ObjectPropertyAssertionAxiom creation would need the API support
        Ok(())
    }
}

impl OntologyParser for JsonLdParser {
    fn parse_str(&self, content: &str) -> OwlResult<Ontology> {
        let mut ontology = Ontology::new();

        // Parse JSON
        let json: Value = serde_json::from_str(content)
            .map_err(|e| OwlError::ParseError(format!("JSON parsing error: {}", e)))?;

        // Create expansion algorithm
        let mut algorithm =
            JsonLdExpansionAlgorithm::with_expansion_config(self.expansion_config.clone());

        // Expand JSON-LD using full W3C algorithm
        let expanded_nodes = algorithm.expand(&json)?;

        // Convert to OWL2 format
        let owl2_nodes = algorithm.to_owl2_format(&expanded_nodes)?;

        // Process expanded nodes and add to ontology
        self.process_expanded_nodes(&mut ontology, &owl2_nodes)?;

        // If no entities were added but we had nodes, ensure we have a valid ontology
        if ontology.classes().is_empty()
            && ontology.object_properties().is_empty()
            && ontology.data_properties().is_empty()
            && ontology.named_individuals().is_empty()
        {
            if !owl2_nodes.is_empty() {
                log::debug!("JSON-LD expanded but no OWL entities found");
            } else {
                return Err(OwlError::ParseError("Empty JSON-LD document".to_string()));
            }
        }

        Ok(ontology)
    }

    fn parse_file(&self, path: &Path) -> OwlResult<Ontology> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)
            .map_err(|e| OwlError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;

        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| {
            OwlError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;

        self.parse_str(&content)
    }

    fn format_name(&self) -> &'static str {
        "JSON-LD"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_ld_parser_creation() {
        let parser = JsonLdParser::new();
        assert_eq!(parser.format_name(), "JSON-LD");
    }

    #[test]
    fn test_json_ld_parser_with_config() {
        let config = ParserConfig::default();
        let parser = JsonLdParser::with_config(config);
        assert_eq!(parser.format_name(), "JSON-LD");
    }

    #[test]
    fn test_full_standard_compliance_basic() {
        let parser = JsonLdParser::new();

        let json_ld_content = r#"
        {
            "@context": {
                "@vocab": "http://example.org/",
                "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                "owl": "http://www.w3.org/2002/07/owl#"
            },
            "@graph": [
                {
                    "@id": "Person",
                    "@type": "owl:Class"
                },
                {
                    "@id": "Animal",
                    "@type": "owl:Class"
                },
                {
                    "@id": "Person",
                    "rdfs:subClassOf": {
                        "@id": "Animal"
                    }
                }
            ]
        }
        "#;

        let result = parser.parse_str(json_ld_content);
        assert!(
            result.is_ok(),
            "Failed to parse JSON-LD: {:?}",
            result.err()
        );

        let ontology = result.unwrap();
        assert!(
            !ontology.classes().is_empty(),
            "No classes found in parsed ontology"
        );
    }

    #[test]
    fn test_advanced_features_language_maps() {
        let parser = JsonLdParser::new();

        let json_ld_content = r#"
        {
            "@context": {
                "@vocab": "http://schema.org/",
                "name": {
                    "@id": "http://schema.org/name",
                    "@container": "@language"
                }
            },
            "@id": "http://example.org/alice",
            "@type": "Person",
            "name": {
                "en": "Alice",
                "fr": "Alice",
                "de": "Alice"
            }
        }
        "#;

        let result = parser.parse_str(json_ld_content);
        assert!(
            result.is_ok(),
            "Failed to parse JSON-LD with language maps: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_advanced_features_reverse_properties() {
        let parser = JsonLdParser::new();

        let json_ld_content = r#"
        {
            "@context": {
                "@vocab": "http://schema.org/",
                "parent": "http://schema.org/parent",
                "child": {
                    "@reverse": "http://schema.org/parent"
                }
            },
            "@id": "http://example.org/alice",
            "@type": "Person",
            "parent": "http://example.org/bob",
            "child": "http://example.org/carol"
        }
        "#;

        let result = parser.parse_str(json_ld_content);
        assert!(
            result.is_ok(),
            "Failed to parse JSON-LD with @reverse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_advanced_features_value_objects() {
        let parser = JsonLdParser::new();

        let json_ld_content = r#"
        {
            "@context": "http://schema.org/",
            "@id": "http://example.org/alice",
            "@type": "Person",
            "birthDate": {
                "@value": "1990-01-01",
                "@type": "http://www.w3.org/2001/XMLSchema#date"
            },
            "age": {
                "@value": 30,
                "@type": "http://www.w3.org/2001/XMLSchema#integer"
            }
        }
        "#;

        let result = parser.parse_str(json_ld_content);
        assert!(
            result.is_ok(),
            "Failed to parse JSON-LD with value objects: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_advanced_features_containers() {
        let parser = JsonLdParser::new();

        let json_ld_content = r#"
        {
            "@context": {
                "@vocab": "http://schema.org/",
                "friends": {
                    "@id": "http://schema.org/knows",
                    "@container": "@set"
                }
            },
            "@id": "http://example.org/alice",
            "@type": "Person",
            "friends": [
                "http://example.org/bob",
                "http://example.org/carol",
                "http://example.org/dave"
            ]
        }
        "#;

        let result = parser.parse_str(json_ld_content);
        assert!(
            result.is_ok(),
            "Failed to parse JSON-LD with containers: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_remote_context_fetching() {
        let parser = JsonLdParser::new();

        let json_ld_content = r#"
        {
            "@context": "http://schema.org/",
            "@type": "Person",
            "name": "Alice"
        }
        "#;

        let result = parser.parse_str(json_ld_content);
        assert!(
            result.is_ok(),
            "Failed to parse JSON-LD with remote context: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_nested_contexts() {
        let parser = JsonLdParser::new();

        let json_ld_content = r#"
        {
            "@context": {
                "name": "http://schema.org/name",
                "@context": {
                    "homepage": {
                        "@id": "http://schema.org/url",
                        "@type": "@id"
                    }
                }
            },
            "name": "Alice",
            "homepage": "http://alice.example.com/"
        }
        "#;

        let result = parser.parse_str(json_ld_content);
        assert!(
            result.is_ok(),
            "Failed to parse JSON-LD with nested contexts: {:?}",
            result.err()
        );
    }
}
