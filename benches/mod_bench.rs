//! Benchmark modules for OWL2 Reasoner

pub mod memory_bench;
pub mod parser_bench;
pub mod query_bench;
pub mod reasoning_bench;
pub mod scalability_bench;

pub use memory_bench::*;
pub use parser_bench::*;
pub use query_bench::*;
pub use reasoning_bench::*;
pub use scalability_bench::*;
