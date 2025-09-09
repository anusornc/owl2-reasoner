//! Ontology structure and management
//! 
//! Defines the main ontology structure that contains all entities and axioms.

use crate::entities::*;
use crate::axioms;
use crate::iri::{IRI, IRIRegistry};
use crate::error::OwlResult;
use std::collections::HashSet;
use std::sync::Arc;

/// An OWL2 ontology
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
        self.axioms.push(axiom_arc);
        Ok(())
    }
    
    /// Get all axioms in the ontology
    pub fn axioms(&self) -> &[Arc<axioms::Axiom>] {
        &self.axioms
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
        self.classes.len() + self.object_properties.len() + self.data_properties.len() + self.named_individuals.len()
    }
    
    /// Get the number of axioms in the ontology
    pub fn axiom_count(&self) -> usize {
        self.axioms.len()
    }
    
    /// Check if the ontology is empty
    pub fn is_empty(&self) -> bool {
        self.entity_count() == 0 && self.axiom_count() == 0
    }
    
    // Axiom-specific accessors for reasoning
    /// Get all subclass axioms
    pub fn subclass_axioms(&self) -> Vec<&crate::axioms::SubClassOfAxiom> {
        self.axioms.iter()
            .filter_map(|axiom| {
                if let crate::axioms::Axiom::SubClassOf(sub_axiom) = axiom.as_ref() {
                    Some(sub_axiom)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get all equivalent classes axioms
    pub fn equivalent_classes_axioms(&self) -> Vec<&crate::axioms::EquivalentClassesAxiom> {
        self.axioms.iter()
            .filter_map(|axiom| {
                if let crate::axioms::Axiom::EquivalentClasses(equiv_axiom) = axiom.as_ref() {
                    Some(equiv_axiom)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get all disjoint classes axioms
    pub fn disjoint_classes_axioms(&self) -> Vec<&crate::axioms::DisjointClassesAxiom> {
        self.axioms.iter()
            .filter_map(|axiom| {
                if let crate::axioms::Axiom::DisjointClasses(disjoint_axiom) = axiom.as_ref() {
                    Some(disjoint_axiom)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get all class assertion axioms
    pub fn class_assertions(&self) -> Vec<&crate::axioms::ClassAssertionAxiom> {
        self.axioms.iter()
            .filter_map(|axiom| {
                if let crate::axioms::Axiom::ClassAssertion(assertion) = axiom.as_ref() {
                    Some(assertion)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get all property assertion axioms
    pub fn property_assertions(&self) -> Vec<&crate::axioms::PropertyAssertionAxiom> {
        self.axioms.iter()
            .filter_map(|axiom| {
                if let crate::axioms::Axiom::PropertyAssertion(assertion) = axiom.as_ref() {
                    Some(assertion)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get all subproperty axioms
    pub fn subproperty_axioms(&self) -> Vec<&crate::axioms::SubObjectPropertyAxiom> {
        self.axioms.iter()
            .filter_map(|axiom| {
                if let crate::axioms::Axiom::SubObjectProperty(sub_axiom) = axiom.as_ref() {
                    Some(sub_axiom)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Add a subclass axiom
    pub fn add_subclass_axiom(&mut self, axiom: axioms::SubClassOfAxiom) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::SubClassOf(axiom))
    }
    
    /// Add an equivalent classes axiom
    pub fn add_equivalent_classes_axiom(&mut self, axiom: axioms::EquivalentClassesAxiom) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::EquivalentClasses(axiom))
    }
    
    /// Add a disjoint classes axiom
    pub fn add_disjoint_classes_axiom(&mut self, axiom: axioms::DisjointClassesAxiom) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::DisjointClasses(axiom))
    }
    
    /// Add a class assertion axiom
    pub fn add_class_assertion(&mut self, axiom: axioms::ClassAssertionAxiom) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::ClassAssertion(axiom))
    }
    
    /// Add a property assertion axiom
    pub fn add_property_assertion(&mut self, axiom: axioms::PropertyAssertionAxiom) -> OwlResult<()> {
        self.add_axiom(axioms::Axiom::PropertyAssertion(axiom))
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
        assert_eq!(ontology.iri().unwrap().as_str(), "http://example.org/ontology");
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