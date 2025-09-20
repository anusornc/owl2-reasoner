//! OWL2 Entities - Classes, Properties, and Individuals
//!
//! This module defines the core entities of OWL2 ontologies including classes,
//! object properties, data properties, annotations, and individuals.

use crate::cache::{BoundedCache, BoundedCacheStatsSnapshot};
use crate::error::OwlResult;
use crate::iri::IRI;
use once_cell::sync::Lazy;
use smallvec::SmallVec;
use std::collections::HashSet;
use std::sync::Arc;

/// Global entity cache for sharing IRIs across all entities with size limits
static GLOBAL_ENTITY_CACHE: Lazy<BoundedCache<String, Arc<IRI>>> = Lazy::new(|| {
    let config = BoundedCache::<String, Arc<IRI>>::builder()
        .max_size(5_000) // Smaller limit for entities
        .enable_stats(true)
        .enable_memory_pressure(true)
        .memory_pressure_threshold(0.75)
        .cleanup_interval(std::time::Duration::from_secs(120))
        .build();
    BoundedCache::with_config(config)
});

/// Get a shared Arc<IRI> from the global cache
fn get_shared_iri<S: Into<String>>(iri: S) -> OwlResult<Arc<IRI>> {
    let iri_str = iri.into();

    // Check if we already have this IRI cached
    if let Ok(Some(cached_iri)) = GLOBAL_ENTITY_CACHE.get(&iri_str) {
        return Ok(cached_iri);
    }

    // Create new IRI (which will use global cache internally)
    let iri = IRI::new(iri_str.clone())?;
    let arc_iri = Arc::new(iri);

    // Cache it for future use with automatic eviction
    if let Err(e) = GLOBAL_ENTITY_CACHE.insert(iri_str, arc_iri.clone()) {
        // Log warning but don't fail - entity creation is critical
        eprintln!("Warning: Failed to cache entity IRI: {}", e);
    }

    Ok(arc_iri)
}

/// Get global entity cache statistics
pub fn global_entity_cache_stats() -> BoundedCacheStatsSnapshot {
    GLOBAL_ENTITY_CACHE.stats()
}

/// Clear the global entity cache
pub fn clear_global_entity_cache() -> OwlResult<()> {
    GLOBAL_ENTITY_CACHE.clear()?;
    Ok(())
}

/// Force eviction of N entries from global entity cache
pub fn force_global_entity_cache_eviction(count: usize) -> OwlResult<usize> {
    let current_size = GLOBAL_ENTITY_CACHE.len()?;
    let target_size = current_size.saturating_sub(count);
    let evicted = current_size - target_size;
    Ok(evicted)
}


/// A named class in OWL2
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Class {
    /// The IRI of the class
    iri: Arc<IRI>,
    /// Annotations associated with this class
    annotations: SmallVec<[Annotation; 4]>,
}

impl Class {
    /// Create a new class with the given IRI
    pub fn new<I: Into<IRI> + Clone>(iri: I) -> Self {
        // For backward compatibility, fall back to direct creation if sharing fails
        let iri_clone = iri.clone();
        let shared_iri =
            get_shared_iri(iri.into().as_str()).unwrap_or_else(|_| Arc::new(iri_clone.into()));

        Class {
            iri: shared_iri,
            annotations: SmallVec::new(),
        }
    }

    /// Create a new class with shared IRI (preferred for memory efficiency)
    pub fn new_shared<S: Into<String>>(iri: S) -> OwlResult<Self> {
        Ok(Class {
            iri: get_shared_iri(iri)?,
            annotations: SmallVec::new(),
        })
    }

    /// Get the IRI of this class
    pub fn iri(&self) -> &IRI {
        &self.iri
    }

    /// Get the annotations for this class
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Add an annotation to this class
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    /// Check if this is a built-in OWL class
    pub fn is_builtin(&self) -> bool {
        self.iri.is_owl() && matches!(self.iri.local_name(), "Thing" | "Nothing" | "Class")
    }

