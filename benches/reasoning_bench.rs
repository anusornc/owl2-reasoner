//! Reasoning performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::entities::Class;
use owl2_reasoner::axioms::{SubClassOfAxiom, ClassExpression};
use owl2_reasoner::reasoning::SimpleReasoner;
use owl2_reasoner::iri::IRI;

/// Benchmark consistency checking performance
pub fn bench_consistency_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("consistency_checking");
    
    for size in [10, 50, 100, 500].iter() {
        let mut ontology = create_hierarchy_ontology(*size);
        let reasoner = SimpleReasoner::new(ontology);
        
        group.bench_with_input(BenchmarkId::new("simple_consistency", size), size, |b, _| {
            b.iter(|| {
                let result = reasoner.is_consistent();
                black_box(result);
            })
        });
    }
    
    group.finish();
}

/// Benchmark class satisfiability checking
pub fn bench_class_satisfiability(c: &mut Criterion) {
    let mut group = c.benchmark_group("class_satisfiability");
    
    for size in [10, 50, 100, 500].iter() {
        let mut ontology = create_hierarchy_ontology(*size);
        let reasoner = SimpleReasoner::new(ontology);
        
        // Test satisfiability of the first class
        if let Some(first_class) = reasoner.ontology().classes().first() {
            group.bench_with_input(BenchmarkId::new("class_satisfiability", size), size, |b, _| {
                b.iter(|| {
                    let result = reasoner.is_class_satisfiable(&first_class.iri());
                    black_box(result);
                })
            });
        }
    }
    
    group.finish();
}

/// Benchmark cache operations
pub fn bench_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");
    
    for size in [10, 50, 100, 500].iter() {
        let mut ontology = create_hierarchy_ontology(*size);
        let mut reasoner = SimpleReasoner::new(ontology);
        
        group.bench_with_input(BenchmarkId::new("cache_clear", size), size, |b, _| {
            b.iter(|| {
                reasoner.clear_caches();
                black_box(());
            })
        });
        
        group.bench_with_input(BenchmarkId::new("cache_stats", size), size, |b, _| {
            b.iter(|| {
                let stats = reasoner.cache_stats();
                black_box(stats);
            })
        });
    }
    
    group.finish();
}

/// Helper function to create a hierarchical ontology for benchmarking
fn create_hierarchy_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();
    let mut classes = Vec::new();
    
    // Create classes
    for i in 0..size {
        let iri = IRI::new(&format!("http://example.org/class{}", i)).unwrap();
        let class = Class::new(iri);
        ontology.add_class(class.clone()).unwrap();
        classes.push(class);
    }
    
    // Create hierarchical relationships
    for i in 1..classes.len().min(size) {
        let parent_idx = (i - 1) / 2; // Create a binary tree structure
        let subclass_axiom = SubClassOfAxiom::new(
            ClassExpression::Class(classes[i].clone()),
            ClassExpression::Class(classes[parent_idx].clone()),
        );
        ontology.add_subclass_axiom(subclass_axiom).unwrap();
    }
    
    ontology
}