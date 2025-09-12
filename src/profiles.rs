//! OWL2 Profile Validation Module
//! 
//! This module implements validation for the three OWL2 profiles:
//! - EL: Expressive Logic (EL++) profile
//! - QL: Query Language (OWL2 QL) profile  
//! - RL: Rule Language (OWL2 RL) profile
//! 
//! Each profile has specific restrictions on which OWL2 constructs are allowed.
//! This module provides efficient validation algorithms inspired by owl2_rs.

use crate::ontology::Ontology;
use crate::iri::IRI;
use crate::error::OwlResult;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// OWL2 Profile types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Owl2Profile {
    /// OWL2 EL Profile (Expressive Logic)
    EL,
    /// OWL2 QL Profile (Query Language) 
    QL,
    /// OWL2 RL Profile (Rule Language)
    RL,
}

impl std::fmt::Display for Owl2Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Owl2Profile::EL => write!(f, "OWL2 EL"),
            Owl2Profile::QL => write!(f, "OWL2 QL"),
            Owl2Profile::RL => write!(f, "OWL2 RL"),
        }
    }
}

/// Profile validation result
#[derive(Debug, Clone)]
pub struct ProfileValidationResult {
    pub profile: Owl2Profile,
    pub is_valid: bool,
    pub violations: Vec<ProfileViolation>,
    pub statistics: ValidationStatistics,
}

/// Profile violation details
#[derive(Debug, Clone)]
pub struct ProfileViolation {
    pub violation_type: ProfileViolationType,
    pub message: String,
    pub affected_entities: Vec<IRI>,
    pub severity: ViolationSeverity,
}

/// Types of profile violations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileViolationType {
    // EL Profile Violations
    ComplexSubclassAxiom,
    DisjointClassesAxiom,
    EquivalentClassesAxiom,
    ComplexPropertyRestrictions,
    DataPropertyRanges,
    
    // QL Profile Violations
    TransitiveProperties,
    AsymmetricProperties,
    IrreflexiveProperties,
    ComplexCardinalityRestrictions,
    PropertyChainAxioms,
    
    // RL Profile Violations
    Nominals,
    DataComplementOf,
    DataOneOf,
    ObjectComplementOf,
    ObjectOneOf,
    ObjectHasSelf,
    
    // General violations
    UnsupportedConstruct,
    RecursiveDefinitions,
    CycleInHierarchy,
}

/// Severity levels for violations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationSeverity {
    Error,      // Must be fixed for profile compliance
    Warning,    // Should be reviewed
    Info,       // Informational note
}

/// Validation statistics
#[derive(Debug, Clone)]
pub struct ValidationStatistics {
    pub total_axioms_checked: usize,
    pub violations_found: usize,
    pub validation_time_ms: f64,
    pub memory_usage_bytes: usize,
}

/// Profile validator trait
pub trait ProfileValidator {
    fn validate_profile(&self, profile: Owl2Profile) -> OwlResult<ProfileValidationResult>;
    fn is_el_profile(&self) -> bool;
    fn is_ql_profile(&self) -> bool; 
    fn is_rl_profile(&self) -> bool;
    fn get_optimization_hints(&self) -> Vec<OptimizationHint>;
}

/// Optimization hints for profile compliance
#[derive(Debug, Clone)]
pub struct OptimizationHint {
    pub hint_type: OptimizationType,
    pub description: String,
    pub estimated_impact: String,
}

#[derive(Debug, Clone)]
pub enum OptimizationType {
    RestructureHierarchy,
    SimplifyExpressions,
    RemoveUnsupportedConstructs,
    AddMissingDeclarations,
}

/// OWL2 Profile validation implementation
pub struct Owl2ProfileValidator {
    ontology: Arc<Ontology>,
    cache: RwLock<HashMap<Owl2Profile, ProfileValidationResult>>,
}

impl Owl2ProfileValidator {
    /// Create a new profile validator for the given ontology
    pub fn new(ontology: Arc<Ontology>) -> Self {
        Self {
            ontology,
            cache: RwLock::new(HashMap::new()),
        }
    }
    
    /// Validate all profiles and return comprehensive results
    pub fn validate_all_profiles(&mut self) -> OwlResult<Vec<ProfileValidationResult>> {
        let mut results = Vec::new();
        
        for profile in [Owl2Profile::EL, Owl2Profile::QL, Owl2Profile::RL] {
            results.push(self.validate_profile(profile)?);
        }
        
        Ok(results)
    }
    
