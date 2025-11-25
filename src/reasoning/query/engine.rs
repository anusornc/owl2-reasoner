//! Query engine implementation for OWL2 ontologies
//!
//! Contains the main QueryEngine struct and core query processing logic.

use crate::axioms::*;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::reasoning::Reasoner;

use super::{
    compute_config_hash, create_cache_key, QueryCache, QueryConfig, QueryEngineStats, QueryPattern,
    QueryResult, QueryType, ResultPool, TriplePattern, RDF_TYPE,
};

use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;

/// Query engine for OWL2 ontologies with advanced optimizations
pub struct QueryEngine {
    ontology: Arc<Ontology>,
    #[allow(dead_code)]
    reasoner: Option<Box<dyn Reasoner>>,
    config: QueryConfig,
    /// Query result cache with LRU eviction
    query_cache: Arc<QueryCache>,
    /// Memory pool for reusing allocations
    #[allow(dead_code)]
    result_pool: Arc<ResultPool>,
    /// Index-based access structures for fast pattern matching
    #[allow(dead_code)]
    type_index: Arc<DashMap<Arc<IRI>, Vec<Arc<ClassAssertionAxiom>>>>,
    #[allow(dead_code)]
    property_index: Arc<DashMap<Arc<IRI>, Vec<Arc<PropertyAssertionAxiom>>>>,
    /// Query execution statistics
    stats: Arc<RwLock<QueryEngineStats>>,
}

impl QueryEngine {
    /// Create a new query engine
    pub fn new(ontology: Ontology) -> Self {
        Self::with_config(ontology, QueryConfig::default())
    }

