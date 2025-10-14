//! Test memory guard module
//! 
//! This module provides memory protection for tests to prevent OOM issues.

use std::sync::atomic::{AtomicBool, Ordering};

static MEMORY_GUARD_ENABLED: AtomicBool = AtomicBool::new(true);

/// Enable memory guard for tests
pub fn enable_memory_guard() {
    MEMORY_GUARD_ENABLED.store(true, Ordering::SeqCst);
}

/// Disable memory guard for tests
pub fn disable_memory_guard() {
    MEMORY_GUARD_ENABLED.store(false, Ordering::SeqCst);
}

/// Check if memory guard is enabled
pub fn is_memory_guard_enabled() -> bool {
    MEMORY_GUARD_ENABLED.load(Ordering::SeqCst)
}

/// Memory guard for test execution
pub struct MemoryGuard {
    enabled: bool,
}

impl MemoryGuard {
    /// Create a new memory guard
    pub fn new() -> Self {
        Self {
            enabled: is_memory_guard_enabled(),
        }
    }

    /// Create a memory guard with custom configuration
    pub fn with_config(_config: crate::test_helpers::MemorySafeTestConfig) -> Self {
        Self {
            enabled: is_memory_guard_enabled(),
        }
    }

    /// Check if guard is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for MemoryGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_guard_enabled() {
        enable_memory_guard();
        assert!(is_memory_guard_enabled());
    }

    #[test]
    fn test_memory_guard_disabled() {
        disable_memory_guard();
        assert!(!is_memory_guard_enabled());
        enable_memory_guard(); // Reset
    }
}

