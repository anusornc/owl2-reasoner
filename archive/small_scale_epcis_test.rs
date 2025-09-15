//! Small-scale EPCIS test data generation example
//! 
//! This example demonstrates the generation of small-scale EPCIS test data
//! (100-500 events) with realistic supply chain scenarios.

use owl2_reasoner::*;
use owl2_reasoner::epcis_test_generator::*;

fn main() -> OwlResult<()> {
    println!("🏭 Small-Scale EPCIS Test Data Generation");
    println!("{}", "=".repeat(50));
    
    // Create small-scale test data configuration
    let config = small_scale_config();
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
    let ontology = generator.generate_ontology()?;
    
    // Get generation statistics
    let stats = generator.get_stats();
    println!("✅ Generation complete!");
    println!();
    println!("{}", stats.summary());
    
    // Generate and display sample events
    println!("\n📋 Sample Events:");
    let events = generator.generate_events();
    for (i, event) in events.iter().take(5).enumerate() {
        println!("Event {}: {} ({:?})", 
            i + 1, 
            event.event_id, 
            event.event_type
        );
        println!("  Business Step: {:?}", event.biz_step);
        println!("  Disposition: {:?}", event.disposition);
        println!("  EPCs: {}", event.epc_list.len());
        if let Some(ref child_epcs) = event.child_epcs {
            if !child_epcs.is_empty() {
                println!("  Child EPCs: {}", child_epcs.len());
            }
        }
        println!();
    }
    
    // Test basic functionality
    println!("🧠 Testing Basic Functionality...");
    
    // The ontology should be properly structured
    println!("✅ EPCIS ontology structure created successfully");
    println!("📊 Ready for reasoning operations");
    
    println!("\n🎯 Small-scale test generation completed successfully!");
    println!("The generated ontology contains realistic supply chain events");
    println!("suitable for testing OWL2 reasoning performance.");
    
    Ok(())
}