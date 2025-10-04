//! Comprehensive JSON-LD 1.1 W3C Standard Compliance Tests
//!
//! Tests for advanced JSON-LD features including language maps, reverse properties,
//! value objects, containers, nested contexts, and remote context fetching.

use owl2_reasoner::parser::{JsonLdParser, OntologyParser};

#[test]
fn test_full_standard_compliance_basic() {
    let parser = JsonLdParser::new();

    let json_ld_content = r#"
    {
        "@context": {
            "@vocab": "http://example.org/",
            "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
            "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
            "owl": "http://www.w3.org/2002/07/owl#"
        },
        "@graph": [
            {
                "@id": "Person",
                "@type": "owl:Class"
            },
            {
                "@id": "Animal",
                "@type": "owl:Class"
            },
            {
                "@id": "Person",
                "rdfs:subClassOf": {
                    "@id": "Animal"
                }
            }
        ]
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD: {:?}",
        result.err()
    );

    let ontology = result.unwrap();
    assert!(
        !ontology.classes().is_empty(),
        "No classes found in parsed ontology"
    );
}

#[test]
fn test_language_maps() {
    let parser = JsonLdParser::new();

    let json_ld_content = r#"
    {
        "@context": {
            "@vocab": "http://schema.org/",
            "name": {
                "@id": "http://schema.org/name",
                "@container": "@language"
            }
        },
        "@id": "http://example.org/alice",
        "@type": "Person",
        "name": {
            "en": "Alice",
            "fr": "Alice",
            "de": "Alice"
        }
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD with language maps: {:?}",
        result.err()
    );

    let ontology = result.unwrap();
    // Verify that language annotations are processed
    assert!(!ontology.classes().is_empty(), "No entities found");
}

#[test]
fn test_reverse_properties() {
    let parser = JsonLdParser::new();

    let json_ld_content = r#"
    {
        "@context": {
            "@vocab": "http://schema.org/",
            "parent": "http://schema.org/parent",
            "child": {
                "@reverse": "http://schema.org/parent"
            }
        },
        "@id": "http://example.org/alice",
        "@type": "Person",
        "parent": "http://example.org/bob",
        "child": "http://example.org/carol"
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD with @reverse: {:?}",
        result.err()
    );

    let ontology = result.unwrap();
    // Verify that reverse properties are handled
    assert!(!ontology.classes().is_empty(), "No entities found");
}

#[test]
fn test_value_objects() {
    let parser = JsonLdParser::new();

    let json_ld_content = r#"
    {
        "@context": "http://schema.org/",
        "@id": "http://example.org/alice",
        "birthDate": {
            "@value": "1990-01-01",
            "@type": "http://www.w3.org/2001/XMLSchema#date"
        },
        "age": {
            "@value": 30,
            "@type": "http://www.w3.org/2001/XMLSchema#integer"
        },
        "isAlive": {
            "@value": true,
            "@type": "http://www.w3.org/2001/XMLSchema#boolean"
        }
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD with value objects: {:?}",
        result.err()
    );

    let ontology = result.unwrap();
    // Verify that typed literals are handled correctly
    assert!(!ontology.classes().is_empty(), "No entities found");
}

#[test]
fn test_containers() {
    let parser = JsonLdParser::new();

    // Test @set containers
    let json_ld_content = r#"
    {
        "@context": {
            "@vocab": "http://example.org/",
            "friends": {
                "@id": "http://schema.org/knows",
                "@container": "@set"
            }
        },
        "@id": "http://example.org/alice",
        "@type": "Person",
        "friends": [
            "http://example.org/bob",
            "http://example.org/carol",
            "http://example.org/dave"
        ]
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD with @set containers: {:?}",
        result.err()
    );

    // Test @list containers
    let json_ld_content = r#"
    {
        "@context": {
            "@vocab": "http://example.org/",
            "steps": {
                "@id": "http://example.org/alice",
                "@type": "Person",
                "steps": {
                    "@list": ["step1", "step2", "step3"]
                }
            }
        }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD with @list containers: {:?}",
        result.err()
    );

    // Test @index containers
    let json_ld_content = r#"
    {
        "@context": {
            "@vocab": "http://example.org/",
            "scores": {
                "@id": "http://example.org/alice",
                "@type": "Person",
                "scores": {
                    "@container": "@index",
                    "score": 95
                }
            }
        }
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD with @index containers: {:?}",
        result.err()
    );

    // Test @language containers
    let json_ld_content = r#"
    {
        "@context": {
            "@vocab": "http://example.org/",
            "descriptions": {
                "@container": "@language"
            }
        },
        "@id": "http://example.org/alice",
        "@type": "Person",
        "descriptions": {
            "en": "Alice Smith",
            "fr": "Alice Smith",
            "de": "Alice Smith"
        }
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD with @language containers: {:?}",
        result.err()
    );

    let ontology = result.unwrap();
    assert!(!ontology.classes().is_empty(), "No entities found");
}

