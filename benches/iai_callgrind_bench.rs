//! IAI Callgrind Benchmarks for Deep Performance Analysis
//!
//! Uses Callgrind (via IAI) to perform instruction-level profiling
//! and detailed performance analysis of reasoning operations.

use iai_callgrind::{library_benchmark, main};

// Include our test data generation utilities
mod memory_profiler;
mod test_data_generator;

use memory_profiler::measure_performance;
use test_data_generator::{
    generate_medium_ontology, generate_ontology_with_size, generate_small_ontology,
    ComplexityLevel, OntologyConfig, OntologyGenerator,
};

#[library_benchmark]
fn ontology_creation_small() {
    let _ = generate_small_ontology();
}

#[library_benchmark]
fn ontology_creation_medium() {
    let _ = generate_medium_ontology();
}

#[library_benchmark]
fn ontology_creation_custom_size() {
    let _ = generate_ontology_with_size(100);
}

#[library_benchmark]
fn reasoner_initialization_small() {
    let ontology = generate_small_ontology();
    let _ = owl2_reasoner::SimpleReasoner::new(ontology);
}

#[library_benchmark]
fn reasoner_initialization_medium() {
    let ontology = generate_medium_ontology();
    let _ = owl2_reasoner::SimpleReasoner::new(ontology);
}

#[library_benchmark]
fn consistency_check_small() {
    let ontology = generate_small_ontology();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
    let _ = reasoner.is_consistent().unwrap();
}

#[library_benchmark]
fn consistency_check_medium() {
    let ontology = generate_medium_ontology();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
    let _ = reasoner.is_consistent().unwrap();
}

#[library_benchmark]
fn consistency_check_custom_size() {
    let ontology = generate_ontology_with_size(100);
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
    let _ = reasoner.is_consistent().unwrap();
}

#[library_benchmark]
fn satisfiability_check_small() {
    let ontology = generate_small_ontology();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    if let Some(first_class) = reasoner.ontology().classes().next() {
        let class_iri = first_class.iri().clone();
        let _ = reasoner.is_class_satisfiability(&class_iri).unwrap();
    }
}

#[library_benchmark]
fn satisfiability_check_medium() {
    let ontology = generate_medium_ontology();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    if let Some(first_class) = reasoner.ontology().classes().next() {
        let class_iri = first_class.iri().clone();
        let _ = reasoner.is_class_satisfiability(&class_iri).unwrap();
    }
}

#[library_benchmark]
fn classification_small() {
    let ontology = generate_small_ontology();
    let mut reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
    let _ = reasoner.classify().unwrap();
}

#[library_benchmark]
fn classification_medium() {
    let ontology = generate_medium_ontology();
    let mut reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
    let _ = reasoner.classify().unwrap();
}

#[library_benchmark]
fn subclass_check_small() {
    let ontology = generate_small_ontology();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    let classes: Vec<_> = reasoner.ontology().classes().take(2).collect();
    if classes.len() >= 2 {
        let sub_class_iri = classes[0].iri().clone();
        let super_class_iri = classes[1].iri().clone();
        let _ = reasoner
            .is_subclass_of(&sub_class_iri, &super_class_iri)
            .unwrap();
    }
}

#[library_benchmark]
fn subclass_check_medium() {
    let ontology = generate_medium_ontology();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    let classes: Vec<_> = reasoner.ontology().classes().take(2).collect();
    if classes.len() >= 2 {
        let sub_class_iri = classes[0].iri().clone();
        let super_class_iri = classes[1].iri().clone();
        let _ = reasoner
            .is_subclass_of(&sub_class_iri, &super_class_iri)
            .unwrap();
    }
}

#[library_benchmark]
fn complex_reasoning_workflow() {
    let ontology = generate_ontology_with_size(50);
    let mut reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Full reasoning workflow
    let _ = reasoner.is_consistent().unwrap();

    if let Some(first_class) = reasoner.ontology().classes().next() {
        let class_iri = first_class.iri().clone();
        let _ = reasoner.is_class_satisfiability(&class_iri).unwrap();
    }

    let _ = reasoner.classify().unwrap();
}

#[library_benchmark]
fn reasoning_with_different_complexities() {
    // Test with different complexity levels
    let complexities = vec![
        ComplexityLevel::Simple,
        ComplexityLevel::Medium,
        ComplexityLevel::Complex,
    ];

    for complexity in complexities {
        let mut config = OntologyConfig::default();
        config.num_classes = 50;
        config.num_subclass_axioms = 100;
        config.complexity = complexity;

        let mut generator = OntologyGenerator::new(config);
        let ontology = generator.generate();
        let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
        let _ = reasoner.is_consistent().unwrap();
    }
}

#[library_benchmark]
fn memory_intensive_operations() {
    // Test operations that stress memory allocation
    let ontology = generate_ontology_with_size(200);
    let mut reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Classification is memory intensive
    let _ = reasoner.classify().unwrap();

    // Multiple satisfiability checks
    let classes: Vec<_> = reasoner.ontology().classes().take(10).collect();
    for class in classes {
        let class_iri = class.iri().clone();
        let _ = reasoner.is_class_satisfiability(&class_iri).unwrap();
    }
}

