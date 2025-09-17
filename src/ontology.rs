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
use crate::entities::*;
use crate::error::OwlResult;
use crate::iri::{IRI, IRIRegistry};
use std::collections::{HashMap, HashSet};
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
    sub_property_chain_axioms: Vec<Arc<axioms::SubPropertyChainOfAxiom>>,
    inverse_object_properties_axioms: Vec<Arc<axioms::InverseObjectPropertiesAxiom>>,
    object_min_qualified_cardinality_axioms: Vec<Arc<axioms::ObjectMinQualifiedCardinalityAxiom>>,
    object_max_qualified_cardinality_axioms: Vec<Arc<axioms::ObjectMaxQualifiedCardinalityAxiom>>,
    object_exact_qualified_cardinality_axioms:
        Vec<Arc<axioms::ObjectExactQualifiedCardinalityAxiom>>,
    data_min_qualified_cardinality_axioms: Vec<Arc<axioms::DataMinQualifiedCardinalityAxiom>>,
    data_max_qualified_cardinality_axioms: Vec<Arc<axioms::DataMaxQualifiedCardinalityAxiom>>,
    data_exact_qualified_cardinality_axioms: Vec<Arc<axioms::DataExactQualifiedCardinalityAxiom>>,

    // Performance indexes
    class_instances: HashMap<IRI, Vec<IRI>>,
    property_domains: HashMap<IRI, Vec<IRI>>,
    property_ranges: HashMap<IRI, Vec<IRI>>,
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
            sub_property_chain_axioms: Vec::new(),
            inverse_object_properties_axioms: Vec::new(),
            object_min_qualified_cardinality_axioms: Vec::new(),
            object_max_qualified_cardinality_axioms: Vec::new(),
            object_exact_qualified_cardinality_axioms: Vec::new(),
            data_min_qualified_cardinality_axioms: Vec::new(),
            data_max_qualified_cardinality_axioms: Vec::new(),
            data_exact_qualified_cardinality_axioms: Vec::new(),
            class_instances: HashMap::new(),
            property_domains: HashMap::new(),
            property_ranges: HashMap::new(),
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

    /// Get all named individuals in the ontology
    pub fn named_individuals(&self) -> &HashSet<Arc<NamedIndividual>> {
        &self.named_individuals
    }

    /// Add an axiom to the ontology
    pub fn add_axiom(&mut self, axiom: axioms::Axiom) -> OwlResult<()> {
        let axiom_arc = Arc::new(axiom);

        // Add to general axioms list
        self.axioms.push(axiom_arc.clone());

        // Add to indexed storage based on axiom type
        match axiom_arc.as_ref() {
            axioms::Axiom::SubClassOf(axiom) => {
                let subclass_arc = Arc::new(axiom.clone());
                self.subclass_axioms.push(subclass_arc);
            }
            axioms::Axiom::EquivalentClasses(axiom) => {
                let equiv_arc = Arc::new(axiom.clone());
                self.equivalent_classes_axioms.push(equiv_arc);
            }
            axioms::Axiom::DisjointClasses(axiom) => {
                let disjoint_arc = Arc::new(axiom.clone());
                self.disjoint_classes_axioms.push(disjoint_arc);
            }
            axioms::Axiom::ClassAssertion(axiom) => {
                let assertion_arc = Arc::new(axiom.clone());
                self.class_assertions.push(assertion_arc);
                // Update class instances index
                if let Some(class_iri) = axiom.class_expr().as_named().map(|c| c.iri().clone()) {
                    self.class_instances
                        .entry(axiom.individual().clone())
                        .or_default()
                        .push(class_iri);
                }
            }
            axioms::Axiom::PropertyAssertion(axiom) => {
                let assertion_arc = Arc::new(axiom.clone());
                self.property_assertions.push(assertion_arc);
                // Update property domains and ranges indexes
                self.property_domains
                    .entry(axiom.property().clone())
                    .or_default()
                    .push(axiom.subject().clone());
                self.property_ranges
                    .entry(axiom.property().clone())
                    .or_default()
                    .push(axiom.object().clone());
            }
            axioms::Axiom::DataPropertyAssertion(axiom) => {
                let assertion_arc = Arc::new(axiom.clone());
                self.data_property_assertions.push(assertion_arc);
                // We don't index literals into property_ranges (IRI-only index)
                self.property_domains
                    .entry(axiom.property().clone())
                    .or_default()
                    .push(axiom.subject().clone());
            }
            axioms::Axiom::SubObjectProperty(axiom) => {
                let subprop_arc = Arc::new(axiom.clone());
                self.subobject_property_axioms.push(subprop_arc);
            }
            axioms::Axiom::EquivalentObjectProperties(axiom) => {
                let equiv_arc = Arc::new(axiom.clone());
                self.equivalent_object_properties_axioms.push(equiv_arc);
            }
            axioms::Axiom::DisjointObjectProperties(axiom) => {
                let disjoint_arc = Arc::new(axiom.clone());
                self.disjoint_object_properties_axioms.push(disjoint_arc);
            }
            axioms::Axiom::FunctionalProperty(axiom) => {
                let functional_arc = Arc::new(axiom.clone());
                self.functional_property_axioms.push(functional_arc);
            }
            axioms::Axiom::InverseFunctionalProperty(axiom) => {
                let inv_functional_arc = Arc::new(axiom.clone());
                self.inverse_functional_property_axioms
                    .push(inv_functional_arc);
            }
            axioms::Axiom::ReflexiveProperty(axiom) => {
                let reflexive_arc = Arc::new(axiom.clone());
                self.reflexive_property_axioms.push(reflexive_arc);
            }
            axioms::Axiom::IrreflexiveProperty(axiom) => {
                let irreflexive_arc = Arc::new(axiom.clone());
                self.irreflexive_property_axioms.push(irreflexive_arc);
            }
            axioms::Axiom::SymmetricProperty(axiom) => {
                let symmetric_arc = Arc::new(axiom.clone());
                self.symmetric_property_axioms.push(symmetric_arc);
            }
            axioms::Axiom::AsymmetricProperty(axiom) => {
                let asymmetric_arc = Arc::new(axiom.clone());
                self.asymmetric_property_axioms.push(asymmetric_arc);
            }
            axioms::Axiom::TransitiveProperty(axiom) => {
                let transitive_arc = Arc::new(axiom.clone());
                self.transitive_property_axioms.push(transitive_arc);
            }
            axioms::Axiom::SubDataProperty(axiom) => {
                let subdata_arc = Arc::new(axiom.clone());
                self.subdata_property_axioms.push(subdata_arc);
            }
            axioms::Axiom::EquivalentDataProperties(axiom) => {
                let equiv_data_arc = Arc::new(axiom.clone());
                self.equivalent_data_properties_axioms.push(equiv_data_arc);
            }
            axioms::Axiom::DisjointDataProperties(axiom) => {
                let disjoint_data_arc = Arc::new(axiom.clone());
                self.disjoint_data_properties_axioms.push(disjoint_data_arc);
            }
            axioms::Axiom::FunctionalDataProperty(axiom) => {
                let functional_data_arc = Arc::new(axiom.clone());
                self.functional_data_property_axioms
                    .push(functional_data_arc);
            }
            axioms::Axiom::SameIndividual(axiom) => {
                let same_individual_arc = Arc::new(axiom.clone());
                self.same_individual_axioms.push(same_individual_arc);
            }
            axioms::Axiom::DifferentIndividuals(axiom) => {
                let different_individuals_arc = Arc::new(axiom.clone());
                self.different_individuals_axioms
                    .push(different_individuals_arc);
            }
            axioms::Axiom::HasKey(axiom) => {
                let has_key_arc = Arc::new(axiom.clone());
                self.has_key_axioms.push(has_key_arc);
            }
            axioms::Axiom::AnnotationAssertion(axiom) => {
                let annotation_assertion_arc = Arc::new(axiom.clone());
                self.annotation_assertion_axioms
                    .push(annotation_assertion_arc);
            }
            axioms::Axiom::SubPropertyChainOf(axiom) => {
                let sub_property_chain_arc = Arc::new(axiom.clone());
                self.sub_property_chain_axioms.push(sub_property_chain_arc);
            }
            axioms::Axiom::InverseObjectProperties(axiom) => {
                let inverse_object_properties_arc = Arc::new(axiom.clone());
                self.inverse_object_properties_axioms
                    .push(inverse_object_properties_arc);
            }
            axioms::Axiom::ObjectMinQualifiedCardinality(axiom) => {
                let object_min_qualified_cardinality_arc = Arc::new(axiom.clone());
                self.object_min_qualified_cardinality_axioms
                    .push(object_min_qualified_cardinality_arc);
            }
            axioms::Axiom::ObjectMaxQualifiedCardinality(axiom) => {
                let object_max_qualified_cardinality_arc = Arc::new(axiom.clone());
                self.object_max_qualified_cardinality_axioms
                    .push(object_max_qualified_cardinality_arc);
            }
            axioms::Axiom::ObjectExactQualifiedCardinality(axiom) => {
                let object_exact_qualified_cardinality_arc = Arc::new(axiom.clone());
                self.object_exact_qualified_cardinality_axioms
                    .push(object_exact_qualified_cardinality_arc);
            }
            axioms::Axiom::DataMinQualifiedCardinality(axiom) => {
                let data_min_qualified_cardinality_arc = Arc::new(axiom.clone());
                self.data_min_qualified_cardinality_axioms
                    .push(data_min_qualified_cardinality_arc);
            }
            axioms::Axiom::DataMaxQualifiedCardinality(axiom) => {
                let data_max_qualified_cardinality_arc = Arc::new(axiom.clone());
                self.data_max_qualified_cardinality_axioms
                    .push(data_max_qualified_cardinality_arc);
            }
            axioms::Axiom::DataExactQualifiedCardinality(axiom) => {
                let data_exact_qualified_cardinality_arc = Arc::new(axiom.clone());
                self.data_exact_qualified_cardinality_axioms
                    .push(data_exact_qualified_cardinality_arc);
            }
        }

        Ok(())
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
        self.add_axiom(axioms::Axiom::SubClassOf(axiom))
    }

    /// Add an equivalent classes axiom
    pub fn add_equivalent_classes_axiom(
        &mut self,
        axiom: axioms::EquivalentClassesAxiom,
    ) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::EquivalentClasses(axiom))
    }

    /// Add a disjoint classes axiom
    pub fn add_disjoint_classes_axiom(
        &mut self,
        axiom: axioms::DisjointClassesAxiom,
    ) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::DisjointClasses(axiom))
    }

    /// Add a class assertion axiom
    pub fn add_class_assertion(&mut self, axiom: axioms::ClassAssertionAxiom) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::ClassAssertion(axiom))
    }

    /// Add a property assertion axiom
    pub fn add_property_assertion(
        &mut self,
        axiom: axioms::PropertyAssertionAxiom,
    ) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::PropertyAssertion(axiom))
    }

    /// Add a data property assertion axiom
    pub fn add_data_property_assertion(
        &mut self,
        axiom: axioms::DataPropertyAssertionAxiom,
    ) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::DataPropertyAssertion(axiom))
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
