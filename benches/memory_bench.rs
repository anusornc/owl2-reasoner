//! Memory usage benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::entities::{Class, NamedIndividual};
use owl2_reasoner::axioms::{SubClassOfAxiom, ClassExpression};
use owl2_reasoner::reasoning::SimpleReasoner;
use owl2_reasoner::iri::IRI;

/// Benchmark memory usage for ontology creation
pub fn bench_ontology_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("ontology_memory");
    
    for size in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::new("create_ontology", size), size, |b, size| {
            b.iter(|| {
                let ontology = create_memory_intensive_ontology(*size);
                black_box(ontology);
            })
        });
    }
    
    group.finish();
}

/// Benchmark memory usage for reasoner creation
pub fn bench_reasoner_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("reasoner_memory");
    
    for size in [100, 500, 1000, 5000].iter() {
        let ontology = create_memory_intensive_ontology(*size);
        
        group.bench_with_input(BenchmarkId::new("create_reasoner", size), size, |b, _| {
            b.iter(|| {
                let reasoner = SimpleReasoner::new(black_box(ontology.clone()));
                black_box(reasoner);
            })
        });
    }
    
    group.finish();
}

/// Benchmark memory usage with cache operations
pub fn bench_cache_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_memory");
    
    for size in [100, 500, 1000, 5000].iter() {
        let ontology = create_memory_intensive_ontology(*size);
        let mut reasoner = SimpleReasoner::new(ontology);
        
        // Perform some operations to populate cache
        let _ = reasoner.is_consistent();
        
        group.bench_with_input(BenchmarkId::new("cache_operations", size), size, |b, _| {
            b.iter(|| {
                reasoner.clear_caches();
                let _ = reasoner.is_consistent();
                let _ = reasoner.cache_stats();
                black_box(());
            })
        });
    }
    
    group.finish();
}

/// Benchmark cloning large ontologies
pub fn bench_ontology_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("ontology_cloning");
    
    for size in [100, 500, 1000, 5000].iter() {
        let ontology = create_memory_intensive_ontology(*size);
        
        group.bench_with_input(BenchmarkId::new("clone_ontology", size), size, |b, _| {
            b.iter(|| {
                let cloned = black_box(ontology.clone());
                black_box(cloned);
            })
        });
    }
    
    group.finish();
}

/// Helper function to create a memory-intensive ontology
fn create_memory_intensive_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();
    let mut classes = Vec::new();
    
    // Create classes
    for i in 0..size {
        let iri = IRI::new(&format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(iri);
        ontology.add_class(class.clone()).unwrap();
        classes.push(class);
    }
    
    // Create individuals
    for i in 0..size * 2 {
        let iri = IRI::new(&format!("http://example.org/Individual{}", i)).unwrap();
        let individual = NamedIndividual::new(iri);
        ontology.add_named_individual(individual).unwrap();
    }
    
    // Create hierarchical relationships
    for i in 1..classes.len() {
        let parent_idx = (i - 1) / 4; // Create a wider tree structure
        let subclass_axiom = SubClassOfAxiom::new(
            ClassExpression::Class(classes[i].clone()),
            ClassExpression::Class(classes[parent_idx].clone()),
        );
        ontology.add_subclass_axiom(subclass_axiom).unwrap();
    }
    
    ontology
}