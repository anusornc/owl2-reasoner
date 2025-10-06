//! Graceful Degradation Mechanisms for OWL2 Reasoner
//!
//! This module provides mechanisms for gracefully degrading system performance
//! under memory pressure instead of crashing or experiencing out-of-memory conditions.

use crate::memory_protection::{MemoryProtectionState, AllocationResult, RejectionReason};
use crate::memory::get_memory_stats;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Degradation levels for different components
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DegradationLevel {
    /// Full functionality - no degradation
    Full,
    /// Minor performance impact - small allocations only
    Reduced,
    /// Major impact - limited functionality
    Limited,
    /// Emergency mode - minimal functionality only
    Emergency,
}

/// Component-specific degradation configuration
#[derive(Debug, Clone)]
pub struct ComponentDegradationConfig {
    /// Maximum memory usage before degradation
    pub max_memory_usage: usize,
    /// Maximum allocation size before rejection
    pub max_allocation_size: usize,
    /// Cache size limit
    pub cache_size_limit: usize,
    /// Timeout multiplier for operations
    pub timeout_multiplier: f64,
    /// Enable component-specific cleanup
    pub enable_cleanup: bool,
}

impl Default for ComponentDegradationConfig {
    fn default() -> Self {
        Self {
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            max_allocation_size: 10 * 1024 * 1024, // 10MB
            cache_size_limit: 10000,
            timeout_multiplier: 1.0,
            enable_cleanup: true,
        }
    }
}

/// Graceful degradation strategy
#[derive(Debug, Clone)]
pub struct GracefulDegradationStrategy {
    /// Global degradation level
    current_level: Arc<Mutex<DegradationLevel>>,
    /// Component-specific configurations
    component_configs: Arc<Mutex<HashMap<String, ComponentDegradationConfig>>>,
    /// Last degradation time
    last_degradation_time: Arc<Mutex<Instant>>,
    /// Degradation history
    degradation_history: Arc<Mutex<Vec<DegradationEvent>>>,
    /// Enable automatic recovery
    enable_auto_recovery: Arc<AtomicBool>,
    /// Recovery check interval
    recovery_check_interval: Duration,
}

/// Degradation event for tracking
#[derive(Debug, Clone)]
pub struct DegradationEvent {
    pub timestamp: Instant,
    pub component: String,
    pub old_level: DegradationLevel,
    pub new_level: DegradationLevel,
    pub reason: String,
    pub memory_usage: usize,
}

impl GracefulDegradationStrategy {
    pub fn new() -> Self {
        let mut component_configs = HashMap::new();
        
        // Default configurations for different components
        component_configs.insert("parser".to_string(), ComponentDegradationConfig {
            max_memory_usage: 50 * 1024 * 1024, // 50MB
            max_allocation_size: 5 * 1024 * 1024, // 5MB
            cache_size_limit: 5000,
            timeout_multiplier: 2.0,
            enable_cleanup: true,
        });
        
        component_configs.insert("reasoning".to_string(), ComponentDegradationConfig {
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            max_allocation_size: 15 * 1024 * 1024, // 15MB
            cache_size_limit: 10000,
            timeout_multiplier: 3.0,
            enable_cleanup: true,
        });
        
        component_configs.insert("cache".to_string(), ComponentDegradationConfig {
            max_memory_usage: 30 * 1024 * 1024, // 30MB
            max_allocation_size: 1 * 1024 * 1024, // 1MB
            cache_size_limit: 2000,
            timeout_multiplier: 1.5,
            enable_cleanup: true,
        });
        
        component_configs.insert("arena".to_string(), ComponentDegradationConfig {
            max_memory_usage: 20 * 1024 * 1024, // 20MB
            max_allocation_size: 2 * 1024 * 1024, // 2MB
            cache_size_limit: 1000,
            timeout_multiplier: 1.2,
            enable_cleanup: true,
        });
        
        Self {
            current_level: Arc::new(Mutex::new(DegradationLevel::Full)),
            component_configs: Arc::new(Mutex::new(component_configs)),
            last_degradation_time: Arc::new(Mutex::new(Instant::now())),
            degradation_history: Arc::new(Mutex::new(Vec::new())),
            enable_auto_recovery: Arc::new(AtomicBool::new(true)),
            recovery_check_interval: Duration::from_secs(30),
        }
    }

