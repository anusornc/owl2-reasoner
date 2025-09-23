//! OWL2 Axioms - Logical statements about entities
//!
//! This module defines all OWL2 axiom types that express logical relationships
//! between classes, properties, and individuals.

pub mod class_expressions;
pub mod property_expressions;

pub use crate::entities::{Annotation, AnonymousIndividual, Literal, ObjectProperty};
pub use class_expressions::*;
pub use property_expressions::*;

use crate::iri::IRI;
use crate::{constants::*, OwlError, OwlResult};

/// Helper function to create IRIs safely with proper error handling
fn create_iri_safe(iri_str: &str) -> OwlResult<IRI> {
    IRI::new(iri_str).map_err(|_| OwlError::IriCreationError {
        iri_str: iri_str.to_string(),
    })
}

/// Helper function to create blank node IRIs safely
fn create_blank_node_iri(node_id: &str) -> OwlResult<IRI> {
    create_iri_safe(&format!("{}{}", BLANK_NODE_PREFIX, node_id))
}

/// Object value for property assertions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropertyAssertionObject {
    /// Named individual (IRI)
    Named(IRI),
    /// Anonymous individual (blank node)
    Anonymous(Box<AnonymousIndividual>),
}

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
    SubAnnotationPropertyOf,
    AnnotationPropertyDomain,
    AnnotationPropertyRange,
    ObjectMinQualifiedCardinality,
    ObjectMaxQualifiedCardinality,
    ObjectExactQualifiedCardinality,
    DataMinQualifiedCardinality,
    DataMaxQualifiedCardinality,
    DataExactQualifiedCardinality,
    ObjectPropertyDomain,
    ObjectPropertyRange,
    DataPropertyDomain,
    DataPropertyRange,
    NegativeObjectPropertyAssertion,
    NegativeDataPropertyAssertion,
    Import,
    Collection,
    Container,
    Reification,
}

