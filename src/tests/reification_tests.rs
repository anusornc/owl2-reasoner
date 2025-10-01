use std::sync::Arc;

/// Test for RDF Reification support (rdf:subject, rdf:predicate, rdf:object)
///
/// This test verifies that the parsers can now handle
/// RDF reification for making statements about statements.
use crate::*;

#[test]
fn test_reification_axiom_creation() -> OwlResult<()> {
    let mut ontology = Ontology::new();

    // Create reification axiom for: :john :hasParent :mary
    let reification_axiom = ReificationAxiom::new(
        Arc::new(IRI::new("http://example.org/statement1")?),
        Arc::new(IRI::new("http://example.org/john")?),
        Arc::new(IRI::new("http://example.org/hasParent")?),
        ReificationObject::Named(Arc::new(IRI::new("http://example.org/mary")?)),
    );

    // Add to ontology
    let _ = ontology.add_axiom(Axiom::Reification(Box::new(reification_axiom.clone())));

    // Verify the reification axiom was added
    let reifications: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(axiom.as_ref(), Axiom::Reification(_)))
        .collect();

    assert_eq!(
        reifications.len(),
        1,
        "Should have exactly one reification axiom"
    );

    if let Axiom::Reification(reification) = &**reifications[0] {
        assert_eq!(
            reification.reification_resource(),
            &Arc::new(IRI::new("http://example.org/statement1")?)
        );
        assert_eq!(
            reification.subject(),
            &Arc::new(IRI::new("http://example.org/john")?)
        );
        assert_eq!(
            reification.predicate(),
            &Arc::new(IRI::new("http://example.org/hasParent")?)
        );

        if let ReificationObject::Named(object) = reification.object() {
            assert_eq!(object, &Arc::new(IRI::new("http://example.org/mary")?));
        } else {
            panic!("Expected named object");
        }

        // Test original statement extraction
        let (subj, pred, obj) = reification.original_statement();
        assert_eq!(subj, &Arc::new(IRI::new("http://example.org/john")?));
        assert_eq!(pred, &Arc::new(IRI::new("http://example.org/hasParent")?));
        if let ReificationObject::Named(obj_iri) = obj {
            assert_eq!(obj_iri, &Arc::new(IRI::new("http://example.org/mary")?));
        }
    }

    // Test property assertion generation
    let assertions = reification_axiom.to_property_assertions()?;
    assert!(
        !assertions.is_empty(),
        "Should generate property assertions"
    );

    // Should have:
    // - rdf:subject, rdf:predicate, rdf:object assertions
    // - rdf:type assertion (rdf:Statement)
    assert_eq!(
        assertions.len(),
        4,
        "Should have 4 assertions for basic reification"
    );

    println!("✅ Reification axiom creation works correctly!");
    println!(
        "   - Reification resource: {}",
        reification_axiom.reification_resource()
    );
    println!(
        "   - Original statement: {} {} {}",
        reification_axiom.subject(),
        reification_axiom.predicate(),
        match reification_axiom.object() {
            ReificationObject::Named(iri) => iri.to_string(),
            ReificationObject::Anonymous(_) => "[blank]".to_string(),
            ReificationObject::Literal(_) => "[literal]".to_string(),
        }
    );
    println!("   - Generated {} property assertions", assertions.len());

    Ok(())
}

#[test]
fn test_reification_with_anonymous_object() -> OwlResult<()> {
    let mut ontology = Ontology::new();

    // Create anonymous individual
    let anon_individual = AnonymousIndividual::new("anon1");

    // Create reification axiom with anonymous object
    let reification_axiom = ReificationAxiom::new(
        Arc::new(IRI::new("http://example.org/statement2")?),
        Arc::new(IRI::new("http://example.org/john")?),
        Arc::new(IRI::new("http://example.org/hasFriend")?),
        ReificationObject::Anonymous(Box::new(anon_individual.clone())),
    );

    // Add to ontology
    ontology.add_anonymous_individual(anon_individual)?;
    let _ = ontology.add_axiom(Axiom::Reification(Box::new(reification_axiom.clone())));

    // Verify anonymous individual was added
    assert!(
        !ontology.anonymous_individuals().is_empty(),
        "Should have anonymous individuals"
    );

    // Test property assertion generation
    let assertions = reification_axiom.to_property_assertions()?;
    assert_eq!(assertions.len(), 4, "Should have 4 assertions");

    println!("✅ Reification with anonymous object works correctly!");

    Ok(())
}

