//! EPCIS Performance Benchmark Suite
//! 
//! This example provides comprehensive performance benchmarking for EPCIS reasoning
//! across different scales, query types, and system configurations.

use owl2_reasoner::*;
use owl2_reasoner::epcis_test_generator::*;
use std::time::Instant;

fn main() -> OwlResult<()> {
    println!("🏆 EPCIS Performance Benchmark Suite");
    println!("{}", "=".repeat(50));
    
    // Test configurations for different scales
    let test_configs = vec![
        ("Small Scale", small_scale_config()),
        ("Medium Scale", medium_scale_config()),
        ("Large Scale", large_scale_config()),
    ];
    
    let mut benchmark_results = Vec::new();
    
    for (scale_name, config) in test_configs {
        println!("\n📊 Benchmarking: {}", scale_name);
        println!("{}", "-".repeat(30));
        
        let results = run_comprehensive_benchmarks(scale_name, config)?;
        benchmark_results.push(results);
    }
    
    // Generate comparative analysis
    println!("\n📈 Comparative Performance Analysis");
    println!("{}", "=".repeat(50));
    generate_comparative_analysis(&benchmark_results);
    
    // Performance recommendations
    println!("\n💡 Performance Recommendations");
    println!("{}", "=".repeat(50));
    generate_performance_recommendations(&benchmark_results);
    
    println!("\n🎯 EPCIS performance benchmarking completed successfully!");
    println!("This benchmark suite provides:");
    println!("- Scalability analysis across different dataset sizes");
    println!("- Query performance metrics for different operation types");
    println!("- Memory usage patterns and optimization opportunities");
    println!("- Comparative analysis between different scales");
    println!("- Performance recommendations for production deployment");
    
    Ok(())
}

/// Run comprehensive benchmarks for a given configuration
fn run_comprehensive_benchmarks(scale_name: &str, config: TestDataConfig) -> OwlResult<BenchmarkResults> {
    let mut results = BenchmarkResults {
        scale_name: scale_name.to_string(),
        event_count: config.event_count,
        generation_time: std::time::Duration::new(0, 0),
        memory_usage: 0,
        query_performance: std::collections::HashMap::new(),
        reasoning_performance: std::collections::HashMap::new(),
        scalability_metrics: ScalabilityMetrics::default(),
    };
    
    // Benchmark 1: Data Generation Performance
    println!("1. Data Generation Performance...");
    let gen_start = Instant::now();
    let mut generator = EPCISTestDataGenerator::new(config);
    let ontology = generator.generate_ontology()?;
    let events = generator.generate_events();
    results.generation_time = gen_start.elapsed();
    
    println!("   Generated {} events in {:?}", events.len(), results.generation_time);
    println!("   Generation rate: {:.0} events/sec", 
        events.len() as f64 / results.generation_time.as_secs_f64());
    
    // Estimate memory usage
    results.memory_usage = estimate_memory_usage(&events);
    println!("   Estimated memory usage: ~{} MB", results.memory_usage);
    
    // Benchmark 2: Query Performance
    println!("2. Query Performance Testing...");
    benchmark_query_performance(&events, &mut results)?;
    
    // Benchmark 3: Reasoning Performance
    println!("3. Reasoning Performance Testing...");
    benchmark_reasoning_performance(&ontology, &events, &mut results)?;
    
    // Benchmark 4: Scalability Metrics
    println!("4. Scalability Analysis...");
    benchmark_scalability(&events, &mut results);
    
    Ok(results)
}

/// Estimate memory usage for events
fn estimate_memory_usage(events: &[EPCISEvent]) -> usize {
    let base_size = events.len() * std::mem::size_of::<EPCISEvent>();
    let epc_strings_size: usize = events.iter()
        .map(|e| e.epc_list.iter().map(|s| s.len()).sum::<usize>())
        .sum();
    let child_epcs_size: usize = events.iter()
        .filter_map(|e| e.child_epcs.as_ref())
        .map(|children| children.iter().map(|s| s.len()).sum::<usize>())
        .sum();
    
    (base_size + epc_strings_size + child_epcs_size) / (1024 * 1024)
}

