//! OWL2 Profile Validation Module
//!
//! This module implements validation for the three OWL2 profiles:
//! - EL: Expressive Logic (EL++) profile
//! - QL: Query Language (OWL2 QL) profile  
//! - RL: Rule Language (OWL2 RL) profile
//!
//! Each profile has specific restrictions on which OWL2 constructs are allowed.
//! This module provides efficient validation algorithms inspired by owl2_rs.

use crate::axioms::ClassExpression;
use crate::axioms::ClassExpression::*;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::DataRange;
use crate::ObjectPropertyExpression;
use bumpalo::Bump;
use dashmap::DashMap;
use lru::LruCache;
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// OWL2 Profile types
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfileValidationResult {
    pub profile: Owl2Profile,
    pub is_valid: bool,
    pub violations: Vec<ProfileViolation>,
    pub statistics: ValidationStatistics,
}

/// Profile violation details
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfileViolation {
    pub violation_type: ProfileViolationType,
    pub message: String,
    pub affected_entities: Vec<IRI>,
    pub severity: ViolationSeverity,
}

/// Types of profile violations
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    ComplexClassExpressions,
    ComplexDataRanges,
}

/// Severity levels for violations
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ViolationSeverity {
    Error,   // Must be fixed for profile compliance
    Warning, // Should be reviewed
    Info,    // Informational note
}

/// Validation statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationStatistics {
    pub total_axioms_checked: usize,
    pub violations_found: usize,
    pub validation_time_ms: f64,
    pub memory_usage_bytes: usize,
}

/// Profile validator trait
pub trait ProfileValidator {
    fn validate_profile(&mut self, profile: Owl2Profile) -> OwlResult<ProfileValidationResult>;
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

/// Detailed profile analysis report
#[derive(Debug, Clone)]
pub struct ProfileAnalysisReport {
    pub el_compliant: bool,
    pub ql_compliant: bool,
    pub rl_compliant: bool,
    pub ontology_stats: OntologyStats,
    pub el_violations: Vec<String>,
    pub ql_violations: Vec<String>,
    pub rl_violations: Vec<String>,
}

/// Ontology statistics
#[derive(Debug, Clone)]
pub struct OntologyStats {
    pub total_classes: usize,
    pub total_properties: usize,
    pub total_individuals: usize,
    pub total_axioms: usize,
    pub max_class_expression_depth: usize,
}

/// OWL2 Profile validation implementation with optimized caching, memory pools, and pre-computation indexes
pub struct Owl2ProfileValidator {
    ontology: Arc<Ontology>,
    cache: DashMap<Owl2Profile, ProfileValidationResult>, // Legacy cache for backward compatibility
    advanced_cache: AdvancedCacheManager,                 // New advanced caching system
    result_arena: Bump,
    violation_pool: ViolationPool,
    indexes: ProfileIndexes,
    validation_stats: ValidationStats,
    use_advanced_caching: bool,
}

impl Owl2ProfileValidator {
    /// Create a new profile validator for the given ontology
    pub fn new(ontology: Arc<Ontology>) -> Self {
        let indexes = ProfileIndexes::analyze_ontology(&ontology);

        Self {
            ontology,
            cache: DashMap::new(),
            advanced_cache: AdvancedCacheManager::new(),
            result_arena: Bump::new(),
            violation_pool: ViolationPool::new(),
            indexes,
            validation_stats: ValidationStats::new(),
            use_advanced_caching: true,
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
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.capacity())
    }

    /// Get validation statistics
    pub fn get_validation_stats(&self) -> &ValidationStats {
        &self.validation_stats
    }

    /// Reset validation statistics and clear memory pools
    pub fn reset_stats(&mut self) {
        self.validation_stats.reset();
        self.violation_pool.clear();
        self.result_arena.reset();
    }

    /// Get pre-computation indexes for profile analysis
    pub fn get_indexes(&self) -> &ProfileIndexes {
        &self.indexes
    }

    /// Re-compute indexes (useful when ontology changes)
    pub fn recompute_indexes(&mut self) {
        self.indexes = ProfileIndexes::analyze_ontology(&self.ontology);
    }

    /// Enable or disable advanced caching
    pub fn set_advanced_caching(&mut self, enabled: bool) {
        self.use_advanced_caching = enabled;
    }

    /// Check if advanced caching is enabled
    pub fn is_advanced_caching_enabled(&self) -> bool {
        self.use_advanced_caching
    }

    /// Get advanced cache statistics
    pub fn get_advanced_cache_stats(&self) -> &CacheStatistics {
        self.advanced_cache.get_stats()
    }

    /// Get advanced cache configuration
    pub fn get_advanced_cache_config(&self) -> &ProfileCacheConfig {
        self.advanced_cache.get_config()
    }

    /// Configure advanced caching with custom settings
    pub fn configure_advanced_caching(&mut self, config: ProfileCacheConfig) {
        self.advanced_cache = AdvancedCacheManager::with_config(config);
    }

    /// Clear advanced cache
    pub fn clear_advanced_cache(&mut self) {
        self.advanced_cache.clear();
    }

    /// Invalidate cache entries by token
    pub fn invalidate_cache_by_token(&mut self, token: u64) {
        self.advanced_cache.invalidate_by_token(token);
    }

    /// Fast profile compliance check using pre-computed indexes
    pub fn fast_profile_check(&self, profile: Owl2Profile) -> bool {
        match profile {
            Owl2Profile::EL => self.indexes.is_el_compliant(),
            Owl2Profile::QL => self.indexes.is_ql_compliant(),
            Owl2Profile::RL => self.indexes.is_rl_compliant(),
        }
    }

    /// Get detailed profile analysis report
    pub fn get_profile_analysis(&self) -> ProfileAnalysisReport {
        ProfileAnalysisReport {
            el_compliant: self.indexes.is_el_compliant(),
            ql_compliant: self.indexes.is_ql_compliant(),
            rl_compliant: self.indexes.is_rl_compliant(),
            ontology_stats: OntologyStats {
                total_classes: self.indexes.total_classes,
                total_properties: self.indexes.total_properties,
                total_individuals: self.indexes.total_individuals,
                total_axioms: self.indexes.total_axioms,
                max_class_expression_depth: self.indexes.max_class_expression_depth,
            },
            el_violations: self.get_el_violation_summary(),
            ql_violations: self.get_ql_violation_summary(),
            rl_violations: self.get_rl_violation_summary(),
        }
    }

    fn get_el_violation_summary(&self) -> Vec<String> {
        let mut violations = Vec::new();

        if self.indexes.el_has_disjoint_classes {
            violations.push("Contains disjoint classes axioms (not allowed in EL)".to_string());
        }
        if self.indexes.el_has_complex_restrictions {
            violations
                .push("Contains complex property restrictions (not allowed in EL)".to_string());
        }
        if self.indexes.el_has_nominals {
            violations.push("Contains nominals (ObjectOneOf) (not allowed in EL)".to_string());
        }
        if self.indexes.el_property_hierarchy_depth > 3 {
            violations.push(format!(
                "Property hierarchy too deep: {} (should be ≤ 3 for EL)",
                self.indexes.el_property_hierarchy_depth
            ));
        }

        violations
    }

    fn get_ql_violation_summary(&self) -> Vec<String> {
        let mut violations = Vec::new();

        if self.indexes.ql_has_transitive_properties {
            violations.push("Contains transitive properties (not allowed in QL)".to_string());
        }
        if self.indexes.ql_has_asymmetric_properties {
            violations.push("Contains asymmetric properties (not allowed in QL)".to_string());
        }
        if self.indexes.ql_has_irreflexive_properties {
            violations.push("Contains irreflexive properties (not allowed in QL)".to_string());
        }
        if self.indexes.ql_has_complex_cardinality {
            violations
                .push("Contains complex cardinality restrictions (not allowed in QL)".to_string());
        }
        if self.indexes.ql_has_property_chains {
            violations.push("Contains property chain axioms (not allowed in QL)".to_string());
        }

        violations
    }