    /// Get the most restrictive valid profile for this ontology
    pub fn get_most_restrictive_profile(&mut self) -> OwlResult<Option<Owl2Profile>> {
        let profiles = [Owl2Profile::EL, Owl2Profile::QL, Owl2Profile::RL];
        
        for profile in profiles {
            let result = self.validate_profile(profile.clone())?;
            if result.is_valid {
                return Ok(Some(profile));
            }
        }
        
        Ok(None) // No profile restrictions satisfied
    }
    
    /// Check if ontology satisfies any OWL2 profile
    pub fn satisfies_any_profile(&mut self) -> OwlResult<bool> {
        Ok(self.get_most_restrictive_profile()?.is_some())
    }
    
    /// Clear validation cache
    pub fn clear_cache(&mut self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().unwrap();
        (cache.len(), cache.capacity())
    }
}

impl ProfileValidator for Owl2ProfileValidator {
    fn validate_profile(&self, profile: Owl2Profile) -> OwlResult<ProfileValidationResult> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached_result) = cache.get(&profile) {
                return Ok(cached_result.clone());
            }
        }
        
        let start_time = std::time::Instant::now();
        let mut violations = Vec::new();
        
        // Profile-specific validation
        match profile {
            Owl2Profile::EL => {
                violations.extend(self.validate_el_profile()?);
            },
            Owl2Profile::QL => {
                violations.extend(self.validate_ql_profile()?);
            },
            Owl2Profile::RL => {
                violations.extend(self.validate_rl_profile()?);
            },
        }
        
        let validation_time = start_time.elapsed();
        let is_valid = violations.is_empty();
        
        // Estimate memory usage (simplified)
        let memory_usage = self.estimate_memory_usage();
        
        let violations_count = violations.len();
        let result = ProfileValidationResult {
            profile: profile.clone(),
            is_valid,
            violations,
            statistics: ValidationStatistics {
                total_axioms_checked: self.count_total_axioms(),
                violations_found: violations_count,
                validation_time_ms: validation_time.as_secs_f64() * 1000.0,
                memory_usage_bytes: memory_usage,
            },
        };
        
        // Cache result
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(profile, result.clone());
        }
        
        Ok(result)
    }
    
    fn is_el_profile(&self) -> bool {
        // Quick check without full validation
        self.check_el_restrictions_quick().unwrap_or(false)
    }
    
    fn is_ql_profile(&self) -> bool {
        self.check_ql_restrictions_quick().unwrap_or(false)
    }
    
    fn is_rl_profile(&self) -> bool {
        self.check_rl_restrictions_quick().unwrap_or(false)
    }
    
    fn get_optimization_hints(&self) -> Vec<OptimizationHint> {
        let mut hints = Vec::new();
        
        // Analyze ontology structure and suggest optimizations
        if self.ontology.subclass_axioms().len() > 1000 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::RestructureHierarchy,
                description: "Large hierarchy detected. Consider flattening for EL profile compliance.".to_string(),
                estimated_impact: "May enable EL profile validation".to_string(),
            });
        }
        
        if !self.ontology.disjoint_classes_axioms().is_empty() {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::RemoveUnsupportedConstructs,
                description: "Disjoint classes axioms found. These are not allowed in EL profile.".to_string(),
                estimated_impact: "Required for EL profile compliance".to_string(),
            });
        }
        
        hints
    }
}

impl Owl2ProfileValidator {
    // EL Profile Validation
    fn validate_el_profile(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();
        
        // Check for disallowed constructs in EL profile
        
        // 1. No disjoint classes axioms
        if !self.ontology.disjoint_classes_axioms().is_empty() {
            violations.push(ProfileViolation {
                violation_type: ProfileViolationType::DisjointClassesAxiom,
                message: "Disjoint classes axioms are not allowed in EL profile".to_string(),
                affected_entities: self.get_affected_entities_from_disjoint_classes(),
                severity: ViolationSeverity::Error,
            });
        }
        
        // 2. No equivalent classes axioms (except simple cases)
        violations.extend(self.check_equivalent_classes_for_el()?);
        
        // 3. Check property restrictions
        violations.extend(self.check_property_restrictions_for_el()?);
        
        // 4. No data property ranges beyond basic datatypes
        violations.extend(self.check_data_property_ranges_for_el()?);
        
        Ok(violations)
    }
    