/// Benchmark query performance
fn benchmark_query_performance(events: &[EPCISEvent], results: &mut BenchmarkResults) -> OwlResult<()> {
    // Test 1: Simple EPC Lookup
    let start = Instant::now();
    let sample_epcs: Vec<_> = events.iter()
        .flat_map(|e| e.epc_list.iter())
        .take(100)
        .cloned()
        .collect();
    
    for epc in &sample_epcs {
        let _found: Vec<_> = events.iter()
            .filter(|e| e.epc_list.contains(epc))
            .collect();
    }
    let epc_lookup_time = start.elapsed();
    results.query_performance.insert("EPC Lookup".to_string(), epc_lookup_time);
    println!("   EPC Lookup (100 queries): {:?}", epc_lookup_time);
    
    // Test 2: Business Step Filtering
    let start = Instant::now();
    let _manufacturing_events: Vec<_> = events.iter()
        .filter(|e| e.biz_step == Some(EPCISBusinessStep::Manufacturing))
        .collect();
    let filter_time = start.elapsed();
    results.query_performance.insert("Business Step Filter".to_string(), filter_time);
    println!("   Business Step Filter: {:?}", filter_time);
    
    // Test 3: Complex Aggregation Query
    let start = Instant::now();
    let aggregation_events: Vec<_> = events.iter()
        .filter(|e| e.event_type == EPCISEventType::AggregationEvent)
        .collect();
    
    let total_child_epcs: usize = aggregation_events.iter()
        .filter_map(|e| e.child_epcs.as_ref())
        .map(|children| children.len())
        .sum();
    let agg_query_time = start.elapsed();
    results.query_performance.insert("Aggregation Analysis".to_string(), agg_query_time);
    println!("   Aggregation Analysis ({} events, {} children): {:?}", 
        aggregation_events.len(), total_child_epcs, agg_query_time);
    
    // Test 4: Temporal Range Query
    let start = Instant::now();
    if let (Some(first), Some(last)) = (events.first(), events.last()) {
        let mid_duration = last.event_time.duration_since(first.event_time).unwrap_or_default() / 2;
        let mid_time = first.event_time + mid_duration;
        let _range_events: Vec<_> = events.iter()
            .filter(|e| e.event_time >= mid_time)
            .collect();
    }
    let temporal_time = start.elapsed();
    results.query_performance.insert("Temporal Range".to_string(), temporal_time);
    println!("   Temporal Range Query: {:?}", temporal_time);
    
    Ok(())
}

/// Benchmark reasoning performance
fn benchmark_reasoning_performance(ontology: &Ontology, events: &[EPCISEvent], results: &mut BenchmarkResults) -> OwlResult<()> {
    let reasoner = SimpleReasoner::new(ontology.clone());
    
    // Test 1: Consistency Checking
    let start = Instant::now();
    let _is_consistent = reasoner.is_consistent()?;
    let consistency_time = start.elapsed();
    results.reasoning_performance.insert("Consistency Check".to_string(), consistency_time);
    println!("   Consistency Check: {:?}", consistency_time);
    
    // Test 2: Class Hierarchy Reasoning
    let start = Instant::now();
    let object_event_iri = IRI::new("http://ns.gs1.org/epcis/ObjectEvent")?;
    let event_iri = IRI::new("http://ns.gs1.org/epcis/Event")?;
    let _is_subclass = reasoner.is_subclass_of(&object_event_iri, &event_iri)?;
    let hierarchy_time = start.elapsed();
    results.reasoning_performance.insert("Hierarchy Reasoning".to_string(), hierarchy_time);
    println!("   Hierarchy Reasoning: {:?}", hierarchy_time);
    
    // Test 3: Multiple Query Performance
    let start = Instant::now();
    let test_iris = vec![
        "http://ns.gs1.org/epcis/ObjectEvent",
        "http://ns.gs1.org/epcis/AggregationEvent",
        "http://ns.gs1.org/epcis/TransactionEvent",
    ];
    
    for iri_str in test_iris {
        let iri = IRI::new(iri_str)?;
        let _ = reasoner.is_subclass_of(&iri, &event_iri)?;
    }
    let multi_query_time = start.elapsed();
    results.reasoning_performance.insert("Multiple Queries".to_string(), multi_query_time);
    println!("   Multiple Queries (3): {:?}", multi_query_time);
    
    // Test 4: Complex Query Performance
    let start = Instant::now();
    
    // Simulate complex traceability reasoning
    let unique_epcs: std::collections::HashSet<_> = events.iter()
        .flat_map(|e| e.epc_list.iter())
        .take(50) // Sample for performance
        .cloned()
        .collect();
    
    let mut traceable_epcs = 0;
    for epc in &unique_epcs {
        let epc_events: Vec<_> = events.iter()
            .filter(|e| e.epc_list.contains(epc))
            .collect();
        
        if epc_events.len() >= 2 { // Minimum for traceability
            traceable_epcs += 1;
        }
    }
    let complex_time = start.elapsed();
    results.reasoning_performance.insert("Complex Traceability".to_string(), complex_time);
    println!("   Complex Traceability ({} EPCs, {} traceable): {:?}", 
        unique_epcs.len(), traceable_epcs, complex_time);
    
    Ok(())
}

