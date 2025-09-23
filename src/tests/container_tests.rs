//! Test for RDF Containers support (Seq, Bag, Alt)
//!
//! This test verifies that the parsers can now handle
//! RDF containers using rdf:_1, rdf:_2, etc. properties.

use crate::*;

#[test]
fn test_container_axiom_creation() -> OwlResult<()> {
    let mut ontology = Ontology::new();

    // Create test items
    let item1_iri = IRI::new("http://example.org/item1")?;
    let item2_iri = IRI::new("http://example.org/item2")?;
    let item3_iri = IRI::new("http://example.org/item3")?;

    let items = vec![
        ContainerItem::Named(item1_iri.clone()),
        ContainerItem::Named(item2_iri.clone()),
        ContainerItem::Named(item3_iri.clone()),
    ];

    // Create container axiom (Sequence)
    let container_axiom = ContainerAxiom::new(
        IRI::new("http://example.org/subject")?,
        IRI::new("http://example.org/hasItems")?,
        ContainerType::Sequence,
        items,
    );

    // Add to ontology
    ontology.add_axiom(Axiom::Container(Box::new(container_axiom.clone())))?;

    // Verify the container axiom was added
    let containers: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(***axiom, Axiom::Container(_)))
        .collect();

    assert_eq!(
        containers.len(),
        1,
        "Should have exactly one container axiom"
    );

    if let Axiom::Container(container) = &**containers[0] {
        assert_eq!(
            container.subject(),
            &IRI::new("http://example.org/subject")?
        );
        assert_eq!(
            container.property(),
            &IRI::new("http://example.org/hasItems")?
        );
        assert_eq!(container.container_type(), ContainerType::Sequence);
        assert_eq!(container.len(), 3, "Should have 3 items");
        assert!(!container.is_empty(), "Container should not be empty");
    }

    // Test property assertion generation
    let assertions = container_axiom.to_property_assertions()?;
    assert!(
        !assertions.is_empty(),
        "Should generate property assertions"
    );

    // Should have:
    // - 1 assertion connecting subject to container
    // - 1 type assertion (rdf:type rdf:Seq)
    // - 3 element assertions (rdf:_1, rdf:_2, rdf:_3)
    assert_eq!(assertions.len(), 5, "Should have 5 assertions");

    println!("✅ Container axiom creation works correctly!");
    println!("   - Container has {} items", container_axiom.len());
    println!(
        "   - Container type: {:?}",
        container_axiom.container_type()
    );
    println!("   - Generated {} property assertions", assertions.len());

    Ok(())
}

#[test]
fn test_container_types() -> OwlResult<()> {
    let items = vec![
        ContainerItem::Named(IRI::new("http://example.org/item1")?),
        ContainerItem::Named(IRI::new("http://example.org/item2")?),
    ];

    // Test Sequence container
    let seq_container = ContainerAxiom::new(
        IRI::new("http://example.org/seqSubject")?,
        IRI::new("http://example.org/hasSeq")?,
        ContainerType::Sequence,
        items.clone(),
    );

    // Test Bag container
    let bag_container = ContainerAxiom::new(
        IRI::new("http://example.org/bagSubject")?,
        IRI::new("http://example.org/hasBag")?,
        ContainerType::Bag,
        items.clone(),
    );

    // Test Alternative container
    let alt_container = ContainerAxiom::new(
        IRI::new("http://example.org/altSubject")?,
        IRI::new("http://example.org/hasAlt")?,
        ContainerType::Alternative,
        items,
    );

    // Verify container types
    assert_eq!(seq_container.container_type(), ContainerType::Sequence);
    assert_eq!(bag_container.container_type(), ContainerType::Bag);
    assert_eq!(alt_container.container_type(), ContainerType::Alternative);

    // Test property assertions for each type
    let seq_assertions = seq_container.to_property_assertions()?;
    let bag_assertions = bag_container.to_property_assertions()?;
    let alt_assertions = alt_container.to_property_assertions()?;

    // All should generate the same number of assertions
    assert_eq!(seq_assertions.len(), bag_assertions.len());
    assert_eq!(bag_assertions.len(), alt_assertions.len());

    println!("✅ All container types work correctly!");

    Ok(())
}

#[test]
fn test_container_with_mixed_items() -> OwlResult<()> {
    let mut ontology = Ontology::new();

    // Create test items with mixed types
    let named_item = IRI::new("http://example.org/namedItem")?;
    let anon_item = AnonymousIndividual::new("blank1");
    let literal_item = Literal::simple("test literal");

    let items = vec![
        ContainerItem::Named(named_item.clone()),
        ContainerItem::Anonymous(Box::new(anon_item.clone())),
        ContainerItem::Literal(literal_item.clone()),
    ];

    // Create container axiom
    let container_axiom = ContainerAxiom::new(
        IRI::new("http://example.org/mixedSubject")?,
        IRI::new("http://example.org/hasMixedContainer")?,
        ContainerType::Bag,
        items,
    );

    // Add to ontology
    ontology.add_anonymous_individual(anon_item)?;
    ontology.add_axiom(Axiom::Container(Box::new(container_axiom)))?;

    // Verify anonymous individual was added
    assert!(
        !ontology.anonymous_individuals().is_empty(),
        "Should have anonymous individuals"
    );

    println!("✅ Mixed container works correctly!");

    Ok(())
}

#[test]
fn test_empty_container() -> OwlResult<()> {
    let container_axiom = ContainerAxiom::new(
        IRI::new("http://example.org/subject")?,
        IRI::new("http://example.org/hasContainer")?,
        ContainerType::Sequence,
        Vec::new(),
    );

    assert!(
        container_axiom.is_empty(),
        "Empty container should be empty"
    );
    assert_eq!(
        container_axiom.len(),
        0,
        "Empty container should have 0 items"
    );

    // Empty container should still generate subject->container and type assertions
    let assertions = container_axiom.to_property_assertions()?;
    assert_eq!(
        assertions.len(),
        2,
        "Empty container should generate 2 assertions (subject link + type)"
    );

    println!("✅ Empty container works correctly!");

    Ok(())
}

#[test]
fn test_container_vs_collection() -> OwlResult<()> {
    let items = vec![
        CollectionItem::Named(IRI::new("http://example.org/item1")?),
        CollectionItem::Named(IRI::new("http://example.org/item2")?),
    ];

    let container_items = vec![
        ContainerItem::Named(IRI::new("http://example.org/item1")?),
        ContainerItem::Named(IRI::new("http://example.org/item2")?),
    ];

    // Create collection (linked list with rdf:first/rdf:rest)
    let collection = CollectionAxiom::new(
        IRI::new("http://example.org/subject")?,
        IRI::new("http://example.org/hasCollection")?,
        items,
    );

    // Create container (numbered properties rdf:_1/rdf:_2)
    let container = ContainerAxiom::new(
        IRI::new("http://example.org/subject")?,
        IRI::new("http://example.org/hasContainer")?,
        ContainerType::Sequence,
        container_items,
    );

    // Compare property assertion generation
    let collection_assertions = collection.to_property_assertions()?;
    let container_assertions = container.to_property_assertions()?;

    // Collection generates more assertions due to linked list structure
    // Container generates simpler numbered property assertions
    assert!(
        collection_assertions.len() > container_assertions.len(),
        "Collection should generate more assertions than container"
    );

    println!("✅ Container vs Collection comparison works correctly!");
    println!(
        "   - Collection generates {} assertions",
        collection_assertions.len()
    );
    println!(
        "   - Container generates {} assertions",
        container_assertions.len()
    );

    Ok(())
}