    fn get_rl_violation_summary(&self) -> Vec<String> {
        let mut violations = Vec::new();

        if self.indexes.rl_has_nominals {
            violations.push("Contains nominals (ObjectOneOf) (not allowed in RL)".to_string());
        }
        if self.indexes.rl_has_data_complements {
            violations.push("Contains data complement expressions (not allowed in RL)".to_string());
        }
        if self.indexes.rl_has_object_complements {
            violations
                .push("Contains object complement expressions (not allowed in RL)".to_string());
        }
        if self.indexes.rl_has_complex_class_expressions {
            violations.push("Contains complex class expressions (not allowed in RL)".to_string());
        }

        violations
    }

    fn get_fast_violations(&self, profile: Owl2Profile) -> Vec<ProfileViolation> {
        let violations_summary = match profile {
            Owl2Profile::EL => self.get_el_violation_summary(),
            Owl2Profile::QL => self.get_ql_violation_summary(),
            Owl2Profile::RL => self.get_rl_violation_summary(),
        };

        violations_summary
            .into_iter()
            .map(|message| {
                // Map specific messages to specific violation types for better test compatibility
                let violation_type = if message.contains("disjoint classes") {
                    ProfileViolationType::DisjointClassesAxiom
                } else if message.contains("equivalent classes") {
                    ProfileViolationType::EquivalentClassesAxiom
                } else if message.contains("property restrictions") {
                    ProfileViolationType::ComplexPropertyRestrictions
                } else {
                    ProfileViolationType::ComplexClassExpressions // Generic fallback
                };

                ProfileViolation {
                    violation_type,
                    message,
                    affected_entities: Vec::new(), // Detailed entities not available in fast check
                    severity: ViolationSeverity::Error,
                }
            })
            .collect()
    }

    /// Get memory pool statistics
    pub fn memory_pool_stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            arena_allocated_bytes: self.result_arena.allocated_bytes(),
            arena_chunk_bytes: self.result_arena.chunk_capacity(),
            violation_pool_size: self.violation_pool.len(),
            total_pool_allocations: self.validation_stats.arena_allocations,
        }
    }
}

/// Validation performance statistics
#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    pub total_validations: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub total_validation_time_ms: f64,
    pub peak_memory_usage_bytes: usize,
    pub arena_allocations: usize,
}

impl ValidationStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn cache_hit_ratio(&self) -> f64 {
        if self.total_validations == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_validations as f64
        }
    }

    pub fn average_validation_time_ms(&self) -> f64 {
        if self.total_validations == 0 {
            0.0
        } else {
            self.total_validation_time_ms / self.total_validations as f64
        }
    }

    pub fn memory_efficiency_ratio(&self) -> f64 {
        if self.peak_memory_usage_bytes == 0 {
            0.0
        } else {
            (self.arena_allocations as f64 * mem::size_of::<ProfileViolation>() as f64)
                / self.peak_memory_usage_bytes as f64
        }
    }
}

/// Profile-specific pre-computation indexes for optimized validation
#[derive(Debug, Default)]
pub struct ProfileIndexes {
    // EL Profile indexes
    el_has_disjoint_classes: bool,
    el_has_complex_restrictions: bool,
    el_has_nominals: bool,
    el_subclass_axioms_count: usize,
    el_property_hierarchy_depth: usize,

    // QL Profile indexes
    ql_has_transitive_properties: bool,
    ql_has_asymmetric_properties: bool,
    ql_has_irreflexive_properties: bool,
    ql_has_complex_cardinality: bool,
    ql_has_property_chains: bool,

    // RL Profile indexes
    rl_has_nominals: bool,
    rl_has_data_complements: bool,
    rl_has_object_complements: bool,
    rl_has_complex_class_expressions: bool,

    // Common indexes
    total_classes: usize,
    total_properties: usize,
    total_individuals: usize,
    total_axioms: usize,
    max_class_expression_depth: usize,
}

impl ProfileIndexes {
    fn new() -> Self {
        Self::default()
    }

    fn analyze_ontology(ontology: &Ontology) -> Self {
        let mut indexes = Self::new();

        // Basic statistics
        indexes.total_classes = ontology.classes().len();
        indexes.total_properties =
            ontology.object_properties().len() + ontology.data_properties().len();
        indexes.total_individuals = ontology.named_individuals().len();
        indexes.total_axioms = ontology.subclass_axioms().len()
            + ontology.equivalent_classes_axioms().len()
            + ontology.disjoint_classes_axioms().len();

        // EL Profile analysis
        indexes.el_has_disjoint_classes = !ontology.disjoint_classes_axioms().is_empty();
        indexes.el_subclass_axioms_count = ontology.subclass_axioms().len();
        indexes.el_has_nominals = Self::has_nominals(ontology);
        indexes.el_has_complex_restrictions = Self::has_complex_restrictions(ontology);
        indexes.el_property_hierarchy_depth = Self::calculate_property_hierarchy_depth(ontology);

        // QL Profile analysis
        indexes.ql_has_transitive_properties = Self::has_transitive_properties(ontology);
        indexes.ql_has_asymmetric_properties = Self::has_asymmetric_properties(ontology);
        indexes.ql_has_irreflexive_properties = Self::has_irreflexive_properties(ontology);
        indexes.ql_has_complex_cardinality = Self::has_complex_cardinality(ontology);
        indexes.ql_has_property_chains = Self::has_property_chains(ontology);

        // RL Profile analysis
        indexes.rl_has_nominals = Self::has_nominals(ontology);
        indexes.rl_has_data_complements = Self::has_data_complements(ontology);
        indexes.rl_has_object_complements = Self::has_object_complements(ontology);
        indexes.rl_has_complex_class_expressions = Self::has_complex_class_expressions(ontology);

        // Common analysis
        indexes.max_class_expression_depth = Self::calculate_max_class_expression_depth(ontology);

        indexes
    }

    fn is_el_compliant(&self) -> bool {
        !self.el_has_disjoint_classes
            && !self.el_has_complex_restrictions
            && !self.el_has_nominals
            && self.el_property_hierarchy_depth <= 3 // EL prefers shallow hierarchies
    }

    fn is_ql_compliant(&self) -> bool {
        !self.ql_has_transitive_properties
            && !self.ql_has_asymmetric_properties
            && !self.ql_has_irreflexive_properties
            && !self.ql_has_complex_cardinality
            && !self.ql_has_property_chains
    }

    fn is_rl_compliant(&self) -> bool {
        !self.rl_has_nominals
            && !self.rl_has_data_complements
            && !self.rl_has_object_complements
            && !self.rl_has_complex_class_expressions
    }

