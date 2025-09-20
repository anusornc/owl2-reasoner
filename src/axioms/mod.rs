//! OWL2 Axioms - Logical statements about entities
//!
//! This module defines all OWL2 axiom types that express logical relationships
//! between classes, properties, and individuals.

pub mod class_expressions;
pub mod property_expressions;

pub use crate::entities::ObjectProperty;
pub use class_expressions::*;
pub use property_expressions::*;

use crate::iri::IRI;

/// OWL2 Axiom type identifiers for indexing and classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AxiomType {
    SubClassOf,
    EquivalentClasses,
    DisjointClasses,
    ClassAssertion,
    PropertyAssertion,
    DataPropertyAssertion,
    SubObjectProperty,
    EquivalentObjectProperties,
    DisjointObjectProperties,
    FunctionalProperty,
    InverseFunctionalProperty,
    ReflexiveProperty,
    IrreflexiveProperty,
    SymmetricProperty,
    AsymmetricProperty,
    TransitiveProperty,
    SubPropertyChainOf,
    InverseObjectProperties,
    SubDataProperty,
    EquivalentDataProperties,
    DisjointDataProperties,
    FunctionalDataProperty,
    SameIndividual,
    DifferentIndividuals,
    HasKey,
    AnnotationAssertion,
    ObjectMinQualifiedCardinality,
    ObjectMaxQualifiedCardinality,
    ObjectExactQualifiedCardinality,
    DataMinQualifiedCardinality,
    DataMaxQualifiedCardinality,
    DataExactQualifiedCardinality,
}

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
    /// Data property assertion: (a, v) ∈ P where v is a literal
    DataPropertyAssertion(DataPropertyAssertionAxiom),
    /// Subproperty axiom: P ⊑ Q
    SubObjectProperty(SubObjectPropertyAxiom),
    /// Equivalent properties axiom: P ≡ Q
    EquivalentObjectProperties(EquivalentObjectPropertiesAxiom),
    /// Disjoint properties axiom: P ⊓ Q ⊑ ⊥
    DisjointObjectProperties(DisjointObjectPropertiesAxiom),
    /// Functional property axiom: ⊤ ⊑ ≤1P
    FunctionalProperty(FunctionalPropertyAxiom),
    /// Inverse functional property axiom: ⊤ ⊑ ≤1P⁻
    InverseFunctionalProperty(InverseFunctionalPropertyAxiom),
    /// Reflexive property axiom: ⊤ ⊑ ∃P.Self
    ReflexiveProperty(ReflexivePropertyAxiom),
    /// Irreflexive property axiom: ⊥ ⊑ ∃P.Self
    IrreflexiveProperty(IrreflexivePropertyAxiom),
    /// Symmetric property axiom: P ≡ P⁻
    SymmetricProperty(SymmetricPropertyAxiom),
    /// Asymmetric property axiom: P ⊓ P⁻ ⊑ ⊥
    AsymmetricProperty(AsymmetricPropertyAxiom),
    /// Transitive property axiom: P⁺ ⊑ P
    TransitiveProperty(TransitivePropertyAxiom),
    /// Property chain axiom: P₁ ∘ ... ∘ Pₙ ⊑ Q
    SubPropertyChainOf(SubPropertyChainOfAxiom),
    /// Inverse object properties axiom: P ≡ Q⁻
    InverseObjectProperties(InverseObjectPropertiesAxiom),
    /// Subdata property axiom: Q ⊑ P
    SubDataProperty(SubDataPropertyAxiom),
    /// Equivalent data properties axiom: P ≡ Q
    EquivalentDataProperties(EquivalentDataPropertiesAxiom),
    /// Disjoint data properties axiom: P ⊓ Q ⊑ ⊥
    DisjointDataProperties(DisjointDataPropertiesAxiom),
    /// Functional data property axiom: ⊤ ⊑ ≤1P
    FunctionalDataProperty(FunctionalDataPropertyAxiom),
    /// Same individual axiom: a = b
    SameIndividual(SameIndividualAxiom),
    /// Different individuals axiom: a ≠ b
    DifferentIndividuals(DifferentIndividualsAxiom),
    /// Has key axiom: P₁,...,Pₙ ⊑ Key(C)
    HasKey(HasKeyAxiom),
    /// Annotation assertion axiom: ⊤ ⊑ ∃r.{@a}
    AnnotationAssertion(AnnotationAssertionAxiom),
    /// Object minimum qualified cardinality: ⊤ ⊑ ≥n R.C
    ObjectMinQualifiedCardinality(ObjectMinQualifiedCardinalityAxiom),
    /// Object maximum qualified cardinality: ⊤ ⊑ ≤n R.C
    ObjectMaxQualifiedCardinality(ObjectMaxQualifiedCardinalityAxiom),
    /// Object exact qualified cardinality: ⊤ ⊑ =n R.C
    ObjectExactQualifiedCardinality(ObjectExactQualifiedCardinalityAxiom),
    /// Data minimum qualified cardinality: ⊤ ⊑ ≥n R.D
    DataMinQualifiedCardinality(DataMinQualifiedCardinalityAxiom),
    /// Data maximum qualified cardinality: ⊤ ⊑ ≤n R.D
    DataMaxQualifiedCardinality(DataMaxQualifiedCardinalityAxiom),
    /// Data exact qualified cardinality: ⊤ ⊑ =n R.D
    DataExactQualifiedCardinality(DataExactQualifiedCardinalityAxiom),
}

