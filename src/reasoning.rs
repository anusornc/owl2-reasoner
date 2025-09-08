//! OWL2 Reasoning Engine
//! 
//! Provides reasoning capabilities for OWL2 ontologies including
//! tableaux-based reasoning, rule-based inference, and query answering.

pub mod simple;
// TODO: Re-enable advanced modules when axioms are fully implemented
// pub mod tableaux;
// pub mod rules;
// pub mod query;
// pub mod consistency;
// pub mod classification;

pub use simple::*;
// pub use tableaux::*;
// pub use rules::*;
// pub use query::*;
// pub use consistency::*;
// pub use classification::*;

use crate::ontology::Ontology;
use crate::iri::IRI;
use crate::error::OwlResult;

/// Main OWL2 reasoning engine
pub struct OwlReasoner {
    simple: SimpleReasoner,
}

/// Reasoning configuration
#[derive(Debug, Clone)]
pub struct ReasoningConfig {
    /// Enable basic reasoning
    pub enable_reasoning: bool,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        ReasoningConfig {
            enable_reasoning: true,
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
    pub fn with_config(ontology: Ontology, _config: ReasoningConfig) -> Self {
        let simple = SimpleReasoner::new(ontology);
        
        OwlReasoner {
            simple,
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
}

impl Reasoner for OwlReasoner {
    fn is_consistent(&mut self) -> OwlResult<bool> {
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
        };
        
        let reasoner = OwlReasoner::with_config(ontology, config);
        assert!(reasoner.simple.ontology.classes().is_empty()); // Empty ontology should have no classes
    }
}