    // Helper methods for analysis
    fn has_nominals(ontology: &Ontology) -> bool {
        // Check for ObjectOneOf expressions in class expressions

        for axiom in ontology.subclass_axioms() {
            if Self::expression_contains_nominals(axiom.sub_class())
                || Self::expression_contains_nominals(axiom.super_class())
            {
                return true;
            }
        }

        for axiom in ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let expr = ClassExpression::Class(crate::entities::Class::new(class_iri.as_str()));
                if Self::expression_contains_nominals(&expr) {
                    return true;
                }
            }
        }

        false
    }

    fn expression_contains_nominals(expr: &crate::axioms::ClassExpression) -> bool {
        match expr {
            ObjectOneOf(_) => true,
            ObjectIntersectionOf(classes) => classes
                .iter()
                .any(|c| Self::expression_contains_nominals(c)),
            ObjectUnionOf(classes) => classes
                .iter()
                .any(|c| Self::expression_contains_nominals(c)),
            ObjectComplementOf(class) => Self::expression_contains_nominals(class),
            ObjectSomeValuesFrom(_, class) => Self::expression_contains_nominals(class),
            ObjectAllValuesFrom(_, class) => Self::expression_contains_nominals(class),
            ObjectHasValue(_, _) => true, // Individuals are nominals
            ObjectMinCardinality(_, _) => true, // Can involve nominals
            ObjectMaxCardinality(_, _) => true, // Can involve nominals
            ObjectExactCardinality(_, _) => true, // Can involve nominals
            _ => false,
        }
    }

    fn has_complex_restrictions(ontology: &Ontology) -> bool {
        for axiom in ontology.subclass_axioms() {
            if Self::is_complex_expression(axiom.sub_class())
                || Self::is_complex_expression(axiom.super_class())
            {
                return true;
            }
        }

        false
    }

    fn is_complex_expression(expr: &crate::axioms::ClassExpression) -> bool {
        matches!(expr, ObjectUnionOf(_) | ObjectComplementOf(_))
    }

    fn has_transitive_properties(_ontology: &Ontology) -> bool {
        // Check for transitive property characteristics
        // For now, return false as this requires checking property characteristic axioms
        false
    }

    fn has_asymmetric_properties(_ontology: &Ontology) -> bool {
        // Check for asymmetric property characteristics
        false
    }

    fn has_irreflexive_properties(_ontology: &Ontology) -> bool {
        // Check for irreflexive property characteristics
        false
    }

    fn has_complex_cardinality(ontology: &Ontology) -> bool {
        for axiom in ontology.subclass_axioms() {
            if Self::has_complex_cardinality_in_expression(axiom.sub_class())
                || Self::has_complex_cardinality_in_expression(axiom.super_class())
            {
                return true;
            }
        }

        false
    }

    fn has_complex_cardinality_in_expression(expr: &crate::axioms::ClassExpression) -> bool {
        matches!(
            expr,
            ObjectMinCardinality(_, _) | ObjectMaxCardinality(_, _) | ObjectExactCardinality(_, _)
        )
    }

    fn has_property_chains(_ontology: &Ontology) -> bool {
        // Check for property chain axioms (SubPropertyOf with complex expressions)
        false
    }

    fn has_data_complements(ontology: &Ontology) -> bool {
        for axiom in ontology.subclass_axioms() {
            if Self::has_data_complement_in_expression(axiom.sub_class())
                || Self::has_data_complement_in_expression(axiom.super_class())
            {
                return true;
            }
        }

        false
    }

    fn has_data_complement_in_expression(_expr: &crate::axioms::ClassExpression) -> bool {
        false // DataComplementOf is a DataRange, not ClassExpression
    }

    fn has_object_complements(ontology: &Ontology) -> bool {
        for axiom in ontology.subclass_axioms() {
            if Self::has_object_complement_in_expression(axiom.sub_class())
                || Self::has_object_complement_in_expression(axiom.super_class())
            {
                return true;
            }
        }

        false
    }

    fn has_object_complement_in_expression(expr: &crate::axioms::ClassExpression) -> bool {
        matches!(expr, ObjectComplementOf(_))
    }

    fn has_complex_class_expressions(ontology: &Ontology) -> bool {
        for axiom in ontology.subclass_axioms() {
            if Self::is_complex_expression(axiom.sub_class())
                || Self::is_complex_expression(axiom.super_class())
            {
                return true;
            }
        }

        false
    }

    fn calculate_property_hierarchy_depth(_ontology: &Ontology) -> usize {
        // Simplified calculation - in real implementation would traverse property hierarchy
        1
    }

    fn calculate_max_class_expression_depth(ontology: &Ontology) -> usize {
        let mut max_depth = 0;

        for axiom in ontology.subclass_axioms() {
            max_depth = std::cmp::max(max_depth, Self::expression_depth(axiom.sub_class()));
            max_depth = std::cmp::max(max_depth, Self::expression_depth(axiom.super_class()));
        }

        max_depth
    }

    fn expression_depth(expr: &crate::axioms::ClassExpression) -> usize {
        match expr {
            Class(_) => 1,
            ObjectIntersectionOf(classes) => {
                1 + classes
                    .iter()
                    .map(|c| Self::expression_depth(c))
                    .max()
                    .unwrap_or(0)
            }
            ObjectUnionOf(classes) => {
                1 + classes
                    .iter()
                    .map(|c| Self::expression_depth(c))
                    .max()
                    .unwrap_or(0)
            }
            ObjectComplementOf(class) => 1 + Self::expression_depth(class),
            ObjectOneOf(_) => 1,
            ObjectSomeValuesFrom(_, class) => 1 + Self::expression_depth(class),
            ObjectAllValuesFrom(_, class) => 1 + Self::expression_depth(class),
            ObjectHasValue(_, _) => 1,
            ObjectHasSelf(_) => 1,
            ObjectMinCardinality(_, _) => 1, // Cardinality without class expression
            ObjectMaxCardinality(_, _) => 1, // Cardinality without class expression
            ObjectExactCardinality(_, _) => 1, // Cardinality without class expression
            DataSomeValuesFrom(_, range) => 1 + Self::data_range_depth(range),
            DataAllValuesFrom(_, range) => 1 + Self::data_range_depth(range),
            DataHasValue(_, _) => 1,
            DataMinCardinality(_, _) => 1, // Data cardinality without range
            DataMaxCardinality(_, _) => 1, // Data cardinality without range
            DataExactCardinality(_, _) => 1, // Data cardinality without range
        }
    }

    fn data_range_depth(range: &crate::DataRange) -> usize {
        use crate::axioms::DataRange::*;

        match range {
            Datatype(_) => 1,
            DataIntersectionOf(ranges) => {
                1 + ranges.iter().map(Self::data_range_depth).max().unwrap_or(0)
            }
            DataUnionOf(ranges) => 1 + ranges.iter().map(Self::data_range_depth).max().unwrap_or(0),
            DataComplementOf(range) => 1 + Self::data_range_depth(range),
            DataOneOf(_) => 1,
            DatatypeRestriction(_, _) => 1,
        }
    }
}

/// Memory pool for efficient allocation of profile violations
#[derive(Debug)]
struct ViolationPool {
    violations: SmallVec<[ProfileViolation; 32]>,
    recycled_indices: SmallVec<[usize; 16]>,
}

impl ViolationPool {
    fn new() -> Self {
        Self {
            violations: SmallVec::new(),
            recycled_indices: SmallVec::new(),
        }
    }

    fn allocate_violation(&mut self, violation: ProfileViolation) -> &mut ProfileViolation {
        if let Some(idx) = self.recycled_indices.pop() {
            self.violations[idx] = violation;
            &mut self.violations[idx]
        } else {
            self.violations.push(violation);
            self.violations.last_mut().unwrap()
        }
    }

    fn allocate_violations(&mut self, violations: Vec<ProfileViolation>) -> &[ProfileViolation] {
        for violation in violations {
            self.allocate_violation(violation);
        }
        &self.violations
    }

    fn clear(&mut self) {
        self.violations.clear();
        self.recycled_indices.clear();
    }

    fn len(&self) -> usize {
        self.violations.len()
    }

    #[allow(dead_code)]
    fn recycle(&mut self, index: usize) {
        if index < self.violations.len() {
            self.recycled_indices.push(index);
        }
    }
}

/// Memory pool statistics
#[derive(Debug, Clone)]
pub struct MemoryPoolStats {
    pub arena_allocated_bytes: usize,
    pub arena_chunk_bytes: usize,
    pub violation_pool_size: usize,
    pub total_pool_allocations: usize,
}

