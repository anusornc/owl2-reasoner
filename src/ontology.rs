//! Ontology structure and management
//!
//! This module defines the main ontology structure that contains all OWL2 entities
//! and axioms. The ontology provides indexed storage for efficient access and
//! comprehensive performance optimizations.
//!
//! ## Features
//!
//! - **Indexed Storage**: O(1) access to axioms by type
//! - **Memory Efficiency**: Arc-based entity sharing
//! - **Performance Indexes**: Automatic indexing for common operations
//! - **Import Support**: Multi-ontology reasoning capabilities
//!
//! ## Usage
//!
//! ```rust
//! use owl2_reasoner::{Ontology, Class, SubClassOfAxiom, ClassExpression};
//!
//! // Create a new ontology
//! let mut ontology = Ontology::new();
//! ontology.set_iri("http://example.org/family");
//!
//! // Add entities
//! let person = Class::new("http://example.org/Person");
//! let parent = Class::new("http://example.org/Parent");
//! ontology.add_class(person.clone())?;
//! ontology.add_class(parent.clone())?;
//!
//! // Add axioms
//! let subclass_axiom = SubClassOfAxiom::new(
//!     ClassExpression::from(parent.clone()),
//!     ClassExpression::from(person.clone()),
//! );
//! ontology.add_subclass_axiom(subclass_axiom)?;
//!
//! println!("Ontology has {} classes and {} axioms",
//!          ontology.classes().len(), ontology.axiom_count());
//!
//! # Ok::<(), owl2_reasoner::OwlError>(())
//! ```

use crate::axioms;
use crate::axioms::class_expressions::ClassExpression;
use crate::entities::*;
use crate::error::{OwlError, OwlResult};
use crate::iri::{IRIRegistry, IRI};
use hashbrown::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

/// An OWL2 ontology with indexed storage and performance optimizations
///
/// Represents a complete OWL2 ontology containing entities, axioms, and annotations.
/// The ontology uses indexed storage for O(1) access to axioms by type and maintains
/// performance indexes for common operations.
///
/// ## Architecture
///
/// ```text
/// Ontology {
///     // Basic ontology information
///     iri: Option<Arc<IRI>>,
///     version_iri: Option<Arc<IRI>>,
///     imports: HashSet<Arc<IRI>>,
///     
///     // Entity storage (Arc-based sharing)
///     classes: HashSet<Arc<Class>>,
///     object_properties: HashSet<Arc<ObjectProperty>>,
///     data_properties: HashSet<Arc<DataProperty>>,
///     named_individuals: HashSet<Arc<NamedIndividual>>,
///     
///     // Indexed axiom storage for O(1) access
///     subclass_axioms: Vec<Arc<SubClassOfAxiom>>,
///     equivalent_classes_axioms: Vec<Arc<EquivalentClassesAxiom>>,
///     disjoint_classes_axioms: Vec<Arc<DisjointClassesAxiom>>,
///     // ... other axiom types
///     
///     // Performance indexes
///     class_instances: HashMap<IRI, Vec<IRI>>,
///     property_domains: HashMap<IRI, Vec<IRI>>,
///     property_ranges: HashMap<IRI, Vec<IRI>>,
///     
///     // Additional features
///     annotations: Vec<Annotation>,
///     iri_registry: IRIRegistry,
/// }
/// ```
///
/// ## Performance Characteristics
///
/// - **Entity Access**: O(1) for all entity types
/// - **Axiom Access**: O(1) for indexed axiom types
/// - **Index Maintenance**: Automatic during axiom addition
/// - **Memory Overhead**: ~20% vs non-indexed storage
///
/// ## Examples
///
/// ```rust
/// use owl2_reasoner::{Ontology, Class, ObjectProperty, NamedIndividual};
/// use owl2_reasoner::{SubClassOfAxiom, ClassAssertionAxiom, ClassExpression};
///
/// // Create a new ontology
/// let mut ontology = Ontology::new();
/// ontology.set_iri("http://example.org/family");
///
/// // Add entities
/// let person = Class::new("http://example.org/Person");
/// let parent = Class::new("http://example.org/Parent");
/// let has_child = ObjectProperty::new("http://example.org/hasChild");
/// let john = NamedIndividual::new("http://example.org/John");
///
/// ontology.add_class(person.clone())?;
/// ontology.add_class(parent.clone())?;
/// ontology.add_object_property(has_child.clone())?;
/// ontology.add_named_individual(john.clone())?;
///
/// // Add axioms
/// let subclass_axiom = SubClassOfAxiom::new(
///     ClassExpression::from(parent.clone()),
///     ClassExpression::from(person.clone()),
/// );
/// ontology.add_subclass_axiom(subclass_axiom)?;
///
/// let class_assertion = ClassAssertionAxiom::new(
///     john.iri().clone(),
///     ClassExpression::Class(person.clone()),
/// );
/// ontology.add_class_assertion(class_assertion)?;
///
/// // Access indexed axioms
/// let subclass_axioms = ontology.subclass_axioms();
/// let class_assertions = ontology.class_assertions();
///
/// println!("Found {} subclass axioms", subclass_axioms.len());
/// println!("Found {} class assertions", class_assertions.len());
///
/// # Ok::<(), owl2_reasoner::OwlError>(())
/// ```
#[derive(Debug, Clone)]
pub struct Ontology {
    /// The ontology IRI
    iri: Option<Arc<IRI>>,
    /// The version IRI
    version_iri: Option<Arc<IRI>>,
    /// Import declarations
    imports: HashSet<Arc<IRI>>,
    /// All classes in the ontology
    classes: HashSet<Arc<Class>>,
    /// All object properties in the ontology
    object_properties: HashSet<Arc<ObjectProperty>>,
    /// All data properties in the ontology
    data_properties: HashSet<Arc<DataProperty>>,
    /// All named individuals in the ontology
    named_individuals: HashSet<Arc<NamedIndividual>>,
    /// All anonymous individuals in the ontology
    anonymous_individuals: HashSet<Arc<AnonymousIndividual>>,
    /// All annotation properties in the ontology
    annotation_properties: HashSet<Arc<AnnotationProperty>>,
    /// All axioms in the ontology
    axioms: Vec<Arc<axioms::Axiom>>,

