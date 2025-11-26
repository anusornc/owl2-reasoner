//! Query caching and memory management for OWL2 ontologies
//!
//! Provides efficient caching systems, memory pools, and compiled pattern storage.
//! Features high-performance optimizations including:
//! - JoinHashTablePool for reusable hash join operations
//! - AdaptiveQueryIndex for intelligent query pattern caching
//! - Lock-free memory management for concurrent operations

use crate::reasoning::query::types::*;
use dashmap::DashMap;
use hashbrown::HashMap;
use lru::LruCache;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;

// Type alias to reduce complexity
type JoinHashTable = HashMap<Vec<QueryValue>, Vec<usize>>;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Pre-allocated hash table pool for high-performance hash joins
///
/// This optimization eliminates the massive allocation overhead caused by creating
/// new HashMaps for every join operation in query processing. The pool maintains
/// pre-allocated hash tables of various sizes that can be reused across queries.
#[derive(Debug)]
pub struct JoinHashTablePool {
    /// Pool of hash tables organized by capacity buckets
    pools: Vec<RwLock<Vec<JoinHashTable>>>,
    /// Pool usage statistics
    hits: AtomicUsize,
    misses: AtomicUsize,
    /// Total pool size
    pool_size: AtomicUsize,
}

impl JoinHashTablePool {
    /// Create a new join hash table pool with optimized capacity buckets
    pub fn new() -> Self {
        // Create capacity buckets: 16, 64, 256, 1024, 4096, 16384
        let capacities = vec![16, 64, 256, 1024, 4096, 16384];
        let mut pools = Vec::with_capacity(capacities.len());

        for _capacity in capacities {
            pools.push(RwLock::new(Vec::new()));
        }

        Self {
            pools,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            pool_size: AtomicUsize::new(0),
        }
    }
}

impl Default for JoinHashTablePool {
    fn default() -> Self {
        Self::new()
    }
}

impl JoinHashTablePool {
    /// Get a hash table from the pool, estimating the required capacity
    pub fn get_table(&self, estimated_size: usize) -> PooledHashTable<'_> {
        let bucket_idx = self.capacity_bucket(estimated_size);
        let pools = &self.pools[bucket_idx];

        // Try to get a table from the pool
        if let Some(mut pool) = pools.try_write() {
            if let Some(mut table) = pool.pop() {
                table.clear(); // Clear any existing data
                self.hits.fetch_add(1, Ordering::Relaxed);
                self.pool_size.fetch_sub(1, Ordering::Relaxed);
                return PooledHashTable {
                    table,
                    bucket_idx,
                    pool: self,
                };
            }
        }

