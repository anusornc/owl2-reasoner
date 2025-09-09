//! Simplified OWL2 Reasoning Engine
//! 
//! Provides basic reasoning capabilities for OWL2 ontologies

use crate::ontology::Ontology;
use crate::iri::IRI;
use crate::error::OwlResult;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Cache entry for reasoning results
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    timestamp: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        CacheEntry {
            value,
            timestamp: Instant::now(),
            ttl,
        }
    }
    
    fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > self.ttl
    }
    
    fn get(&self) -> Option<&T> {
        if self.is_expired() {
            None
        } else {
            Some(&self.value)
        }
    }
}

/// Simple reasoner for basic OWL2 reasoning with caching
pub struct SimpleReasoner {
    pub ontology: Ontology,
    
    // Caching layers
    consistency_cache: RwLock<Option<CacheEntry<bool>>>,
    subclass_cache: RwLock<HashMap<(IRI, IRI), CacheEntry<bool>>>,
    satisfiability_cache: RwLock<HashMap<IRI, CacheEntry<bool>>>,
    instances_cache: RwLock<HashMap<IRI, CacheEntry<Vec<IRI>>>>,
}

impl SimpleReasoner {
    /// Create a new simple reasoner
    pub fn new(ontology: Ontology) -> Self {
        SimpleReasoner {
            ontology,
            consistency_cache: RwLock::new(None),
            subclass_cache: RwLock::new(HashMap::new()),
            satisfiability_cache: RwLock::new(HashMap::new()),
            instances_cache: RwLock::new(HashMap::new()),
        }
    }
    
    /// Clear all caches
    pub fn clear_caches(&self) {
        let mut consistency = self.consistency_cache.write().unwrap();
        *consistency = None;
        
        let mut subclass = self.subclass_cache.write().unwrap();
        subclass.clear();
        
        let mut satisfiability = self.satisfiability_cache.write().unwrap();
        satisfiability.clear();
        
        let mut instances = self.instances_cache.write().unwrap();
        instances.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        let consistency = self.consistency_cache.read().unwrap();
        stats.insert("consistency".to_string(), consistency.as_ref().map_or(0, |_| 1));
        
        let subclass = self.subclass_cache.read().unwrap();
        stats.insert("subclass".to_string(), subclass.len());
        
        let satisfiability = self.satisfiability_cache.read().unwrap();
        stats.insert("satisfiability".to_string(), satisfiability.len());
        
        let instances = self.instances_cache.read().unwrap();
        stats.insert("instances".to_string(), instances.len());
        
        stats
    }
    
    /// Check if the ontology is consistent (cached)
    pub fn is_consistent(&self) -> OwlResult<bool> {
        // Check cache first
        {
            let cache = self.consistency_cache.read().unwrap();
            if let Some(entry) = cache.as_ref() {
                if let Some(result) = entry.get() {
                    return Ok(*result);
                }
            }
        }
        
        // Compute result
        let result = self.compute_consistency()?;
        
        // Cache result (5 minute TTL for consistency)
        let mut cache = self.consistency_cache.write().unwrap();
        *cache = Some(CacheEntry::new(result, Duration::from_secs(300)));
        
        Ok(result)
    }
    
    /// Compute consistency (internal method)
    fn compute_consistency(&self) -> OwlResult<bool> {
        // For now, assume empty ontology is consistent
        Ok(true)
    }
    
    /// Check if a class is satisfiable (cached)
    pub fn is_class_satisfiable(&self, class_iri: &IRI) -> OwlResult<bool> {
        // Check cache first
        {
            let cache = self.satisfiability_cache.read().unwrap();
            if let Some(entry) = cache.get(class_iri) {
                if let Some(result) = entry.get() {
                    return Ok(*result);
                }
            }
        }
        
        // Compute result
        let result = self.compute_satisfiability(class_iri)?;
        
        // Cache result (2 minute TTL for satisfiability)
        let mut cache = self.satisfiability_cache.write().unwrap();
        cache.insert(class_iri.clone(), CacheEntry::new(result, Duration::from_secs(120)));
        
        Ok(result)
    }
    
    /// Compute satisfiability (internal method)
    fn compute_satisfiability(&self, class_iri: &IRI) -> OwlResult<bool> {
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
    
    /// Check if one class is a subclass of another (cached)
    pub fn is_subclass_of(&self, sub: &IRI, sup: &IRI) -> OwlResult<bool> {
        let key = (sub.clone(), sup.clone());
        
        // Check cache first
        {
            let cache = self.subclass_cache.read().unwrap();
            if let Some(entry) = cache.get(&key) {
                if let Some(result) = entry.get() {
                    return Ok(*result);
                }
            }
        }
        
        // Compute result
        let result = self.compute_subclass_of(sub, sup)?;
        
        // Cache result (1 minute TTL for subclass relationships)
        let mut cache = self.subclass_cache.write().unwrap();
        cache.insert(key, CacheEntry::new(result, Duration::from_secs(60)));
        
        Ok(result)
    }
    
    /// Compute subclass relationship (internal method)
    fn compute_subclass_of(&self, sub: &IRI, sup: &IRI) -> OwlResult<bool> {
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
    
    /// Get all instances of a class (cached)
    pub fn get_instances(&self, class_iri: &IRI) -> OwlResult<Vec<IRI>> {
        // Check cache first
        {
            let cache = self.instances_cache.read().unwrap();
            if let Some(entry) = cache.get(class_iri) {
                if let Some(result) = entry.get() {
                    return Ok(result.clone());
                }
            }
        }
        
        // Compute result
        let result = self.compute_instances(class_iri)?;
        
        // Cache result (30 second TTL for instances - they might change frequently)
        let mut cache = self.instances_cache.write().unwrap();
        cache.insert(class_iri.clone(), CacheEntry::new(result.clone(), Duration::from_secs(30)));
        
        Ok(result)
    }
    
    /// Compute instances (internal method)
    fn compute_instances(&self, class_iri: &IRI) -> OwlResult<Vec<IRI>> {
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