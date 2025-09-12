//! Benchmark suite for the OWL2 Reasoner
//!
//! This module contains comprehensive benchmarks for all major components
//! of the OWL2 reasoning system using the criterion benchmarking framework.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::entities::{Class, NamedIndividual};
use owl2_reasoner::axioms::{SubClassOfAxiom, ClassExpression};
use owl2_reasoner::reasoning::SimpleReasoner;
use owl2_reasoner::iri::IRI;

mod reasoning_bench;
mod parser_bench;
mod query_bench;
mod memory_bench;
mod scalability_bench;

pub use reasoning_bench::*;
pub use parser_bench::*;
pub use query_bench::*;
pub use memory_bench::*;
pub use scalability_bench::*;