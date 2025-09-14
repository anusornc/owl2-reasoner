// OWL2 Reasoner Optimization with OpenEvolve
// This program implements core OWL2 reasoning algorithms that can be evolved
// to optimize performance, memory usage, and adaptability

use std::collections::{HashMap, HashSet, VecDeque};

// EVOLVE-BLOCK-START
// Initial implementation: Basic reasoning algorithms
// This can be evolved to:
// - Advanced tableaux algorithms with optimizations
// - Hybrid reasoning strategies
// - Parallel processing implementations
// - Memory-efficient data structures
// - Adaptive caching mechanisms

pub struct ReasoningEngine {
    // Basic ontology storage
    classes: HashSet<String>,
    properties: HashSet<String>,
    subclass_relations: HashMap<String, Vec<String>>,

    // Performance tracking
    operations_count: u64,
    total_time_ns: u64,
}

impl ReasoningEngine {
    pub fn new() -> Self {
        ReasoningEngine {
            classes: HashSet::new(),
            properties: HashSet::new(),
            subclass_relations: HashMap::new(),
            operations_count: 0,
            total_time_ns: 0,
        }
    }

    pub fn add_class(&mut self, class: String) {
        self.classes.insert(class);
    }

    pub fn add_property(&mut self, property: String) {
        self.properties.insert(property);
    }

    pub fn add_subclass_relation(&mut self, sub: String, sup: String) {
        self.subclass_relations.entry(sub.clone()).or_insert_with(Vec::new).push(sup);
    }

    // Core reasoning function to evolve: subclass checking
    pub fn is_subclass_of(&mut self, sub_class: &str, super_class: &str) -> bool {
        let start = std::time::Instant::now();

        // Basic implementation with O(n²) complexity
        // This is the target for optimization
        let result = self.is_subclass_of_basic(sub_class, super_class);

        let elapsed = start.elapsed();
        self.operations_count += 1;
        self.total_time_ns += elapsed.as_nanos() as u64;

        result
    }

    // Optimized implementation using Breadth-First Search (BFS) for O(N + E) complexity
    fn is_subclass_of_basic(&self, sub_class: &str, super_class: &str) -> bool {
        if sub_class == super_class {
            return true;
        }

        let mut queue: VecDeque<&str> = VecDeque::new();
        let mut visited: HashSet<&str> = HashSet::new();

        // Start BFS from the sub_class
        queue.push_back(sub_class);
        visited.insert(sub_class);

        while let Some(current_class) = queue.pop_front() {
            // Get direct superclasses of the current class
            if let Some(supers) = self.subclass_relations.get(current_class) {
                for sup in supers {
                    // If the super_class is found, return true
                    if sup == super_class {
                        return true;
                    }
                    // If this superclass hasn't been visited, add it to the queue
                    if visited.insert(sup) { // `insert` returns true if the value was not present
                        queue.push_back(sup);
                    }
                }
            }
        }

        // If the queue is empty and super_class was not found, it's not a subclass
        false
    }

    // Performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            operations_count: self.operations_count,
            total_time_ns: self.total_time_ns,
            avg_time_ns: if self.operations_count > 0 {
                self.total_time_ns / self.operations_count
            } else {
                0
            },
        }
    }
}

#[derive(Debug)]
pub struct PerformanceMetrics {
    pub operations_count: u64,
    pub total_time_ns: u64,
    pub avg_time_ns: u64,
}

// Benchmark function to test reasoning performance
pub fn run_reasoning_benchmark(test_cases: Vec<BenchmarkTestCase>) -> BenchmarkResults {
    let mut engine = ReasoningEngine::new();
    let mut results = BenchmarkResults {
        times: Vec::new(),
        correctness: Vec::new(),
        scalability_score: 0.0,
        memory_efficiency_score: 0.0,
    };

    // Setup test ontology
    setup_test_ontology(&mut engine, &test_cases);

    for test_case in test_cases {
        let start = std::time::Instant::now();

        let result = engine.is_subclass_of(&test_case.sub_class, &test_case.super_class);

        let elapsed = start.elapsed();
        results.times.push(elapsed.as_nanos() as f64);
        results.correctness.push(result == test_case.expected_result);
    }

    // Calculate scalability score (based on performance consistency)
    if results.times.len() > 1 {
        let mean_time: f64 = results.times.iter().sum::<f64>() / results.times.len() as f64;
        let variance: f64 = results.times.iter()
            .map(|t| (t - mean_time).powi(2))
            .sum::<f64>() / results.times.len() as f64;

        // Lower variance means better scalability
        results.scalability_score = 1.0 / (1.0 + variance.sqrt() / 1000.0);
    }

    // Basic memory efficiency calculation
    let metrics = engine.get_performance_metrics();
    results.memory_efficiency_score = if metrics.avg_time_ns > 0 {
        1_000_000.0 / metrics.avg_time_ns as f64 // Operations per second
    } else {
        0.0
    };

    results
}