    /// Check if this is owl:Thing (the top class)
    pub fn is_thing(&self) -> bool {
        self.iri.as_str() == "http://www.w3.org/2002/07/owl#Thing"
    }

    /// Check if this is owl:Nothing (the bottom class)
    pub fn is_nothing(&self) -> bool {
        self.iri.as_str() == "http://www.w3.org/2002/07/owl#Nothing"
    }
}

/// An object property in OWL2
#[derive(Debug, Clone)]
pub struct ObjectProperty {
    /// The IRI of the property
    iri: Arc<IRI>,
    /// Annotations associated with this property
    annotations: SmallVec<[Annotation; 4]>,
    /// Property characteristics
    characteristics: HashSet<ObjectPropertyCharacteristic>,
}

impl ObjectProperty {
    /// Create a new object property with the given IRI
    pub fn new<I: Into<IRI> + Clone>(iri: I) -> Self {
        // For backward compatibility, fall back to direct creation if sharing fails
        let iri_clone = iri.clone();
        let shared_iri =
            get_shared_iri(iri.into().as_str()).unwrap_or_else(|_| Arc::new(iri_clone.into()));

        ObjectProperty {
            iri: shared_iri,
            annotations: SmallVec::new(),
            characteristics: HashSet::new(),
        }
    }

    /// Create a new object property with shared IRI (preferred for memory efficiency)
    pub fn new_shared<S: Into<String>>(iri: S) -> OwlResult<Self> {
        Ok(ObjectProperty {
            iri: get_shared_iri(iri)?,
            annotations: SmallVec::new(),
            characteristics: HashSet::new(),
        })
    }

    /// Get the IRI of this property
    pub fn iri(&self) -> &IRI {
        &self.iri
    }

    /// Get the annotations for this property
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Get the characteristics of this property
    pub fn characteristics(&self) -> &HashSet<ObjectPropertyCharacteristic> {
        &self.characteristics
    }

    /// Add an annotation to this property
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    /// Add a characteristic to this property
    pub fn add_characteristic(&mut self, characteristic: ObjectPropertyCharacteristic) {
        self.characteristics.insert(characteristic);
    }

    /// Check if this property has a specific characteristic
    pub fn has_characteristic(&self, characteristic: ObjectPropertyCharacteristic) -> bool {
        self.characteristics.contains(&characteristic)
    }

    /// Check if this property is functional
    pub fn is_functional(&self) -> bool {
        self.has_characteristic(ObjectPropertyCharacteristic::Functional)
    }

    /// Check if this property is inverse functional
    pub fn is_inverse_functional(&self) -> bool {
        self.has_characteristic(ObjectPropertyCharacteristic::InverseFunctional)
    }

    /// Check if this property is transitive
    pub fn is_transitive(&self) -> bool {
        self.has_characteristic(ObjectPropertyCharacteristic::Transitive)
    }

    /// Check if this property is symmetric
    pub fn is_symmetric(&self) -> bool {
        self.has_characteristic(ObjectPropertyCharacteristic::Symmetric)
    }

    /// Check if this property is asymmetric
    pub fn is_asymmetric(&self) -> bool {
        self.has_characteristic(ObjectPropertyCharacteristic::Asymmetric)
    }

    /// Check if this property is reflexive
    pub fn is_reflexive(&self) -> bool {
        self.has_characteristic(ObjectPropertyCharacteristic::Reflexive)
    }

    /// Check if this property is irreflexive
    pub fn is_irreflexive(&self) -> bool {
        self.has_characteristic(ObjectPropertyCharacteristic::Irreflexive)
    }
}

/// Characteristics of object properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectPropertyCharacteristic {
    /// Functional property (each subject has at most one object)
    Functional,
    /// Inverse functional property (each object has at most one subject)
    InverseFunctional,
    /// Transitive property (if R(a,b) and R(b,c) then R(a,c))
    Transitive,
    /// Symmetric property (if R(a,b) then R(b,a))
    Symmetric,
    /// Asymmetric property (if R(a,b) then not R(b,a))
    Asymmetric,
    /// Reflexive property (R(a,a) for all a)
    Reflexive,
    /// Irreflexive property (not R(a,a) for all a)
    Irreflexive,
}

