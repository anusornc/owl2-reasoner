//! Consistency Benchmark for External Testing
//!
//! This example provides a simple consistency checking benchmark
//! for external comparison with established reasoners.

use owl2_reasoner::{
    parser::{OntologyParser, ParserFactory},
    reasoning::{OwlReasoner, Reasoner},
    OwlResult,
};
use std::env;
use std::path::Path;
use std::time::Instant;

fn main() -> OwlResult<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run --example consistency_benchmark -- <ontology_file>");
        eprintln!(
            "Example: cargo run --example consistency_benchmark -- test_suite/family_test.ttl"
        );
        std::process::exit(1);
    }

    let ontology_file = &args[1];

    // Check if file exists
    if !Path::new(ontology_file).exists() {
        eprintln!("❌ Error: Ontology file not found: {}", ontology_file);
        std::process::exit(1);
    }

    // Parse the ontology
    let start_time = Instant::now();
    let content = std::fs::read_to_string(ontology_file)?;

    let parser = ParserFactory::auto_detect(&content).ok_or_else(|| {
        owl2_reasoner::OwlError::ParseError("Could not detect parser".to_string())
    })?;
    let ontology = parser.parse_str(&content)?;
    let parse_time = start_time.elapsed();

    // Create reasoner and check consistency
    let mut reasoner = OwlReasoner::new(ontology);
    let consistency_start = Instant::now();
    let is_consistent = reasoner.is_consistent()?;
    let consistency_time = consistency_start.elapsed();

    // Output results in a format easy to parse
    println!("🧪 OWL2-Reasoner Consistency Check");
    println!("====================================");
    println!("File: {}", ontology_file);
    println!("Parse Time: {:?}", parse_time);
    println!("Consistency Check Time: {:?}", consistency_time);
    println!("Total Time: {:?}", parse_time + consistency_time);

    if is_consistent {
        println!("✅ Consistent");
    } else {
        println!("❌ Inconsistent");
    }

    // Additional performance metrics
    println!("Performance Metrics:");
    println!("  - Classes: {}", reasoner.ontology().classes().len());
    println!(
        "  - Object Properties: {}",
        reasoner.ontology().object_properties().len()
    );
    println!(
        "  - Data Properties: {}",
        reasoner.ontology().data_properties().len()
    );
    println!(
        "  - Individuals: {}",
        reasoner.ontology().named_individuals().len()
    );
    println!("  - Axioms: {}", reasoner.ontology().axioms().len());

    Ok(())
}