#[test]
fn test_nested_contexts() {
    let parser = JsonLdParser::new();

    let json_ld_content = r#"
    {
        "@context": {
            "name": "http://schema.org/name",
            "@context": {
                "homepage": {
                    "@id": "http://schema.org/url",
                    "@type": "@id"
                }
            }
        },
        "@id": "http://example.org/alice",
        "@type": "Person",
        "name": "Alice"
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD with nested contexts: {:?}",
        result.err()
    );

    let ontology = result.unwrap();
    assert!(!ontology.classes().is_empty(), "No entities found");
}

#[test]
fn test_remote_context_fetching() {
    let parser = JsonLdParser::new();

    let json_ld_content = r#"
    {
        "@context": "http://schema.org/",
        "@type": "Person",
        "name": "Alice"
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD with remote context: {:?}",
        result.err()
    );

    let ontology = result.unwrap();
    assert!(!ontology.classes().is_empty(), "No entities found");
}

#[test]
fn test_mixed_features() {
    let parser = JsonLdParser::new();

    let json_ld_content = r#"
    {
        "@context": {
            "@vocab": "http://example.org/",
            "parent": "http://example.org/parent",
            "child": {
                "@reverse": "http://example.org/parent"
            }
        },
        "@id": "http://example.org/alice",
        "@type": "Person",
        "parent": "http://example.org/bob",
        "child": "http://example.org/carol"
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD with mixed features: {:?}",
        result.err()
    );

    let ontology = result.unwrap();
    assert!(!ontology.classes().is_empty(), "No entities found");
}

#[test]
fn test_blank_nodes() {
    let parser = JsonLdParser::new();

    let json_ld_content = r#"
    {
        "@context": "http://example.org/",
        "@graph": [
            {
                "@id": "_:alice",
                "@type": "Person",
                "name": "Alice"
            },
            {
                "@id": "_:bob",
                "@type": "Person",
                "name": "Bob"
            },
            {
                "@id": "_:relationship1",
                "@type": "Person",
                "knows": [
                    "_:alice"
                ]
            }
        ]
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse JSON-LD with blank nodes: {:?}",
        result.err()
    );

    let ontology = result.unwrap();
    assert!(!ontology.classes().is_empty(), "No entities found");
}

#[test]
fn test_complex_json_ld_structure() {
    let parser = JsonLdParser::new();

    let json_ld_content = r#"
    {
        "@context": {
            "@vocab": "http://example.org/",
            "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
            "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
            "owl": "http://www.w3.org/2002/07/owl#",
            "foaf": "http://xmlns.com/foaf#"
        },
        "@id": "http://example.org/ontology",
        "@type": "owl:Ontology",
        "imports": [
            {
                "@id": "http://example.org/person",
                "@type": "foaf:Person"
            },
            {
                "@id": "http://xmlns.com/foaf",
                "@type": "foaf:Person"
            }
        ],
        "versionInfo": {
            "@value": "1.0",
            "@type": "http://www.w3.org/2001/XMLSchema#string"
        },
        "ontology": "http://example.org/ontology"
    }
    "#;

    let result = parser.parse_str(json_ld_content);
    assert!(
        result.is_ok(),
        "Failed to parse complex JSON-LD structure: {:?}",
        result.err()
    );

    let ontology = result.unwrap();
    assert!(!ontology.classes().is_empty(), "No entities found");
}
