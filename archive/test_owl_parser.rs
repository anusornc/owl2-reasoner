use owl2_reasoner::{OwlFunctionalSyntaxParser, OntologyParser};

fn main() {
    let test_content = r#"
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

    let parser = OwlFunctionalSyntaxParser::new();
    match parser.parse_str(test_content) {
        Ok(ontology) => {
            println!("Successfully parsed ontology!");
            println!("Classes: {}", ontology.classes().len());
            println!("Individuals: {}", ontology.named_individuals().len());
            println!("Axioms: {}", ontology.axiom_count());
        }
        Err(e) => {
            println!("Failed to parse: {:?}", e);
        }
    }
}