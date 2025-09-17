//! Main benchmark runner for OWL2 Reasoner
//!
//! This file runs all benchmarks for the OWL2 reasoning system.

use criterion::{Criterion, criterion_group, criterion_main};

mod mod_bench;
use mod_bench::*;

fn benchmark_suite(c: &mut Criterion) {
    println!("Running OWL2 Reasoner Benchmark Suite...");
    println!("==========================================");

    // Reasoning benchmarks
    bench_consistency_checking(c);
    bench_class_satisfiability(c);
    bench_cache_operations(c);

    // Parser benchmarks
    bench_turtle_parsing(c);

    // Query benchmarks
    bench_query_engine_creation(c);
    bench_simple_queries(c);
    bench_class_queries(c);
    bench_subclass_queries(c);

    // Memory benchmarks
    bench_ontology_memory_usage(c);
    bench_reasoner_memory_usage(c);
    bench_cache_memory_usage(c);
    bench_ontology_cloning(c);

    // Scalability benchmarks
    bench_large_ontology_handling(c);
    bench_ontology_loading(c);
    bench_deep_hierarchy_reasoning(c);
    bench_wide_hierarchy_reasoning(c);
    bench_concurrent_access(c);

    println!("==========================================");
    println!("Benchmark suite completed!");
}

criterion_group!(benches, benchmark_suite);
criterion_main!(benches);
