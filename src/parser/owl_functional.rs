//! OWL Functional Syntax parser for OWL2 ontologies
//!
//! Implements parsing of the OWL2 Functional Syntax serialization format.

use crate::axioms::*;
use crate::entities::*;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::parser::{OntologyParser, ParserConfig};
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
        }

        Ok(())
    }

    /// Parse axioms
    fn parse_axioms(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
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

    /// Parse EquivalentClasses axiom
    fn parse_equivalent_classes(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let class_iris = self.parse_iri_list(content)?;
        if class_iris.len() >= 2 {
            let classes: Vec<IRI> = class_iris;
            let equiv_axiom = EquivalentClassesAxiom::new(classes);
            ontology.add_axiom(Axiom::EquivalentClasses(equiv_axiom))?;
        }
        Ok(())
    }

    /// Parse DisjointClasses axiom
    fn parse_disjoint_classes(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let class_iris = self.parse_iri_list(content)?;
        if class_iris.len() >= 2 {
            let classes: Vec<IRI> = class_iris;
            let disjoint_axiom = DisjointClassesAxiom::new(classes);
            ontology.add_axiom(Axiom::DisjointClasses(disjoint_axiom))?;
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
            ontology.add_axiom(Axiom::DisjointClasses(disjoint_axiom))?;
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
            ontology.add_axiom(Axiom::SubObjectProperty(sub_axiom))?;
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
            ontology.add_axiom(Axiom::EquivalentObjectProperties(equiv_axiom))?;
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
            ontology.add_axiom(Axiom::DisjointObjectProperties(disjoint_axiom))?;
        }
        Ok(())
    }

    /// Parse ObjectPropertyDomain axiom
    fn parse_object_property_domain(
        &self,
        content: &str,
        _ontology: &mut Ontology,
    ) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let _prop_iri = self.resolve_iri(parts[0])?;
            let _domain_iri = self.resolve_iri(parts[1])?;

            // For now, skip ObjectPropertyDomain as it's not directly supported
            // TODO: Implement using SubClassOf with restrictions
        }
        Ok(())
    }

    /// Parse ObjectPropertyRange axiom
    fn parse_object_property_range(&self, content: &str, _ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let _prop_iri = self.resolve_iri(parts[0])?;
            let _range_iri = self.resolve_iri(parts[1])?;

            // For now, skip ObjectPropertyRange as it's not directly supported
            // TODO: Implement using SubClassOf with restrictions
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
                ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(prop1_iri)),
                ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(prop2_iri)),
            );
            ontology.add_axiom(Axiom::InverseObjectProperties(inverse_axiom))?;
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
        ontology.add_axiom(Axiom::FunctionalProperty(functional_axiom))?;
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
        ontology.add_axiom(Axiom::InverseFunctionalProperty(inverse_functional_axiom))?;
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
        ontology.add_axiom(Axiom::ReflexiveProperty(reflexive_axiom))?;
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
        ontology.add_axiom(Axiom::IrreflexiveProperty(irreflexive_axiom))?;
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
        ontology.add_axiom(Axiom::SymmetricProperty(symmetric_axiom))?;
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
        ontology.add_axiom(Axiom::AsymmetricProperty(asymmetric_axiom))?;
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
        ontology.add_axiom(Axiom::TransitiveProperty(transitive_axiom))?;
        Ok(())
    }

    /// Parse SubDataPropertyOf axiom
    fn parse_sub_data_property_of(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            let sub_prop_iri = self.resolve_iri(parts[0])?;
            let super_prop_iri = self.resolve_iri(parts[1])?;

            let sub_axiom = SubDataPropertyAxiom::new(sub_prop_iri, super_prop_iri);
            ontology.add_axiom(Axiom::SubDataProperty(sub_axiom))?;
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
            ontology.add_axiom(Axiom::EquivalentDataProperties(equiv_axiom))?;
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
            ontology.add_axiom(Axiom::DisjointDataProperties(disjoint_axiom))?;
        }
        Ok(())
    }

    /// Parse DataPropertyDomain axiom
    fn parse_data_property_domain(&self, _content: &str, _ontology: &mut Ontology) -> OwlResult<()> {
        // For now, skip DataPropertyDomain as it's not directly supported
        // TODO: Implement using proper restrictions
        Ok(())
    }

    /// Parse DataPropertyRange axiom
    fn parse_data_property_range(&self, _content: &str, _ontology: &mut Ontology) -> OwlResult<()> {
        // For now, skip DataPropertyRange as it's not directly supported
        // TODO: Implement using proper restrictions
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
            ontology.add_axiom(Axiom::PropertyAssertion(assertion))?;
        }
        Ok(())
    }

    /// Parse DataPropertyAssertion axiom
    fn parse_data_property_assertion(
        &self,
        _content: &str,
        _ontology: &mut Ontology,
    ) -> OwlResult<()> {
        // For now, skip data property assertions as they need literal handling
        // TODO: Implement proper literal parsing
        Ok(())
    }

    /// Parse NegativeObjectPropertyAssertion axiom
    fn parse_negative_object_property_assertion(
        &self,
        _content: &str,
        _ontology: &mut Ontology,
    ) -> OwlResult<()> {
        // TODO: Implement negative object property assertion
        Ok(())
    }

    /// Parse NegativeDataPropertyAssertion axiom
    fn parse_negative_data_property_assertion(
        &self,
        _content: &str,
        _ontology: &mut Ontology,
    ) -> OwlResult<()> {
        // TODO: Implement negative data property assertion
        Ok(())
    }

    /// Parse SameIndividual axiom
    fn parse_same_individual(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let individual_iris = self.parse_iri_list(content)?;
        if individual_iris.len() >= 2 {
            let same_axiom = SameIndividualAxiom::new(individual_iris);
            ontology.add_axiom(Axiom::SameIndividual(same_axiom))?;
        }
        Ok(())
    }

    /// Parse DifferentIndividuals axiom
    fn parse_different_individuals(&self, content: &str, ontology: &mut Ontology) -> OwlResult<()> {
        let individual_iris = self.parse_iri_list(content)?;
        if individual_iris.len() >= 2 {
            let different_axiom = DifferentIndividualsAxiom::new(individual_iris);
            ontology.add_axiom(Axiom::DifferentIndividuals(different_axiom))?;
        }
        Ok(())
    }

    /// Parse HasKey axiom
    fn parse_has_key(&self, _content: &str, _ontology: &mut Ontology) -> OwlResult<()> {
        // TODO: Implement HasKey axiom
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
