// Basic parser tests
use owl2_reasoner::*;

#[test]
fn test_turtle_parser_creation() {
    let parser = parser::TurtleParser::new();

    // Test creation succeeded (parser is not a Result type)
    assert!(true); // If we get here, parser creation succeeded
}

#[test]
fn test_rdf_xml_parser_creation() {
    let parser = parser::RdfXmlParser::new();

    // Test creation succeeded (parser is not a Result type)
    assert!(true); // If we get here, parser creation succeeded
}

#[test]
fn test_owl_xml_parser_creation() {
    let parser = parser::OwlXmlParser::new();

    // Test creation succeeded (parser is not a Result type)
    assert!(true); // If we get here, parser creation succeeded
}