#[test]
fn test_reification_with_literal_object() -> OwlResult<()> {
    let reification_axiom = ReificationAxiom::new(
        Arc::new(IRI::new("http://example.org/statement3")?),
        Arc::new(IRI::new("http://example.org/john")?),
        Arc::new(IRI::new("http://example.org/hasAge")?),
        ReificationObject::Literal(Literal::simple("25")),
    );

    // Test property assertion generation
    let assertions = reification_axiom.to_property_assertions()?;
    assert_eq!(assertions.len(), 4, "Should have 4 assertions");

    if let ReificationObject::Literal(lit) = reification_axiom.object() {
        assert_eq!(lit.lexical_form(), "25");
    } else {
        panic!("Expected literal object");
    }

    println!("✅ Reification with literal object works correctly!");

    Ok(())
}

#[test]
fn test_reification_with_additional_properties() -> OwlResult<()> {
    let mut ontology = Ontology::new();

    // Create additional property (e.g., dc:creator for provenance)
    let creator_property = PropertyAssertionAxiom::new(
        Arc::new(IRI::new("http://example.org/statement4")?),
        Arc::new(IRI::new("http://purl.org/dc/elements/1.1/creator")?),
        Arc::new(IRI::new("http://example.org/alice")?),
    );

    // Create reification axiom with additional properties
    let mut reification_axiom = ReificationAxiom::new(
        Arc::new(IRI::new("http://example.org/statement4")?),
        Arc::new(IRI::new("http://example.org/john")?),
        Arc::new(IRI::new("http://example.org/hasParent")?),
        ReificationObject::Named(Arc::new(IRI::new("http://example.org/mary")?)),
    );

    // Add additional property
    reification_axiom.add_property(creator_property);

    // Add to ontology
    let _ = ontology.add_axiom(Axiom::Reification(Box::new(reification_axiom.clone())));

    // Test property assertion generation
    let assertions = reification_axiom.to_property_assertions()?;

    // Should have basic 4 assertions plus additional properties
    assert_eq!(
        assertions.len(),
        5,
        "Should have 5 assertions (4 basic + 1 additional)"
    );
    assert_eq!(
        reification_axiom.properties().len(),
        1,
        "Should have 1 additional property"
    );

    println!("✅ Reification with additional properties works correctly!");
    println!("   - Total assertions: {}", assertions.len());
    println!(
        "   - Additional properties: {}",
        reification_axiom.properties().len()
    );

    Ok(())
}

#[test]
fn test_reification_with_properties_constructor() -> OwlResult<()> {
    let mut ontology = Ontology::new();

    // Create additional properties
    let properties = vec![
        PropertyAssertionAxiom::new(
            Arc::new(IRI::new("http://example.org/statement5")?),
            Arc::new(IRI::new("http://purl.org/dc/elements/1.1/creator")?),
            Arc::new(IRI::new("http://example.org/alice")?),
        ),
        PropertyAssertionAxiom::new(
            Arc::new(IRI::new("http://example.org/statement5")?),
            Arc::new(IRI::new("http://purl.org/dc/elements/1.1/date")?),
            Arc::new(IRI::new("http://example.org/2023-01-01")?),
        ),
    ];

    // Create reification axiom with properties using constructor
    let reification_axiom = ReificationAxiom::with_properties(
        Arc::new(IRI::new("http://example.org/statement5")?),
        Arc::new(IRI::new("http://example.org/john")?),
        Arc::new(IRI::new("http://example.org/hasParent")?),
        ReificationObject::Named(Arc::new(IRI::new("http://example.org/mary")?)),
        properties,
    );

    // Add to ontology
    let _ = ontology.add_axiom(Axiom::Reification(Box::new(reification_axiom.clone())));

    // Test property assertion generation
    let assertions = reification_axiom.to_property_assertions()?;

    // Should have basic 4 assertions plus 2 additional properties
    assert_eq!(
        assertions.len(),
        6,
        "Should have 6 assertions (4 basic + 2 additional)"
    );
    assert_eq!(
        reification_axiom.properties().len(),
        2,
        "Should have 2 additional properties"
    );

    println!("✅ Reification with properties constructor works correctly!");

    Ok(())
}

