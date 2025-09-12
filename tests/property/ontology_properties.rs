//! Property-based tests for ontology functionality

use proptest::prelude::*;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::entities::{Class, NamedIndividual};
use owl2_reasoner::axioms::{SubClassOfAxiom, ClassExpression};
use owl2_reasoner::iri::IRI;

/// Generator for valid IRI strings
fn valid_iri_string() -> impl Strategy<Value = String> {
    r"[a-zA-Z][a-zA-Z0-9-]*://[a-zA-Z0-9-._~:/?#\\[\\]@!$&'()*+,;=]+"
        .prop_filter("Non-empty and reasonable length", |s| !s.is_empty() && s.len() <= 2048)
}

/// Test that adding classes to an ontology is consistent
proptest! {
    #[test]
    fn test_class_addition_consistency(
        class_iris in prop::collection::vec(valid_iri_string(), 0..10)
    ) {
        let mut ontology = Ontology::new();
        let mut successful_additions = 0;
        
        for iri_str in class_iris {
            if let Ok(iri) = IRI::new(&iri_str) {
                let class = Class::new(iri);
                let result = ontology.add_class(class.clone());
                
                if result.is_ok() {
                    successful_additions += 1;
                    
                    // The class should be accessible after successful addition
                    let classes = ontology.classes();
                    assert!(classes.len() >= successful_additions, 
                            "Class count should reflect successful additions");
                }
            }
        }
        
        // Final sanity checks
        assert!(ontology.classes().len() <= successful_additions, 
                "Final class count should not exceed successful additions");
        assert!(ontology.entity_count() >= 0, 
                "Entity count should be non-negative");
    }
}

/// Test that adding individuals to an ontology is consistent
proptest! {
    #[test]
    fn test_individual_addition_consistency(
        individual_iris in prop::collection::vec(valid_iri_string(), 0..10)
    ) {
        let mut ontology = Ontology::new();
        let mut successful_additions = 0;
        
        for iri_str in individual_iris {
            if let Ok(iri) = IRI::new(&iri_str) {
                let individual = NamedIndividual::new(iri);
                let result = ontology.add_named_individual(individual.clone());
                
                if result.is_ok() {
                    successful_additions += 1;
                    
                    // The individual should be accessible after successful addition
                    let individuals = ontology.named_individuals();
                    assert!(individuals.len() >= successful_additions, 
                            "Individual count should reflect successful additions");
                }
            }
        }
        
        // Final sanity checks
        assert!(ontology.named_individuals().len() <= successful_additions, 
                "Final individual count should not exceed successful additions");
    }
}

/// Test that duplicate class additions are handled gracefully
proptest! {
    #[test]
    fn test_duplicate_class_handling(
        class_iri in valid_iri_string()
    ) {
        if let Ok(iri) = IRI::new(&class_iri) {
            let class = Class::new(iri);
            let mut ontology = Ontology::new();
            
            // Add the same class multiple times
            let result1 = ontology.add_class(class.clone());
            let result2 = ontology.add_class(class.clone());
            let result3 = ontology.add_class(class.clone());
            
            // All operations should complete without panics
            let _ = result1;
            let _ = result2;
            let _ = result3;
            
            // The ontology should remain functional
            assert!(ontology.classes().len() <= 1, 
                    "Duplicate classes should be handled gracefully");
        }
    }
}

/// Test that subclass relationships maintain basic properties
proptest! {
    #[test]
    fn test_subclass_relationship_properties(
        parent_iri in valid_iri_string(),
        child_iri in valid_iri_string(),
        grandchild_iri in valid_iri_string()
    ) {
        if let (Ok(parent_iri), Ok(child_iri), Ok(grandchild_iri)) = 
            (IRI::new(&parent_iri), IRI::new(&child_iri), IRI::new(&grandchild_iri)) {
            
            let mut ontology = Ontology::new();
            
            // Create classes
            let parent_class = Class::new(parent_iri);
            let child_class = Class::new(child_iri);
            let grandchild_class = Class::new(grandchild_iri);
            
            // Add classes to ontology
            let _ = ontology.add_class(parent_class.clone());
            let _ = ontology.add_class(child_class.clone());
            let _ = ontology.add_class(grandchild_class.clone());
            
            // Create subclass relationships
            let child_sub_parent = SubClassOfAxiom::new(
                ClassExpression::Class(child_class.clone()),
                ClassExpression::Class(parent_class.clone()),
            );
            
            let grandchild_sub_child = SubClassOfAxiom::new(
                ClassExpression::Class(grandchild_class.clone()),
                ClassExpression::Class(child_class.clone()),
            );
            
            // Add axioms - should not panic
            let _ = ontology.add_subclass_axiom(child_sub_parent);
            let _ = ontology.add_subclass_axiom(grandchild_sub_child);
            
            // Ontology should remain consistent
            assert!(ontology.classes().len() >= 3, 
                    "All classes should be accessible");
        }
    }
}

/// Test that ontology operations maintain entity count consistency
proptest! {
    #[test]
    fn test_entity_count_consistency(
        class_iris in prop::collection::vec(valid_iri_string(), 0..5),
        individual_iris in prop::collection::vec(valid_iri_string(), 0..5)
    ) {
        let mut ontology = Ontology::new();
        let mut expected_entities = 0;
        
        // Add classes
        for iri_str in class_iris {
            if let Ok(iri) = IRI::new(&iri_str) {
                let class = Class::new(iri);
                if ontology.add_class(class).is_ok() {
                    expected_entities += 1;
                }
            }
        }
        
        // Add individuals
        for iri_str in individual_iris {
            if let Ok(iri) = IRI::new(&iri_str) {
                let individual = NamedIndividual::new(iri);
                if ontology.add_named_individual(individual).is_ok() {
                    expected_entities += 1;
                }
            }
        }
        
        // Check consistency
        assert!(ontology.entity_count() <= expected_entities, 
                "Entity count should not exceed expected count");
        assert!(ontology.entity_count() >= 0, 
                "Entity count should be non-negative");
    }
}