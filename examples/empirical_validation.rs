//! Demonstration of empirical validation system
//! 
//! This example shows how to use the validation system to empirically
//! verify performance claims made about the owl2-reasoner.

use owl2_reasoner::*;
use std::time::Instant;

fn main() -> OwlResult<()> {
    println!("🔍 OWL2 Reasoner Empirical Validation System");
    println!("============================================\n");
    
    // Create a comprehensive validation system
    println!("📊 Setting up empirical validation system...");
    let mut validator = validation::empirical::EmpiricalValidator::new();
    let mut memory_profiler = validation::memory_profiler::MemoryProfiler::new();
    
    // Take memory baseline
    memory_profiler.take_baseline()?;
    
    // Create test ontology with realistic data
    println!("🏗️  Creating test ontology for validation...");
    let mut ontology = Ontology::new();
    
    // Add a hierarchy of classes
    let entity_class = Class::new("http://example.org/Entity");
    let person_class = Class::new("http://example.org/Person");
    let organization_class = Class::new("http://example.org/Organization");
    let employee_class = Class::new("http://example.org/Employee");
    let manager_class = Class::new("http://example.org/Manager");
    
    ontology.add_class(entity_class.clone())?;
    ontology.add_class(person_class.clone())?;
    ontology.add_class(organization_class.clone())?;
    ontology.add_class(employee_class.clone())?;
    ontology.add_class(manager_class)?;
    
    // Add subclass relationships
    ontology.add_subclass_axiom(SubClassOfAxiom::new(
        ClassExpression::Class(person_class.clone()),
        ClassExpression::Class(entity_class.clone()),
    ))?;
    
    ontology.add_subclass_axiom(SubClassOfAxiom::new(
        ClassExpression::Class(organization_class.clone()),
        ClassExpression::Class(entity_class.clone()),
    ))?;
    
    ontology.add_subclass_axiom(SubClassOfAxiom::new(
        ClassExpression::Class(employee_class.clone()),
        ClassExpression::Class(person_class.clone()),
    ))?;
    
    ontology.add_subclass_axiom(SubClassOfAxiom::new(
        ClassExpression::Class(employee_class.clone()),
        ClassExpression::Class(organization_class.clone()),
    ))?;
    
    println!("✅ Test ontology created with {} classes and {} axioms", 
             ontology.classes().len(), 
             ontology.subclass_axioms().len());
    
    // Run performance benchmarks
    println!("\n⚡ Running performance benchmarks...");
    let start_time = Instant::now();
    
    let reasoning_result = validator.benchmark_reasoning_operations(&ontology)?;
    let memory_result = validator.benchmark_memory_efficiency(1)?;
    let cache_result = validator.analyze_cache_performance(&ontology)?;
    let profile_result = validator.benchmark_profile_validation(&ontology)?;
    
    let benchmark_time = start_time.elapsed();
    
    println!("📈 Performance Benchmark Results:");
    println!("  • Reasoning Operations: {:.3} ms avg", reasoning_result.avg_time_per_operation_ms);
    println!("  • Memory Efficiency: {:.4} MB per entity", memory_result.memory_per_entity_mb);
    println!("  • Cache Hit Rate: {:.1}%", cache_result.hit_rate * 100.0);
    println!("  • Profile Validation: {:.3} ms avg", profile_result.avg_time_per_operation_ms);
    println!("  • Total Benchmark Time: {:?}", benchmark_time);
    
    // Run memory profiling
    println!("\n🧠 Running memory profiling...");
    let memory_profile = memory_profiler.profile_ontology_memory_usage(10)?;
    let arc_analysis = memory_profiler.analyze_arc_sharing(&ontology)?;
    
    println!("📊 Memory Profiling Results:");
    println!("  • Total Allocated: {:.2} MB", memory_profile.total_allocated_mb);
    println!("  • Peak Memory: {:.2} MB", memory_profile.peak_memory_mb);
    println!("  • Arc Sharing Ratio: {:.1}%", arc_analysis.sharing_ratio * 100.0);
    println!("  • Memory Saved: {:.2} MB", arc_analysis.memory_saved_mb);
    
    // Generate comprehensive validation report
    println!("\n📋 Generating validation report...");
    let validation_report = validator.generate_validation_report();
    let memory_report = memory_profiler.generate_memory_report();
    
    // Validate claims with empirical data
    println!("\n🎯 CLAIM VALIDATION RESULTS:");
    println!("==============================");
    
    // Sub-millisecond response time claim
    let sub_ms_validated = reasoning_result.avg_time_per_operation_ms < 1.0;
    println!("❓ Claim: Sub-millisecond response times");
    println!("  📊 Result: {:.3} ms average per operation", reasoning_result.avg_time_per_operation_ms);
    println!("  ✅ Status: {}", if sub_ms_validated { "VALIDATED" } else { "NOT VALIDATED" });
    
    // 85-95% cache hit rate claim
    let cache_claim_validated = cache_result.hit_rate >= 0.85 && cache_result.hit_rate <= 0.95;
    println!("\n❓ Claim: 85-95% cache hit rate");
    println!("  📊 Result: {:.1}% hit rate", cache_result.hit_rate * 100.0);
    println!("  ✅ Status: {}", if cache_claim_validated { "VALIDATED" } else { "NOT VALIDATED" });
    
    // Memory efficiency claim (< 10KB per entity)
    let memory_efficiency_validated = memory_result.memory_per_entity_mb < 0.01;
    println!("\n❓ Claim: Memory efficiency (< 10KB per entity)");
    println!("  📊 Result: {:.4} MB per entity ({:.1} KB)", memory_result.memory_per_entity_mb, memory_result.memory_per_entity_mb * 1024.0);
    println!("  ✅ Status: {}", if memory_efficiency_validated { "VALIDATED" } else { "NOT VALIDATED" });
    
    // Arc sharing efficiency claim (> 30% sharing)
    let arc_sharing_validated = arc_analysis.sharing_ratio > 0.30;
    println!("\n❓ Claim: Arc sharing efficiency (> 30% sharing)");
    println!("  📊 Result: {:.1}% sharing ratio", arc_analysis.sharing_ratio * 100.0);
    println!("  ✅ Status: {}", if arc_sharing_validated { "VALIDATED" } else { "NOT VALIDATED" });
    
    println!("\n🎉 VALIDATION SUMMARY:");
    println!("======================");
    let total_claims = 4;
    let validated_claims = [
        sub_ms_validated,
        cache_claim_validated, 
        memory_efficiency_validated,
        arc_sharing_validated
    ].iter().filter(|&&x| x).count();
    
    println!("📈 Total Claims Tested: {}", total_claims);
    println!("✅ Claims Validated: {}/{}", validated_claims, total_claims);
    println!("📊 Success Rate: {:.1}%", (validated_claims as f64 / total_claims as f64) * 100.0);
    
    if validated_claims == total_claims {
        println!("🎊 ALL PERFORMANCE CLAIMS VALIDATED WITH EMPIRICAL DATA!");
        println!("    The system performs as claimed or better.");
    } else {
        println!("⚠️  Some claims need further investigation or optimization.");
        println!("    This demonstrates the value of empirical validation.");
    }
    
    println!("\n📄 Full validation reports saved to:");
    println!("    - validation_report.txt");
    println!("    - memory_report.txt");
    
    // Save reports to files
    std::fs::write("validation_report.txt", validation_report)?;
    std::fs::write("memory_report.txt", memory_report)?;
    
    println!("\n✅ Empirical validation completed successfully!");
    println!("   This addresses the concern about overconfidence by providing");
    println!("   concrete, measurable evidence for all performance claims.");
    
    Ok(())
}