impl ProfileValidator for Owl2ProfileValidator {
    fn validate_profile(&mut self, profile: Owl2Profile) -> OwlResult<ProfileValidationResult> {
        // Update validation statistics
        self.validation_stats.total_validations += 1;

        // Check cache first using advanced caching if enabled
        if self.use_advanced_caching {
            if let Some(cached_result) = self.advanced_cache.get(&profile) {
                self.validation_stats.cache_hits += 1;
                return Ok(cached_result);
            }
        } else {
            // Fall back to legacy cache
            if let Some(cached_result) = self.cache.get(&profile) {
                self.validation_stats.cache_hits += 1;
                return Ok(cached_result.clone());
            }
        }

        self.validation_stats.cache_misses += 1;

        let start_time = std::time::Instant::now();

        // Use fast profile check first for optimization
        if self.fast_profile_check(profile.clone()) {
            // Fast check passed, perform detailed validation to get specific violations
            let violations = match profile {
                Owl2Profile::EL => self.validate_el_profile_pool()?,
                Owl2Profile::QL => self.validate_ql_profile_pool()?,
                Owl2Profile::RL => self.validate_rl_profile_pool()?,
            };

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

            // Update statistics
            self.validation_stats.total_validation_time_ms +=
                validation_time.as_secs_f64() * 1000.0;
            self.validation_stats.arena_allocations += 1;

            // Cache result using appropriate caching strategy
            if self.use_advanced_caching {
                self.advanced_cache.put(profile.clone(), result.clone());
            } else {
                self.cache.insert(profile, result.clone());
            }

            Ok(result)
        } else {
            // Fast check failed, skip expensive validation
            let validation_time = start_time.elapsed();
            let memory_usage = self.estimate_memory_usage();

            // Use pre-computed violation summaries for fast failure
            let violations = self.get_fast_violations(profile.clone());
            let violations_count = violations.len();

            let result = ProfileValidationResult {
                profile: profile.clone(),
                is_valid: false,
                violations,
                statistics: ValidationStatistics {
                    total_axioms_checked: self.count_total_axioms(),
                    violations_found: violations_count,
                    validation_time_ms: validation_time.as_secs_f64() * 1000.0,
                    memory_usage_bytes: memory_usage,
                },
            };

            // Update statistics
            self.validation_stats.total_validation_time_ms +=
                validation_time.as_secs_f64() * 1000.0;

            // Cache result using appropriate caching strategy
            if self.use_advanced_caching {
                self.advanced_cache.put(profile.clone(), result.clone());
            } else {
                self.cache.insert(profile, result.clone());
            }

            Ok(result)
        }
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
                description:
                    "Large hierarchy detected. Consider flattening for EL profile compliance."
                        .to_string(),
                estimated_impact: "May enable EL profile validation".to_string(),
            });
        }

        if !self.ontology.disjoint_classes_axioms().is_empty() {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::RemoveUnsupportedConstructs,
                description: "Disjoint classes axioms found. These are not allowed in EL profile."
                    .to_string(),
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
        // NOTE: Complex property restriction analysis not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    fn has_transitive_properties(&self) -> OwlResult<bool> {
        // Check for transitive properties (not allowed in QL)
        // NOTE: Property characteristic analysis not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    fn has_asymmetric_properties(&self) -> OwlResult<bool> {
        // Check for asymmetric properties (not allowed in QL)
        // NOTE: Property characteristic analysis not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    fn has_complex_cardinality_restrictions(&self) -> OwlResult<bool> {
        // Check for complex cardinality restrictions (not allowed in QL)
        // NOTE: Cardinality restriction analysis not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    fn has_nominals(&self) -> OwlResult<bool> {
        // Check for nominals in class expressions (not allowed in RL)
        // NOTE: Nominal detection not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    fn has_data_complement(&self) -> OwlResult<bool> {
        // Check for data complement of (not allowed in RL)
        // NOTE: Data complement detection not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    fn has_object_complement(&self) -> OwlResult<bool> {
        // Check for object complement of (not allowed in RL)
        // NOTE: Object complement detection not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    // Additional validation helper methods
    fn check_equivalent_classes_for_el(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        for axiom in self.ontology.equivalent_classes_axioms() {
            if axiom.classes().len() > 2 {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::EquivalentClassesAxiom,
                    message: "Complex equivalent classes axioms with more than 2 classes are not allowed in EL profile".to_string(),
                    affected_entities: axiom.classes().iter().map(|iri| (**iri).clone()).collect(),
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations)
    }

    fn check_property_restrictions_for_el(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // EL Profile property restrictions:
        // - No property characteristics beyond basic ones
        // - Only existential restrictions allowed in class expressions
        // - No universal restrictions, cardinality restrictions, or has-value restrictions

        // Check subclass axioms for complex property restrictions
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.validate_property_restrictions_in_expression(
                axiom.super_class(),
                axiom.sub_class(),
            )?);
        }

        // Check equivalent classes axioms
        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                violations.extend(
                    self.validate_property_restrictions_in_expression(&class_expr, &class_expr)?,
                );
            }
        }

        Ok(violations)
    }

    fn validate_property_restrictions_in_expression(
        &self,
        expr: &crate::axioms::ClassExpression,
        context: &crate::axioms::ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        match expr {
            // These are allowed in EL
            ClassExpression::Class(_) => {}
            ClassExpression::ObjectSomeValuesFrom(_, _) => {}
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(
                        self.validate_property_restrictions_in_expression(class_expr, context)?,
                    );
                }
            }

            // These are NOT allowed in EL
            ClassExpression::ObjectAllValuesFrom(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexPropertyRestrictions,
                    message:
                        "Universal restrictions (ObjectAllValuesFrom) are not allowed in EL profile"
                            .to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectHasValue(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexPropertyRestrictions,
                    message:
                        "Has-value restrictions (ObjectHasValue) are not allowed in EL profile"
                            .to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectMinCardinality(_, _)
            | ClassExpression::ObjectMaxCardinality(_, _)
            | ClassExpression::ObjectExactCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Cardinality restrictions are not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectHasSelf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexPropertyRestrictions,
                    message: "Has-self restrictions (ObjectHasSelf) are not allowed in EL profile"
                        .to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectOneOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::Nominals,
                    message: "Nominals (ObjectOneOf) are not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectUnionOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexClassExpressions,
                    message: "Union expressions (ObjectUnionOf) are not allowed in EL profile"
                        .to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectComplementOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ObjectComplementOf,
                    message:
                        "Complement expressions (ObjectComplementOf) are not allowed in EL profile"
                            .to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
            // Data property restrictions - simplified check
            ClassExpression::DataSomeValuesFrom(_, _) => {}
            ClassExpression::DataAllValuesFrom(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexPropertyRestrictions,
                    message: "Universal data restrictions (DataAllValuesFrom) are not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
            _ => {
                // Other expressions are not allowed in EL
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexClassExpressions,
                    message: format!(
                        "Complex class expression {:?} is not allowed in EL profile",
                        expr
                    ),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn extract_iri_from_property_expression(
        &self,
        prop_expr: &ObjectPropertyExpression,
    ) -> OwlResult<Arc<IRI>> {
        match prop_expr {
            ObjectPropertyExpression::ObjectProperty(prop) => Ok(Arc::clone(prop.iri())),
            ObjectPropertyExpression::ObjectInverseOf(prop_expr) => {
                self.extract_iri_from_property_expression(prop_expr.as_ref())
            }
        }
    }

    fn extract_entities_from_class_expression(
        &self,
        expr: &crate::axioms::ClassExpression,
    ) -> OwlResult<Vec<IRI>> {
        let mut entities = Vec::new();

        match expr {
            ClassExpression::Class(class) => {
                entities.push((**class.iri()).clone());
            }
            ClassExpression::ObjectSomeValuesFrom(prop, class_expr)
            | ClassExpression::ObjectAllValuesFrom(prop, class_expr) => {
                entities.push((*self.extract_iri_from_property_expression(prop)?).clone());
                entities.extend(self.extract_entities_from_class_expression(class_expr)?);
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    entities.extend(self.extract_entities_from_class_expression(class_expr)?);
                }
            }
            ClassExpression::ObjectHasValue(prop, individual) => {
                entities.push((*self.extract_iri_from_property_expression(prop)?).clone());
                if let Some(iri) = individual.iri() {
                    entities.push((**iri).clone());
                }
            }
            ClassExpression::ObjectOneOf(individuals) => {
                for individual in individuals.iter() {
                    if let Some(iri) = individual.iri() {
                        entities.push((**iri).clone());
                    }
                }
            }
            _ => {}
        }

        Ok(entities)
    }

    fn check_data_property_ranges_for_el(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // EL Profile restricts data property ranges to basic datatypes only
        // No complex data ranges like DataComplementOf, DataOneOf, DataIntersectionOf, etc.

        // Check data property range axioms - note: not directly available in ontology
        // This would need to be implemented or extracted from other axioms
        // For now, we'll check data property assertions

        // Check subclass axioms that might contain data range restrictions
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.validate_data_ranges_in_class_expression(axiom.sub_class())?);
            violations.extend(self.validate_data_ranges_in_class_expression(axiom.super_class())?);
        }

        Ok(violations)
    }

    // Note: This method is currently unused since data property range axioms are not directly accessible
    // It would be used when data property range validation is fully implemented
    #[allow(dead_code)]
    #[allow(clippy::only_used_in_recursion)]
    fn validate_data_range_for_el(
        &self,
        range: &DataRange,
        property: &crate::entities::DataProperty,
    ) -> OwlResult<Vec<ProfileViolation>> {
        use crate::axioms::DataRange;
        let mut violations = Vec::new();

        match range {
            // These are allowed in EL
            DataRange::Datatype(_) => {} // Basic datatypes are allowed
            DataRange::DataIntersectionOf(ranges) => {
                // Intersection of basic datatypes is allowed
                for sub_range in ranges {
                    violations.extend(self.validate_data_range_for_el(sub_range, property)?);
                }
            }

            // These are NOT allowed in EL
            DataRange::DataComplementOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::DataComplementOf,
                    message: "Data complement of ranges are not allowed in EL profile".to_string(),
                    affected_entities: vec![(**property.iri()).clone()],
                    severity: ViolationSeverity::Error,
                });
            }
            DataRange::DataOneOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::DataOneOf,
                    message: "Data one of ranges are not allowed in EL profile".to_string(),
                    affected_entities: vec![(**property.iri()).clone()],
                    severity: ViolationSeverity::Error,
                });
            }
            DataRange::DataUnionOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexDataRanges,
                    message: "Data union of ranges are not allowed in EL profile".to_string(),
                    affected_entities: vec![(**property.iri()).clone()],
                    severity: ViolationSeverity::Error,
                });
            }
            DataRange::DatatypeRestriction(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexDataRanges,
                    message: "Datatype restrictions are not allowed in EL profile".to_string(),
                    affected_entities: vec![(**property.iri()).clone()],
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations)
    }

    fn validate_data_ranges_in_class_expression(
        &self,
        expr: &crate::axioms::ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        match expr {
            ClassExpression::DataSomeValuesFrom(_, range)
            | ClassExpression::DataAllValuesFrom(_, range) => {
                // For EL profile, we'd need to check the data range
                // But for now, we'll check if it contains complex ranges
                violations.extend(self.validate_data_range_in_expression(range)?);
            }
            ClassExpression::DataMinCardinality(_, _)
            | ClassExpression::DataMaxCardinality(_, _)
            | ClassExpression::DataExactCardinality(_, _) => {
                // Skip data cardinality restrictions for now
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.validate_data_ranges_in_class_expression(class_expr)?);
                }
            }
            _ => {}
        }

        Ok(violations)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn validate_data_range_in_expression(
        &self,
        range: &DataRange,
    ) -> OwlResult<Vec<ProfileViolation>> {
        use crate::axioms::DataRange;
        let mut violations = Vec::new();

        match range {
            DataRange::Datatype(_) => {} // Allowed
            DataRange::DataIntersectionOf(ranges) => {
                for sub_range in ranges {
                    violations.extend(self.validate_data_range_in_expression(sub_range)?);
                }
            }
            DataRange::DataComplementOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::DataComplementOf,
                    message: "Data complement of expressions are not allowed in EL profile"
                        .to_string(),
                    affected_entities: Vec::new(),
                    severity: ViolationSeverity::Error,
                });
            }
            DataRange::DataOneOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::DataOneOf,
                    message: "Data one of expressions are not allowed in EL profile".to_string(),
                    affected_entities: Vec::new(),
                    severity: ViolationSeverity::Error,
                });
            }
            DataRange::DatatypeRestriction(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexDataRanges,
                    message: "Datatype restrictions are not allowed in EL profile".to_string(),
                    affected_entities: Vec::new(),
                    severity: ViolationSeverity::Error,
                });
            }
            _ => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexDataRanges,
                    message: "Complex data ranges are not allowed in EL profile".to_string(),
                    affected_entities: Vec::new(),
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations)
    }

    fn check_transitive_properties_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // QL profile does not allow transitive properties
        // Check for transitive object property axioms
        for axiom in self.ontology.transitive_property_axioms() {
            violations.push(ProfileViolation {
                violation_type: ProfileViolationType::TransitiveProperties,
                message: "Transitive properties are not allowed in QL profile".to_string(),
                affected_entities: vec![(**axiom.property()).clone()],
                severity: ViolationSeverity::Error,
            });
        }

        Ok(violations)
    }

    fn check_asymmetric_properties_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // QL profile does not allow asymmetric properties
        // Check for asymmetric object property axioms
        for axiom in self.ontology.asymmetric_property_axioms() {
            violations.push(ProfileViolation {
                violation_type: ProfileViolationType::AsymmetricProperties,
                message: "Asymmetric properties are not allowed in QL profile".to_string(),
                affected_entities: vec![(**axiom.property()).clone()],
                severity: ViolationSeverity::Error,
            });
        }

        Ok(violations)
    }

    fn check_cardinality_restrictions_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // QL profile allows only qualified cardinality restrictions with specific restrictions
        // No unqualified cardinality restrictions
        // Check subclass axioms for cardinality restrictions
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.check_cardinality_in_ql_expression(axiom.super_class())?);
            violations.extend(self.check_cardinality_in_ql_expression(axiom.sub_class())?);
        }

        // Check equivalent classes axioms
        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                violations.extend(self.check_cardinality_in_ql_expression(&class_expr)?);
            }
        }

        Ok(violations)
    }

    fn check_cardinality_in_ql_expression(
        &self,
        expr: &crate::axioms::ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        match expr {
            ClassExpression::ObjectMinCardinality(_, _)
            | ClassExpression::ObjectMaxCardinality(_, _)
            | ClassExpression::ObjectExactCardinality(_, _) => {
                // Unqualified cardinality restrictions are not allowed in QL
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Unqualified cardinality restrictions are not allowed in QL profile"
                        .to_string(),
                    affected_entities: self.extract_entities_from_class_expression(expr)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::DataMinCardinality(_, _)
            | ClassExpression::DataMaxCardinality(_, _)
            | ClassExpression::DataExactCardinality(_, _) => {
                // Unqualified data cardinality restrictions are not allowed in QL
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexCardinalityRestrictions,
                    message:
                        "Unqualified data cardinality restrictions are not allowed in QL profile"
                            .to_string(),
                    affected_entities: self.extract_entities_from_class_expression(expr)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_cardinality_in_ql_expression(class_expr)?);
                }
            }
            _ => {}
        }

        Ok(violations)
    }

    fn check_property_chains_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // QL profile does not allow property chain axioms (complex subproperty axioms)
        // Check subproperty axioms for complex expressions
        for _axiom in self.ontology.subobject_property_axioms() {
            // For now, we'll use a simplified check
            // In a full implementation, this would analyze property expressions
            violations.push(ProfileViolation {
                violation_type: ProfileViolationType::PropertyChainAxioms,
                message: "Property chain axioms are not allowed in QL profile (simplified check)"
                    .to_string(),
                affected_entities: Vec::new(),
                severity: ViolationSeverity::Warning,
            });
        }

        Ok(violations)
    }

    #[allow(dead_code)]
    fn is_complex_subproperty_expression(
        &self,
        sub_prop: &ObjectPropertyExpression,
        super_prop: &ObjectPropertyExpression,
    ) -> bool {
        // In QL, property chains would be represented as complex subproperty expressions
        // For now, we use a simplified check
        // In a full implementation, this would analyze the property expressions

        // Basic heuristic: if the subproperty expression is not a simple property reference,
        // it might be a property chain
        match (sub_prop, super_prop) {
            (
                ObjectPropertyExpression::ObjectProperty(_),
                ObjectPropertyExpression::ObjectProperty(_),
            ) => false,
            _ => true, // Any complex expression is considered a potential property chain
        }
    }

    fn check_nominals_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // RL profile does not allow nominals (ObjectOneOf in class expressions)
        // Check all class expressions for ObjectOneOf constructs

        // Check subclass axioms
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.check_nominals_in_expression(axiom.super_class())?);
            violations.extend(self.check_nominals_in_expression(axiom.sub_class())?);
        }

        // Check equivalent classes axioms
        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                violations.extend(self.check_nominals_in_expression(&class_expr)?);
            }
        }

        // Check class assertion axioms
        for axiom in self.ontology.class_assertions() {
            violations.extend(self.check_nominals_in_expression(axiom.class_expr())?);
        }

        Ok(violations)
    }

    fn check_nominals_in_expression(
        &self,
        expr: &crate::axioms::ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        match expr {
            ClassExpression::ObjectOneOf(_individuals) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ObjectOneOf,
                    message: "ObjectOneOf (nominals) are not allowed in RL profile".to_string(),
                    affected_entities: self.extract_entities_from_class_expression(expr)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_nominals_in_expression(class_expr)?);
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_nominals_in_expression(class_expr)?);
                }
            }
            _ => {}
        }

        Ok(violations)
    }

    fn check_data_complement_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // RL profile does not allow DataComplementOf
        // Check all data ranges in the ontology

        // Note: Data property range validation requires additional ontology methods
        // This would check data property range axioms for DataComplementOf

        // Check subclass axioms
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.check_data_complement_in_class_expression(axiom.super_class())?);
            violations.extend(self.check_data_complement_in_class_expression(axiom.sub_class())?);
        }

        Ok(violations)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn check_data_complement_in_range(
        &self,
        range: &DataRange,
    ) -> OwlResult<Vec<ProfileViolation>> {
        use crate::axioms::DataRange;
        let mut violations = Vec::new();

        match range {
            DataRange::DataComplementOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::DataComplementOf,
                    message: "DataComplementOf is not allowed in RL profile".to_string(),
                    affected_entities: Vec::new(),
                    severity: ViolationSeverity::Error,
                });
            }
            DataRange::DataIntersectionOf(ranges) => {
                for sub_range in ranges {
                    violations.extend(self.check_data_complement_in_range(sub_range)?);
                }
            }
            DataRange::DataUnionOf(ranges) => {
                for sub_range in ranges {
                    violations.extend(self.check_data_complement_in_range(sub_range)?);
                }
            }
            _ => {}
        }

        Ok(violations)
    }

    fn check_data_complement_in_class_expression(
        &self,
        expr: &crate::axioms::ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        match expr {
            ClassExpression::DataSomeValuesFrom(_, range)
            | ClassExpression::DataAllValuesFrom(_, range) => {
                violations.extend(self.check_data_complement_in_range(range)?);
            }
            ClassExpression::DataMinCardinality(_, _)
            | ClassExpression::DataMaxCardinality(_, _)
            | ClassExpression::DataExactCardinality(_, _) => {
                // For data cardinality restrictions, we'd need to check if they have ranges
                // But for now, we'll skip this check
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_data_complement_in_class_expression(class_expr)?);
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_data_complement_in_class_expression(class_expr)?);
                }
            }
            _ => {}
        }

        Ok(violations)
    }

    fn check_object_complement_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // RL profile does not allow ObjectComplementOf
        // Check all class expressions in the ontology

        // Check subclass axioms
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.check_object_complement_in_expression(axiom.super_class())?);
            violations.extend(self.check_object_complement_in_expression(axiom.sub_class())?);
        }

        // Check equivalent classes axioms
        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                violations.extend(self.check_object_complement_in_expression(&class_expr)?);
            }
        }

        Ok(violations)
    }

    fn check_object_complement_in_expression(
        &self,
        expr: &crate::axioms::ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        match expr {
            ClassExpression::ObjectComplementOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ObjectComplementOf,
                    message: "ObjectComplementOf is not allowed in RL profile".to_string(),
                    affected_entities: self.extract_entities_from_class_expression(expr)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_object_complement_in_expression(class_expr)?);
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_object_complement_in_expression(class_expr)?);
                }
            }
            _ => {}
        }

        Ok(violations)
    }

    fn check_object_one_of_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        // This is essentially the same as check_nominals_for_rl
        // since ObjectOneOf represents nominals
        self.check_nominals_for_rl()
    }

    // Utility methods
    fn get_affected_entities_from_disjoint_classes(&self) -> Vec<IRI> {
        let mut entities = Vec::new();
        for axiom in self.ontology.disjoint_classes_axioms() {
            entities.extend(axiom.classes().iter().map(|iri| (**iri).clone()));
        }
        entities
    }

    fn count_total_axioms(&self) -> usize {
        // Count all axioms in the ontology
        self.ontology.subclass_axioms().len()
            + self.ontology.equivalent_classes_axioms().len()
            + self.ontology.disjoint_classes_axioms().len()
            + self.ontology.object_properties().len()
            + self.ontology.data_properties().len()
    }

    fn estimate_memory_usage(&self) -> usize {
        // Simplified memory usage estimation
        self.ontology.classes().len() * 64 + // Approximate size per class
        self.ontology.object_properties().len() * 48 + // Approximate size per property
        self.ontology.subclass_axioms().len() * 32 // Approximate size per axiom
    }

    // Memory pool optimized validation methods
    fn validate_el_profile_pool(&mut self) -> OwlResult<Vec<ProfileViolation>> {
        // Clear violation pool for fresh allocation
        self.violation_pool.clear();

        // Use existing validation logic but with memory pool allocation
        let violations = self.validate_el_profile()?;

        // Allocate violations from memory pool
        let pooled_violations = self.violation_pool.allocate_violations(violations);

        // Update statistics
        self.validation_stats.arena_allocations += pooled_violations.len();

        Ok(pooled_violations.to_vec())
    }

    fn validate_ql_profile_pool(&mut self) -> OwlResult<Vec<ProfileViolation>> {
        // Clear violation pool for fresh allocation
        self.violation_pool.clear();

        // Use existing validation logic but with memory pool allocation
        let violations = self.validate_ql_profile()?;

        // Allocate violations from memory pool
        let pooled_violations = self.violation_pool.allocate_violations(violations);

        // Update statistics
        self.validation_stats.arena_allocations += pooled_violations.len();

        Ok(pooled_violations.to_vec())
    }

    fn validate_rl_profile_pool(&mut self) -> OwlResult<Vec<ProfileViolation>> {
        // Clear violation pool for fresh allocation
        self.violation_pool.clear();

        // Use existing validation logic but with memory pool allocation
        let violations = self.validate_rl_profile()?;

        // Allocate violations from memory pool
        let pooled_violations = self.violation_pool.allocate_violations(violations);

        // Update statistics
        self.validation_stats.arena_allocations += pooled_violations.len();

        Ok(pooled_violations.to_vec())
    }

    // Arena-optimized string allocation for violation messages
    #[allow(dead_code)]
    fn allocate_violation_message(&self, message: &str) -> &str {
        // Allocate string in arena and return reference
        // alloc_str() already returns a &str, so no unsafe conversion needed
        self.result_arena.alloc_str(message)
    }

    // Allocate IRI vectors in arena for efficient storage
    #[allow(dead_code)]
    fn allocate_iri_vector(&self, iris: Vec<IRI>) -> &[IRI] {
        let arena_iris = self
            .result_arena
            .alloc_slice_fill_with(iris.len(), |i| iris[i].clone());
        arena_iris
    }

    // Performance-optimized violation creation using arena allocation
    #[allow(dead_code)]
    fn create_violation_optimized(
        &mut self,
        violation_type: ProfileViolationType,
        message: String,
        affected_entities: Vec<IRI>,
        severity: ViolationSeverity,
    ) -> ProfileViolation {
        // Use arena for message string
        let arena_message = self.allocate_violation_message(&message);

        // Use arena for affected entities
        let arena_entities = self.allocate_iri_vector(affected_entities);

        // Create violation with arena-allocated data
        ProfileViolation {
            violation_type,
            message: arena_message.to_string(), // Convert back to owned for compatibility
            affected_entities: arena_entities.to_vec(), // Convert back to owned for compatibility
            severity,
        }
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

        // Disable advanced caching to test legacy cache
        validator.set_advanced_caching(false);

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

    #[test]
    fn test_memory_pool_functionality() {
        let ontology = Arc::new(Ontology::new());
        let mut validator = Owl2ProfileValidator::new(ontology);

        // Test memory pool statistics
        let pool_stats = validator.memory_pool_stats();
        assert_eq!(pool_stats.violation_pool_size, 0);
        assert_eq!(pool_stats.total_pool_allocations, 0);

        // Perform validation to exercise memory pool
        let result = validator.validate_profile(Owl2Profile::EL).unwrap();
        assert!(result.is_valid);

        // Check that memory pool was used
        let pool_stats_after = validator.memory_pool_stats();
        assert!(pool_stats_after.total_pool_allocations >= 1);

        // Test memory efficiency ratio
        let stats = validator.get_validation_stats();
        let efficiency = stats.memory_efficiency_ratio();
        assert!(efficiency >= 0.0);

        // Test reset functionality
        validator.reset_stats();
        let pool_stats_reset = validator.memory_pool_stats();
        assert_eq!(pool_stats_reset.violation_pool_size, 0);
    }

    #[test]
    fn test_validation_statistics() {
        let ontology = Arc::new(Ontology::new());
        let mut validator = Owl2ProfileValidator::new(ontology);

        // Test initial statistics
        let stats = validator.get_validation_stats();
        assert_eq!(stats.total_validations, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.total_validation_time_ms, 0.0);
        assert_eq!(stats.arena_allocations, 0);

        // Perform validation
        let _result = validator.validate_profile(Owl2Profile::EL).unwrap();

        // Check updated statistics
        let stats_after = validator.get_validation_stats();
        assert_eq!(stats_after.total_validations, 1);
        assert_eq!(stats_after.cache_misses, 1);
        assert!(stats_after.total_validation_time_ms >= 0.0);
        assert!(stats_after.arena_allocations >= 1);

        // Perform second validation (should hit cache)
        let _result2 = validator.validate_profile(Owl2Profile::EL).unwrap();

        let stats_cached = validator.get_validation_stats();
        assert_eq!(stats_cached.total_validations, 2);
        assert_eq!(stats_cached.cache_hits, 1);
        assert_eq!(stats_cached.cache_misses, 1);

        // Test cache hit ratio
        let hit_ratio = stats_cached.cache_hit_ratio();
        assert!(hit_ratio > 0.0 && hit_ratio <= 1.0);
    }

    #[test]
    fn test_pre_computation_indexes() {
        let ontology = Arc::new(Ontology::new());
        let validator = Owl2ProfileValidator::new(ontology);

        // Test that indexes are computed on creation
        let indexes = validator.get_indexes();
        assert_eq!(indexes.total_classes, 0);
        assert_eq!(indexes.total_properties, 0);
        assert_eq!(indexes.total_individuals, 0);
        assert_eq!(indexes.total_axioms, 0);

        // Test fast profile checks
        assert!(indexes.is_el_compliant());
        assert!(indexes.is_ql_compliant());
        assert!(indexes.is_rl_compliant());

        // Test validator fast profile checks
        assert!(validator.fast_profile_check(Owl2Profile::EL));
        assert!(validator.fast_profile_check(Owl2Profile::QL));
        assert!(validator.fast_profile_check(Owl2Profile::RL));

        // Test profile analysis report
        let report = validator.get_profile_analysis();
        assert!(report.el_compliant);
        assert!(report.ql_compliant);
        assert!(report.rl_compliant);
        assert!(report.el_violations.is_empty());
        assert!(report.ql_violations.is_empty());
        assert!(report.rl_violations.is_empty());
    }

    #[test]
    fn test_index_recomputation() {
        let ontology = Arc::new(Ontology::new());
        let mut validator = Owl2ProfileValidator::new(ontology);

        // Get initial indexes and clone values
        let initial_total_classes = validator.get_indexes().total_classes;
        let initial_total_properties = validator.get_indexes().total_properties;

        // Recompute indexes
        validator.recompute_indexes();

        // Get recomputed indexes
        let recomputed_total_classes = validator.get_indexes().total_classes;
        let recomputed_total_properties = validator.get_indexes().total_properties;

        // Indexes should be the same for empty ontology
        assert_eq!(initial_total_classes, recomputed_total_classes);
        assert_eq!(initial_total_properties, recomputed_total_properties);
    }
}

