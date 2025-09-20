//! OWL2 Reasoning Engine
//!
//! Provides reasoning capabilities for OWL2 ontologies including
//! tableaux-based reasoning, rule-based inference, and query answering.

pub mod classification;
pub mod consistency;
pub mod query;
pub mod rules;
pub mod simple;
pub mod tableaux;

pub use classification::*;
pub use consistency::*;
pub use query::*;
pub use rules::*;
pub use simple::*;
pub use tableaux::*;

use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;

/// Main OWL2 reasoning engine
pub struct OwlReasoner {
    simple: SimpleReasoner,
    tableaux: Option<TableauxReasoner>,
    use_advanced_reasoning: bool,
}

/// Reasoning configuration
#[derive(Debug, Clone)]
pub struct ReasoningConfig {
    /// Enable basic reasoning
    pub enable_reasoning: bool,
    /// Use advanced tableaux reasoning
    pub use_advanced_reasoning: bool,
    /// Tableaux reasoning configuration
    pub tableaux_config: tableaux::ReasoningConfig,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        ReasoningConfig {
            enable_reasoning: true,
            use_advanced_reasoning: true,
            tableaux_config: tableaux::ReasoningConfig::default(),
        }
    }
}

/// Reasoning capabilities
pub trait Reasoner {
    /// Check if the ontology is consistent
    fn is_consistent(&mut self) -> OwlResult<bool>;

    /// Check if one class is a subclass of another
    fn is_subclass_of(&mut self, sub: &IRI, sup: &IRI) -> OwlResult<bool>;

    /// Check if two classes are equivalent
    fn are_equivalent_classes(&mut self, a: &IRI, b: &IRI) -> OwlResult<bool>;

    /// Check if two classes are disjoint
    fn are_disjoint_classes(&mut self, a: &IRI, b: &IRI) -> OwlResult<bool>;

    /// Get all instances of a class
    fn get_instances(&mut self, class: &IRI) -> OwlResult<Vec<IRI>>;

    /// Check if an individual is an instance of a class
    fn is_instance_of(&mut self, individual: &IRI, class: &IRI) -> OwlResult<bool>;
}

impl OwlReasoner {
    /// Create a new OWL2 reasoner
    pub fn new(ontology: Ontology) -> Self {
        Self::with_config(ontology, ReasoningConfig::default())
    }

    /// Create a new OWL2 reasoner with custom configuration
    pub fn with_config(ontology: Ontology, config: ReasoningConfig) -> Self {
        let simple = SimpleReasoner::new(ontology.clone());
        let tableaux = if config.use_advanced_reasoning {
            Some(TableauxReasoner::with_config(
                &ontology,
                config.tableaux_config,
            ))
        } else {
            None
        };

        OwlReasoner {
            simple,
            tableaux,
            use_advanced_reasoning: config.use_advanced_reasoning,
        }
    }

    /// Get the underlying ontology
    pub fn ontology(&self) -> &Ontology {
        &self.simple.ontology
    }

    /// Check if a class is satisfiable
    pub fn is_class_satisfiable(&mut self, class_iri: &IRI) -> OwlResult<bool> {
        self.simple.is_class_satisfiable(class_iri)
    }

    /// Create a query engine for this reasoner
    pub fn query_engine(&self) -> QueryEngine {
        QueryEngine::new(self.simple.ontology.clone())
    }

    /// Execute a SPARQL-like query
    pub fn query(&mut self, _query: &str) -> OwlResult<QueryResult> {
        let mut engine = self.query_engine();
        // Parse the query string into a query pattern
        // For now, we'll use a simple placeholder implementation
        let pattern = QueryPattern::BasicGraphPattern(vec![]);
        engine.execute_query(&pattern)
    }
}

impl Reasoner for OwlReasoner {
    fn is_consistent(&mut self) -> OwlResult<bool> {
        if self.use_advanced_reasoning {
            if let Some(tableaux) = &mut self.tableaux {
                // Use tableaux reasoning for proper consistency checking
                // Check if owl:Thing is satisfiable - if not, ontology is inconsistent
                let thing_iri = IRI::new("http://www.w3.org/2002/07/owl#Thing").unwrap();
                return tableaux.is_class_satisfiable(&thing_iri);
            }
        }
        self.simple.is_consistent()
    }

    fn is_subclass_of(&mut self, sub: &IRI, sup: &IRI) -> OwlResult<bool> {
        self.simple.is_subclass_of(sub, sup)
    }

    fn are_equivalent_classes(&mut self, a: &IRI, b: &IRI) -> OwlResult<bool> {
        // For now, check if a ⊑ b and b ⊑ a
        Ok(self.is_subclass_of(a, b)? && self.is_subclass_of(b, a)?)
    }

    fn are_disjoint_classes(&mut self, _a: &IRI, _b: &IRI) -> OwlResult<bool> {
        // For now, assume no classes are disjoint
        Ok(false)
    }

    fn get_instances(&mut self, class: &IRI) -> OwlResult<Vec<IRI>> {
        self.simple.get_instances(class)
    }

    fn is_instance_of(&mut self, individual: &IRI, class: &IRI) -> OwlResult<bool> {
        // For now, check if individual is in instances of class
        let instances = self.get_instances(class)?;
        Ok(instances.contains(individual))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::Ontology;

    #[test]
    fn test_reasoner_creation() {
        let ontology = Ontology::new();
        let reasoner = OwlReasoner::new(ontology);

        assert!(reasoner.ontology().classes().is_empty());
    }

    #[test]
    fn test_reasoner_consistency() {
        let ontology = Ontology::new();
        let mut reasoner = OwlReasoner::new(ontology);

        // Empty ontology should be consistent
        assert!(reasoner.is_consistent().unwrap());
    }

    #[test]
    fn test_reasoner_with_config() {
        let ontology = Ontology::new();
        let config = ReasoningConfig {
            enable_reasoning: false,
            use_advanced_reasoning: false,
            tableaux_config: tableaux::ReasoningConfig::default(),
        };

        let reasoner = OwlReasoner::with_config(ontology, config);
        assert!(reasoner.simple.ontology.classes().is_empty()); // Empty ontology should have no classes
    }
}
