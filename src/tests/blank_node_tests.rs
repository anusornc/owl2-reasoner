//! Test for blank node property assertions in Turtle parser
//!
//! This test verifies that the Turtle parser can now handle
//! blank node property assertions correctly.

use crate::*;

#[test]
fn test_turtle_blank_node_property_assertion() -> OwlResult<()> {
    let turtle_content = r#"
@prefix : <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:Person a rdfs:Class .
:knows a rdf:ObjectProperty .

:john a :Person ;
    :knows _:blank1 ;
    :knows [ a :Person ; :name "Anonymous Person" ] .
"#;

    // Parse the Turtle content
    let parser = TurtleParser::new();
    let ontology = parser.parse_str(turtle_content)?;

    // Verify that the ontology contains the expected individuals
    let john_iri = IRI::new("http://example.org/john")?;
    assert!(ontology
        .named_individuals()
        .iter()
        .any(|ni| ni.iri() == &john_iri));

    // Check that we have anonymous individuals
    let anonymous_individuals = ontology.anonymous_individuals();
    println!(
        "Debug: Found {} anonymous individuals",
        anonymous_individuals.len()
    );
    println!("Debug: All axioms in ontology:");
    for (i, axiom) in ontology.axioms().iter().enumerate() {
        println!("  {}: {:?}", i + 1, axiom);
    }
    println!("Debug: Named individuals:");
    for ni in ontology.named_individuals() {
        println!("  - {}", ni.iri());
    }
    println!("Debug: Object properties:");
    for op in ontology.object_properties() {
        println!("  - {}", op.iri());
    }
    assert!(
        !anonymous_individuals.is_empty(),
        "Should have anonymous individuals"
    );

    // Count property assertions
    let property_assertions: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(***axiom, Axiom::PropertyAssertion(_)))
        .collect();

    // Should have at least 2 property assertions (one for _:blank1, one for nested blank node)
    assert!(
        property_assertions.len() >= 2,
        "Expected at least 2 property assertions, got {}",
        property_assertions.len()
    );

    // Verify that property assertions with anonymous individuals were created
    let mut anon_assertions = 0;
    for axiom in &property_assertions {
        if let Axiom::PropertyAssertion(assertion) = axiom.as_ref() {
            if assertion.object_anonymous().is_some() {
                anon_assertions += 1;
            }
        }
    }

    assert!(
        anon_assertions >= 1,
        "Expected at least 1 property assertion with anonymous individual, got {}",
        anon_assertions
    );

    println!("✅ Turtle blank node property assertions work correctly!");
    println!(
        "   - Total property assertions: {}",
        property_assertions.len()
    );
    println!("   - Anonymous individual assertions: {}", anon_assertions);
    println!(
        "   - Anonymous individuals: {}",
        anonymous_individuals.len()
    );

    Ok(())
}

#[test]
fn test_property_assertion_axiom_with_anonymous() -> OwlResult<()> {
    // Test the new PropertyAssertionAxiom functionality directly

    let mut ontology = Ontology::new();

    // Create named individual
    let john_iri = IRI::new("http://example.org/john")?;
    let john = NamedIndividual::new(john_iri.clone());
    ontology.add_named_individual(john)?;

    // Create anonymous individual
    let anon_individual = AnonymousIndividual::new("blank1");
    ontology.add_anonymous_individual(anon_individual.clone())?;

    // Create property
    let knows_iri = IRI::new("http://example.org/knows")?;
    let knows = ObjectProperty::new(knows_iri.clone());
    ontology.add_object_property(knows)?;

    // Create property assertion with anonymous individual
    let assertion = PropertyAssertionAxiom::new_with_anonymous(
        john_iri.clone(),
        knows_iri.clone(),
        anon_individual,
    );

    ontology.add_axiom(Axiom::PropertyAssertion(Box::new(assertion)))?;

    // Verify the axiom was added
    let assertions: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| match axiom.as_ref() {
            Axiom::PropertyAssertion(pa) => {
                pa.subject() == &john_iri && pa.property() == &knows_iri
            }
            _ => false,
        })
        .collect();

    assert_eq!(
        assertions.len(),
        1,
        "Should have exactly one property assertion"
    );

    if let Axiom::PropertyAssertion(pa) = assertions[0].as_ref() {
        assert!(
            pa.object_anonymous().is_some(),
            "Object should be anonymous"
        );
        assert!(
            pa.object_iri().is_none(),
            "Object should not be a named IRI"
        );
    }

    println!("✅ PropertyAssertionAxiom with anonymous individuals works correctly!");

    Ok(())
}