#[test]
fn test_reification_statement_structure() -> OwlResult<()> {
    // Test different types of statements and their reification

    // Test 1: Named subject, predicate, object
    let stmt1 = ReificationAxiom::new(
        Arc::new(IRI::new("http://example.org/statement1")?),
        Arc::new(IRI::new("http://example.org/john")?),
        Arc::new(IRI::new("http://example.org/knows")?),
        ReificationObject::Named(Arc::new(IRI::new("http://example.org/mary")?)),
    );

    let assertions1 = stmt1.to_property_assertions()?;
    assert_eq!(assertions1.len(), 4);

    // Test 2: Named subject, predicate, anonymous object
    let anon = AnonymousIndividual::new("anon1");
    let stmt2 = ReificationAxiom::new(
        Arc::new(IRI::new("http://example.org/statement2")?),
        Arc::new(IRI::new("http://example.org/john")?),
        Arc::new(IRI::new("http://example.org/hasFriend")?),
        ReificationObject::Anonymous(Box::new(anon)),
    );

    let assertions2 = stmt2.to_property_assertions()?;
    assert_eq!(assertions2.len(), 4);

    // Test 3: Named subject, predicate, literal object
    let stmt3 = ReificationAxiom::new(
        Arc::new(IRI::new("http://example.org/statement3")?),
        Arc::new(IRI::new("http://example.org/john")?),
        Arc::new(IRI::new("http://example.org/hasAge")?),
        ReificationObject::Literal(Literal::simple("30")),
    );

    let assertions3 = stmt3.to_property_assertions()?;
    assert_eq!(assertions3.len(), 4);

    // All should generate the same basic structure
    assert_eq!(assertions1.len(), assertions2.len());
    assert_eq!(assertions2.len(), assertions3.len());

    println!("✅ Reification statement structure works correctly for all object types!");

    Ok(())
}

#[test]
fn test_reification_rdf_statement_type() -> OwlResult<()> {
    let reification_axiom = ReificationAxiom::new(
        Arc::new(IRI::new("http://example.org/statement1")?),
        Arc::new(IRI::new("http://example.org/john")?),
        Arc::new(IRI::new("http://example.org/hasParent")?),
        ReificationObject::Named(Arc::new(IRI::new("http://example.org/mary")?)),
    );

    let assertions = reification_axiom.to_property_assertions()?;

    // Find the rdf:type assertion
    let type_assertions: Vec<_> = assertions
        .iter()
        .filter(|assertion| {
            assertion.property().as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
        })
        .collect();

    assert_eq!(
        type_assertions.len(),
        1,
        "Should have exactly one rdf:type assertion"
    );

    let type_assertion = type_assertions[0];
    assert_eq!(
        **type_assertion.subject(),
        Arc::new(IRI::new("http://example.org/statement1")?).into()
    );

    if let Some(object_iri) = type_assertion.object_iri() {
        assert_eq!(
            **object_iri,
            Arc::new(IRI::new(
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#Statement"
            )?)
            .into()
        );
    } else {
        panic!("Expected IRI object for rdf:type");
    }

    println!("✅ Reification rdf:Statement type assertion works correctly!");

    Ok(())
}