impl Axiom {
    /// Get the type of this axiom
    pub fn axiom_type(&self) -> AxiomType {
        match self {
            Axiom::SubClassOf(_) => AxiomType::SubClassOf,
            Axiom::EquivalentClasses(_) => AxiomType::EquivalentClasses,
            Axiom::DisjointClasses(_) => AxiomType::DisjointClasses,
            Axiom::ClassAssertion(_) => AxiomType::ClassAssertion,
            Axiom::PropertyAssertion(_) => AxiomType::PropertyAssertion,
            Axiom::DataPropertyAssertion(_) => AxiomType::DataPropertyAssertion,
            Axiom::SubObjectProperty(_) => AxiomType::SubObjectProperty,
            Axiom::EquivalentObjectProperties(_) => AxiomType::EquivalentObjectProperties,
            Axiom::DisjointObjectProperties(_) => AxiomType::DisjointObjectProperties,
            Axiom::FunctionalProperty(_) => AxiomType::FunctionalProperty,
            Axiom::InverseFunctionalProperty(_) => AxiomType::InverseFunctionalProperty,
            Axiom::ReflexiveProperty(_) => AxiomType::ReflexiveProperty,
            Axiom::IrreflexiveProperty(_) => AxiomType::IrreflexiveProperty,
            Axiom::SymmetricProperty(_) => AxiomType::SymmetricProperty,
            Axiom::AsymmetricProperty(_) => AxiomType::AsymmetricProperty,
            Axiom::TransitiveProperty(_) => AxiomType::TransitiveProperty,
            Axiom::SubPropertyChainOf(_) => AxiomType::SubPropertyChainOf,
            Axiom::InverseObjectProperties(_) => AxiomType::InverseObjectProperties,
            Axiom::SubDataProperty(_) => AxiomType::SubDataProperty,
            Axiom::EquivalentDataProperties(_) => AxiomType::EquivalentDataProperties,
            Axiom::DisjointDataProperties(_) => AxiomType::DisjointDataProperties,
            Axiom::FunctionalDataProperty(_) => AxiomType::FunctionalDataProperty,
            Axiom::SameIndividual(_) => AxiomType::SameIndividual,
            Axiom::DifferentIndividuals(_) => AxiomType::DifferentIndividuals,
            Axiom::HasKey(_) => AxiomType::HasKey,
            Axiom::AnnotationAssertion(_) => AxiomType::AnnotationAssertion,
            Axiom::ObjectMinQualifiedCardinality(_) => AxiomType::ObjectMinQualifiedCardinality,
            Axiom::ObjectMaxQualifiedCardinality(_) => AxiomType::ObjectMaxQualifiedCardinality,
            Axiom::ObjectExactQualifiedCardinality(_) => AxiomType::ObjectExactQualifiedCardinality,
            Axiom::DataMinQualifiedCardinality(_) => AxiomType::DataMinQualifiedCardinality,
            Axiom::DataMaxQualifiedCardinality(_) => AxiomType::DataMaxQualifiedCardinality,
            Axiom::DataExactQualifiedCardinality(_) => AxiomType::DataExactQualifiedCardinality,
        }
    }