/// OWL2 Axiom types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Axiom {
    /// Subclass axiom: C ⊑ D
    SubClassOf(Box<SubClassOfAxiom>),
    /// Equivalent classes axiom: C ≡ D
    EquivalentClasses(Box<EquivalentClassesAxiom>),
    /// Disjoint classes axiom: C ⊓ D ⊑ ⊥
    DisjointClasses(Box<DisjointClassesAxiom>),
    /// Class assertion: a ∈ C
    ClassAssertion(Box<ClassAssertionAxiom>),
    /// Property assertion: (a, b) ∈ P
    PropertyAssertion(Box<PropertyAssertionAxiom>),
    /// Data property assertion: (a, v) ∈ P where v is a literal
    DataPropertyAssertion(Box<DataPropertyAssertionAxiom>),
    /// Subproperty axiom: P ⊑ Q
    SubObjectProperty(Box<SubObjectPropertyAxiom>),
    /// Equivalent properties axiom: P ≡ Q
    EquivalentObjectProperties(Box<EquivalentObjectPropertiesAxiom>),
    /// Disjoint properties axiom: P ⊓ Q ⊑ ⊥
    DisjointObjectProperties(Box<DisjointObjectPropertiesAxiom>),
    /// Functional property axiom: ⊤ ⊑ ≤1P
    FunctionalProperty(Box<FunctionalPropertyAxiom>),
    /// Inverse functional property axiom: ⊤ ⊑ ≤1P⁻
    InverseFunctionalProperty(Box<InverseFunctionalPropertyAxiom>),
    /// Reflexive property axiom: ⊤ ⊑ ∃P.Self
    ReflexiveProperty(Box<ReflexivePropertyAxiom>),
    /// Irreflexive property axiom: ⊥ ⊑ ∃P.Self
    IrreflexiveProperty(Box<IrreflexivePropertyAxiom>),
    /// Symmetric property axiom: P ≡ P⁻
    SymmetricProperty(Box<SymmetricPropertyAxiom>),
    /// Asymmetric property axiom: P ⊓ P⁻ ⊑ ⊥
    AsymmetricProperty(Box<AsymmetricPropertyAxiom>),
    /// Transitive property axiom: P⁺ ⊑ P
    TransitiveProperty(Box<TransitivePropertyAxiom>),
    /// Property chain axiom: P₁ ∘ ... ∘ Pₙ ⊑ Q
    SubPropertyChainOf(Box<SubPropertyChainOfAxiom>),
    /// Inverse object properties axiom: P ≡ Q⁻
    InverseObjectProperties(Box<InverseObjectPropertiesAxiom>),
    /// Subdata property axiom: Q ⊑ P
    SubDataProperty(Box<SubDataPropertyAxiom>),
    /// Equivalent data properties axiom: P ≡ Q
    EquivalentDataProperties(Box<EquivalentDataPropertiesAxiom>),
    /// Disjoint data properties axiom: P ⊓ Q ⊑ ⊥
    DisjointDataProperties(Box<DisjointDataPropertiesAxiom>),
    /// Functional data property axiom: ⊤ ⊑ ≤1P
    FunctionalDataProperty(FunctionalDataPropertyAxiom),
    /// Same individual axiom: a = b
    SameIndividual(Box<SameIndividualAxiom>),
    /// Different individuals axiom: a ≠ b
    DifferentIndividuals(Box<DifferentIndividualsAxiom>),
    /// Has key axiom: P₁,...,Pₙ ⊑ Key(C)
    HasKey(Box<HasKeyAxiom>),
    /// Annotation assertion axiom: ⊤ ⊑ ∃r.{@a}
    AnnotationAssertion(Box<AnnotationAssertionAxiom>),
    /// Sub-annotation property axiom: P ⊑ Q
    SubAnnotationPropertyOf(SubAnnotationPropertyOfAxiom),
    /// Annotation property domain axiom: ∀P.C ⊑ D
    AnnotationPropertyDomain(AnnotationPropertyDomainAxiom),
    /// Annotation property range axiom: ∀P.C ⊑ D
    AnnotationPropertyRange(AnnotationPropertyRangeAxiom),
    /// Object minimum qualified cardinality: ⊤ ⊑ ≥n R.C
    ObjectMinQualifiedCardinality(Box<ObjectMinQualifiedCardinalityAxiom>),
    /// Object maximum qualified cardinality: ⊤ ⊑ ≤n R.C
    ObjectMaxQualifiedCardinality(Box<ObjectMaxQualifiedCardinalityAxiom>),
    /// Object exact qualified cardinality: ⊤ ⊑ =n R.C
    ObjectExactQualifiedCardinality(Box<ObjectExactQualifiedCardinalityAxiom>),
    /// Data minimum qualified cardinality: ⊤ ⊑ ≥n R.D
    DataMinQualifiedCardinality(Box<DataMinQualifiedCardinalityAxiom>),
    /// Data maximum qualified cardinality: ⊤ ⊑ ≤n R.D
    DataMaxQualifiedCardinality(Box<DataMaxQualifiedCardinalityAxiom>),
    /// Data exact qualified cardinality: ⊤ ⊑ =n R.D
    DataExactQualifiedCardinality(Box<DataExactQualifiedCardinalityAxiom>),
    /// Object property domain: ∀P.C ⊑ D
    ObjectPropertyDomain(Box<ObjectPropertyDomainAxiom>),
    /// Object property range: ∀P.D ⊑ C
    ObjectPropertyRange(Box<ObjectPropertyRangeAxiom>),
    /// Data property domain: ∀Q.C ⊑ D
    DataPropertyDomain(Box<DataPropertyDomainAxiom>),
    /// Data property range: ∃Q.l ⊑ D
    DataPropertyRange(Box<DataPropertyRangeAxiom>),
    /// Negative object property assertion: (a, b) ∉ P
    NegativeObjectPropertyAssertion(Box<NegativeObjectPropertyAssertionAxiom>),
    /// Negative data property assertion: (a, l) ∉ Q
    NegativeDataPropertyAssertion(Box<NegativeDataPropertyAssertionAxiom>),
    /// Import declaration: imports ontology with given IRI
    Import(ImportAxiom),
    /// RDF Collection axiom: represents ordered list using rdf:first, rdf:rest, rdf:nil
    Collection(Box<CollectionAxiom>),
    /// RDF Container axiom: represents Seq, Bag, or Alt containers
    Container(Box<ContainerAxiom>),
    /// RDF Reification axiom: represents statements about statements
    Reification(Box<ReificationAxiom>),
}