/// Benchmark scalability metrics
fn benchmark_scalability(events: &[EPCISEvent], results: &mut BenchmarkResults) {
    let event_count = events.len();
    
    // Calculate scalability metrics
    results.scalability_metrics.events_per_second = 
        event_count as f64 / results.generation_time.as_secs_f64();
    
    // Memory efficiency
    results.scalability_metrics.memory_per_event = 
        (results.memory_usage as f64 * 1024.0 * 1024.0) / event_count as f64;
    
    // Query efficiency (average of all query times)
    let avg_query_time: f64 = results.query_performance.values()
        .map(|d| d.as_secs_f64())
        .sum::<f64>() / results.query_performance.len().max(1) as f64;
    results.scalability_metrics.queries_per_second = 1.0 / avg_query_time;
    
    // Reasoning efficiency
    let avg_reasoning_time: f64 = results.reasoning_performance.values()
        .map(|d| d.as_secs_f64())
        .sum::<f64>() / results.reasoning_performance.len().max(1) as f64;
    results.scalability_metrics.reasoning_ops_per_second = 1.0 / avg_reasoning_time;
    
    println!("   Events/Second: {:.0}", results.scalability_metrics.events_per_second);
    println!("   Memory/Event: {:.0} bytes", results.scalability_metrics.memory_per_event);
    println!("   Queries/Second: {:.0}", results.scalability_metrics.queries_per_second);
    println!("   Reasoning Ops/Second: {:.0}", results.scalability_metrics.reasoning_ops_per_second);
}

/// Generate comparative analysis
fn generate_comparative_analysis(results: &[BenchmarkResults]) {
    println!("Scalability Analysis:");
    
    for result in results {
        println!("\n{} ({} events):", result.scale_name, result.event_count);
        println!("  Generation Time: {:?}", result.generation_time);
        println!("  Memory Usage: {} MB", result.memory_usage);
        println!("  Events/Second: {:.0}", result.scalability_metrics.events_per_second);
        
        println!("  Query Performance:");
        for (query_type, time) in &result.query_performance {
            println!("    {}: {:?}", query_type, time);
        }
        
        println!("  Reasoning Performance:");
        for (reasoning_type, time) in &result.reasoning_performance {
            println!("    {}: {:?}", reasoning_type, time);
        }
    }
    
    // Performance scaling analysis
    if results.len() >= 2 {
        println!("\nPerformance Scaling Analysis:");
        
        let small = &results[0];
        let medium = &results[1];
        let large = &results[2];
        
        // Generation time scaling
        let small_to_medium_ratio = medium.event_count as f64 / small.event_count as f64;
        let medium_to_large_ratio = large.event_count as f64 / medium.event_count as f64;
        
        let gen_time_small_to_medium = medium.generation_time.as_secs_f64() / small.generation_time.as_secs_f64();
        let gen_time_medium_to_large = large.generation_time.as_secs_f64() / medium.generation_time.as_secs_f64();
        
        println!("  Generation Time Scaling:");
        println!("    Small→Medium: {:.1}x data, {:.1}x time (efficiency: {:.1}%)", 
            small_to_medium_ratio, gen_time_small_to_medium,
            (small_to_medium_ratio / gen_time_small_to_medium) * 100.0);
        println!("    Medium→Large: {:.1}x data, {:.1}x time (efficiency: {:.1}%)", 
            medium_to_large_ratio, gen_time_medium_to_large,
            (medium_to_large_ratio / gen_time_medium_to_large) * 100.0);
        
        // Memory scaling
        let mem_small_to_medium = medium.memory_usage as f64 / small.memory_usage as f64;
        let mem_medium_to_large = large.memory_usage as f64 / medium.memory_usage as f64;
        
        println!("  Memory Scaling:");
        println!("    Small→Medium: {:.1}x data, {:.1}x memory", 
            small_to_medium_ratio, mem_small_to_medium);
        println!("    Medium→Large: {:.1}x data, {:.1}x memory", 
            medium_to_large_ratio, mem_medium_to_large);
    }
}

