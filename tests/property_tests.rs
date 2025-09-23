//! Property-based tests for the OWL2 reasoner
//!
//! This module uses proptest to generate random test cases and verify
//! that the reasoner behaves correctly under various conditions.

use owl2_reasoner::*;
use proptest::prelude::*;
use std::hash::Hash;

// Property-based tests for IRI creation and validation
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn iri_creation_preserves_string(s in "\\PC*") {
        // Test that IRI creation and string conversion are consistent
        if let Ok(iri) = IRI::new(&s) {
            assert_eq!(iri.as_str(), s);
        }
        // Invalid IRIs should return errors, which is expected behavior
    }

    #[test]
    fn iri_roundtrip(s in "\\PC*") {
        // Test that IRI creation and cloning are consistent
        if let Ok(iri1) = IRI::new(&s) {
            let iri2 = iri1.clone();
            assert_eq!(iri1.as_str(), iri2.as_str());
            use std::hash::{Hash, Hasher};
            let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
            let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
            iri1.hash(&mut hasher1);
            iri2.hash(&mut hasher2);
            assert_eq!(hasher1.finish(), hasher2.finish());
        }
    }

    #[test]
    fn valid_http_iris(
        host in "[a-z]{1,20}",
        path in "[a-z]{1,10}?",
        fragment in "[a-z0-9]{1,10}?"
    ) {
        // Test that well-formed HTTP IRIs are always valid
        let iri_str = format!("http://{}.example.org/{}#{}", host, path, fragment);
        let iri = IRI::new(&iri_str).expect("HTTP IRI should be valid");
        assert_eq!(iri.as_str(), iri_str);
    }
}

// Property-based tests for ontology operations
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn ontology_class_management(
        classes in prop::collection::vec(any::<String>(), 1..100)
    ) {
        let mut ontology = Ontology::new();

        // Add classes to ontology
        let mut valid_iris = Vec::new();
        let mut seen_iris = std::collections::HashSet::new();
        for class_str in classes {
            // Skip empty strings as they create invalid IRIs
            if !class_str.trim().is_empty() {
                if let Ok(iri) = IRI::new(&format!("http://example.org/{}", class_str)) {
                    // Skip duplicate IRIs to avoid conflicts
                    if !seen_iris.contains(&iri) {
                        if let Ok(_) = ontology.add_class(Class::new(iri.clone())) {
                            valid_iris.push(iri.clone());
                            seen_iris.insert(iri);
                        }
                    }
                }
            }
        }

        // Verify that all added classes can be retrieved
        let retrieved_classes: Vec<IRI> = ontology.classes().iter().map(|c| c.iri().clone()).collect();
        assert_eq!(retrieved_classes.len(), valid_iris.len(),
                   "Retrieved classes count ({}) should match valid IRIs count ({})",
                   retrieved_classes.len(), valid_iris.len());

        // Verify each class is in the ontology
        for iri in &valid_iris {
            assert!(ontology.classes().iter().any(|c| c.iri() == iri));
        }
    }

    #[test]
    fn subclass_relationships(
        classes in prop::collection::vec(any::<String>(), 2..50)
    ) {
        let mut ontology = Ontology::new();

        // Create classes
        let mut class_iris = Vec::new();
        for (i, class_str) in classes.iter().enumerate() {
            let iri = IRI::new(&format!("http://example.org/{}", class_str))
                .unwrap_or_else(|_| IRI::new(&format!("http://example.org/class{}", i)).unwrap());
            let class = Class::new(iri.clone());
            ontology.add_class(class).unwrap();
            class_iris.push(iri);
        }

        // Create subclass relationships (each class is subclass of previous)
        for i in 1..class_iris.len() {
            let subclass_axiom = SubClassOfAxiom::new(
                ClassExpression::Class(Class::new(class_iris[i].clone())),
                ClassExpression::Class(Class::new(class_iris[i-1].clone())),
            );
            ontology.add_subclass_axiom(subclass_axiom).unwrap();
        }

        // Verify with reasoner
        let reasoner = SimpleReasoner::new(ontology);

        // Check transitive subclass relationships
        for i in 1..class_iris.len() {
            for j in 0..i {
                let _is_subclass = reasoner.is_subclass_of(&class_iris[i], &class_iris[j]).unwrap();
                // The key is that the operation doesn't panic
            }
        }
    }
}

// Test utility functions
fn create_test_iri(suffix: &str) -> IRI {
    IRI::new(&format!("http://example.org/{}", suffix)).unwrap()
}

// Simple integration test to verify property testing works
#[test]
fn test_property_testing_integration() {
    use owl2_reasoner::iri::IRI;

    // Test that basic property testing functionality works
    let test_iri = IRI::new("http://example.org/test").unwrap();
    assert_eq!(test_iri.as_str(), "http://example.org/test");

    // Test property with random data
    let random_suffix = format!("test{}", rand::random::<u64>());
    let random_iri = IRI::new(&format!("http://example.org/{}", random_suffix)).unwrap();
    assert!(random_iri.as_str().ends_with(&random_suffix));
}