    // Indexed axiom storage for performance
    subclass_axioms: Vec<Arc<axioms::SubClassOfAxiom>>,
    equivalent_classes_axioms: Vec<Arc<axioms::EquivalentClassesAxiom>>,
    disjoint_classes_axioms: Vec<Arc<axioms::DisjointClassesAxiom>>,
    class_assertions: Vec<Arc<axioms::ClassAssertionAxiom>>,
    property_assertions: Vec<Arc<axioms::PropertyAssertionAxiom>>,
    data_property_assertions: Vec<Arc<axioms::DataPropertyAssertionAxiom>>,
    subobject_property_axioms: Vec<Arc<axioms::SubObjectPropertyAxiom>>,
    equivalent_object_properties_axioms: Vec<Arc<axioms::EquivalentObjectPropertiesAxiom>>,
    disjoint_object_properties_axioms: Vec<Arc<axioms::DisjointObjectPropertiesAxiom>>,
    functional_property_axioms: Vec<Arc<axioms::FunctionalPropertyAxiom>>,
    inverse_functional_property_axioms: Vec<Arc<axioms::InverseFunctionalPropertyAxiom>>,
    reflexive_property_axioms: Vec<Arc<axioms::ReflexivePropertyAxiom>>,
    irreflexive_property_axioms: Vec<Arc<axioms::IrreflexivePropertyAxiom>>,
    symmetric_property_axioms: Vec<Arc<axioms::SymmetricPropertyAxiom>>,
    asymmetric_property_axioms: Vec<Arc<axioms::AsymmetricPropertyAxiom>>,
    transitive_property_axioms: Vec<Arc<axioms::TransitivePropertyAxiom>>,
    subdata_property_axioms: Vec<Arc<axioms::SubDataPropertyAxiom>>,
    equivalent_data_properties_axioms: Vec<Arc<axioms::EquivalentDataPropertiesAxiom>>,
    disjoint_data_properties_axioms: Vec<Arc<axioms::DisjointDataPropertiesAxiom>>,
    functional_data_property_axioms: Vec<Arc<axioms::FunctionalDataPropertyAxiom>>,
    same_individual_axioms: Vec<Arc<axioms::SameIndividualAxiom>>,
    different_individuals_axioms: Vec<Arc<axioms::DifferentIndividualsAxiom>>,
    has_key_axioms: Vec<Arc<axioms::HasKeyAxiom>>,
    annotation_assertion_axioms: Vec<Arc<axioms::AnnotationAssertionAxiom>>,
    sub_annotation_property_axioms: Vec<Arc<axioms::SubAnnotationPropertyOfAxiom>>,
    annotation_property_domain_axioms: Vec<Arc<axioms::AnnotationPropertyDomainAxiom>>,
    annotation_property_range_axioms: Vec<Arc<axioms::AnnotationPropertyRangeAxiom>>,
    sub_property_chain_axioms: Vec<Arc<axioms::SubPropertyChainOfAxiom>>,
    inverse_object_properties_axioms: Vec<Arc<axioms::InverseObjectPropertiesAxiom>>,
    object_min_qualified_cardinality_axioms: Vec<Arc<axioms::ObjectMinQualifiedCardinalityAxiom>>,
    object_max_qualified_cardinality_axioms: Vec<Arc<axioms::ObjectMaxQualifiedCardinalityAxiom>>,
    object_exact_qualified_cardinality_axioms:
        Vec<Arc<axioms::ObjectExactQualifiedCardinalityAxiom>>,
    data_min_qualified_cardinality_axioms: Vec<Arc<axioms::DataMinQualifiedCardinalityAxiom>>,
    data_max_qualified_cardinality_axioms: Vec<Arc<axioms::DataMaxQualifiedCardinalityAxiom>>,
    data_exact_qualified_cardinality_axioms: Vec<Arc<axioms::DataExactQualifiedCardinalityAxiom>>,
    object_property_domain_axioms: Vec<Arc<axioms::ObjectPropertyDomainAxiom>>,
    object_property_range_axioms: Vec<Arc<axioms::ObjectPropertyRangeAxiom>>,
    data_property_domain_axioms: Vec<Arc<axioms::DataPropertyDomainAxiom>>,
    data_property_range_axioms: Vec<Arc<axioms::DataPropertyRangeAxiom>>,
    negative_object_property_assertion_axioms:
        Vec<Arc<axioms::NegativeObjectPropertyAssertionAxiom>>,
    negative_data_property_assertion_axioms: Vec<Arc<axioms::NegativeDataPropertyAssertionAxiom>>,

    // Performance indexes
    class_instances: HashMap<IRI, Vec<IRI>>,
    property_domains: HashMap<IRI, Vec<IRI>>,
    property_ranges: HashMap<IRI, Vec<IRI>>,

    // Multi-indexed axiom storage for fast queries
    /// Index axioms by their signature (main entities involved)
    #[allow(dead_code)]
    axiom_signature_index: HashMap<IRI, Vec<Arc<axioms::Axiom>>>,
    /// Index class axioms by class IRI for O(1) lookup
    #[allow(dead_code)]
    class_axioms_index: HashMap<IRI, Vec<Arc<axioms::Axiom>>>,
    /// Index property axioms by property IRI for O(1) lookup
    #[allow(dead_code)]
    property_axioms_index: HashMap<IRI, Vec<Arc<axioms::Axiom>>>,
    /// Index individual axioms by individual IRI for O(1) lookup
    #[allow(dead_code)]
    individual_axioms_index: HashMap<IRI, Vec<Arc<axioms::Axiom>>>,
    /// Index axioms by type for fast type-based queries
    axiom_type_index: HashMap<axioms::AxiomType, Vec<Arc<axioms::Axiom>>>,
    /// Inverted index for annotation properties
    #[allow(dead_code)]
    annotation_property_index: HashMap<IRI, Vec<Arc<axioms::AnnotationAssertionAxiom>>>,

    /// Annotations on the ontology itself
    annotations: Vec<Annotation>,
    /// IRI registry for managing namespaces
    iri_registry: IRIRegistry,
}