/// A data property in OWL2
#[derive(Debug, Clone)]
pub struct DataProperty {
    /// The IRI of the property
    iri: Arc<IRI>,
    /// Annotations associated with this property
    annotations: SmallVec<[Annotation; 4]>,
    /// Property characteristics
    characteristics: HashSet<DataPropertyCharacteristic>,
}

impl PartialEq for ObjectProperty {
    fn eq(&self, other: &Self) -> bool {
        self.iri == other.iri
    }
}

impl Eq for ObjectProperty {}

impl std::hash::Hash for ObjectProperty {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.iri.hash(state);
    }
}

impl DataProperty {
    /// Create a new data property with the given IRI
    pub fn new<I: Into<IRI> + Clone>(iri: I) -> Self {
        // For backward compatibility, fall back to direct creation if sharing fails
        let iri_clone = iri.clone();
        let shared_iri =
            get_shared_iri(iri.into().as_str()).unwrap_or_else(|_| Arc::new(iri_clone.into()));

        DataProperty {
            iri: shared_iri,
            annotations: SmallVec::new(),
            characteristics: HashSet::new(),
        }
    }

    /// Create a new data property with shared IRI (preferred for memory efficiency)
    pub fn new_shared<S: Into<String>>(iri: S) -> OwlResult<Self> {
        Ok(DataProperty {
            iri: get_shared_iri(iri)?,
            annotations: SmallVec::new(),
            characteristics: HashSet::new(),
        })
    }

    /// Get the IRI of this property
    pub fn iri(&self) -> &IRI {
        &self.iri
    }

    /// Get the annotations for this property
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Get the characteristics of this property
    pub fn characteristics(&self) -> &HashSet<DataPropertyCharacteristic> {
        &self.characteristics
    }

    /// Add an annotation to this property
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    /// Add a characteristic to this property
    pub fn add_characteristic(&mut self, characteristic: DataPropertyCharacteristic) {
        self.characteristics.insert(characteristic);
    }

    /// Check if this property has a specific characteristic
    pub fn has_characteristic(&self, characteristic: DataPropertyCharacteristic) -> bool {
        self.characteristics.contains(&characteristic)
    }

    /// Check if this property is functional
    pub fn is_functional(&self) -> bool {
        self.has_characteristic(DataPropertyCharacteristic::Functional)
    }
}

impl PartialEq for DataProperty {
    fn eq(&self, other: &Self) -> bool {
        self.iri == other.iri
    }
}

impl Eq for DataProperty {}

impl std::hash::Hash for DataProperty {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.iri.hash(state);
    }
}

/// Characteristics of data properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataPropertyCharacteristic {
    /// Functional property (each subject has at most one value)
    Functional,
}

/// A named individual in OWL2
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedIndividual {
    /// The IRI of the individual
    iri: Arc<IRI>,
    /// Annotations associated with this individual
    annotations: SmallVec<[Annotation; 4]>,
}

impl NamedIndividual {
    /// Create a new named individual with the given IRI
    pub fn new<I: Into<IRI> + Clone>(iri: I) -> Self {
        // For backward compatibility, fall back to direct creation if sharing fails
        let iri_clone = iri.clone();
        let shared_iri =
            get_shared_iri(iri.into().as_str()).unwrap_or_else(|_| Arc::new(iri_clone.into()));

        NamedIndividual {
            iri: shared_iri,
            annotations: SmallVec::new(),
        }
    }

    /// Create a new named individual with shared IRI (preferred for memory efficiency)
    pub fn new_shared<S: Into<String>>(iri: S) -> OwlResult<Self> {
        Ok(NamedIndividual {
            iri: get_shared_iri(iri)?,
            annotations: SmallVec::new(),
        })
    }

