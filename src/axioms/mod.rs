//! OWL2 Axioms - Logical statements about entities
//! 
//! This module defines all OWL2 axiom types that express logical relationships
//! between classes, properties, and individuals.

pub mod class_expressions;
pub mod property_expressions;

pub use class_expressions::*;
pub use property_expressions::*;

use crate::iri::IRI;

/// OWL2 Axiom types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Axiom {
    /// Subclass axiom: C ⊑ D
    SubClassOf(SubClassOfAxiom),
    /// Equivalent classes axiom: C ≡ D
    EquivalentClasses(EquivalentClassesAxiom),
    /// Disjoint classes axiom: C ⊓ D ⊑ ⊥
    DisjointClasses(DisjointClassesAxiom),
    /// Class assertion: a ∈ C
    ClassAssertion(ClassAssertionAxiom),
    /// Property assertion: (a, b) ∈ P
    PropertyAssertion(PropertyAssertionAxiom),
    /// Subproperty axiom: P ⊑ Q
    SubObjectProperty(SubObjectPropertyAxiom),
    /// Equivalent properties axiom: P ≡ Q
    EquivalentObjectProperties(EquivalentObjectPropertiesAxiom),
    /// Disjoint properties axiom: P ⊓ Q ⊑ ⊥
    DisjointObjectProperties(DisjointObjectPropertiesAxiom),
}

/// Subclass axiom: C ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubClassOfAxiom {
    sub_class: class_expressions::ClassExpression,
    super_class: class_expressions::ClassExpression,
}

impl SubClassOfAxiom {
    /// Create a new subclass axiom
    pub fn new(sub_class: class_expressions::ClassExpression, super_class: class_expressions::ClassExpression) -> Self {
        SubClassOfAxiom {
            sub_class,
            super_class,
        }
    }
    
    /// Get the subclass
    pub fn sub_class(&self) -> &class_expressions::ClassExpression {
        &self.sub_class
    }
    
    /// Get the superclass
    pub fn super_class(&self) -> &class_expressions::ClassExpression {
        &self.super_class
    }
    
    /// Check if this axiom involves a specific class
    pub fn involves_class(&self, class_iri: &IRI) -> bool {
        self.sub_class.contains_class(class_iri) || self.super_class.contains_class(class_iri)
    }
}

/// Equivalent classes axiom: C ≡ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquivalentClassesAxiom {
    classes: Vec<IRI>,
}

impl EquivalentClassesAxiom {
    /// Create a new equivalent classes axiom
    pub fn new(classes: Vec<IRI>) -> Self {
        EquivalentClassesAxiom { classes }
    }
    
    /// Get the equivalent classes
    pub fn classes(&self) -> &Vec<IRI> {
        &self.classes
    }
}

/// Disjoint classes axiom: C ⊓ D ⊑ ⊥
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisjointClassesAxiom {
    classes: Vec<IRI>,
}

impl DisjointClassesAxiom {
    /// Create a new disjoint classes axiom
    pub fn new(classes: Vec<IRI>) -> Self {
        DisjointClassesAxiom { classes }
    }
    
    /// Get the disjoint classes
    pub fn classes(&self) -> &Vec<IRI> {
        &self.classes
    }
}

/// Class assertion axiom: a ∈ C
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassAssertionAxiom {
    individual: IRI,
    class_expr: class_expressions::ClassExpression,
}

impl ClassAssertionAxiom {
    /// Create a new class assertion axiom
    pub fn new(individual: IRI, class_expr: class_expressions::ClassExpression) -> Self {
        ClassAssertionAxiom {
            individual,
            class_expr,
        }
    }
    
    /// Get the individual
    pub fn individual(&self) -> &IRI {
        &self.individual
    }
    
    /// Get the class expression
    pub fn class_expr(&self) -> &class_expressions::ClassExpression {
        &self.class_expr
    }
}

/// Property assertion axiom: (a, b) ∈ P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyAssertionAxiom {
    subject: IRI,
    property: IRI,
    object: IRI,
}

impl PropertyAssertionAxiom {
    /// Create a new property assertion axiom
    pub fn new(subject: IRI, property: IRI, object: IRI) -> Self {
        PropertyAssertionAxiom {
            subject,
            property,
            object,
        }
    }
    
    /// Get the subject
    pub fn subject(&self) -> &IRI {
        &self.subject
    }
    
    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }
    
    /// Get the object
    pub fn object(&self) -> &IRI {
        &self.object
    }
}

