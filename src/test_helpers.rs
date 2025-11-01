//! Test helpers module
//!
//! This module provides common helper functions and utilities for tests.

use crate::iri::IRI;
use std::sync::Arc;

/// Helper function to create a test IRI
pub fn create_test_iri(iri_str: &str) -> Arc<IRI> {
    Arc::new(IRI::new(iri_str.to_string()).expect("Failed to create test IRI"))
}

/// Helper function to setup test environment
pub fn setup_test_env() {
    // Initialize test environment
    // TODO: Add more comprehensive setup logic
}

/// Helper function to cleanup test environment
pub fn cleanup_test_env() {
    // Cleanup test environment
    // TODO: Add more comprehensive cleanup logic
}

/// Configuration for memory-safe testing
#[derive(Debug, Clone)]
pub struct MemorySafeTestConfig {
    pub max_memory_mb: usize,
    pub timeout_seconds: u64,
}

impl Default for MemorySafeTestConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024,  // 1GB default
            timeout_seconds: 300, // 5 minutes default
        }
    }
}
