use owl2_reasoner::{OwlFunctionalSyntaxParser, OntologyParser, ParserConfig};
use std::fs;

fn main() {
    // Test the actual file
    let test_content = fs::read_to_string("benchmarking/established_reasoners/test_simple.owl")
        .expect("Failed to read file");

    println!("=== Testing actual file ===");
    println!("File content:\n{}", test_content);
Prefix(:=<http://example.org/test#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)

Ontology(<http://example.org/test>

Declaration(Class(:Person))
Declaration(Class(:Student))
Declaration(NamedIndividual(:John))

SubClassOf(:Student :Person)
ClassAssertion(:Student :John)

)
"#;

    let config = ParserConfig::default();
    let parser = OwlFunctionalSyntaxParser::with_config(config);
    match parser.parse_str(test_content) {
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