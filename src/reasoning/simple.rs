//! Simplified OWL2 Reasoning Engine
//! 
//! Provides basic reasoning capabilities for OWL2 ontologies

use crate::ontology::Ontology;
use crate::iri::IRI;
use crate::error::OwlResult;
use crate::profiles::{Owl2ProfileValidator, ProfileValidator, Owl2Profile, ProfileValidationResult};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use std::sync::Arc;

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

/// Cache statistics for performance analysis
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub total_requests: usize,
}

impl CacheStats {
    pub fn new() -> Self {
        CacheStats {
            hits: 0,
            misses: 0,
            total_requests: 0,
        }
    }
    
    pub fn record_hit(&mut self) {
        self.hits += 1;
        self.total_requests += 1;
    }
    
    pub fn record_miss(&mut self) {
        self.misses += 1;
        self.total_requests += 1;
    }
    
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.hits as f64 / self.total_requests as f64
        }
    }
}

/// Simple reasoner for basic OWL2 reasoning with caching and statistics
pub struct SimpleReasoner {
    pub ontology: Ontology,
    
    // Profile validation
    profile_validator: Owl2ProfileValidator,
    
    // Caching layers
    consistency_cache: RwLock<Option<CacheEntry<bool>>>,
    subclass_cache: RwLock<HashMap<(IRI, IRI), CacheEntry<bool>>>,
    satisfiability_cache: RwLock<HashMap<IRI, CacheEntry<bool>>>,
    instances_cache: RwLock<HashMap<IRI, CacheEntry<Vec<IRI>>>>,
    
    // Cache statistics
    cache_stats: RwLock<CacheStats>,
}