impl Axiom {
    /// Get the type of this axiom
    pub fn axiom_type(&self) -> AxiomType {
        // Macro to map axiom variants to their corresponding types
        macro_rules! axiom_type_map {
            ($($variant:ident => $type:ident),*) => {
                match self {
                    $(Axiom::$variant(_) => AxiomType::$type),*
                }
            };
        }

        axiom_type_map! {
            // Class expression axioms
            SubClassOf => SubClassOf,
            EquivalentClasses => EquivalentClasses,
            DisjointClasses => DisjointClasses,
            ClassAssertion => ClassAssertion,

            // Property assertion axioms
            PropertyAssertion => PropertyAssertion,
            DataPropertyAssertion => DataPropertyAssertion,

            // Object property axioms
            SubObjectProperty => SubObjectProperty,
            EquivalentObjectProperties => EquivalentObjectProperties,
            DisjointObjectProperties => DisjointObjectProperties,
            FunctionalProperty => FunctionalProperty,
            InverseFunctionalProperty => InverseFunctionalProperty,
            ReflexiveProperty => ReflexiveProperty,
            IrreflexiveProperty => IrreflexiveProperty,
            SymmetricProperty => SymmetricProperty,
            AsymmetricProperty => AsymmetricProperty,
            TransitiveProperty => TransitiveProperty,
            SubPropertyChainOf => SubPropertyChainOf,
            InverseObjectProperties => InverseObjectProperties,
            ObjectPropertyDomain => ObjectPropertyDomain,
            ObjectPropertyRange => ObjectPropertyRange,
            NegativeObjectPropertyAssertion => NegativeObjectPropertyAssertion,

            // Data property axioms
            SubDataProperty => SubDataProperty,
            EquivalentDataProperties => EquivalentDataProperties,
            DisjointDataProperties => DisjointDataProperties,
            FunctionalDataProperty => FunctionalDataProperty,
            DataPropertyDomain => DataPropertyDomain,
            DataPropertyRange => DataPropertyRange,
            NegativeDataPropertyAssertion => NegativeDataPropertyAssertion,

            // Individual axioms
            SameIndividual => SameIndividual,
            DifferentIndividuals => DifferentIndividuals,

            // Cardinality axioms
            HasKey => HasKey,
            ObjectMinQualifiedCardinality => ObjectMinQualifiedCardinality,
            ObjectMaxQualifiedCardinality => ObjectMaxQualifiedCardinality,
            ObjectExactQualifiedCardinality => ObjectExactQualifiedCardinality,
            DataMinQualifiedCardinality => DataMinQualifiedCardinality,
            DataMaxQualifiedCardinality => DataMaxQualifiedCardinality,
            DataExactQualifiedCardinality => DataExactQualifiedCardinality,

            // Annotation axioms
            AnnotationAssertion => AnnotationAssertion,
            SubAnnotationPropertyOf => SubAnnotationPropertyOf,
            AnnotationPropertyDomain => AnnotationPropertyDomain,
            AnnotationPropertyRange => AnnotationPropertyRange,

            // Special axioms
            Import => Import,
            Collection => Collection,
            Container => Container,
            Reification => Reification
        }
    }