fn setup_test_ontology(engine: &mut ReasoningEngine, test_cases: &[BenchmarkTestCase]) {
    // Create a test hierarchy based on common patterns
    let mut all_classes = HashSet::new();

    // Collect all unique classes from test cases
    for case in test_cases {
        all_classes.insert(case.sub_class.clone());
        all_classes.insert(case.super_class.clone());
    }

    // Add classes to engine
    for class in all_classes.iter() {
        engine.add_class(class.clone());
    }

    // --- Enhanced Ontology Generation ---
    // Create a deeper, more branched hierarchy for better testing.
    // This is a simplified example; a more robust generator could be used.

    // Base classes
    engine.add_subclass_relation("entity".to_string(), "thing".to_string());
    engine.add_subclass_relation("disease".to_string(), "entity".to_string());
    engine.add_subclass_relation("gene".to_string(), "entity".to_string());
    engine.add_subclass_relation("protein".to_string(), "entity".to_string());
    engine.add_subclass_relation("cell".to_string(), "entity".to_string());

    // Branching from disease
    engine.add_subclass_relation("cancer".to_string(), "disease".to_string());
    engine.add_subclass_relation("infection".to_string(), "disease".to_string());
    engine.add_subclass_relation("autoimmune_disease".to_string(), "disease".to_string());

    // Deeper branching from cancer
    engine.add_subclass_relation("lung_cancer".to_string(), "cancer".to_string());
    engine.add_subclass_relation("breast_cancer".to_string(), "cancer".to_string());
    engine.add_subclass_relation("leukemia".to_string(), "cancer".to_string());

    // Branching from gene
    engine.add_subclass_relation("oncogene".to_string(), "gene".to_string());
    engine.add_subclass_relation("tumor_suppressor_gene".to_string(), "gene".to_string());
    engine.add_subclass_relation("transcription_factor".to_string(), "gene".to_string());

    // Branching from protein
    engine.add_subclass_relation("enzyme".to_string(), "protein".to_string());
    engine.add_subclass_relation("receptor".to_string(), "protein".to_string());

    // Example of a longer chain
    engine.add_subclass_relation("sarcoma".to_string(), "cancer".to_string());
    engine.add_subclass_relation("bone_sarcoma".to_string(), "sarcoma".to_string());
    engine.add_subclass_relation("osteosarcoma".to_string(), "bone_sarcoma".to_string());

    // Ensure all classes mentioned in test cases are also added explicitly
    // This prevents issues if a test case refers to a class not covered by the above
    for case in test_cases {
        if !all_classes.contains(&case.sub_class) {
            engine.add_class(case.sub_class.clone());
            all_classes.insert(case.sub_class.clone());
        }
        if !all_classes.contains(&case.super_class) {
            engine.add_class(case.super_class.clone());
            all_classes.insert(case.super_class.clone());
        }
    }
}

#[derive(Debug)]
pub struct BenchmarkTestCase {
    pub sub_class: String,
    pub super_class: String,
    pub expected_result: bool,
}

#[derive(Debug)]
pub struct BenchmarkResults {
    pub times: Vec<f64>,
    pub correctness: Vec<bool>,
    pub scalability_score: f64,
    pub memory_efficiency_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_reasoning() {
        let mut engine = ReasoningEngine::new();
        engine.add_class("A".to_string());
        engine.add_class("B".to_string());
        engine.add_subclass_relation("A".to_string(), "B".to_string());

        assert!(engine.is_subclass_of("A", "B"));
        assert!(!engine.is_subclass_of("B", "A"));
        assert!(engine.is_subclass_of("A", "A")); // Reflexivity
    }

    #[test]
    fn test_transitive_reasoning() {
        let mut engine = ReasoningEngine::new();
        engine.add_class("A".to_string());
        engine.add_class("B".to_string());
        engine.add_class("C".to_string());
        engine.add_subclass_relation("A".to_string(), "B".to_string());
        engine.add_subclass_relation("B".to_string(), "C".to_string());

        assert!(engine.is_subclass_of("A", "C")); // Transitivity
    }
}
// EVOLVE-BLOCK-END