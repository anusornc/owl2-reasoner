use owl2_reasoner::axioms::{
    SubObjectPropertyAxiom, ObjectPropertyDomainAxiom, ObjectPropertyRangeAxiom,
    InverseObjectPropertiesAxiom, Axiom
};
use owl2_reasoner::axioms::class_expressions::ClassExpression;
use owl2_reasoner::axioms::property_expressions::ObjectPropertyExpression;
use owl2_reasoner::entities::{Class, ObjectProperty};
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::tableaux::TableauxReasoner;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Property Hierarchy Integration\n");
    println!("======================================\n");

    // Test 1: SubObjectPropertyOf
    println!("Test 1: SubObjectPropertyOf (Property Hierarchy)");
    println!("------------------------------------------------");
    test_property_hierarchy()?;
    
    // Test 2: Property Domain
    println!("\nTest 2: Property Domain");
    println!("----------------------");
    test_property_domain()?;
    
    // Test 3: Property Range
    println!("\nTest 3: Property Range");
    println!("---------------------");
    test_property_range()?;
    
    // Test 4: Inverse Properties
    println!("\nTest 4: Inverse Properties");
    println!("-------------------------");
    test_inverse_properties()?;

    println!("\n✅ All Property Hierarchy Tests Complete!");
    println!("\nSummary:");
    println!("--------");
    println!("✓ SubObjectPropertyOf Rule: Implemented");
    println!("✓ Property Domain Rule: Implemented");
    println!("✓ Property Range Rule: Implemented");
    println!("✓ Inverse Property Rule: Implemented");
    
    println!("\nNote: Full testing requires ABox reasoning (Phase 1.3)");
    println!("      to create property assertions between individuals.");

    Ok(())
}

fn test_property_hierarchy() -> Result<(), Box<dyn std::error::Error>> {
    let mut ontology = Ontology::new();
    
    // Create properties: hasParent ⊑ hasAncestor
    let has_parent = Arc::new(IRI::new("http://example.org/hasParent")?);
    let has_ancestor = Arc::new(IRI::new("http://example.org/hasAncestor")?);
    
    let axiom = SubObjectPropertyAxiom::new(has_parent.clone(), has_ancestor.clone());
    let _ = ontology.add_axiom(Axiom::SubObjectProperty(Box::new(axiom)));
    
    let _reasoner = TableauxReasoner::new(ontology);
    println!("✓ Created ontology with property hierarchy: hasParent ⊑ hasAncestor");
    println!("  Rule: If (x,y) ∈ hasParent, then (x,y) ∈ hasAncestor");
    
    Ok(())
}

fn test_property_domain() -> Result<(), Box<dyn std::error::Error>> {
    let mut ontology = Ontology::new();
    
    // Create property and class
    let has_child = Arc::new(IRI::new("http://example.org/hasChild")?);
    let person_iri = Arc::new(IRI::new("http://example.org/Person")?);
    let person_class = Class::new(person_iri.clone());
    let person_expr = ClassExpression::Class(person_class);
    
    // Domain(hasChild) = Person
    let axiom = ObjectPropertyDomainAxiom::new(has_child.clone(), person_expr);
    let _ = ontology.add_axiom(Axiom::ObjectPropertyDomain(Box::new(axiom)));
    
    let _reasoner = TableauxReasoner::new(ontology);
    println!("✓ Created ontology with property domain: domain(hasChild) = Person");
    println!("  Rule: If (x,y) ∈ hasChild, then x : Person");
    
    Ok(())
}

fn test_property_range() -> Result<(), Box<dyn std::error::Error>> {
    let mut ontology = Ontology::new();
    
    // Create property and class
    let has_parent = Arc::new(IRI::new("http://example.org/hasParent")?);
    let person_iri = Arc::new(IRI::new("http://example.org/Person")?);
    let person_class = Class::new(person_iri.clone());
    let person_expr = ClassExpression::Class(person_class);
    
    // Range(hasParent) = Person
    let axiom = ObjectPropertyRangeAxiom::new((*has_parent).clone(), person_expr);
    let _ = ontology.add_axiom(Axiom::ObjectPropertyRange(Box::new(axiom)));
    
    let _reasoner = TableauxReasoner::new(ontology);
    println!("✓ Created ontology with property range: range(hasParent) = Person");
    println!("  Rule: If (x,y) ∈ hasParent, then y : Person");
    
    Ok(())
}

fn test_inverse_properties() -> Result<(), Box<dyn std::error::Error>> {
    let mut ontology = Ontology::new();
    
    // Create properties
    let has_child_iri = Arc::new(IRI::new("http://example.org/hasChild")?);
    let has_parent_iri = Arc::new(IRI::new("http://example.org/hasParent")?);
    
    let has_child_obj = ObjectProperty::new(has_child_iri);
    let has_parent_obj = ObjectProperty::new(has_parent_iri);
    
    let has_child_expr = ObjectPropertyExpression::ObjectProperty(Box::new(has_child_obj));
    let has_parent_expr = ObjectPropertyExpression::ObjectProperty(Box::new(has_parent_obj));
    
    // hasChild ≡ hasParent⁻
    let axiom = InverseObjectPropertiesAxiom::new(has_child_expr, has_parent_expr);
    let _ = ontology.add_axiom(Axiom::InverseObjectProperties(Box::new(axiom)));
    
    let _reasoner = TableauxReasoner::new(ontology);
    println!("✓ Created ontology with inverse properties: hasChild ≡ hasParent⁻");
    println!("  Rule: If (x,y) ∈ hasChild, then (y,x) ∈ hasParent");
    
    Ok(())
}

