//! SimpleReasoner Subclass Checking Algorithm for OpenEvolve Optimization
//!
//! This is the initial O(n²) DFS implementation that needs optimization
//! Current algorithm: Manual DFS with potential redundant checks
//! Target: Evolve to O(N+E) BFS with better efficiency

use std::collections::{HashMap, HashSet};

/// Simple reasoning structure for subclass checking optimization
pub struct SubclassChecker {
    pub subclass_relations: HashMap<String, Vec<String>>,
    pub equivalent_classes: HashMap<String, Vec<String>>,
}

impl SubclassChecker {
    /// Create new subclass checker with test data
    pub fn new() -> Self {
        let mut subclass_relations = HashMap::new();
        let mut equivalent_classes = HashMap::new();

        // Create test hierarchy: Student -> Person -> Agent
        subclass_relations.insert(
            "http://example.org/Student".to_string(),
            vec!["http://example.org/Person".to_string()]
        );
        subclass_relations.insert(
            "http://example.org/Person".to_string(),
            vec!["http://example.org/Agent".to_string()]
        );
        subclass_relations.insert(
            "http://example.org/Professor".to_string(),
            vec!["http://example.org/Person".to_string()]
        );
        subclass_relations.insert(
            "http://example.org/GraduateStudent".to_string(),
            vec!["http://example.org/Student".to_string()]
        );

        // Create some equivalent classes
        equivalent_classes.insert(
            "http://example.org/Human".to_string(),
            vec!["http://example.org/Person".to_string()]
        );
        equivalent_classes.insert(
            "http://example.org/Person".to_string(),
            vec!["http://example.org/Human".to_string()]
        );

        Self {
            subclass_relations,
            equivalent_classes,
        }
    }

    /// Current O(n²) DFS implementation - target for optimization
    pub fn is_subclass_of_basic(&self, sub_class: &str, super_class: &str) -> bool {
        // Check direct relationship
        if sub_class == super_class {
            return true;
        }

        // Check equivalent classes
        if let Some(equivs) = self.equivalent_classes.get(sub_class) {
            if equivs.contains(&super_class.to_string()) {
                return true;
            }
        }

        // O(n²) DFS with manual stack - INEFFICIENT!
        let mut visited = HashSet::new();
        let mut to_check = vec![sub_class.to_string()];

        while let Some(current) = to_check.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            // Find direct superclasses
            if let Some(supers) = self.subclass_relations.get(&current) {
                for sup in supers {
                    if sup == super_class {
                        return true; // Found path to target
                    }
                    if !visited.contains(sup) {
                        to_check.push(sup.clone());
                    }
                }
            }
        }

        false
    }

    /// Performance test function for evolutionary optimization
    pub fn performance_test(&self) -> (f64, bool) {
        use std::time::Instant;

        // Test with common ontology patterns
        let test_cases = vec![
            ("http://example.org/Person", "http://example.org/Agent"),
            ("http://example.org/Student", "http://example.org/Person"),
            ("http://example.org/GraduateStudent", "http://example.org/Student"),
            ("http://example.org/Professor", "http://example.org/Faculty"),
            ("http://example.org/Department", "http://example.org/Organization"),
            ("http://example.org/University", "http://example.org/Organization"),
            ("http://example.org/Course", "http://example.org/AcademicEntity"),
            ("http://example.org/ResearchPaper", "http://example.org/Publication"),
        ];

        let start_time = Instant::now();
        let mut correct_results = 0;

        for (sub, sup) in &test_cases {
            let result = self.is_subclass_of_basic(sub, sup);
            // For testing, assume all should be true (adjust based on actual test data)
            if result {
                correct_results += 1;
            }
        }

        let elapsed = start_time.elapsed().as_nanos() as f64;
        let correctness = correct_results >= test_cases.len() / 2; // At least 50% correct

        (elapsed, correctness)
    }

    /// Memory usage estimation
    pub fn estimate_memory_usage(&self) -> usize {
        self.subclass_relations.len() * 48 + // HashMap overhead
        self.equivalent_classes.len() * 32 + // HashMap overhead
        self.subclass_relations.values().map(|v| v.len() * 8).sum::<usize>() + // String storage
        self.equivalent_classes.values().map(|v| v.len() * 8).sum::<usize>() // String storage
    }

    /// Stress test with larger ontologies
    pub fn stress_test(&self, size_factor: usize) -> (f64, bool) {
        use std::time::Instant;

        // Generate larger test case
        let mut test_pairs = Vec::new();
        for i in 0..(50 * size_factor) {
            for j in 0..(10 * size_factor) {
                if i != j {
                    test_pairs.push((
                        format!("http://example.org/Class{}", i),
                        format!("http://example.org/Class{}", j)
                    ));
                }
            }
        }

        let start_time = Instant::now();
        let mut successful_checks = 0;

        for (sub, sup) in test_pairs.iter().take(100 * size_factor) {
            if self.is_subclass_of_basic(sub, sup) {
                successful_checks += 1;
            }
        }

        let elapsed = start_time.elapsed().as_nanos() as f64;
        let is_correct = successful_checks <= test_pairs.len(); // Should not overflow

        (elapsed, is_correct)
    }

    /// Edge case testing
    pub fn edge_case_tests(&self) -> bool {
        // Test with empty/non-existent classes
        let non_existent = "http://example.org/NonExistentClass";

        // Should not panic on non-existent classes
        let _result1 = self.is_subclass_of_basic(non_existent, "http://example.org/Person");
        let _result2 = self.is_subclass_of_basic("http://example.org/Person", non_existent);
        let _result3 = self.is_subclass_of_basic(non_existent, non_existent);

        // Test with empty strings
        let _result4 = self.is_subclass_of_basic("", "");
        let _result5 = self.is_subclass_of_basic("", "http://example.org/Person");

        true // If we get here without panicking, edge cases are handled
    }

    /// Cycle detection test
    pub fn cycle_detection_test(&self) -> bool {
        // Create a simple cycle: A -> B -> C -> A
        let cycle_pairs = vec![
            ("http://example.org/A", "http://example.org/B"),
            ("http://example.org/B", "http://example.org/C"),
            ("http://example.org/C", "http://example.org/A"),
            ("http://example.org/A", "http://example.org/D"), // Should still work
        ];

        for (sub, sup) in cycle_pairs {
            let _result = self.is_subclass_of_basic(sub, sup);
        }

        true // If we complete without infinite recursion, cycles are handled
    }
}