#[library_benchmark]
fn cache_behavior_analysis() {
    let ontology = generate_medium_ontology();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Get some classes for cache testing
    let classes: Vec<_> = reasoner.ontology().classes().take(5).collect();

    // First pass (cache misses)
    for class in &classes {
        let class_iri = class.iri().clone();
        let _ = reasoner.is_class_satisfiability(&class_iri).unwrap();
    }

    // Second pass (cache hits)
    for class in &classes {
        let class_iri = class.iri().clone();
        let _ = reasoner.is_class_satisfiability(&class_iri).unwrap();
    }
}

#[library_benchmark]
fn error_handling_performance() {
    // Test error handling overhead
    let ontology = generate_small_ontology();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Create an invalid IRI for error handling test
    let invalid_iri =
        owl2_reasoner::iri::IRI::new("http://invalid.example.org/nonexistent").unwrap();

    // This should handle errors gracefully
    let _ = reasoner.is_class_satisfiable(&invalid_iri);
}

#[library_benchmark]
fn allocation_patterns() {
    // Test different allocation patterns
    let mut ontologies = Vec::new();

    // Multiple small allocations
    for _ in 0..10 {
        ontologies.push(generate_small_ontology());
    }

    // Process them
    for ontology in ontologies {
        let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
        let _ = reasoner.is_consistent().unwrap();
    }
}

#[library_benchmark]
fn repeated_operations() {
    let ontology = generate_small_ontology();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Repeated operations to test optimization
    for _ in 0..10 {
        let _ = reasoner.is_consistent().unwrap();
    }

    if let Some(first_class) = reasoner.ontology().classes().next() {
        let class_iri = first_class.iri().clone();
        for _ in 0..10 {
            let _ = reasoner.is_class_satisfiability(&class_iri).unwrap();
        }
    }
}

#[library_benchmark]
fn stress_test_large_ontology() {
    // Stress test with larger ontologies
    let mut config = OntologyConfig::default();
    config.num_classes = 200;
    config.num_subclass_axioms = 400;
    config.num_object_properties = 40;
    config.num_individuals = 100;
    config.complexity = ComplexityLevel::Medium;

    let mut generator = OntologyGenerator::new(config);
    let ontology = generator.generate();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Multiple operations
    let _ = reasoner.is_consistent().unwrap();
    let _ = reasoner.classify().unwrap();

    // Multiple satisfiability checks
    let classes: Vec<_> = reasoner.ontology().classes().take(20).collect();
    for class in classes {
        let class_iri = class.iri().clone();
        let _ = reasoner.is_class_satisfiability(&class_iri).unwrap();
    }
}

// Additional performance analysis functions
fn run_instruction_level_analysis() {
    println!("=== Running Instruction-Level Performance Analysis ===");

    let test_cases = vec![
        ("Small Ontology Creation", || generate_small_ontology()),
        ("Medium Ontology Creation", || generate_medium_ontology()),
        ("Small Consistency Check", || {
            let ontology = generate_small_ontology();
            let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
            reasoner.is_consistent().unwrap()
        }),
        ("Medium Consistency Check", || {
            let ontology = generate_medium_ontology();
            let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
            reasoner.is_consistent().unwrap()
        }),
        ("Small Classification", || {
            let ontology = generate_small_ontology();
            let mut reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
            reasoner.classify().unwrap()
        }),
    ];

    for (name, operation) in test_cases {
        println!("Analyzing: {}", name);
        let (_result, measurement) = measure_performance(name, operation);
        println!("  Duration: {:.2} ms", measurement.duration_ms);
        println!(
            "  Memory delta: {} bytes",
            measurement.memory_delta.heap_allocated
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iai_setup() {
        let ontology = generate_small_ontology();
        let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

        // Should be able to perform basic operations
        assert!(reasoner.is_consistent().unwrap());
        assert!(reasoner.ontology().classes().count() > 0);
    }

    #[test]
    fn test_measurement_function() {
        let (result, measurement) = measure_performance("test", || {
            std::thread::sleep(std::time::Duration::from_millis(1));
            42
        });

        assert_eq!(result, 42);
        assert!(measurement.duration_ms >= 1.0);
        assert_eq!(measurement.operation_name, "test");
    }
}

main!(
    ontology_creation_small,
    ontology_creation_medium,
    ontology_creation_custom_size,
    reasoner_initialization_small,
    reasoner_initialization_medium,
    consistency_check_small,
    consistency_check_medium,
    consistency_check_custom_size,
    satisfiability_check_small,
    satisfiability_check_medium,
    classification_small,
    classification_medium,
    subclass_check_small,
    subclass_check_medium,
    complex_reasoning_workflow,
    reasoning_with_different_complexities,
    memory_intensive_operations,
    cache_behavior_analysis,
    error_handling_performance,
    allocation_patterns,
    repeated_operations,
    stress_test_large_ontology
);

// Uncomment to run the detailed analysis when this benchmark is executed directly
// pub fn main() {
//     run_instruction_level_analysis();
// }
