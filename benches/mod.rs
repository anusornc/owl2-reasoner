//! Benchmark suite for the OWL2 Reasoner
//!
//! This module contains comprehensive benchmarks for all major components
//! of the OWL2 reasoning system using the criterion benchmarking framework.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use owl2_reasoner::axioms::{ClassExpression, SubClassOfAxiom};
use owl2_reasoner::entities::{Class, NamedIndividual};
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::SimpleReasoner;

mod memory_bench;
mod parser_bench;
mod query_bench;
mod reasoning_bench;
mod scalability_bench;
mod profile_validation_bench;

pub use memory_bench::*;
pub use parser_bench::*;
pub use query_bench::*;
pub use reasoning_bench::*;
pub use scalability_bench::*;
pub use profile_validation_bench::*;
