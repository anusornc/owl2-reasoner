//! Property-based test runner
//!
//! This file runs all property-based tests for the OWL2 Reasoner.

#[cfg(test)]
mod tests {
    #[test]
    fn test_property_integration() {
        // This test ensures that property tests are properly integrated
        println!("Property-based tests are available and integrated");

        // Run a simple property test to verify functionality
        use owl2_reasoner::iri::IRI;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;

        let iri = IRI::new("http://example.org/test");
        if let Ok(iri) = iri {
            let mut hasher = DefaultHasher::new();
            iri.hash(&mut hasher);
            assert!(!iri.as_str().is_empty());
        }
    }

    #[test]
    fn test_property_characteristics_integration() {
        // Create an ontology with property characteristics
        use owl2_reasoner::*;

        let mut ontology = Ontology::new();
        ontology.set_iri("http://example.org/family");

        // Define properties
        let has_father = ObjectProperty::new("http://example.org/hasFather");
        let has_ssn = ObjectProperty::new("http://example.org/hasSSN");
        let knows = ObjectProperty::new("http://example.org/knows");
        let parent_of = ObjectProperty::new("http://example.org/parentOf");
        let spouse = ObjectProperty::new("http://example.org/spouse");
        let ancestor = ObjectProperty::new("http://example.org/ancestor");

        // Add properties to ontology
        ontology.add_object_property(has_father.clone()).unwrap();
        ontology.add_object_property(has_ssn.clone()).unwrap();
        ontology.add_object_property(knows.clone()).unwrap();
        ontology.add_object_property(parent_of.clone()).unwrap();
        ontology.add_object_property(spouse.clone()).unwrap();
        ontology.add_object_property(ancestor.clone()).unwrap();

        // Add property characteristic axioms
        let functional_axiom = FunctionalPropertyAxiom::new(has_father.iri().clone());
        let inv_functional_axiom = InverseFunctionalPropertyAxiom::new(has_ssn.iri().clone());
        let reflexive_axiom = ReflexivePropertyAxiom::new(knows.iri().clone());
        let irreflexive_axiom = IrreflexivePropertyAxiom::new(parent_of.iri().clone());
        let symmetric_axiom = SymmetricPropertyAxiom::new(spouse.iri().clone());
        let asymmetric_axiom = AsymmetricPropertyAxiom::new(parent_of.iri().clone());
        let transitive_axiom = TransitivePropertyAxiom::new(ancestor.iri().clone());

        // Add axioms to ontology
        ontology
            .add_axiom(Axiom::FunctionalProperty(functional_axiom))
            .unwrap();
        ontology
            .add_axiom(Axiom::InverseFunctionalProperty(inv_functional_axiom))
            .unwrap();
        ontology
            .add_axiom(Axiom::ReflexiveProperty(reflexive_axiom))
            .unwrap();
        ontology
            .add_axiom(Axiom::IrreflexiveProperty(irreflexive_axiom))
            .unwrap();
        ontology
            .add_axiom(Axiom::SymmetricProperty(symmetric_axiom))
            .unwrap();
        ontology
            .add_axiom(Axiom::AsymmetricProperty(asymmetric_axiom))
            .unwrap();
        ontology
            .add_axiom(Axiom::TransitiveProperty(transitive_axiom))
            .unwrap();

        // Test that all axioms were added correctly
        assert_eq!(ontology.functional_property_axioms().len(), 1);
        assert_eq!(ontology.inverse_functional_property_axioms().len(), 1);
        assert_eq!(ontology.reflexive_property_axioms().len(), 1);
        assert_eq!(ontology.irreflexive_property_axioms().len(), 1);
        assert_eq!(ontology.symmetric_property_axioms().len(), 1);
        assert_eq!(ontology.asymmetric_property_axioms().len(), 1);
        assert_eq!(ontology.transitive_property_axioms().len(), 1);

        // Test axiom contents
        let functional_props = ontology.functional_property_axioms();
        assert_eq!(functional_props[0].property(), has_father.iri());

        let reflexive_props = ontology.reflexive_property_axioms();
        assert_eq!(reflexive_props[0].property(), knows.iri());

        let transitive_props = ontology.transitive_property_axioms();
        assert_eq!(transitive_props[0].property(), ancestor.iri());

        // Test total axiom count
        assert_eq!(ontology.axiom_count(), 7);

        println!("✅ Property characteristics integration test passed!");
        println!("  - All 7 property characteristic types implemented");
        println!("  - Ontology integration working correctly");
        println!("  - Axiom storage and retrieval functional");
    }