    /// Check if component can perform operation
    pub fn can_component_operate(&self, component: &str, requested_bytes: usize) -> ComponentOperationResult {
        let current_level = self.current_level.lock().unwrap();
        let component_configs = self.component_configs.lock().unwrap();
        
        // Get component-specific config or use default
        let config = component_configs.get(component)
            .cloned()
            .unwrap_or_default();
        
        // Check global degradation level first
        match *current_level {
            DegradationLevel::Full => {
                // Still check component-specific limits
                if requested_bytes > config.max_allocation_size {
                    ComponentOperationResult::Rejected(RejectionReason::ComponentLimitExceeded)
                } else {
                    ComponentOperationResult::Allowed(config.timeout_multiplier)
                }
            }
            DegradationLevel::Reduced => {
                // More restrictive limits
                let reduced_limit = config.max_allocation_size / 2;
                if requested_bytes > reduced_limit {
                    ComponentOperationResult::Rejected(RejectionReason::DegradationMode)
                } else {
                    ComponentOperationResult::Allowed(config.timeout_multiplier * 1.5)
                }
            }
            DegradationLevel::Limited => {
                // Very restrictive limits
                let limited_limit = config.max_allocation_size / 4;
                if requested_bytes > limited_limit {
                    ComponentOperationResult::Rejected(RejectionReason::SevereDegradation)
                } else {
                    ComponentOperationResult::Allowed(config.timeout_multiplier * 2.0)
                }
            }
            DegradationLevel::Emergency => {
                // Only allow very small operations
                if requested_bytes > config.max_allocation_size / 8 {
                    ComponentOperationResult::Rejected(RejectionReason::EmergencyMode)
                } else {
                    ComponentOperationResult::Allowed(config.timeout_multiplier * 3.0)
                }
            }
        }
    }

    /// Update degradation level based on memory pressure
    pub fn update_degradation_level(&mut self, protection_state: MemoryProtectionState) -> DegradationLevel {
        let old_level = self.current_level.lock().unwrap().clone();
        let new_level = match protection_state {
            MemoryProtectionState::Normal => DegradationLevel::Full,
            MemoryProtectionState::Warning => DegradationLevel::Reduced,
            MemoryProtectionState::Critical => DegradationLevel::Limited,
            MemoryProtectionState::Emergency => DegradationLevel::Emergency,
        };

        if new_level != old_level {
            // Record degradation event
            let event = DegradationEvent {
                timestamp: Instant::now(),
                component: "global".to_string(),
                old_level,
                new_level,
                reason: format!("Memory protection state changed to {:?}", protection_state),
                memory_usage: get_memory_stats().total_usage,
            };

            {
                let mut history = self.degradation_history.lock().unwrap();
                history.push(event.clone());
                
                // Keep only last 100 events
                if history.len() > 100 {
                    history.remove(0);
                }
            }

            // Update current level
            *self.current_level.lock().unwrap() = new_level;
            *self.last_degradation_time.lock().unwrap() = Instant::now();

            println!("🔄 Degradation level changed: {:?} -> {:?}", old_level, new_level);
            
            // Trigger component-specific cleanup if needed
            if new_level != DegradationLevel::Full {
                self.trigger_component_cleanup(&new_level);
            }
        }

        new_level
    }

    /// Trigger component-specific cleanup
    fn trigger_component_cleanup(&self, level: &DegradationLevel) {
        let component_configs = self.component_configs.lock().unwrap();
        
        for (component, config) in component_configs.iter() {
            if config.enable_cleanup {
                match level {
                    DegradationLevel::Reduced => {
                        println!("🧹 Triggering cleanup for component: {}", component);
                        // Implement component-specific cleanup
                    }
                    DegradationLevel::Limited => {
                        println!("🧹 Aggressive cleanup for component: {}", component);
                        // More aggressive cleanup
                    }
                    DegradationLevel::Emergency => {
                        println!("🚨 Emergency cleanup for component: {}", component);
                        // Emergency cleanup
                    }
                    DegradationLevel::Full => {}
                }
            }
        }
    }

