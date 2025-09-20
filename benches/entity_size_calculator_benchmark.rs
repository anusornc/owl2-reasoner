//! Basic Entity Size Calculator Benchmark
//!
//! This benchmark provides basic measurements of entity size calculations
//! without making fake breakthrough claims or hardcoded assertions.

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use owl2_reasoner::axioms::ClassExpression;
use owl2_reasoner::axioms::SubClassOfAxiom;
use owl2_reasoner::entities::{Class, ObjectProperty};
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::simple::SimpleReasoner;
use owl2_reasoner::validation::memory_profiler::EntitySizeCalculator;
use std::time::Instant;

/// Main benchmark suite for entity size calculations
fn entity_size_benchmark_suite(c: &mut Criterion) {
    println!("📊 Entity Size Calculator Benchmark");
    println!("=================================");

    // Measure basic calculation performance
    measure_calculation_performance(c);
    measure_memory_usage(c);
    measure_scaling_performance(c);

    println!("=================================");
    println!("✅ Entity size benchmark complete!");
}

/// Measure basic calculation performance
fn measure_calculation_performance(c: &mut Criterion) {
    println!("⚡ MEASUREMENT: Calculation Performance");
    println!("   Measuring entity size calculation speed");

    let mut group = c.benchmark_group("calculation_speed");

    for size in [100, 500, 1000].iter() {
        let ontology = create_test_ontology(*size);

        group.bench_with_input(BenchmarkId::new("calculation_time", size), size, |b, _| {
            b.iter(|| {
                let mut total_bytes = 0;

                // Time the EntitySizeCalculator performance
                for class in ontology.classes() {
                    total_bytes += EntitySizeCalculator::estimate_class_size(black_box(class));
                }

                for prop in ontology.object_properties() {
                    total_bytes +=
                        EntitySizeCalculator::estimate_object_property_size(black_box(prop));
                }

                for axiom in ontology.subclass_axioms() {
                    total_bytes +=
                        EntitySizeCalculator::estimate_subclass_axiom_size(black_box(axiom));
                }

                // Return result (no assertions - just measurement)
                black_box(total_bytes);
            })
        });
    }

    group.finish();
    println!("   ✅ Calculation performance measured");
}

/// Measure memory usage patterns
fn measure_memory_usage(c: &mut Criterion) {
    println!("💾 MEASUREMENT: Memory Usage");
    println!("   Measuring estimated entity memory usage");

    let mut group = c.benchmark_group("memory_usage");

    for size in [50, 200, 800].iter() {
        let ontology = create_test_ontology(*size);

        group.bench_with_input(BenchmarkId::new("memory_estimation", size), size, |b, _| {
            b.iter(|| {
                let mut total_bytes = 0;
                let mut entity_count = 0;

                // Calculate all entity sizes
                for class in ontology.classes() {
                    total_bytes += EntitySizeCalculator::estimate_class_size(class);
                    entity_count += 1;
                }

                for prop in ontology.object_properties() {
                    total_bytes += EntitySizeCalculator::estimate_object_property_size(prop);
                    entity_count += 1;
                }

                for axiom in ontology.subclass_axioms() {
                    total_bytes += EntitySizeCalculator::estimate_subclass_axiom_size(axiom);
                    entity_count += 1;
                }

                let average_bytes = total_bytes / entity_count.max(1);

                // Return measurement results (no assertions)
                black_box((total_bytes, entity_count, average_bytes));
            })
        });
    }

    group.finish();
    println!("   ✅ Memory usage measured");
}

/// Measure scaling performance with larger ontologies
fn measure_scaling_performance(c: &mut Criterion) {
    println!("📈 MEASUREMENT: Scaling Performance");
    println!("   Measuring performance with ontology size scaling");

    let mut group = c.benchmark_group("scaling_performance");

    for size in [100, 500, 2000].iter() {
        let ontology = create_test_ontology(*size);

        group.bench_with_input(BenchmarkId::new("scaling_time", size), size, |b, _| {
            b.iter(|| {
                let start_time = Instant::now();

                // Perform comprehensive size calculation
                let mut total_bytes = 0;

                for class in ontology.classes() {
                    total_bytes += EntitySizeCalculator::estimate_class_size(&class);
                }

                for prop in ontology.object_properties() {
                    total_bytes += EntitySizeCalculator::estimate_object_property_size(&prop);
                }

                for axiom in ontology.subclass_axioms() {
                    total_bytes += EntitySizeCalculator::estimate_subclass_axiom_size(&axiom);
                }

                let elapsed_time = start_time.elapsed();
                let entities_count = ontology.classes().len()
                    + ontology.object_properties().len()
                    + ontology.subclass_axioms().len();

                // Return scaling metrics (no assertions)
                black_box((total_bytes, entities_count, elapsed_time));
            })
        });
    }

    group.finish();
    println!("   ✅ Scaling performance measured");
}

/// Create test ontology for benchmarking
fn create_test_ontology(entity_count: usize) -> Ontology {
    let mut ontology = Ontology::new();

    // Add classes
    for i in 0..entity_count {
        let iri = IRI::new(&format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(iri);
        ontology.add_class(class).unwrap();
    }

    // Add properties
    for i in 0..(entity_count / 10).max(1) {
        let iri = IRI::new(&format!("http://example.org/hasProperty{}", i)).unwrap();
        let prop = ObjectProperty::new(iri);
        ontology.add_object_property(prop).unwrap();
    }

    // Add subclass relationships
    for i in 1..(entity_count / 5).max(1) {
        let child_iri = IRI::new(&format!("http://example.org/Class{}", i)).unwrap();
        let parent_iri = IRI::new(&format!("http://example.org/Class{}", i / 2)).unwrap();

        let child = ClassExpression::Class(Class::new(child_iri));
        let parent = ClassExpression::Class(Class::new(parent_iri));
        let axiom = SubClassOfAxiom::new(child, parent);
        ontology.add_subclass_axiom(axiom).unwrap();
    }

    ontology
}

criterion_group!(benches, entity_size_benchmark_suite);
criterion_main!(benches);
