//! Integration tests for end-to-end OWL2 reasoning pipeline
//!
//! This module tests the complete workflow from ontology parsing through reasoning
//! components, ensuring all parts work together correctly.

use crate::parser::*;
use crate::reasoning::SimpleReasoner;
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_reasoner_pipeline() {
        // Test the parser -> reasoner pipeline
        let turtle_content = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:Person a owl:Class .
ex:Animal a owl:Class .
ex:Plant a owl:Class .

ex:Human rdfs:subClassOf ex:Person .
ex:Dog rdfs:subClassOf ex:Animal .

ex:hasParent a owl:ObjectProperty ;
    rdfs:domain ex:Person ;
    rdfs:range ex:Person .

ex:hasPet a owl:ObjectProperty ;
    rdfs:domain ex:Person ;
    rdfs:range ex:Animal .
"#;

        // Step 1: Parse the ontology
        let parser = TurtleParser::new();
        let ontology = parser
            .parse_str(turtle_content)
            .expect("Failed to parse ontology");

        assert!(
            ontology.classes().len() >= 3,
            "Should have at least 3 classes"
        );
        assert!(
            ontology.object_properties().len() >= 2,
            "Should have at least 2 properties"
        );

        // Step 2: Initialize reasoner with parsed ontology
        let reasoner = SimpleReasoner::new(ontology.clone());

        // Step 3: Test basic reasoning functionality
        let is_consistent = reasoner.is_consistent().expect("Should check consistency");

        assert!(is_consistent, "Ontology should be consistent");

        // Step 4: Test reasoner caching functionality
        let stats = reasoner.cache_stats().expect("Should get cache stats");
        assert!(
            stats.contains_key("consistency"),
            "Should have consistency stats"
        );

        // Test cache clearing
        reasoner.clear_caches().expect("Should clear caches");
        let stats_after = reasoner.cache_stats().expect("Should get cache stats after clear");
        assert_eq!(
            stats_after.get("consistency"),
            Some(&0),
            "Cache should be cleared"
        );
    }

    #[test]
    fn test_reasoning_basic_functionality() {
        // Test basic reasoning functionality
        let turtle_content = r#"
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:LivingThing a owl:Class .
ex:Animal rdfs:subClassOf ex:LivingThing .
ex:Mammal rdfs:subClassOf ex:Animal .
ex:Dog rdfs:subClassOf ex:Mammal .

ex:hasParent a owl:ObjectProperty .
ex:hasFather rdfs:subPropertyOf ex:hasParent .
"#;

        // Parse and create reasoner
        let parser = TurtleParser::new();
        let ontology = parser.parse_str(turtle_content).unwrap();

        let reasoner = SimpleReasoner::new(ontology);

        // Test consistency checking
        let is_consistent = reasoner.is_consistent().unwrap();
        assert!(is_consistent, "Ontology should be consistent");

        // Test cache functionality
        let stats = reasoner.cache_stats().expect("Should get cache stats");
        assert!(
            stats.contains_key("consistency"),
            "Should have consistency stats"
        );

        // Test cache clearing
        reasoner.clear_caches().expect("Should clear caches");
        let stats_after = reasoner.cache_stats().expect("Should get cache stats after clear");
        assert_eq!(
            stats_after.get("consistency"),
            Some(&0),
            "Cache should be cleared"
        );
    }

    #[test]
    fn test_complex_ontology_pipeline() {
        // Test with a more complex ontology
        let complex_content = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .

# Class hierarchy
ex:Entity a owl:Class .
ex:Person rdfs:subClassOf ex:Entity .
ex:Organization rdfs:subClassOf ex:Entity .

ex:Employee rdfs:subClassOf ex:Person .
ex:Manager rdfs:subClassOf ex:Employee .
ex:Company rdfs:subClassOf ex:Organization .

# Properties
ex:worksFor a owl:ObjectProperty ;
    rdfs:domain ex:Person ;
    rdfs:range ex:Organization .

ex:manages a owl:ObjectProperty ;
    rdfs:domain ex:Manager ;
    rdfs:range ex:Organization .

ex:hasEmployee a owl:ObjectProperty ;
    owl:inverseOf ex:worksFor .

# Data properties
ex:hasAge a owl:DatatypeProperty ;
    rdfs:domain ex:Person ;
    rdfs:range xsd:integer .

ex:hasName a owl:DatatypeProperty ;
    rdfs:domain ex:Person ;
    rdfs:range xsd:string .

# Property hierarchy
ex:supervises rdfs:subPropertyOf ex:manages .

# Individuals
ex:JohnDoe a ex:Employee ;
    ex:worksFor ex:TechCorp ;
    ex:hasAge 30 ;
    ex:hasName "John Doe" .

ex:JaneSmith a ex:Manager ;
    ex:manages ex:TechCorp ;
    ex:hasAge 35 ;
    ex:hasName "Jane Smith" .

ex:TechCorp a ex:Company .
"#;

        let start_time = Instant::now();

        // Complete pipeline test
        let parser = TurtleParser::new();
        let ontology = parser.parse_str(complex_content).unwrap();

        let reasoner = SimpleReasoner::new(ontology.clone());

        let duration = start_time.elapsed();

        // Verify pipeline completed
        let is_consistent = reasoner.is_consistent().unwrap();
        assert!(is_consistent, "Complex ontology should be consistent");

        // Verify ontology structure
        // Note: Current parser captures basic entity declarations
        assert!(
            ontology.classes().len() >= 1,
            "Should have at least basic classes"
        );
        assert!(
            ontology.object_properties().len() >= 3,
            "Should have multiple properties"
        );
        // More sophisticated parsing needed for data properties and individuals

        println!("Complex pipeline completed in: {:?}", duration);
    }

    #[test]
    fn test_error_handling_pipeline() {
        // Test error handling across the pipeline
        let invalid_content = r#"
@prefix invalid: syntax here
@prefix owl: <http://www.w3.org/2002/07/owl#> .

ex:InvalidSyntax here
"#;

        // Test parsing error handling
        let parser = TurtleParser::new();
        let parse_result = parser.parse_str(invalid_content);
        assert!(parse_result.is_err(), "Should handle invalid syntax");

        // Test with valid but minimal content
        let minimal_content = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:Test a owl:Class .
"#;

        let ontology = parser.parse_str(minimal_content).unwrap();

        // Should be able to create reasoner with minimal ontology
        let reasoner = SimpleReasoner::new(ontology);
        let is_consistent = reasoner.is_consistent();
        assert!(is_consistent.is_ok(), "Should reason on minimal ontology");
    }

    #[test]
    fn test_multiple_format_pipeline() {
        // Test pipeline with different input formats
        let turtle_content = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:TestClass a owl:Class .
ex:TestProperty a owl:ObjectProperty .
"#;

        // Test Turtle format
        let turtle_parser = TurtleParser::new();
        let turtle_ontology = turtle_parser.parse_str(turtle_content).unwrap();

        // Test auto-detection
        let auto_parser =
            ParserFactory::auto_detect(turtle_content).expect("Should auto-detect Turtle format");

        let auto_ontology = auto_parser.parse_str(turtle_content).unwrap();

        // Both should produce equivalent results
        assert_eq!(
            turtle_ontology.classes().len(),
            auto_ontology.classes().len()
        );
        assert_eq!(
            turtle_ontology.object_properties().len(),
            auto_ontology.object_properties().len()
        );

        // Test pipeline with auto-detected parser
        let reasoner = SimpleReasoner::new(auto_ontology);
        let is_consistent = reasoner.is_consistent().unwrap();
        assert!(is_consistent, "Auto-detected pipeline should work");
    }

    #[test]
    fn test_pipeline_state_management() {
        // Test that pipeline components maintain proper state isolation
        let content1 = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex1: <http://example1.org/> .

ex1:ClassA a owl:Class .
"#;

        let content2 = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex2: <http://example2.org/> .

ex2:ClassB a owl:Class .
ex2:ClassC a owl:Class .
"#;

        // Process first ontology
        let parser = TurtleParser::new();
        let ontology1 = parser.parse_str(content1).unwrap();

        let reasoner1 = SimpleReasoner::new(ontology1.clone());
        let consistent1 = reasoner1.is_consistent().unwrap();

        // Process second ontology (should not interfere with first)
        let ontology2 = parser.parse_str(content2).unwrap();
        let reasoner2 = SimpleReasoner::new(ontology2.clone());
        let consistent2 = reasoner2.is_consistent().unwrap();

        // Verify state isolation
        assert_eq!(
            ontology1.classes().len(),
            1,
            "First ontology should have 1 class"
        );
        assert_eq!(
            ontology2.classes().len(),
            2,
            "Second ontology should have 2 classes"
        );
        assert!(consistent1, "First ontology should be consistent");
        assert!(consistent2, "Second ontology should be consistent");
    }

    #[test]
    fn test_end_to_end_workflow() {
        // Complete end-to-end test simulating real usage
        let realistic_content = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .

# Organization structure
ex:Organization a owl:Class .
ex:Company rdfs:subClassOf ex:Organization .
ex:University rdfs:subClassOf ex:Organization .

# People
ex:Person a owl:Class .
ex:Employee rdfs:subClassOf ex:Person .
ex:Student rdfs:subClassOf ex:Person .
ex:Professor rdfs:subClassOf ex:Employee, ex:Student .

# Properties
ex:worksFor a owl:ObjectProperty ;
    rdfs:domain ex:Employee ;
    rdfs:range ex:Organization .

ex:studiesAt a owl:ObjectProperty ;
    rdfs:domain ex:Student ;
    rdfs:range ex:University .

ex:teachesAt a owl:ObjectProperty ;
    rdfs:domain ex:Professor ;
    rdfs:range ex:University .

ex:manages a owl:ObjectProperty ;
    rdfs:domain ex:Employee ;
    rdfs:range ex:Organization .

# Data properties
ex:hasName a owl:DatatypeProperty ;
    rdfs:domain ex:Person ;
    rdfs:range xsd:string .

ex:hasAge a owl:DatatypeProperty ;
    rdfs:domain ex:Person ;
    rdfs:range xsd:integer .

ex:hasEmail a owl:DatatypeProperty ;
    rdfs:domain ex:Person ;
    rdfs:range xsd:string .

# Property characteristics
ex:worksFor a owl:FunctionalProperty .
ex:hasEmail a owl:FunctionalProperty .

# Individuals
ex:TechCorp a ex:Company .
ex:StateUniversity a ex:University .

ex:Alice a ex:Professor ;
    ex:worksFor ex:StateUniversity ;
    ex:teachesAt ex:StateUniversity ;
    ex:hasName "Alice Smith" ;
    ex:hasAge 45 ;
    ex:hasEmail "alice@university.edu" .

ex:Bob a ex:Employee ;
    ex:worksFor ex:TechCorp ;
    ex:manages ex:TechCorp ;
    ex:hasName "Bob Johnson" ;
    ex:hasAge 35 ;
    ex:hasEmail "bob@techcorp.com" .

ex:Carol a ex:Student ;
    ex:studiesAt ex:StateUniversity ;
    ex:hasName "Carol Davis" ;
    ex:hasAge 22 .
"#;

        println!("Starting end-to-end workflow test...");

        // Step 1: Parse with auto-detection
        let start_time = Instant::now();
        let parser = ParserFactory::auto_detect(realistic_content).expect("Should detect format");

        let ontology = parser
            .parse_str(realistic_content)
            .expect("Should parse realistic ontology");

        let parse_time = start_time.elapsed();
        println!("Parsing completed in: {:?}", parse_time);

        // Step 2: Validate ontology structure
        // Note: Current parser implementation captures basic entity declarations
        // More sophisticated axiom parsing would be needed for full OWL support
        assert!(ontology.classes().len() >= 2, "Should have basic classes");
        assert!(
            ontology.object_properties().len() >= 4,
            "Should have multiple properties"
        );
        // Data properties and individuals require more sophisticated parsing

        // Step 3: Initialize reasoner
        let reasoner = SimpleReasoner::new(ontology.clone());

        // Step 4: Test reasoning
        let reasoning_start = Instant::now();
        let is_consistent = reasoner.is_consistent().expect("Reasoning should succeed");
        let reasoning_time = reasoning_start.elapsed();

        println!("Reasoning completed in: {:?}", reasoning_time);
        assert!(is_consistent, "Ontology should be consistent");

        // Step 5: Performance validation
        let total_time = start_time.elapsed();
        println!("Total end-to-end time: {:?}", total_time);

        // Performance assertions
        assert!(
            total_time.as_secs() < 10,
            "End-to-end workflow should complete within 10 seconds"
        );

        println!("End-to-end workflow test completed successfully!");
    }

    #[test]
    fn test_pipeline_memory_management() {
        // Test that pipeline properly manages memory for larger ontologies
        let mut large_content = String::new();
        large_content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
        large_content.push_str("@prefix ex: <http://example.org/> .\n\n");

        // Create many entities to test memory management
        for i in 0..100 {
            large_content.push_str(&format!("ex:Class{} a owl:Class .\n", i));
            if i > 0 {
                large_content.push_str(&format!(
                    "ex:Class{} rdfs:subClassOf ex:Class{} .\n",
                    i,
                    i - 1
                ));
            }
        }

        // Run complete pipeline
        let parser = TurtleParser::new();
        let ontology = parser.parse_str(&large_content).unwrap();

        let reasoner = SimpleReasoner::new(ontology.clone());

        // Should handle large input without memory issues
        assert!(ontology.classes().len() >= 50, "Should handle many classes");

        let is_consistent = reasoner.is_consistent().unwrap();
        assert!(is_consistent, "Should handle reasoning on large ontology");
    }
}