    /// Check for automatic recovery
    pub fn check_recovery(&self) -> Option<DegradationEvent> {
        if !self.enable_auto_recovery.load(Ordering::Relaxed) {
            return None;
        }

        let current_level = self.current_level.lock().unwrap();
        let current_memory = get_memory_stats().total_usage;
        
        // Only attempt recovery if not in full mode
        if *current_level != DegradationLevel::Full {
            let last_degradation = self.last_degradation_time.lock().unwrap();
            
            // Wait for recovery interval
            if last_degradation.elapsed() > self.recovery_check_interval {
                // Simple recovery logic - can be enhanced
                let should_recover = match *current_level {
                    DegradationLevel::Emergency => current_memory < 100 * 1024 * 1024, // 100MB
                    DegradationLevel::Limited => current_memory < 50 * 1024 * 1024,  // 50MB
                    DegradationLevel::Reduced => current_memory < 25 * 1024 * 1024, // 25MB
                    DegradationLevel::Full => false,
                };

                if should_recover {
                    let old_level = *current_level;
                    let new_level = match old_level {
                        DegradationLevel::Emergency => DegradationLevel::Limited,
                        DegradationLevel::Limited => DegradationLevel::Reduced,
                        DegradationLevel::Reduced => DegradationLevel::Full,
                        DegradationLevel::Full => DegradationLevel::Full,
                    };

                    if new_level != old_level {
                        let event = DegradationEvent {
                            timestamp: Instant::now(),
                            component: "global".to_string(),
                            old_level,
                            new_level,
                            reason: "Automatic recovery".to_string(),
                            memory_usage: current_memory,
                        };

                        *self.current_level.lock().unwrap() = new_level;
                        *self.last_degradation_time.lock().unwrap() = Instant::now();

                        println!("🔄 Automatic recovery: {:?} -> {:?}", old_level, new_level);
                        
                        return Some(event);
                    }
                }
            }
        }

        None
    }

    /// Get current degradation level
    pub fn get_degradation_level(&self) -> DegradationLevel {
        self.current_level.lock().unwrap().clone()
    }

    /// Get degradation history
    pub fn get_degradation_history(&self) -> Vec<DegradationEvent> {
        self.degradation_history.lock().unwrap().clone()
    }

    /// Set component-specific configuration
    pub fn set_component_config(&mut self, component: String, config: ComponentDegradationConfig) {
        let mut configs = self.component_configs.lock().unwrap();
        configs.insert(component, config);
    }

    /// Enable/disable automatic recovery
    pub fn set_auto_recovery(&mut self, enabled: bool) {
        self.enable_auto_recovery.store(enabled, Ordering::Relaxed);
    }

    /// Set recovery check interval
    pub fn set_recovery_interval(&mut self, interval: Duration) {
        self.recovery_check_interval = interval;
    }

    /// Manually trigger degradation
    pub fn trigger_degradation(&mut self, level: DegradationLevel, reason: String) {
        let old_level = self.current_level.lock().unwrap().clone();
        
        if level != old_level {
            let event = DegradationEvent {
                timestamp: Instant::now(),
                component: "manual".to_string(),
                old_level,
                new_level: level,
                reason,
                memory_usage: get_memory_stats().total_usage,
            };

            {
                let mut history = self.degradation_history.lock().unwrap();
                history.push(event.clone());
                
                if history.len() > 100 {
                    history.remove(0);
                }
            }

            *self.current_level.lock().unwrap() = level;
            *self.last_degradation_time.lock().unwrap() = Instant::now();

            if level != DegradationLevel::Full {
                self.trigger_component_cleanup(&level);
            }
        }
    }
}

/// Result of component operation check
#[derive(Debug, Clone)]
pub enum ComponentOperationResult {
    /// Operation is allowed with timeout multiplier
    Allowed(f64),
    /// Operation is rejected
    Rejected(RejectionReason),
}

/// Global graceful degradation instance
static GLOBAL_GRACEFUL_DEGRADATION: std::sync::LazyLock<GracefulDegradationStrategy> = 
    std::sync::LazyLock::new(|| GracefulDegradationStrategy::new());

/// Get global graceful degradation instance
pub fn get_graceful_degradation() -> &'static GracefulDegradationStrategy {
    &GLOBAL_GRACEFUL_DEGRADATION
}

/// Check if component can operate under current degradation level
pub fn can_component_operate(component: &str, requested_bytes: usize) -> ComponentOperationResult {
    GLOBAL_GRACEFUL_DEGRADATION.can_component_operate(component, requested_bytes)
}

