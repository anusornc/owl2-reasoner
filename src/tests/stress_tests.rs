//! Stress testing for large ontologies
//!
//! This module tests the parser's performance and stability with large datasets.
//! All tests are now memory-safe and will fail gracefully before causing OOM.

use crate::parser::*;
use crate::test_helpers::MemorySafeTestConfig;
use crate::{memory_safe_bench_test, memory_safe_stress_test, memory_safe_test};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    memory_safe_test!(
        test_large_turtle_ontology_parsing,
        MemorySafeTestConfig::medium(),
        {
            // Generate a large Turtle ontology with many classes
            let mut turtle_content = String::new();
            turtle_content
                .push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
            turtle_content.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
            turtle_content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
            turtle_content.push_str("@prefix ex: <http://example.org/> .\n\n");

            // Generate 1000 classes
            for i in 0..1000 {
                turtle_content.push_str(&format!("ex:Class{} a owl:Class .\n", i));
            }

            // Generate 500 subclass relationships
            for i in 0..500 {
                turtle_content.push_str(&format!(
                    "ex:Class{} rdfs:subClassOf ex:Class{} .\n",
                    i + 500,
                    i
                ));
            }

            // Generate 100 object properties
            for i in 0..100 {
                turtle_content.push_str(&format!("ex:Property{} a owl:ObjectProperty .\n", i));
            }

            let start_time = Instant::now();
            let parser = TurtleParser::new();
            let result = parser.parse_str(&turtle_content);
            let duration = start_time.elapsed();

            assert!(result.is_ok(), "Large ontology parsing should succeed");
            let ontology = result.unwrap();

            // Verify we parsed the expected number of entities
            assert!(
                ontology.classes().len() >= 1000,
                "Should have at least 1000 classes"
            );

            println!("Large ontology parsing took: {:?}", duration);
            println!(
                "Parsed {} classes, {} object properties",
                ontology.classes().len(),
                ontology.object_properties().len()
            );
        }
    );

    memory_safe_test!(test_parser_memory_usage, MemorySafeTestConfig::small(), {
        // Test with moderate-sized ontology to check memory management
        let mut turtle_content = String::new();
        turtle_content.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        turtle_content.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
        turtle_content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
        turtle_content.push_str("@prefix ex: <http://example.org/> .\n\n");

        // Create ontology with hierarchy
        for i in 0..100 {
            turtle_content.push_str(&format!("ex:Class{} a owl:Class .\n", i));
            if i > 0 {
                turtle_content.push_str(&format!(
                    "ex:Class{} rdfs:subClassOf ex:Class{} .\n",
                    i,
                    i - 1
                ));
            }
        }

        let parser = TurtleParser::new();
        let result = parser.parse_str(&turtle_content);

        assert!(result.is_ok(), "Memory usage test should succeed");
        let ontology = result.unwrap();

        // Verify the hierarchy was parsed correctly
        assert_eq!(
            ontology.classes().len(),
            100,
            "Should have exactly 100 classes"
        );

        // The parser should handle this without excessive memory usage
        // (this is more of a smoke test than a precise memory measurement)
    });

    memory_safe_test!(
        test_parser_factory_auto_detect_large_content,
        MemorySafeTestConfig::small(),
        {
            // Test auto-detection with large content
            let mut content = String::new();
            content.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
            content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
            content.push_str("@prefix ex: <http://example.org/> .\n\n");

            for i in 0..500 {
                content.push_str(&format!("ex:Entity{} a owl:Class .\n", i));
            }

            let parser = ParserFactory::auto_detect(&content);
            assert!(
                parser.is_some(),
                "Should auto-detect parser for large content"
            );

            let parser = parser.unwrap();
            assert_eq!(
                parser.format_name(),
                "Turtle",
                "Should detect as Turtle format"
            );

            let result = parser.parse_str(&content);

            // For debugging, let's see what error we get
            if let Err(ref e) = result {
                println!("Parsing error: {:?}", e);
            }

            assert!(
                result.is_ok(),
                "Auto-detected parser should handle large content"
            );
        }
    );

    memory_safe_test!(
        test_parser_with_deep_hierarchy,
        MemorySafeTestConfig::small(),
        {
            // Test parsing ontologies with deep class hierarchies
            let mut turtle_content = String::new();
            turtle_content.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
            turtle_content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
            turtle_content.push_str("@prefix ex: <http://example.org/> .\n\n");

            // Create a deep hierarchy: Class0 <- Class1 <- ... <- Class99
            turtle_content.push_str("ex:Class0 a owl:Class .\n");
            for i in 1..100 {
                turtle_content.push_str(&format!("ex:Class{} a owl:Class .\n", i));
                turtle_content.push_str(&format!(
                    "ex:Class{} rdfs:subClassOf ex:Class{} .\n",
                    i,
                    i - 1
                ));
            }

            let parser = TurtleParser::new();
            let result = parser.parse_str(&turtle_content);

            assert!(result.is_ok(), "Deep hierarchy parsing should succeed");
            let ontology = result.unwrap();

            assert_eq!(
                ontology.classes().len(),
                100,
                "Should have 100 classes in deep hierarchy"
            );
        }
    );

    memory_safe_test!(
        test_multiple_large_imports,
        MemorySafeTestConfig::small(),
        {
            // Test handling of multiple import statements
            let turtle_content = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:MainOntology a owl:Ontology .
ex:MainOntology owl:imports <http://example.org/ontology1> .
ex:MainOntology owl:imports <http://example.org/ontology2> .
ex:MainOntology owl:imports <http://example.org/ontology3> .
ex:MainOntology owl:imports <http://example.org/ontology4> .
ex:MainOntology owl:imports <http://example.org/ontology5> .

ex:MainClass a owl:Class .
"#;

            let parser = TurtleParser::new();
            let result = parser.parse_str(turtle_content);

            assert!(result.is_ok(), "Multiple imports should be handled");
            let ontology = result.unwrap();

            assert_eq!(ontology.imports().len(), 5, "Should have 5 imports");
            assert!(
                !ontology.classes().is_empty(),
                "Should have at least the main class"
            );
        }
    );

    memory_safe_test!(
        test_large_number_of_properties,
        MemorySafeTestConfig::small(),
        {
            // Test with many object and data properties
            let mut turtle_content = String::new();
            turtle_content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
            turtle_content.push_str("@prefix ex: <http://example.org/> .\n\n");

            // Create many object properties
            for i in 0..200 {
                turtle_content
                    .push_str(&format!("ex:ObjectProperty{} a owl:ObjectProperty .\n", i));
            }

            // Create many data properties
            for i in 0..200 {
                turtle_content.push_str(&format!("ex:DataProperty{} a owl:DataProperty .\n", i));
            }

            let parser = TurtleParser::new();
            let result = parser.parse_str(&turtle_content);

            assert!(result.is_ok(), "Many properties should be handled");
            let ontology = result.unwrap();

            assert!(
                ontology.object_properties().len() >= 200,
                "Should have many object properties"
            );
            assert!(
                ontology.data_properties().len() >= 200,
                "Should have many data properties"
            );
        }
    );

    memory_safe_stress_test!(test_stress_test_mixed_content, {
        // Test with mixed and complex content
        let mut turtle_content = String::new();
        turtle_content.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        turtle_content.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
        turtle_content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
        turtle_content.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n");
        turtle_content.push_str("@prefix ex: <http://example.org/> .\n\n");

        // Mix of different entity types
        for i in 0..100 {
            turtle_content.push_str(&format!("ex:Class{} a owl:Class .\n", i));
            turtle_content.push_str(&format!("ex:Prop{} a owl:ObjectProperty .\n", i));
            turtle_content.push_str(&format!("ex:DataProp{} a owl:DataProperty .\n", i));
            turtle_content.push_str(&format!("ex:Individual{} a owl:NamedIndividual .\n", i));

            // Add some subclass relationships
            if i > 0 && i % 10 == 0 {
                turtle_content.push_str(&format!(
                    "ex:Class{} rdfs:subClassOf ex:Class{} .\n",
                    i,
                    i / 10
                ));
            }
        }

        let start_time = Instant::now();
        let parser = TurtleParser::new();
        let result = parser.parse_str(&turtle_content);
        let duration = start_time.elapsed();

        assert!(result.is_ok(), "Mixed content parsing should succeed");
        let ontology = result.unwrap();

        println!("Mixed content stress test took: {:?}", duration);
        println!(
            "Parsed: {} classes, {} object properties, {} data properties, {} individuals",
            ontology.classes().len(),
            ontology.object_properties().len(),
            ontology.data_properties().len(),
            ontology.named_individuals().len()
        );
    });

    memory_safe_bench_test!(test_parser_performance_scaling, 4, {
        // Test how parser performance scales with input size
        let sizes = vec![100, 500, 1000, 2000];

        for size in sizes {
            let mut content = String::new();
            content.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
            content.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
            content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
            content.push_str("@prefix ex: <http://example.org/> .\n\n");

            for i in 0..size {
                content.push_str(&format!("ex:Entity{} a owl:Class .\n", i));
                if i > 0 {
                    content.push_str(&format!(
                        "ex:Entity{} rdfs:subClassOf ex:Entity{} .\n",
                        i,
                        (i - 1) / 10
                    ));
                }
            }

            let start_time = Instant::now();
            let parser = TurtleParser::new();
            let result = parser.parse_str(&content);
            let duration = start_time.elapsed();

            assert!(result.is_ok(), "Parser should handle size {}", size);
            let ontology = result.unwrap();

            println!(
                "Size {}: {:?} ({} classes parsed)",
                size,
                duration,
                ontology.classes().len()
            );

            // Performance should be reasonable (not a precise benchmark, just a sanity check)
            assert!(
                duration.as_secs() < 20,
                "Parsing {} entities should not take more than 20 seconds",
                size
            );
        }
    });
}
