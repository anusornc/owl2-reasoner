//! Basic Entity Size Calculator Benchmark
//!
//! This benchmark provides basic measurements of entity size calculations
//! without making fake breakthrough claims or hardcoded assertions.
//!
//! TEMPORARILY DISABLED - Depends on validation module

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use owl2_reasoner::axioms::ClassExpression;
use owl2_reasoner::axioms::SubClassOfAxiom;
use owl2_reasoner::entities::{Class, ObjectProperty};
use owl2_reasoner::ontology::Ontology;
// use owl2_reasoner::validation::memory_profiler::EntitySizeCalculator; // Temporarily disabled
use std::time::Instant;

/// Main benchmark suite for entity size calculations
fn entity_size_benchmark_suite(c: &mut Criterion) {
    println!("📊 Entity Size Calculator Benchmark");
    println!("=================================");
    println!("⚠️  Benchmark temporarily disabled - validation module dependency");

    // Create a basic ontology for testing
    let mut ontology = Ontology::new();

    // Add some basic entities
    let person_class = Class::new("http://example.org/Person");
    let animal_class = Class::new("http://example.org/Animal");
    let has_parent_prop = ObjectProperty::new("http://example.org/hasParent");

    ontology.add_class(person_class.clone()).unwrap();
    ontology.add_class(animal_class.clone()).unwrap();
    ontology
        .add_object_property(has_parent_prop.clone())
        .unwrap();

    // Add subclass axiom
    let subclass_axiom = SubClassOfAxiom::new(
        ClassExpression::Class(person_class),
        ClassExpression::Class(animal_class),
    );
    ontology.add_subclass_axiom(subclass_axiom).unwrap();

    // Simple benchmark without validation dependency
    c.bench_function("basic_ontology_operations", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Basic operations that don't depend on validation
            let _classes = ontology.classes().len();
            let _properties = ontology.object_properties().len();
            let _axioms = ontology.axioms().len();
            black_box(start.elapsed());
        })
    });

    println!("✅ Basic benchmark completed");
}

criterion_group!(benches, entity_size_benchmark_suite);

criterion_main!(benches);
