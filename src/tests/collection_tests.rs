use std::sync::Arc;
// Test for RDF Collections support (rdf:first, rdf:rest, rdf:nil)
//
// This test verifies that the parsers can now handle
// RDF collections using linked list structures.

use crate::*;

#[test]
fn test_collection_axiom_creation() -> OwlResult<()> {
    let mut ontology = Ontology::new();

    // Create test items
    let item1_iri = Arc::new(IRI::new("http://example.org/item1")?);
    let item2_iri = Arc::new(IRI::new("http://example.org/item2")?);
    let item3_iri = Arc::new(IRI::new("http://example.org/item3")?);

    let items = vec![
        CollectionItem::Named(item1_iri.clone()),
        CollectionItem::Named(item2_iri.clone()),
        CollectionItem::Named(item3_iri.clone()),
    ];

    // Create collection axiom
    let collection_axiom = CollectionAxiom::new(
        Arc::new(IRI::new("http://example.org/subject")?),
        Arc::new(IRI::new("http://example.org/hasList")?),
        items,
    );

    // Add to ontology
    ontology.add_axiom(Axiom::Collection(Box::new(collection_axiom.clone())))?;

    // Verify the collection axiom was added
    let collections: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(axiom.as_ref(), Axiom::Collection(_)))
        .collect();

    assert_eq!(
        collections.len(),
        1,
        "Should have exactly one collection axiom"
    );

    if let Axiom::Collection(collection) = &**collections[0] {
        assert_eq!(
            collection.subject(),
            &Arc::new(IRI::new("http://example.org/subject")?)
        );
        assert_eq!(
            collection.property(),
            &Arc::new(IRI::new("http://example.org/hasList")?)
        );
        assert_eq!(collection.len(), 3, "Should have 3 items");
        assert!(!collection.is_empty(), "Collection should not be empty");
    }

    // Test property assertion generation
    let assertions = collection_axiom.to_property_assertions()?;
    assert!(
        !assertions.is_empty(),
        "Should generate property assertions"
    );

    // Should have rdf:first and rdf:rest assertions for each node
    // plus one assertion connecting subject to first node
    assert_eq!(
        assertions.len(),
        7,
        "Should have 7 assertions (3 nodes × 2 + 1 subject link)"
    );

    println!("✅ Collection axiom creation works correctly!");
    println!("   - Collection has {} items", collection_axiom.len());
    println!("   - Generated {} property assertions", assertions.len());

    Ok(())
}

#[test]
fn test_collection_with_mixed_items() -> OwlResult<()> {
    let mut ontology = Ontology::new();

    // Create test items with mixed types
    let named_item = Arc::new(IRI::new("http://example.org/namedItem")?);
    let anon_item = AnonymousIndividual::new("blank1");
    let literal_item = Literal::simple("test literal");

    let items = vec![
        CollectionItem::Named(named_item.clone()),
        CollectionItem::Anonymous(Box::new(anon_item.clone())),
        CollectionItem::Literal(literal_item.clone()),
    ];

    // Create collection axiom
    let collection_axiom = CollectionAxiom::new(
        Arc::new(IRI::new("http://example.org/mixedSubject")?),
        Arc::new(IRI::new("http://example.org/hasMixedList")?),
        items,
    );

    // Add to ontology
    ontology.add_anonymous_individual(anon_item)?;
    ontology.add_axiom(Axiom::Collection(Box::new(collection_axiom)))?;

    // Verify anonymous individual was added
    assert!(
        !ontology.anonymous_individuals().is_empty(),
        "Should have anonymous individuals"
    );

    println!("✅ Mixed collection works correctly!");

    Ok(())
}

#[test]
fn test_empty_collection() -> OwlResult<()> {
    let collection_axiom = CollectionAxiom::new(
        Arc::new(IRI::new("http://example.org/subject")?),
        Arc::new(IRI::new("http://example.org/hasList")?),
        Vec::new(),
    );

    assert!(
        collection_axiom.is_empty(),
        "Empty collection should be empty"
    );
    assert_eq!(
        collection_axiom.len(),
        0,
        "Empty collection should have 0 items"
    );

    let assertions = collection_axiom.to_property_assertions()?;
    assert!(
        assertions.is_empty(),
        "Empty collection should generate no assertions"
    );

    println!("✅ Empty collection works correctly!");

    Ok(())
}

#[test]
fn test_turtle_collection_parsing() -> OwlResult<()> {
    let turtle_content = r#"@prefix : <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

:subject :hasList ( :item1 :item2 :item3 ) ."#;

    // Parse the Turtle content
    let parser = TurtleParser::new();
    let ontology = parser.parse_str(turtle_content)?;

    // Verify that the ontology contains property assertions
    let property_assertions: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(axiom.as_ref(), Axiom::PropertyAssertion(_)))
        .collect();

    // Should have property assertions from the collection
    assert!(
        !property_assertions.is_empty(),
        "Expected property assertions from collection, got {}",
        property_assertions.len()
    );

    // Look for rdf:first and rdf:rest assertions
    let mut first_assertions = 0;
    let mut rest_assertions = 0;

    for axiom in &property_assertions {
        if let Axiom::PropertyAssertion(pa) = axiom.as_ref() {
            if pa.property().as_str().ends_with("first") {
                first_assertions += 1;
            } else if pa.property().as_str().ends_with("rest") {
                rest_assertions += 1;
            }
        }
    }

    assert!(
        first_assertions >= 1,
        "Should have at least 1 rdf:first assertion"
    );
    assert!(
        rest_assertions >= 1,
        "Should have at least 1 rdf:rest assertion"
    );

    println!("✅ Turtle collection parsing works correctly!");
    println!(
        "   - Total property assertions: {}",
        property_assertions.len()
    );
    println!("   - rdf:first assertions: {}", first_assertions);
    println!("   - rdf:rest assertions: {}", rest_assertions);

    Ok(())
}
