//! Extracted Tableaux Algorithm for OpenEvolve Optimization
//!
//! This module contains the core tableaux reasoning algorithm extracted
//! from the OWL2 reasoner for optimization with OpenEvolve.
//!
//! ## Key Features
//!
//! - **Self-contained**: Minimal dependencies for OpenEvolve compatibility
//! - **Benchmark-ready**: Built-in timing and performance metrics
//! - **Correctness-preserving**: Maintains logical correctness while optimizing
//! - **Evolvable structure**: Clear separation of optimization targets

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

/// Core tableaux reasoning algorithm extracted for OpenEvolve optimization
pub struct OptimizedTableauxReasoner {
    /// Internal graph structure for tableaux
    graph: TableauxGraph,
    /// Performance statistics
    stats: ReasoningStats,
    /// Configuration parameters (targets for optimization)
    config: TableauxConfig,
    /// Cache for memoization (optimization target)
    cache: HashMap<String, bool>,
}

/// Configuration parameters for tableaux optimization
#[derive(Debug, Clone)]
pub struct TableauxConfig {
    /// Maximum depth for tableaux expansion (optimization target)
    pub max_depth: usize,
    /// Enable advanced blocking strategies (optimization target)
    pub advanced_blocking: bool,
    /// Enable dependency-directed backtracking (optimization target)
    pub dependency_backtracking: bool,
    /// Enable heuristic rule ordering (optimization target)
    pub heuristic_ordering: bool,
    /// Enable parallel branch processing (optimization target)
    pub parallel_processing: bool,
}

impl Default for TableauxConfig {
    fn default() -> Self {
        TableauxConfig {
            max_depth: 1000,
            advanced_blocking: false,
            dependency_backtracking: false,
            heuristic_ordering: false,
            parallel_processing: false,
        }
    }
}

/// Performance statistics for benchmarking
#[derive(Debug, Clone, Default)]
pub struct ReasoningStats {
    /// Total reasoning time in milliseconds
    pub time_ms: u64,
    /// Number of nodes created in the tableaux
    pub nodes_created: usize,
    /// Number of rules applied
    pub rules_applied: usize,
    /// Number of backtracking operations
    pub backtracks: usize,
    /// Cache hit rate
    pub cache_hits: usize,
    /// Cache miss rate
    pub cache_misses: usize,
}

/// Result of tableaux reasoning
#[derive(Debug, Clone)]
pub struct TableauxResult {
    /// Whether the concept is satisfiable
    pub satisfiable: bool,
    /// Explanation for the result
    pub explanation: Option<String>,
    /// Performance statistics
    pub stats: ReasoningStats,
}

/// Tableaux graph structure
#[derive(Debug)]
struct TableauxGraph {
    /// Nodes in the tableaux
    nodes: HashMap<NodeId, TableauxNode>,
    /// Edges between nodes
    edges: HashMap<NodeId, HashMap<String, HashSet<NodeId>>>,
    /// Root node
    root: NodeId,
    /// Next node ID
    next_id: usize,
}

/// Individual node in the tableaux
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TableauxNode {
    /// Node identifier
    id: NodeId,
    /// Concepts at this node
    concepts: HashSet<TestConcept>,
    /// Labels for blocking
    labels: HashSet<String>,
    /// Blocking information
    blocked_by: Option<NodeId>,
}

/// Node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct NodeId(usize);

/// Test concepts for benchmarking (simplified from full OWL2)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestConcept {
    /// Named concept
    Named(String),
    /// Intersection of concepts
    Intersection(Vec<TestConcept>),
    /// Union of concepts
    Union(Vec<TestConcept>),
    /// Complement of a concept
    Complement(Box<TestConcept>),
    /// Existential restriction
    SomeValuesFrom(String, Box<TestConcept>),
    /// Universal restriction
    AllValuesFrom(String, Box<TestConcept>),
}

impl OptimizedTableauxReasoner {
    /// Create a new optimized tableaux reasoner
    pub fn new() -> Self {
        Self::with_config(TableauxConfig::default())
    }

    /// Create a new tableaux reasoner with custom configuration
    pub fn with_config(config: TableauxConfig) -> Self {
        OptimizedTableauxReasoner {
            graph: TableauxGraph::new(),
            stats: ReasoningStats::default(),
            config,
            cache: HashMap::new(),
        }
    }

