//! Simplified OWL2 Reasoning Engine
//! 
//! Provides basic reasoning capabilities for OWL2 ontologies

use crate::ontology::Ontology;
use crate::iri::IRI;
use crate::error::OwlResult;

/// Simple reasoner for basic OWL2 reasoning
pub struct SimpleReasoner {
    pub ontology: Ontology,
}

impl SimpleReasoner {
    /// Create a new simple reasoner
    pub fn new(ontology: Ontology) -> Self {
        SimpleReasoner { ontology }
    }
    
    /// Check if the ontology is consistent
    pub fn is_consistent(&self) -> OwlResult<bool> {
        // For now, assume empty ontology is consistent
        Ok(true)
    }
    
    /// Check if a class is satisfiable
    pub fn is_class_satisfiable(&self, class_iri: &IRI) -> OwlResult<bool> {
        // For now, assume all classes are satisfiable unless they're explicitly disjoint with themselves
        // This is a simplified check - a full implementation would use tableaux reasoning
        for axiom in self.ontology.disjoint_classes_axioms() {
            let classes = axiom.classes();
            if classes.contains(class_iri) && classes.len() == 1 {
                return Ok(false); // Class is disjoint with itself - unsatisfiable
            }
        }
        Ok(true)
    }
    
    /// Check if one class is a subclass of another
    pub fn is_subclass_of(&self, sub: &IRI, sup: &IRI) -> OwlResult<bool> {
        // Check direct subclass relationships
        for axiom in self.ontology.subclass_axioms() {
            if let (crate::axioms::ClassExpression::Class(sub_axiom), crate::axioms::ClassExpression::Class(sup_axiom)) = 
                (axiom.sub_class(), axiom.super_class()) {
                if sub_axiom.iri() == sub && sup_axiom.iri() == sup {
                    return Ok(true);
                }
            }
        }
        
        // Check equivalent classes (if A ≡ B, then A ⊑ B and B ⊑ A)
        for axiom in self.ontology.equivalent_classes_axioms() {
            let classes = axiom.classes();
            if classes.contains(sub) && classes.contains(sup) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Get all instances of a class
    pub fn get_instances(&self, class_iri: &IRI) -> OwlResult<Vec<IRI>> {
        let mut instances = Vec::new();
        
        // Get direct class assertions
        for axiom in self.ontology.class_assertions() {
            if axiom.class_expr().contains_class(class_iri) {
                instances.push(axiom.individual().clone());
            }
        }
        
        // Get instances of equivalent classes
        for axiom in self.ontology.equivalent_classes_axioms() {
            let classes = axiom.classes();
            if classes.contains(class_iri) {
                for equiv_class in classes {
                    if equiv_class != class_iri {
                        // Get instances of the equivalent class
                        for assertion in self.ontology.class_assertions() {
                            if assertion.class_expr().contains_class(equiv_class) {
                                instances.push(assertion.individual().clone());
                            }
                        }
                    }
                }
            }
        }
        
        // Remove duplicates
        instances.sort();
        instances.dedup();
        
        Ok(instances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::Ontology;
    
    #[test]
    fn test_simple_reasoner_creation() {
        let ontology = Ontology::new();
        let reasoner = SimpleReasoner::new(ontology);
        
        assert!(reasoner.is_consistent().unwrap());
    }
    
    #[test]
    fn test_class_satisfiability() {
        let ontology = Ontology::new();
        let reasoner = SimpleReasoner::new(ontology);
        let class_iri = IRI::new("http://example.org/Person").unwrap();
        
        assert!(reasoner.is_class_satisfiable(&class_iri).unwrap());
    }
}