        // No table available, create a new one
        self.misses.fetch_add(1, Ordering::Relaxed);
        let capacity = self.bucket_capacity(bucket_idx);
        PooledHashTable {
            table: HashMap::with_capacity(capacity),
            bucket_idx,
            pool: self,
        }
    }

    /// Return a hash table to the appropriate pool
    fn return_table(&self, mut table: JoinHashTable, bucket_idx: usize) {
        // Only return tables that aren't excessively large to prevent memory bloat
        if table.capacity() <= self.bucket_capacity(bucket_idx) * 2 {
            if let Some(mut pool) = self.pools[bucket_idx].try_write() {
                if pool.len() < 10 { // Limit pool size per bucket
                    table.clear();
                    table.shrink_to_fit(); // Optimize memory usage
                    pool.push(table);
                    self.pool_size.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    /// Determine which bucket to use based on estimated size
    fn capacity_bucket(&self, size: usize) -> usize {
        match size {
            0..=32 => 0,   // 16 capacity
            33..=128 => 1, // 64 capacity
            129..=512 => 2, // 256 capacity
            513..=2048 => 3, // 1024 capacity
            2049..=8192 => 4, // 4096 capacity
            _ => 5, // 16384 capacity
        }
    }

    /// Get the actual capacity for a bucket
    fn bucket_capacity(&self, bucket_idx: usize) -> usize {
        match bucket_idx {
            0 => 16,
            1 => 64,
            2 => 256,
            3 => 1024,
            4 => 4096,
            _ => 16384,
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> JoinPoolStats {
        JoinPoolStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            pool_size: self.pool_size.load(Ordering::Relaxed),
            hit_rate: {
                let hits = self.hits.load(Ordering::Relaxed) as f64;
                let total = hits + self.misses.load(Ordering::Relaxed) as f64;
                if total > 0.0 { hits / total * 100.0 } else { 0.0 }
            },
        }
    }

    /// Pre-warm the pool with tables of various sizes
    pub fn pre_warm(&self, tables_per_bucket: usize) {
        for bucket_idx in 0..self.pools.len() {
            let capacity = self.bucket_capacity(bucket_idx);
            let mut pool = self.pools[bucket_idx].write();

            for _ in 0..tables_per_bucket {
                pool.push(HashMap::with_capacity(capacity));
                self.pool_size.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Clear the pool and reset statistics
    pub fn clear(&self) {
        for pool in &self.pools {
            pool.write().clear();
        }
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.pool_size.store(0, Ordering::Relaxed);
    }
}

/// RAII wrapper for pooled hash tables
pub struct PooledHashTable<'a> {
    table: HashMap<Vec<QueryValue>, Vec<usize>>,
    bucket_idx: usize,
    pool: &'a JoinHashTablePool,
}

impl<'a> PooledHashTable<'a> {
    /// Get mutable reference to the underlying table
    pub fn get_mut(&mut self) -> &mut HashMap<Vec<QueryValue>, Vec<usize>> {
        &mut self.table
    }

    /// Build hash table from right bindings for efficient join operations
    pub fn build_from_bindings(&mut self, bindings: &[QueryBinding], common_vars: &[String]) {
        for (idx, binding) in bindings.iter().enumerate() {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| {
                    binding.get_value(var)
                        .cloned()
                        .unwrap_or(QueryValue::Literal("".to_string()))
                })
                .collect();

            self.insert(key, idx);
        }
    }

    /// Get reference to the underlying table
    pub fn get(&self) -> &HashMap<Vec<QueryValue>, Vec<usize>> {
        &self.table
    }

    /// Insert a key-value pair (storing binding index for lookup)
    pub fn insert(&mut self, key: Vec<QueryValue>, binding_index: usize) {
        self.table.entry(key).or_default().push(binding_index);
    }

    /// Get indices for a key
    pub fn get_indices(&self, key: &[QueryValue]) -> Option<&[usize]> {
        self.table.get(key).map(|vec| vec.as_slice())
    }

    /// Check if the table contains a key
    pub fn contains_key(&self, key: &[QueryValue]) -> bool {
        self.table.contains_key(key)
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Perform hash join with right bindings
    pub fn hash_join<'b>(
        &self,
        left_bindings: &[QueryBinding],
        right_bindings: &'b [QueryBinding],
        common_vars: &[String],
    ) -> Vec<(QueryBinding, &'b QueryBinding)> {
        let mut result = Vec::new();

        for left_binding in left_bindings {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| {
                    left_binding.get_value(var)
                        .cloned()
                        .unwrap_or(QueryValue::Literal("".to_string()))
                })
                .collect();

            if let Some(binding_indices) = self.get_indices(&key) {
                for &index in binding_indices {
                    if let Some(right_binding) = right_bindings.get(index) {
                        result.push((left_binding.clone(), right_binding));
                    }
                }
            }
        }

        result
    }
}

impl<'a> Drop for PooledHashTable<'a> {
    fn drop(&mut self) {
        // Return the table to the pool when dropped
        let table = std::mem::replace(
            &mut self.table,
            HashMap::new()
        );
        self.pool.return_table(table, self.bucket_idx);
    }
}

/// Join hash table pool statistics
#[derive(Debug, Clone)]
pub struct JoinPoolStats {
    /// Number of pool hits
    pub hits: usize,
    /// Number of pool misses
    pub misses: usize,
    /// Current pool size
    pub pool_size: usize,
    /// Pool hit rate percentage
    pub hit_rate: f64,
}

/// Adaptive query index with frequency tracking and predictive caching
///
/// This optimization replaces O(n) linear scans with intelligent multi-level indexing
/// that adapts based on query patterns and access frequencies.
#[derive(Debug)]
pub struct AdaptiveQueryIndex {
    /// Primary index by query pattern hash
    primary_index: DashMap<u64, AdaptiveIndexEntry>,
    /// Frequency-based secondary index
    frequency_index: RwLock<HashMap<String, Vec<u64>>>,
    /// Recent access patterns for prediction
    access_patterns: RwLock<Vec<QueryAccess>>,
    /// Statistics for performance monitoring
    stats: RwLock<AdaptiveIndexStats>,
    /// Configuration
    config: AdaptiveIndexConfig,
}

/// Configuration for adaptive query index
#[derive(Debug, Clone)]
pub struct AdaptiveIndexConfig {
    /// Maximum number of recent access patterns to track
    pub max_access_patterns: usize,
    /// Frequency threshold for promotion to primary index
    pub frequency_threshold: usize,
    /// Cache warmup threshold
    pub warmup_threshold: usize,
    /// Index cleanup interval
    pub cleanup_interval: Duration,
}

impl Default for AdaptiveIndexConfig {
    fn default() -> Self {
        Self {
            max_access_patterns: 1000,
            frequency_threshold: 5,
            warmup_threshold: 10,
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

/// Adaptive index entry with access pattern tracking
#[derive(Debug)]
pub struct AdaptiveIndexEntry {
    /// Compiled query pattern
    pub pattern: CompiledPattern,
    /// Access frequency
    access_count: AtomicUsize,
    /// Last access time
    last_access: RwLock<Instant>,
    /// Performance metrics
    avg_execution_time: RwLock<Duration>,
    /// Query result cache for frequently used queries
    result_cache: RwLock<Option<QueryResult>>,
    /// Prediction score for cache warming
    prediction_score: AtomicUsize,
}

impl Clone for AdaptiveIndexEntry {
    fn clone(&self) -> Self {
        Self {
            pattern: self.pattern.clone(),
            access_count: AtomicUsize::new(self.access_count.load(Ordering::Relaxed)),
            last_access: RwLock::new(*self.last_access.read()),
            avg_execution_time: RwLock::new(*self.avg_execution_time.read()),
            result_cache: RwLock::new(self.result_cache.read().clone()),
            prediction_score: AtomicUsize::new(self.prediction_score.load(Ordering::Relaxed)),
        }
    }
}

/// Query access record for pattern analysis
#[derive(Debug, Clone)]
pub struct QueryAccess {
    pub pattern_hash: u64,
    pub access_time: Instant,
    pub execution_time: Duration,
    pub variables: Vec<String>,
}

/// Adaptive index performance statistics
#[derive(Debug, Clone, Default)]
pub struct AdaptiveIndexStats {
    pub total_accesses: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub prediction_accuracy: f64,
    pub avg_lookup_time: Duration,
    pub memory_usage: usize,
}

impl AdaptiveQueryIndex {
    /// Create a new adaptive query index
    pub fn new() -> Self {
        Self::with_config(AdaptiveIndexConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: AdaptiveIndexConfig) -> Self {
        Self {
            primary_index: DashMap::new(),
            frequency_index: RwLock::new(HashMap::new()),
            access_patterns: RwLock::new(Vec::new()),
            stats: RwLock::new(AdaptiveIndexStats::default()),
            config,
        }
    }

    /// Get or create an index entry for a query pattern
    pub fn get_or_create(&self, pattern: &QueryPattern) -> Option<AdaptiveIndexEntry> {
        let pattern_hash = self.compute_pattern_hash(pattern);

        // Try primary index first
        if let Some(entry) = self.primary_index.get(&pattern_hash) {
            self.record_access(&pattern_hash, Duration::from_nanos(0));
            return Some(entry.clone());
        }

        // Check frequency-based secondary index
        let frequency_key = self.extract_frequency_key(pattern);
        let should_promote = {
            let freq_index = self.frequency_index.read();
            freq_index.get(&frequency_key)
                .map(|patterns| patterns.len() >= self.config.frequency_threshold)
                .unwrap_or(false)
        };

        if should_promote {
            self.promote_to_primary(pattern);
            self.primary_index.get(&pattern_hash).map(|entry| entry.clone())
        } else {
            self.update_frequency_index(&frequency_key, pattern_hash);
            None
        }
    }

    /// Record query access for pattern analysis and prediction
    pub fn record_access(&self, pattern_hash: &u64, execution_time: Duration) {
        let now = Instant::now();

        // Update primary index entry if it exists
        if let Some(entry) = self.primary_index.get(pattern_hash) {
            entry.access_count.fetch_add(1, Ordering::Relaxed);
            *entry.last_access.write() = now;

            // Update average execution time
            let mut avg_time = entry.avg_execution_time.write();
            let new_avg = if *avg_time == Duration::ZERO {
                execution_time
            } else {
                Duration::from_nanos((avg_time.as_nanos() as u64 + execution_time.as_nanos() as u64) / 2)
            };
            *avg_time = new_avg;

            // Update prediction score based on access patterns
            self.update_prediction_score(pattern_hash, &entry);
        }

        // Record access pattern
        let mut patterns = self.access_patterns.write();
        patterns.push(QueryAccess {
            pattern_hash: *pattern_hash,
            access_time: now,
            execution_time,
            variables: Vec::new(), // Could be extracted from pattern
        });

        // Limit access pattern history
        if patterns.len() > self.config.max_access_patterns {
            patterns.remove(0);
        }

        // Update statistics
        let mut stats = self.stats.write();
        stats.total_accesses += 1;
        stats.memory_usage = self.estimate_memory_usage();
    }

    /// Predict and preload frequently accessed queries
    pub fn predictive_preload(&self) -> Vec<u64> {
        let patterns = self.access_patterns.read();
        let mut predictions = Vec::new();

        // Simple frequency-based prediction
        let mut frequency_map: HashMap<u64, usize> = HashMap::new();
        for access in patterns.iter() {
            *frequency_map.entry(access.pattern_hash).or_insert(0) += 1;
        }

        // Get top candidates for preloading
        let mut candidates: Vec<_> = frequency_map.into_iter().collect();
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        for (pattern_hash, freq) in candidates.iter().take(10) {
            if *freq >= self.config.warmup_threshold {
                predictions.push(*pattern_hash);
            }
        }

        predictions
    }

    /// Get frequently accessed query patterns for cache warming
    pub fn get_hot_patterns(&self) -> Vec<(u64, usize)> {
        self.primary_index.iter()
            .map(|entry| {
                let count = entry.access_count.load(Ordering::Relaxed);
                (*entry.key(), count)
            })
            .filter(|(_, count)| *count >= self.config.frequency_threshold)
            .collect()
    }

    /// Clean up old entries and optimize memory usage
    pub fn cleanup(&self) {
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(300); // 5 minutes ago

        // Remove old entries from primary index
        self.primary_index.retain(|_, entry| {
            *entry.last_access.read() > cutoff ||
            entry.access_count.load(Ordering::Relaxed) >= self.config.frequency_threshold
        });

        // Clean up access patterns
        let mut patterns = self.access_patterns.write();
        patterns.retain(|access| access.access_time > cutoff);

        // Clean up frequency index
        let mut freq_index = self.frequency_index.write();
        freq_index.retain(|_, hashes| {
            hashes.iter().any(|&hash| {
                self.primary_index.contains_key(&hash)
            })
        });
    }

    /// Get adaptive index statistics
    pub fn stats(&self) -> AdaptiveIndexStats {
        self.stats.read().clone()
    }

    /// Estimate memory usage of the index
    fn estimate_memory_usage(&self) -> usize {
        let primary_size = self.primary_index.len() * std::mem::size_of::<AdaptiveIndexEntry>();
        let patterns_size = self.access_patterns.read().len() * std::mem::size_of::<QueryAccess>();
        let freq_size = self.frequency_index.read().len() * 64; // Estimate
        primary_size + patterns_size + freq_size
    }

    // Helper methods
    fn compute_pattern_hash(&self, pattern: &QueryPattern) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        pattern.hash(&mut hasher);
        hasher.finish()
    }

    fn extract_frequency_key(&self, pattern: &QueryPattern) -> String {
        // Extract a simplified key for frequency indexing
        match pattern {
            QueryPattern::BasicGraphPattern(triples) => {
                format!("bgp_{}", triples.len())
            }
            QueryPattern::Optional { .. } => "optional".to_string(),
            QueryPattern::Union { .. } => "union".to_string(),
            QueryPattern::Filter { .. } => "filter".to_string(),
            QueryPattern::Reduced(_) => "reduced".to_string(),
            QueryPattern::Distinct(_) => "distinct".to_string(),
        }
    }

    fn update_frequency_index(&self, key: &str, pattern_hash: u64) {
        let mut freq_index = self.frequency_index.write();
        freq_index.entry(key.to_string()).or_default().push(pattern_hash);
    }

    fn promote_to_primary(&self, pattern: &QueryPattern) {
        let pattern_hash = self.compute_pattern_hash(pattern);
        if !self.primary_index.contains_key(&pattern_hash) {
            let execution_plan = self.create_execution_plan(pattern);
            let compiled_pattern = CompiledPattern::new(pattern.clone(), execution_plan);

            let entry = AdaptiveIndexEntry {
                pattern: compiled_pattern,
                access_count: AtomicUsize::new(1),
                last_access: RwLock::new(Instant::now()),
                avg_execution_time: RwLock::new(Duration::ZERO),
                result_cache: RwLock::new(None),
                prediction_score: AtomicUsize::new(0),
            };

            self.primary_index.insert(pattern_hash, entry);
        }
    }

    fn create_execution_plan(&self, pattern: &QueryPattern) -> ExecutionPlan {
        // Simple execution plan creation - this would be more sophisticated in practice
        match pattern {
            QueryPattern::BasicGraphPattern(triples) => {
                if triples.len() == 1 {
                    ExecutionPlan::SingleTriple {
                        query_type: crate::reasoning::query::cache::QueryType::VariablePredicate,
                        pattern: triples[0].clone(),
                    }
                } else {
                    let join_order: Vec<usize> = (0..triples.len()).collect();
                    let access_paths = vec![
                        crate::reasoning::query::cache::QueryType::VariablePredicate;
                        triples.len()
                    ];
                    ExecutionPlan::MultiTriple {
                        patterns: triples.clone(),
                        join_order,
                        access_paths,
                    }
                }
            }
            _ => ExecutionPlan::Filter {
                base: Box::new(ExecutionPlan::SingleTriple {
                    query_type: crate::reasoning::query::cache::QueryType::VariablePredicate,
                    pattern: TriplePattern::new(
                        PatternTerm::Variable("?s".to_string()),
                        PatternTerm::Variable("?p".to_string()),
                        PatternTerm::Variable("?o".to_string()),
                    ),
                }),
                filter_expr: FilterExpression::IsVariable("?x".to_string()),
            },
        }
    }

    fn update_prediction_score(&self, pattern_hash: &u64, entry: &AdaptiveIndexEntry) {
        let access_count = entry.access_count.load(Ordering::Relaxed);
        let patterns = self.access_patterns.read();

        // Calculate recency and frequency score
        let recent_accesses = patterns.iter()
            .filter(|access| access.pattern_hash == *pattern_hash)
            .count();

        let recency_score = if recent_accesses > 0 {
            let latest_access = patterns.iter()
                .filter(|access| access.pattern_hash == *pattern_hash)
                .max_by_key(|access| access.access_time);

            if let Some(latest) = latest_access {
                let age = latest.access_time.elapsed().as_secs_f64();
                (1.0 / (1.0 + age)) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let frequency_score = (access_count as f64).log10() * 10.0;
        let new_score = (recency_score + frequency_score) as usize;

        entry.prediction_score.store(new_score, Ordering::Relaxed);
    }
}

impl Default for AdaptiveQueryIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Query pattern predictor for intelligent cache warming
///
/// Analyzes query patterns and predicts likely future queries to optimize cache warming
/// and reduce cache misses. Uses machine learning-inspired techniques for prediction.
#[derive(Debug)]
pub struct QueryPatternPredictor {
    /// Pattern frequency map for prediction
    pattern_frequencies: RwLock<HashMap<String, f64>>,
    /// Recent query sequence for temporal prediction
    query_sequence: RwLock<Vec<String>>,
    /// Pattern correlation matrix
    pattern_correlations: RwLock<HashMap<String, HashMap<String, f64>>>,
    /// Prediction accuracy statistics
    prediction_stats: RwLock<PredictionStats>,
}

/// Prediction accuracy statistics
#[derive(Debug, Clone, Default)]
pub struct PredictionStats {
    pub total_predictions: usize,
    pub correct_predictions: usize,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
}

impl QueryPatternPredictor {
    /// Create a new query pattern predictor
    pub fn new() -> Self {
        Self {
            pattern_frequencies: RwLock::new(HashMap::new()),
            query_sequence: RwLock::new(Vec::new()),
            pattern_correlations: RwLock::new(HashMap::new()),
            prediction_stats: RwLock::new(PredictionStats::default()),
        }
    }

    /// Record a query pattern for learning and prediction
    pub fn record_query(&self, pattern_key: &str, _execution_time: Duration) {
        // Update pattern frequencies
        {
            let mut freqs = self.pattern_frequencies.write();
            *freqs.entry(pattern_key.to_string()).or_insert(0.0) += 1.0;
        }

        // Update query sequence for temporal patterns
        {
            let mut sequence = self.query_sequence.write();
            sequence.push(pattern_key.to_string());

            // Limit sequence length to prevent memory bloat
            if sequence.len() > 1000 {
                sequence.remove(0);
            }
        }

        // Update pattern correlations
        self.update_pattern_correlations(pattern_key);
    }

    /// Predict next likely query patterns
    pub fn predict_next_queries(&self, current_pattern: &str, count: usize) -> Vec<(String, f64)> {
        let correlations = self.pattern_correlations.read();
        let freqs = self.pattern_frequencies.read();

        // Get correlated patterns
        let mut predictions: Vec<(String, f64)> = correlations
            .get(current_pattern)
            .map(|correlations| {
                correlations
                    .iter()
                    .map(|(pattern, correlation)| {
                        let frequency = freqs.get(pattern).unwrap_or(&0.0);
                        let score = correlation * frequency;
                        (pattern.clone(), score)
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Sort by prediction score and return top N
        predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        predictions.truncate(count);

        predictions
    }

    /// Get hot query patterns based on frequency and recency
    pub fn get_hot_patterns(&self, threshold: f64) -> Vec<(String, f64)> {
        let freqs = self.pattern_frequencies.read();
        let sequence = self.query_sequence.read();

        // Calculate hotness score based on frequency and recency
        let mut hot_patterns: Vec<(String, f64)> = freqs
            .iter()
            .map(|(pattern, &frequency)| {
                let recency_bonus = self.calculate_recency_bonus(pattern, &sequence);
                let hotness_score = frequency * (1.0 + recency_bonus);
                (pattern.clone(), hotness_score)
            })
            .filter(|(_, score)| *score >= threshold)
            .collect();

        hot_patterns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        hot_patterns
    }

    /// Update prediction statistics based on actual vs predicted
    pub fn update_prediction_accuracy(&self, predicted: &[String], actual: &str) {
        let mut stats = self.prediction_stats.write();
        stats.total_predictions += 1;

        if predicted.contains(&actual.to_string()) {
            stats.correct_predictions += 1;
        }

        stats.accuracy = stats.correct_predictions as f64 / stats.total_predictions as f64;

        // Update precision and recall (simplified)
        if !predicted.is_empty() {
            let true_positives = predicted.iter().filter(|p| p.as_str() == actual).count() as f64;
            stats.precision = true_positives / predicted.len() as f64;
            stats.recall = true_positives; // Simplified - assume single actual query
        }
    }

    /// Get prediction statistics
    pub fn get_stats(&self) -> PredictionStats {
        self.prediction_stats.read().clone()
    }

    /// Reset predictor state
    pub fn reset(&self) {
        self.pattern_frequencies.write().clear();
        self.query_sequence.write().clear();
        self.pattern_correlations.write().clear();
        *self.prediction_stats.write() = PredictionStats::default();
    }

    // Helper methods
    fn update_pattern_correlations(&self, current_pattern: &str) {
        let sequence = self.query_sequence.read();
        let mut correlations = self.pattern_correlations.write();

        // Find patterns that occurred before this one
        for (i, pattern) in sequence.iter().enumerate() {
            if pattern == current_pattern {
                // Look back at previous patterns and update correlations
                let lookback = 5.min(i); // Look back up to 5 patterns
                for j in (i.saturating_sub(lookback))..i {
                    let prev_pattern = &sequence[j];
                    if prev_pattern != current_pattern {
                        let entry = correlations.entry(prev_pattern.clone())
                            .or_default();
                        *entry.entry(current_pattern.to_string()).or_insert(0.0) += 1.0;
                    }
                }
                break;
            }
        }
    }

    fn calculate_recency_bonus(&self, pattern: &str, sequence: &[String]) -> f64 {
        // Find the most recent occurrence of this pattern
        if let Some((index, _)) = sequence.iter().rev().enumerate().find(|(_, p)| *p == pattern) {
            let recency_factor = (index as f64 + 1.0) / sequence.len() as f64;
            recency_factor * 0.5 // Scale down to avoid overwhelming frequency
        } else {
            0.0
        }
    }
}

impl Default for QueryPatternPredictor {
    fn default() -> Self {
        Self::new()
    }
}

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