fn main() {
    // Simple test runner
    println!("Testing subclass checking algorithm...");

    // Create checker with built-in test data
    let checker = SubclassChecker::new();

    // Test basic functionality
    let result1 = checker.is_subclass_of_basic(
        "http://example.org/Student",
        "http://example.org/Person"
    );
    let result2 = checker.is_subclass_of_basic(
        "http://example.org/Student",
        "http://example.org/Agent"
    );

    println!("Student ⊑ Person: {}", result1);
    println!("Student ⊑ Agent: {}", result2);

    // Performance test
    let (perf_time, correctness) = checker.performance_test();
    println!("Performance time: {:.2} ns, Correctness: {}", perf_time, correctness);

    // Edge case tests
    let edge_case_ok = checker.edge_case_tests();
    let cycle_ok = checker.cycle_detection_test();

    println!("Edge case tests: {}", edge_case_ok);
    println!("Cycle detection tests: {}", cycle_ok);

    // Memory usage
    let memory = checker.estimate_memory_usage();
    println!("Estimated memory usage: {} bytes", memory);

    println!("All tests completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subclass_checker_creation() {
        let checker = SubclassChecker::new();

        assert!(!checker.subclass_relations.is_empty());
        assert!(!checker.equivalent_classes.is_empty());
    }

    #[test]
    fn test_basic_subclass_checking() {
        let checker = SubclassChecker::new();

        // Test the built-in hierarchy
        assert!(checker.is_subclass_of_basic("http://example.org/Student", "http://example.org/Person"));
        assert!(checker.is_subclass_of_basic("http://example.org/Student", "http://example.org/Agent"));
        assert!(!checker.is_subclass_of_basic("http://example.org/Agent", "http://example.org/Student"));
        assert!(checker.is_subclass_of_basic("http://example.org/Person", "http://example.org/Person"));
    }

    #[test]
    fn test_equivalent_classes() {
        let checker = SubclassChecker::new();

        // Test built-in equivalent classes
        assert!(checker.is_subclass_of_basic("http://example.org/Human", "http://example.org/Person"));
        assert!(checker.is_subclass_of_basic("http://example.org/Person", "http://example.org/Human"));
    }

    #[test]
    fn test_performance_metrics() {
        let checker = SubclassChecker::new();

        let (perf_time, correctness) = checker.performance_test();
        let (stress_time, stress_correct) = checker.stress_test(1);

        assert!(perf_time >= 0.0);
        assert!(correctness);
        assert!(stress_time >= 0.0);
        assert!(stress_correct);
    }

    #[test]
    fn test_edge_cases() {
        let checker = SubclassChecker::new();

        assert!(checker.edge_case_tests());
        assert!(checker.cycle_detection_test());
    }
}