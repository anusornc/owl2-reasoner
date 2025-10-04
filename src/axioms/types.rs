//! Core types and utilities for OWL2 axioms
//!
//! This module contains shared types, enums, and utility functions
//! used across different axiom modules.

use crate::entities::{Annotation, AnonymousIndividual};
use crate::iri::IRI;
use crate::{constants::*, OwlError, OwlResult};
use std::sync::Arc;

/// Helper function to create IRIs safely with proper error handling
pub fn create_iri_safe(iri_str: &str) -> OwlResult<Arc<IRI>> {
    IRI::new_optimized(iri_str).map_err(|_| OwlError::IriCreationError {
        iri_str: iri_str.to_string(),
    })
}

/// Helper function to create blank node IRIs safely
pub fn create_blank_node_iri(node_id: &str) -> OwlResult<Arc<IRI>> {
    IRI::new_optimized(format!("{}{}", BLANK_NODE_PREFIX, node_id)).map_err(|_| {
        OwlError::IriCreationError {
            iri_str: format!("{}{}", BLANK_NODE_PREFIX, node_id),
        }
    })
}

/// Object value for property assertions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropertyAssertionObject {
    /// Named individual (IRI)
    Named(Arc<IRI>),
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

impl AxiomType {
    /// Get a human-readable name for the axiom type
    pub fn name(&self) -> &'static str {
        match self {
            AxiomType::SubClassOf => "SubClassOf",
            AxiomType::EquivalentClasses => "EquivalentClasses",
            AxiomType::DisjointClasses => "DisjointClasses",
            AxiomType::ClassAssertion => "ClassAssertion",
            AxiomType::PropertyAssertion => "PropertyAssertion",
            AxiomType::DataPropertyAssertion => "DataPropertyAssertion",
            AxiomType::SubObjectProperty => "SubObjectProperty",
            AxiomType::EquivalentObjectProperties => "EquivalentObjectProperties",
            AxiomType::DisjointObjectProperties => "DisjointObjectProperties",
            AxiomType::FunctionalProperty => "FunctionalProperty",
            AxiomType::InverseFunctionalProperty => "InverseFunctionalProperty",
            AxiomType::ReflexiveProperty => "ReflexiveProperty",
            AxiomType::IrreflexiveProperty => "IrreflexiveProperty",
            AxiomType::SymmetricProperty => "SymmetricProperty",
            AxiomType::AsymmetricProperty => "AsymmetricProperty",
            AxiomType::TransitiveProperty => "TransitiveProperty",
            AxiomType::SubPropertyChainOf => "SubPropertyChainOf",
            AxiomType::InverseObjectProperties => "InverseObjectProperties",
            AxiomType::SubDataProperty => "SubDataProperty",
            AxiomType::EquivalentDataProperties => "EquivalentDataProperties",
            AxiomType::DisjointDataProperties => "DisjointDataProperties",
            AxiomType::FunctionalDataProperty => "FunctionalDataProperty",
            AxiomType::SameIndividual => "SameIndividual",
            AxiomType::DifferentIndividuals => "DifferentIndividuals",
            AxiomType::HasKey => "HasKey",
            AxiomType::AnnotationAssertion => "AnnotationAssertion",
            AxiomType::SubAnnotationPropertyOf => "SubAnnotationPropertyOf",
            AxiomType::AnnotationPropertyDomain => "AnnotationPropertyDomain",
            AxiomType::AnnotationPropertyRange => "AnnotationPropertyRange",
            AxiomType::ObjectMinQualifiedCardinality => "ObjectMinQualifiedCardinality",
            AxiomType::ObjectMaxQualifiedCardinality => "ObjectMaxQualifiedCardinality",
            AxiomType::ObjectExactQualifiedCardinality => "ObjectExactQualifiedCardinality",
            AxiomType::DataMinQualifiedCardinality => "DataMinQualifiedCardinality",
            AxiomType::DataMaxQualifiedCardinality => "DataMaxQualifiedCardinality",
            AxiomType::DataExactQualifiedCardinality => "DataExactQualifiedCardinality",
            AxiomType::ObjectPropertyDomain => "ObjectPropertyDomain",
            AxiomType::ObjectPropertyRange => "ObjectPropertyRange",
            AxiomType::DataPropertyDomain => "DataPropertyDomain",
            AxiomType::DataPropertyRange => "DataPropertyRange",
            AxiomType::NegativeObjectPropertyAssertion => "NegativeObjectPropertyAssertion",
            AxiomType::NegativeDataPropertyAssertion => "NegativeDataPropertyAssertion",
            AxiomType::Import => "Import",
            AxiomType::Collection => "Collection",
            AxiomType::Container => "Container",
            AxiomType::Reification => "Reification",
        }
    }
}