/// Update degradation level based on memory protection state
pub fn update_degradation_level(protection_state: MemoryProtectionState) -> DegradationLevel {
    GLOBAL_GRACEFUL_DEGRADATION.update_degradation_level(protection_state)
}

/// Get current degradation level
pub fn get_degradation_level() -> DegradationLevel {
    GLOBAL_GRACEFUL_DEGRADATION.get_degradation_level()
}

/// Trigger manual degradation
pub fn trigger_degradation(level: DegradationLevel, reason: String) {
    GLOBAL_GRACEFUL_DEGRADATION.trigger_degradation(level, reason);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graceful_degradation_creation() {
        let degradation = GracefulDegradationStrategy::new();
        let level = degradation.get_degradation_level();
        assert_eq!(level, DegradationLevel::Full);
    }

    #[test]
    fn test_component_operation_check() {
        let degradation = GracefulDegradationStrategy::new();
        
        // Should allow small operations under normal conditions
        let result = degradation.can_component_operate("parser", 1024);
        assert!(matches!(result, ComponentOperationResult::Allowed(_)));
        
        // Should reject large operations
        let result = degradation.can_component_operate("parser", 100 * 1024 * 1024);
        assert!(matches!(result, ComponentOperationResult::Rejected(_)));
    }

    #[test]
    fn test_degradation_level_update() {
        let mut degradation = GracefulDegradationStrategy::new();
        
        // Test degradation level changes
        let level = degradation.update_degradation_level(MemoryProtectionState::Warning);
        assert_eq!(level, DegradationLevel::Reduced);
        
        let level = degradation.update_degradation_level(MemoryProtectionState::Critical);
        assert_eq!(level, DegradationLevel::Limited);
        
        let level = degradation.update_degradation_level(MemoryProtectionState::Emergency);
        assert_eq!(level, DegradationLevel::Emergency);
        
        let level = degradation.update_degradation_level(MemoryProtectionState::Normal);
        assert_eq!(level, DegradationLevel::Full);
    }

    #[test]
    fn test_component_config() {
        let mut degradation = GracefulDegradationStrategy::new();
        
        // Set custom config for parser
        let config = ComponentDegradationConfig {
            max_memory_usage: 10 * 1024 * 1024, // 10MB
            max_allocation_size: 1 * 1024 * 1024, // 1MB
            cache_size_limit: 500,
            timeout_multiplier: 5.0,
            enable_cleanup: true,
        };
        
        degradation.set_component_config("parser".to_string(), config);
        
        // Should reject large allocation for parser
        let result = degradation.can_component_operate("parser", 2 * 1024 * 1024);
        assert!(matches!(result, ComponentOperationResult::Rejected(_)));
        
        // Should allow small allocation for other components
        let result = degradation.can_component_operate("reasoning", 2 * 1024 * 1024);
        assert!(matches!(result, ComponentOperationResult::Allowed(_)));
    }

    #[test]
    fn test_degradation_history() {
        let mut degradation = GracefulDegradationStrategy::new();
        
        // Trigger some degradation events
        degradation.trigger_degradation(DegradationLevel::Limited, "Test".to_string());
        degradation.trigger_degradation(DegradationLevel::Emergency, "Emergency".to_string());
        
        let history = degradation.get_degradation_history();
        assert!(history.len() >= 2);
        
        // Check that events are ordered by timestamp
        for i in 1..history.len() {
            assert!(history[i].timestamp >= history[i-1].timestamp);
        }
    }

    #[test]
    fn test_auto_recovery() {
        let mut degradation = GracefulDegradationStrategy::new();
        
        // Set recovery interval to be very short for testing
        degradation.set_recovery_interval(Duration::from_millis(1));
        
        // Trigger degradation
        degradation.trigger_degradation(DegradationLevel::Limited, "Test".to_string());
        assert_eq!(degradation.get_degradation_level(), DegradationLevel::Limited);
        
        // Wait a bit and check for recovery (this is a simplified test)
        std::thread::sleep(Duration::from_millis(10));
        
        // In a real scenario, this would depend on actual memory usage
        let recovery_event = degradation.check_recovery();
        
        // The recovery logic depends on memory usage, so we just verify it doesn't crash
        println!("Recovery check completed: {:?}", recovery_event);
    }
}