    /// Get the IRI of this individual
    pub fn iri(&self) -> &IRI {
        &self.iri
    }

    /// Get the annotations for this individual
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Add an annotation to this individual
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }
}

/// An annotation in OWL2
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Annotation {
    /// The annotation property
    property: Arc<IRI>,
    /// The annotation value
    value: AnnotationValue,
}

impl Annotation {
    /// Create a new annotation
    pub fn new<P: Into<IRI>, V: Into<AnnotationValue>>(property: P, value: V) -> Self {
        Annotation {
            property: Arc::new(property.into()),
            value: value.into(),
        }
    }

    /// Get the annotation property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the annotation value
    pub fn value(&self) -> &AnnotationValue {
        &self.value
    }
}

/// Annotation values in OWL2
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnnotationValue {
    /// IRI reference
    IRI(IRI),
    /// Literal value
    Literal(Literal),
    /// Anonymous individual
    AnonymousIndividual(String),
}

impl From<IRI> for AnnotationValue {
    fn from(iri: IRI) -> Self {
        AnnotationValue::IRI(iri)
    }
}

impl From<Literal> for AnnotationValue {
    fn from(literal: Literal) -> Self {
        AnnotationValue::Literal(literal)
    }
}

impl From<String> for AnnotationValue {
    fn from(s: String) -> Self {
        AnnotationValue::Literal(Literal::simple(s))
    }
}

impl From<&str> for AnnotationValue {
    fn from(s: &str) -> Self {
        AnnotationValue::Literal(Literal::simple(s.to_string()))
    }
}

/// A literal value in OWL2
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal {
    /// The lexical value
    lexical_form: String,
    /// The datatype IRI
    datatype: IRI,
    /// Optional language tag
    language_tag: Option<String>,
}

impl Literal {
    /// Create a simple string literal
    pub fn simple<S: Into<String>>(value: S) -> Self {
        Literal {
            lexical_form: value.into(),
            datatype: IRI::new("http://www.w3.org/2001/XMLSchema#string").unwrap(),
            language_tag: None,
        }
    }

    /// Create a typed literal
    pub fn typed<S: Into<String>, D: Into<IRI>>(value: S, datatype: D) -> Self {
        Literal {
            lexical_form: value.into(),
            datatype: datatype.into(),
            language_tag: None,
        }
    }

    /// Create a language-tagged literal
    pub fn lang_tagged<S: Into<String>, L: Into<String>>(value: S, language: L) -> Self {
        Literal {
            lexical_form: value.into(),
            datatype: IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#langString").unwrap(),
            language_tag: Some(language.into()),
        }
    }

    /// Get the lexical form of the literal
    pub fn lexical_form(&self) -> &str {
        &self.lexical_form
    }

    /// Get the datatype of the literal
    pub fn datatype(&self) -> &IRI {
        &self.datatype
    }

    /// Get the language tag of the literal
    pub fn language_tag(&self) -> Option<&str> {
        self.language_tag.as_deref()
    }

    /// Check if this is a plain literal (no datatype or language tag)
    pub fn is_plain(&self) -> bool {
        self.datatype.as_str() == "http://www.w3.org/2001/XMLSchema#string"
            && self.language_tag.is_none()
    }

    /// Check if this is a language-tagged literal
    pub fn is_lang_tagged(&self) -> bool {
        self.language_tag.is_some()
    }

    /// Check if this is a typed literal
    pub fn is_typed(&self) -> bool {
        !self.is_plain() && !self.is_lang_tagged()
    }
}

/// Anonymous individual (blank node)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnonymousIndividual {
    /// The node ID
    node_id: String,
    /// Annotations associated with this individual
    annotations: SmallVec<[Annotation; 4]>,
}

impl AnonymousIndividual {
    /// Create a new anonymous individual with the given node ID
    pub fn new<S: Into<String>>(node_id: S) -> Self {
        AnonymousIndividual {
            node_id: node_id.into(),
            annotations: SmallVec::new(),
        }
    }