    /// Get the signature IRIs of this axiom (main entities involved)
    pub fn signature(&self) -> Vec<IRI> {
        // Simplified signature extraction - will be enhanced with proper axiom methods
        vec![] // Placeholder implementation
    }
}

/// Subclass axiom: C ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubClassOfAxiom {
    sub_class: class_expressions::ClassExpression,
    super_class: class_expressions::ClassExpression,
}

impl SubClassOfAxiom {
    /// Create a new subclass axiom
    pub fn new(
        sub_class: class_expressions::ClassExpression,
        super_class: class_expressions::ClassExpression,
    ) -> Self {
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

/// Data property assertion axiom: (a, v) ∈ P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPropertyAssertionAxiom {
    subject: IRI,
    property: IRI,
    value: crate::entities::Literal,
}

impl DataPropertyAssertionAxiom {
    /// Create a new data property assertion axiom
    pub fn new(subject: IRI, property: IRI, value: crate::entities::Literal) -> Self {
        DataPropertyAssertionAxiom {
            subject,
            property,
            value,
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

    /// Get the literal value
    pub fn value(&self) -> &crate::entities::Literal {
        &self.value
    }
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

/// Functional property axiom: ⊤ ⊑ ≤1P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionalPropertyAxiom {
    property: IRI,
}

impl FunctionalPropertyAxiom {
    /// Create a new functional property axiom
    pub fn new(property: IRI) -> Self {
        FunctionalPropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }
}

/// Inverse functional property axiom: ⊤ ⊑ ≤1P⁻
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InverseFunctionalPropertyAxiom {
    property: IRI,
}

impl InverseFunctionalPropertyAxiom {
    /// Create a new inverse functional property axiom
    pub fn new(property: IRI) -> Self {
        InverseFunctionalPropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }
}

/// Reflexive property axiom: ⊤ ⊑ ∃P.Self
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflexivePropertyAxiom {
    property: IRI,
}

impl ReflexivePropertyAxiom {
    /// Create a new reflexive property axiom
    pub fn new(property: IRI) -> Self {
        ReflexivePropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }
}

/// Irreflexive property axiom: ⊥ ⊑ ∃P.Self
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrreflexivePropertyAxiom {
    property: IRI,
}

impl IrreflexivePropertyAxiom {
    /// Create a new irreflexive property axiom
    pub fn new(property: IRI) -> Self {
        IrreflexivePropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }
}

/// Symmetric property axiom: P ≡ P⁻
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymmetricPropertyAxiom {
    property: IRI,
}

impl SymmetricPropertyAxiom {
    /// Create a new symmetric property axiom
    pub fn new(property: IRI) -> Self {
        SymmetricPropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }
}

/// Asymmetric property axiom: P ⊓ P⁻ ⊑ ⊥
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsymmetricPropertyAxiom {
    property: IRI,
}

impl AsymmetricPropertyAxiom {
    /// Create a new asymmetric property axiom
    pub fn new(property: IRI) -> Self {
        AsymmetricPropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }
}

/// Transitive property axiom: P⁺ ⊑ P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitivePropertyAxiom {
    property: IRI,
}

impl TransitivePropertyAxiom {
    /// Create a new transitive property axiom
    pub fn new(property: IRI) -> Self {
        TransitivePropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }
}

/// Property chain axiom: P₁ ∘ ... ∘ Pₙ ⊑ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubPropertyChainOfAxiom {
    property_chain: Vec<ObjectPropertyExpression>,
    super_property: ObjectPropertyExpression,
}

impl SubPropertyChainOfAxiom {
    /// Create a new property chain axiom
    pub fn new(
        property_chain: Vec<ObjectPropertyExpression>,
        super_property: ObjectPropertyExpression,
    ) -> Self {
        SubPropertyChainOfAxiom {
            property_chain,
            super_property,
        }
    }

    /// Get the property chain
    pub fn property_chain(&self) -> &[ObjectPropertyExpression] {
        &self.property_chain
    }

    /// Get the super property
    pub fn super_property(&self) -> &ObjectPropertyExpression {
        &self.super_property
    }
}

/// Inverse object properties axiom: P ≡ Q⁻
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InverseObjectPropertiesAxiom {
    property1: ObjectPropertyExpression,
    property2: ObjectPropertyExpression,
}

impl InverseObjectPropertiesAxiom {
    /// Create a new inverse object properties axiom
    pub fn new(property1: ObjectPropertyExpression, property2: ObjectPropertyExpression) -> Self {
        InverseObjectPropertiesAxiom {
            property1,
            property2,
        }
    }