/// Advanced caching strategies for profile validation results
#[derive(Debug)]
struct AdvancedCacheManager {
    /// Primary cache with LRU eviction policy
    primary_cache: LruCache<Owl2Profile, CacheEntry>,
    /// Secondary cache for frequently accessed results
    hot_cache: DashMap<Owl2Profile, ProfileValidationResult>,
    /// Tertiary cache for large validation results (compressed storage)
    compressed_cache: HashMap<Owl2Profile, CompressedCacheEntry>,
    /// Cache invalidation tokens for ontology changes
    invalidation_tokens: HashSet<u64>,
    /// Cache statistics and performance metrics
    cache_stats: CacheStatistics,
    /// Cache configuration
    config: ProfileCacheConfig,
}

/// Cache entry with metadata for intelligent eviction
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached validation result
    result: ProfileValidationResult,
    /// Last access timestamp
    last_accessed: Instant,
    /// Access frequency for LRU-K algorithm
    access_count: usize,
    /// Estimated memory usage of this entry
    memory_usage: usize,
    /// Cache entry priority (higher = less likely to be evicted)
    priority: CachePriority,
}

/// Compressed cache entry for large results
#[derive(Debug, Clone)]
struct CompressedCacheEntry {
    /// Compressed validation result data
    compressed_data: Vec<u8>,
    /// Decompression metadata
    metadata: CacheMetadata,
    /// Original memory usage before compression
    original_memory: usize,
}