    /// Get the signature IRIs of this axiom (main entities involved)
    pub fn signature(&self) -> Vec<IRI> {
        // Simplified signature extraction - will be enhanced with proper axiom methods
        Vec::new() // Placeholder implementation
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
    object: PropertyAssertionObject,
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
    /// Create a new property assertion axiom with named individual (backward compatibility)
    pub fn new(subject: IRI, property: IRI, object: IRI) -> Self {
        PropertyAssertionAxiom {
            subject,
            property,
            object: PropertyAssertionObject::Named(object),
        }
    }

    /// Create a new property assertion axiom with anonymous individual (blank node)
    pub fn new_with_anonymous(subject: IRI, property: IRI, object: AnonymousIndividual) -> Self {
        PropertyAssertionAxiom {
            subject,
            property,
            object: PropertyAssertionObject::Anonymous(Box::new(object)),
        }
    }

    /// Create a new property assertion axiom with property assertion object
    pub fn new_with_object(subject: IRI, property: IRI, object: PropertyAssertionObject) -> Self {
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
    pub fn object(&self) -> &PropertyAssertionObject {
        &self.object
    }

    /// Get the object as IRI if it's a named individual, returns None for anonymous
    pub fn object_iri(&self) -> Option<&IRI> {
        match &self.object {
            PropertyAssertionObject::Named(iri) => Some(iri),
            PropertyAssertionObject::Anonymous(_) => None,
        }
    }

    /// Get the object as anonymous individual if it's anonymous, returns None for named
    pub fn object_anonymous(&self) -> Option<&AnonymousIndividual> {
        match &self.object {
            PropertyAssertionObject::Named(_) => None,
            PropertyAssertionObject::Anonymous(individual) => Some(&**individual),
        }
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

/// Sub-annotation property axiom: P ⊑ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubAnnotationPropertyOfAxiom {
    sub_property: IRI,
    super_property: IRI,
}

impl SubAnnotationPropertyOfAxiom {
    /// Create a new sub-annotation property axiom
    pub fn new(sub_property: IRI, super_property: IRI) -> Self {
        SubAnnotationPropertyOfAxiom {
            sub_property,
            super_property,
        }
    }

    /// Get the sub-property
    pub fn sub_property(&self) -> &IRI {
        &self.sub_property
    }

    /// Get the super-property
    pub fn super_property(&self) -> &IRI {
        &self.super_property
    }
}

/// Annotation property domain axiom: ∀P.C ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationPropertyDomainAxiom {
    property: IRI,
    domain: IRI,
}

impl AnnotationPropertyDomainAxiom {
    /// Create a new annotation property domain axiom
    pub fn new(property: IRI, domain: IRI) -> Self {
        AnnotationPropertyDomainAxiom { property, domain }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the domain
    pub fn domain(&self) -> &IRI {
        &self.domain
    }
}

/// Annotation property range axiom: ∀P.C ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationPropertyRangeAxiom {
    property: IRI,
    range: IRI,
}

impl AnnotationPropertyRangeAxiom {
    /// Create a new annotation property range axiom
    pub fn new(property: IRI, range: IRI) -> Self {
        AnnotationPropertyRangeAxiom { property, range }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the range
    pub fn range(&self) -> &IRI {
        &self.range
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

/// Object property domain axiom: ∀P.C ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectPropertyDomainAxiom {
    property: IRI,
    domain: class_expressions::ClassExpression,
}

impl ObjectPropertyDomainAxiom {
    /// Create a new object property domain axiom
    pub fn new(property: IRI, domain: class_expressions::ClassExpression) -> Self {
        ObjectPropertyDomainAxiom { property, domain }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the domain class expression
    pub fn domain(&self) -> &class_expressions::ClassExpression {
        &self.domain
    }
}

/// Object property range axiom: ∀P.D ⊑ C
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectPropertyRangeAxiom {
    property: IRI,
    range: class_expressions::ClassExpression,
}

impl ObjectPropertyRangeAxiom {
    /// Create a new object property range axiom
    pub fn new(property: IRI, range: class_expressions::ClassExpression) -> Self {
        ObjectPropertyRangeAxiom { property, range }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the range class expression
    pub fn range(&self) -> &class_expressions::ClassExpression {
        &self.range
    }
}

/// Data property domain axiom: ∀Q.C ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPropertyDomainAxiom {
    property: IRI,
    domain: class_expressions::ClassExpression,
}

impl DataPropertyDomainAxiom {
    /// Create a new data property domain axiom
    pub fn new(property: IRI, domain: class_expressions::ClassExpression) -> Self {
        DataPropertyDomainAxiom { property, domain }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the domain class expression
    pub fn domain(&self) -> &class_expressions::ClassExpression {
        &self.domain
    }
}

/// Data property range axiom: ∃Q.l ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPropertyRangeAxiom {
    property: IRI,
    range: IRI,
}

impl DataPropertyRangeAxiom {
    /// Create a new data property range axiom
    pub fn new(property: IRI, range: IRI) -> Self {
        DataPropertyRangeAxiom { property, range }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the range datatype
    pub fn range(&self) -> &IRI {
        &self.range
    }
}

/// Negative object property assertion axiom: (a, b) ∉ P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativeObjectPropertyAssertionAxiom {
    subject: IRI,
    property: IRI,
    object: IRI,
}

impl NegativeObjectPropertyAssertionAxiom {
    /// Create a new negative object property assertion axiom
    pub fn new(subject: IRI, property: IRI, object: IRI) -> Self {
        NegativeObjectPropertyAssertionAxiom {
            subject,
            property,
            object,
        }
    }

    /// Get the subject individual
    pub fn subject(&self) -> &IRI {
        &self.subject
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the object individual
    pub fn object(&self) -> &IRI {
        &self.object
    }
}

/// Negative data property assertion axiom: (a, l) ∉ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativeDataPropertyAssertionAxiom {
    subject: IRI,
    property: IRI,
    value: crate::entities::Literal,
}

impl NegativeDataPropertyAssertionAxiom {
    /// Create a new negative data property assertion axiom
    pub fn new(subject: IRI, property: IRI, value: crate::entities::Literal) -> Self {
        NegativeDataPropertyAssertionAxiom {
            subject,
            property,
            value,
        }
    }

    /// Get the subject individual
    pub fn subject(&self) -> &IRI {
        &self.subject
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the negated literal value
    pub fn value(&self) -> &crate::entities::Literal {
        &self.value
    }
}

/// Import axiom: imports ontology with given IRI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportAxiom {
    imported_ontology: IRI,
}

impl ImportAxiom {
    /// Create a new import axiom
    pub fn new(imported_ontology: IRI) -> Self {
        ImportAxiom { imported_ontology }
    }

    /// Get the imported ontology IRI
    pub fn imported_ontology(&self) -> &IRI {
        &self.imported_ontology
    }
}

/// RDF Collection axiom representing ordered lists using rdf:first, rdf:rest, rdf:nil
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionAxiom {
    /// The subject that has the collection
    subject: IRI,
    /// The property that relates the subject to the collection
    property: IRI,
    /// The list of items in the collection
    items: Vec<CollectionItem>,
}

/// Individual item in a collection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionItem {
    Named(IRI),
    Anonymous(Box<AnonymousIndividual>),
    Literal(Literal),
}

impl CollectionAxiom {
    /// Create a new collection axiom
    pub fn new(subject: IRI, property: IRI, items: Vec<CollectionItem>) -> Self {
        CollectionAxiom {
            subject,
            property,
            items,
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

    /// Get the items
    pub fn items(&self) -> &Vec<CollectionItem> {
        &self.items
    }

    /// Get the number of items in the collection
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Create property assertions from the collection
    pub fn to_property_assertions(&self) -> OwlResult<Vec<PropertyAssertionAxiom>> {
        let mut assertions = Vec::new();

        // Create a blank node for each collection node
        let _previous_node: Option<AnonymousIndividual> = None;

        // Process items in reverse order to build the linked list
        for (index, item) in self.items.iter().enumerate() {
            let node_id = format!(
                "{}_{}",
                self.subject
                    .as_str()
                    .replace("http://", "")
                    .replace("/", "_"),
                index
            );
            let _anon_node = AnonymousIndividual::new(&node_id);

            // Add rdf:first assertion
            let first_assertion = match item {
                CollectionItem::Named(iri) => PropertyAssertionAxiom::new(
                    create_blank_node_iri(&node_id)?,
                    rdf::first(),
                    iri.clone(),
                ),
                CollectionItem::Anonymous(anon) => PropertyAssertionAxiom::new_with_anonymous(
                    create_blank_node_iri(&node_id)?,
                    rdf::first(),
                    *(*anon).clone(),
                ),
                CollectionItem::Literal(_lit) => {
                    // For literals, we'd need to create a data property assertion
                    // This is a simplified version
                    PropertyAssertionAxiom::new(
                        create_blank_node_iri(&node_id)?,
                        rdf::first(),
                        test::property("literal"), // placeholder
                    )
                }
            };

            // Add rdf:rest assertion
            let rest_assertion = if index == self.items.len() - 1 {
                // Last item points to rdf:nil
                PropertyAssertionAxiom::new(
                    create_blank_node_iri(&node_id)?,
                    rdf::rest(),
                    rdf::nil(),
                )
            } else {
                // Points to next node
                let next_node_id = format!(
                    "{}_{}",
                    self.subject
                        .as_str()
                        .replace("http://", "")
                        .replace("/", "_"),
                    index + 1
                );
                PropertyAssertionAxiom::new(
                    create_blank_node_iri(&node_id)?,
                    rdf::rest(),
                    create_blank_node_iri(&next_node_id)?,
                )
            };

            assertions.push(first_assertion);
            assertions.push(rest_assertion);

            // If this is the first item, connect it to the subject
            if index == 0 {
                let subject_assertion = PropertyAssertionAxiom::new(
                    self.subject.clone(),
                    self.property.clone(),
                    create_blank_node_iri(&node_id)?,
                );
                assertions.insert(0, subject_assertion);
            }

            // _previous_node is tracked but not used in current implementation
            // previous_node = Some(anon_node);
        }

        Ok(assertions)
    }
}

/// RDF Container types (Seq, Bag, Alt)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerType {
    /// Ordered sequence (rdf:Seq)
    Sequence,
    /// Unordered bag (rdf:Bag)
    Bag,
    /// Alternative (rdf:Alt)
    Alternative,
}

/// RDF Container axiom: represents Seq, Bag, or Alt containers
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerAxiom {
    /// The subject that has the container
    subject: IRI,
    /// The property that relates the subject to the container
    property: IRI,
    /// The type of container (Seq, Bag, Alt)
    container_type: ContainerType,
    /// The list of items in the container
    items: Vec<ContainerItem>,
}

/// Individual item in a container
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerItem {
    Named(IRI),
    Anonymous(Box<AnonymousIndividual>),
    Literal(Literal),
}

impl ContainerAxiom {
    /// Create a new container axiom
    pub fn new(
        subject: IRI,
        property: IRI,
        container_type: ContainerType,
        items: Vec<ContainerItem>,
    ) -> Self {
        ContainerAxiom {
            subject,
            property,
            container_type,
            items,
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

    /// Get the container type
    pub fn container_type(&self) -> ContainerType {
        self.container_type
    }

    /// Get the items
    pub fn items(&self) -> &Vec<ContainerItem> {
        &self.items
    }

    /// Get the number of items in the container
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the container is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Create property assertions from the container
    pub fn to_property_assertions(&self) -> OwlResult<Vec<PropertyAssertionAxiom>> {
        let mut assertions = Vec::new();

        // Create the container resource
        let container_id = format!(
            "{}_container",
            self.subject
                .as_str()
                .replace("http://", "")
                .replace("/", "_")
        );
        let container_iri = create_blank_node_iri(&container_id)?;

        // Connect subject to container
        let subject_to_container = PropertyAssertionAxiom::new(
            self.subject.clone(),
            self.property.clone(),
            container_iri.clone(),
        );
        assertions.push(subject_to_container);

        // Add type assertion for the container
        let type_property = rdf::type_property();

        let type_value = match self.container_type {
            ContainerType::Sequence => rdf::seq(),
            ContainerType::Bag => rdf::bag(),
            ContainerType::Alternative => rdf::alt(),
        };

        let type_assertion =
            PropertyAssertionAxiom::new(container_iri.clone(), type_property, type_value);
        assertions.push(type_assertion);

        // Add numbered elements (rdf:_1, rdf:_2, etc.)
        for (index, item) in self.items.iter().enumerate() {
            let element_property =
                create_iri_safe(&format!("{}{}", RDF_ELEMENT_PREFIX, index + 1))?;

            let element_assertion = match item {
                ContainerItem::Named(iri) => PropertyAssertionAxiom::new(
                    container_iri.clone(),
                    element_property,
                    iri.clone(),
                ),
                ContainerItem::Anonymous(anon) => PropertyAssertionAxiom::new(
                    container_iri.clone(),
                    element_property,
                    create_blank_node_iri(anon.node_id())?,
                ),
                ContainerItem::Literal(_lit) => {
                    // For literals, we need to use a data property assertion
                    // This is a simplification - in practice, containers typically use IRIs
                    PropertyAssertionAxiom::new(
                        container_iri.clone(),
                        element_property,
                        create_blank_node_iri(&format!("literal_{}", index))?,
                    )
                }
            };
            assertions.push(element_assertion);
        }

        Ok(assertions)
    }
}

/// RDF Reification axiom: represents statements about statements using rdf:subject, rdf:predicate, rdf:object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReificationAxiom {
    /// The reified statement resource (blank node or named resource)
    reification_resource: IRI,
    /// The subject of the original statement
    subject: IRI,
    /// The predicate of the original statement
    predicate: IRI,
    /// The object of the original statement
    object: ReificationObject,
    /// Additional properties about the reified statement
    properties: Vec<PropertyAssertionAxiom>,
}

/// Object in a reified statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReificationObject {
    Named(IRI),
    Anonymous(Box<AnonymousIndividual>),
    Literal(Literal),
}

impl ReificationAxiom {
    /// Create a new reification axiom
    pub fn new(
        reification_resource: IRI,
        subject: IRI,
        predicate: IRI,
        object: ReificationObject,
    ) -> Self {
        ReificationAxiom {
            reification_resource,
            subject,
            predicate,
            object,
            properties: Vec::new(),
        }
    }

    /// Create a new reification axiom with additional properties
    pub fn with_properties(
        reification_resource: IRI,
        subject: IRI,
        predicate: IRI,
        object: ReificationObject,
        properties: Vec<PropertyAssertionAxiom>,
    ) -> Self {
        ReificationAxiom {
            reification_resource,
            subject,
            predicate,
            object,
            properties,
        }
    }

    /// Get the reification resource
    pub fn reification_resource(&self) -> &IRI {
        &self.reification_resource
    }

    /// Get the subject of the original statement
    pub fn subject(&self) -> &IRI {
        &self.subject
    }

    /// Get the predicate of the original statement
    pub fn predicate(&self) -> &IRI {
        &self.predicate
    }

    /// Get the object of the original statement
    pub fn object(&self) -> &ReificationObject {
        &self.object
    }

    /// Get additional properties about the reified statement
    pub fn properties(&self) -> &Vec<PropertyAssertionAxiom> {
        &self.properties
    }

    /// Add a property to the reified statement
    pub fn add_property(&mut self, property: PropertyAssertionAxiom) {
        self.properties.push(property);
    }

    /// Create property assertions from the reification
    pub fn to_property_assertions(&self) -> OwlResult<Vec<PropertyAssertionAxiom>> {
        let mut assertions = Vec::new();

        // Add rdf:subject assertion
        let subject_assertion = PropertyAssertionAxiom::new(
            self.reification_resource.clone(),
            rdf::subject(),
            self.subject.clone(),
        );
        assertions.push(subject_assertion);

        // Add rdf:predicate assertion
        let predicate_assertion = PropertyAssertionAxiom::new(
            self.reification_resource.clone(),
            rdf::predicate(),
            self.predicate.clone(),
        );
        assertions.push(predicate_assertion);

        // Add rdf:object assertion
        let object_iri = match &self.object {
            ReificationObject::Named(iri) => iri.clone(),
            ReificationObject::Anonymous(anon) => create_blank_node_iri(anon.node_id())?,
            ReificationObject::Literal(lit) => {
                // For literals, create a temporary IRI (simplification)
                create_blank_node_iri(&format!("literal_{}", lit.lexical_form()))?
            }
        };

        let object_assertion = PropertyAssertionAxiom::new(
            self.reification_resource.clone(),
            rdf::object(),
            object_iri,
        );
        assertions.push(object_assertion);

        // Add additional properties
        assertions.extend(self.properties.clone());

        // Add rdf:type assertion to identify as rdf:Statement
        let type_assertion = PropertyAssertionAxiom::new(
            self.reification_resource.clone(),
            rdf::type_property(),
            rdf::statement(),
        );
        assertions.push(type_assertion);

        Ok(assertions)
    }

    /// Get the original statement as a triple (subject, predicate, object)
    pub fn original_statement(&self) -> (&IRI, &IRI, &ReificationObject) {
        (&self.subject, &self.predicate, &self.object)
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
        let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
        let mary_iri = IRI::new("http://example.org/mary").unwrap();

        let axiom =
            PropertyAssertionAxiom::new(john_iri.clone(), has_parent_iri.clone(), mary_iri.clone());

        assert_eq!(axiom.subject(), &john_iri);
        assert_eq!(axiom.property(), &has_parent_iri);
        assert_eq!(axiom.object_iri(), Some(&mary_iri));
    }

    // Tests for property characteristic axioms
    #[test]
    fn test_functional_property_axiom() {
        let has_father_iri = IRI::new("http://example.org/hasFather").unwrap();
        let axiom = FunctionalPropertyAxiom::new(has_father_iri.clone());

        assert_eq!(axiom.property(), &has_father_iri);
    }

    #[test]
    fn test_inverse_functional_property_axiom() {
        let has_ssn_iri = IRI::new("http://example.org/hasSSN").unwrap();
        let axiom = InverseFunctionalPropertyAxiom::new(has_ssn_iri.clone());

        assert_eq!(axiom.property(), &has_ssn_iri);
    }

    #[test]
    fn test_reflexive_property_axiom() {
        let knows_iri = IRI::new("http://example.org/knows").unwrap();
        let axiom = ReflexivePropertyAxiom::new(knows_iri.clone());

        assert_eq!(axiom.property(), &knows_iri);
    }

    #[test]
    fn test_irreflexive_property_axiom() {
        let parent_of_iri = IRI::new("http://example.org/parentOf").unwrap();
        let axiom = IrreflexivePropertyAxiom::new(parent_of_iri.clone());

        assert_eq!(axiom.property(), &parent_of_iri);
    }

    #[test]
    fn test_symmetric_property_axiom() {
        let spouse_iri = IRI::new("http://example.org/spouse").unwrap();
        let axiom = SymmetricPropertyAxiom::new(spouse_iri.clone());

        assert_eq!(axiom.property(), &spouse_iri);
    }

    #[test]
    fn test_asymmetric_property_axiom() {
        let parent_of_iri = IRI::new("http://example.org/parentOf").unwrap();
        let axiom = AsymmetricPropertyAxiom::new(parent_of_iri.clone());

        assert_eq!(axiom.property(), &parent_of_iri);
    }

    #[test]
    fn test_transitive_property_axiom() {
        let ancestor_iri = IRI::new("http://example.org/ancestor").unwrap();
        let axiom = TransitivePropertyAxiom::new(ancestor_iri.clone());

        assert_eq!(axiom.property(), &ancestor_iri);
    }

    #[test]
    fn test_axiom_enum_property_characteristics() {
        let has_father_iri = IRI::new("http://example.org/hasFather").unwrap();
        let knows_iri = IRI::new("http://example.org/knows").unwrap();
        let ancestor_iri = IRI::new("http://example.org/ancestor").unwrap();

        let functional_axiom = Axiom::FunctionalProperty(Box::new(FunctionalPropertyAxiom::new(
            has_father_iri.clone(),
        )));
        let reflexive_axiom =
            Axiom::ReflexiveProperty(Box::new(ReflexivePropertyAxiom::new(knows_iri.clone())));
        let transitive_axiom =
            Axiom::TransitiveProperty(Box::new(TransitivePropertyAxiom::new(ancestor_iri.clone())));

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

        let sub_data_axiom = Axiom::SubDataProperty(Box::new(SubDataPropertyAxiom::new(
            has_age_iri.clone(),
            height_iri.clone(),
        )));
        let functional_data_axiom = Axiom::FunctionalDataProperty(
            FunctionalDataPropertyAxiom::new(has_birth_date_iri.clone()),
        );
        let equivalent_data_axiom =
            Axiom::EquivalentDataProperties(Box::new(EquivalentDataPropertiesAxiom::new(vec![
                has_age_iri.clone(),
                height_iri.clone(),
            ])));
        let disjoint_data_axiom =
            Axiom::DisjointDataProperties(Box::new(DisjointDataPropertiesAxiom::new(vec![
                has_age_iri.clone(),
                weight_iri.clone(),
            ])));

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

        let same_axiom = Axiom::SameIndividual(Box::new(SameIndividualAxiom::new(vec![
            individual1.clone(),
            individual2.clone(),
        ])));
        let different_axiom =
            Axiom::DifferentIndividuals(Box::new(DifferentIndividualsAxiom::new(vec![
                individual1.clone(),
                individual2.clone(),
            ])));

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

        let has_parent = ObjectPropertyExpression::ObjectProperty(Box::new(ObjectProperty::new(
            has_parent_iri.clone(),
        )));
        let has_grandparent = ObjectPropertyExpression::ObjectProperty(Box::new(
            ObjectProperty::new(has_grandparent_iri.clone()),
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

        let has_parent = ObjectPropertyExpression::ObjectProperty(Box::new(ObjectProperty::new(
            has_parent_iri.clone(),
        )));
        let has_child = ObjectPropertyExpression::ObjectProperty(Box::new(ObjectProperty::new(
            has_child_iri.clone(),
        )));

        let axiom = InverseObjectPropertiesAxiom::new(has_parent.clone(), has_child.clone());

        assert_eq!(axiom.property1(), &has_parent);
        assert_eq!(axiom.property2(), &has_child);
    }

    #[test]
    fn test_property_chain_with_inverse() {
        let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
        let has_child_iri = IRI::new("http://example.org/hasChild").unwrap();

        let has_parent = ObjectPropertyExpression::ObjectProperty(Box::new(ObjectProperty::new(
            has_parent_iri.clone(),
        )));
        let has_child_inverse = ObjectPropertyExpression::ObjectInverseOf(Box::new(
            ObjectPropertyExpression::ObjectProperty(Box::new(ObjectProperty::new(
                has_child_iri.clone(),
            ))),
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

        let has_parent = ObjectPropertyExpression::ObjectProperty(Box::new(ObjectProperty::new(
            has_parent_iri.clone(),
        )));
        let has_grandparent = ObjectPropertyExpression::ObjectProperty(Box::new(
            ObjectProperty::new(has_grandparent_iri.clone()),
        ));

        let chain_axiom = Axiom::SubPropertyChainOf(Box::new(SubPropertyChainOfAxiom::new(
            vec![has_parent.clone()],
            has_grandparent.clone(),
        )));

        match chain_axiom {
            Axiom::SubPropertyChainOf(_) => assert!(true),
            _ => assert!(false, "Expected SubPropertyChainOf axiom"),
        }
    }

    #[test]
    fn test_inverse_properties_axioms_in_enum() {
        let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
        let has_child_iri = IRI::new("http://example.org/hasChild").unwrap();

        let has_parent = ObjectPropertyExpression::ObjectProperty(Box::new(ObjectProperty::new(
            has_parent_iri.clone(),
        )));
        let has_child = ObjectPropertyExpression::ObjectProperty(Box::new(ObjectProperty::new(
            has_child_iri.clone(),
        )));

        let inverse_axiom = Axiom::InverseObjectProperties(Box::new(
            InverseObjectPropertiesAxiom::new(has_parent.clone(), has_child.clone()),
        ));

        match inverse_axiom {
            Axiom::InverseObjectProperties(_) => assert!(true),
            _ => assert!(false, "Expected InverseObjectProperties axiom"),
        }
    }
}
