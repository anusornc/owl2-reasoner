//! Medium-scale EPCIS test data generation example
//! 
//! This example demonstrates the generation of medium-scale EPCIS test data
//! (1K-5K events) with realistic supply chain scenarios and performance testing.

use owl2_reasoner::*;
use owl2_reasoner::epcis_test_generator::*;

fn main() -> OwlResult<()> {
    println!("🏭 Medium-Scale EPCIS Test Data Generation");
    println!("{}", "=".repeat(50));
    
    // Create medium-scale test data configuration
    let config = medium_scale_config();
    println!("Configuration:");
    println!("- Event Count: {}", config.event_count);
    println!("- Scale: {:?}", config.scale);
    println!("- Include Complex Scenarios: {}", config.include_complex_scenarios);
    println!("- Seed: {:?}", config.seed);
    println!();
    
    // Initialize test data generator
    let mut generator = EPCISTestDataGenerator::new(config);
    
    // Generate ontology with events
    println!("📊 Generating test ontology...");
    let start_time = std::time::Instant::now();
    let ontology = generator.generate_ontology()?;
    let generation_time = start_time.elapsed();
    
    // Get generation statistics
    let stats = generator.get_stats();
    println!("✅ Generation complete!");
    println!("⏱️  Generation time: {:?}", generation_time);
    println!();
    println!("{}", stats.summary());
    
    // Generate events for performance testing
    println!("📋 Generating events for performance testing...");
    let events_start = std::time::Instant::now();
    let events = generator.generate_events();
    let events_time = events_start.elapsed();
    
    println!("✅ Generated {} events in {:?}", events.len(), events_time);
    println!("📊 Event generation rate: {:.0} events/second", 
        events.len() as f64 / events_time.as_secs_f64());
    
    // Display sample events
    println!("\n📋 Sample Events (first 10):");
    for (i, event) in events.iter().take(10).enumerate() {
        println!("Event {}: {} ({:?})", 
            i + 1, 
            event.event_id, 
            event.event_type
        );
        println!("  Business Step: {:?}, Disposition: {:?}", 
            event.biz_step, event.disposition);
        println!("  EPCs: {}, Child EPCs: {:?}", 
            event.epc_list.len(),
            event.child_epcs.as_ref().map(|children| children.len())
        );
        println!();
    }
    
    // Test reasoning performance
    println!("🧠 Testing Reasoning Performance...");
    
    // Test basic consistency checking
    let consistency_start = std::time::Instant::now();
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent()?;
    let consistency_time = consistency_start.elapsed();
    
    println!("✅ Consistency check: {} ({:?})", is_consistent, consistency_time);
    
    // Test class hierarchy reasoning
    println!("🔍 Testing class hierarchy reasoning...");
    let hierarchy_start = std::time::Instant::now();
    
    // Test some basic subclass relationships
    let object_event_iri = IRI::new("http://ns.gs1.org/epcis/ObjectEvent")?;
    let event_iri = IRI::new("http://ns.gs1.org/epcis/Event")?;
    let is_subclass = reasoner.is_subclass_of(&object_event_iri, &event_iri)?;
    
    let hierarchy_time = hierarchy_start.elapsed();
    println!("✅ Subclass reasoning: {} ({:?})", is_subclass, hierarchy_time);
    
    // Performance summary
    println!("\n📊 Performance Summary:");
    println!("Ontology Generation: {:?}", generation_time);
    println!("Event Generation: {:?} ({:.0} events/sec)", 
        events_time, events.len() as f64 / events_time.as_secs_f64());
    println!("Consistency Check: {:?}", consistency_time);
    println!("Hierarchy Reasoning: {:?}", hierarchy_time);
    
    // Memory usage estimation
    println!("\n💾 Memory Usage Estimate:");
    println!("Total Events: {}", events.len());
    let avg_epc_count = events.iter().map(|e| e.epc_list.len()).sum::<usize>() / events.len().max(1);
    println!("Average Event Size: ~{} bytes", 
        std::mem::size_of::<EPCISEvent>() + avg_epc_count * 24); // Rough estimate
    println!("Estimated Total Memory: ~{} MB", 
        (events.len() * (std::mem::size_of::<EPCISEvent>() + 100)) / (1024 * 1024));
    
    println!("\n🎯 Medium-scale test generation completed successfully!");
    println!("The generated ontology is suitable for performance benchmarking");
    println!("and stress testing of OWL2 reasoning capabilities.");
    
    Ok(())
}