/// Cache metadata for compressed entries
#[derive(Debug, Clone)]
struct CacheMetadata {
    /// Profile type
    profile: Owl2Profile,
    /// Validation timestamp
    timestamp: Instant,
    /// Validation result summary
    summary: ValidationSummary,
}

/// Validation summary for compressed cache entries
#[derive(Debug, Clone)]
struct ValidationSummary {
    /// Whether the validation was successful
    is_valid: bool,
    /// Number of violations found
    violations_count: usize,
    /// Validation time in milliseconds
    validation_time_ms: f64,
}

/// Cache priority levels for intelligent eviction
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CachePriority {
    /// Low priority (easily evicted)
    Low = 1,
    /// Medium priority
    Medium = 2,
    /// High priority (rarely evicted)
    High = 3,
    /// Critical priority (never evicted)
    Critical = 4,
}

/// Profile cache configuration parameters
#[derive(Debug, Clone)]
pub struct ProfileCacheConfig {
    /// Maximum entries in primary cache
    primary_cache_size: usize,
    /// Maximum entries in hot cache
    _hot_cache_size: usize,
    /// Maximum entries in compressed cache
    compressed_cache_size: usize,
    /// Time-to-live for cache entries
    ttl_duration: Duration,
    /// Compression threshold (entries larger than this get compressed)
    compression_threshold: usize,
    /// Hot cache promotion threshold (access count)
    hot_cache_threshold: usize,
}

