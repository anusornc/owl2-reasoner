//! Query caching and memory management for OWL2 ontologies
//!
//! Provides efficient caching systems, memory pools, and compiled pattern storage.

use crate::reasoning::query::types::*;
use hashbrown::HashMap;
use lru::LruCache;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;


/// Query cache key for result caching
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct QueryCacheKey {
    pattern_hash: u64,
    config_hash: u64,
}

impl QueryCacheKey {
    /// Create a new cache key
    pub fn new(pattern_hash: u64, config_hash: u64) -> Self {
        Self {
            pattern_hash,
            config_hash,
        }
    }

    /// Get the pattern hash
    pub fn pattern_hash(&self) -> u64 {
        self.pattern_hash
    }

    /// Get the config hash
    pub fn config_hash(&self) -> u64 {
        self.config_hash
    }
}

/// Compiled query pattern for fast execution
#[derive(Debug, Clone)]
pub struct CompiledPattern {
    /// Original pattern
    #[allow(dead_code)]
    pattern: QueryPattern,
    /// Pre-computed hash for caching
    #[allow(dead_code)]
    hash: u64,
    /// Optimized execution plan
    execution_plan: ExecutionPlan,
    /// Variable positions for fast binding
    variable_positions: Vec<String>,
}

impl CompiledPattern {
    /// Create a new compiled pattern
    pub fn new(pattern: QueryPattern, execution_plan: ExecutionPlan) -> Self {
        let hash = Self::compute_pattern_hash(&pattern);
        let variable_positions = Self::extract_variables(&pattern);

        Self {
            pattern,
            hash,
            execution_plan,
            variable_positions,
        }
    }

    /// Get the execution plan
    pub fn execution_plan(&self) -> &ExecutionPlan {
        &self.execution_plan
    }

    /// Get the variable positions
    pub fn variable_positions(&self) -> &[String] {
        &self.variable_positions
    }

    /// Get the pattern hash
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Extract variables from pattern
    fn extract_variables(pattern: &QueryPattern) -> Vec<String> {
        let mut variables = HashSet::new();
        Self::collect_variables(pattern, &mut variables);
        variables.into_iter().collect()
    }

    /// Recursively collect variables
    fn collect_variables(pattern: &QueryPattern, variables: &mut HashSet<String>) {
        match pattern {
            QueryPattern::BasicGraphPattern(triples) => {
                for triple in triples {
                    Self::collect_pattern_variables(triple, variables);
                }
            }
            QueryPattern::Optional { left, right } | QueryPattern::Union { left, right } => {
                Self::collect_variables(left, variables);
                Self::collect_variables(right, variables);
            }
            QueryPattern::Filter { pattern, .. } => {
                Self::collect_variables(pattern, variables);
            }
            QueryPattern::Reduced(inner) | QueryPattern::Distinct(inner) => {
                Self::collect_variables(inner, variables);
            }
        }
    }

    /// Collect variables from triple pattern
    fn collect_pattern_variables(triple: &TriplePattern, variables: &mut HashSet<String>) {
        if let PatternTerm::Variable(var) = &triple.subject {
            variables.insert(var.clone());
        }
        if let PatternTerm::Variable(var) = &triple.predicate {
            variables.insert(var.clone());
        }
        if let PatternTerm::Variable(var) = &triple.object {
            variables.insert(var.clone());
        }
    }

    /// Compute pattern hash
    fn compute_pattern_hash(pattern: &QueryPattern) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();

        match pattern {
            QueryPattern::BasicGraphPattern(triples) => {
                0u8.hash(&mut hasher);
                for triple in triples {
                    triple.hash(&mut hasher);
                }
            }
            QueryPattern::Optional { left, right } => {
                1u8.hash(&mut hasher);
                Self::compute_pattern_hash(left).hash(&mut hasher);
                Self::compute_pattern_hash(right).hash(&mut hasher);
            }
            QueryPattern::Union { left, right } => {
                2u8.hash(&mut hasher);
                Self::compute_pattern_hash(left).hash(&mut hasher);
                Self::compute_pattern_hash(right).hash(&mut hasher);
            }
            QueryPattern::Filter {
                pattern,
                expression,
            } => {
                3u8.hash(&mut hasher);
                Self::compute_pattern_hash(pattern).hash(&mut hasher);
                expression.hash(&mut hasher);
            }
            QueryPattern::Reduced(inner) => {
                4u8.hash(&mut hasher);
                Self::compute_pattern_hash(inner).hash(&mut hasher);
            }
            QueryPattern::Distinct(inner) => {
                5u8.hash(&mut hasher);
                Self::compute_pattern_hash(inner).hash(&mut hasher);
            }
        }

        hasher.finish()
    }
}

// Safety: All fields in CompiledPattern are Send + Sync
unsafe impl Send for CompiledPattern {}
unsafe impl Sync for CompiledPattern {}

/// Query execution plan for optimized evaluation
#[derive(Debug, Clone)]
pub enum ExecutionPlan {
    /// Single triple pattern with optimized access path
    SingleTriple {
        query_type: QueryType,
        pattern: TriplePattern,
    },
    /// Multi-triple pattern with join ordering
    MultiTriple {
        patterns: Vec<TriplePattern>,
        join_order: Vec<usize>,
        access_paths: Vec<QueryType>,
    },
    /// Optional pattern with left outer join
    Optional {
        base: Box<ExecutionPlan>,
        optional: Box<ExecutionPlan>,
    },
    /// Union pattern with parallel execution
    Union { plans: Vec<ExecutionPlan> },
    /// Filter pattern with early filtering
    Filter {
        base: Box<ExecutionPlan>,
        filter_expr: FilterExpression,
    },
}

