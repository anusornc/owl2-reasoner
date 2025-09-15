//! Demo program for tableaux algorithm optimization
//!
//! This is a simplified version that can be easily compiled and tested
//! for OpenEvolve optimization.

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

// Simplified concepts for optimization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TestConcept {
    Named(String),
    Intersection(Vec<TestConcept>),
    Union(Vec<TestConcept>),
    Complement(Box<TestConcept>),
    SomeValuesFrom(String, Box<TestConcept>),
    AllValuesFrom(String, Box<TestConcept>),
}

// Simplified node structure
#[derive(Debug, Clone)]
struct TableauxNode {
    id: usize,
    concepts: HashSet<TestConcept>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct NodeId(usize);

// Simplified graph structure
#[derive(Debug)]
struct TableauxGraph {
    nodes: HashMap<NodeId, TableauxNode>,
    edges: HashMap<NodeId, HashMap<String, HashSet<NodeId>>>,
    root: NodeId,
    next_id: usize,
}

// Performance statistics
#[derive(Debug, Clone, Default)]
pub struct ReasoningStats {
    pub time_ms: u64,
    pub nodes_created: usize,
    pub rules_applied: usize,
    pub backtracks: usize,
    pub cache_hits: usize,
}

// Tableaux result
#[derive(Debug, Clone)]
pub struct TableauxResult {
    pub satisfiable: bool,
    pub explanation: Option<String>,
    pub stats: ReasoningStats,
}

// Main tableaux reasoner for optimization
pub struct OptimizedTableauxReasoner {
    graph: TableauxGraph,
    stats: ReasoningStats,
    cache: HashMap<String, bool>,
}

impl OptimizedTableauxReasoner {
    pub fn new() -> Self {
        OptimizedTableauxReasoner {
            graph: TableauxGraph::new(),
            stats: ReasoningStats::default(),
            cache: HashMap::new(),
        }
    }

    // Main optimization target: tableaux algorithm
    pub fn check_satisfiability(&mut self, concept: &TestConcept) -> TableauxResult {
        let start_time = Instant::now();

        // Check cache
        let cache_key = self.concept_to_key(concept);
        if let Some(&result) = self.cache.get(&cache_key) {
            self.stats.cache_hits += 1;
            return TableauxResult {
                satisfiable: result,
                explanation: Some("Cache hit".to_string()),
                stats: self.stats.clone(),
            };
        }

        // Initialize tableaux
        self.graph = TableauxGraph::new();
        let root = self.graph.root;
        self.graph.add_concept(root, concept.clone());

        // Run tableaux algorithm
        let result = self.run_tableaux(concept);

        // Cache result
        self.cache.insert(cache_key, result.satisfiable);
        self.stats.time_ms = start_time.elapsed().as_millis() as u64;

        result
    }

    // Core tableaux algorithm (main optimization target)
    fn run_tableaux(&mut self, _concept: &TestConcept) -> TableauxResult {
        let mut queue = VecDeque::new();
        queue.push_back(self.graph.root);

        while let Some(node_id) = queue.pop_front() {
            // Clone concepts to avoid borrowing issues
            let concepts: Vec<TestConcept> = self.graph.nodes.get(&node_id)
                .map(|node| node.concepts.iter().cloned().collect())
                .unwrap_or_default();

            // Apply rules
            for concept in concepts {
                if let Some(new_concepts) = self.apply_rules(&concept, node_id) {
                    self.stats.rules_applied += 1;

                    for new_concept in new_concepts {
                        self.graph.add_concept(node_id, new_concept);
                    }
                }
            }

            // Check contradictions
            if let Some(node) = self.graph.nodes.get(&node_id) {
                if self.check_contradiction(node) {
                    self.stats.backtracks += 1;
                    return TableauxResult {
                        satisfiable: false,
                        explanation: Some("Contradiction".to_string()),
                        stats: self.stats.clone(),
                    };
                }
            }

            // Add successors to queue
            for successors in self.graph.get_successors(node_id) {
                for successor_id in successors {
                    queue.push_back(*successor_id);
                }
            }
        }

        TableauxResult {
            satisfiable: true,
            explanation: None,
            stats: self.stats.clone(),
        }
    }

