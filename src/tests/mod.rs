//! Comprehensive test suite module
//! All tests are now memory-safe and will fail gracefully before causing OOM.

pub mod aggressive_memory_test;
pub mod blank_node_tests;
pub mod collection_tests;
pub mod comma_test;
pub mod comprehensive;
pub mod comprehensive_axiom_coverage_test;
pub mod comprehensive_test_runner;
pub mod concurrency;
pub mod container_tests;
pub mod cross_format_parser_tests;
pub mod debug_tokenizer_test;
// Temporarily commented out due to missing implementations
pub mod documentation_verification;
pub mod error_handling;
pub mod integration_tests;
pub mod integration_validation;
pub mod memory_limit_test;
pub mod memory_safety_validation;
pub mod memory_stress_tests;
pub mod negative_tests;
pub mod owl_functional_annotation_tests;
pub mod owl_functional_data_property_tests;
pub mod owl_functional_data_range_tests;
pub mod performance_regression_tests;
pub mod profile_validation_tests;
pub mod rdf_constructs_comprehensive_test;
pub mod rdf_constructs_performance_test;
pub mod rdf_xml_blank_node_tests;
pub mod regression_validation;
pub mod reification_tests;
pub mod stress_tests;
pub mod test_setup;

pub use comprehensive::*;
