//! Turtle/TTL format parser for OWL2 ontologies
//!
//! Implements parsing of the Terse RDF Triple Language format.

use crate::axioms::*;
use crate::entities::*;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::parser::{OntologyParser, ParserConfig};
use std::collections::HashMap;
use std::path::Path;

/// Turtle format parser
pub struct TurtleParser {
    config: ParserConfig,
    prefixes: HashMap<String, String>,
}

impl TurtleParser {
    /// Create a new Turtle parser with default configuration
    pub fn new() -> Self {
        Self::with_config(ParserConfig::default())
    }

    /// Create a new Turtle parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        let mut prefixes = HashMap::new();
        for (prefix, namespace) in &config.prefixes {
            prefixes.insert(prefix.clone(), namespace.clone());
        }

        // Add standard OWL/RDF prefixes by default for robustness
        prefixes
            .entry("owl".to_string())
            .or_insert_with(|| "http://www.w3.org/2002/07/owl#".to_string());
        prefixes
            .entry("rdf".to_string())
            .or_insert_with(|| "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string());
        prefixes
            .entry("rdfs".to_string())
            .or_insert_with(|| "http://www.w3.org/2000/01/rdf-schema#".to_string());
        prefixes
            .entry("xsd".to_string())
            .or_insert_with(|| "http://www.w3.org/2001/XMLSchema#".to_string());

        TurtleParser { config, prefixes }
    }

    /// Parse Turtle content and build an ontology
    fn parse_content(&mut self, content: &str) -> OwlResult<Ontology> {
        if self.config.strict_validation && content.trim().is_empty() {
            return Err(crate::error::OwlError::ValidationError(
                "Ontology contains no entities or imports".to_string(),
            ));
        }
        let mut ontology = Ontology::new();

        // Simple line-based Turtle parser for basic constructs
        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue; // Skip empty lines and comments
            }

            // Parse prefix declarations
            if line.starts_with("@prefix") {
                let (prefix, namespace) = self.parse_prefix_declaration(line)?;
                self.prefixes.insert(prefix, namespace);
                continue;
            }

            // Strip inline comments for validation
            let stmt = line.split('#').next().unwrap_or("").trim_end();
            if stmt.is_empty() {
                continue;
            }

            // In strict mode, require statements to end with a dot or continue characters
            if self.config.strict_validation
                && !(stmt.ends_with('.') || stmt.ends_with(';') || stmt.ends_with(','))
            {
                return Err(crate::error::OwlError::ParseError(
                    "Expected '.' at end of statement".to_string(),
                ));
            }