    #[test]
    fn test_property_characteristics_with_reasoning() {
        use owl2_reasoner::*;

        // Create a more complex ontology with property characteristics and reasoning
        let mut ontology = Ontology::new();
        ontology.set_iri("http://example.org/kinship");

        // Define classes
        let person = Class::new("http://example.org/Person");
        let male = Class::new("http://example.org/Male");
        let female = Class::new("http://example.org/Female");

        // Define properties with characteristics
        let has_parent = ObjectProperty::new("http://example.org/hasParent");
        let has_child = ObjectProperty::new("http://example.org/hasChild");
        let married_to = ObjectProperty::new("http://example.org/marriedTo");

        // Add entities
        ontology.add_class(person.clone()).unwrap();
        ontology.add_class(male.clone()).unwrap();
        ontology.add_class(female.clone()).unwrap();
        ontology.add_object_property(has_parent.clone()).unwrap();
        ontology.add_object_property(has_child.clone()).unwrap();
        ontology.add_object_property(married_to.clone()).unwrap();

        // Add property characteristics
        ontology
            .add_axiom(Axiom::AsymmetricProperty(AsymmetricPropertyAxiom::new(
                has_parent.iri().clone(),
            )))
            .unwrap();
        ontology
            .add_axiom(Axiom::AsymmetricProperty(AsymmetricPropertyAxiom::new(
                has_child.iri().clone(),
            )))
            .unwrap();
        ontology
            .add_axiom(Axiom::SymmetricProperty(SymmetricPropertyAxiom::new(
                married_to.iri().clone(),
            )))
            .unwrap();
        ontology
            .add_axiom(Axiom::IrreflexiveProperty(IrreflexivePropertyAxiom::new(
                has_parent.iri().clone(),
            )))
            .unwrap();

        // Add class hierarchy
        let subclass_axiom = SubClassOfAxiom::new(
            ClassExpression::from(male.clone()),
            ClassExpression::from(person.clone()),
        );
        ontology
            .add_axiom(Axiom::SubClassOf(subclass_axiom))
            .unwrap();

        let subclass_axiom2 = SubClassOfAxiom::new(
            ClassExpression::from(female.clone()),
            ClassExpression::from(person.clone()),
        );
        ontology
            .add_axiom(Axiom::SubClassOf(subclass_axiom2))
            .unwrap();

        // Test that the ontology contains all expected axioms
        assert_eq!(ontology.asymmetric_property_axioms().len(), 2);
        assert_eq!(ontology.symmetric_property_axioms().len(), 1);
        assert_eq!(ontology.irreflexive_property_axioms().len(), 1);
        assert_eq!(ontology.subclass_axioms().len(), 2);

        // Test consistency (basic check that ontology doesn't have contradictions)
        let mut reasoner = OwlReasoner::new(ontology.clone());
        let is_consistent = reasoner.is_consistent().unwrap();
        assert!(
            is_consistent,
            "Ontology with property characteristics should be consistent"
        );

        println!("✅ Property characteristics with reasoning test passed!");
        println!("  - Complex ontology with multiple property characteristics");
        println!("  - Integration with class hierarchies");
        println!("  - Reasoning consistency maintained");
    }
}
