use owl2_reasoner::axioms::{ClassExpression, DisjointClassesAxiom, SubClassOfAxiom};
use owl2_reasoner::entities::Class;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::TableauxReasoner;
use smallvec::smallvec;

#[test]
fn test_disjunction_with_clash_in_first_branch() {
    let mut ontology = Ontology::new();

    let class_a = Class::new("http://example.org/A");
    let class_b = Class::new("http://example.org/B");
    let class_c = Class::new("http://example.org/C");
    let class_d = Class::new("http://example.org/D");

    // Top ⊑ A ⊔ B
    let sub_class_axiom = SubClassOfAxiom::new(
        ClassExpression::Class(Class::new("http://www.w3.org/2002/07/owl#Thing")),
        ClassExpression::ObjectUnionOf(smallvec![
            Box::new(ClassExpression::Class(class_a.clone())),
            Box::new(ClassExpression::Class(class_b.clone())),
        ]),
    );
    ontology.add_subclass_axiom(sub_class_axiom).unwrap();

    // A ⊑ C
    let sub_class_axiom_1 = SubClassOfAxiom::new(
        ClassExpression::Class(class_a.clone()),
        ClassExpression::Class(class_c.clone()),
    );
    ontology.add_subclass_axiom(sub_class_axiom_1).unwrap();

    // A ⊑ D
    let sub_class_axiom_2 = SubClassOfAxiom::new(
        ClassExpression::Class(class_a.clone()),
        ClassExpression::Class(class_d.clone()),
    );
    ontology.add_subclass_axiom(sub_class_axiom_2).unwrap();

    // C and D are disjoint
    let disjoint_axiom = DisjointClassesAxiom::new(vec![class_c.iri().clone(), class_d.iri().clone()]);
    ontology.add_disjoint_classes_axiom(disjoint_axiom).unwrap();

    let mut reasoner = TableauxReasoner::new(ontology);
    assert!(reasoner.check_consistency().unwrap());
}

#[test]
fn test_disjunction_where_both_branches_are_consistent() {
    let mut ontology = Ontology::new();

    let class_a = Class::new("http://example.org/A");
    let class_b = Class::new("http://example.org/B");

    // Top ⊑ A ⊔ B
    let sub_class_axiom = SubClassOfAxiom::new(
        ClassExpression::Class(Class::new("http://www.w3.org/2002/07/owl#Thing")),
        ClassExpression::ObjectUnionOf(smallvec![
            Box::new(ClassExpression::Class(class_a.clone())),
            Box::new(ClassExpression::Class(class_b.clone())),
        ]),
    );
    ontology.add_subclass_axiom(sub_class_axiom).unwrap();

    let mut reasoner = TableauxReasoner::new(ontology);
    assert!(reasoner.check_consistency().unwrap());
}

#[test]
fn test_nested_disjunctions_with_backtracking() {
    let mut ontology = Ontology::new();

    let class_a = Class::new("http://example.org/A");
    let class_b = Class::new("http://example.org/B");
    let class_c = Class::new("http://example.org/C");
    let class_d = Class::new("http://example.org/D");
    let class_e = Class::new("http://example.org/E");
    let class_f = Class::new("http://example.org/F");

    // Top ⊑ (A ⊔ B) ⊓ (C ⊔ D)
    let sub_class_axiom = SubClassOfAxiom::new(
        ClassExpression::Class(Class::new("http://www.w3.org/2002/07/owl#Thing")),
        ClassExpression::ObjectIntersectionOf(smallvec![
            Box::new(ClassExpression::ObjectUnionOf(smallvec![
                Box::new(ClassExpression::Class(class_a.clone())),
                Box::new(ClassExpression::Class(class_b.clone())),
            ])),
            Box::new(ClassExpression::ObjectUnionOf(smallvec![
                Box::new(ClassExpression::Class(class_c.clone())),
                Box::new(ClassExpression::Class(class_d.clone())),
            ])),
        ]),
    );
    ontology.add_subclass_axiom(sub_class_axiom).unwrap();

    // A ⊑ E
    let sub_class_axiom_1 = SubClassOfAxiom::new(
        ClassExpression::Class(class_a.clone()),
        ClassExpression::Class(class_e.clone()),
    );
    ontology.add_subclass_axiom(sub_class_axiom_1).unwrap();

    // C ⊑ F
    let sub_class_axiom_2 = SubClassOfAxiom::new(
        ClassExpression::Class(class_c.clone()),
        ClassExpression::Class(class_f.clone()),
    );
    ontology.add_subclass_axiom(sub_class_axiom_2).unwrap();

    // E and F are disjoint
    let disjoint_axiom = DisjointClassesAxiom::new(vec![class_e.iri().clone(), class_f.iri().clone()]);
    ontology.add_disjoint_classes_axiom(disjoint_axiom).unwrap();

    // This should be consistent. The reasoner should be able to find a model, for example, by choosing B and D.
    // If it chooses A and C, it will lead to a clash (E and F are disjoint).
    let mut reasoner = TableauxReasoner::new(ontology);
    assert!(reasoner.check_consistency().unwrap());
}

#[test]
fn test_cardinality_and_disjunction_interaction() {
    use owl2_reasoner::axioms::property_expressions::ObjectPropertyExpression;
    use owl2_reasoner::entities::ObjectProperty;

    let mut ontology = Ontology::new();

    let class_a = Class::new("http://example.org/A");
    let class_b = Class::new("http://example.org/B");
    let prop_r = ObjectProperty::new("http://example.org/R");

    // A ⊔ B
    let sub_class_axiom = SubClassOfAxiom::new(
        ClassExpression::Class(Class::new("http://www.w3.org/2002/07/owl#Thing")),
        ClassExpression::ObjectUnionOf(smallvec![
            Box::new(ClassExpression::Class(class_a.clone())),
            Box::new(ClassExpression::Class(class_b.clone())),
        ]),
    );
    ontology.add_subclass_axiom(sub_class_axiom).unwrap();

    // A ⊑ <= 0 R
    let max_cardinality_axiom = SubClassOfAxiom::new(
        ClassExpression::Class(class_a.clone()),
        ClassExpression::ObjectMaxCardinality(0, Box::new(ObjectPropertyExpression::ObjectProperty(Box::new(prop_r.clone())))),
    );
    ontology.add_subclass_axiom(max_cardinality_axiom).unwrap();

    // B ⊑ >= 1 R
    let min_cardinality_axiom = SubClassOfAxiom::new(
        ClassExpression::Class(Class::new("http://www.w3.org/2002/07/owl#Thing")),
        ClassExpression::ObjectMinCardinality(1, Box::new(ObjectPropertyExpression::ObjectProperty(Box::new(prop_r.clone())))),
    );
    ontology.add_subclass_axiom(min_cardinality_axiom).unwrap();

    // This ontology is consistent. The reasoner must not choose A, because that would create a clash with the min cardinality of 1.
    // It should choose B.
    let mut reasoner = TableauxReasoner::new(ontology);
    assert!(reasoner.check_consistency().unwrap());
}