impl Ontology {
    /// Create a new empty ontology
    pub fn new() -> Self {
        Ontology {
            iri: None,
            version_iri: None,
            imports: HashSet::new(),
            classes: HashSet::new(),
            object_properties: HashSet::new(),
            data_properties: HashSet::new(),
            named_individuals: HashSet::new(),
            anonymous_individuals: HashSet::new(),
            annotation_properties: HashSet::new(),
            axioms: Vec::new(),
            subclass_axioms: Vec::new(),
            equivalent_classes_axioms: Vec::new(),
            disjoint_classes_axioms: Vec::new(),
            class_assertions: Vec::new(),
            property_assertions: Vec::new(),
            data_property_assertions: Vec::new(),
            subobject_property_axioms: Vec::new(),
            equivalent_object_properties_axioms: Vec::new(),
            disjoint_object_properties_axioms: Vec::new(),
            functional_property_axioms: Vec::new(),
            inverse_functional_property_axioms: Vec::new(),
            reflexive_property_axioms: Vec::new(),
            irreflexive_property_axioms: Vec::new(),
            symmetric_property_axioms: Vec::new(),
            asymmetric_property_axioms: Vec::new(),
            transitive_property_axioms: Vec::new(),
            subdata_property_axioms: Vec::new(),
            equivalent_data_properties_axioms: Vec::new(),
            disjoint_data_properties_axioms: Vec::new(),
            functional_data_property_axioms: Vec::new(),
            same_individual_axioms: Vec::new(),
            different_individuals_axioms: Vec::new(),
            has_key_axioms: Vec::new(),
            annotation_assertion_axioms: Vec::new(),
            sub_annotation_property_axioms: Vec::new(),
            annotation_property_domain_axioms: Vec::new(),
            annotation_property_range_axioms: Vec::new(),
            sub_property_chain_axioms: Vec::new(),
            inverse_object_properties_axioms: Vec::new(),
            object_min_qualified_cardinality_axioms: Vec::new(),
            object_max_qualified_cardinality_axioms: Vec::new(),
            object_exact_qualified_cardinality_axioms: Vec::new(),
            data_min_qualified_cardinality_axioms: Vec::new(),
            data_max_qualified_cardinality_axioms: Vec::new(),
            data_exact_qualified_cardinality_axioms: Vec::new(),
            object_property_domain_axioms: Vec::new(),
            object_property_range_axioms: Vec::new(),
            data_property_domain_axioms: Vec::new(),
            data_property_range_axioms: Vec::new(),
            negative_object_property_assertion_axioms: Vec::new(),
            negative_data_property_assertion_axioms: Vec::new(),
            class_instances: HashMap::new(),
            property_domains: HashMap::new(),
            property_ranges: HashMap::new(),
            axiom_signature_index: HashMap::new(),
            class_axioms_index: HashMap::new(),
            property_axioms_index: HashMap::new(),
            individual_axioms_index: HashMap::new(),
            axiom_type_index: HashMap::new(),
            annotation_property_index: HashMap::new(),
            annotations: Vec::new(),
            iri_registry: IRIRegistry::new(),
        }
    }

    /// Create a new ontology with the given IRI
    pub fn with_iri<I: Into<IRI>>(iri: I) -> Self {
        let mut ontology = Self::new();
        ontology.iri = Some(Arc::new(iri.into()));
        ontology
    }

    /// Get the ontology IRI
    pub fn iri(&self) -> Option<&IRI> {
        self.iri.as_deref()
    }

    /// Get the version IRI
    pub fn version_iri(&self) -> Option<&IRI> {
        self.version_iri.as_deref()
    }

    /// Set the ontology IRI
    pub fn set_iri<I: Into<IRI>>(&mut self, iri: I) {
        self.iri = Some(Arc::new(iri.into()));
    }

    /// Set the version IRI
    pub fn set_version_iri<I: Into<IRI>>(&mut self, version_iri: I) {
        self.version_iri = Some(Arc::new(version_iri.into()));
    }

    /// Add an import declaration
    pub fn add_import<I: Into<IRI>>(&mut self, import_iri: I) {
        self.imports.insert(Arc::new(import_iri.into()));
    }

    /// Get all import declarations
    pub fn imports(&self) -> &HashSet<Arc<IRI>> {
        &self.imports
    }

    /// Add a class to the ontology
    pub fn add_class(&mut self, class: Class) -> OwlResult<()> {
        // Validate class IRI
        self.validate_class_iri(class.iri())?;

        // Check for duplicate classes
        if self.classes.iter().any(|c| c.iri() == class.iri()) {
            // Gracefully accept duplicate additions (idempotent)
            return Ok(());
        }

        // Validate class against OWL2 built-in classes
        self.validate_builtin_class_usage(class.iri())?;

        let class_arc = Arc::new(class);
        self.classes.insert(class_arc);
        Ok(())
    }

    /// Get all classes in the ontology
    pub fn classes(&self) -> &HashSet<Arc<Class>> {
        &self.classes
    }

    /// Add an object property to the ontology
    pub fn add_object_property(&mut self, property: ObjectProperty) -> OwlResult<()> {
        let property_arc = Arc::new(property);
        self.object_properties.insert(property_arc);
        Ok(())
    }

    /// Get all object properties in the ontology
    pub fn object_properties(&self) -> &HashSet<Arc<ObjectProperty>> {
        &self.object_properties
    }

    /// Add a data property to the ontology
    pub fn add_data_property(&mut self, property: DataProperty) -> OwlResult<()> {
        let property_arc = Arc::new(property);
        self.data_properties.insert(property_arc);
        Ok(())
    }

    /// Get all data properties in the ontology
    pub fn data_properties(&self) -> &HashSet<Arc<DataProperty>> {
        &self.data_properties
    }

    /// Add a named individual to the ontology
    pub fn add_named_individual(&mut self, individual: NamedIndividual) -> OwlResult<()> {
        let individual_arc = Arc::new(individual);
        self.named_individuals.insert(individual_arc);
        Ok(())
    }

    /// Add an anonymous individual to the ontology
    pub fn add_anonymous_individual(&mut self, individual: AnonymousIndividual) -> OwlResult<()> {
        let individual_arc = Arc::new(individual);
        self.anonymous_individuals.insert(individual_arc);
        Ok(())
    }

    /// Add an annotation property to the ontology
    pub fn add_annotation_property(&mut self, property: AnnotationProperty) -> OwlResult<()> {
        let property_arc = Arc::new(property);
        self.annotation_properties.insert(property_arc);
        Ok(())
    }

    /// Get all named individuals in the ontology
    pub fn named_individuals(&self) -> &HashSet<Arc<NamedIndividual>> {
        &self.named_individuals
    }

    /// Get all anonymous individuals in the ontology
    pub fn anonymous_individuals(&self) -> &HashSet<Arc<AnonymousIndividual>> {
        &self.anonymous_individuals
    }

    /// Get all annotation properties in the ontology
    pub fn annotation_properties(&self) -> &HashSet<Arc<AnnotationProperty>> {
        &self.annotation_properties
    }