    /// Create a new query engine with custom configuration
    pub fn with_config(ontology: Ontology, config: QueryConfig) -> Self {
        let ontology = Arc::new(ontology);

        Self {
            query_cache: Arc::new(if let Some(size) = config.cache_size {
                QueryCache::new(size)
            } else {
                QueryCache::default()
            }),
            result_pool: Arc::new(ResultPool::new()),
            type_index: Arc::new(DashMap::new()),
            property_index: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(QueryEngineStats::new())),
            ontology,
            reasoner: None, // TODO: Initialize reasoner
            config,
        }
    }

    /// Execute a query pattern
    pub fn execute(&self, pattern: &QueryPattern) -> OwlResult<QueryResult> {
        let start_time = std::time::Instant::now();

        // Compute cache key
        let pattern_hash = super::compute_pattern_hash(pattern);
        let config_hash = compute_config_hash(
            self.config.enable_reasoning,
            self.config.enable_parallel,
            self.config.max_results,
        );
        let cache_key = create_cache_key(pattern_hash, config_hash);

        // Check cache
        if self.config.enable_caching {
            if let Some(cached_result) = self.query_cache.get(&cache_key) {
                let mut stats = self.stats.write();
                stats.record_cache_hit();
                stats.record_success(start_time.elapsed().as_millis() as u64);
                return Ok(cached_result);
            } else {
                let mut stats = self.stats.write();
                stats.record_cache_miss();
            }
        }

        // Execute query
        let result = if self.config.enable_parallel && pattern.supports_parallel() {
            self.execute_parallel(pattern)?
        } else {
            self.execute_sequential(pattern)?
        };

        // Cache result
        if self.config.enable_caching {
            self.query_cache.put(cache_key, result.clone());
        }

        // Record statistics
        let elapsed = start_time.elapsed().as_millis() as u64;
        let mut stats = self.stats.write();
        stats.record_success(elapsed);
        stats.record_reasoning_operation();

        Ok(result)
    }

    /// Execute a triple pattern query
    pub fn execute_triple(&self, triple: TriplePattern) -> OwlResult<QueryResult> {
        let pattern = QueryPattern::BasicGraphPattern(vec![triple]);
        self.execute(&pattern)
    }

    /// Execute a basic class query (get all instances of a class)
    pub fn get_class_instances(&self, class_iri: &IRI) -> OwlResult<QueryResult> {

        // Get class assertions
        let instances: Vec<IRI> = self
            .ontology
            .class_assertions()
            .iter()
            .filter(|axiom| axiom.class_expr().contains_class(class_iri))
            .map(|axiom| (**axiom.individual()).clone())
            .collect();

        // Create query result
        let mut result = QueryResult::new();
        result.variables = vec!["instance".to_string()];

        for instance in instances {
            let mut binding = super::QueryBinding::new();
            binding.add_binding("instance".to_string(), super::QueryValue::IRI(instance));
            result.add_binding(binding);
        }

        result.stats.results_count = result.len();
        result.stats.reasoning_used = self.config.enable_reasoning;

        Ok(result)
    }

    /// Execute a basic property query (get all property values for a subject)
    pub fn get_property_values(
        &self,
        subject_iri: &IRI,
        property_iri: &IRI,
    ) -> OwlResult<QueryResult> {

        // Get property assertions
        let values: Vec<super::QueryValue> = self
            .ontology
            .property_assertions()
            .iter()
            .filter(|axiom| {
                (**axiom.subject()) == *subject_iri && (**axiom.property()) == *property_iri
            })
            .filter_map(|axiom| match axiom.object() {
                PropertyAssertionObject::Named(individual) => {
                    Some(super::QueryValue::IRI((**individual).clone()))
                }
                PropertyAssertionObject::Anonymous(_) => None,
            })
            .collect();

        // Create query result
        let mut result = QueryResult::new();
        result.variables = vec!["value".to_string()];

        for value in values {
            let mut binding = super::QueryBinding::new();
            binding.add_binding("value".to_string(), value);
            result.add_binding(binding);
        }

        result.stats.results_count = result.len();
        result.stats.reasoning_used = self.config.enable_reasoning;

        Ok(result)
    }

    /// Get all classes in the ontology
    pub fn get_all_classes(&self) -> OwlResult<QueryResult> {
        let classes: Vec<IRI> = self
            .ontology
            .classes()
            .iter()
            .map(|class| (**class.iri()).clone())
            .collect();

        // Create query result
        let mut result = QueryResult::new();
        result.variables = vec!["class".to_string()];

        for class in classes {
            let mut binding = super::QueryBinding::new();
            binding.add_binding("class".to_string(), super::QueryValue::IRI(class));
            result.add_binding(binding);
        }

        result.stats.results_count = result.len();

        Ok(result)
    }

    /// Get all individuals in the ontology
    pub fn get_all_individuals(&self) -> OwlResult<QueryResult> {
        let individuals: Vec<IRI> = self
            .ontology
            .named_individuals()
            .iter()
            .map(|individual| (**individual.iri()).clone())
            .collect();

        // Create query result
        let mut result = QueryResult::new();
        result.variables = vec!["individual".to_string()];

        for individual in individuals {
            let mut binding = super::QueryBinding::new();
            binding.add_binding("individual".to_string(), super::QueryValue::IRI(individual));
            result.add_binding(binding);
        }

        result.stats.results_count = result.len();

        Ok(result)
    }

    /// Get engine statistics
    pub fn stats(&self) -> QueryEngineStats {
        self.stats.read().clone()
    }

    /// Get engine configuration
    pub fn config(&self) -> &QueryConfig {
        &self.config
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.stats.write().reset();
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        self.query_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        self.query_cache.stats()
    }

    // Private methods

    /// Execute query in parallel
    fn execute_parallel(&self, pattern: &QueryPattern) -> OwlResult<QueryResult> {
        let mut stats = self.stats.write();
        stats.record_parallel_execution();

        // For now, fall back to sequential execution
        // TODO: Implement proper parallel execution
        self.execute_sequential(pattern)
    }

    /// Execute query sequentially
    fn execute_sequential(&self, pattern: &QueryPattern) -> OwlResult<QueryResult> {
        match pattern {
            QueryPattern::BasicGraphPattern(triples) => self.execute_basic_graph_pattern(triples),
            QueryPattern::Optional { left, right } => self.execute_optional_pattern(left, right),
            QueryPattern::Union { left, right } => self.execute_union_pattern(left, right),
            QueryPattern::Filter {
                pattern,
                expression,
            } => self.execute_filter_pattern(pattern, expression),
            QueryPattern::Reduced(inner) => {
                let mut result = self.execute_sequential(inner)?;
                // Sort by string representation for consistent ordering
                result
                    .bindings
                    .sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
                result.bindings.dedup();
                Ok(result)
            }
            QueryPattern::Distinct(inner) => {
                let mut result = self.execute_sequential(inner)?;
                // Sort by variable count and string representation for consistent ordering
                result.bindings.sort_by(|a, b| {
                    a.variables.len().cmp(&b.variables.len()).then_with(|| {
                        format!("{:?}", a.variables).cmp(&format!("{:?}", b.variables))
                    })
                });
                result.bindings.dedup();
                Ok(result)
            }
        }
    }

    /// Execute basic graph pattern
    fn execute_basic_graph_pattern(&self, triples: &[TriplePattern]) -> OwlResult<QueryResult> {
        if triples.is_empty() {
            return Ok(QueryResult::new());
        }

        // Start with the first triple pattern
        let mut current_result = self.execute_single_triple(&triples[0])?;

        // Join with remaining patterns
        for triple in triples.iter().skip(1) {
            current_result = self.join_results(&current_result, triple)?;
        }

        Ok(current_result)
    }

    /// Execute a single triple pattern
    fn execute_single_triple(&self, triple: &TriplePattern) -> OwlResult<QueryResult> {
        // Determine query type
        let query_type = self.determine_query_type(triple);

        match query_type {
            QueryType::TypeQuery => self.execute_type_query(triple),
            QueryType::PropertyQuery => self.execute_property_query(triple),
            QueryType::VariablePredicate => self.execute_variable_predicate_query(triple),
        }
    }

    /// Determine the type of query based on the triple pattern
    fn determine_query_type(&self, triple: &TriplePattern) -> QueryType {
        match &triple.predicate {
            super::PatternTerm::IRI(pred_iri) => {
                if pred_iri.as_str() == RDF_TYPE {
                    QueryType::TypeQuery
                } else {
                    QueryType::PropertyQuery
                }
            }
            _ => QueryType::VariablePredicate,
        }
    }

    /// Execute type query (rdf:type pattern)
    fn execute_type_query(&self, triple: &TriplePattern) -> OwlResult<QueryResult> {
        if let super::PatternTerm::IRI(class_iri) = &triple.object {
            self.get_class_instances(class_iri)
        } else {
            Ok(QueryResult::new())
        }
    }

    /// Execute property query
    fn execute_property_query(&self, triple: &TriplePattern) -> OwlResult<QueryResult> {
        match (&triple.subject, &triple.predicate, &triple.object) {
            (
                super::PatternTerm::IRI(subject),
                super::PatternTerm::IRI(predicate),
                super::PatternTerm::Variable(_),
            ) => self.get_property_values(subject, predicate),
            _ => {
                // For more complex patterns, implement generic property query
                Ok(QueryResult::new())
            }
        }
    }

    /// Execute variable predicate query
    fn execute_variable_predicate_query(&self, _triple: &TriplePattern) -> OwlResult<QueryResult> {
        // TODO: Implement variable predicate queries
        Ok(QueryResult::new())
    }

    /// Execute optional pattern
    fn execute_optional_pattern(
        &self,
        left: &QueryPattern,
        right: &QueryPattern,
    ) -> OwlResult<QueryResult> {
        let left_result = self.execute_sequential(left)?;
        let right_result = self.execute_sequential(right)?;

        // Simple left outer join implementation
        let mut result = QueryResult::new();
        result.variables =
            Self::merge_variable_lists(&left_result.variables, &right_result.variables);

        for left_binding in &left_result.bindings {
            let mut found_match = false;

            for right_binding in &right_result.bindings {
                if let Some(merged) = left_binding.join(right_binding) {
                    result.add_binding(merged);
                    found_match = true;
                }
            }

            // If no match found, include left binding only
            if !found_match {
                result.add_binding(left_binding.clone());
            }
        }

        result.stats.results_count = result.len();
        result.stats.reasoning_used = self.config.enable_reasoning;

        Ok(result)
    }

    /// Execute union pattern
    fn execute_union_pattern(
        &self,
        left: &QueryPattern,
        right: &QueryPattern,
    ) -> OwlResult<QueryResult> {
        let left_result = self.execute_sequential(left)?;
        let right_result = self.execute_sequential(right)?;

        let mut result = QueryResult::new();
        result.variables =
            Self::merge_variable_lists(&left_result.variables, &right_result.variables);

        // Combine results
        result.bindings.extend(left_result.bindings);
        result.bindings.extend(right_result.bindings);

        result.stats.results_count = result.len();
        result.stats.reasoning_used = self.config.enable_reasoning;

        Ok(result)
    }

    /// Execute filter pattern
    fn execute_filter_pattern(
        &self,
        pattern: &QueryPattern,
        _expression: &super::FilterExpression,
    ) -> OwlResult<QueryResult> {
        // TODO: Implement filter evaluation
        self.execute_sequential(pattern)
    }

    /// Join two result sets
    fn join_results(
        &self,
        left: &QueryResult,
        right_triple: &TriplePattern,
    ) -> OwlResult<QueryResult> {
        let right_result = self.execute_single_triple(right_triple)?;

        let mut result = QueryResult::new();
        result.variables = Self::merge_variable_lists(&left.variables, &right_result.variables);

        for left_binding in &left.bindings {
            for right_binding in &right_result.bindings {
                if let Some(merged) = left_binding.join(right_binding) {
                    result.add_binding(merged);
                }
            }
        }

        result.stats.results_count = result.len();
        result.stats.reasoning_used = self.config.enable_reasoning;

        Ok(result)
    }

    /// Merge two variable lists
    fn merge_variable_lists(left: &[String], right: &[String]) -> Vec<String> {
        let mut merged: Vec<String> = left.to_vec();
        let left_set: HashSet<_> = left.iter().collect();

        for var in right {
            if !left_set.contains(var) {
                merged.push(var.clone());
            }
        }

        merged
    }
}

/// Extension trait for QueryPattern to support parallel execution
trait QueryPatternExt {
    fn supports_parallel(&self) -> bool;
}

impl QueryPatternExt for QueryPattern {
    fn supports_parallel(&self) -> bool {
        match self {
            QueryPattern::BasicGraphPattern(triples) => triples.len() > 1,
            QueryPattern::Union { .. } => true,
            QueryPattern::Optional { .. } => false,
            QueryPattern::Filter { .. } => false,
            QueryPattern::Reduced(_) => false,
            QueryPattern::Distinct(_) => false,
        }
    }
}