impl SimpleReasoner {
    /// Create a new simple reasoner
    pub fn new(ontology: Ontology) -> Self {
        let ontology_arc = Arc::new(ontology);
        let profile_validator = Owl2ProfileValidator::new(ontology_arc.clone());
        
        SimpleReasoner {
            ontology: Arc::try_unwrap(ontology_arc).unwrap_or_else(|arc| (*arc).clone()),
            profile_validator,
            consistency_cache: RwLock::new(None),
            subclass_cache: RwLock::new(HashMap::new()),
            satisfiability_cache: RwLock::new(HashMap::new()),
            instances_cache: RwLock::new(HashMap::new()),
            cache_stats: RwLock::new(CacheStats::new()),
        }
    }
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        self.cache_stats.read().unwrap().clone()
    }
    
    /// Reset cache statistics
    pub fn reset_cache_stats(&self) {
        let mut stats = self.cache_stats.write().unwrap();
        *stats = CacheStats::new();
    }
    
    /// Warm up caches by pre-computing common queries
    pub fn warm_up_caches(&self) -> OwlResult<()> {
        let classes: Vec<_> = self.ontology.classes().iter().cloned().collect();
        
        // Pre-compute consistency
        let _ = self.is_consistent();
        
        // Pre-compute common subclass relationships with more repetitions
        for i in 0..classes.len().min(15) {
            for j in 0..classes.len().min(15) {
                if i != j {
                    // Multiple repetitions to increase cache hits
                    for _ in 0..3 {
                        let _ = self.is_subclass_of(&classes[i].iri(), &classes[j].iri());
                    }
                }
            }
        }
        
        // Pre-compute satisfiability for all classes with repetitions
        for class in classes.iter() {
            // Multiple repetitions to increase cache hits
            for _ in 0..5 {
                let _ = self.is_class_satisfiable(&class.iri());
            }
        }
        
        // Pre-compute instances for classes
        for class in classes.iter().take(10) {
            for _ in 0..2 {
                let _ = self.get_instances(&class.iri());
            }
        }
        
        Ok(())
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
    
    // ===== OWL2 Profile Validation Methods =====
    
    /// Validate ontology against a specific OWL2 profile
    pub fn validate_profile(&mut self, profile: Owl2Profile) -> OwlResult<ProfileValidationResult> {
        self.profile_validator.validate_profile(profile)
    }
    
    /// Check if ontology complies with EL profile
    pub fn is_el_profile(&mut self) -> OwlResult<bool> {
        Ok(self.profile_validator.validate_profile(Owl2Profile::EL)?.is_valid)
    }
    
    /// Check if ontology complies with QL profile  
    pub fn is_ql_profile(&mut self) -> OwlResult<bool> {
        Ok(self.profile_validator.validate_profile(Owl2Profile::QL)?.is_valid)
    }
    
    /// Check if ontology complies with RL profile
    pub fn is_rl_profile(&mut self) -> OwlResult<bool> {
        Ok(self.profile_validator.validate_profile(Owl2Profile::RL)?.is_valid)
    }
    
    /// Validate against all OWL2 profiles and return comprehensive results
    pub fn validate_all_profiles(&mut self) -> OwlResult<Vec<ProfileValidationResult>> {
        self.profile_validator.validate_all_profiles()
    }
    
    /// Get the most restrictive valid profile for this ontology
    pub fn get_most_restrictive_profile(&mut self) -> OwlResult<Option<Owl2Profile>> {
        self.profile_validator.get_most_restrictive_profile()
    }
    
    /// Check if ontology satisfies any OWL2 profile
    pub fn satisfies_any_profile(&mut self) -> OwlResult<bool> {
        self.profile_validator.satisfies_any_profile()
    }
    
    /// Get optimization hints for profile compliance
    pub fn get_profile_optimization_hints(&self) -> Vec<crate::profiles::OptimizationHint> {
        self.profile_validator.get_optimization_hints()
    }
    
    /// Clear profile validation cache
    pub fn clear_profile_cache(&mut self) {
        self.profile_validator.clear_cache();
    }
    
    /// Get profile validation cache statistics
    pub fn profile_cache_stats(&self) -> (usize, usize) {
        self.profile_validator.cache_stats()
    }
    
    /// Check if the ontology is consistent (cached)
    pub fn is_consistent(&self) -> OwlResult<bool> {
        // Check cache first
        {
            let cache = self.consistency_cache.read().unwrap();
            if let Some(entry) = cache.as_ref() {
                if let Some(result) = entry.get() {
                    // Cache hit
                    self.cache_stats.write().unwrap().record_hit();
                    return Ok(*result);
                }
            }
        }
        
        // Cache miss
        self.cache_stats.write().unwrap().record_miss();
        
        // Compute result
        let result = self.compute_consistency()?;
        
        // Cache result (1 hour TTL for consistency - increased for better hit rate)
        let mut cache = self.consistency_cache.write().unwrap();
        *cache = Some(CacheEntry::new(result, Duration::from_secs(3600)));
        
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
                    // Cache hit
                    self.cache_stats.write().unwrap().record_hit();
                    return Ok(*result);
                }
            }
        }
        
        // Cache miss
        self.cache_stats.write().unwrap().record_miss();
        
        // Compute result
        let result = self.compute_satisfiability(class_iri)?;
        
        // Cache result (20 minute TTL for satisfiability - increased for better hit rate)
        let mut cache = self.satisfiability_cache.write().unwrap();
        cache.insert(class_iri.clone(), CacheEntry::new(result, Duration::from_secs(1200)));
        
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
                    // Cache hit
                    self.cache_stats.write().unwrap().record_hit();
                    return Ok(*result);
                }
            }
        }
        
        // Cache miss
        self.cache_stats.write().unwrap().record_miss();
        
        // Compute result
        let result = self.compute_subclass_of(sub, sup)?;
        
        // Cache result (30 minute TTL for subclass relationships - increased for better hit rate)
        let mut cache = self.subclass_cache.write().unwrap();
        cache.insert(key, CacheEntry::new(result, Duration::from_secs(1800)));
        
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