    /// Get the first property
    pub fn property1(&self) -> &ObjectPropertyExpression {
        &self.property1
    }

    /// Get the second property
    pub fn property2(&self) -> &ObjectPropertyExpression {
        &self.property2
    }
}

/// Subdata property axiom: Q ⊑ P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubDataPropertyAxiom {
    sub_property: IRI,
    super_property: IRI,
}

impl SubDataPropertyAxiom {
    /// Create a new subdata property axiom
    pub fn new(sub_property: IRI, super_property: IRI) -> Self {
        SubDataPropertyAxiom {
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

/// Equivalent data properties axiom: P ≡ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquivalentDataPropertiesAxiom {
    properties: Vec<IRI>,
}

impl EquivalentDataPropertiesAxiom {
    /// Create a new equivalent data properties axiom
    pub fn new(properties: Vec<IRI>) -> Self {
        EquivalentDataPropertiesAxiom { properties }
    }

    /// Get the equivalent properties
    pub fn properties(&self) -> &Vec<IRI> {
        &self.properties
    }
}

/// Disjoint data properties axiom: P ⊓ Q ⊑ ⊥
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisjointDataPropertiesAxiom {
    properties: Vec<IRI>,
}

impl DisjointDataPropertiesAxiom {
    /// Create a new disjoint data properties axiom
    pub fn new(properties: Vec<IRI>) -> Self {
        DisjointDataPropertiesAxiom { properties }
    }

    /// Get the disjoint properties
    pub fn properties(&self) -> &Vec<IRI> {
        &self.properties
    }
}

/// Functional data property axiom: ⊤ ⊑ ≤1P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionalDataPropertyAxiom {
    property: IRI,
}

impl FunctionalDataPropertyAxiom {
    /// Create a new functional data property axiom
    pub fn new(property: IRI) -> Self {
        FunctionalDataPropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }
}

/// Same individual axiom: a = b
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SameIndividualAxiom {
    individuals: Vec<IRI>,
}

impl SameIndividualAxiom {
    /// Create a new same individual axiom
    pub fn new(individuals: Vec<IRI>) -> Self {
        SameIndividualAxiom { individuals }
    }

    /// Get the individuals
    pub fn individuals(&self) -> &[IRI] {
        &self.individuals
    }
}

/// Different individuals axiom: a ≠ b
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferentIndividualsAxiom {
    individuals: Vec<IRI>,
}

impl DifferentIndividualsAxiom {
    /// Create a new different individuals axiom
    pub fn new(individuals: Vec<IRI>) -> Self {
        DifferentIndividualsAxiom { individuals }
    }

    /// Get the individuals
    pub fn individuals(&self) -> &[IRI] {
        &self.individuals
    }
}

/// Has key axiom: P₁,...,Pₙ ⊑ Key(C)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HasKeyAxiom {
    class_expression: class_expressions::ClassExpression,
    properties: Vec<IRI>,
}

impl HasKeyAxiom {
    /// Create a new has key axiom
    pub fn new(class_expression: class_expressions::ClassExpression, properties: Vec<IRI>) -> Self {
        HasKeyAxiom {
            class_expression,
            properties,
        }
    }

    /// Get the class expression
    pub fn class_expression(&self) -> &class_expressions::ClassExpression {
        &self.class_expression
    }

    /// Get the properties
    pub fn properties(&self) -> &[IRI] {
        &self.properties
    }
}

/// Annotation assertion axiom: ⊤ ⊑ ∃r.{@a}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationAssertionAxiom {
    annotation_property: IRI,
    subject: IRI,
    value: crate::entities::AnnotationValue,
}

impl AnnotationAssertionAxiom {
    /// Create a new annotation assertion axiom
    pub fn new(
        annotation_property: IRI,
        subject: IRI,
        value: crate::entities::AnnotationValue,
    ) -> Self {
        AnnotationAssertionAxiom {
            annotation_property,
            subject,
            value,
        }
    }

    /// Get the annotation property
    pub fn annotation_property(&self) -> &IRI {
        &self.annotation_property
    }

    /// Get the subject
    pub fn subject(&self) -> &IRI {
        &self.subject
    }

    /// Get the annotation value
    pub fn value(&self) -> &crate::entities::AnnotationValue {
        &self.value
    }
}