    // Rule application (optimization target)
    fn apply_rules(&mut self, concept: &TestConcept, node_id: NodeId) -> Option<Vec<TestConcept>> {
        match concept {
            TestConcept::Intersection(operands) => Some(operands.clone()),
            TestConcept::Union(operands) => operands.first().cloned().map(|c| vec![c]),
            TestConcept::SomeValuesFrom(property, filler) => {
                let successor_id = self.graph.add_node();
                self.graph.add_edge(node_id, property.clone(), successor_id);
                self.graph.add_concept(successor_id, (**filler).clone());
                self.stats.nodes_created += 1;
                Some(vec![(**filler).clone()])
            }
            _ => None,
        }
    }

    // Contradiction detection (optimization target)
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

    fn concept_to_key(&self, concept: &TestConcept) -> String {
        match concept {
            TestConcept::Named(name) => format!("named:{}", name),
            TestConcept::Intersection(operands) => {
                let keys: Vec<String> = operands.iter().map(|c| self.concept_to_key(c)).collect();
                format!("intersection:{}", keys.join(","))
            }
            TestConcept::Union(operands) => {
                let keys: Vec<String> = operands.iter().map(|c| self.concept_to_key(c)).collect();
                format!("union:{}", keys.join(","))
            }
            TestConcept::Complement(expr) => format!("complement:{}", self.concept_to_key(expr)),
            TestConcept::SomeValuesFrom(prop, filler) => {
                format!("some:{}:{}", prop, self.concept_to_key(filler))
            }
            TestConcept::AllValuesFrom(prop, filler) => {
                format!("all:{}:{}", prop, self.concept_to_key(filler))
            }
        }
    }

    pub fn get_stats(&self) -> &ReasoningStats {
        &self.stats
    }
}

impl TableauxGraph {
    fn new() -> Self {
        let root = NodeId(0);
        let mut nodes = HashMap::new();
        nodes.insert(root, TableauxNode {
            id: 0,
            concepts: HashSet::new(),
        });

        TableauxGraph {
            nodes,
            edges: HashMap::new(),
            root,
            next_id: 1,
        }
    }

    fn add_node(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        self.nodes.insert(id, TableauxNode {
            id: self.next_id - 1,
            concepts: HashSet::new(),
        });

        id
    }

    fn add_concept(&mut self, node_id: NodeId, concept: TestConcept) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.concepts.insert(concept);
        }
    }

    fn add_edge(&mut self, from: NodeId, property: String, to: NodeId) {
        self.edges.entry(from).or_insert_with(HashMap::new)
            .entry(property).or_insert_with(HashSet::new)
            .insert(to);
    }

    fn get_successors(&self, node_id: NodeId) -> Vec<&HashSet<NodeId>> {
        self.edges.get(&node_id)
            .map(|edges| edges.values().collect())
            .unwrap_or_default()
    }
}

// Benchmark function
pub fn run_benchmark() -> f64 {
    let mut reasoner = OptimizedTableauxReasoner::new();
    let mut total_time = 0.0;
    let num_tests = 100;

    for i in 0..num_tests {
        let concept = TestConcept::Intersection(vec![
            TestConcept::Named(format!("Class{}", i % 10)),
            TestConcept::SomeValuesFrom(
                format!("prop{}", i % 5),
                Box::new(TestConcept::Named(format!("Target{}", i % 8))),
            ),
        ]);

        let start = Instant::now();
        let result = reasoner.check_satisfiability(&concept);
        total_time += start.elapsed().as_secs_f64();

        if !result.satisfiable {
            panic!("Expected satisfiable concept");
        }
    }

    total_time / num_tests as f64 * 1000.0 // Return average time in milliseconds
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--test" => {
                if args.len() > 2 {
                    run_correctness_test(&args[2]);
                } else {
                    println!("Usage: --test <test_type>");
                }
            }
            "--benchmark" => {
                let avg_time = run_benchmark();
                println!("Average reasoning time: {:.3}ms", avg_time);
            }
            "--memory-test" => {
                run_memory_test();
            }
            "--scalability" => {
                if args.len() > 2 {
                    run_scalability_test(&args[2]);
                } else {
                    println!("Usage: --scalability <complexity>");
                }
            }
            _ => {
                run_standard_demo();
            }
        }
    } else {
        run_standard_demo();
    }
}

