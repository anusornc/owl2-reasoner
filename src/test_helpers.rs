//! Test helpers module
//! 
//! This module provides common helper functions and utilities for tests.

use std::sync::Arc;
use crate::iri::IRI;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_iri() {
        let iri = create_test_iri("http://example.org/test");
        assert_eq!(iri.as_str(), "http://example.org/test");
    }
}



/// Risk level for tests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestRiskLevel {
    /// Low risk test
    Low,
    /// Medium risk test
    Medium,
    /// High risk test
    High,
    /// Critical risk test
    Critical,
}

/// Memory-safe test configuration
#[derive(Debug, Clone)]
pub struct MemorySafeTestConfig {
    /// Maximum memory allowed in bytes
    pub max_memory: usize,
    /// Risk level of the test
    pub risk_level: TestRiskLevel,
    /// Whether to enable memory guard
    pub enable_guard: bool,
}

impl Default for MemorySafeTestConfig {
    fn default() -> Self {
        Self {
            max_memory: 100 * 1024 * 1024, // 100 MB default
            risk_level: TestRiskLevel::Low,
            enable_guard: true,
        }
    }
}

impl MemorySafeTestConfig {
    /// Create a new memory-safe test configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum memory
    pub fn with_max_memory(mut self, max_memory: usize) -> Self {
        self.max_memory = max_memory;
        self
    }

    /// Set risk level
    pub fn with_risk_level(mut self, risk_level: TestRiskLevel) -> Self {
        self.risk_level = risk_level;
        self
    }

    /// Enable or disable memory guard
    pub fn with_guard(mut self, enable_guard: bool) -> Self {
        self.enable_guard = enable_guard;
        self
    }
}



/// Macro for memory-safe tests
#[macro_export]
macro_rules! memory_safe_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            let _guard = $crate::test_memory_guard::MemoryGuard::new();
            $body
        }
    };
    ($name:ident, $config:expr, $body:block) => {
        #[test]
        fn $name() {
            let _guard = $crate::test_memory_guard::MemoryGuard::new();
            let _config = $config;
            $body
        }
    };
}

/// Macro for memory-safe stress tests
#[macro_export]
macro_rules! memory_safe_stress_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            let _guard = $crate::test_memory_guard::MemoryGuard::new();
            $body
        }
    };
    (#[ignore] $name:ident, $body:block) => {
        #[test]
        #[ignore]
        fn $name() {
            let _guard = $crate::test_memory_guard::MemoryGuard::new();
            $body
        }
    };
}

/// Macro for risk-aware tests
#[macro_export]
macro_rules! risk_aware_test {
    ($name:ident, $risk_level:expr, $body:block) => {
        #[test]
        fn $name() {
            let _guard = $crate::test_memory_guard::MemoryGuard::new();
            let _config = $crate::test_helpers::MemorySafeTestConfig::new()
                .with_risk_level($risk_level);
            $body
        }
    };
}

/// Macro for memory-safe benchmark tests
#[macro_export]
macro_rules! memory_safe_bench_test {
    ($name:ident, $iterations:expr, $body:block) => {
        #[test]
        fn $name() {
            let _guard = $crate::test_memory_guard::MemoryGuard::new();
            for _i in 0..$iterations {
                $body
            }
        }
    };
}



// Re-export TestRiskLevel variants for easier access
impl TestRiskLevel {
    /// Critical risk level
    pub const CRITICAL: TestRiskLevel = TestRiskLevel::High;
}



impl MemorySafeTestConfig {
    /// Create a small memory configuration (10 MB)
    pub fn small() -> Self {
        Self {
            max_memory: 10 * 1024 * 1024,
            risk_level: TestRiskLevel::Low,
            enable_guard: true,
        }
    }

    /// Create a medium memory configuration (50 MB)
    pub fn medium() -> Self {
        Self {
            max_memory: 50 * 1024 * 1024,
            risk_level: TestRiskLevel::Medium,
            enable_guard: true,
        }
    }

    /// Create a large memory configuration (200 MB)
    pub fn large() -> Self {
        Self {
            max_memory: 200 * 1024 * 1024,
            risk_level: TestRiskLevel::High,
            enable_guard: true,
        }
    }
}



/// Type alias for backward compatibility
pub type TestMemoryConfig = MemorySafeTestConfig;

/// Type alias for backward compatibility
pub type TestMemoryGuard = crate::test_memory_guard::MemoryGuard;

/// Memory guard error type
#[derive(Debug, Clone)]
pub enum MemoryGuardError {
    /// Memory limit exceeded
    LimitExceeded(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for MemoryGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryGuardError::LimitExceeded(msg) => write!(f, "Memory limit exceeded: {}", msg),
            MemoryGuardError::Other(msg) => write!(f, "Memory guard error: {}", msg),
        }
    }
}

impl std::error::Error for MemoryGuardError {}