/// Object minimum qualified cardinality axiom: ⊤ ⊑ ≥n R.C
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectMinQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: class_expressions::ClassExpression,
}

impl ObjectMinQualifiedCardinalityAxiom {
    /// Create a new object minimum qualified cardinality axiom
    pub fn new(
        cardinality: u32,
        property: ObjectPropertyExpression,
        filler: class_expressions::ClassExpression,
    ) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler class expression
    pub fn filler(&self) -> &class_expressions::ClassExpression {
        &self.filler
    }
}

/// Object maximum qualified cardinality axiom: ⊤ ⊑ ≤n R.C
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectMaxQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: class_expressions::ClassExpression,
}

impl ObjectMaxQualifiedCardinalityAxiom {
    /// Create a new object maximum qualified cardinality axiom
    pub fn new(
        cardinality: u32,
        property: ObjectPropertyExpression,
        filler: class_expressions::ClassExpression,
    ) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler class expression
    pub fn filler(&self) -> &class_expressions::ClassExpression {
        &self.filler
    }
}

/// Object exact qualified cardinality axiom: ⊤ ⊑ =n R.C
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectExactQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: class_expressions::ClassExpression,
}

impl ObjectExactQualifiedCardinalityAxiom {
    /// Create a new object exact qualified cardinality axiom
    pub fn new(
        cardinality: u32,
        property: ObjectPropertyExpression,
        filler: class_expressions::ClassExpression,
    ) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler class expression
    pub fn filler(&self) -> &class_expressions::ClassExpression {
        &self.filler
    }
}

/// Data minimum qualified cardinality axiom: ⊤ ⊑ ≥n R.D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataMinQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: IRI,
}

impl DataMinQualifiedCardinalityAxiom {
    /// Create a new data minimum qualified cardinality axiom
    pub fn new(cardinality: u32, property: ObjectPropertyExpression, filler: IRI) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler datatype IRI
    pub fn filler(&self) -> &IRI {
        &self.filler
    }
}

/// Data maximum qualified cardinality axiom: ⊤ ⊑ ≤n R.D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataMaxQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: IRI,
}

impl DataMaxQualifiedCardinalityAxiom {
    /// Create a new data maximum qualified cardinality axiom
    pub fn new(cardinality: u32, property: ObjectPropertyExpression, filler: IRI) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler datatype IRI
    pub fn filler(&self) -> &IRI {
        &self.filler
    }
}

/// Data exact qualified cardinality axiom: ⊤ ⊑ =n R.D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataExactQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: IRI,
}

impl DataExactQualifiedCardinalityAxiom {
    /// Create a new data exact qualified cardinality axiom
    pub fn new(cardinality: u32, property: ObjectPropertyExpression, filler: IRI) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler datatype IRI
    pub fn filler(&self) -> &IRI {
        &self.filler
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
            class_expressions::ClassExpression::Class(crate::entities::Class::new(
                person_iri.clone(),
            )),
            class_expressions::ClassExpression::Class(crate::entities::Class::new(
                animal_iri.clone(),
            )),
        );