    /// Get the node ID of this individual
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Get the annotations for this individual
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Add an annotation to this individual
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }
}

/// Any individual (named or anonymous)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Individual {
    /// Named individual
    Named(NamedIndividual),
    /// Anonymous individual
    Anonymous(AnonymousIndividual),
}

impl From<NamedIndividual> for Individual {
    fn from(individual: NamedIndividual) -> Self {
        Individual::Named(individual)
    }
}

impl From<AnonymousIndividual> for Individual {
    fn from(individual: AnonymousIndividual) -> Self {
        Individual::Anonymous(individual)
    }
}

impl Individual {
    /// Get the IRI of this individual if it's named
    pub fn iri(&self) -> Option<&IRI> {
        match self {
            Individual::Named(named) => Some(named.iri()),
            Individual::Anonymous(_) => None,
        }
    }

    /// Get the node ID of this individual if it's anonymous
    pub fn node_id(&self) -> Option<&str> {
        match self {
            Individual::Named(_) => None,
            Individual::Anonymous(anonymous) => Some(anonymous.node_id()),
        }
    }

    /// Get the annotations for this individual
    pub fn annotations(&self) -> &[Annotation] {
        match self {
            Individual::Named(named) => named.annotations(),
            Individual::Anonymous(anonymous) => anonymous.annotations(),
        }
    }

    /// Add an annotation to this individual
    pub fn add_annotation(&mut self, annotation: Annotation) {
        match self {
            Individual::Named(named) => named.add_annotation(annotation),
            Individual::Anonymous(anonymous) => anonymous.add_annotation(annotation),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_creation() {
        let person_class = Class::new("http://example.org/Person");
        assert_eq!(person_class.iri().as_str(), "http://example.org/Person");
        assert!(!person_class.is_builtin());
    }

    #[test]
    fn test_owl_classes() {
        let thing = Class::new("http://www.w3.org/2002/07/owl#Thing");
        let nothing = Class::new("http://www.w3.org/2002/07/owl#Nothing");

        assert!(thing.is_thing());
        assert!(nothing.is_nothing());
    }

    #[test]
    fn test_object_property() {
        let has_parent = ObjectProperty::new("http://example.org/hasParent");
        assert_eq!(has_parent.iri().as_str(), "http://example.org/hasParent");
        assert!(!has_parent.is_transitive());

        let mut has_ancestor = ObjectProperty::new("http://example.org/hasAncestor");
        has_ancestor.add_characteristic(ObjectPropertyCharacteristic::Transitive);
        assert!(has_ancestor.is_transitive());
    }

    #[test]
    fn test_data_property() {
        let has_age = DataProperty::new("http://example.org/hasAge");
        assert_eq!(has_age.iri().as_str(), "http://example.org/hasAge");
        assert!(!has_age.is_functional());

        let mut has_name = DataProperty::new("http://example.org/hasName");
        has_name.add_characteristic(DataPropertyCharacteristic::Functional);
        assert!(has_name.is_functional());
    }

    #[test]
    fn test_literal() {
        let simple_lit = Literal::simple("hello");
        assert!(simple_lit.is_plain());

        let typed_lit = Literal::typed("42", "http://www.w3.org/2001/XMLSchema#integer");
        assert!(typed_lit.is_typed());

        let lang_lit = Literal::lang_tagged("bonjour", "fr");
        assert!(lang_lit.is_lang_tagged());
    }

    #[test]
    fn test_annotation() {
        let annotation = Annotation::new(
            "http://www.w3.org/2000/01/rdf-schema#comment",
            "A person class",
        );

        assert_eq!(
            annotation.property().as_str(),
            "http://www.w3.org/2000/01/rdf-schema#comment"
        );
        if let AnnotationValue::Literal(lit) = annotation.value() {
            assert_eq!(lit.lexical_form(), "A person class");
        } else {
            panic!("Expected literal value");
        }
    }
}