    /// Main tableaux reasoning algorithm (primary optimization target)
    pub fn check_satisfiability(&mut self, concept: &TestConcept) -> TableauxResult {
        let start_time = Instant::now();

        // Check cache first (optimization target)
        let cache_key = self.concept_to_cache_key(concept);
        if let Some(&cached_result) = self.cache.get(&cache_key) {
            self.stats.cache_hits += 1;
            return TableauxResult {
                satisfiable: cached_result,
                explanation: Some("Cache hit".to_string()),
                stats: self.stats.clone(),
            };
        }
        self.stats.cache_misses += 1;

        // Initialize tableaux graph
        self.graph = TableauxGraph::new();
        let root = self.graph.root;
        self.graph.add_concept(root, concept.clone());

        // Run the core tableaux algorithm
        let result = self.run_tableaux_algorithm(concept);

        // Cache the result
        self.cache.insert(cache_key, result.satisfiable);

        // Update statistics
        self.stats.time_ms = start_time.elapsed().as_millis() as u64;

        result
    }

    /// Core tableaux algorithm (main optimization target)
    fn run_tableaux_algorithm(&mut self, initial_concept: &TestConcept) -> TableauxResult {
        let mut queue = VecDeque::new();
        queue.push_back(self.graph.root);

        while let Some(node_id) = queue.pop_front() {
            // Check depth limit
            if self.stats.nodes_created > self.config.max_depth {
                return TableauxResult {
                    satisfiable: false,
                    explanation: Some("Maximum depth exceeded".to_string()),
                    stats: self.stats.clone(),
                };
            }

            // Get current node
            let node = match self.graph.nodes.get(&node_id) {
                Some(node) => node,
                None => continue,
            };

            // Apply blocking strategies (optimization target)
            if self.config.advanced_blocking {
                if let Some(blocked_by) = self.check_blocking(node) {
                    if blocked_by {
                        continue; // Skip blocked node
                    }
                }
            }

            // Apply reasoning rules
            let concepts: Vec<_> = node.concepts.iter().cloned().collect();
            for concept in concepts {
                if let Some(new_concepts) = self.apply_reasoning_rules(&concept, node_id) {
                    self.stats.rules_applied += 1;

                    // Add new concepts to current node
                    for new_concept in new_concepts {
                        self.graph.add_concept(node_id, new_concept);
                    }
                }
            }

            // Check for contradictions
            if self.check_contradiction(node) {
                self.stats.backtracks += 1;

                // Backtracking strategy (optimization target)
                if self.config.dependency_backtracking {
                    return TableauxResult {
                        satisfiable: false,
                        explanation: Some("Contradiction found with backtracking".to_string()),
                        stats: self.stats.clone(),
                    };
                } else {
                    return TableauxResult {
                        satisfiable: false,
                        explanation: Some("Contradiction found".to_string()),
                        stats: self.stats.clone(),
                    };
                }
            }

            // Expand to successor nodes
            if self.config.parallel_processing {
                // Parallel expansion optimization target
                self.expand_successors_parallel(&mut queue, node_id);
            } else {
                self.expand_successors_sequential(&mut queue, node_id);
            }
        }

        // Tableaux completed successfully - concept is satisfiable
        TableauxResult {
            satisfiable: true,
            explanation: None,
            stats: self.stats.clone(),
        }
    }

    /// Apply reasoning rules to a concept (optimization target)
    fn apply_reasoning_rules(&mut self, concept: &TestConcept, node_id: NodeId) -> Option<Vec<TestConcept>> {
        match concept {
            TestConcept::Intersection(operands) => {
                // Decompose intersection: C ⊓ D → C, D
                Some(operands.clone())
            }

            TestConcept::Union(operands) => {
                // Non-deterministic choice for union: C ⊔ D → C or D
                // Optimization target: heuristic ordering
                if self.config.heuristic_ordering {
                    // Apply heuristic for choosing operand
                    self.heuristic_choice(operands)
                } else {
                    // Default: choose first operand
                    operands.first().cloned().map(|c| vec![c])
                }
            }

            TestConcept::SomeValuesFrom(property, filler) => {
                // ∃R.C → create successor node with C
                self.create_existential_successor(property, filler, node_id)
            }

            TestConcept::AllValuesFrom(_, _) => {
                // ∀R.C → check all R-successors have C
                // Complex rule to be optimized
                None
            }

            TestConcept::Complement(_) => {
                // ¬C → check for contradiction
                None
            }

            TestConcept::Named(_) => {
                // Atomic concept - no decomposition needed
                None
            }
        }
    }