        assert_eq!(
            axiom.sub_class(),
            &class_expressions::ClassExpression::Class(crate::entities::Class::new(
                person_iri.clone()
            ))
        );
        assert_eq!(
            axiom.super_class(),
            &class_expressions::ClassExpression::Class(crate::entities::Class::new(
                animal_iri.clone()
            ))
        );
        assert!(axiom.involves_class(&person_iri));
        assert!(axiom.involves_class(&animal_iri));
    }

    #[test]
    fn test_equivalent_classes_axiom() {
        let person_iri = IRI::new("http://example.org/Person").unwrap();
        let human_iri = IRI::new("http://example.org/Human").unwrap();

        let axiom = EquivalentClassesAxiom::new(vec![person_iri.clone(), human_iri.clone()]);

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
            class_expressions::ClassExpression::Class(crate::entities::Class::new(
                person_iri.clone(),
            )),
        );

        assert_eq!(axiom.individual(), &john_iri);
        assert_eq!(
            axiom.class_expr(),
            &class_expressions::ClassExpression::Class(crate::entities::Class::new(
                person_iri.clone()
            ))
        );
    }

    #[test]
    fn test_property_assertion_axiom() {
        let john_iri = IRI::new("http://example.org/john").unwrap();
        let hasParent_iri = IRI::new("http://example.org/hasParent").unwrap();
        let mary_iri = IRI::new("http://example.org/mary").unwrap();

        let axiom =
            PropertyAssertionAxiom::new(john_iri.clone(), hasParent_iri.clone(), mary_iri.clone());

        assert_eq!(axiom.subject(), &john_iri);
        assert_eq!(axiom.property(), &hasParent_iri);
        assert_eq!(axiom.object(), &mary_iri);
    }

    // Tests for property characteristic axioms
    #[test]
    fn test_functional_property_axiom() {
        let hasFather_iri = IRI::new("http://example.org/hasFather").unwrap();
        let axiom = FunctionalPropertyAxiom::new(hasFather_iri.clone());

        assert_eq!(axiom.property(), &hasFather_iri);
    }

    #[test]
    fn test_inverse_functional_property_axiom() {
        let hasSSN_iri = IRI::new("http://example.org/hasSSN").unwrap();
        let axiom = InverseFunctionalPropertyAxiom::new(hasSSN_iri.clone());

        assert_eq!(axiom.property(), &hasSSN_iri);
    }

    #[test]
    fn test_reflexive_property_axiom() {
        let knows_iri = IRI::new("http://example.org/knows").unwrap();
        let axiom = ReflexivePropertyAxiom::new(knows_iri.clone());

        assert_eq!(axiom.property(), &knows_iri);
    }

    #[test]
    fn test_irreflexive_property_axiom() {
        let parentOf_iri = IRI::new("http://example.org/parentOf").unwrap();
        let axiom = IrreflexivePropertyAxiom::new(parentOf_iri.clone());

        assert_eq!(axiom.property(), &parentOf_iri);
    }

    #[test]
    fn test_symmetric_property_axiom() {
        let spouse_iri = IRI::new("http://example.org/spouse").unwrap();
        let axiom = SymmetricPropertyAxiom::new(spouse_iri.clone());

        assert_eq!(axiom.property(), &spouse_iri);
    }

    #[test]
    fn test_asymmetric_property_axiom() {
        let parentOf_iri = IRI::new("http://example.org/parentOf").unwrap();
        let axiom = AsymmetricPropertyAxiom::new(parentOf_iri.clone());

        assert_eq!(axiom.property(), &parentOf_iri);
    }

    #[test]
    fn test_transitive_property_axiom() {
        let ancestor_iri = IRI::new("http://example.org/ancestor").unwrap();
        let axiom = TransitivePropertyAxiom::new(ancestor_iri.clone());

        assert_eq!(axiom.property(), &ancestor_iri);
    }

    #[test]
    fn test_axiom_enum_property_characteristics() {
        let hasFather_iri = IRI::new("http://example.org/hasFather").unwrap();
        let knows_iri = IRI::new("http://example.org/knows").unwrap();
        let ancestor_iri = IRI::new("http://example.org/ancestor").unwrap();

        let functional_axiom =
            Axiom::FunctionalProperty(FunctionalPropertyAxiom::new(hasFather_iri.clone()));
        let reflexive_axiom =
            Axiom::ReflexiveProperty(ReflexivePropertyAxiom::new(knows_iri.clone()));
        let transitive_axiom =
            Axiom::TransitiveProperty(TransitivePropertyAxiom::new(ancestor_iri.clone()));

        // Test that axioms can be created and matched
        match functional_axiom {
            Axiom::FunctionalProperty(_) => assert!(true),
            _ => assert!(false, "Expected FunctionalProperty axiom"),
        }

        match reflexive_axiom {
            Axiom::ReflexiveProperty(_) => assert!(true),
            _ => assert!(false, "Expected ReflexiveProperty axiom"),
        }

        match transitive_axiom {
            Axiom::TransitiveProperty(_) => assert!(true),
            _ => assert!(false, "Expected TransitiveProperty axiom"),
        }
    }

    // Tests for data property axioms
    #[test]
    fn test_sub_data_property_axiom() {
        let has_age_iri = IRI::new("http://example.org/hasAge").unwrap();
        let has_height_iri = IRI::new("http://example.org/hasHeight").unwrap();
        let axiom = SubDataPropertyAxiom::new(has_age_iri.clone(), has_height_iri.clone());

        assert_eq!(axiom.sub_property(), &has_age_iri);
        assert_eq!(axiom.super_property(), &has_height_iri);
    }

    #[test]
    fn test_equivalent_data_properties_axiom() {
        let has_age_iri = IRI::new("http://example.org/hasAge").unwrap();
        let age_in_years_iri = IRI::new("http://example.org/ageInYears").unwrap();

        let axiom =
            EquivalentDataPropertiesAxiom::new(vec![has_age_iri.clone(), age_in_years_iri.clone()]);

        assert_eq!(axiom.properties().len(), 2);
        assert!(axiom.properties().contains(&has_age_iri));
        assert!(axiom.properties().contains(&age_in_years_iri));
    }

    #[test]
    fn test_disjoint_data_properties_axiom() {
        let has_age_iri = IRI::new("http://example.org/hasAge").unwrap();
        let has_weight_iri = IRI::new("http://example.org/hasWeight").unwrap();

        let axiom =
            DisjointDataPropertiesAxiom::new(vec![has_age_iri.clone(), has_weight_iri.clone()]);

        assert_eq!(axiom.properties().len(), 2);
        assert!(axiom.properties().contains(&has_age_iri));
        assert!(axiom.properties().contains(&has_weight_iri));
    }

    #[test]
    fn test_functional_data_property_axiom() {
        let has_birth_date_iri = IRI::new("http://example.org/hasBirthDate").unwrap();
        let axiom = FunctionalDataPropertyAxiom::new(has_birth_date_iri.clone());

        assert_eq!(axiom.property(), &has_birth_date_iri);
    }

    #[test]
    fn test_axiom_enum_data_properties() {
        let has_age_iri = IRI::new("http://example.org/hasAge").unwrap();
        let has_birth_date_iri = IRI::new("http://example.org/hasBirthDate").unwrap();
        let height_iri = IRI::new("http://example.org/height").unwrap();
        let weight_iri = IRI::new("http://example.org/weight").unwrap();

        let sub_data_axiom = Axiom::SubDataProperty(SubDataPropertyAxiom::new(
            has_age_iri.clone(),
            height_iri.clone(),
        ));
        let functional_data_axiom = Axiom::FunctionalDataProperty(
            FunctionalDataPropertyAxiom::new(has_birth_date_iri.clone()),
        );
        let equivalent_data_axiom =
            Axiom::EquivalentDataProperties(EquivalentDataPropertiesAxiom::new(vec![
                has_age_iri.clone(),
                height_iri.clone(),
            ]));
        let disjoint_data_axiom =
            Axiom::DisjointDataProperties(DisjointDataPropertiesAxiom::new(vec![
                has_age_iri.clone(),
                weight_iri.clone(),
            ]));

        // Test that axioms can be created and matched
        match sub_data_axiom {
            Axiom::SubDataProperty(_) => assert!(true),
            _ => assert!(false, "Expected SubDataProperty axiom"),
        }

        match functional_data_axiom {
            Axiom::FunctionalDataProperty(_) => assert!(true),
            _ => assert!(false, "Expected FunctionalDataProperty axiom"),
        }

        match equivalent_data_axiom {
            Axiom::EquivalentDataProperties(_) => assert!(true),
            _ => assert!(false, "Expected EquivalentDataProperties axiom"),
        }

        match disjoint_data_axiom {
            Axiom::DisjointDataProperties(_) => assert!(true),
            _ => assert!(false, "Expected DisjointDataProperties axiom"),
        }
    }

    #[test]
    fn test_same_individual_axiom() {
        let individual1 = IRI::new("http://example.org/individual1").unwrap();
        let individual2 = IRI::new("http://example.org/individual2").unwrap();

        let axiom = SameIndividualAxiom::new(vec![individual1.clone(), individual2.clone()]);

        assert_eq!(axiom.individuals().len(), 2);
        assert_eq!(axiom.individuals()[0], individual1);
        assert_eq!(axiom.individuals()[1], individual2);
    }

    #[test]
    fn test_different_individuals_axiom() {
        let individual1 = IRI::new("http://example.org/individual1").unwrap();
        let individual2 = IRI::new("http://example.org/individual2").unwrap();
        let individual3 = IRI::new("http://example.org/individual3").unwrap();

        let axiom = DifferentIndividualsAxiom::new(vec![
            individual1.clone(),
            individual2.clone(),
            individual3.clone(),
        ]);

        assert_eq!(axiom.individuals().len(), 3);
        assert_eq!(axiom.individuals()[0], individual1);
        assert_eq!(axiom.individuals()[1], individual2);
        assert_eq!(axiom.individuals()[2], individual3);
    }

    #[test]
    fn test_individual_axioms_in_enum() {
        let individual1 = IRI::new("http://example.org/individual1").unwrap();
        let individual2 = IRI::new("http://example.org/individual2").unwrap();

        let same_axiom = Axiom::SameIndividual(SameIndividualAxiom::new(vec![
            individual1.clone(),
            individual2.clone(),
        ]));
        let different_axiom = Axiom::DifferentIndividuals(DifferentIndividualsAxiom::new(vec![
            individual1.clone(),
            individual2.clone(),
        ]));

        match same_axiom {
            Axiom::SameIndividual(_) => assert!(true),
            _ => assert!(false, "Expected SameIndividual axiom"),
        }

        match different_axiom {
            Axiom::DifferentIndividuals(_) => assert!(true),
            _ => assert!(false, "Expected DifferentIndividuals axiom"),
        }
    }

    #[test]
    fn test_sub_property_chain_of_axiom() {
        let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
        let has_grandparent_iri = IRI::new("http://example.org/hasGrandparent").unwrap();

        let has_parent =
            ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(has_parent_iri.clone()));
        let has_grandparent = ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(
            has_grandparent_iri.clone(),
        ));

        let axiom = SubPropertyChainOfAxiom::new(
            vec![has_parent.clone(), has_parent.clone()],
            has_grandparent.clone(),
        );

        assert_eq!(axiom.property_chain().len(), 2);
        assert_eq!(axiom.property_chain()[0], has_parent);
        assert_eq!(axiom.property_chain()[1], has_parent);
        assert_eq!(axiom.super_property(), &has_grandparent);
    }

    #[test]
    fn test_inverse_object_properties_axiom() {
        let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
        let has_child_iri = IRI::new("http://example.org/hasChild").unwrap();

        let has_parent =
            ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(has_parent_iri.clone()));
        let has_child =
            ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(has_child_iri.clone()));

        let axiom = InverseObjectPropertiesAxiom::new(has_parent.clone(), has_child.clone());

        assert_eq!(axiom.property1(), &has_parent);
        assert_eq!(axiom.property2(), &has_child);
    }

    #[test]
    fn test_property_chain_with_inverse() {
        let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
        let has_child_iri = IRI::new("http://example.org/hasChild").unwrap();

        let has_parent =
            ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(has_parent_iri.clone()));
        let has_child_inverse = ObjectPropertyExpression::ObjectInverseOf(Box::new(
            ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(has_child_iri.clone())),
        ));

        let axiom =
            SubPropertyChainOfAxiom::new(vec![has_child_inverse.clone()], has_parent.clone());

        assert_eq!(axiom.property_chain().len(), 1);
        assert_eq!(axiom.property_chain()[0], has_child_inverse);
        assert_eq!(axiom.super_property(), &has_parent);
    }

    #[test]
    fn test_property_chain_axioms_in_enum() {
        let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
        let has_grandparent_iri = IRI::new("http://example.org/hasGrandparent").unwrap();

        let has_parent =
            ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(has_parent_iri.clone()));
        let has_grandparent = ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(
            has_grandparent_iri.clone(),
        ));

        let chain_axiom = Axiom::SubPropertyChainOf(SubPropertyChainOfAxiom::new(
            vec![has_parent.clone()],
            has_grandparent.clone(),
        ));

        match chain_axiom {
            Axiom::SubPropertyChainOf(_) => assert!(true),
            _ => assert!(false, "Expected SubPropertyChainOf axiom"),
        }
    }

    #[test]
    fn test_inverse_properties_axioms_in_enum() {
        let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
        let has_child_iri = IRI::new("http://example.org/hasChild").unwrap();

        let has_parent =
            ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(has_parent_iri.clone()));
        let has_child =
            ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(has_child_iri.clone()));

        let inverse_axiom = Axiom::InverseObjectProperties(InverseObjectPropertiesAxiom::new(
            has_parent.clone(),
            has_child.clone(),
        ));

        match inverse_axiom {
            Axiom::InverseObjectProperties(_) => assert!(true),
            _ => assert!(false, "Expected InverseObjectProperties axiom"),
        }
    }
}
