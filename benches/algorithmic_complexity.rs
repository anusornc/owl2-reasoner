//! Algorithmic Complexity Testing Benchmarks
//!
//! Tests reasoning performance across different ontology sizes to understand
//! the algorithmic complexity characteristics of different reasoning operations.

use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};
use std::time::Duration;

// Include our test data generation utilities
mod memory_profiler;
mod test_data_generator;

use memory_profiler::{measure_performance, PerformanceResults};
use test_data_generator::{ComplexityLevel, OntologyConfig, OntologyGenerator};

/// Test consistency checking complexity across different ontology sizes
fn bench_consistency_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("consistency_complexity");

    // Configure plot for complexity analysis
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    let sizes = vec![10, 25, 50, 100, 250, 500, 1000, 2000];

    for size in sizes {
        let mut config = OntologyConfig::default();
        config.num_classes = size;
        config.num_subclass_axioms = size * 2;
        config.complexity = ComplexityLevel::Simple;

        let mut generator = OntologyGenerator::new(config);
        let ontology = generator.generate();
        let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

        group.bench_with_input(BenchmarkId::new("consistency", size), size, |b, &_size| {
            b.iter(|| {
                let result = black_box(reasoner.is_consistent().unwrap());
                black_box(result)
            })
        });
    }

    group.finish();
}