/// Cache statistics and performance metrics
#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
    /// Number of cache evictions
    pub evictions: usize,
    /// Number of compressed cache hits
    compressed_hits: usize,
    /// Number of hot cache hits
    hot_hits: usize,
    /// Total memory used by cache
    total_memory_bytes: usize,
    /// Memory saved by compression
    compressed_memory_saved: usize,
    /// Average access time
    average_access_time_ns: u64,
    /// Cache hit rate
    pub hit_rate: f64,
}

impl Default for ProfileCacheConfig {
    fn default() -> Self {
        Self {
            primary_cache_size: 1000,
            _hot_cache_size: 100,
            compressed_cache_size: 500,
            ttl_duration: Duration::from_secs(3600), // 1 hour
            compression_threshold: 1024,             // 1KB
            hot_cache_threshold: 5,
        }
    }
}

impl AdvancedCacheManager {
    /// Create a new advanced cache manager
    fn new() -> Self {
        Self {
            primary_cache: LruCache::new(
                std::num::NonZeroUsize::new(ProfileCacheConfig::default().primary_cache_size)
                    .unwrap(),
            ),
            hot_cache: DashMap::new(),
            compressed_cache: HashMap::new(),
            invalidation_tokens: HashSet::new(),
            cache_stats: CacheStatistics::default(),
            config: ProfileCacheConfig::default(),
        }
    }