// Safety: All variants in ExecutionPlan contain Send + Sync types
unsafe impl Send for ExecutionPlan {}
unsafe impl Sync for ExecutionPlan {}

/// Types of triple pattern queries
#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    TypeQuery,
    PropertyQuery,
    VariablePredicate,
}

/// Memory pool for reusing query result allocations
#[derive(Debug)]
pub struct ResultPool {
    binding_pool: RwLock<Vec<QueryBinding>>,
    #[allow(dead_code)]
    result_pool: RwLock<Vec<QueryResult>>,
}

impl ResultPool {
    /// Create a new result pool
    pub fn new() -> Self {
        Self {
            binding_pool: RwLock::new(Vec::with_capacity(1000)),
            result_pool: RwLock::new(Vec::with_capacity(100)),
        }
    }

    /// Get a binding from the pool
    pub fn get_binding(&self) -> QueryBinding {
        let mut pool = self.binding_pool.write();
        pool.pop().unwrap_or_default()
    }

    /// Return a binding to the pool
    pub fn return_binding(&self, mut binding: QueryBinding) {
        binding.variables.clear();
        let mut pool = self.binding_pool.write();
        if pool.len() < 1000 {
            pool.push(binding);
        }
    }

    /// Get a result from the pool
    #[allow(dead_code)]
    pub fn get_result(&self) -> QueryResult {
        let mut pool = self.result_pool.write();
        pool.pop().unwrap_or_default()
    }

    /// Return a result to the pool
    #[allow(dead_code)]
    pub fn return_result(&self, mut result: QueryResult) {
        result.bindings.clear();
        result.variables.clear();
        let mut pool = self.result_pool.write();
        if pool.len() < 100 {
            pool.push(result);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize) {
        let binding_pool = self.binding_pool.read();
        let result_pool = self.result_pool.read();
        (binding_pool.len(), result_pool.len())
    }

    /// Clear all pools
    pub fn clear(&self) {
        self.binding_pool.write().clear();
        self.result_pool.write().clear();
    }
}

impl Default for ResultPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe query cache
#[derive(Debug)]
pub struct QueryCache {
    cache: Arc<RwLock<LruCache<QueryCacheKey, QueryResult>>>,
    pattern_cache: Arc<RwLock<HashMap<u64, CompiledPattern>>>,
}

impl QueryCache {
    /// Create a new query cache
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            pattern_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a cached result
    pub fn get(&self, key: &QueryCacheKey) -> Option<QueryResult> {
        let mut cache = self.cache.write();
        cache.get(key).cloned()
    }

    /// Put a result in the cache
    pub fn put(&self, key: QueryCacheKey, result: QueryResult) {
        let mut cache = self.cache.write();
        cache.put(key, result);
    }

    /// Get a cached compiled pattern
    pub fn get_pattern(&self, hash: u64) -> Option<CompiledPattern> {
        let cache = self.pattern_cache.read();
        cache.get(&hash).cloned()
    }

    /// Put a compiled pattern in the cache
    pub fn put_pattern(&self, hash: u64, pattern: CompiledPattern) {
        let mut cache = self.pattern_cache.write();
        cache.insert(hash, pattern);
    }

    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        let result_cache = self.cache.read();
        let pattern_cache = self.pattern_cache.read();
        (result_cache.len(), pattern_cache.len())
    }

    /// Clear all caches
    pub fn clear(&self) {
        self.cache.write().clear();
        self.pattern_cache.write().clear();
    }

    /// Check if result cache contains key
    pub fn contains(&self, key: &QueryCacheKey) -> bool {
        self.cache.read().contains(key)
    }

    /// Remove a specific key from cache
    pub fn remove(&self, key: &QueryCacheKey) -> Option<QueryResult> {
        self.cache.write().pop(key)
    }

    /// Resize the result cache
    pub fn resize(&self, new_capacity: NonZeroUsize) {
        let mut cache = self.cache.write();
        *cache = LruCache::new(new_capacity);
    }

    /// Get the current cache capacity
    pub fn capacity(&self) -> usize {
        self.cache.read().cap().into()
    }

    /// Get current usage percentage
    pub fn usage_percentage(&self) -> f64 {
        let cache = self.cache.read();
        if cache.cap() == NonZeroUsize::new(1).expect("1 > 0") {
            0.0
        } else {
            (cache.len() as f64 / cache.cap().get() as f64) * 100.0
        }
    }
}

impl Clone for QueryCache {
    fn clone(&self) -> Self {
        // Create a new cache with the same capacity but empty contents
        // Use a safe default if capacity conversion fails
        let capacity = NonZeroUsize::new(self.capacity())
            .unwrap_or_else(|| NonZeroUsize::new(1000).expect("1000 > 0"));
        Self::new(capacity)
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new(NonZeroUsize::new(1000).expect("1000 > 0"))
    }
}

/// Compute hash for query configuration
pub fn compute_config_hash(
    enable_reasoning: bool,
    enable_parallel: bool,
    max_results: Option<usize>,
) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();

    enable_reasoning.hash(&mut hasher);
    enable_parallel.hash(&mut hasher);
    max_results.hash(&mut hasher);

    hasher.finish()
}

/// Compute hash for query pattern
pub fn compute_pattern_hash(pattern: &QueryPattern) -> u64 {
    CompiledPattern::compute_pattern_hash(pattern)
}

/// Create a cache key from pattern and config
pub fn create_cache_key(pattern_hash: u64, config_hash: u64) -> QueryCacheKey {
    QueryCacheKey::new(pattern_hash, config_hash)
}
