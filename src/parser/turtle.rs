//! Turtle/TTL format parser for OWL2 ontologies
//! 
//! Implements parsing of the Terse RDF Triple Language format.

use crate::parser::{ParserConfig, OntologyParser};
use crate::ontology::Ontology;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::entities::*;
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
        
        TurtleParser {
            config,
            prefixes,
        }
    }
    
    /// Parse Turtle content and build an ontology
    fn parse_content(&mut self, content: &str) -> OwlResult<Ontology> {
        let mut ontology = Ontology::new();
        
        // Simple line-based Turtle parser for basic constructs
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue; // Skip empty lines and comments
            }
            
            // Parse prefix declarations
            if line.starts_with("@prefix") {
                if let Some((prefix, namespace)) = self.parse_prefix_declaration(line) {
                    self.prefixes.insert(prefix, namespace);
                }
                continue;
            }
            
            // Parse triples (simplified)
            if let Some((subject, predicate, object)) = self.parse_triple(line) {
                self.process_triple(&mut ontology, subject, predicate, object)?;
            }
        }
        
        if self.config.strict_validation {
            self.validate_ontology(&ontology)?;
        }
        
        Ok(ontology)
    }
    
    /// Parse a prefix declaration
    fn parse_prefix_declaration(&self, line: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "@prefix" {
            let prefix = parts[1].trim_end_matches(':');
            let namespace = parts[2].trim_matches('<').trim_matches('>');
            Some((prefix.to_string(), namespace.to_string()))
        } else {
            None
        }
    }
    
    /// Parse a simple triple (simplified parser)
    fn parse_triple(&self, line: &str) -> Option<(IRI, IRI, ObjectValue)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let subject = self.parse_curie_or_iri(parts[0]).ok()?;
            
            // Handle "a" keyword for rdf:type
            let predicate = if parts[1] == "a" {
                IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap()
            } else {
                self.parse_curie_or_iri(parts[1]).ok()?
            };
            
            // Simple object parsing (just IRIs for now)
            let object_str = parts[2].trim_end_matches(['.', ';', ',']);
            let object = self.parse_curie_or_iri(object_str).ok()?;
            
            Some((subject, predicate, ObjectValue::IRI(object)))
        } else {
            None
        }
    }
    
    /// Parse a CURIE or IRI
    fn parse_curie_or_iri(&self, s: &str) -> OwlResult<IRI> {
        if s.starts_with('<') && s.ends_with('>') {
            // Full IRI
            IRI::new(&s[1..s.len()-1])
        } else if let Some(colon_pos) = s.find(':') {
            // CURIE
            let prefix = &s[..colon_pos];
            let local = &s[colon_pos + 1..];
            
            if let Some(namespace) = self.prefixes.get(prefix) {
                IRI::new(&format!("{}{}", namespace, local))
            } else {
                // Treat as full IRI
                IRI::new(s)
            }
        } else {
            // Treat as full IRI
            IRI::new(s)
        }
    }
    
    /// Process a single triple
    fn process_triple(&self, ontology: &mut Ontology, subject: IRI, predicate: IRI, object: ObjectValue) -> OwlResult<()> {
        // Check for ontology declaration
        if predicate.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
            if let ObjectValue::IRI(obj_iri) = &object {
                match obj_iri.as_str() {
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
                        ontology.add_named_individual(NamedIndividual::new(subject))?;
                    }
                    _ => {}
                }
            }
        }
        
        // Handle imports
        if predicate.as_str() == "http://www.w3.org/2002/07/owl#imports" {
            if let ObjectValue::IRI(import_iri) = object {
                ontology.add_import(import_iri);
            }
        }
        
        Ok(())
    }
    
    /// Validate the parsed ontology
    fn validate_ontology(&self, ontology: &Ontology) -> OwlResult<()> {
        // Basic validation checks - allow ontologies with only imports
        if ontology.classes().is_empty() && ontology.object_properties().is_empty() 
            && ontology.data_properties().is_empty() && ontology.named_individuals().is_empty()
            && ontology.imports().is_empty() {
            return Err(crate::error::OwlError::ValidationError("Ontology contains no entities or imports".to_string()));
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
                return Err(crate::error::OwlError::ParseError(format!("File size exceeds maximum allowed size: {} bytes", self.config.max_file_size)));
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

/// Object values in Turtle (IRI, Literal, or Blank Node)
#[derive(Debug, Clone)]
enum ObjectValue {
    IRI(IRI),
    Literal(Literal),
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
        
        let mut parser = TurtleParser::new();
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
        
        let mut parser = TurtleParser::new();
        let ontology = parser.parse_str(turtle_content).unwrap();
        
        assert_eq!(ontology.imports().len(), 1);
    }
}