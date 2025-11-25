//! Command Line Interface for OWL2 Reasoner Benchmarking
//!
//! Provides a simple CLI for consistency checking and classification
//! that can be used alongside other OWL2 reasoners in benchmarks

use owl2_reasoner::parser::ParserFactory;
use owl2_reasoner::reasoning::SimpleReasoner;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <operation> <ontology_file>", args[0]);
        eprintln!("Operations:");
        eprintln!("  --classify    Perform classification");
        eprintln!("  --consistent  Check consistency");
        std::process::exit(1);
    }

    let operation = &args[1];
    let ontology_file = &args[2];

    // Validate operation
    if operation != "--classify" && operation != "--consistent" {
        eprintln!("Error: Invalid operation '{}'", operation);
        eprintln!("Valid operations: --classify, --consistent");
        std::process::exit(1);
    }

    // Check if file exists
    let path = Path::new(ontology_file);
    if !path.exists() {
        eprintln!("Error: Ontology file '{}' does not exist", ontology_file);
        std::process::exit(1);
    }

    // Get file extension to determine parser
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("ttl");

    // Create appropriate parser
    let parser = ParserFactory::for_file_extension(extension)
        .ok_or_else(|| format!("Unsupported file extension: {}", extension))?;

    println!("Loading ontology from {}...", ontology_file);
    let start_time = Instant::now();

    // Parse ontology
    let ontology = parser
        .parse_file(path)
        .map_err(|e| format!("Failed to parse ontology: {}", e))?;

    let parse_time = start_time.elapsed();
    println!(
        "Parsed ontology with {} axioms in {:?}",
        ontology.axiom_count(),
        parse_time
    );

    // Create reasoner
    let reasoner = SimpleReasoner::new(ontology);

    // Perform requested operation
    let start_time = Instant::now();

    match operation.as_str() {
        "--classify" => {
            println!("Performing classification (subclass reasoning)...");
            // Get all classes and perform subclass reasoning
            let classes: Vec<_> = reasoner.ontology.classes().iter().collect();
            let mut subclass_count = 0;

            for class in &classes {
                for other_class in &classes {
                    if class != other_class {
                        if let Ok(is_subclass) =
                            reasoner.is_subclass_of(class.iri(), other_class.iri())
                        {
                            if is_subclass {
                                subclass_count += 1;
                            }
                        }
                    }
                }
            }

            let reasoning_time = start_time.elapsed();

            println!("Classification completed in {:?}", reasoning_time);
            println!(
                "Processed {} classes, found {} subclass relationships",
                classes.len(),
                subclass_count
            );
        }
        "--consistent" => {
            println!("Checking consistency...");
            let is_consistent = reasoner
                .is_consistent()
                .map_err(|e| format!("Consistency checking failed: {}", e))?;
            let reasoning_time = start_time.elapsed();

            println!("Consistency check completed in {:?}", reasoning_time);
            println!(
                "Ontology is {}consistent",
                if is_consistent { "" } else { "in" }
            );
        }
        _ => unreachable!(),
    }

    Ok(())
}