/// Test classification complexity
fn bench_classification_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("classification_complexity");

    // Classification is more expensive, so use smaller sizes and longer timeout
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(20); // Reduce sample size for expensive operations

    let sizes = vec![10, 25, 50, 100, 250, 500, 1000];

    for size in sizes {
        let mut config = OntologyConfig::default();
        config.num_classes = size;
        config.num_subclass_axioms = size * 2;
        config.complexity = ComplexityLevel::Medium;

        group.bench_with_input(
            BenchmarkId::new("classification", size),
            size,
            |b, &_size| {
                b.iter(|| {
                    // Create fresh reasoner for each iteration to avoid state effects
                    let mut generator = OntologyGenerator::new(config.clone());
                    let ontology = generator.generate();
                    let mut reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

                    let result = black_box(reasoner.classify().unwrap());
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Test satisfiability complexity
fn bench_satisfiability_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("satisfiability_complexity");

    let sizes = vec![10, 25, 50, 100, 250, 500, 1000, 2000];

    for size in sizes {
        let mut config = OntologyConfig::default();
        config.num_classes = size;
        config.num_subclass_axioms = size * 2;
        config.complexity = ComplexityLevel::Simple;

        let mut generator = OntologyGenerator::new(config);
        let ontology = generator.generate();
        let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

        // Get a class IRI for testing
        if let Some(first_class) = reasoner.ontology().classes().next() {
            let class_iri = first_class.iri().clone();

            group.bench_with_input(
                BenchmarkId::new("satisfiability", size),
                size,
                |b, &_size| {
                    b.iter(|| {
                        let result = black_box(reasoner.is_class_satisfiable(&class_iri).unwrap());
                        black_box(result)
                    })
                },
            );
        }
    }

    group.finish();
}

/// Test complexity impact of different ontology features
fn bench_feature_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("feature_complexity");

    let sizes = vec![100, 500, 1000];
    let complexities = vec![
        (ComplexityLevel::Simple, "simple"),
        (ComplexityLevel::Medium, "medium"),
        (ComplexityLevel::Complex, "complex"),
    ];

    for size in sizes {
        for (complexity, complexity_name) in complexities {
            let mut config = OntologyConfig::default();
            config.num_classes = size;
            config.num_subclass_axioms = size * 2;
            config.complexity = complexity;

            group.bench_with_input(
                BenchmarkId::new(format!("consistency_{}", complexity_name), size),
                size,
                |b, &_size| {
                    b.iter(|| {
                        let mut generator = OntologyGenerator::new(config.clone());
                        let ontology = generator.generate();
                        let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

                        let result = black_box(reasoner.is_consistent().unwrap());
                        black_box(result)
                    })
                },
            );
        }
    }

    group.finish();
}

/// Test memory usage complexity
fn bench_memory_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_complexity");
    group.measurement_time(Duration::from_secs(60)); // Longer time for memory measurements

    let sizes = vec![10, 50, 100, 500, 1000, 2000];

    for size in sizes {
        let mut config = OntologyConfig::default();
        config.num_classes = size;
        config.num_subclass_axioms = size * 3; // More axioms for memory pressure
        config.complexity = ComplexityLevel::Medium;

        group.bench_with_input(BenchmarkId::new("memory_usage", size), size, |b, &_size| {
            b.iter(|| {
                let mut generator = OntologyGenerator::new(config.clone());
                let ontology = generator.generate();

                let (reasoner, measurement) = measure_performance("reasoning_creation", || {
                    owl2_reasoner::SimpleReasoner::new(ontology)
                });

                // Perform reasoning to activate memory usage
                let _result = black_box(reasoner.is_consistent().unwrap());

                // Return memory measurement (though Criterion doesn't directly use it)
                black_box(measurement)
            })
        });
    }

    group.finish();
}

/// Comprehensive complexity analysis
fn run_comprehensive_analysis() -> PerformanceResults {
    let mut results = PerformanceResults::new();

    println!("=== Running Comprehensive Complexity Analysis ===");

    let sizes = vec![10, 50, 100, 500, 1000];
    let complexities = vec![
        ComplexityLevel::Simple,
        ComplexityLevel::Medium,
        ComplexityLevel::Complex,
    ];

    for size in sizes {
        for complexity in complexities.iter() {
            println!("Testing size: {}, complexity: {:?}", size, complexity);

            let mut config = OntologyConfig::default();
            config.num_classes = size;
            config.num_subclass_axioms = size * 2;
            config.complexity = *complexity;

            // Test consistency checking
            let (_, measurement) = measure_performance("consistency_check", || {
                let mut generator = OntologyGenerator::new(config.clone());
                let ontology = generator.generate();
                let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
                reasoner.is_consistent().unwrap()
            });
            results.add_measurement(measurement);

            // Test classification for smaller sizes only
            if size <= 500 {
                let (_, measurement) = measure_performance("classification", || {
                    let mut generator = OntologyGenerator::new(config.clone());
                    let ontology = generator.generate();
                    let mut reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
                    reasoner.classify().unwrap()
                });
                results.add_measurement(measurement);
            }

            // Test satisfiability
            let mut generator = OntologyGenerator::new(config.clone());
            let ontology = generator.generate();
            let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

            if let Some(first_class) = reasoner.ontology().classes().next() {
                let class_iri = first_class.iri().clone();

                let (_, measurement) = measure_performance("satisfiability_check", || {
                    reasoner.is_class_satisfiable(&class_iri).unwrap()
                });
                results.add_measurement(measurement);
            }
        }
    }

    results.complete();
    results
}

/// Analyze and report complexity results
fn analyze_complexity_results() {
    println!("\n=== Complexity Analysis ===");
    let results = run_comprehensive_analysis();

    println!("{}", results.generate_summary());

    // Print detailed memory report
    println!("\n=== Memory Usage Report ===");
    println!(
        "{}",
        memory_profiler::utils::generate_memory_report(&results.measurements)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_setup() {
        let config = OntologyConfig {
            num_classes: 100,
            num_subclass_axioms: 200,
            complexity: ComplexityLevel::Medium,
            ..Default::default()
        };

        let mut generator = OntologyGenerator::new(config);
        let ontology = generator.generate();
        let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

        // Should be able to perform basic operations
        assert!(reasoner.is_consistent().unwrap());
        assert!(reasoner.ontology().classes().count() == 100);
    }

    #[test]
    fn test_performance_measurement() {
        let (result, measurement) = measure_performance("test", || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(measurement.duration_ms >= 8.0); // Allow some variance
        assert_eq!(measurement.operation_name, "test");
    }
}

// Uncomment to run the comprehensive analysis when this benchmark is executed directly
// pub fn main() {
//     analyze_complexity_results();
// }

criterion_group!(
    complexity_benchmarks,
    bench_consistency_complexity,
    bench_classification_complexity,
    bench_satisfiability_complexity,
    bench_feature_complexity,
    bench_memory_complexity
);

criterion_main!(complexity_benchmarks);