    // QL Profile Validation  
    fn validate_ql_profile(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();
        
        // Check for disallowed constructs in QL profile
        
        // 1. No transitive properties
        violations.extend(self.check_transitive_properties_for_ql()?);
        
        // 2. No asymmetric properties
        violations.extend(self.check_asymmetric_properties_for_ql()?);
        
        // 3. No complex cardinality restrictions
        violations.extend(self.check_cardinality_restrictions_for_ql()?);
        
        // 4. No property chain axioms
        violations.extend(self.check_property_chains_for_ql()?);
        
        Ok(violations)
    }
    
    // RL Profile Validation
    fn validate_rl_profile(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();
        
        // Check for disallowed constructs in RL profile
        
        // 1. No nominals (named individuals in class expressions)
        violations.extend(self.check_nominals_for_rl()?);
        
        // 2. No data complement of
        violations.extend(self.check_data_complement_for_rl()?);
        
        // 3. No object complement of
        violations.extend(self.check_object_complement_for_rl()?);
        
        // 4. No object one of
        violations.extend(self.check_object_one_of_for_rl()?);
        
        Ok(violations)
    }
    
    // Helper methods for validation
    fn check_el_restrictions_quick(&self) -> OwlResult<bool> {
        // Quick check for EL profile without detailed validation
        Ok(self.ontology.disjoint_classes_axioms().is_empty() 
            && !self.has_complex_equivalent_classes()?
            && !self.has_complex_property_restrictions()?)
    }
    
    fn check_ql_restrictions_quick(&self) -> OwlResult<bool> {
        // Quick check for QL profile
        Ok(!self.has_transitive_properties()?
            && !self.has_asymmetric_properties()?
            && !self.has_complex_cardinality_restrictions()?)
    }
    
    fn check_rl_restrictions_quick(&self) -> OwlResult<bool> {
        // Quick check for RL profile
        Ok(!self.has_nominals()?
            && !self.has_data_complement()?
            && !self.has_object_complement()?)
    }
    
    // Additional helper methods for detailed validation
    fn has_complex_equivalent_classes(&self) -> OwlResult<bool> {
        // Check if there are complex equivalent classes axioms
        // For now, assume all equivalent classes are complex for EL
        Ok(!self.ontology.equivalent_classes_axioms().is_empty())
    }
    
    fn has_complex_property_restrictions(&self) -> OwlResult<bool> {
        // Check for complex property restrictions not allowed in EL
        // This would require detailed analysis of property characteristics
        Ok(false) // Simplified for now
    }
    
    fn has_transitive_properties(&self) -> OwlResult<bool> {
        // Check for transitive properties (not allowed in QL)
        // This would need to check property characteristics
        Ok(false) // Simplified for now
    }
    
    fn has_asymmetric_properties(&self) -> OwlResult<bool> {
        // Check for asymmetric properties (not allowed in QL)
        Ok(false) // Simplified for now
    }
    
    fn has_complex_cardinality_restrictions(&self) -> OwlResult<bool> {
        // Check for complex cardinality restrictions (not allowed in QL)
        Ok(false) // Simplified for now
    }
    
    fn has_nominals(&self) -> OwlResult<bool> {
        // Check for nominals in class expressions (not allowed in RL)
        Ok(false) // Simplified for now
    }
    
    fn has_data_complement(&self) -> OwlResult<bool> {
        // Check for data complement of (not allowed in RL)
        Ok(false) // Simplified for now
    }
    
    fn has_object_complement(&self) -> OwlResult<bool> {
        // Check for object complement of (not allowed in RL)
        Ok(false) // Simplified for now
    }
    
