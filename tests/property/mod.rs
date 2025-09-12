//! Property-based testing suite for the OWL2 Reasoner
//!
//! This module contains property-based tests that systematically verify
//! the correctness and robustness of the OWL2 reasoner implementation.

use proptest::prelude::*;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::entities::{Class, NamedIndividual};
use owl2_reasoner::axioms::{SubClassOfAxiom, ClassExpression};
use owl2_reasoner::reasoning::SimpleReasoner;
use owl2_reasoner::iri::IRI;

mod iri_properties;
mod ontology_properties;
mod reasoning_properties;

pub use iri_properties::*;
pub use ontology_properties::*;
pub use reasoning_properties::*;