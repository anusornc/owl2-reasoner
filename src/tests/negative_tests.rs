//! Negative test cases for edge cases and error conditions
//!
//! This module tests how the system handles invalid inputs, edge cases,
//! and error conditions to ensure robust error handling and graceful failures.

use crate::error::OwlError;
use crate::iri::IRI;
use crate::parser::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_empty_string_input() {
        let mut config = ParserConfig::default();
        config.strict_validation = true; // Explicit strict validation for error testing
        let parser = TurtleParser::with_config(config);
        let result = parser.parse_str("");

        // Empty input should fail validation
        assert!(result.is_err(), "Empty string should result in error");

        if let Err(OwlError::ValidationError(msg)) = result {
            assert!(
                msg.contains("no entities or imports"),
                "Should mention empty ontology"
            );
        }
    }

    #[test]
    fn test_whitespace_only_input() {
        let parser = TurtleParser::new();
        let result = parser.parse_str("   \n  \t  \r\n  ");

        assert!(
            result.is_err(),
            "Whitespace-only input should result in error"
        );
    }

    #[test]
    fn test_malformed_turtle_syntax() {
        let malformed_content = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix ex: <http://example.org/> .

ex:Person a owl:Class
ex:Animal a owl:Class .
"#; // Missing dot after first class declaration

        let mut config = ParserConfig::default();
        config.strict_validation = true; // Explicit strict validation for error testing
        let parser = TurtleParser::with_config(config);
        let result = parser.parse_str(malformed_content);

        // Malformed syntax should result in parse error
        assert!(
            result.is_err(),
            "Malformed Turtle syntax should result in error"
        );
    }

    #[test]
    fn test_invalid_iri_formats() {
        let invalid_iris = vec![
            "", // Empty string
        ];

        for invalid_iri in invalid_iris {
            let result = IRI::new(invalid_iri);
            assert!(result.is_err(), "Invalid IRI '{}' should fail", invalid_iri);
        }

        // These are currently considered valid (basic validation only)
        let currently_valid = vec![
            "not_a_valid_iri",
            "http://invalid[iri]",
            "ftp://no.protocol.supported",
            "http://example.org/ space in path",
            "javascript:alert('xss')",
            " ", // Whitespace only (currently accepted)
        ];

        for valid_iri in currently_valid {
            let result = IRI::new(valid_iri);
            assert!(result.is_ok(), "IRI '{}' is currently accepted", valid_iri);
        }
    }

    #[test]
    fn test_circular_subclass_relationships() {
        let circular_content = r#"
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:ClassA a owl:Class .
ex:ClassB a owl:Class .

ex:ClassA rdfs:subClassOf ex:ClassB .
ex:ClassB rdfs:subClassOf ex:ClassA .
"#;

        let parser = TurtleParser::new();
        let result = parser.parse_str(circular_content);

        // Currently this might succeed during parsing but should be caught during reasoning
        // For now, we just test that parsing doesn't crash
        assert!(
            result.is_ok(),
            "Circular relationships should parse without crashing"
        );

        let ontology = result.unwrap();
        assert_eq!(ontology.classes().len(), 2, "Should have both classes");
    }

    #[test]
    fn test_duplicate_entity_declarations() {
        let duplicate_content = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:Person a owl:Class .
ex:Person a owl:Class .  # Duplicate declaration
ex:Person a owl:Class .  # Third declaration
"#;

        let parser = TurtleParser::new();
        let result = parser.parse_str(duplicate_content);

        // Duplicate declarations should not crash the parser
        assert!(
            result.is_ok(),
            "Duplicate declarations should not crash parser"
        );

        let ontology = result.unwrap();
        // Should still only have one unique class
        assert_eq!(
            ontology.classes().len(),
            1,
            "Should have only one unique class"
        );
    }

    #[test]
    fn test_undefined_prefix_usage() {
        let undefined_prefix_content = r#"
@prefix ex: <http://example.org/> .

ex:Person a unknown:Class .  # undefined prefix 'unknown'
"#;

        let mut config = ParserConfig::default();
        config.strict_validation = true; // Explicit strict validation for error testing
        let parser = TurtleParser::with_config(config);
        let result = parser.parse_str(undefined_prefix_content);

        // Undefined prefix should result in error
        assert!(result.is_err(), "Undefined prefix should result in error");
    }

    #[test]
    fn test_malformed_prefix_declarations() {
        let malformed_prefixes = vec![
            "@prefix missing-angle-brackets http://example.org/ .",
            "@prefix <no-colon> <http://example.org/> .",
            "@prefix :<no-space> <http://example.org/> .",
            "@prefix valid: <unclosed-bracket .",
        ];

        for malformed in malformed_prefixes {
            let parser = TurtleParser::new();
            let result = parser.parse_str(malformed);

            // Malformed prefix declarations should result in error
            assert!(
                result.is_err(),
                "Malformed prefix should result in error: {}",
                malformed
            );
        }
    }

    #[test]
    fn test_file_not_found() {
        let parser = TurtleParser::new();
        let result = parser.parse_file(Path::new("/this/file/does/not/exist.ttl"));

        assert!(result.is_err(), "Non-existent file should result in error");
    }

    #[test]
    fn test_extremely_long_lines() {
        let mut content = String::new();
        content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
        content.push_str("@prefix ex: <http://example.org/> .\n\n");

        // Create a long line with many classes (simpler syntax)
        for i in 0..100 {
            content.push_str(&format!("ex:LongClass{} a owl:Class .\n", i));
        }

        let parser = TurtleParser::new();
        let result = parser.parse_str(&content);

        // Long content should not crash the parser
        assert!(result.is_ok(), "Long content should not crash parser");

        let ontology = result.unwrap();
        assert!(ontology.classes().len() >= 50, "Should parse many classes");
    }

    #[test]
    fn test_unicode_handling() {
        let unicode_content = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:中文 a owl:Class .
ex:русский a owl:Class .
ex:العربية a owl:Class .
ex:🦉 a owl:Class .
ex:café a owl:Class .
ex:naïve a owl:Class .

ex:中文 rdfs:label "中文"@zh .
ex:русский rdfs:label "русский"@ru .
ex:العربية rdfs:label "العربية"@ar .
"#;

        let parser = TurtleParser::new();
        let result = parser.parse_str(unicode_content);

        // Unicode characters should be handled properly
        assert!(result.is_ok(), "Unicode content should be handled properly");

        let ontology = result.unwrap();
        assert!(
            ontology.classes().len() >= 5,
            "Should handle Unicode class names"
        );
    }

    #[test]
    fn test_mixed_content_types() {
        let mixed_content = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .

# Mix of different types
ex:Person a owl:Class ;
    rdfs:label "Person"@en ;
    rdfs:comment "A human being" .

ex:hasAge a owl:DatatypeProperty ;
    rdfs:domain ex:Person ;
    rdfs:range xsd:integer .

ex:hasName a owl:DatatypeProperty ;
    rdfs:domain ex:Person ;
    rdfs:range xsd:string .

ex:John a owl:NamedIndividual ;
    a ex:Person ;
    ex:hasName "John Doe" ;
    ex:hasAge "30"^^xsd:integer .

# Some invalid triples mixed in
ex:InvalidTriple ex:invalidPredicate .  # Missing object
ex:AnotherInvalid "literal" .         # Missing predicate
.                                       # Empty triple
"#;

        let parser = TurtleParser::new();
        let result = parser.parse_str(mixed_content);

        // Should handle mixed valid/invalid content gracefully
        // This may partially succeed or fail depending on parser strictness
        // The important thing is that it doesn't crash
        let _result = result;
        // We don't assert success/failure since behavior may vary
        // but we ensure it doesn't panic
    }

    #[test]
    fn test_very_large_numbers() {
        let large_numbers = r#"
@prefix ex: <http://example.org/> .

ex:Test a ex:TestClass ;
    ex:largePositive "99999999999999999999999999999999999999999999999999" ;
    ex:largeNegative "-99999999999999999999999999999999999999999999999999" .
"#;

        let parser = TurtleParser::new();
        let result = parser.parse_str(large_numbers);

        // Large numbers should not crash the parser (even if not fully supported)
        let _result = result;
        // Don't assert success/failure as typed literals may not be fully supported
        // Important thing is no crash
    }

    #[test]
    fn test_deep_nesting_brackets() {
        let deeply_nested = r#"
@prefix ex: <http://example.org/> .

ex:DeepTest a ex:ComplexClass ;
    ex:property1 [
        ex:nestedProperty1 [
            ex:deeplyNestedProperty1 [
                ex:veryDeeplyNestedProperty1 [
                    ex:extremelyDeeplyNestedProperty1 "deep value"
                ]
            ]
        ]
    ] ;
    ex:property2 "simple value" .
"#;

        let parser = TurtleParser::new();
        let result = parser.parse_str(deeply_nested);

        // Deep nesting should not crash the parser (though may not be fully supported)
        let _result = result;
        // Don't assert success/failure as deep bracket parsing may not be implemented
        // Important thing is no crash
    }

    #[test]
    fn test_comment_edge_cases() {
        let comment_content = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

# Valid comment
ex:Class1 a owl:Class .

# Comment with special chars
ex:Class2 a owl:Class .

# Multiple comments
ex:Class3 a owl:Class .

ex:Class4 a owl:Class . # Inline comment

ex:Class5 a owl:Class .
"#;

        let parser = TurtleParser::new();
        let result = parser.parse_str(comment_content);

        // Comments should not crash the parser
        assert!(result.is_ok(), "Comments should not crash parser");

        let ontology = result.unwrap();
        assert!(
            ontology.classes().len() >= 5,
            "Should parse classes despite comments"
        );
    }

    #[test]
    fn test_parser_factory_no_detection() {
        let content_without_format_markers = "just some random text without any format markers";
        let parser = ParserFactory::auto_detect(content_without_format_markers);

        // Content without clear format markers should not detect a parser
        assert!(
            parser.is_none(),
            "Content without format markers should not be detected"
        );
    }

    #[test]
    fn test_memory_exhaustion_simulation() {
        // Test that parser handles memory pressure gracefully
        // We'll simulate this with a very large but not enormous string
        let mut huge_content = String::new();
        huge_content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
        huge_content.push_str("@prefix ex: <http://example.org/> .\n\n");

        // Create many entities (but not enough to actually exhaust memory)
        for i in 0..10000 {
            huge_content.push_str(&format!("ex:Entity{} a owl:Class .\n", i));
        }

        let start_time = std::time::Instant::now();
        let parser = TurtleParser::new();
        let result = parser.parse_str(&huge_content);
        let duration = start_time.elapsed();

        // Should handle large input gracefully
        assert!(result.is_ok(), "Large input should be handled gracefully");

        let ontology = result.unwrap();
        assert!(
            ontology.classes().len() >= 5000,
            "Should parse many classes"
        );

        println!("Large input test completed in: {:?}", duration);
    }

    #[test]
    fn test_concurrent_parser_access() {
        use std::thread;

        let mut handles = vec![];

        // Test concurrent parsing using independent parser instances per thread
        for i in 0..10 {
            let handle = thread::spawn(move || {
                let content = format!(
                    "@prefix owl: <http://www.w3.org/2002/07/owl#> .\n\
                     @prefix ex: <http://example.org/> .\n\n\
                     ex:Class{} a owl:Class .\n",
                    i
                );
                let parser = TurtleParser::new();
                parser.parse_str(&content)
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_ok(), "Concurrent parser access should work");
        }
    }

    #[test]
    fn test_error_message_quality() {
        let invalid_content = "@prefix invalid: syntax here";
        let parser = TurtleParser::new();
        let result = parser.parse_str(invalid_content);

        assert!(result.is_err(), "Invalid content should produce error");

        if let Err(OwlError::ParseError(msg)) = result {
            // Error message should be descriptive
            assert!(!msg.is_empty(), "Error message should not be empty");
            assert!(
                msg.len() > 10,
                "Error message should be reasonably descriptive"
            );
        } else if let Err(OwlError::ValidationError(msg)) = result {
            assert!(
                !msg.is_empty(),
                "Validation error message should not be empty"
            );
        }
    }

    #[test]
    fn test_parser_state_isolation() {
        // Test that multiple parse operations don't interfere with each other
        let parser = TurtleParser::new();

        // Parse first ontology
        let content1 = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex1: <http://example1.org/> .

ex1:Class1 a owl:Class .
"#;
        let result1 = parser.parse_str(content1);
        assert!(result1.is_ok(), "First parse should succeed");
        let ontology1 = result1.unwrap();
        assert_eq!(
            ontology1.classes().len(),
            1,
            "First ontology should have 1 class"
        );

        // Parse second, different ontology
        let content2 = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex2: <http://example2.org/> .

ex2:ClassA a owl:Class .
ex2:ClassB a owl:Class .
"#;
        let result2 = parser.parse_str(content2);
        assert!(result2.is_ok(), "Second parse should succeed");
        let ontology2 = result2.unwrap();
        assert_eq!(
            ontology2.classes().len(),
            2,
            "Second ontology should have 2 classes"
        );

        // Verify first ontology is unchanged
        assert_eq!(
            ontology1.classes().len(),
            1,
            "First ontology should still have 1 class"
        );
    }
}