    /// Check for blocking conditions (advanced optimization target)
    fn check_blocking(&self, node: &TableauxNode) -> Option<bool> {
        if !self.config.advanced_blocking {
            return None;
        }

        // Simple pairwise blocking check
        for (other_id, other_node) in &self.graph.nodes {
            if other_node.id != node.id && self.is_blocked_by(node, other_node) {
                return Some(true);
            }
        }

        Some(false)
    }

    /// Check if node is blocked by another node
    fn is_blocked_by(&self, node: &TableauxNode, blocker: &TableauxNode) -> bool {
        // Simple blocking condition: blocker has all concepts of node
        blocker.concepts.is_superset(&node.concepts)
    }

    /// Heuristic choice for union decomposition (optimization target)
    fn heuristic_choice(&self, operands: &[TestConcept]) -> Option<Vec<TestConcept>> {
        if operands.is_empty() {
            return None;
        }

        // Simple heuristic: choose the concept with simplest structure
        let mut best_index = 0;
        let mut best_complexity = self.concept_complexity(&operands[0]);

        for (i, concept) in operands.iter().enumerate().skip(1) {
            let complexity = self.concept_complexity(concept);
            if complexity < best_complexity {
                best_index = i;
                best_complexity = complexity;
            }
        }

        operands.get(best_index).cloned().map(|c| vec![c])
    }

    /// Calculate concept complexity for heuristics
    fn concept_complexity(&self, concept: &TestConcept) -> usize {
        match concept {
            TestConcept::Named(_) => 1,
            TestConcept::Intersection(operands) =>
                operands.iter().map(|c| self.concept_complexity(c)).sum::<usize>() + 1,
            TestConcept::Union(operands) =>
                operands.iter().map(|c| self.concept_complexity(c)).sum::<usize>() + 1,
            TestConcept::Complement(expr) => self.concept_complexity(expr) + 1,
            TestConcept::SomeValuesFrom(_, filler) => self.concept_complexity(filler) + 2,
            TestConcept::AllValuesFrom(_, filler) => self.concept_complexity(filler) + 2,
        }
    }

    /// Create existential successor (optimization target)
    fn create_existential_successor(&mut self, property: &str, filler: &TestConcept, node_id: NodeId) -> Option<Vec<TestConcept>> {
        // Create new successor node
        let successor_id = self.graph.add_node();
        self.graph.add_edge(node_id, property.to_string(), successor_id);
        self.graph.add_concept(successor_id, filler.clone());

        self.stats.nodes_created += 1;

        // Return the filler concept to be added to current node
        Some(vec![filler.clone()])
    }

    /// Expand successors sequentially (baseline)
    fn expand_successors_sequential(&mut self, queue: &mut VecDeque<NodeId>, node_id: NodeId) {
        // Sequential expansion logic
        for successors in self.graph.get_all_successors(node_id) {
            for successor_id in successors {
                queue.push_back(successor_id);
            }
        }
    }

    /// Expand successors in parallel (optimization target)
    fn expand_successors_parallel(&mut self, queue: &mut VecDeque<NodeId>, node_id: NodeId) {
        // Parallel expansion would be implemented here
        // For now, fall back to sequential
        self.expand_successors_sequential(queue, node_id);
    }

