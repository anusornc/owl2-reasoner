//! OWL Functional Syntax parser for OWL2 ontologies
//!
//! Implements parsing of the OWL2 Functional Syntax serialization format.

use crate::axioms::*;
use crate::entities::*;
use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::parser::{OntologyParser, ParserConfig};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::path::Path;

type ParserError = OwlError;

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

        OwlFunctionalSyntaxParser { config, prefixes }
    }

    /// Parse OWL Functional Syntax content and build an ontology
    fn parse_content(&mut self, content: &str) -> OwlResult<Ontology> {
        if self.config.strict_validation && content.trim().is_empty() {
            return Err(crate::error::OwlError::ValidationError(
                "Ontology contains no entities or imports".to_string(),
            ));
        }
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
                let prefix_content = &line[7..line.len() - 1];
                if let Some((prefix_part, namespace_part)) = prefix_content.split_once('=') {
                    let mut prefix = prefix_part.trim().trim_matches('<').trim_matches('>');
                    let namespace = namespace_part.trim().trim_matches('<').trim_matches('>');

                    // Special case: empty prefix ":=" should be stored as ":"
                    if prefix == ":" {
                        prefix = ":";
                    } else {
                        // Remove trailing colon for non-empty prefixes
                        prefix = prefix.trim_end_matches(':');
                    }

                    self.prefixes
                        .insert(prefix.to_string(), namespace.to_string());
                }
            }
        }
        Ok(())
    }

    /// Parse ontology declaration
    fn parse_ontology_declaration(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Ontology(") {
                // Handle both single-line and multi-line ontology declarations
                if line.ends_with(")") {
                    // Single-line declaration
                    let ontology_content = &line[9..line.len() - 1];
                    let iri = ontology_content.trim().trim_matches('<').trim_matches('>');
                    return Some(iri.to_string());
                } else {
                    // Multi-line declaration - extract IRI from first line
                    let ontology_content = &line[9..];
                    let iri = ontology_content.trim().trim_matches('<').trim_matches('>');
                    if !iri.is_empty() {
                        return Some(iri.to_string());
                    }
                }
            }
        }
        None
    }

    /// Parse entity declarations
    fn parse_declarations(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Declaration(") && line.ends_with(")") {
                let declaration_content = &line[12..line.len() - 1];
                self.parse_declaration(declaration_content, ontology)?;
            }
        }
        Ok(())
    }

    /// Parse individual declaration
    fn parse_declaration(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let content = content.trim();

        if content.starts_with("Class(") {
            let iri_str = &content[6..content.len() - 1];
            let iri = self.resolve_iri(iri_str)?;
            let class = Class::new(iri);
            ontology.add_class(class)?;
        } else if content.starts_with("ObjectProperty(") {
            let iri_str = &content[15..content.len() - 1];
            let iri = self.resolve_iri(iri_str)?;
            let prop = ObjectProperty::new(iri);
            ontology.add_object_property(prop)?;
        } else if content.starts_with("DataProperty(") {
            let iri_str = &content[13..content.len() - 1];
            let iri = self.resolve_iri(iri_str)?;
            let prop = DataProperty::new(iri);
            ontology.add_data_property(prop)?;
        } else if content.starts_with("NamedIndividual(") {
            let iri_str = &content[16..content.len() - 1];
            let iri = self.resolve_iri(iri_str)?;
            let individual = NamedIndividual::new(iri);
            ontology.add_named_individual(individual)?;
        } else if content.starts_with("AnonymousIndividual(") {
            let node_id_str = &content[19..content.len() - 1];
            // Clean up the node ID by removing unwanted characters
            let node_id = node_id_str.trim()
                .trim_start_matches('(')
                .trim_start_matches('"')
                .trim_end_matches('"')
                .trim_end_matches(')');
            let individual = AnonymousIndividual::new(node_id);
            ontology.add_anonymous_individual(individual)?;
        } else if content.starts_with("AnnotationProperty(") {
            let iri_str = &content[19..content.len() - 1];
            let iri = self.resolve_iri(iri_str)?;
            let prop = AnnotationProperty::new(iri);
            ontology.add_annotation_property(prop)?;
        }

        Ok(())
    }

    /// Parse axioms
    fn parse_axioms(&mut self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        for line in content.lines() {
            let line = line.trim();

            // Class axioms
            if line.starts_with("SubClassOf(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_subclass_of(axiom_content, ontology)?;
            } else if line.starts_with("EquivalentClasses(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_equivalent_classes(axiom_content, ontology)?;
            } else if line.starts_with("DisjointClasses(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_disjoint_classes(axiom_content, ontology)?;
            } else if line.starts_with("DisjointUnion(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_disjoint_union(axiom_content, ontology)?;
            }
            // Property axioms
            else if line.starts_with("SubObjectPropertyOf(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_sub_object_property_of(axiom_content, ontology)?;
            } else if line.starts_with("EquivalentObjectProperties(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_equivalent_object_properties(axiom_content, ontology)?;
            } else if line.starts_with("DisjointObjectProperties(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_disjoint_object_properties(axiom_content, ontology)?;
            } else if line.starts_with("ObjectPropertyDomain(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_object_property_domain(axiom_content, ontology)?;
            } else if line.starts_with("ObjectPropertyRange(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_object_property_range(axiom_content, ontology)?;
            } else if line.starts_with("InverseObjectProperties(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_inverse_object_properties(axiom_content, ontology)?;
            } else if line.starts_with("FunctionalObjectProperty(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_functional_object_property(axiom_content, ontology)?;
            } else if line.starts_with("InverseFunctionalObjectProperty(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_inverse_functional_object_property(axiom_content, ontology)?;
            } else if line.starts_with("ReflexiveObjectProperty(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_reflexive_object_property(axiom_content, ontology)?;
            } else if line.starts_with("IrreflexiveObjectProperty(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_irreflexive_object_property(axiom_content, ontology)?;
            } else if line.starts_with("SymmetricObjectProperty(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_symmetric_object_property(axiom_content, ontology)?;
            } else if line.starts_with("AsymmetricObjectProperty(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_asymmetric_object_property(axiom_content, ontology)?;
            } else if line.starts_with("TransitiveObjectProperty(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_transitive_object_property(axiom_content, ontology)?;
            }
            // Data property axioms
            else if line.starts_with("SubDataPropertyOf(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_sub_data_property_of(axiom_content, ontology)?;
            } else if line.starts_with("EquivalentDataProperties(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_equivalent_data_properties(axiom_content, ontology)?;
            } else if line.starts_with("DisjointDataProperties(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_disjoint_data_properties(axiom_content, ontology)?;
            } else if line.starts_with("DataPropertyDomain(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_data_property_domain(axiom_content, ontology)?;
            } else if line.starts_with("DataPropertyRange(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_data_property_range(axiom_content, ontology)?;
            } else if line.starts_with("FunctionalDataProperty(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_functional_data_property(axiom_content, ontology)?;
            }
            // Assertion axioms
            else if line.starts_with("ClassAssertion(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_class_assertion(axiom_content, ontology)?;
            } else if line.starts_with("ObjectPropertyAssertion(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_object_property_assertion(axiom_content, ontology)?;
            } else if line.starts_with("DataPropertyAssertion(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_data_property_assertion(axiom_content, ontology)?;
            } else if line.starts_with("NegativeObjectPropertyAssertion(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_negative_object_property_assertion(axiom_content, ontology)?;
            } else if line.starts_with("NegativeDataPropertyAssertion(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_negative_data_property_assertion(axiom_content, ontology)?;
            } else if line.starts_with("SameIndividual(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_same_individual(axiom_content, ontology)?;
            } else if line.starts_with("DifferentIndividuals(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_different_individuals(axiom_content, ontology)?;
            }
            // Annotation axioms
            else if line.starts_with("AnnotationAssertion(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_annotation_assertion(axiom_content, ontology)?;
            } else if line.starts_with("SubAnnotationPropertyOf(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_sub_annotation_property_of(axiom_content, ontology)?;
            } else if line.starts_with("AnnotationPropertyDomain(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_annotation_property_domain(axiom_content, ontology)?;
            } else if line.starts_with("AnnotationPropertyRange(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_annotation_property_range(axiom_content, ontology)?;
            } else if line.starts_with("Import(") && line.ends_with(")") {
                let open = line.find('(').unwrap();
                let axiom_content = &line[open + 1..line.len() - 1];
                self.parse_import(axiom_content, ontology)?;
            }
            // Other axioms
            else if line.starts_with("HasKey(") && line.ends_with(")") {
                let axiom_content = &line[8..line.len() - 1];
                self.parse_has_key(axiom_content, ontology)?;
            } else if line.starts_with("Declaration(") && line.ends_with(")") {
                // Already handled in parse_declarations
            } else if line.starts_with("Ontology(") {
                // Skip ontology declaration lines (handled in parse_ontology_declaration)
                continue;
            } else if line.trim() == ")" {
                // Skip closing parenthesis of multi-line ontology declaration
                continue;
            } else if !line.is_empty() && !line.starts_with("Prefix(") {
                // Unknown axiom type
                if self.config.strict_validation {
                    return Err(crate::error::OwlError::ParseError(format!(
                        "Unknown axiom type: {line}"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parse a class expression from OWL Functional Syntax
    fn parse_class_expression(&mut self, expr_str: &str) -> OwlResult<crate::axioms::class_expressions::ClassExpression> {
        use crate::axioms::class_expressions::ClassExpression;

        // Handle complex class expressions
        if expr_str.starts_with("ObjectIntersectionOf(") {
            let content = &expr_str["ObjectIntersectionOf(".len()..expr_str.len() - 1];
            let operands = self.parse_class_expression_list(content)?;
            Ok(ClassExpression::ObjectIntersectionOf(SmallVec::from_vec(
                operands.into_iter().map(Box::new).collect()
            )))
        } else if expr_str.starts_with("ObjectUnionOf(") {
            let content = &expr_str["ObjectUnionOf(".len()..expr_str.len() - 1];
            let operands = self.parse_class_expression_list(content)?;
            Ok(ClassExpression::ObjectUnionOf(SmallVec::from_vec(
                operands.into_iter().map(Box::new).collect()
            )))
        } else if expr_str.starts_with("ObjectComplementOf(") {
            let content = &expr_str["ObjectComplementOf(".len()..expr_str.len() - 1];
            let operand = Box::new(self.parse_class_expression(content)?);
            Ok(ClassExpression::ObjectComplementOf(operand))
        } else if expr_str.starts_with("ObjectSomeValuesFrom(") {
            let content = &expr_str["ObjectSomeValuesFrom(".len()..expr_str.len() - 1];
            let (property, filler) = self.parse_property_restriction_with_expression(content)?;
            Ok(ClassExpression::ObjectSomeValuesFrom(
                Box::new(property),
                Box::new(filler),
            ))
        } else if expr_str.starts_with("ObjectAllValuesFrom(") {
            let content = &expr_str["ObjectAllValuesFrom(".len()..expr_str.len() - 1];
            let (property, filler) = self.parse_property_restriction_with_expression(content)?;
            Ok(ClassExpression::ObjectAllValuesFrom(
                Box::new(property),
                Box::new(filler),
            ))
        } else if expr_str.starts_with("ObjectHasValue(") {
            let content = &expr_str["ObjectHasValue(".len()..expr_str.len() - 1];
            let (property, individual) = self.parse_object_has_value_with_expression(content)?;
            Ok(ClassExpression::ObjectHasValue(
                Box::new(property),
                individual,
            ))
        } else if expr_str.starts_with("ObjectHasSelf(") {
            let content = &expr_str["ObjectHasSelf(".len()..expr_str.len() - 1];
            let property = self.parse_object_property_expression(content)?;
            Ok(ClassExpression::ObjectHasSelf(
                Box::new(property)
            ))
        } else if expr_str.starts_with("ObjectMinCardinality(") {
            let content = &expr_str["ObjectMinCardinality(".len()..expr_str.len() - 1];
            let (cardinality, property) = self.parse_cardinality_restriction_with_expression(content)?;
            Ok(ClassExpression::ObjectMinCardinality(
                cardinality,
                Box::new(property),
            ))
        } else if expr_str.starts_with("ObjectMaxCardinality(") {
            let content = &expr_str["ObjectMaxCardinality(".len()..expr_str.len() - 1];
            let (cardinality, property) = self.parse_cardinality_restriction_with_expression(content)?;
            Ok(ClassExpression::ObjectMaxCardinality(
                cardinality,
                Box::new(property),
            ))
        } else if expr_str.starts_with("ObjectExactCardinality(") {
            let content = &expr_str["ObjectExactCardinality(".len()..expr_str.len() - 1];
            let (cardinality, property) = self.parse_cardinality_restriction_with_expression(content)?;
            Ok(ClassExpression::ObjectExactCardinality(
                cardinality,
                Box::new(property),
            ))
        // Handle data range expressions
        } else if expr_str.starts_with("DataSomeValuesFrom(") {
            let content = &expr_str["DataSomeValuesFrom(".len()..expr_str.len() - 1];
            let (property, range) = self.parse_data_property_restriction(content)?;
            Ok(ClassExpression::DataSomeValuesFrom(
                Box::new(property),
                Box::new(range),
            ))
        } else if expr_str.starts_with("DataAllValuesFrom(") {
            let content = &expr_str["DataAllValuesFrom(".len()..expr_str.len() - 1];
            let (property, range) = self.parse_data_property_restriction(content)?;
            Ok(ClassExpression::DataAllValuesFrom(
                Box::new(property),
                Box::new(range),
            ))
        } else if expr_str.starts_with("DataHasValue(") {
            let content = &expr_str["DataHasValue(".len()..expr_str.len() - 1];
            let (property, literal) = self.parse_data_has_value(content)?;
            Ok(ClassExpression::DataHasValue(
                Box::new(property),
                literal,
            ))
        } else if expr_str.starts_with("DataMinCardinality(") {
            let content = &expr_str["DataMinCardinality(".len()..expr_str.len() - 1];
            let (cardinality, property) = self.parse_data_cardinality_restriction(content)?;
            Ok(ClassExpression::DataMinCardinality(
                cardinality,
                Box::new(property),
            ))
        } else if expr_str.starts_with("DataMaxCardinality(") {
            let content = &expr_str["DataMaxCardinality(".len()..expr_str.len() - 1];
            let (cardinality, property) = self.parse_data_cardinality_restriction(content)?;
            Ok(ClassExpression::DataMaxCardinality(
                cardinality,
                Box::new(property),
            ))
        } else if expr_str.starts_with("DataExactCardinality(") {
            let content = &expr_str["DataExactCardinality(".len()..expr_str.len() - 1];
            let (cardinality, property) = self.parse_data_cardinality_restriction(content)?;
            Ok(ClassExpression::DataExactCardinality(
                cardinality,
                Box::new(property),
            ))
        } else if expr_str.starts_with("ObjectOneOf(") {
            let content = &expr_str["ObjectOneOf(".len()..expr_str.len() - 1];
            let individuals = self.parse_individual_list(content)?;
            Ok(ClassExpression::ObjectOneOf(Box::new(SmallVec::from_vec(individuals))))
        } else {
            // Simple named class
            let class_iri = self.resolve_iri(expr_str)?;
            let class = Class::new(class_iri);
            Ok(ClassExpression::Class(class))
        }
    }

    /// Parse a list of individuals for ObjectOneOf
    fn parse_individual_list(&mut self, content: &str) -> OwlResult<Vec<crate::entities::Individual>> {
        let mut individuals = Vec::new();
        let mut current = String::new();
        let mut paren_depth = 0;

        for ch in content.chars() {
            match ch {
                '(' => paren_depth += 1,
                ')' => paren_depth -= 1,
                ' ' if paren_depth == 0 => {
                    if !current.is_empty() {
                        individuals.push(self.parse_individual(&current)?);
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        // Add the last individual if there's one remaining
        if !current.is_empty() {
            individuals.push(self.parse_individual(&current)?);
        }

        Ok(individuals)
    }

    /// Parse a single individual (named or anonymous)
    fn parse_individual(&mut self, individual_str: &str) -> OwlResult<crate::entities::Individual> {
        let trimmed = individual_str.trim();

        // Check if it's an anonymous individual (starts with _)
        if trimmed.starts_with('_') {
            // For now, treat as named individual with the ID
            // In a full implementation, you'd create AnonymousIndividual
            let iri = self.resolve_iri(trimmed)?;
            let named_individual = NamedIndividual::new(iri);
            Ok(crate::entities::Individual::Named(named_individual))
        } else {
            // Named individual
            let iri = self.resolve_iri(trimmed)?;
            let named_individual = NamedIndividual::new(iri);
            Ok(crate::entities::Individual::Named(named_individual))
        }
    }

    /// Parse an object property expression (handle inverse properties)
    fn parse_object_property_expression(&self, expr_str: &str) -> OwlResult<crate::axioms::property_expressions::ObjectPropertyExpression> {
        
        if expr_str.starts_with("ObjectInverseOf(") {
            let inverse_content = &expr_str["ObjectInverseOf(".len()..expr_str.len() - 1];
            let base_property = Box::new(self.parse_object_property_expression(inverse_content)?);
            Ok(ObjectPropertyExpression::ObjectInverseOf(base_property))
        } else {
            // Simple named property
            let property_iri = self.resolve_iri(expr_str)?;
            let property = crate::entities::ObjectProperty::new(property_iri);
            Ok(ObjectPropertyExpression::ObjectProperty(Box::new(property)))
        }
    }

    /// Parse a property restriction with property expression support
    fn parse_property_restriction_with_expression(&mut self, content: &str) -> OwlResult<(crate::axioms::property_expressions::ObjectPropertyExpression, crate::axioms::class_expressions::ClassExpression)> {
        // Find the split between property and filler expressions
        let mut paren_count = 0;
        let mut split_pos = None;

        for (i, ch) in content.chars().enumerate() {
            if ch == '(' {
                paren_count += 1;
            } else if ch == ')' {
                paren_count -= 1;
            } else if ch == ' ' && paren_count == 0 {
                split_pos = Some(i);
                break;
            }
        }

        if let Some(pos) = split_pos {
            let property_expr = &content[..pos];
            let filler_expr = &content[pos + 1..];

            let property = self.parse_object_property_expression(property_expr)?;
            let filler = self.parse_class_expression(filler_expr)?;

            Ok((property, filler))
        } else {
            Err(crate::error::OwlError::ParseError(
                format!("Invalid property restriction format: {}", content)
            ))
        }
    }

    /// Parse ObjectHasValue expression with property expression support
    fn parse_object_has_value_with_expression(&self, content: &str) -> OwlResult<(crate::axioms::property_expressions::ObjectPropertyExpression, crate::entities::Individual)> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let property_expr = parts[0];
            let individual_iri = self.resolve_iri(parts[1])?;

            let property = self.parse_object_property_expression(property_expr)?;
            let individual = crate::entities::NamedIndividual::new(individual_iri);
            Ok((property, crate::entities::Individual::Named(individual)))
        } else {
            Err(crate::error::OwlError::ParseError(
                format!("Invalid ObjectHasValue format: {}", content)
            ))
        }
    }

    /// Parse cardinality restriction with property expression support
    fn parse_cardinality_restriction_with_expression(&self, content: &str) -> OwlResult<(u32, crate::axioms::property_expressions::ObjectPropertyExpression)> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let cardinality_str = parts[0];
            let property_expr = parts[1];

            // Parse cardinality as u32
            let cardinality = cardinality_str.parse::<u32>().map_err(|_| {
                crate::error::OwlError::ParseError(
                    format!("Invalid cardinality number: {}", cardinality_str)
                )
            })?;

            let property = self.parse_object_property_expression(property_expr)?;
            Ok((cardinality, property))
        } else {
            Err(crate::error::OwlError::ParseError(
                format!("Invalid cardinality restriction format: {}", content)
            ))
        }
    }

    /// Parse a list of class expressions
    fn parse_class_expression_list(&mut self, content: &str) -> OwlResult<Vec<crate::axioms::class_expressions::ClassExpression>> {
        let mut expressions = Vec::new();
        let mut current_expr = String::new();
        let mut paren_count = 0;

        for ch in content.chars() {
            if ch == '(' {
                paren_count += 1;
                current_expr.push(ch);
            } else if ch == ')' {
                paren_count -= 1;
                current_expr.push(ch);
                if paren_count == 0 && !current_expr.trim().is_empty() {
                    expressions.push(self.parse_class_expression(current_expr.trim())?);
                    current_expr.clear();
                }
            } else if ch == ' ' && paren_count == 0 && !current_expr.trim().is_empty() {
                expressions.push(self.parse_class_expression(current_expr.trim())?);
                current_expr.clear();
            } else {
                current_expr.push(ch);
            }
        }

        // Add the last expression if there's one
        if !current_expr.trim().is_empty() {
            expressions.push(self.parse_class_expression(current_expr.trim())?);
        }

        Ok(expressions)
    }

    /// Parse SubClassOf axiom
    fn parse_subclass_of(&mut self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        // Find the split between subclass and superclass expressions
        let mut paren_count = 0;
        let mut split_pos = None;

        for (i, ch) in content.chars().enumerate() {
            if ch == '(' {
                paren_count += 1;
            } else if ch == ')' {
                paren_count -= 1;
            } else if ch == ' ' && paren_count == 0 {
                split_pos = Some(i);
                break;
            }
        }

        if let Some(pos) = split_pos {
            let sub_expr = &content[..pos];
            let super_expr = &content[pos + 1..];

            let sub_class = self.parse_class_expression(sub_expr)?;
            let super_class = self.parse_class_expression(super_expr)?;

            let subclass_axiom = SubClassOfAxiom::new(sub_class, super_class);
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
            let iri_content = &iri_str[1..iri_str.len() - 1];
            IRI::new(iri_content)
        } else if iri_str.contains(':') {
            // Prefixed name
            let mut parts = iri_str.splitn(2, ':');
            let first_part = parts.next().unwrap_or("");
            let local_name = parts.next().unwrap_or("");

            // Handle the case where the prefix is empty (e.g., ":Person")
            let prefix = if first_part.is_empty() {
                ":" // Use ":" as the prefix for names like ":Person"
            } else {
                first_part
            };

            if let Some(namespace) = self.prefixes.get(prefix) {
                let full_iri = format!("{namespace}{local_name}");
                IRI::new(&full_iri)
            } else {
                Err(crate::error::OwlError::ParseError(format!(
                    "Unknown prefix: {prefix}"
                )))
            }
        } else {
            // Assume it's a full IRI without angle brackets
            IRI::new(iri_str)
        }
    }

    /// Extract IRI from a ClassExpression, returning error if it's not a simple class
    fn extract_iri_from_class_expression(&self, expr: &ClassExpression) -> OwlResult<IRI> {
        match expr {
            ClassExpression::Class(class) => Ok(class.iri().clone()),
            _ => Err(crate::error::OwlError::ParseError(
                format!("Expected simple class, found complex expression: {:?}", expr)
            ))
        }
    }

    /// Parse EquivalentClasses axiom
    fn parse_equivalent_classes(&mut self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let class_expressions = self.parse_class_expression_list(content)?;
        if class_expressions.len() >= 2 {
            let mut class_iris = Vec::new();
            for expr in class_expressions {
                class_iris.push(self.extract_iri_from_class_expression(&expr)?);
            }
            let equiv_axiom = EquivalentClassesAxiom::new(class_iris);
            ontology.add_axiom(Axiom::EquivalentClasses(Box::new(equiv_axiom)))?;
        }
        Ok(())
    }

    /// Parse DisjointClasses axiom
    fn parse_disjoint_classes(&mut self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let class_expressions = self.parse_class_expression_list(content)?;
        if class_expressions.len() >= 2 {
            let mut class_iris = Vec::new();
            for expr in class_expressions {
                class_iris.push(self.extract_iri_from_class_expression(&expr)?);
            }
            let disjoint_axiom = DisjointClassesAxiom::new(class_iris);
            ontology.add_axiom(Axiom::DisjointClasses(Box::new(disjoint_axiom)))?;
        }
        Ok(())
    }

    /// Parse DisjointUnion axiom
    fn parse_disjoint_union(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let main_class_iri = self.resolve_iri(parts[0])?;
            let mut class_iris = vec![main_class_iri.clone()];

            for part in &parts[1..] {
                class_iris.push(self.resolve_iri(part)?);
            }

            let disjoint_axiom = DisjointClassesAxiom::new(class_iris[1..].to_vec());
            ontology.add_axiom(Axiom::DisjointClasses(Box::new(disjoint_axiom)))?;
        }
        Ok(())
    }

    /// Parse SubObjectPropertyOf axiom
    fn parse_sub_object_property_of(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let sub_prop_iri = self.resolve_iri(parts[0])?;
            let super_prop_iri = self.resolve_iri(parts[1])?;

            let sub_axiom = SubObjectPropertyAxiom::new(sub_prop_iri, super_prop_iri);
            ontology.add_axiom(Axiom::SubObjectProperty(Box::new(sub_axiom)))?;
        }
        Ok(())
    }

    /// Parse EquivalentObjectProperties axiom
    fn parse_equivalent_object_properties(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let prop_iris = self.parse_iri_list(content)?;
        if prop_iris.len() >= 2 {
            let equiv_axiom = EquivalentObjectPropertiesAxiom::new(prop_iris);
            ontology.add_axiom(Axiom::EquivalentObjectProperties(Box::new(equiv_axiom)))?;
        }
        Ok(())
    }

    /// Parse DisjointObjectProperties axiom
    fn parse_disjoint_object_properties(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let prop_iris = self.parse_iri_list(content)?;
        if prop_iris.len() >= 2 {
            let disjoint_axiom = DisjointObjectPropertiesAxiom::new(prop_iris);
            ontology.add_axiom(Axiom::DisjointObjectProperties(Box::new(disjoint_axiom)))?;
        }
        Ok(())
    }

    /// Parse ObjectPropertyDomain axiom
    fn parse_object_property_domain(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let prop_iri = self.resolve_iri(parts[0])?;
            let domain_iri = self.resolve_iri(parts[1])?;

            let domain_axiom = ObjectPropertyDomainAxiom::new(
                prop_iri.clone(),
                ClassExpression::Class(Class::new(domain_iri.clone())),
            );
            ontology.add_axiom(Axiom::ObjectPropertyDomain(Box::new(domain_axiom)))?;
        }
        Ok(())
    }

    /// Parse ObjectPropertyRange axiom
    fn parse_object_property_range(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let prop_iri = self.resolve_iri(parts[0])?;
            let range_iri = self.resolve_iri(parts[1])?;

            let range_axiom = ObjectPropertyRangeAxiom::new(
                prop_iri.clone(),
                ClassExpression::Class(Class::new(range_iri.clone())),
            );
            ontology.add_axiom(Axiom::ObjectPropertyRange(Box::new(range_axiom)))?;
        }
        Ok(())
    }

    /// Parse InverseObjectProperties axiom
    fn parse_inverse_object_properties(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let prop1_iri = self.resolve_iri(parts[0])?;
            let prop2_iri = self.resolve_iri(parts[1])?;

            let inverse_axiom = InverseObjectPropertiesAxiom::new(
                ObjectPropertyExpression::ObjectProperty(Box::new(ObjectProperty::new(prop1_iri))),
                ObjectPropertyExpression::ObjectProperty(Box::new(ObjectProperty::new(prop2_iri))),
            );
            ontology.add_axiom(Axiom::InverseObjectProperties(Box::new(inverse_axiom)))?;
        }
        Ok(())
    }

    /// Parse FunctionalObjectProperty axiom
    fn parse_functional_object_property(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let iri = self.resolve_iri(content.trim())?;
        let functional_axiom = FunctionalPropertyAxiom::new(iri);
        ontology.add_axiom(Axiom::FunctionalProperty(Box::new(functional_axiom)))?;
        Ok(())
    }

    /// Parse InverseFunctionalObjectProperty axiom
    fn parse_inverse_functional_object_property(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let iri = self.resolve_iri(content.trim())?;
        let inverse_functional_axiom = InverseFunctionalPropertyAxiom::new(iri);
        ontology.add_axiom(Axiom::InverseFunctionalProperty(Box::new(inverse_functional_axiom)))?;
        Ok(())
    }

    /// Parse ReflexiveObjectProperty axiom
    fn parse_reflexive_object_property(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let iri = self.resolve_iri(content.trim())?;
        let reflexive_axiom = ReflexivePropertyAxiom::new(iri);
        ontology.add_axiom(Axiom::ReflexiveProperty(Box::new(reflexive_axiom)))?;
        Ok(())
    }

    /// Parse IrreflexiveObjectProperty axiom
    fn parse_irreflexive_object_property(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let iri = self.resolve_iri(content.trim())?;
        let irreflexive_axiom = IrreflexivePropertyAxiom::new(iri);
        ontology.add_axiom(Axiom::IrreflexiveProperty(Box::new(irreflexive_axiom)))?;
        Ok(())
    }

    /// Parse SymmetricObjectProperty axiom
    fn parse_symmetric_object_property(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let iri = self.resolve_iri(content.trim())?;
        let symmetric_axiom = SymmetricPropertyAxiom::new(iri);
        ontology.add_axiom(Axiom::SymmetricProperty(Box::new(symmetric_axiom)))?;
        Ok(())
    }

    /// Parse AsymmetricObjectProperty axiom
    fn parse_asymmetric_object_property(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let iri = self.resolve_iri(content.trim())?;
        let asymmetric_axiom = AsymmetricPropertyAxiom::new(iri);
        ontology.add_axiom(Axiom::AsymmetricProperty(Box::new(asymmetric_axiom)))?;
        Ok(())
    }

    /// Parse TransitiveObjectProperty axiom
    fn parse_transitive_object_property(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let iri = self.resolve_iri(content.trim())?;
        let transitive_axiom = TransitivePropertyAxiom::new(iri);
        ontology.add_axiom(Axiom::TransitiveProperty(Box::new(transitive_axiom)))?;
        Ok(())
    }

    /// Parse SubDataPropertyOf axiom
    fn parse_sub_data_property_of(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let sub_prop_iri = self.resolve_iri(parts[0])?;
            let super_prop_iri = self.resolve_iri(parts[1])?;

            let sub_axiom = SubDataPropertyAxiom::new(sub_prop_iri, super_prop_iri);
            ontology.add_axiom(Axiom::SubDataProperty(Box::new(sub_axiom)))?;
        }
        Ok(())
    }

    /// Parse EquivalentDataProperties axiom
    fn parse_equivalent_data_properties(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let prop_iris = self.parse_iri_list(content)?;
        if prop_iris.len() >= 2 {
            let equiv_axiom = EquivalentDataPropertiesAxiom::new(prop_iris);
            ontology.add_axiom(Axiom::EquivalentDataProperties(Box::new(equiv_axiom)))?;
        }
        Ok(())
    }

    /// Parse DisjointDataProperties axiom
    fn parse_disjoint_data_properties(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let prop_iris = self.parse_iri_list(content)?;
        if prop_iris.len() >= 2 {
            let disjoint_axiom = DisjointDataPropertiesAxiom::new(prop_iris);
            ontology.add_axiom(Axiom::DisjointDataProperties(Box::new(disjoint_axiom)))?;
        }
        Ok(())
    }

    /// Parse DataPropertyDomain axiom
    fn parse_data_property_domain(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let prop_iri = self.resolve_iri(parts[0])?;
            let domain_iri = self.resolve_iri(parts[1])?;

            let domain_axiom = DataPropertyDomainAxiom::new(
                prop_iri.clone(),
                ClassExpression::Class(Class::new(domain_iri.clone())),
            );
            ontology.add_axiom(Axiom::DataPropertyDomain(Box::new(domain_axiom)))?;
        }
        Ok(())
    }

    /// Parse DataPropertyRange axiom
    fn parse_data_property_range(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let prop_iri = self.resolve_iri(parts[0])?;
            let range_iri = self.resolve_iri(parts[1])?;

            let range_axiom = DataPropertyRangeAxiom::new(prop_iri.clone(), range_iri.clone());
            ontology.add_axiom(Axiom::DataPropertyRange(Box::new(range_axiom)))?;
        }
        Ok(())
    }

    /// Parse FunctionalDataProperty axiom
    fn parse_functional_data_property(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let iri = self.resolve_iri(content.trim())?;
        let functional_axiom = FunctionalDataPropertyAxiom::new(iri);
        ontology.add_axiom(Axiom::FunctionalDataProperty(functional_axiom))?;
        Ok(())
    }

    /// Parse ObjectPropertyAssertion axiom
    fn parse_object_property_assertion(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            let prop_iri = self.resolve_iri(parts[0])?;
            let subject_iri = self.resolve_iri(parts[1])?;
            let object_iri = self.resolve_iri(parts[2])?;

            let assertion = PropertyAssertionAxiom::new(subject_iri, prop_iri, object_iri);
            ontology.add_axiom(Axiom::PropertyAssertion(Box::new(assertion)))?;
        }
        Ok(())
    }

    /// Parse DataPropertyAssertion axiom
    fn parse_data_property_assertion(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            let subject_iri = self.resolve_iri(parts[0])?;
            let prop_iri = self.resolve_iri(parts[1])?;

            // Parse literal from remaining parts
            let (literal, _remaining_parts) = self.parse_literal_from_parts(&parts[2..])?;

            if let Some(lit) = literal {
                let assertion = DataPropertyAssertionAxiom::new(
                    subject_iri.clone(),
                    prop_iri.clone(),
                    lit,
                );
                ontology.add_axiom(Axiom::DataPropertyAssertion(Box::new(assertion)))?;
            }
        }
        Ok(())
    }

    /// Parse a literal from parts, handling different literal formats
    fn parse_literal_from_parts<'a>(&self, parts: &[&'a str]) -> OwlResult<(Option<crate::entities::Literal>, Vec<&'a str>)> {
        if parts.is_empty() {
            return Ok((None, Vec::new()));
        }

        let first_part = parts[0];

        // Handle quoted literals
        if first_part.starts_with('"') {
            let mut full_literal = first_part.to_string();
            let mut parts_consumed = 1;

            // Handle case where the literal spans multiple parts
            for part in &parts[1..] {
                if full_literal.contains('"') && !full_literal.ends_with('"') {
                    // Still looking for closing quote
                    full_literal.push(' ');
                    full_literal.push_str(part);
                    parts_consumed += 1;
                } else {
                    break;
                }
            }

            // Extract the literal value and any datatype/language tag
            let (literal_value, datatype, language_tag) = if let Some(closing_quote_pos) = full_literal.rfind('"') {
                let after_quote = &full_literal[closing_quote_pos + 1..];

                if after_quote.starts_with("^^") {
                    // Typed literal: "value"^^datatype
                    let datatype_part = &after_quote[2..];
                    let datatype_iri = self.resolve_iri(datatype_part)?;
                    (full_literal[1..closing_quote_pos].to_string(), Some(datatype_iri), None)
                } else if after_quote.starts_with('@') {
                    // Language-tagged literal: "value"@language
                    let language_part = &after_quote[1..];
                    (full_literal[1..closing_quote_pos].to_string(), None, Some(language_part.to_string()))
                } else {
                    // Simple string literal
                    (full_literal[1..closing_quote_pos].to_string(), None, None)
                }
            } else {
                return Ok((None, parts.to_vec()));
            };

            // Create the literal
            let literal = if let Some(datatype_iri) = datatype {
                Some(crate::entities::Literal::typed(literal_value, datatype_iri))
            } else if let Some(lang_tag) = language_tag {
                Some(crate::entities::Literal::lang_tagged(literal_value, lang_tag))
            } else {
                Some(crate::entities::Literal::simple(literal_value))
            };

            let remaining = parts[parts_consumed..].to_vec();
            Ok((literal, remaining))
        } else {
            // Not a quoted literal, treat as simple IRI or error
            Ok((None, parts.to_vec()))
        }
    }

    /// Parse NegativeObjectPropertyAssertion axiom
    fn parse_negative_object_property_assertion(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            let subject_iri = self.resolve_iri(parts[0])?;
            let prop_iri = self.resolve_iri(parts[1])?;
            let object_iri = self.resolve_iri(parts[2])?;

            let neg_assertion = NegativeObjectPropertyAssertionAxiom::new(
                subject_iri.clone(),
                prop_iri.clone(),
                object_iri.clone(),
            );
            ontology.add_axiom(Axiom::NegativeObjectPropertyAssertion(Box::new(neg_assertion)))?;
        }
        Ok(())
    }

    /// Parse NegativeDataPropertyAssertion axiom
    fn parse_negative_data_property_assertion(
        &self,
        content: &str,
        ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            let subject_iri = self.resolve_iri(parts[0])?;
            let prop_iri = self.resolve_iri(parts[1])?;

            // Parse literal from remaining parts using the same sophisticated parser
            let (literal, _remaining_parts) = self.parse_literal_from_parts(&parts[2..])?;

            if let Some(lit) = literal {
                let neg_assertion = NegativeDataPropertyAssertionAxiom::new(
                    subject_iri.clone(),
                    prop_iri.clone(),
                    lit,
                );
                ontology.add_axiom(Axiom::NegativeDataPropertyAssertion(Box::new(neg_assertion)))?;
            }
        }
        Ok(())
    }

    /// Parse SameIndividual axiom
    fn parse_same_individual(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let individual_iris = self.parse_iri_list(content)?;
        if individual_iris.len() >= 2 {
            let same_axiom = SameIndividualAxiom::new(individual_iris);
            ontology.add_axiom(Axiom::SameIndividual(Box::new(same_axiom)))?;
        }
        Ok(())
    }

    /// Parse DifferentIndividuals axiom
    fn parse_different_individuals(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let individual_iris = self.parse_iri_list(content)?;
        if individual_iris.len() >= 2 {
            let different_axiom = DifferentIndividualsAxiom::new(individual_iris);
            ontology.add_axiom(Axiom::DifferentIndividuals(Box::new(different_axiom)))?;
        }
        Ok(())
    }

    /// Parse HasKey axiom
    fn parse_has_key(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        // HasKey syntax: HasKey(Class (property1 property2 ...))
        // Need to extract the class and handle parenthesized property list
        let trimmed = content.trim();
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_parens = false;

        for ch in trimmed.chars() {
            if ch == '(' {
                in_parens = true;
            } else if ch == ')' {
                in_parens = false;
                if !current.is_empty() {
                    parts.push(current.trim().to_string());
                    current.clear();
                }
            } else if ch.is_whitespace() && !in_parens {
                if !current.is_empty() {
                    parts.push(current.trim().to_string());
                    current.clear();
                }
            } else {
                current.push(ch);
            }
        }

        if !current.is_empty() {
            parts.push(current.trim().to_string());
        }

        if parts.len() >= 2 {
            let class_iri = self.resolve_iri(&parts[0])?;
            let mut property_iris = Vec::new();

            // Parse remaining parts as property IRIs
            for part in &parts[1..] {
                let clean_part = part.trim_matches(|c| c == '(' || c == ')');
                if !clean_part.is_empty() {
                    let prop_iri = self.resolve_iri(clean_part)?;
                    property_iris.push(prop_iri);
                }
            }

            if !property_iris.is_empty() {
                let has_key_axiom = HasKeyAxiom::new(
                    ClassExpression::Class(Class::new(class_iri.clone())),
                    property_iris,
                );
                ontology.add_axiom(Axiom::HasKey(Box::new(has_key_axiom)))?;
            }
        }
        Ok(())
    }

    /// Parse a list of IRIs from content
    fn parse_iri_list(&self, content: &str) -> OwlResult<Vec<IRI>> {
        let mut iris = Vec::new();
        let parts: Vec<&str> = content.split_whitespace().collect();

        for part in parts {
            let iri = self.resolve_iri(part)?;
            iris.push(iri);
        }

        Ok(iris)
    }

    /// Validate the parsed ontology
    fn validate_ontology(&self, ontology: &Ontology) -> OwlResult<()> {
        if ontology.classes().is_empty()
            && ontology.object_properties().is_empty()
            && ontology.data_properties().is_empty()
            && ontology.named_individuals().is_empty()
            && ontology.imports().is_empty()
        {
            return Err(crate::error::OwlError::ValidationError(
                "Ontology contains no entities or imports".to_string(),
            ));
        }
        Ok(())
    }

    /// Parse a data property restriction for DataSomeValuesFrom and DataAllValuesFrom
    fn parse_data_property_restriction(
        &mut self,
        content: &str,
    ) -> Result<(DataPropertyExpression, DataRange), ParserError> {
        let trimmed = content.trim();
        let space_pos = trimmed.find(' ').ok_or_else(|| {
            ParserError::ParseError("Expected space between property and data range".to_string())
        })?;

        let property_str = &trimmed[..space_pos];
        let range_str = &trimmed[space_pos + 1..];

        let property = self.parse_data_property_expression(property_str)?;
        let range = self.parse_data_range(range_str)?;

        Ok((property, range))
    }

    /// Parse DataHasValue restriction
    fn parse_data_has_value(
        &mut self,
        content: &str,
    ) -> Result<(DataPropertyExpression, Literal), ParserError> {
        let trimmed = content.trim();
        let space_pos = trimmed.find(' ').ok_or_else(|| {
            ParserError::ParseError("Expected space between property and value".to_string())
        })?;

        let property_str = &trimmed[..space_pos];
        let value_str = &trimmed[space_pos + 1..];

        let property = self.parse_data_property_expression(property_str)?;
        let literal = self.parse_literal(value_str)?;

        Ok((property, literal))
    }

    /// Parse data cardinality restriction
    fn parse_data_cardinality_restriction(
        &mut self,
        content: &str,
    ) -> Result<(u32, DataPropertyExpression), ParserError> {
        let trimmed = content.trim();
        let space_pos = trimmed.find(' ').ok_or_else(|| {
            ParserError::ParseError("Expected space between cardinality and property".to_string())
        })?;

        let cardinality_str = &trimmed[..space_pos];
        let property_str = &trimmed[space_pos + 1..];

        let cardinality = cardinality_str.parse().map_err(|_| {
            ParserError::ParseError(format!("Invalid cardinality: {}", cardinality_str))
        })?;

        let property = self.parse_data_property_expression(property_str)?;

        Ok((cardinality, property))
    }

    /// Parse data property expression
    fn parse_data_property_expression(
        &mut self,
        input: &str,
    ) -> Result<DataPropertyExpression, ParserError> {
        let trimmed = input.trim();
        // Currently only supports simple named data properties
        let property_iri = self.resolve_iri(trimmed)?;
        let property = DataProperty::new(property_iri);
        Ok(DataPropertyExpression::DataProperty(property))
    }

    /// Parse data range
    fn parse_data_range(&mut self, input: &str) -> Result<DataRange, ParserError> {
        let trimmed = input.trim();

        if trimmed.starts_with("DataIntersectionOf(") {
            let content = &trimmed["DataIntersectionOf(".len()..trimmed.len() - 1];
            let ranges = self.parse_data_range_list(content)?;
            Ok(DataRange::DataIntersectionOf(ranges))
        } else if trimmed.starts_with("DataUnionOf(") {
            let content = &trimmed["DataUnionOf(".len()..trimmed.len() - 1];
            let ranges = self.parse_data_range_list(content)?;
            Ok(DataRange::DataUnionOf(ranges))
        } else if trimmed.starts_with("DataComplementOf(") {
            let content = &trimmed["DataComplementOf(".len()..trimmed.len() - 1];
            let range = Box::new(self.parse_data_range(content)?);
            Ok(DataRange::DataComplementOf(range))
        } else if trimmed.starts_with("DataOneOf(") {
            let content = &trimmed["DataOneOf(".len()..trimmed.len() - 1];
            let literals = self.parse_literal_list(content)?;
            Ok(DataRange::DataOneOf(literals))
        } else if trimmed.starts_with("DatatypeRestriction(") {
            let content = &trimmed["DatatypeRestriction(".len()..trimmed.len() - 1];
            let (datatype_iri, restrictions) = self.parse_datatype_restriction(content)?;
            Ok(DataRange::DatatypeRestriction(datatype_iri, restrictions))
        } else {
            // Simple datatype
            let datatype_iri = self.resolve_iri(trimmed)?;
            Ok(DataRange::Datatype(datatype_iri))
        }
    }

    /// Parse list of data ranges
    fn parse_data_range_list(&mut self, input: &str) -> Result<Vec<DataRange>, ParserError> {
        let mut ranges = Vec::new();
        let mut current = String::new();
        let mut paren_count = 0;

        for ch in input.chars() {
            if ch == '(' {
                paren_count += 1;
                current.push(ch);
            } else if ch == ')' {
                paren_count -= 1;
                current.push(ch);
                if paren_count == 0 {
                    if !current.trim().is_empty() {
                        ranges.push(self.parse_data_range(&current)?);
                    }
                    current.clear();
                }
            } else if ch.is_whitespace() && paren_count == 0 {
                if !current.trim().is_empty() {
                    ranges.push(self.parse_data_range(&current)?);
                    current.clear();
                }
            } else {
                current.push(ch);
            }
        }

        if !current.trim().is_empty() {
            ranges.push(self.parse_data_range(&current)?);
        }

        Ok(ranges)
    }

    /// Parse list of literals
    fn parse_literal_list(&mut self, input: &str) -> Result<Vec<Literal>, ParserError> {
        let mut literals = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for ch in input.chars() {
            if ch == '"' {
                in_quotes = !in_quotes;
                current.push(ch);
            } else if ch.is_whitespace() && !in_quotes {
                if !current.trim().is_empty() {
                    literals.push(self.parse_literal(&current)?);
                    current.clear();
                }
            } else {
                current.push(ch);
            }
        }

        if !current.trim().is_empty() {
            literals.push(self.parse_literal(&current)?);
        }

        Ok(literals)
    }

    /// Parse datatype restriction
    fn parse_datatype_restriction(
        &mut self,
        input: &str,
    ) -> Result<(IRI, Vec<FacetRestriction>), ParserError> {
        let trimmed = input.trim();
        let space_pos = trimmed.find(' ').ok_or_else(|| {
            ParserError::ParseError("Expected space between datatype and restrictions".to_string())
        })?;

        let datatype_str = &trimmed[..space_pos];
        let restrictions_str = &trimmed[space_pos + 1..];

        let datatype_iri = self.resolve_iri(datatype_str)?;
        let mut restrictions = Vec::new();

        // Parse restriction facets - for now, we'll support basic patterns
        let mut current = String::new();
        let mut paren_count = 0;

        for ch in restrictions_str.chars() {
            if ch == '(' {
                paren_count += 1;
                current.push(ch);
            } else if ch == ')' {
                paren_count -= 1;
                current.push(ch);
                if paren_count == 0 {
                    if !current.trim().is_empty() {
                        // Parse facet restriction like "minInclusive 18"
                        let facet_parts: Vec<&str> = current.trim().split_whitespace().collect();
                        if facet_parts.len() >= 2 {
                            let facet_iri = self.resolve_iri(facet_parts[0])?;
                            let literal = self.parse_literal(facet_parts[1])?;
                            restrictions.push(FacetRestriction::new(facet_iri, literal));
                        }
                    }
                    current.clear();
                }
            } else if ch.is_whitespace() && paren_count == 0 {
                if !current.trim().is_empty() {
                    let facet_parts: Vec<&str> = current.trim().split_whitespace().collect();
                    if facet_parts.len() >= 2 {
                        let facet_iri = self.resolve_iri(facet_parts[0])?;
                        let literal = self.parse_literal(facet_parts[1])?;
                        restrictions.push(FacetRestriction::new(facet_iri, literal));
                    }
                    current.clear();
                }
            } else {
                current.push(ch);
            }
        }

        Ok((datatype_iri, restrictions))
    }

    /// Parse a literal (string, typed, language-tagged)
    fn parse_literal(&mut self, input: &str) -> Result<Literal, ParserError> {
        let input = input.trim();

        // Check for typed literal (must check before language-tagged to avoid confusion)
        if input.starts_with('"') && input.contains("^^") {
            if let Some(type_pos) = input.find("^^") {
                let string_part = &input[..type_pos];
                let type_iri_str = &input[type_pos + 2..].trim();

                // Parse the quoted string
                if let Some(quote_end) = string_part.rfind('"') {
                    if let Some(quote_start) = string_part.find('"') {
                        let content = &string_part[quote_start + 1..quote_end];
                        let datatype_iri = self.resolve_iri(type_iri_str)?;
                        return Ok(Literal::typed(content, datatype_iri));
                    }
                }
            }
        }

        // Check for language-tagged string
        if input.starts_with('"') && input.contains('@') {
            if let Some(lang_pos) = input.rfind('@') {
                let string_part = &input[..lang_pos];
                let lang_part = &input[lang_pos + 1..];

                // Parse the quoted string
                if let Some(quote_end) = string_part.rfind('"') {
                    if let Some(quote_start) = string_part.find('"') {
                        let content = &string_part[quote_start + 1..quote_end];
                        return Ok(Literal::lang_tagged(content, lang_part));
                    }
                }
            }
        }

        // Check for simple string literal
        if input.starts_with('"') {
            if let Some(quote_end) = input.rfind('"') {
                if let Some(quote_start) = input.find('"') {
                    let content = &input[quote_start + 1..quote_end];
                    return Ok(Literal::simple(content));
                }
            }
        }

        // Check for boolean literals
        match input {
            "true" => return Ok(Literal::typed("true", "http://www.w3.org/2001/XMLSchema#boolean")),
            "false" => return Ok(Literal::typed("false", "http://www.w3.org/2001/XMLSchema#boolean")),
            _ => {}
        }

        // Check for integer literals
        if let Ok(int_value) = input.parse::<i64>() {
            return Ok(Literal::typed(int_value.to_string(), "http://www.w3.org/2001/XMLSchema#integer"));
        }

        // Check for float literals
        if let Ok(float_value) = input.parse::<f64>() {
            return Ok(Literal::typed(float_value.to_string(), "http://www.w3.org/2001/XMLSchema#double"));
        }

        // Try to parse as IRI (could be an individual IRI used as a value)
        if input.starts_with('<') && input.ends_with('>') {
            let iri_content = &input[1..input.len() - 1];
            let _iri = self.resolve_iri(iri_content)?;
            return Ok(Literal::simple(iri_content));
        }

        // Try to parse as prefixed name
        if !input.is_empty() && input.chars().next().unwrap().is_alphabetic() {
            return self.resolve_iri(input).map(|_iri| Literal::simple(input));
        }

        Err(ParserError::ParseError(format!("Invalid literal: {}", input)))
    }

    /// Parse AnnotationAssertion axiom
    fn parse_annotation_assertion(&mut self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        // Parse the annotation property (first token)
        let mut chars = content.chars();
        let mut property_str = String::new();
        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                break;
            }
            property_str.push(c);
        }

        // Skip whitespace and parse the subject (second token)
        while let Some(c) = chars.next() {
            if !c.is_whitespace() {
                break;
            }
        }

        let mut subject_str = String::new();
        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                break;
            }
            subject_str.push(c);
        }

        // The rest is the annotation value
        let value_str: String = chars.collect();

        if !property_str.is_empty() && !subject_str.is_empty() && !value_str.is_empty() {
            let property_iri = self.resolve_iri(&property_str)?;
            let subject_iri = self.resolve_iri(&subject_str)?;

            // Parse the annotation value (could be IRI or literal)
            let annotation_value = if value_str.trim().starts_with('"') {
                // It's a literal
                crate::entities::AnnotationValue::Literal(self.parse_literal(&value_str)?)
            } else {
                // It's an IRI
                let value_iri = self.resolve_iri(&value_str)?;
                crate::entities::AnnotationValue::IRI(value_iri)
            };

            let annotation_assertion = AnnotationAssertionAxiom::new(property_iri, subject_iri, annotation_value);
            ontology.add_axiom(Axiom::AnnotationAssertion(Box::new(annotation_assertion)))?;
        }
        Ok(())
    }

    /// Parse SubAnnotationPropertyOf axiom
    fn parse_sub_annotation_property_of(&mut self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let sub_prop_iri = self.resolve_iri(parts[0])?;
            let super_prop_iri = self.resolve_iri(parts[1])?;

            let sub_axiom = SubAnnotationPropertyOfAxiom::new(sub_prop_iri, super_prop_iri);
            ontology.add_axiom(Axiom::SubAnnotationPropertyOf(sub_axiom))?;
        }
        Ok(())
    }

    /// Parse AnnotationPropertyDomain axiom
    fn parse_annotation_property_domain(&mut self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let prop_iri = self.resolve_iri(parts[0])?;
            let domain_iri = self.resolve_iri(parts[1])?;

            let domain_axiom = AnnotationPropertyDomainAxiom::new(prop_iri, domain_iri);
            ontology.add_axiom(Axiom::AnnotationPropertyDomain(domain_axiom))?;
        }
        Ok(())
    }

    /// Parse AnnotationPropertyRange axiom
    fn parse_annotation_property_range(&mut self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let prop_iri = self.resolve_iri(parts[0])?;
            let range_iri = self.resolve_iri(parts[1])?;

            let range_axiom = AnnotationPropertyRangeAxiom::new(prop_iri, range_iri);
            ontology.add_axiom(Axiom::AnnotationPropertyRange(range_axiom))?;
        }
        Ok(())
    }

    /// Parse Import axiom
    fn parse_import(&mut self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let iri = self.resolve_iri(content.trim())?;
        let import_axiom = ImportAxiom::new(iri);
        ontology.add_axiom(Axiom::Import(import_axiom))?;
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
                return Err(crate::error::OwlError::ParseError(format!(
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
            assert!(
                ontology.classes().len() >= 3,
                "Should have parsed at least 3 classes"
            );
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
    fn test_hyphenated_prefix_parsing() {
        let hyphen_test = r#"
Prefix(univ-bench:=<http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)

Ontology(<http://example.org/test>

Declaration(Class(univ-bench:University))

)
"#;

        let parser = OwlFunctionalSyntaxParser::new();
        let result = parser.parse_str(hyphen_test);

        assert!(
            result.is_ok(),
            "Hyphenated prefix parsing failed: {:?}",
            result
        );

        if let Ok(ontology) = result {
            assert_eq!(ontology.classes().len(), 1, "Should have parsed 1 class");
        }
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
            ..Default::default()
        };

        let parser = OwlFunctionalSyntaxParser::with_config(config);
        assert_eq!(parser.format_name(), "OWL Functional Syntax");
    }

    #[test]
    fn test_comprehensive_owl_functional_parsing() {
        let comprehensive_owl = r#"
Prefix(:=<http://example.org/test#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)
Prefix(rdf:=<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)
Prefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)
Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)

Ontology(<http://example.org/test>

Declaration(Class(:Person))
Declaration(Class(:Student))
Declaration(Class(:Professor))
Declaration(Class(:Course))
Declaration(Class(:EnrolledStudent))
Declaration(ObjectProperty(:hasEnrollment))
Declaration(ObjectProperty(:teaches))
Declaration(DataProperty(:hasAge))
Declaration(NamedIndividual(:John))
Declaration(NamedIndividual(:Mary))
Declaration(NamedIndividual(:CS101))
Declaration(NamedIndividual(:JohnDoe))

SubClassOf(:Student :Person)
SubClassOf(:Professor :Person)
EquivalentClasses(:Student :EnrolledStudent)
FunctionalObjectProperty(:hasEnrollment)
SymmetricObjectProperty(:teaches)
TransitiveObjectProperty(:teaches)
ObjectPropertyDomain(:hasEnrollment :Student)
ObjectPropertyRange(:hasEnrollment :Course)
DataPropertyDomain(:hasAge :Person)
DataPropertyRange(:hasAge xsd:integer)
FunctionalDataProperty(:hasAge)
ClassAssertion(:Student :John)
ClassAssertion(:Professor :Mary)
ObjectPropertyAssertion(:teaches :Mary :CS101)
SameIndividual(:John :JohnDoe)
DifferentIndividuals(:John :Mary)

)
"#;

        let parser = OwlFunctionalSyntaxParser::new();
        let result = parser.parse_str(comprehensive_owl);

        assert!(result.is_ok(), "Comprehensive parsing failed: {:?}", result);

        if let Ok(ontology) = result {
            // Check that we parsed the expected number of entities
            assert!(
                ontology.classes().len() >= 4,
                "Should have parsed at least 4 classes"
            );
            assert!(
                ontology.object_properties().len() >= 2,
                "Should have parsed at least 2 object properties"
            );
            assert!(
                ontology.data_properties().len() >= 1,
                "Should have parsed at least 1 data property"
            );
            assert!(
                ontology.named_individuals().len() >= 3,
                "Should have parsed at least 3 named individuals"
            );

            // Check that axioms were added
            println!("Total axioms parsed: {}", ontology.axioms().len());
            assert!(
                ontology.axioms().len() >= 10,
                "Should have parsed multiple axioms"
            );
        }
    }
}