            // Parse triples (simplified)
            if let Some((subject, predicate, object)) = self.parse_triple(stmt) {
                self.process_triple(&mut ontology, subject, predicate, object)?;
            } else {
                // Leniently skip lines we can't parse (multi-line constructs), strictness enforced by other checks
                continue;
            }
        }

        if self.config.strict_validation {
            self.validate_ontology(&ontology)?;
        }

        Ok(ontology)
    }

    /// Parse a prefix declaration
    fn parse_prefix_declaration(&self, line: &str) -> OwlResult<(String, String)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "@prefix" {
            let prefix_token = parts[1];
            let ns_token = parts[2];

            // Validate prefix token ends with ':'
            if !prefix_token.ends_with(':') {
                return Err(crate::error::OwlError::ParseError(
                    "Malformed @prefix: missing trailing ':'".to_string(),
                ));
            }
            let prefix = prefix_token.trim_end_matches(':');

            // Namespace must be enclosed in <>
            if !(ns_token.starts_with('<') && ns_token.ends_with('>')) {
                return Err(crate::error::OwlError::ParseError(
                    "Malformed @prefix: namespace must be <...>".to_string(),
                ));
            }
            let namespace = ns_token.trim_matches('<').trim_matches('>');
            return Ok((prefix.to_string(), namespace.to_string()));
        }
        if self.config.strict_validation {
            return Err(crate::error::OwlError::ParseError(
                "Malformed @prefix declaration".to_string(),
            ));
        }
        Err(crate::error::OwlError::ParseError(
            "Malformed @prefix declaration".to_string(),
        ))
    }

    /// Parse a Turtle triple with support for complex constructs
    fn parse_triple(&self, line: &str) -> Option<(IRI, IRI, ObjectValue)> {
        let line = line.trim_end_matches(['.', ';', ',']);
        let tokens = self.tokenize_turtle_line(line);

        if tokens.len() < 3 {
            return None;
        }

        let subject = self.parse_subject(&tokens[0])?;
        let predicate = self.parse_predicate(&tokens[1])?;
        let (object, remaining_tokens) = self.parse_object(&tokens[2..])?;

        Some((subject, predicate, object))
    }

    /// Tokenize a Turtle line handling quotes and nested structures
    fn tokenize_turtle_line(&self, line: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut in_blank_node = false;
        let mut bracket_depth = 0;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '"' => {
                    in_quotes = !in_quotes;
                    current.push(c);
                }
                '[' if !in_quotes => {
                    if bracket_depth == 0 {
                        if !current.trim().is_empty() {
                            tokens.push(current.trim().to_string());
                            current.clear();
                        }
                        in_blank_node = true;
                    }
                    bracket_depth += 1;
                    current.push(c);
                }
                ']' if !in_quotes && in_blank_node => {
                    bracket_depth -= 1;
                    current.push(c);
                    if bracket_depth == 0 {
                        tokens.push(current.clone());
                        current.clear();
                        in_blank_node = false;
                    }
                }
                '(' if !in_quotes => {
                    bracket_depth += 1;
                    current.push(c);
                }
                ')' if !in_quotes => {
                    bracket_depth -= 1;
                    current.push(c);
                }
                ' ' | '\t' if !in_quotes && bracket_depth == 0 => {
                    if !current.trim().is_empty() {
                        tokens.push(current.trim().to_string());
                        current.clear();
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if !current.trim().is_empty() {
            tokens.push(current.trim().to_string());
        }

        tokens
    }

    /// Parse a subject (IRI or blank node)
    fn parse_subject(&self, token: &str) -> Option<IRI> {
        if let Some(stripped) = token.strip_prefix("_:") {
            // Blank node - generate temporary IRI for processing
            Some(IRI::new(format!("http://blank.node/{}", stripped)).unwrap())
        } else {
            self.parse_curie_or_iri(token).ok()
        }
    }

    /// Parse a predicate (handle "a" keyword)
    fn parse_predicate(&self, token: &str) -> Option<IRI> {
        if token == "a" {
            Some(IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap())
        } else {
            self.parse_curie_or_iri(token).ok()
        }
    }

    /// Parse an object with support for complex structures
    fn parse_object(&self, tokens: &[String]) -> Option<(ObjectValue, Vec<String>)> {
        if tokens.is_empty() {
            return None;
        }

        let first_token = &tokens[0];

        if let Some(stripped) = first_token.strip_prefix("_:") {
            // Blank node
            Some((ObjectValue::BlankNode(stripped.to_string()), tokens[1..].to_vec()))
        } else if first_token.starts_with('"') {
            // Literal
            let literal = self.parse_literal(first_token)?;
            Some((ObjectValue::Literal(literal), tokens[1..].to_vec()))
        } else if first_token.starts_with('[') {
            // Blank node with properties (nested structure)
            let (nested_object, consumed) = self.parse_blank_node_structure(first_token)?;
            Some((
                ObjectValue::Nested(Box::new(nested_object)),
                tokens[consumed..].to_vec(),
            ))
        } else if first_token.starts_with('(') {
            // Collection (list)
            let (list_items, consumed) = self.parse_collection(&tokens)?;
            let nested_object = NestedObject {
                object_type: "Collection".to_string(),
                properties: HashMap::new(),
                list_items,
            };
            Some((
                ObjectValue::Nested(Box::new(nested_object)),
                tokens[consumed..].to_vec(),
            ))
        } else {
            // Simple IRI
            let iri = self.parse_curie_or_iri(first_token).ok()?;
            Some((ObjectValue::IRI(iri), tokens[1..].to_vec()))
        }
    }

    /// Parse a literal value
    fn parse_literal(&self, token: &str) -> Option<Literal> {
        if !token.starts_with('"') || !token.ends_with('"') {
            return None;
        }

        let content = &token[1..token.len() - 1];
        let literal = Literal::simple(content.to_string());
        Some(literal)
    }

    /// Parse blank node structure [ ... ]
    fn parse_blank_node_structure(&self, content: &str) -> Option<(NestedObject, usize)> {
        // For now, return a simple placeholder
        // In a full implementation, this would parse the nested structure
        let nested_object = NestedObject {
            object_type: "BlankNode".to_string(),
            properties: HashMap::new(),
            list_items: Vec::new(),
        };
        Some((nested_object, 1))
    }

    /// Parse collection ( ... )
    fn parse_collection(&self, tokens: &[String]) -> Option<(Vec<ObjectValue>, usize)> {
        let mut items = Vec::new();
        let mut consumed = 0;

        for token in tokens {
            consumed += 1;
            if token == ")" {
                break;
            }

            if token != "(" {
                if let Ok(iri) = self.parse_curie_or_iri(token) {
                    items.push(ObjectValue::IRI(iri));
                }
            }
        }

        Some((items, consumed))
    }

    /// Parse a CURIE or IRI
    fn parse_curie_or_iri(&self, s: &str) -> OwlResult<IRI> {
        if s.starts_with('<') && s.ends_with('>') {
            // Full IRI
            IRI::new(&s[1..s.len() - 1])
        } else if let Some(colon_pos) = s.find(':') {
            // CURIE
            let prefix = &s[..colon_pos];
            let local = &s[colon_pos + 1..];

            if let Some(namespace) = self.prefixes.get(prefix) {
                IRI::new(&format!("{namespace}{local}"))
            } else {
                if self.config.strict_validation {
                    Err(crate::error::OwlError::ParseError(format!(
                        "Undefined prefix: {}",
                        prefix
                    )))
                } else {
                    // Treat as full IRI in non-strict mode
                    IRI::new(s)
                }
            }
        } else {
            // Treat as full IRI
            IRI::new(s)
        }
    }

    /// Process a single triple with comprehensive OWL2 support
    fn process_triple(
        &self,
        ontology: &mut Ontology,
        subject: IRI,
        predicate: IRI,
        object: ObjectValue,
    ) -> OwlResult<()> {
        match predicate.as_str() {
            // RDF type declarations (entity declarations)
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" => {
                self.process_type_declaration(ontology, subject, object)?;
            }

            // RDFS subclass relationships
            "http://www.w3.org/2000/01/rdf-schema#subClassOf" => {
                if let ObjectValue::IRI(super_class_iri) = object {
                    let subclass_axiom = SubClassOfAxiom::new(
                        ClassExpression::Class(Class::new(subject)),
                        ClassExpression::Class(Class::new(super_class_iri)),
                    );
                    ontology.add_axiom(Axiom::SubClassOf(subclass_axiom))?;
                }
            }

            // OWL equivalent classes
            "http://www.w3.org/2002/07/owl#equivalentClass" => {
                if let ObjectValue::IRI(equiv_class_iri) = object {
                    let equiv_axiom =
                        EquivalentClassesAxiom::new(vec![subject.clone(), equiv_class_iri.clone()]);
                    ontology.add_axiom(Axiom::EquivalentClasses(equiv_axiom))?;
                } else if let ObjectValue::Nested(nested) = object {
                    // Handle complex equivalent class expressions (restrictions, intersections, etc.)
                    if let Some(class_expr) = self.parse_nested_class_expression(&nested) {
                        // For complex expressions, we need to use two SubClassOf axioms
                        let subclass_axiom1 = SubClassOfAxiom::new(
                            ClassExpression::Class(Class::new(subject.clone())),
                            class_expr.clone(),
                        );
                        let subclass_axiom2 = SubClassOfAxiom::new(
                            class_expr,
                            ClassExpression::Class(Class::new(subject.clone())),
                        );
                        ontology.add_axiom(Axiom::SubClassOf(subclass_axiom1))?;
                        ontology.add_axiom(Axiom::SubClassOf(subclass_axiom2))?;
                    }
                }
            }

            // OWL disjoint classes
            "http://www.w3.org/2002/07/owl#disjointWith" => {
                if let ObjectValue::IRI(disjoint_class_iri) = object {
                    let disjoint_axiom = DisjointClassesAxiom::new(vec![
                        subject.clone(),
                        disjoint_class_iri.clone(),
                    ]);
                    ontology.add_axiom(Axiom::DisjointClasses(disjoint_axiom))?;
                }
            }

            // OWL property characteristics
            "http://www.w3.org/2002/07/owl#equivalentProperty" => {
                if let ObjectValue::IRI(equiv_prop_iri) = object {
                    let equiv_axiom = EquivalentObjectPropertiesAxiom::new(vec![
                        subject.clone(),
                        equiv_prop_iri.clone(),
                    ]);
                    ontology.add_axiom(Axiom::EquivalentObjectProperties(equiv_axiom))?;
                }
            }

            "http://www.w3.org/2002/07/owl#inverseOf" => {
                if let ObjectValue::IRI(inverse_prop_iri) = object {
                    let inverse_axiom = InverseObjectPropertiesAxiom::new(
                        ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(subject)),
                        ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(
                            inverse_prop_iri,
                        )),
                    );
                    ontology.add_axiom(Axiom::InverseObjectProperties(inverse_axiom))?;
                }
            }

            // Property domain and range
            "http://www.w3.org/2000/01/rdf-schema#domain" => {
                if let ObjectValue::IRI(domain_iri) = object {
                    // Add domain as a subclass axiom: ∀p.Domain ⊑ Domain
                    let domain_class = ClassExpression::Class(Class::new(domain_iri));
                    let property_expr = ObjectPropertyExpression::ObjectProperty(
                        ObjectProperty::new(subject.clone()),
                    );
                    let restriction = ClassExpression::ObjectAllValuesFrom(
                        Box::new(property_expr),
                        Box::new(domain_class),
                    );

                    let subclass_axiom = SubClassOfAxiom::new(
                        ClassExpression::ObjectSomeValuesFrom(
                            Box::new(ObjectPropertyExpression::ObjectProperty(
                                ObjectProperty::new(subject),
                            )),
                            Box::new(ClassExpression::Class(Class::new(
                                IRI::new("http://www.w3.org/2002/07/owl#Thing").unwrap(),
                            ))),
                        ),
                        restriction,
                    );
                    ontology.add_axiom(Axiom::SubClassOf(subclass_axiom))?;
                }
            }

            "http://www.w3.org/2000/01/rdf-schema#range" => {
                if let ObjectValue::IRI(range_iri) = object {
                    // Add range constraint: ∀p.∃range ⊑ Range
                    let range_class = ClassExpression::Class(Class::new(range_iri));
                    let property_expr = ObjectPropertyExpression::ObjectProperty(
                        ObjectProperty::new(subject.clone()),
                    );

                    let subclass_axiom = SubClassOfAxiom::new(
                        ClassExpression::ObjectAllValuesFrom(
                            Box::new(property_expr),
                            Box::new(range_class),
                        ),
                        ClassExpression::Class(Class::new(
                            IRI::new("http://www.w3.org/2002/07/owl#Thing").unwrap(),
                        )),
                    );
                    ontology.add_axiom(Axiom::SubClassOf(subclass_axiom))?;
                }
            }

            // OWL imports
            "http://www.w3.org/2002/07/owl#imports" => {
                if let ObjectValue::IRI(import_iri) = object {
                    ontology.add_import(import_iri);
                }
            }

            // Property assertions (individual relationships)
            _ => {
                // Handle as property assertion between individuals
                self.process_property_assertion(ontology, subject, predicate, object)?;
            }
        }

        Ok(())
    }

    /// Process RDF type declarations
    fn process_type_declaration(
        &self,
        ontology: &mut Ontology,
        subject: IRI,
        object: ObjectValue,
    ) -> OwlResult<()> {
        if let ObjectValue::IRI(type_iri) = object {
            match type_iri.as_str() {
                "http://www.w3.org/2002/07/owl#Ontology" => {
                    ontology.set_iri(subject);
                }
                "http://www.w3.org/2002/07/owl#Class" => {
                    ontology.add_class(Class::new(subject))?;
                }
                "http://www.w3.org/2002/07/owl#ObjectProperty" => {
                    ontology.add_object_property(ObjectProperty::new(subject))?;
                }
                "http://www.w3.org/2002/07/owl#DataProperty" => {
                    ontology.add_data_property(DataProperty::new(subject))?;
                }
                "http://www.w3.org/2002/07/owl#NamedIndividual" => {
                    let individual = NamedIndividual::new(subject.clone());
                    ontology.add_named_individual(individual.clone())?;

                    // Create class assertion
                    let class_assertion = ClassAssertionAxiom::new(
                        subject.clone(),
                        ClassExpression::Class(Class::new(subject.clone())),
                    );
                    ontology.add_axiom(Axiom::ClassAssertion(class_assertion))?;
                }
                // Handle other types as potential named individuals
                _ => {
                    // Add as individual and create class assertion
                    let individual = NamedIndividual::new(subject.clone());
                    ontology.add_named_individual(individual.clone())?;

                    let class_assertion = ClassAssertionAxiom::new(
                        subject.clone(),
                        ClassExpression::Class(Class::new(type_iri)),
                    );
                    ontology.add_axiom(Axiom::ClassAssertion(class_assertion))?;
                }
            }
        }
        Ok(())
    }

    /// Process property assertions between individuals
    fn process_property_assertion(
        &self,
        ontology: &mut Ontology,
        subject: IRI,
        predicate: IRI,
        object: ObjectValue,
    ) -> OwlResult<()> {
        // Create or ensure subject individual exists
        let subject_individual = NamedIndividual::new(subject.clone());
        ontology.add_named_individual(subject_individual.clone())?;

        match object {
            ObjectValue::IRI(object_iri) => {
                // Object property assertion
                let object_individual = NamedIndividual::new(object_iri.clone());
                ontology.add_named_individual(object_individual.clone())?;

                let property_assertion = PropertyAssertionAxiom::new(
                    subject_individual.iri().clone(),
                    predicate,
                    object_individual.iri().clone(),
                );
                ontology.add_axiom(Axiom::PropertyAssertion(property_assertion))?;
            }
            ObjectValue::Literal(_literal) => {
                // Data property assertion - skip for now as PropertyAssertionAxiom expects IRIs
                // TODO: Implement proper data property assertion with literal values
            }
            ObjectValue::BlankNode(_) => {
                // Handle blank nodes as anonymous individuals if needed
                // For now, skip blank node object assertions
            }
            ObjectValue::Nested(_) => {
                // Complex nested objects - handle based on structure
                // For now, skip nested object assertions
            }
        }
        Ok(())
    }

    /// Parse nested class expressions from complex structures
    fn parse_nested_class_expression(&self, nested: &NestedObject) -> Option<ClassExpression> {
        match nested.object_type.as_str() {
            "Collection" => {
                // Handle intersectionOf, unionOf, oneOf
                if !nested.list_items.is_empty() {
                    // Default to intersection for collections
                    let classes: Vec<ClassExpression> = nested
                        .list_items
                        .iter()
                        .filter_map(|item| {
                            if let ObjectValue::IRI(iri) = item {
                                Some(ClassExpression::Class(Class::new(iri.clone())))
                            } else {
                                None
                            }
                        })
                        .collect();

                    if classes.len() >= 2 {
                        return Some(ClassExpression::ObjectIntersectionOf(classes));
                    }
                }
                None
            }
            "BlankNode" => {
                // Check for restriction patterns in properties
                if let Some(on_property) = nested
                    .properties
                    .get("http://www.w3.org/2002/07/owl#onProperty")
                {
                    if let ObjectValue::IRI(prop_iri) = on_property {
                        let property_expr = ObjectPropertyExpression::ObjectProperty(
                            ObjectProperty::new(prop_iri.clone()),
                        );

                        // Check for someValuesFrom
                        if let Some(some_values) = nested
                            .properties
                            .get("http://www.w3.org/2002/07/owl#someValuesFrom")
                        {
                            if let ObjectValue::IRI(range_iri) = some_values {
                                return Some(ClassExpression::ObjectSomeValuesFrom(
                                    Box::new(property_expr),
                                    Box::new(ClassExpression::Class(Class::new(range_iri.clone()))),
                                ));
                            }
                        }

                        // Check for allValuesFrom
                        if let Some(all_values) = nested
                            .properties
                            .get("http://www.w3.org/2002/07/owl#allValuesFrom")
                        {
                            if let ObjectValue::IRI(range_iri) = all_values {
                                return Some(ClassExpression::ObjectAllValuesFrom(
                                    Box::new(property_expr),
                                    Box::new(ClassExpression::Class(Class::new(range_iri.clone()))),
                                ));
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Validate the parsed ontology
    fn validate_ontology(&self, ontology: &Ontology) -> OwlResult<()> {
        // Basic validation checks - allow ontologies with only imports
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

impl OntologyParser for TurtleParser {
    fn parse_str(&self, content: &str) -> OwlResult<Ontology> {
        // Create a mutable copy for parsing
        let mut parser_copy = TurtleParser::with_config(self.config.clone());
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
        "Turtle"
    }
}

impl Default for TurtleParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Object values in Turtle (IRI, Literal, Blank Node, or nested structure)
#[derive(Debug, Clone)]
enum ObjectValue {
    IRI(IRI),
    Literal(Literal),
    BlankNode(String),
    /// For complex nested structures like restrictions, intersections, etc.
    Nested(Box<NestedObject>),
}

/// Complex nested objects in Turtle (restrictions, class expressions, etc.)
#[derive(Debug, Clone)]
struct NestedObject {
    object_type: String,
    properties: HashMap<String, ObjectValue>,
    /// For list-like structures (intersectionOf, oneOf, etc.)
    list_items: Vec<ObjectValue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_turtle_parsing() {
        let turtle_content = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:Person a owl:Class .
ex:Animal a owl:Class .
ex:hasParent a owl:ObjectProperty .
"#;

        let parser = TurtleParser::new();
        let ontology = parser.parse_str(turtle_content).unwrap();

        assert_eq!(ontology.classes().len(), 2);
        assert_eq!(ontology.object_properties().len(), 1);
    }

    #[test]
    fn test_turtle_with_imports() {
        let turtle_content = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:MyOntology a owl:Ontology .
ex:MyOntology owl:imports <http://example.org/other-ontology> .
"#;

        let parser = TurtleParser::new();
        let ontology = parser.parse_str(turtle_content).unwrap();

        assert_eq!(ontology.imports().len(), 1);
    }
}