fn run_standard_demo() {
    println!("=== Tableaux Algorithm Optimization Demo ===");

    // Run benchmark
    let avg_time = run_benchmark();
    println!("Average reasoning time: {:.3}ms", avg_time);

    // Test individual cases
    let mut reasoner = OptimizedTableauxReasoner::new();

    let test_cases = vec![
        ("Simple", TestConcept::Named("Person".to_string())),
        ("Intersection", TestConcept::Intersection(vec![
            TestConcept::Named("Person".to_string()),
            TestConcept::Named("Parent".to_string()),
        ])),
        ("Union", TestConcept::Union(vec![
            TestConcept::Named("Person".to_string()),
            TestConcept::Named("Organization".to_string()),
        ])),
        ("Existential", TestConcept::SomeValuesFrom(
            "hasChild".to_string(),
            Box::new(TestConcept::Named("Person".to_string())),
        )),
        ("Complex", TestConcept::Intersection(vec![
            TestConcept::Named("Person".to_string()),
            TestConcept::SomeValuesFrom(
                "hasChild".to_string(),
                Box::new(TestConcept::Named("Doctor".to_string())),
            ),
            TestConcept::Complement(Box::new(TestConcept::Named("Student".to_string()))),
        ])),
    ];

    println!("\n=== Individual Test Cases ===");
    for (name, concept) in test_cases {
        let start = Instant::now();
        let result = reasoner.check_satisfiability(&concept);
        let duration = start.elapsed().as_millis() as u64;

        println!("{}: {}ms (satisfiable: {}, nodes: {}, rules: {})",
            name, duration, result.satisfiable, result.stats.nodes_created, result.stats.rules_applied);
    }

    println!("\n=== Performance Summary ===");
    println!("Cache hits: {}", reasoner.get_stats().cache_hits);
    println!("Nodes created: {}", reasoner.get_stats().nodes_created);
    println!("Rules applied: {}", reasoner.get_stats().rules_applied);
    println!("Total time: {}ms", reasoner.get_stats().time_ms);
}

fn run_correctness_test(test_type: &str) {
    let mut reasoner = OptimizedTableauxReasoner::new();

    let (concept, expected_satisfiable) = match test_type {
        "simple" => (
            TestConcept::Named("Person".to_string()),
            true
        ),
        "intersection" => (
            TestConcept::Intersection(vec![
                TestConcept::Named("Person".to_string()),
                TestConcept::Named("Parent".to_string()),
            ]),
            true
        ),
        "union" => (
            TestConcept::Union(vec![
                TestConcept::Named("Person".to_string()),
                TestConcept::Named("Organization".to_string()),
            ]),
            true
        ),
        "existential" => (
            TestConcept::SomeValuesFrom(
                "hasChild".to_string(),
                Box::new(TestConcept::Named("Person".to_string())),
            ),
            true
        ),
        "contradiction" => (
            TestConcept::Intersection(vec![
                TestConcept::Named("Person".to_string()),
                TestConcept::Complement(Box::new(TestConcept::Named("Person".to_string()))),
            ]),
            false
        ),
        "complex" => (
            TestConcept::Intersection(vec![
                TestConcept::Named("Person".to_string()),
                TestConcept::SomeValuesFrom(
                    "hasChild".to_string(),
                    Box::new(TestConcept::Named("Doctor".to_string())),
                ),
                TestConcept::Complement(Box::new(TestConcept::Named("Student".to_string()))),
            ]),
            true
        ),
        _ => (
            TestConcept::Named("Unknown".to_string()),
            true
        ),
    };

    let result = reasoner.check_satisfiability(&concept);
    println!("{} test - satisfiable: {} (expected: {})",
        test_type, result.satisfiable, expected_satisfiable);
}

fn run_memory_test() {
    let mut reasoner = OptimizedTableauxReasoner::new();

    // Simulate memory-intensive operations
    for i in 0..1000 {
        let concept = TestConcept::Intersection(vec![
            TestConcept::Named(format!("Class{}", i % 50)),
            TestConcept::SomeValuesFrom(
                format!("prop{}", i % 20),
                Box::new(TestConcept::Named(format!("Target{}", i % 30))),
            ),
        ]);
        let _result = reasoner.check_satisfiability(&concept);
    }

    println!("Memory test completed");
}

fn run_scalability_test(complexity: &str) {
    let mut reasoner = OptimizedTableauxReasoner::new();

    let num_concepts = match complexity {
        "small" => 10,
        "medium" => 50,
        "large" => 200,
        _ => 20,
    };

    for i in 0..num_concepts {
        let concept = TestConcept::Intersection(vec![
            TestConcept::Named(format!("Class{}", i % 25)),
            TestConcept::SomeValuesFrom(
                format!("prop{}", i % 15),
                Box::new(TestConcept::Named(format!("Target{}", i % 20))),
            ),
        ]);
        let _result = reasoner.check_satisfiability(&concept);
    }

    println!("Scalability test ({}) completed", complexity);
}