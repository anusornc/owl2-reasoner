//! Reasoning performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use owl2_reasoner::axioms::{ClassExpression, SubClassOfAxiom};
use owl2_reasoner::entities::Class;
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::{
    OwlReasoner, ReasoningConfig, SimpleReasoner, tableaux::ReasoningConfig as TableauxConfig,
};
use owl2_reasoner::Reasoner;

/// Benchmark consistency checking performance across different reasoning modes
pub fn bench_consistency_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("consistency_checking");

    for size in [10, 50, 100, 500].iter() {
        let ontology = create_hierarchy_ontology(*size);

        // Simple Reasoner
        let simple_reasoner = SimpleReasoner::new(ontology.clone());
        group.bench_with_input(
            BenchmarkId::new("simple_consistency", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut reasoner = SimpleReasoner::new(black_box(ontology.clone()));
                    let result = reasoner.is_consistent();
                    black_box(result);
                })
            },
        );

        // Advanced Tableaux Reasoner
        let tableaux_config = TableauxConfig {
            max_depth: 1000,
            debug: false,
            incremental: true,
            timeout: Some(30000),
        };
        let advanced_config = ReasoningConfig {
            enable_reasoning: true,
            use_advanced_reasoning: true,
            tableaux_config,
        };
        // Use advanced reasoning configuration during each iteration
        group.bench_with_input(
            BenchmarkId::new("advanced_tableaux_consistency", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut reasoner = OwlReasoner::with_config(
                        black_box(ontology.clone()),
                        black_box(advanced_config.clone()),
                    );
                    let result = reasoner.is_consistent();
                    black_box(result);
                })
            },
        );

        // Hybrid Reasoner
        let hybrid_config = ReasoningConfig {
            enable_reasoning: true,
            use_advanced_reasoning: true,
            tableaux_config: TableauxConfig {
                max_depth: 2000,
                debug: false,
                incremental: true,
                timeout: Some(60000),
            },
        };
        // Hybrid reasoning configuration
        group.bench_with_input(
            BenchmarkId::new("hybrid_consistency", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut reasoner = OwlReasoner::with_config(
                        black_box(ontology.clone()),
                        black_box(hybrid_config.clone()),
                    );
                    let result = reasoner.is_consistent();
                    black_box(result);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark class satisfiability checking across reasoning modes
pub fn bench_class_satisfiability(c: &mut Criterion) {
    let mut group = c.benchmark_group("class_satisfiability");

    for size in [10, 50, 100, 500].iter() {
        let ontology = create_hierarchy_ontology(*size);

        // Test satisfiability of the first class with SimpleReasoner
        let simple_reasoner = SimpleReasoner::new(ontology.clone());
        if let Some(first_class) = simple_reasoner.ontology.classes().iter().next() {
            group.bench_with_input(
                BenchmarkId::new("simple_satisfiability", size),
                size,
                |b, _| {
                    b.iter(|| {
                        let mut reasoner = SimpleReasoner::new(black_box(ontology.clone()));
                        let result = reasoner.is_class_satisfiable(black_box(&first_class.iri()));
                        black_box(result);
                    })
                },
            );
        }

        // Test with Advanced Tableaux Reasoner
        let tableaux_config = TableauxConfig {
            max_depth: 1000,
            debug: false,
            incremental: true,
            timeout: Some(30000),
        };
        let advanced_config = ReasoningConfig {
            enable_reasoning: true,
            use_advanced_reasoning: true,
            tableaux_config,
        };
        if let Some(first_class) = ontology.classes().iter().next() {
            group.bench_with_input(
                BenchmarkId::new("advanced_tableaux_satisfiability", size),
                size,
                |b, _| {
                    b.iter(|| {
                        let mut reasoner = OwlReasoner::with_config(
                            black_box(ontology.clone()),
                            black_box(advanced_config.clone()),
                        );
                        let result = reasoner.is_class_satisfiable(black_box(&first_class.iri()));
                        black_box(result);
                    })
                },
            );
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

/// Benchmark subclass relationship checking
pub fn bench_subclass_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("subclass_checking");

    for size in [10, 50, 100, 500].iter() {
        let ontology = create_hierarchy_ontology(*size);

        // Simple Reasoner
        let simple_reasoner = SimpleReasoner::new(ontology.clone());
        let mut iter = simple_reasoner.ontology.classes().iter();
        if let (Some(first_class), Some(second_class)) = (iter.next(), iter.next()) {
            group.bench_with_input(BenchmarkId::new("simple_subclass", size), size, |b, _| {
                b.iter(|| {
                    let mut reasoner = SimpleReasoner::new(black_box(ontology.clone()));
                    let result = reasoner.is_subclass_of(
                        black_box(&first_class.iri()),
                        black_box(&second_class.iri()),
                    );
                    black_box(result);
                })
            });
        }

        // Advanced Tableaux Reasoner
        let tableaux_config = TableauxConfig {
            max_depth: 1000,
            debug: false,
            incremental: true,
            timeout: Some(30000),
        };
        let advanced_config = ReasoningConfig {
            enable_reasoning: true,
            use_advanced_reasoning: true,
            tableaux_config,
        };
        let mut iter2 = ontology.classes().iter();
        if let (Some(first_class), Some(second_class)) = (iter2.next(), iter2.next()) {
            group.bench_with_input(
                BenchmarkId::new("advanced_tableaux_subclass", size),
                size,
                |b, _| {
                    b.iter(|| {
                        let mut reasoner = OwlReasoner::with_config(
                            black_box(ontology.clone()),
                            black_box(advanced_config.clone()),
                        );
                        let result = reasoner.is_subclass_of(
                            black_box(&first_class.iri()),
                            black_box(&second_class.iri()),
                        );
                        black_box(result);
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark memory usage across reasoning modes
pub fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    for size in [10, 50, 100, 500].iter() {
        let ontology = create_hierarchy_ontology(*size);

        // Measure memory for SimpleReasoner
        group.bench_with_input(BenchmarkId::new("simple_memory", size), size, |b, _| {
            b.iter(|| {
                let reasoner = SimpleReasoner::new(black_box(ontology.clone()));
                black_box(reasoner);
            })
        });

        // Measure memory for Advanced Tableaux Reasoner
        let tableaux_config = TableauxConfig {
            max_depth: 1000,
            debug: false,
            incremental: true,
            timeout: Some(30000),
        };
        let advanced_config = ReasoningConfig {
            enable_reasoning: true,
            use_advanced_reasoning: true,
            tableaux_config,
        };
        group.bench_with_input(
            BenchmarkId::new("advanced_tableaux_memory", size),
            size,
            |b, _| {
                b.iter(|| {
                    let reasoner = OwlReasoner::with_config(
                        black_box(ontology.clone()),
                        black_box(advanced_config.clone()),
                    );
                    black_box(reasoner);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark large-scale ontologies
pub fn bench_large_scale_ontologies(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale");

    for size in [1000, 5000, 10000].iter() {
        let ontology = create_large_hierarchy_ontology(*size);

        // Simple Reasoner on large ontologies
        group.bench_with_input(
            BenchmarkId::new("simple_large_scale", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut reasoner = SimpleReasoner::new(black_box(ontology.clone()));
                    let result = reasoner.is_consistent();
                    black_box(result);
                })
            },
        );

        // Advanced Tableaux Reasoner on large ontologies
        let tableaux_config = TableauxConfig {
            max_depth: 5000,
            debug: false,
            incremental: true,
            timeout: Some(120000),
        };
        let advanced_config = ReasoningConfig {
            enable_reasoning: true,
            use_advanced_reasoning: true,
            tableaux_config,
        };
        group.bench_with_input(
            BenchmarkId::new("advanced_tableaux_large_scale", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut reasoner = OwlReasoner::with_config(
                        black_box(ontology.clone()),
                        black_box(advanced_config.clone()),
                    );
                    let result = reasoner.is_consistent();
                    black_box(result);
                })
            },
        );
    }

    group.finish();
}

/// Helper function to create a large hierarchical ontology
fn create_large_hierarchy_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();
    let mut classes = Vec::new();

    // Create classes
    for i in 0..size {
        let iri = IRI::new(&format!("http://example.org/large_class{}", i)).unwrap();
        let class = Class::new(iri);
        ontology.add_class(class.clone()).unwrap();
        classes.push(class);
    }

    // Create complex hierarchical relationships
    for i in 1..classes.len().min(size) {
        // Multiple parent relationships for complexity
        for j in 0..(i / 2).min(5) {
            let parent_idx = (i - 1 - j) % i;
            let subclass_axiom = SubClassOfAxiom::new(
                ClassExpression::Class(classes[i].clone()),
                ClassExpression::Class(classes[parent_idx].clone()),
            );
            ontology.add_subclass_axiom(subclass_axiom).unwrap();
        }
    }

    ontology
}
#![allow(unused_imports, unused_must_use, unused_variables, unused_mut)]