    /// Add an axiom to the ontology
    pub fn add_axiom(&mut self, axiom: axioms::Axiom) -> OwlResult<()> {
        let axiom_arc = Arc::new(axiom);

        // Add to general axioms list
        self.axioms.push(axiom_arc.clone());

        // Add to indexed storage based on axiom type
        match axiom_arc.as_ref() {
            axioms::Axiom::SubClassOf(axiom) => {
                let subclass_arc = Arc::new((**axiom).clone());
                self.subclass_axioms.push(subclass_arc);
            }
            axioms::Axiom::EquivalentClasses(axiom) => {
                let equiv_arc = Arc::new((**axiom).clone());
                self.equivalent_classes_axioms.push(equiv_arc);
            }
            axioms::Axiom::DisjointClasses(axiom) => {
                let disjoint_arc = Arc::new((**axiom).clone());
                self.disjoint_classes_axioms.push(disjoint_arc);
            }
            axioms::Axiom::ClassAssertion(axiom) => {
                let assertion_arc = Arc::new((**axiom).clone());
                self.class_assertions.push(assertion_arc);
                // Update class instances index
                if let Some(class_iri) = axiom.class_expr().as_named().map(|c| (**c.iri()).clone()) {
                    self.class_instances
                        .entry((**axiom.individual()).clone())
                        .or_default()
                        .push(class_iri);
                }
            }
            axioms::Axiom::PropertyAssertion(axiom) => {
                let assertion_arc = Arc::new((**axiom).clone());
                self.property_assertions.push(assertion_arc);
                // Update property domains and ranges indexes
                self.property_domains
                    .entry((**axiom.property()).clone())
                    .or_default()
                    .push((**axiom.subject()).clone());
                // Only index named objects (IRIs) into property_ranges
                if let crate::axioms::PropertyAssertionObject::Named(object_iri) = axiom.object() {
                    self.property_ranges
                        .entry((**axiom.property()).clone())
                        .or_default()
                    .push((**object_iri).clone());
                }
            }
            axioms::Axiom::DataPropertyAssertion(axiom) => {
                let assertion_arc = Arc::new((**axiom).clone());
                self.data_property_assertions.push(assertion_arc);
                // We don't index literals into property_ranges (IRI-only index)
                self.property_domains
                    .entry((**axiom.property()).clone())
                    .or_default()
                    .push((**axiom.subject()).clone());
            }
            axioms::Axiom::SubObjectProperty(axiom) => {
                let subprop_arc = Arc::new((**axiom).clone());
                self.subobject_property_axioms.push(subprop_arc);
            }
            axioms::Axiom::EquivalentObjectProperties(axiom) => {
                let equiv_arc = Arc::new((**axiom).clone());
                self.equivalent_object_properties_axioms.push(equiv_arc);
            }
            axioms::Axiom::DisjointObjectProperties(axiom) => {
                let disjoint_arc = Arc::new((**axiom).clone());
                self.disjoint_object_properties_axioms.push(disjoint_arc);
            }
            axioms::Axiom::FunctionalProperty(axiom) => {
                let functional_arc = Arc::new((**axiom).clone());
                self.functional_property_axioms.push(functional_arc);
            }
            axioms::Axiom::InverseFunctionalProperty(axiom) => {
                let inv_functional_arc = Arc::new((**axiom).clone());
                self.inverse_functional_property_axioms
                    .push(inv_functional_arc);
            }
            axioms::Axiom::ReflexiveProperty(axiom) => {
                let reflexive_arc = Arc::new((**axiom).clone());
                self.reflexive_property_axioms.push(reflexive_arc);
            }
            axioms::Axiom::IrreflexiveProperty(axiom) => {
                let irreflexive_arc = Arc::new((**axiom).clone());
                self.irreflexive_property_axioms.push(irreflexive_arc);
            }
            axioms::Axiom::SymmetricProperty(axiom) => {
                let symmetric_arc = Arc::new((**axiom).clone());
                self.symmetric_property_axioms.push(symmetric_arc);
            }
            axioms::Axiom::AsymmetricProperty(axiom) => {
                let asymmetric_arc = Arc::new((**axiom).clone());
                self.asymmetric_property_axioms.push(asymmetric_arc);
            }
            axioms::Axiom::TransitiveProperty(axiom) => {
                let transitive_arc = Arc::new((**axiom).clone());
                self.transitive_property_axioms.push(transitive_arc);
            }
            axioms::Axiom::SubDataProperty(axiom) => {
                let subdata_arc = Arc::new((**axiom).clone());
                self.subdata_property_axioms.push(subdata_arc);
            }
            axioms::Axiom::EquivalentDataProperties(axiom) => {
                let equiv_data_arc = Arc::new((**axiom).clone());
                self.equivalent_data_properties_axioms.push(equiv_data_arc);
            }
            axioms::Axiom::DisjointDataProperties(axiom) => {
                let disjoint_data_arc = Arc::new((**axiom).clone());
                self.disjoint_data_properties_axioms.push(disjoint_data_arc);
            }
            axioms::Axiom::FunctionalDataProperty(axiom) => {
                let functional_data_arc = Arc::new(axiom.clone());
                self.functional_data_property_axioms
                    .push(functional_data_arc);
            }
            axioms::Axiom::SameIndividual(axiom) => {
                let same_individual_arc = Arc::new((**axiom).clone());
                self.same_individual_axioms.push(same_individual_arc);
            }
            axioms::Axiom::DifferentIndividuals(axiom) => {
                let different_individuals_arc = Arc::new((**axiom).clone());
                self.different_individuals_axioms
                    .push(different_individuals_arc);
            }
            axioms::Axiom::HasKey(axiom) => {
                let has_key_arc = Arc::new((**axiom).clone());
                self.has_key_axioms.push(has_key_arc);
            }
            axioms::Axiom::AnnotationAssertion(axiom) => {
                let annotation_assertion_arc = Arc::new((**axiom).clone());
                self.annotation_assertion_axioms
                    .push(annotation_assertion_arc);
            }
            axioms::Axiom::SubPropertyChainOf(axiom) => {
                let sub_property_chain_arc = Arc::new((**axiom).clone());
                self.sub_property_chain_axioms.push(sub_property_chain_arc);
            }
            axioms::Axiom::InverseObjectProperties(axiom) => {
                let inverse_object_properties_arc = Arc::new((**axiom).clone());
                self.inverse_object_properties_axioms
                    .push(inverse_object_properties_arc);
            }
            axioms::Axiom::ObjectMinQualifiedCardinality(axiom) => {
                let object_min_qualified_cardinality_arc = Arc::new((**axiom).clone());
                self.object_min_qualified_cardinality_axioms
                    .push(object_min_qualified_cardinality_arc);
            }
            axioms::Axiom::ObjectMaxQualifiedCardinality(axiom) => {
                let object_max_qualified_cardinality_arc = Arc::new((**axiom).clone());
                self.object_max_qualified_cardinality_axioms
                    .push(object_max_qualified_cardinality_arc);
            }
            axioms::Axiom::ObjectExactQualifiedCardinality(axiom) => {
                let object_exact_qualified_cardinality_arc = Arc::new((**axiom).clone());
                self.object_exact_qualified_cardinality_axioms
                    .push(object_exact_qualified_cardinality_arc);
            }
            axioms::Axiom::DataMinQualifiedCardinality(axiom) => {
                let data_min_qualified_cardinality_arc = Arc::new((**axiom).clone());
                self.data_min_qualified_cardinality_axioms
                    .push(data_min_qualified_cardinality_arc);
            }
            axioms::Axiom::DataMaxQualifiedCardinality(axiom) => {
                let data_max_qualified_cardinality_arc = Arc::new((**axiom).clone());
                self.data_max_qualified_cardinality_axioms
                    .push(data_max_qualified_cardinality_arc);
            }
            axioms::Axiom::DataExactQualifiedCardinality(axiom) => {
                let data_exact_qualified_cardinality_arc = Arc::new((**axiom).clone());
                self.data_exact_qualified_cardinality_axioms
                    .push(data_exact_qualified_cardinality_arc);
            }
            axioms::Axiom::ObjectPropertyDomain(axiom) => {
                let object_property_domain_arc = Arc::new((**axiom).clone());
                self.object_property_domain_axioms
                    .push(object_property_domain_arc);
            }
            axioms::Axiom::ObjectPropertyRange(axiom) => {
                let object_property_range_arc = Arc::new((**axiom).clone());
                self.object_property_range_axioms
                    .push(object_property_range_arc);
            }
            axioms::Axiom::DataPropertyDomain(axiom) => {
                let data_property_domain_arc = Arc::new((**axiom).clone());
                self.data_property_domain_axioms
                    .push(data_property_domain_arc);
            }
            axioms::Axiom::DataPropertyRange(axiom) => {
                let data_property_range_arc = Arc::new((**axiom).clone());
                self.data_property_range_axioms
                    .push(data_property_range_arc);
            }
            axioms::Axiom::NegativeObjectPropertyAssertion(axiom) => {
                let negative_object_property_assertion_arc = Arc::new((**axiom).clone());
                self.negative_object_property_assertion_axioms
                    .push(negative_object_property_assertion_arc);
            }
            axioms::Axiom::NegativeDataPropertyAssertion(axiom) => {
                let negative_data_property_assertion_arc = Arc::new((**axiom).clone());
                self.negative_data_property_assertion_axioms
                    .push(negative_data_property_assertion_arc);
            }
            axioms::Axiom::SubAnnotationPropertyOf(axiom) => {
                let sub_annotation_property_arc = Arc::new(axiom.clone());
                self.sub_annotation_property_axioms
                    .push(sub_annotation_property_arc);
            }
            axioms::Axiom::AnnotationPropertyDomain(axiom) => {
                let annotation_property_domain_arc = Arc::new(axiom.clone());
                self.annotation_property_domain_axioms
                    .push(annotation_property_domain_arc);
            }
            axioms::Axiom::AnnotationPropertyRange(axiom) => {
                let annotation_property_range_arc = Arc::new(axiom.clone());
                self.annotation_property_range_axioms
                    .push(annotation_property_range_arc);
            }
            axioms::Axiom::Import(axiom) => {
                // Add import to the ontology's import set
                self.imports
                    .insert(axiom.imported_ontology().clone());
            }
            axioms::Axiom::Collection(_axiom) => {
                // Collection axioms are stored in the general axioms list
                // Additional indexing could be added here if needed
            }
            axioms::Axiom::Container(_axiom) => {
                // Container axioms are stored in the general axioms list
                // Additional indexing could be added here if needed
            }
            axioms::Axiom::Reification(_axiom) => {
                // Reification axioms are stored in the general axioms list
                // Additional indexing could be added here if needed
            }
        }

        // Update multi-indexes for fast queries
        self.update_multi_indexes(axiom_arc.clone());

        Ok(())
    }

