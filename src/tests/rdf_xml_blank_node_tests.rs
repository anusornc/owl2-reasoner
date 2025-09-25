//! Test for blank node property assertions in RDF/XML parser
//!
//! This test verifies that the RDF/XML parser can now handle
//! blank node property assertions correctly.

use crate::*;

#[test]
fn test_rdf_xml_blank_node_property_assertion() -> OwlResult<()> {
    let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:ex="http://example.org/">
    <owl:Ontology rdf:about="http://example.org/test"/>

    <owl:Class rdf:about="http://example.org/Person"/>
    <owl:ObjectProperty rdf:about="http://example.org/knows"/>

    <owl:NamedIndividual rdf:about="http://example.org/john">
        <ex:knows>
            <rdf:Description rdf:nodeID="blank1">
                <rdf:type rdf:resource="http://example.org/Person"/>
            </rdf:Description>
        </ex:knows>
        <ex:knows rdf:nodeID="blank2"/>
    </owl:NamedIndividual>

    <rdf:Description rdf:nodeID="blank2">
        <ex:name rdf:datatype="http://www.w3.org/2001/XMLSchema#string">Anonymous Person</ex:name>
    </rdf:Description>
</rdf:RDF>"#;

    // Parse the RDF/XML content
    let mut parser = RdfXmlParser::new();
    parser.config.strict_validation = false; // Use streaming parser
    let ontology = parser.parse_str(rdf_xml_content)?;

    // Verify that the ontology contains the expected individuals
    let john_iri = IRI::new("http://example.org/john")?;
    assert!(ontology
        .named_individuals()
        .iter()
        .any(|ni| ni(*iri()).as_ref() == &john_iri));

    // Check that we have anonymous individuals
    let anonymous_individuals = ontology.anonymous_individuals();
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

    // Should have property assertions
    assert!(
        !property_assertions.is_empty(),
        "Expected property assertions, got {}",
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

    println!("✅ RDF/XML blank node property assertions work correctly!");
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
fn test_rdf_xml_property_assertion_axiom_with_anonymous() -> OwlResult<()> {
    // Test the new PropertyAssertionAxiom functionality directly in RDF/XML context

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

    println!("✅ RDF/XML PropertyAssertionAxiom with anonymous individuals works correctly!");

    Ok(())
}

#[test]
fn test_rdf_xml_nested_blank_nodes() -> OwlResult<()> {
    // Test nested blank nodes (anonymous individuals within anonymous individuals)
    let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
    <rdf:Description rdf:about="http://example.org/john">
        <ex:hasFriend>
            <rdf:Description rdf:nodeID="friend1">
                <ex:hasAddress>
                    <rdf:Description rdf:nodeID="addr1">
                        <ex:city>New York</ex:city>
                    </rdf:Description>
                </ex:hasAddress>
            </rdf:Description>
        </ex:hasFriend>
    </rdf:Description>
</rdf:RDF>"#;

    let mut parser = RdfXmlParser::new();
    parser.config.strict_validation = false;
    let ontology = parser.parse_str(rdf_xml_content)?;

    // Should have anonymous individuals
    let anonymous_individuals = ontology.anonymous_individuals();
    assert!(
        anonymous_individuals.len() >= 2,
        "Should have multiple anonymous individuals"
    );

    // Should have property assertions
    let property_assertions: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(***axiom, Axiom::PropertyAssertion(_)))
        .collect();

    assert!(
        property_assertions.len() >= 2,
        "Should have multiple property assertions"
    );

    println!("✅ RDF/XML nested blank nodes work correctly!");
    println!(
        "   - Anonymous individuals: {}",
        anonymous_individuals.len()
    );
    println!("   - Property assertions: {}", property_assertions.len());

    Ok(())
}

#[test]
fn test_rdf_xml_mixed_named_and_blank_nodes() -> OwlResult<()> {
    // Test mixed named and blank node assertions
    let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
    <rdf:Description rdf:about="http://example.org/john">
        <ex:knows rdf:resource="http://example.org/mary"/>
        <ex:knows rdf:nodeID="anonymous1"/>
    </rdf:Description>

    <rdf:Description rdf:about="http://example.org/mary">
        <ex:name>Mary Smith</ex:name>
    </rdf:Description>
</rdf:RDF>"#;

    println!("Debug: About to create RDF/XML parser");
    let mut parser = RdfXmlParser::new();
    parser.config.strict_validation = false;
    println!("Debug: About to parse RDF/XML content");
    let ontology = parser.parse_str(rdf_xml_content)?;
    println!("Debug: RDF/XML parsing completed");

    // Should have named individuals
    let named_individuals = ontology.named_individuals();
    println!(
        "Debug: Found {} named individuals:",
        named_individuals.len()
    );
    for ni in named_individuals.iter() {
        println!("  - {}", ni.iri());
    }
    assert!(
        named_individuals.len() >= 2,
        "Should have named individuals"
    );

    // Should have anonymous individuals
    let anonymous_individuals = ontology.anonymous_individuals();
    assert!(
        !anonymous_individuals.is_empty(),
        "Should have anonymous individuals"
    );

    // Should have both types of property assertions
    let mut named_assertions = 0;
    let mut anon_assertions = 0;

    for axiom in ontology.axioms() {
        if let Axiom::PropertyAssertion(pa) = axiom.as_ref() {
            if pa.object_anonymous().is_some() {
                anon_assertions += 1;
            } else if pa.object_iri().is_some() {
                named_assertions += 1;
            }
        }
    }

    assert!(
        named_assertions >= 1,
        "Should have at least 1 named object assertion"
    );
    assert!(
        anon_assertions >= 1,
        "Should have at least 1 anonymous object assertion"
    );

    println!("✅ RDF/XML mixed named and blank nodes work correctly!");
    println!("   - Named individuals: {}", named_individuals.len());
    println!(
        "   - Anonymous individuals: {}",
        anonymous_individuals.len()
    );
    println!("   - Named object assertions: {}", named_assertions);
    println!("   - Anonymous object assertions: {}", anon_assertions);

    Ok(())
}