/// Subobject property axiom: P ⊑ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubObjectPropertyAxiom {
    sub_property: IRI,
    super_property: IRI,
}

impl SubObjectPropertyAxiom {
    /// Create a new subobject property axiom
    pub fn new(sub_property: IRI, super_property: IRI) -> Self {
        SubObjectPropertyAxiom {
            sub_property,
            super_property,
        }
    }
    
    /// Get the subproperty
    pub fn sub_property(&self) -> &IRI {
        &self.sub_property
    }
    
    /// Get the superproperty
    pub fn super_property(&self) -> &IRI {
        &self.super_property
    }
}

/// Equivalent object properties axiom: P ≡ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquivalentObjectPropertiesAxiom {
    properties: Vec<IRI>,
}

impl EquivalentObjectPropertiesAxiom {
    /// Create a new equivalent object properties axiom
    pub fn new(properties: Vec<IRI>) -> Self {
        EquivalentObjectPropertiesAxiom { properties }
    }
    
    /// Get the equivalent properties
    pub fn properties(&self) -> &Vec<IRI> {
        &self.properties
    }
}

/// Disjoint object properties axiom: P ⊓ Q ⊑ ⊥
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisjointObjectPropertiesAxiom {
    properties: Vec<IRI>,
}

impl DisjointObjectPropertiesAxiom {
    /// Create a new disjoint object properties axiom
    pub fn new(properties: Vec<IRI>) -> Self {
        DisjointObjectPropertiesAxiom { properties }
    }
    
    /// Get the disjoint properties
    pub fn properties(&self) -> &Vec<IRI> {
        &self.properties
    }
}

#[cfg(test)]
mod tests {
    use super::*;
        
    #[test]
    fn test_subclass_axiom() {
        let person_iri = IRI::new("http://example.org/Person").unwrap();
        let animal_iri = IRI::new("http://example.org/Animal").unwrap();
        
        let axiom = SubClassOfAxiom::new(
            class_expressions::ClassExpression::Class(crate::entities::Class::new(person_iri.clone())),
            class_expressions::ClassExpression::Class(crate::entities::Class::new(animal_iri.clone())),
        );
        
        assert_eq!(axiom.sub_class(), &class_expressions::ClassExpression::Class(crate::entities::Class::new(person_iri.clone())));
        assert_eq!(axiom.super_class(), &class_expressions::ClassExpression::Class(crate::entities::Class::new(animal_iri.clone())));
        assert!(axiom.involves_class(&person_iri));
        assert!(axiom.involves_class(&animal_iri));
    }
    
    #[test]
    fn test_equivalent_classes_axiom() {
        let person_iri = IRI::new("http://example.org/Person").unwrap();
        let human_iri = IRI::new("http://example.org/Human").unwrap();
        
        let axiom = EquivalentClassesAxiom::new(vec![
            person_iri.clone(),
            human_iri.clone(),
        ]);
        
        assert_eq!(axiom.classes().len(), 2);
        assert!(axiom.classes().contains(&person_iri));
        assert!(axiom.classes().contains(&human_iri));
    }
    
    #[test]
    fn test_class_assertion_axiom() {
        let person_iri = IRI::new("http://example.org/Person").unwrap();
        let john_iri = IRI::new("http://example.org/john").unwrap();
        
        let axiom = ClassAssertionAxiom::new(
            john_iri.clone(),
            class_expressions::ClassExpression::Class(crate::entities::Class::new(person_iri.clone())),
        );
        
        assert_eq!(axiom.individual(), &john_iri);
        assert_eq!(axiom.class_expr(), &class_expressions::ClassExpression::Class(crate::entities::Class::new(person_iri.clone())));
    }
    
    #[test]
    fn test_property_assertion_axiom() {
        let john_iri = IRI::new("http://example.org/john").unwrap();
        let hasParent_iri = IRI::new("http://example.org/hasParent").unwrap();
        let mary_iri = IRI::new("http://example.org/mary").unwrap();
        
        let axiom = PropertyAssertionAxiom::new(
            john_iri.clone(),
            hasParent_iri.clone(),
            mary_iri.clone(),
        );
        
        assert_eq!(axiom.subject(), &john_iri);
        assert_eq!(axiom.property(), &hasParent_iri);
        assert_eq!(axiom.object(), &mary_iri);
    }
}