/// Generate performance recommendations
fn generate_performance_recommendations(results: &[BenchmarkResults]) {
    println!("Based on benchmark results, here are the performance recommendations:");
    
    // Analyze generation performance
    let avg_events_per_sec: f64 = results.iter()
        .map(|r| r.scalability_metrics.events_per_second)
        .sum::<f64>() / results.len() as f64;
    
    if avg_events_per_sec < 50000.0 {
        println!("⚠️  Data Generation:");
        println!("   - Consider optimizing event creation algorithms");
        println!("   - Implement batch processing for large datasets");
        println!("   - Use memory pooling for event objects");
    } else {
        println!("✅ Data Generation:");
        println!("   - Generation performance is excellent");
        println!("   - Current implementation scales well");
    }
    
    // Analyze memory usage
    let avg_memory_per_event: f64 = results.iter()
        .map(|r| r.scalability_metrics.memory_per_event)
        .sum::<f64>() / results.len() as f64;
    
    if avg_memory_per_event > 1000.0 {
        println!("⚠️  Memory Usage:");
        println!("   - Consider string interning for EPC identifiers");
        println!("   - Implement more compact data structures");
        println!("   - Use lazy loading for large event collections");
    } else {
        println!("✅ Memory Usage:");
        println!("   - Memory efficiency is good");
        println!("   - Current data structures are well-optimized");
    }
    
    // Analyze query performance
    let avg_queries_per_sec: f64 = results.iter()
        .map(|r| r.scalability_metrics.queries_per_second)
        .sum::<f64>() / results.len() as f64;
    
    if avg_queries_per_sec < 1000.0 {
        println!("⚠️  Query Performance:");
        println!("   - Implement indexing for frequently queried fields");
        println!("   - Consider query caching mechanisms");
        println!("   - Optimize filter predicates for better performance");
    } else {
        println!("✅ Query Performance:");
        println!("   - Query performance is satisfactory");
        println!("   - Current filtering mechanisms are efficient");
    }
    
    // Analyze reasoning performance
    let avg_reasoning_ops: f64 = results.iter()
        .map(|r| r.scalability_metrics.reasoning_ops_per_second)
        .sum::<f64>() / results.len() as f64;
    
    if avg_reasoning_ops < 100.0 {
        println!("⚠️  Reasoning Performance:");
        println!("   - Consider implementing incremental reasoning");
        println!("   - Optimize axiom indexing strategies");
        println!("   - Implement result caching for repeated queries");
    } else {
        println!("✅ Reasoning Performance:");
        println!("   - Reasoning engine performs well");
        println!("   - Current implementation is suitable for production");
    }
    
    println!("\n🔧 General Recommendations:");
    println!("   - Monitor performance metrics in production");
    println!("   - Implement automated performance regression testing");
    println!("   - Consider horizontal scaling for very large datasets");
    println!("   - Profile memory usage for optimization opportunities");
}

/// Benchmark results structure
#[derive(Debug)]
struct BenchmarkResults {
    scale_name: String,
    event_count: usize,
    generation_time: std::time::Duration,
    memory_usage: usize,
    query_performance: std::collections::HashMap<String, std::time::Duration>,
    reasoning_performance: std::collections::HashMap<String, std::time::Duration>,
    scalability_metrics: ScalabilityMetrics,
}

/// Scalability metrics
#[derive(Debug, Default)]
struct ScalabilityMetrics {
    events_per_second: f64,
    memory_per_event: f64,
    queries_per_second: f64,
    reasoning_ops_per_second: f64,
}