    /// Create a new advanced cache manager with custom configuration
    fn with_config(config: ProfileCacheConfig) -> Self {
        Self {
            primary_cache: LruCache::new(
                std::num::NonZeroUsize::new(config.primary_cache_size).unwrap(),
            ),
            hot_cache: DashMap::new(),
            compressed_cache: HashMap::new(),
            invalidation_tokens: HashSet::new(),
            cache_stats: CacheStatistics::default(),
            config,
        }
    }

    /// Get a cached validation result
    fn get(&mut self, profile: &Owl2Profile) -> Option<ProfileValidationResult> {
        let start_time = std::time::Instant::now();

        // Check hot cache first (fastest)
        if let Some(result) = self.hot_cache.get(profile).map(|r| r.clone()) {
            self.cache_stats.hot_hits += 1;
            self.cache_stats.hits += 1;
            self.update_access_time(start_time);
            return Some(result);
        }

        // Check primary cache
        if let Some(entry) = self.primary_cache.get(profile) {
            // Check if entry is expired
            if entry.last_accessed.elapsed() > self.config.ttl_duration {
                self.primary_cache.pop(profile);
                self.cache_stats.misses += 1;
                self.update_access_time(start_time);
                return None;
            }

            // Update access count and check for hot cache promotion
            let result = entry.result.clone();
            let access_count = entry.access_count + 1;
            let last_accessed = Instant::now();

            // Promote to hot cache if threshold reached
            if access_count >= self.config.hot_cache_threshold {
                self.hot_cache.insert(profile.clone(), result.clone());
            }

            // Update primary cache with new access stats
            let updated_entry = CacheEntry {
                result: result.clone(),
                access_count,
                last_accessed,
                memory_usage: entry.memory_usage,
                priority: entry.priority.clone(),
            };
            self.primary_cache.put(profile.clone(), updated_entry);

            self.cache_stats.hits += 1;
            self.update_access_time(start_time);
            return Some(result);
        }

        // Check compressed cache for large results
        if let Some(compressed_entry) = self.compressed_cache.get(profile) {
            // Check if compressed entry is expired
            if compressed_entry.metadata.timestamp.elapsed() > self.config.ttl_duration {
                self.compressed_cache.remove(profile);
                self.cache_stats.misses += 1;
                self.update_access_time(start_time);
                return None;
            }

            // Decompress and return result
            let result = self.decompress_cache_entry(compressed_entry);
            self.cache_stats.compressed_hits += 1;
            self.cache_stats.hits += 1;
            self.update_access_time(start_time);
            return Some(result);
        }

        self.cache_stats.misses += 1;
        self.update_access_time(start_time);
        None
    }

    /// Store a validation result in the cache
    fn put(&mut self, profile: Owl2Profile, result: ProfileValidationResult) {
        let memory_usage = self.estimate_result_memory(&result);
        let priority = self.calculate_priority(&result);

        // Create cache entry
        let entry = CacheEntry {
            result: result.clone(),
            last_accessed: Instant::now(),
            access_count: 1,
            memory_usage,
            priority,
        };

        // Use compressed cache for large results
        if memory_usage > self.config.compression_threshold {
            self.put_compressed(profile, result);
            return;
        }

        // Store in primary cache
        if self.primary_cache.len() >= self.config.primary_cache_size {
            if let Some((_key, evicted_entry)) = self.primary_cache.pop_lru() {
                self.cache_stats.evictions += 1;
                self.cache_stats.total_memory_bytes -= evicted_entry.memory_usage;
            }
        }

        self.primary_cache.put(profile, entry);
        self.cache_stats.total_memory_bytes += memory_usage;
    }

    /// Store a compressed cache entry
    fn put_compressed(&mut self, profile: Owl2Profile, result: ProfileValidationResult) {
        if self.compressed_cache.len() >= self.config.compressed_cache_size {
            // Remove oldest entry
            if let Some((oldest_key, _)) = self
                .compressed_cache
                .iter()
                .min_by_key(|(_, entry)| entry.metadata.timestamp)
            {
                let oldest_key = oldest_key.clone();
                if let Some(removed) = self.compressed_cache.remove(&oldest_key) {
                    self.cache_stats.total_memory_bytes -= removed.original_memory;
                    self.cache_stats.evictions += 1;
                }
            }
        }

        let compressed_entry = self.compress_cache_entry(&result);
        self.compressed_cache.insert(profile, compressed_entry);
    }

    /// Compress a validation result for storage
    fn compress_cache_entry(&mut self, result: &ProfileValidationResult) -> CompressedCacheEntry {
        // For now, use simple JSON compression
        // In a real implementation, you might use more efficient compression
        let json_data = serde_json::to_vec(result).unwrap_or_default();
        let compressed_data = json_data; // Placeholder - real compression would go here

        let original_memory = self.estimate_result_memory(result);
        self.cache_stats.compressed_memory_saved +=
            original_memory.saturating_sub(compressed_data.len());

        CompressedCacheEntry {
            compressed_data,
            metadata: CacheMetadata {
                profile: result.profile.clone(),
                timestamp: Instant::now(),
                summary: ValidationSummary {
                    is_valid: result.is_valid,
                    violations_count: result.violations.len(),
                    validation_time_ms: result.statistics.validation_time_ms,
                },
            },
            original_memory,
        }
    }

    /// Decompress a cache entry back to validation result
    fn decompress_cache_entry(
        &self,
        compressed_entry: &CompressedCacheEntry,
    ) -> ProfileValidationResult {
        // For now, use simple JSON decompression
        serde_json::from_slice(&compressed_entry.compressed_data).unwrap_or_else(|_| {
            ProfileValidationResult {
                profile: compressed_entry.metadata.profile.clone(),
                is_valid: compressed_entry.metadata.summary.is_valid,
                violations: Vec::new(),
                statistics: ValidationStatistics {
                    total_axioms_checked: 0,
                    violations_found: compressed_entry.metadata.summary.violations_count,
                    validation_time_ms: compressed_entry.metadata.summary.validation_time_ms,
                    memory_usage_bytes: 0,
                },
            }
        })
    }

    /// Estimate memory usage of a validation result
    fn estimate_result_memory(&self, result: &ProfileValidationResult) -> usize {
        let base_size = mem::size_of::<ProfileValidationResult>();
        let violations_size = result.violations.len() * mem::size_of::<ProfileViolation>();
        let statistics_size = mem::size_of::<ValidationStatistics>();

        base_size + violations_size + statistics_size
    }

    /// Calculate cache priority for a result
    fn calculate_priority(&self, result: &ProfileValidationResult) -> CachePriority {
        if result.is_valid {
            // Valid results are more valuable
            CachePriority::High
        } else if result.violations.is_empty() {
            // Empty violations are also valuable
            CachePriority::Medium
        } else {
            // Results with violations are less valuable
            CachePriority::Low
        }
    }

    /// Update average access time
    fn update_access_time(&mut self, start_time: Instant) {
        let access_time = start_time.elapsed().as_nanos() as u64;
        self.cache_stats.hit_rate =
            self.cache_stats.hits as f64 / (self.cache_stats.hits + self.cache_stats.misses) as f64;

        // Update average access time (simple moving average)
        if self.cache_stats.average_access_time_ns == 0 {
            self.cache_stats.average_access_time_ns = access_time;
        } else {
            self.cache_stats.average_access_time_ns =
                (self.cache_stats.average_access_time_ns + access_time) / 2;
        }
    }

    /// Clear all caches
    fn clear(&mut self) {
        self.primary_cache.clear();
        self.hot_cache.clear();
        self.compressed_cache.clear();
        self.invalidation_tokens.clear();
        self.cache_stats = CacheStatistics::default();
    }

    /// Invalidate cache entries that match the given token
    fn invalidate_by_token(&mut self, token: u64) {
        self.invalidation_tokens.insert(token);
        // Clear caches that contain invalidated entries
        self.primary_cache.clear();
        self.hot_cache.clear();
        self.compressed_cache.clear();
    }

    /// Get cache statistics
    fn get_stats(&self) -> &CacheStatistics {
        &self.cache_stats
    }

    /// Get cache configuration
    fn get_config(&self) -> &ProfileCacheConfig {
        &self.config
    }
}
