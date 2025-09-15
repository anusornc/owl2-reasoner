use std::path::Path;
use owl2_reasoner::parser::ParserFactory;

fn main() {
    let ontology_file = "benchmarking/established_reasoners/test_simple.owl";
    let path = Path::new(ontology_file);

    // Get file extension to determine parser (exactly like benchmark CLI)
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("ttl");

    // Create appropriate parser (exactly like benchmark CLI)
    let parser = ParserFactory::for_file_extension(extension)
        .expect("Failed to create parser");

    println!("Using parser: {}", parser.format_name());
    println!("Loading ontology from {}...", ontology_file);

    // Parse ontology (exactly like benchmark CLI)
    match parser.parse_file(path) {
        Ok(ontology) => {
            println!("Successfully parsed ontology!");
            println!("Classes: {}", ontology.classes().len());
            println!("Individuals: {}", ontology.named_individuals().len());
            println!("Axioms: {}", ontology.axiom_count());
            println!("Object properties: {}", ontology.object_properties().len());
            println!("Data properties: {}", ontology.data_properties().len());
            println!("Imports: {}", ontology.imports().len());

            if ontology.classes().is_empty() && ontology.object_properties().is_empty()
                && ontology.data_properties().is_empty() && ontology.named_individuals().is_empty()
                && ontology.imports().is_empty() {
                println!("VALIDATION WOULD FAIL: Ontology contains no entities or imports");
            } else {
                println!("VALIDATION WOULD PASS: Ontology has entities or imports");
            }
        }
        Err(e) => {
            println!("Failed to parse: {:?}", e);
        }
    }
}