//! Comprehensive test suite module

pub mod comprehensive;
pub mod error_handling;
pub mod stress_tests;
pub mod negative_tests;
pub mod integration_tests;
pub mod profile_validation_tests;

pub use comprehensive::*;
pub use error_handling::*;
pub use stress_tests::*;
pub use negative_tests::*;
pub use integration_tests::*;
pub use profile_validation_tests::*;