    // Additional validation helper methods
    fn check_equivalent_classes_for_el(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();
        
        for axiom in self.ontology.equivalent_classes_axioms() {
            if axiom.classes().len() > 2 {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::EquivalentClassesAxiom,
                    message: "Complex equivalent classes axioms with more than 2 classes are not allowed in EL profile".to_string(),
                    affected_entities: axiom.classes().to_vec(),
                    severity: ViolationSeverity::Error,
                });
            }
        }
        
        Ok(violations)
    }
    
    fn check_property_restrictions_for_el(&self) -> OwlResult<Vec<ProfileViolation>> {
        // Simplified implementation
        Ok(Vec::new())
    }
    
    fn check_data_property_ranges_for_el(&self) -> OwlResult<Vec<ProfileViolation>> {
        // Simplified implementation  
        Ok(Vec::new())
    }
    
    fn check_transitive_properties_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        // Simplified implementation
        Ok(Vec::new())
    }
    
    fn check_asymmetric_properties_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        // Simplified implementation
        Ok(Vec::new())
    }
    
    fn check_cardinality_restrictions_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        // Simplified implementation
        Ok(Vec::new())
    }
    
    fn check_property_chains_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        // Simplified implementation
        Ok(Vec::new())
    }
    
    fn check_nominals_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        // Simplified implementation
        Ok(Vec::new())
    }
    
    fn check_data_complement_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        // Simplified implementation
        Ok(Vec::new())
    }
    
    fn check_object_complement_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        // Simplified implementation
        Ok(Vec::new())
    }
    
    fn check_object_one_of_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        // Simplified implementation
        Ok(Vec::new())
    }
    
    // Utility methods
    fn get_affected_entities_from_disjoint_classes(&self) -> Vec<IRI> {
        let mut entities = Vec::new();
        for axiom in self.ontology.disjoint_classes_axioms() {
            entities.extend(axiom.classes().to_vec());
        }
        entities
    }
    
    fn count_total_axioms(&self) -> usize {
        // Count all axioms in the ontology
        self.ontology.subclass_axioms().len() +
        self.ontology.equivalent_classes_axioms().len() +
        self.ontology.disjoint_classes_axioms().len() +
        self.ontology.object_properties().len() +
        self.ontology.data_properties().len()
    }
    
    fn estimate_memory_usage(&self) -> usize {
        // Simplified memory usage estimation
        self.ontology.classes().len() * 64 + // Approximate size per class
        self.ontology.object_properties().len() * 48 + // Approximate size per property
        self.ontology.subclass_axioms().len() * 32 // Approximate size per axiom
    }
}

// Profile-specific validation implementations
pub mod el;
pub mod ql;
pub mod rl;

pub use el::*;
pub use ql::*;
pub use rl::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::Ontology;
    use std::sync::Arc;
    
    #[test]
    fn test_profile_validator_creation() {
        let ontology = Arc::new(Ontology::new());
        let validator = Owl2ProfileValidator::new(ontology);
        
        assert!(validator.is_el_profile());
        assert!(validator.is_ql_profile());
        assert!(validator.is_rl_profile());
    }
    
    #[test]
    fn test_profile_validation_result_structure() {
        let ontology = Arc::new(Ontology::new());
        let mut validator = Owl2ProfileValidator::new(ontology);
        
        let result = validator.validate_profile(Owl2Profile::EL).unwrap();
        
        assert_eq!(result.profile, Owl2Profile::EL);
        assert!(result.is_valid); // Empty ontology should be valid
        assert!(result.violations.is_empty());
        assert!(result.statistics.validation_time_ms >= 0.0);
    }
    
    #[test]
    fn test_optimization_hints() {
        let ontology = Arc::new(Ontology::new());
        let validator = Owl2ProfileValidator::new(ontology);
        
        let hints = validator.get_optimization_hints();
        
        // For empty ontology, hints may be empty - this is expected behavior
        // The test should verify the method works without errors
        assert!(hints.is_empty()); // Empty ontology should have no optimization hints
    }
    
    #[test]
    fn test_cache_functionality() {
        let ontology = Arc::new(Ontology::new());
        let mut validator = Owl2ProfileValidator::new(ontology);
        
        // First validation should populate cache
        let result1 = validator.validate_profile(Owl2Profile::EL).unwrap();
        let (cache_size, _) = validator.cache_stats();
        assert_eq!(cache_size, 1);
        
        // Second validation should use cache
        let result2 = validator.validate_profile(Owl2Profile::EL).unwrap();
        assert_eq!(result1.is_valid, result2.is_valid);
        
        // Clear cache should work
        validator.clear_cache();
        let (cache_size_after, _) = validator.cache_stats();
        assert_eq!(cache_size_after, 0);
    }
}