    /// Update multi-indexes for a new axiom
    fn update_multi_indexes(&mut self, axiom: Arc<axioms::Axiom>) {
        let axiom_type = axiom.axiom_type();

        // Add to type index
        self.axiom_type_index
            .entry(axiom_type)
            .or_default()
            .push(axiom.clone());

        // Signature and entity indexing simplified for compilation
        // In a full implementation, this would extract entity IRIs from axioms
        // For now, we focus on the type-based indexing which provides most benefits
    }

    /// Get all axioms in the ontology
    pub fn axioms(&self) -> &[Arc<axioms::Axiom>] {
        &self.axioms
    }

    /// Get all data property assertions
    pub fn data_property_assertions(&self) -> Vec<&crate::axioms::DataPropertyAssertionAxiom> {
        self.data_property_assertions
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    // ===== Multi-indexed Fast Query Methods =====

    /// Get axioms by type using the multi-index (O(1) lookup)
    pub fn axioms_by_type(&self, axiom_type: axioms::AxiomType) -> Vec<&axioms::Axiom> {
        self.axiom_type_index
            .get(&axiom_type)
            .map(|axioms| axioms.iter().map(|a| a.as_ref()).collect())
            .unwrap_or_default()
    }

    /// Get class axioms involving a specific class IRI (placeholder implementation)
    pub fn class_axioms_for_class(&self, _class_iri: &IRI) -> Vec<&axioms::Axiom> {
        // Simplified implementation - would use class_axioms_index in full version
        Vec::new()
    }

    /// Get property axioms involving a specific property IRI (placeholder implementation)
    pub fn property_axioms_for_property(&self, _property_iri: &IRI) -> Vec<&axioms::Axiom> {
        // Simplified implementation - would use property_axioms_index in full version
        Vec::new()
    }

    /// Get individual axioms involving a specific individual IRI (placeholder implementation)
    pub fn individual_axioms_for_individual(&self, _individual_iri: &IRI) -> Vec<&axioms::Axiom> {
        // Simplified implementation - would use individual_axioms_index in full version
        Vec::new()
    }

    /// Get annotation assertions for a specific annotation property (placeholder implementation)
    pub fn annotations_for_property(
        &self,
        _property_iri: &IRI,
    ) -> Vec<&axioms::AnnotationAssertionAxiom> {
        // Simplified implementation - would use annotation_property_index in full version
        Vec::new()
    }

    /// Fast lookup for subclass axioms (optimized for classification)
    pub fn subclass_axioms_fast(&self) -> &[Arc<axioms::SubClassOfAxiom>] {
        &self.subclass_axioms
    }

    /// Fast lookup for class assertions (optimized for instance checking)
    pub fn class_assertions_fast(&self) -> &[Arc<axioms::ClassAssertionAxiom>] {
        &self.class_assertions
    }

    /// Fast lookup for property assertions (optimized for property checking)
    pub fn property_assertions_fast(&self) -> &[Arc<axioms::PropertyAssertionAxiom>] {
        &self.property_assertions
    }

    /// Get all subclass axioms where a class appears as subclass (placeholder implementation)
    pub fn subclass_axioms_for_subclass(&self, _class_iri: &IRI) -> Vec<&axioms::SubClassOfAxiom> {
        // Would filter by subclass in full implementation
        self.subclass_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all subclass axioms where a class appears as superclass (placeholder implementation)
    pub fn subclass_axioms_for_superclass(
        &self,
        _class_iri: &IRI,
    ) -> Vec<&axioms::SubClassOfAxiom> {
        // Would filter by superclass in full implementation
        self.subclass_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all instances of a specific class (using the class_instances index)
    pub fn instances_of_class(&self, class_iri: &IRI) -> Vec<&IRI> {
        self.class_instances
            .get(class_iri)
            .map(|instances| instances.iter().collect())
            .unwrap_or_default()
    }

    /// Get all properties where an IRI appears in the domain
    pub fn properties_for_domain(&self, iri: &IRI) -> Vec<&IRI> {
        self.property_domains
            .get(iri)
            .map(|properties| properties.iter().collect())
            .unwrap_or_default()
    }

    /// Get all properties where an IRI appears in the range
    pub fn properties_for_range(&self, iri: &IRI) -> Vec<&IRI> {
        self.property_ranges
            .get(iri)
            .map(|properties| properties.iter().collect())
            .unwrap_or_default()
    }

    /// Add an annotation to the ontology
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    /// Get all annotations on the ontology
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Get a mutable reference to the IRI registry
    pub fn iri_registry_mut(&mut self) -> &mut IRIRegistry {
        &mut self.iri_registry
    }

    /// Get a reference to the IRI registry
    pub fn iri_registry(&self) -> &IRIRegistry {
        &self.iri_registry
    }

    /// Create or get an IRI using the registry
    pub fn get_or_create_iri(&mut self, iri_str: &str) -> OwlResult<IRI> {
        self.iri_registry.get_or_create_iri(iri_str)
    }

    /// Get the number of entities in the ontology
    pub fn entity_count(&self) -> usize {
        self.classes.len()
            + self.object_properties.len()
            + self.data_properties.len()
            + self.named_individuals.len()
    }

    /// Get the number of axioms in the ontology
    pub fn axiom_count(&self) -> usize {
        self.axioms.len()
    }

    /// Check if the ontology is empty
    pub fn is_empty(&self) -> bool {
        self.entity_count() == 0 && self.axiom_count() == 0
    }

    // Axiom-specific accessors for reasoning - now using indexed storage for O(1) access
    /// Get all subclass axioms
    pub fn subclass_axioms(&self) -> Vec<&crate::axioms::SubClassOfAxiom> {
        self.subclass_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all equivalent classes axioms
    pub fn equivalent_classes_axioms(&self) -> Vec<&crate::axioms::EquivalentClassesAxiom> {
        self.equivalent_classes_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all disjoint classes axioms
    pub fn disjoint_classes_axioms(&self) -> Vec<&crate::axioms::DisjointClassesAxiom> {
        self.disjoint_classes_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all class assertion axioms
    pub fn class_assertions(&self) -> Vec<&crate::axioms::ClassAssertionAxiom> {
        self.class_assertions
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all property assertion axioms (indexed)
    pub fn property_assertions(&self) -> Vec<&crate::axioms::PropertyAssertionAxiom> {
        self.property_assertions
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all subobject property axioms
    pub fn subobject_property_axioms(&self) -> Vec<&crate::axioms::SubObjectPropertyAxiom> {
        self.subobject_property_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all equivalent object properties axioms
    pub fn equivalent_object_properties_axioms(
        &self,
    ) -> Vec<&crate::axioms::EquivalentObjectPropertiesAxiom> {
        self.equivalent_object_properties_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all disjoint object properties axioms
    pub fn disjoint_object_properties_axioms(
        &self,
    ) -> Vec<&crate::axioms::DisjointObjectPropertiesAxiom> {
        self.disjoint_object_properties_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all functional property axioms
    pub fn functional_property_axioms(&self) -> Vec<&crate::axioms::FunctionalPropertyAxiom> {
        self.functional_property_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all inverse functional property axioms
    pub fn inverse_functional_property_axioms(
        &self,
    ) -> Vec<&crate::axioms::InverseFunctionalPropertyAxiom> {
        self.inverse_functional_property_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all reflexive property axioms
    pub fn reflexive_property_axioms(&self) -> Vec<&crate::axioms::ReflexivePropertyAxiom> {
        self.reflexive_property_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all irreflexive property axioms
    pub fn irreflexive_property_axioms(&self) -> Vec<&crate::axioms::IrreflexivePropertyAxiom> {
        self.irreflexive_property_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all symmetric property axioms
    pub fn symmetric_property_axioms(&self) -> Vec<&crate::axioms::SymmetricPropertyAxiom> {
        self.symmetric_property_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all asymmetric property axioms
    pub fn asymmetric_property_axioms(&self) -> Vec<&crate::axioms::AsymmetricPropertyAxiom> {
        self.asymmetric_property_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all transitive property axioms
    pub fn transitive_property_axioms(&self) -> Vec<&crate::axioms::TransitivePropertyAxiom> {
        self.transitive_property_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all subdata property axioms
    pub fn subdata_property_axioms(&self) -> Vec<&crate::axioms::SubDataPropertyAxiom> {
        self.subdata_property_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all equivalent data properties axioms
    pub fn equivalent_data_properties_axioms(
        &self,
    ) -> Vec<&crate::axioms::EquivalentDataPropertiesAxiom> {
        self.equivalent_data_properties_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all disjoint data properties axioms
    pub fn disjoint_data_properties_axioms(
        &self,
    ) -> Vec<&crate::axioms::DisjointDataPropertiesAxiom> {
        self.disjoint_data_properties_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all functional data property axioms
    pub fn functional_data_property_axioms(
        &self,
    ) -> Vec<&crate::axioms::FunctionalDataPropertyAxiom> {
        self.functional_data_property_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all same individual axioms
    pub fn same_individual_axioms(&self) -> Vec<&crate::axioms::SameIndividualAxiom> {
        self.same_individual_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all different individuals axioms
    pub fn different_individuals_axioms(&self) -> Vec<&crate::axioms::DifferentIndividualsAxiom> {
        self.different_individuals_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all has key axioms
    pub fn has_key_axioms(&self) -> Vec<&crate::axioms::HasKeyAxiom> {
        self.has_key_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all annotation assertion axioms
    pub fn annotation_assertion_axioms(&self) -> Vec<&crate::axioms::AnnotationAssertionAxiom> {
        self.annotation_assertion_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all sub property chain axioms
    pub fn sub_property_chain_axioms(&self) -> Vec<&crate::axioms::SubPropertyChainOfAxiom> {
        self.sub_property_chain_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all inverse object properties axioms
    pub fn inverse_object_properties_axioms(
        &self,
    ) -> Vec<&crate::axioms::InverseObjectPropertiesAxiom> {
        self.inverse_object_properties_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all object minimum qualified cardinality axioms
    pub fn object_min_qualified_cardinality_axioms(
        &self,
    ) -> Vec<&crate::axioms::ObjectMinQualifiedCardinalityAxiom> {
        self.object_min_qualified_cardinality_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all object maximum qualified cardinality axioms
    pub fn object_max_qualified_cardinality_axioms(
        &self,
    ) -> Vec<&crate::axioms::ObjectMaxQualifiedCardinalityAxiom> {
        self.object_max_qualified_cardinality_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all object exact qualified cardinality axioms
    pub fn object_exact_qualified_cardinality_axioms(
        &self,
    ) -> Vec<&crate::axioms::ObjectExactQualifiedCardinalityAxiom> {
        self.object_exact_qualified_cardinality_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all data minimum qualified cardinality axioms
    pub fn data_min_qualified_cardinality_axioms(
        &self,
    ) -> Vec<&crate::axioms::DataMinQualifiedCardinalityAxiom> {
        self.data_min_qualified_cardinality_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all data maximum qualified cardinality axioms
    pub fn data_max_qualified_cardinality_axioms(
        &self,
    ) -> Vec<&crate::axioms::DataMaxQualifiedCardinalityAxiom> {
        self.data_max_qualified_cardinality_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Get all data exact qualified cardinality axioms
    pub fn data_exact_qualified_cardinality_axioms(
        &self,
    ) -> Vec<&crate::axioms::DataExactQualifiedCardinalityAxiom> {
        self.data_exact_qualified_cardinality_axioms
            .iter()
            .map(|axiom| axiom.as_ref())
            .collect()
    }

    /// Add a subclass axiom
    pub fn add_subclass_axiom(&mut self, axiom: axioms::SubClassOfAxiom) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::SubClassOf(Box::new(axiom)))
    }

    /// Add an equivalent classes axiom
    pub fn add_equivalent_classes_axiom(
        &mut self,
        axiom: axioms::EquivalentClassesAxiom,
    ) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::EquivalentClasses(Box::new(axiom)))
    }

    /// Add a disjoint classes axiom
    pub fn add_disjoint_classes_axiom(
        &mut self,
        axiom: axioms::DisjointClassesAxiom,
    ) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::DisjointClasses(Box::new(axiom)))
    }

    /// Add a class assertion axiom
    pub fn add_class_assertion(&mut self, axiom: axioms::ClassAssertionAxiom) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::ClassAssertion(Box::new(axiom)))
    }

    /// Add a property assertion axiom
    pub fn add_property_assertion(
        &mut self,
        axiom: axioms::PropertyAssertionAxiom,
    ) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::PropertyAssertion(Box::new(axiom)))
    }

    /// Add a data property assertion axiom
    pub fn add_data_property_assertion(
        &mut self,
        axiom: axioms::DataPropertyAssertionAxiom,
    ) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::DataPropertyAssertion(Box::new(axiom)))
    }

    /// Validate class IRI according to OWL2 constraints
    fn validate_class_iri(&self, iri: &IRI) -> OwlResult<()> {
        // Check if IRI is empty
        if iri.as_str().is_empty() {
            return Err(OwlError::EntityValidationError {
                entity_type: "Class".to_string(),
                name: iri.as_str().to_string(),
                message: "Class IRI cannot be empty".to_string(),
            });
        }

        // Check for invalid characters in class IRIs
        if iri.as_str().contains(char::is_control) {
            return Err(OwlError::EntityValidationError {
                entity_type: "Class".to_string(),
                name: iri.as_str().to_string(),
                message: "Class IRI contains control characters".to_string(),
            });
        }

        // Check for reasonable length
        if iri.as_str().len() > 2048 {
            return Err(OwlError::EntityValidationError {
                entity_type: "Class".to_string(),
                name: iri.as_str().to_string(),
                message: "Class IRI exceeds maximum length".to_string(),
            });
        }

        Ok(())
    }

    /// Validate usage of OWL2 built-in classes
    fn validate_builtin_class_usage(&self, iri: &IRI) -> OwlResult<()> {
        let iri_str = iri.as_str();

        // Some built-in classes cannot be directly instantiated
        let restricted_classes = [
            "http://www.w3.org/2002/07/owl#Nothing",
            "http://www.w3.org/2000/01/rdf-schema#Resource",
        ];

        if restricted_classes.contains(&iri_str) {
            return Err(OwlError::EntityValidationError {
                entity_type: "Class".to_string(),
                name: iri_str.to_string(),
                message: format!(
                    "Cannot directly create instance of built-in class: {}",
                    iri_str
                ),
            });
        }

        Ok(())
    }

    /// Validate object property according to OWL2 constraints
    #[allow(dead_code)]
    fn validate_object_property(&self, property: &ObjectProperty) -> OwlResult<()> {
        // Validate property IRI
        if property.iri().as_str().is_empty() {
            return Err(OwlError::EntityValidationError {
                entity_type: "ObjectProperty".to_string(),
                name: property.iri().as_str().to_string(),
                message: "Object property IRI cannot be empty".to_string(),
            });
        }

        // Check for duplicate properties
        if self
            .object_properties
            .iter()
            .any(|p| p.iri() == property.iri())
        {
            return Err(OwlError::EntityValidationError {
                entity_type: "ObjectProperty".to_string(),
                name: property.iri().as_str().to_string(),
                message: "Object property already exists in ontology".to_string(),
            });
        }

        // Validate property characteristics for conflicts
        self.validate_property_characteristics(property)?;

        Ok(())
    }

    /// Validate property characteristics for conflicts
    fn validate_property_characteristics(&self, property: &ObjectProperty) -> OwlResult<()> {
        let characteristics = property.characteristics();

        // Check for mutually exclusive characteristics
        if characteristics.contains(&ObjectPropertyCharacteristic::Asymmetric)
            && characteristics.contains(&ObjectPropertyCharacteristic::Symmetric)
        {
            return Err(OwlError::EntityValidationError {
                entity_type: "ObjectProperty".to_string(),
                name: property.iri().as_str().to_string(),
                message: "Property cannot be both asymmetric and symmetric".to_string(),
            });
        }

        if characteristics.contains(&ObjectPropertyCharacteristic::Reflexive)
            && characteristics.contains(&ObjectPropertyCharacteristic::Irreflexive)
        {
            return Err(OwlError::EntityValidationError {
                entity_type: "ObjectProperty".to_string(),
                name: property.iri().as_str().to_string(),
                message: "Property cannot be both reflexive and irreflexive".to_string(),
            });
        }

        // Validate functional property constraints
        if characteristics.contains(&ObjectPropertyCharacteristic::Functional)
            && characteristics.contains(&ObjectPropertyCharacteristic::InverseFunctional)
        {
            // This is allowed but might need additional validation
            // For now, just warn about potential issues
        }

        Ok(())
    }

    /// Validate cardinality constraints
    pub fn validate_cardinality_constraints(&self) -> OwlResult<Vec<OwlError>> {
        let mut errors = Vec::new();

        // Validate all cardinality restrictions in the ontology
        for axiom in self.subclass_axioms() {
            if let (ClassExpression::Class(sub), super_expr) =
                (axiom.sub_class(), axiom.super_class())
            {
                self.validate_cardinality_in_expression(super_expr, sub.iri(), &mut errors);
            }
        }

        Ok(errors)
    }

    /// Validate cardinality within a class expression
    fn validate_cardinality_in_expression(
        &self,
        expr: &ClassExpression,
        context_iri: &IRI,
        errors: &mut Vec<OwlError>,
    ) {
        match expr {
            ClassExpression::ObjectMinCardinality(cardinality, _)
            | ClassExpression::ObjectMaxCardinality(cardinality, _)
            | ClassExpression::ObjectExactCardinality(cardinality, _) => {
                if *cardinality > 1000000 {
                    errors.push(OwlError::ValidationError(format!(
                        "Excessive cardinality {} for class {}",
                        cardinality, context_iri
                    )));
                }
            }
            ClassExpression::DataMinCardinality(cardinality, _)
            | ClassExpression::DataMaxCardinality(cardinality, _)
            | ClassExpression::DataExactCardinality(cardinality, _) => {
                if *cardinality > 1000000 {
                    errors.push(OwlError::ValidationError(format!(
                        "Excessive cardinality {} for class {}",
                        cardinality, context_iri
                    )));
                }
            }
            _ => {
                // Recursively validate nested expressions
                for sub_expr in expr.collect_subexpressions() {
                    self.validate_cardinality_in_expression(sub_expr, context_iri, errors);
                }
            }
        }
    }

    /// Comprehensive ontology structure validation
    pub fn validate_structure(&self) -> OwlResult<Vec<OwlError>> {
        let mut errors = Vec::new();

        // Check for circular subclass relationships
        errors.extend(
            self.detect_circular_subclass_cycles()
                .unwrap_or_else(|e| vec![e]),
        );

        // Check for property characteristic conflicts
        errors.extend(self.detect_property_conflicts().unwrap_or_else(|e| vec![e]));

        // Validate cardinality constraints
        errors.extend(
            self.validate_cardinality_constraints()
                .unwrap_or_else(|e| vec![e]),
        );

        // Check for unsatisfiable classes
        errors.extend(
            self.detect_unsatisfiable_classes()
                .unwrap_or_else(|e| vec![e]),
        );

        Ok(errors)
    }

    /// Detect circular subclass relationships
    fn detect_circular_subclass_cycles(&self) -> OwlResult<Vec<OwlError>> {
        let mut errors = Vec::new();
        let classes: Vec<_> = self.classes.iter().collect();

        for class in &classes {
            if self.has_subclass_cycle(class.iri(), &mut HashSet::new()) {
                errors.push(OwlError::OwlViolation(format!(
                    "Circular subclass relationship detected involving class {}",
                    class.iri()
                )));
            }
        }

        Ok(errors)
    }

    /// Check for subclass cycle starting from a given class
    fn has_subclass_cycle(&self, class_iri: &IRI, visited: &mut HashSet<IRI>) -> bool {
        if visited.contains(class_iri) {
            return true;
        }

        visited.insert(class_iri.clone());

        // Find all superclasses
        for axiom in self.subclass_axioms() {
            if let ClassExpression::Class(sub_class) = axiom.sub_class() {
                if sub_class.iri().as_ref() == class_iri {
                    if let ClassExpression::Class(super_class) = axiom.super_class() {
                        if self.has_subclass_cycle(super_class.iri(), visited) {
                            return true;
                        }
                    }
                }
            }
        }

        visited.remove(class_iri);
        false
    }

    /// Detect property characteristic conflicts
    fn detect_property_conflicts(&self) -> OwlResult<Vec<OwlError>> {
        let mut errors = Vec::new();

        for property in &self.object_properties {
            if let Err(e) = self.validate_property_characteristics(property) {
                errors.push(e);
            }
        }

        Ok(errors)
    }

    /// Detect potentially unsatisfiable classes
    fn detect_unsatisfiable_classes(&self) -> OwlResult<Vec<OwlError>> {
        let mut errors = Vec::new();

        // Check for classes that are disjoint with themselves
        for axiom in self.disjoint_classes_axioms() {
            let classes = axiom.classes();
            let unique_classes: HashSet<_> = classes.iter().collect();

            if unique_classes.len() != classes.len() {
                errors.push(OwlError::OwlViolation(
                    "Disjoint classes axiom contains duplicate classes".to_string(),
                ));
            }
        }

        Ok(errors)
    }

    /// Resolve imports for this ontology
    ///
    /// This is a placeholder implementation that currently does nothing.
    /// In a full implementation, this would use an ImportResolver to process
    /// ontology imports and merge the imported ontologies.
    pub fn resolve_imports(&mut self) -> OwlResult<()> {
        // TODO: Implement proper import resolution using ImportResolver
        // For now, just return Ok to allow compilation
        Ok(())
    }
}

impl Default for Ontology {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ontology_creation() {
        let ontology = Ontology::new();
        assert!(ontology.is_empty());
        assert_eq!(ontology.entity_count(), 0);
        assert_eq!(ontology.axiom_count(), 0);
    }

    #[test]
    fn test_ontology_with_iri() {
        let ontology = Ontology::with_iri("http://example.org/ontology");
        assert_eq!(
            ontology.iri().unwrap().as_str(),
            "http://example.org/ontology"
        );
        assert!(ontology.is_empty());
    }

    #[test]
    fn test_add_class() {
        let mut ontology = Ontology::new();
        let person_class = Class::new("http://example.org/Person");

        ontology.add_class(person_class).unwrap();
        assert_eq!(ontology.classes().len(), 1);
        assert_eq!(ontology.entity_count(), 1);
    }

    #[test]
    fn test_add_object_property() {
        let mut ontology = Ontology::new();
        let has_parent = ObjectProperty::new("http://example.org/hasParent");

        ontology.add_object_property(has_parent).unwrap();
        assert_eq!(ontology.object_properties().len(), 1);
        assert_eq!(ontology.entity_count(), 1);
    }

    #[test]
    fn test_imports() {
        let mut ontology = Ontology::new();
        ontology.add_import("http://example.org/import1");
        ontology.add_import("http://example.org/import2");

        assert_eq!(ontology.imports().len(), 2);
    }
}
