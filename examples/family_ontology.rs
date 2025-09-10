//! Family Ontology Example
//! 
//! This example demonstrates how to create a family relationship ontology
//! using the OWL2 Reasoner library, including property characteristics and
//! basic reasoning.

use owl2_reasoner::*;

fn main() -> OwlResult<()> {
    println!("=== Family Ontology Example ===\n");

    // Create a new ontology
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/family");

    // Define classes
    let person = Class::new("http://example.org/Person");
    let parent = Class::new("http://example.org/Parent");
    let child = Class::new("http://example.org/Child");
    let male = Class::new("http://example.org/Male");
    let female = Class::new("http://example.org/Female");
    let father = Class::new("http://example.org/Father");
    let mother = Class::new("http://example.org/Mother");

    // Add classes to ontology
    for class in &[&person, &parent, &child, &male, &female, &father, &mother] {
        ontology.add_class(class.clone())?;
    }

    println!("✓ Added {} classes", ontology.classes().len());

    // Define properties with characteristics
    let mut has_parent = ObjectProperty::new("http://example.org/hasParent");
    let mut has_child = ObjectProperty::new("http://example.org/hasChild");
    let mut has_spouse = ObjectProperty::new("http://example.org/hasSpouse");
    let mut has_father = ObjectProperty::new("http://example.org/hasFather");
    let mut has_mother = ObjectProperty::new("http://example.org/hasMother");

    // Add property characteristics
    has_parent.add_characteristic(ObjectPropertyCharacteristic::Transitive);
    has_parent.add_characteristic(ObjectPropertyCharacteristic::Asymmetric);
    has_parent.add_characteristic(ObjectPropertyCharacteristic::Irreflexive);

    has_child.add_characteristic(ObjectPropertyCharacteristic::Asymmetric);
    has_child.add_characteristic(ObjectPropertyCharacteristic::Irreflexive);

    has_spouse.add_characteristic(ObjectPropertyCharacteristic::Symmetric);
    has_spouse.add_characteristic(ObjectPropertyCharacteristic::Irreflexive);

    // Add properties to ontology
    for prop in &[&has_parent, &has_child, &has_spouse, &has_father, &has_mother] {
        ontology.add_object_property(prop.clone())?;
    }

    println!("✓ Added {} object properties", ontology.object_properties().len());

    // Add subclass relationships
    let subclass_axioms = vec![
        SubClassOfAxiom::new(ClassExpression::from(parent.clone()), ClassExpression::from(person.clone())),
        SubClassOfAxiom::new(ClassExpression::from(child.clone()), ClassExpression::from(person.clone())),
        SubClassOfAxiom::new(ClassExpression::from(father.clone()), ClassExpression::from(parent.clone())),
        SubClassOfAxiom::new(ClassExpression::from(father.clone()), ClassExpression::from(male.clone())),
        SubClassOfAxiom::new(ClassExpression::from(mother.clone()), ClassExpression::from(parent.clone())),
        SubClassOfAxiom::new(ClassExpression::from(mother.clone()), ClassExpression::from(female.clone())),
    ];

    for axiom in subclass_axioms {
        ontology.add_subclass_axiom(axiom)?;
    }

    println!("✓ Added {} subclass axioms", ontology.subclass_axioms().len());

    // Add property hierarchy
    let prop_subclass_axioms = vec![
        SubObjectPropertyAxiom::new(
            ObjectPropertyExpression::ObjectProperty(has_father.clone()),
            ObjectPropertyExpression::ObjectProperty(has_parent.clone()),
        ),
        SubObjectPropertyAxiom::new(
            ObjectPropertyExpression::ObjectProperty(has_mother.clone()),
            ObjectPropertyExpression::ObjectProperty(has_parent.clone()),
        ),
    ];

    for axiom in prop_subclass_axioms {
        ontology.add_subobject_property_axiom(axiom)?;
    }

    println!("✓ Added {} subproperty axioms", ontology.subobject_property_axioms().len());

    // Add individuals
    let john = NamedIndividual::new("http://example.org/John");
    let mary = NamedIndividual::new("http://example.org/Mary");
    let alice = NamedIndividual::new("http://example.org/Alice");
    let bob = NamedIndividual::new("http://example.org/Bob");

    for individual in &[&john, &mary, &alice, &bob] {
        ontology.add_named_individual(individual.clone())?;
    }

    println!("✓ Added {} named individuals", ontology.named_individuals().len());

    // Add class assertions
    let class_assertions = vec![
        ClassAssertionAxiom::new(ClassExpression::from(male.clone()), john.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(female.clone()), mary.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(female.clone()), alice.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(male.clone()), bob.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(parent.clone()), john.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(parent.clone()), mary.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(child.clone()), alice.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(child.clone()), bob.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(father.clone()), john.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(mother.clone()), mary.clone()),
    ];

    for assertion in class_assertions {
        ontology.add_class_assertion(assertion)?;
    }

    println!("✓ Added {} class assertions", ontology.class_assertions().len());

    // Add property assertions
    let property_assertions = vec![
        PropertyAssertionAxiom::new(has_child.clone(), john.clone(), alice.clone()),
        PropertyAssertionAxiom::new(has_child.clone(), john.clone(), bob.clone()),
        PropertyAssertionAxiom::new(has_child.clone(), mary.clone(), alice.clone()),
        PropertyAssertionAxiom::new(has_child.clone(), mary.clone(), bob.clone()),
        PropertyAssertionAxiom::new(has_spouse.clone(), john.clone(), mary.clone()),
        PropertyAssertionAxiom::new(has_spouse.clone(), mary.clone(), john.clone()),
        PropertyAssertionAxiom::new(has_father.clone(), alice.clone(), john.clone()),
        PropertyAssertionAxiom::new(has_father.clone(), bob.clone(), john.clone()),
        PropertyAssertionAxiom::new(has_mother.clone(), alice.clone(), mary.clone()),
        PropertyAssertionAxiom::new(has_mother.clone(), bob.clone(), mary.clone()),
    ];

    for assertion in property_assertions {
        ontology.add_property_assertion(assertion)?;
    }

    println!("✓ Added {} property assertions", ontology.property_assertions().len());

    // Create reasoner and perform reasoning
    println!("\n=== Reasoning Results ===");
    let reasoner = SimpleReasoner::new(ontology);

    // Check consistency
    let is_consistent = reasoner.is_consistent()?;
    println!("✓ Ontology is consistent: {}", is_consistent);

    // Check subclass relationships
    let subclass_checks = vec![
        (father.clone(), person.clone(), "Father ⊑ Person"),
        (mother.clone(), person.clone(), "Mother ⊑ Person"),
        (parent.clone(), person.clone(), "Parent ⊑ Person"),
        (child.clone(), person.clone(), "Child ⊑ Person"),
    ];

    for (sub, sup, desc) in subclass_checks {
        let result = reasoner.is_subclass_of(&sub, &sup)?;
        println!("✓ {}: {}", desc, result);
    }

    // Get instances
    println!("\n=== Instance Retrieval ===");
    let instance_checks = vec![
        (person.clone(), "Persons"),
        (parent.clone(), "Parents"),
        (child.clone(), "Children"),
        (father.clone(), "Fathers"),
        (mother.clone(), "Mothers"),
        (male.clone(), "Males"),
        (female.clone(), "Females"),
    ];

    for (class, desc) in instance_checks {
        let instances = reasoner.get_instances(&class)?;
        println!("✓ {}: {:?}", desc, instances);
    }

    // Query examples
    println!("\n=== Query Examples ===");
    let mut query_engine = QueryEngine::new(&reasoner.ontology);

    // Find all parents
    let parent_pattern = QueryPattern::Basic {
        subject: None,
        predicate: Some(QueryValue::IRI(IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?)),
        object: Some(QueryValue::IRI(parent.clone())),
    };

    let parents = query_engine.query_pattern(&parent_pattern)?;
    println!("✓ Found {} parents using query", parents.len());

    // Find all parent-child relationships
    let family_pattern = QueryPattern::Basic {
        subject: None,
        predicate: Some(QueryValue::IRI(has_child.clone())),
        object: None,
    };

    let relationships = query_engine.query_pattern(&family_pattern)?;
    println!("✓ Found {} parent-child relationships", relationships.len());

    // Performance statistics
    println!("\n=== Performance Statistics ===");
    println!("✓ Total entities: {}", reasoner.ontology.entity_count());
    println!("✓ Total axioms: {}", reasoner.ontology.axiom_count());
    println!("✓ Cache stats: {:?}", reasoner.cache_stats());

    // IRI cache statistics
    let iri_stats = global_iri_cache_stats();
    println!("✓ IRI cache hit rate: {:.2}%", iri_stats.hit_rate() * 100.0);

    println!("\n=== Example Complete ===");
    Ok(())
}