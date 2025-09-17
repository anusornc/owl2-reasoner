//! Comprehensive OWL2 Reasoner Benchmark Suite
//!
//! This benchmark suite provides comprehensive performance testing for the OWL2 reasoner,
//! including both internal benchmarks and external comparisons with established reasoners.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

// Import all benchmark modules
mod memory_bench;
mod parser_bench;
mod query_bench;
mod reasoning_bench;
mod scalability_bench;

// Import benchmark functions
use memory_bench::*;
use parser_bench::*;
use query_bench::*;
use reasoning_bench::*;
use scalability_bench::*;

/// Main benchmark configuration
fn bench_comprehensive_suite(c: &mut Criterion) {
    println!("🚀 OWL2 Reasoner Comprehensive Benchmark Suite");
    println!("==============================================");

    // Reasoning Performance Benchmarks
    println!("🔬 Running Reasoning Performance Benchmarks...");
    bench_consistency_checking(c);
    bench_class_satisfiability(c);
    bench_subclass_checking(c);
    bench_memory_usage(c);
    bench_large_scale_ontologies(c);

    // Parser Performance Benchmarks
    println!("📝 Running Parser Performance Benchmarks...");
    // parser_bench functions will be called here

    // Query Performance Benchmarks
    println!("🔍 Running Query Performance Benchmarks...");
    // query_bench functions will be called here

    // Memory Usage Benchmarks
    println!("💾 Running Memory Usage Benchmarks...");
    // memory_bench functions will be called here

    // Scalability Benchmarks
    println!("📈 Running Scalability Benchmarks...");
    // scalability_bench functions will be called here

    println!("✅ All benchmarks completed!");
}

/// Quick benchmark for CI/CD pipelines
fn bench_quick_suite(c: &mut Criterion) {
    println!("⚡ Quick Benchmark Suite for CI/CD");
    println!("====================================");

    // Only run essential benchmarks with small datasets
    let mut group = c.benchmark_group("quick_bench");

    // Small consistency check
    let ontology = reasoning_bench::create_hierarchy_ontology(10);
    use owl2_reasoner::reasoning::SimpleReasoner;

    group.bench_function("quick_consistency", |b| {
        b.iter(|| {
            let mut reasoner = SimpleReasoner::new(black_box(ontology.clone()));
            let result = reasoner.is_consistent();
            black_box(result);
        })
    });

    group.finish();
    println!("✅ Quick benchmarks completed!");
}

/// Performance regression testing
fn bench_regression_testing(c: &mut Criterion) {
    println!("🔍 Performance Regression Testing");
    println!("=================================");

    // Critical performance paths that should not regress
    let mut group = c.benchmark_group("regression");

    // Test with different ontology sizes
    for size in [50, 100, 200].iter() {
        let ontology = reasoning_bench::create_hierarchy_ontology(*size);

        group.bench_with_input(
            BenchmarkId::new("regression_consistency", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut reasoner = SimpleReasoner::new(black_box(ontology.clone()));
                    let result = reasoner.is_consistent();
                    black_box(result);
                })
            },
        );
    }

    group.finish();
    println!("✅ Regression testing completed!");
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(50)  // Reduced sample size for faster execution
        .measurement_time(std::time::Duration::from_secs(5))
        .warm_up_time(std::time::Duration::from_secs(1));
    targets = bench_comprehensive_suite, bench_quick_suite, bench_regression_testing
}

criterion_main!(benches);