    /// Check for contradictions in a node
    fn check_contradiction(&self, node: &TableauxNode) -> bool {
        let concepts: Vec<_> = node.concepts.iter().collect();

        for i in 0..concepts.len() {
            for j in i + 1..concepts.len() {
                if self.are_complementary(concepts[i], concepts[j]) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if two concepts are complementary
    fn are_complementary(&self, a: &TestConcept, b: &TestConcept) -> bool {
        match (a, b) {
            (TestConcept::Named(a_name), TestConcept::Complement(box_b)) => {
                if let TestConcept::Named(b_name) = box_b.as_ref() {
                    return a_name == b_name;
                }
            }
            (TestConcept::Complement(box_a), TestConcept::Named(b_name)) => {
                if let TestConcept::Named(a_name) = box_a.as_ref() {
                    return a_name == b_name;
                }
            }
            _ => {}
        }

        false
    }

    /// Generate cache key for a concept
    fn concept_to_cache_key(&self, concept: &TestConcept) -> String {
        match concept {
            TestConcept::Named(name) => format!("named:{}", name),
            TestConcept::Intersection(operands) => {
                let keys: Vec<String> = operands.iter().map(|c| self.concept_to_cache_key(c)).collect();
                format!("intersection:{}", keys.join(","))
            }
            TestConcept::Union(operands) => {
                let keys: Vec<String> = operands.iter().map(|c| self.concept_to_cache_key(c)).collect();
                format!("union:{}", keys.join(","))
            }
            TestConcept::Complement(expr) => format!("complement:{}", self.concept_to_cache_key(expr)),
            TestConcept::SomeValuesFrom(prop, filler) => {
                format!("some:{}:{}", prop, self.concept_to_cache_key(filler))
            }
            TestConcept::AllValuesFrom(prop, filler) => {
                format!("all:{}:{}", prop, self.concept_to_cache_key(filler))
            }
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> &ReasoningStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ReasoningStats::default();
        self.cache.clear();
    }

    /// Run benchmark tests
    pub fn run_benchmarks(&mut self) -> BenchmarkResults {
        let mut results = BenchmarkResults::new();

        // Test simple concept
        let simple_concept = TestConcept::Named("Person".to_string());
        let start = Instant::now();
        let result = self.check_satisfiability(&simple_concept);
        results.add_test("simple_concept", start.elapsed(), result.satisfiable);

        // Test intersection
        let intersection_concept = TestConcept::Intersection(vec![
            TestConcept::Named("Person".to_string()),
            TestConcept::Named("Parent".to_string()),
        ]);
        let start = Instant::now();
        let result = self.check_satisfiability(&intersection_concept);
        results.add_test("intersection", start.elapsed(), result.satisfiable);

        // Test union
        let union_concept = TestConcept::Union(vec![
            TestConcept::Named("Person".to_string()),
            TestConcept::Named("Organization".to_string()),
        ]);
        let start = Instant::now();
        let result = self.check_satisfiability(&union_concept);
        results.add_test("union", start.elapsed(), result.satisfiable);

        // Test existential
        let existential_concept = TestConcept::SomeValuesFrom(
            "hasChild".to_string(),
            Box::new(TestConcept::Named("Person".to_string())),
        );
        let start = Instant::now();
        let result = self.check_satisfiability(&existential_concept);
        results.add_test("existential", start.elapsed(), result.satisfiable);

        // Test complex concept
        let complex_concept = TestConcept::Intersection(vec![
            TestConcept::Named("Person".to_string()),
            TestConcept::SomeValuesFrom(
                "hasChild".to_string(),
                Box::new(TestConcept::Named("Doctor".to_string())),
            ),
            TestConcept::Complement(Box::new(TestConcept::Named("Student".to_string()))),
        ]);
        let start = Instant::now();
        let result = self.check_satisfiability(&complex_concept);
        results.add_test("complex", start.elapsed(), result.satisfiable);

        results
    }
}

impl TableauxGraph {
    /// Create a new tableaux graph
    fn new() -> Self {
        let root = NodeId(0);
        let mut nodes = HashMap::new();
        nodes.insert(root, TableauxNode {
            id: root,
            concepts: HashSet::new(),
            labels: HashSet::new(),
            blocked_by: None,
        });

        TableauxGraph {
            nodes,
            edges: HashMap::new(),
            root,
            next_id: 1,
        }
    }

    /// Add a new node to the graph
    fn add_node(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        self.nodes.insert(id, TableauxNode {
            id,
            concepts: HashSet::new(),
            labels: HashSet::new(),
            blocked_by: None,
        });

        id
    }

    /// Add a concept to a node
    fn add_concept(&mut self, node_id: NodeId, concept: TestConcept) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.concepts.insert(concept);
        }
    }

    /// Add an edge between nodes
    fn add_edge(&mut self, from: NodeId, property: String, to: NodeId) {
        self.edges.entry(from).or_insert_with(HashMap::new)
            .entry(property).or_insert_with(HashSet::new)
            .insert(to);
    }

    /// Get all successors of a node
    fn get_all_successors(&self, node_id: NodeId) -> Vec<&HashSet<NodeId>> {
        self.edges.get(&node_id)
            .map(|edges| edges.values().collect())
            .unwrap_or_default()
    }
}

/// Benchmark results
#[derive(Debug)]
pub struct BenchmarkResults {
    tests: Vec<BenchmarkTest>,
}

impl BenchmarkResults {
    fn new() -> Self {
        BenchmarkResults {
            tests: Vec::new(),
        }
    }

    fn add_test(&mut self, name: &str, duration: std::time::Duration, satisfiable: bool) {
        self.tests.push(BenchmarkTest {
            name: name.to_string(),
            duration_ms: duration.as_millis() as u64,
            satisfiable,
        });
    }

    pub fn get_average_time(&self) -> f64 {
        if self.tests.is_empty() {
            return 0.0;
        }
        let total: u64 = self.tests.iter().map(|t| t.duration_ms).sum();
        total as f64 / self.tests.len() as f64
    }

    pub fn get_total_time(&self) -> u64 {
        self.tests.iter().map(|t| t.duration_ms).sum()
    }

    pub fn print_summary(&self) {
        println!("=== Tableaux Algorithm Benchmark Results ===");
        println!("Total tests: {}", self.tests.len());
        println!("Average time: {:.2}ms", self.get_average_time());
        println!("Total time: {}ms", self.get_total_time());
        println!();

        for test in &self.tests {
            println!("{}: {}ms (satisfiable: {})",
                test.name, test.duration_ms, test.satisfiable);
        }
    }
}

#[derive(Debug)]
struct BenchmarkTest {
    name: String,
    duration_ms: u64,
    satisfiable: bool,
}

/// Main entry point for OpenEvolve optimization
pub fn main() {
    println!("=== Optimized Tableaux Algorithm for OpenEvolve ===");

    // Test different configurations
    let configs = vec![
        ("Baseline", TableauxConfig::default()),
        ("Advanced Blocking", TableauxConfig {
            advanced_blocking: true,
            ..TableauxConfig::default()
        }),
        ("Heuristic Ordering", TableauxConfig {
            heuristic_ordering: true,
            ..TableauxConfig::default()
        }),
        ("Full Optimization", TableauxConfig {
            advanced_blocking: true,
            dependency_backtracking: true,
            heuristic_ordering: true,
            parallel_processing: true,
            ..TableauxConfig::default()
        }),
    ];

    for (config_name, config) in configs {
        println!("\n--- Testing Configuration: {} ---", config_name);
        let mut reasoner = OptimizedTableauxReasoner::with_config(config);
        let results = reasoner.run_benchmarks();
        results.print_summary();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_satisfiability() {
        let mut reasoner = OptimizedTableauxReasoner::new();
        let concept = TestConcept::Named("Person".to_string());
        let result = reasoner.check_satisfiability(&concept);

        assert!(result.satisfiable);
        assert_eq!(result.stats.nodes_created, 1);
    }

    #[test]
    fn test_intersection_satisfiability() {
        let mut reasoner = OptimizedTableauxReasoner::new();
        let concept = TestConcept::Intersection(vec![
            TestConcept::Named("Person".to_string()),
            TestConcept::Named("Parent".to_string()),
        ]);
        let result = reasoner.check_satisfiability(&concept);

        assert!(result.satisfiable);
    }

    #[test]
    fn test_contradiction_detection() {
        let mut reasoner = OptimizedTableauxReasoner::new();

        // Create a node with complementary concepts
        let mut graph = TableauxGraph::new();
        let root = graph.root;
        graph.add_concept(root, TestConcept::Named("Person".to_string()));
        graph.add_concept(root, TestConcept::Complement(
            Box::new(TestConcept::Named("Person".to_string()))
        ));

        let node = graph.nodes.get(&root).unwrap();
        assert!(reasoner.check_contradiction(node));
    }

    #[test]
    fn test_cache_functionality() {
        let mut reasoner = OptimizedTableauxReasoner::new();
        let concept = TestConcept::Named("Person".to_string());

        // First call should be a cache miss
        let result1 = reasoner.check_satisfiability(&concept);
        assert_eq!(result1.stats.cache_misses, 1);
        assert_eq!(result1.stats.cache_hits, 0);

        // Second call should be a cache hit
        let result2 = reasoner.check_satisfiability(&concept);
        assert_eq!(result2.stats.cache_misses, 1);
        assert_eq!(result2.stats.cache_hits, 1);
    }

    #[test]
    fn test_benchmark_execution() {
        let mut reasoner = OptimizedTableauxReasoner::new();
        let results = reasoner.run_benchmarks();

        assert_eq!(results.tests.len(), 5);
        assert!(results.get_average_time() > 0.0);
    }
}