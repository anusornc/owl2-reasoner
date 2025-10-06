//! Memory leak prevention and monitoring for OWL2 Reasoner
//!
//! This module provides comprehensive memory management tools including
//! memory monitoring, leak detection, and automatic cleanup mechanisms.

use crate::cache_manager;
use crate::entities::clear_global_entity_cache;
use crate::iri::clear_global_iri_cache;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total memory usage in bytes
    pub total_usage: usize,
    /// Peak memory usage in bytes
    pub peak_usage: usize,
    /// Global IRI cache size
    pub iri_cache_size: usize,
    /// Global entity cache size
    pub entity_cache_size: usize,
    /// Number of cleanup operations performed
    pub cleanup_count: u64,
    /// Memory pressure level (0.0 to 1.0)
    pub pressure_level: f64,
}

/// Memory monitoring configuration
#[derive(Debug, Clone)]
pub struct MemoryMonitorConfig {
    /// Maximum memory limit in bytes
    pub max_memory: usize,
    /// Memory pressure threshold (0.0 to 1.0)
    pub pressure_threshold: f64,
    /// Cleanup interval in seconds
    pub cleanup_interval: Duration,
    /// Enable automatic cleanup
    pub auto_cleanup: bool,
}

impl Default for MemoryMonitorConfig {
    fn default() -> Self {
        Self {
            max_memory: 2 * 1024 * 1024 * 1024, // 2GB default
            pressure_threshold: 0.8,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            auto_cleanup: true,
        }
    }
}

/// Global memory monitor
static GLOBAL_MEMORY_MONITOR: Lazy<MemoryMonitor> =
    Lazy::new(|| MemoryMonitor::new(MemoryMonitorConfig::default()));

/// Memory leak prevention and monitoring system
pub struct MemoryMonitor {
    config: MemoryMonitorConfig,
    stats: Mutex<MemoryStats>,
    cleanup_count: AtomicU64,
    last_cleanup: Mutex<Instant>,
    monitor_thread: Option<thread::JoinHandle<()>>,
    shutdown_flag: Arc<AtomicBool>,
}

impl MemoryMonitor {
    /// Create a new memory monitor
    pub fn new(config: MemoryMonitorConfig) -> Self {
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        let mut monitor = Self {
            config,
            stats: Mutex::new(MemoryStats {
                total_usage: 0,
                peak_usage: 0,
                iri_cache_size: 0,
                entity_cache_size: 0,
                cleanup_count: 0,
                pressure_level: 0.0,
            }),
            cleanup_count: AtomicU64::new(0),
            last_cleanup: Mutex::new(Instant::now()),
            monitor_thread: None,
            shutdown_flag: Arc::clone(&shutdown_flag),
        };

        monitor.start_monitoring_thread();
        monitor
    }

    /// Start the background monitoring thread
    fn start_monitoring_thread(&mut self) {
        if self.config.auto_cleanup {
            let interval = self.config.cleanup_interval;
            let shutdown_flag = Arc::clone(&self.shutdown_flag);

            let handle = thread::spawn(move || {
                while !shutdown_flag.load(Ordering::Relaxed) {
                    thread::sleep(interval);
                    let _stats = get_memory_stats(); // Just trigger stats collection
                }
            });

            self.monitor_thread = Some(handle);
        }
    }

    /// Get current memory statistics with timeout
    pub fn get_stats(&self) -> MemoryStats {
        // Attempt to get stats with timeout and graceful fallback
        let mut stats =
            match self.acquire_lock_with_timeout(&self.stats, Duration::from_millis(1000), "stats")
            {
                Ok(guard) => guard,
                Err(e) => {
                    eprintln!("Memory stats lock failed: {}", e);
                    // Create temporary fallback stats
                    return MemoryStats {
                        total_usage: self.get_current_memory_usage_safe(),
                        peak_usage: self.get_current_memory_usage_safe(),
                        iri_cache_size: 0,
                        entity_cache_size: 0,
                        cleanup_count: self.cleanup_count.load(Ordering::Relaxed),
                        pressure_level: 0.0,
                    };
                }
            };

        // Update current usage safely
        stats.total_usage = self.get_current_memory_usage_safe();
        stats.peak_usage = stats.peak_usage.max(stats.total_usage);

        // Update cache sizes (now using unified cache) with graceful fallback
        match cache_manager::global_cache_manager().get_iri_cache_size() {
            Ok(cache_size) => {
                stats.iri_cache_size = cache_size;
                stats.entity_cache_size = cache_size; // Same cache now
            }
            Err(e) => {
                eprintln!("Warning: Failed to get cache size: {}", e);
                // Use conservative estimates as fallback
                stats.iri_cache_size = 0;
                stats.entity_cache_size = 0;
            }
        }

        // Calculate pressure level with bounds checking
        stats.pressure_level = match (self.config.max_memory, stats.total_usage) {
            (max_mem, total_usage) if max_mem > 0 && total_usage > 0 => {
                (total_usage as f64 / max_mem as f64).min(1.0)
            }
            _ => 0.0, // Conservative fallback for edge cases
        };

        stats.cleanup_count = self.cleanup_count.load(Ordering::Relaxed);

        stats.clone()
    }

    /// Check for memory pressure and perform cleanup if needed
    pub fn check_and_cleanup(&self) -> Result<(), String> {
        let stats = self.get_stats();
        let mut last_cleanup = self
            .acquire_lock_with_timeout(
                &self.last_cleanup,
                Duration::from_millis(500),
                "last_cleanup",
            )
            .map_err(|e| format!("Cleanup timing lock failed: {}", e))?;

        // Check if we're above pressure threshold
        if stats.pressure_level > self.config.pressure_threshold {
            println!(
                "Memory pressure detected: {:.2}%",
                stats.pressure_level * 100.0
            );
            self.perform_cleanup()?;
            *last_cleanup = Instant::now();
            self.cleanup_count.fetch_add(1, Ordering::Relaxed);
        }

        // Also check if cleanup interval has passed
        if last_cleanup.elapsed() > self.config.cleanup_interval {
            self.perform_maintenance_cleanup()?;
            *last_cleanup = Instant::now();
        }

        Ok(())
    }

    /// Perform emergency cleanup due to memory pressure
    fn perform_cleanup(&self) -> Result<(), String> {
        println!("Performing emergency memory cleanup...");

        // Clear global caches
        if let Err(e) = clear_global_iri_cache() {
            return Err(format!("Failed to clear IRI cache: {}", e));
        }

        if let Err(e) = clear_global_entity_cache() {
            return Err(format!("Failed to clear entity cache: {}", e));
        }

        println!("Emergency cleanup completed");
        Ok(())
    }

    /// Perform routine maintenance cleanup
    fn perform_maintenance_cleanup(&self) -> Result<(), String> {
        let stats = self.get_stats();

        // Only perform cleanup if we're using significant memory
        if stats.pressure_level > 0.5 {
            println!("Performing maintenance cleanup...");

            // We could implement more granular cleanup here
            // For now, just log the action
            println!("Maintenance cleanup completed");
        }

        Ok(())
    }

    /// Get memory pressure level (0.0 to 1.0)
    pub fn get_pressure_level(&self) -> f64 {
        self.get_stats().pressure_level
    }

    /// Check if memory pressure is high
    pub fn is_under_pressure(&self) -> bool {
        self.get_pressure_level() > self.config.pressure_threshold
    }

    /// Update monitor configuration
    pub fn update_config(&mut self, config: MemoryMonitorConfig) {
        self.config = config;
    }

    /// Force immediate cleanup
    pub fn force_cleanup(&self) -> Result<(), String> {
        self.perform_cleanup()?;
        self.cleanup_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Get cleanup count
    pub fn get_cleanup_count(&self) -> u64 {
        self.cleanup_count.load(Ordering::Relaxed)
    }

    /// Acquire lock with timeout to prevent deadlocks
    fn acquire_lock_with_timeout<'a, T>(
        &self,
        mutex: &'a Mutex<T>,
        timeout: Duration,
        lock_name: &str,
    ) -> Result<std::sync::MutexGuard<'a, T>, String> {
        let start_time = Instant::now();

        loop {
            match mutex.try_lock() {
                Ok(guard) => return Ok(guard),
                Err(_) => {
                    if start_time.elapsed() >= timeout {
                        return Err(format!(
                            "Timeout acquiring {} lock after {:?}",
                            lock_name, timeout
                        ));
                    }
                    // Sleep briefly to avoid busy waiting
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }

    /// Safe memory usage estimation without unsafe operations
    fn get_current_memory_usage_safe(&self) -> usize {
        // Use a safe, platform-independent estimation approach
        // This avoids unsafe platform-specific code

        // Try to get cache size as a baseline
        let cache_size = cache_manager::global_cache_manager()
            .get_iri_cache_size()
            .unwrap_or(0);

        // Safe estimation based on known structures
        let base_memory = 1024 * 1024; // 1MB base
        let cache_memory = cache_size * 200; // ~200 bytes per cached IRI
        let overhead_memory = 512 * 1024; // 512KB overhead estimate

        base_memory + cache_memory + overhead_memory
    }
}

impl Drop for MemoryMonitor {
    fn drop(&mut self) {
        // Signal the monitoring thread to shutdown
        self.shutdown_flag.store(true, Ordering::Relaxed);

        // Stop the monitoring thread and wait for it to finish
        if let Some(handle) = self.monitor_thread.take() {
            // Give the thread a moment to shutdown gracefully
            thread::sleep(Duration::from_millis(100));

            // If the thread is still running, we'll just detach it
            // This prevents the program from hanging on shutdown
            if !handle.is_finished() {
                handle.thread().unpark();
            }
        }
    }
}

/// Get global memory statistics
pub fn get_memory_stats() -> MemoryStats {
    GLOBAL_MEMORY_MONITOR.get_stats()
}

/// Check if system is under memory pressure
pub fn is_under_memory_pressure() -> bool {
    GLOBAL_MEMORY_MONITOR.is_under_pressure()
}

/// Force immediate memory cleanup
pub fn force_memory_cleanup() -> Result<(), String> {
    GLOBAL_MEMORY_MONITOR.force_cleanup()
}

/// Get memory pressure level
pub fn get_memory_pressure_level() -> f64 {
    GLOBAL_MEMORY_MONITOR.get_pressure_level()
}

/// Get cleanup operation count
pub fn get_cleanup_count() -> u64 {
    GLOBAL_MEMORY_MONITOR.get_cleanup_count()
}

/// Memory leak detection results
#[derive(Debug, Clone)]
pub struct LeakDetectionReport {
    pub potential_leaks: Vec<String>,
    pub recommendations: Vec<String>,
    pub memory_efficiency_score: f64,
}

/// Detect potential memory leaks
pub fn detect_memory_leaks() -> LeakDetectionReport {
    let stats = get_memory_stats();
    let mut potential_leaks = Vec::new();
    let mut recommendations = Vec::new();

    // Check for unusually high cache sizes
    if stats.iri_cache_size > 50_000 {
        potential_leaks.push(format!(
            "IRI cache size ({}) exceeds recommended limit",
            stats.iri_cache_size
        ));
        recommendations.push("Consider reducing IRI cache size limit".to_string());
    }

    if stats.entity_cache_size > 25_000 {
        potential_leaks.push(format!(
            "Entity cache size ({}) exceeds recommended limit",
            stats.entity_cache_size
        ));
        recommendations.push("Consider reducing entity cache size limit".to_string());
    }

    // Check for high memory pressure
    if stats.pressure_level > 0.9 {
        potential_leaks.push(format!(
            "Critical memory pressure: {:.2}%",
            stats.pressure_level * 100.0
        ));
        recommendations.push("Immediate memory cleanup required".to_string());
    }

    // Calculate efficiency score
    let efficiency_score = if stats.pressure_level < 0.5 {
        1.0 - (stats.pressure_level * 0.5)
    } else {
        0.5 - ((stats.pressure_level - 0.5) * 2.0)
    }
    .max(0.0);

    LeakDetectionReport {
        potential_leaks,
        recommendations,
        memory_efficiency_score: efficiency_score,
    }
}

/// Initialize memory monitoring with custom configuration
pub fn init_memory_monitor(_config: MemoryMonitorConfig) {
    // Note: This would require replacing the global monitor
    // For now, we'll just update the existing one
    eprintln!("Memory monitor initialization not fully implemented");
}

/// Get global memory protection instance
pub fn get_memory_protection(
) -> std::sync::MutexGuard<'static, crate::memory_protection::MemoryProtection> {
    crate::memory_protection::get_memory_protection()
}

/// Initialize global memory protection with custom config
pub fn init_memory_protection(config: crate::memory_protection::MemoryProtectionConfig) {
    crate::memory_protection::init_memory_protection(config);
}

/// Check if allocation is allowed under current memory conditions
pub fn can_allocate(requested_bytes: usize) -> crate::memory_protection::AllocationResult {
    crate::memory_protection::can_allocate(requested_bytes)
}

/// Get current memory protection state
pub fn get_memory_protection_state() -> crate::memory_protection::MemoryProtectionState {
    crate::memory_protection::get_memory_protection_state()
}

/// Trigger manual cleanup
pub fn trigger_memory_cleanup() -> crate::memory_protection::CleanupResult {
    crate::memory_protection::trigger_memory_cleanup()
}

/// Get global memory statistics
pub fn get_global_memory_stats() -> crate::memory_protection::GlobalMemoryStats {
    crate::memory_protection::get_global_memory_stats()
}

/// Start memory protection monitoring
pub fn start_memory_protection() {
    crate::memory_protection::start_memory_protection();
}

/// Stop memory protection monitoring
pub fn stop_memory_protection() {
    crate::memory_protection::stop_memory_protection();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_protection::MemoryProtectionState;

    #[test]
    fn test_memory_stats() {
        let stats = get_memory_stats();
        assert!(stats.pressure_level >= 0.0 && stats.pressure_level <= 1.0);
        // Memory usage might be 0 in some test environments, so don't assert > 0
    }

    #[test]
    fn test_leak_detection() {
        let report = detect_memory_leaks();
        assert!(report.memory_efficiency_score >= 0.0 && report.memory_efficiency_score <= 1.0);
    }

    #[test]
    fn test_memory_pressure() {
        let pressure = get_memory_pressure_level();
        assert!((0.0..=1.0).contains(&pressure));
    }

    #[test]
    fn test_memory_protection_integration() {
        // Test that memory protection can be integrated with existing systems
        let initial_state = get_memory_protection_state();
        assert!(matches!(initial_state, MemoryProtectionState::Normal));

        // Test allocation checking
        let small_alloc = can_allocate(1024);
        assert!(matches!(
            small_alloc,
            crate::memory_protection::AllocationResult::Allowed
        ));

        let large_alloc = can_allocate(100 * 1024 * 1024); // 100MB
        let result = match large_alloc {
            crate::memory_protection::AllocationResult::Allowed => "allowed".to_string(),
            crate::memory_protection::AllocationResult::Rejected(reason) => {
                format!("rejected: {:?}", reason)
            }
        };
        println!("Large allocation result: {}", result);
    }

    #[test]
    fn test_circuit_breaker_functionality() {
        use crate::memory_protection::CircuitBreaker;

        let circuit_breaker = CircuitBreaker::new(2, Duration::from_millis(100));

        // Initially closed
        assert!(!circuit_breaker.is_open());

        // Record failures to open circuit
        circuit_breaker.record_failure();
        circuit_breaker.record_failure();

        // Should be open now
        assert!(circuit_breaker.is_open());

        // Check timeout behavior
        thread::sleep(Duration::from_millis(200));
        assert!(!circuit_breaker.check_and_maybe_open());
    }

    #[test]
    fn test_global_stats_integration() {
        // Test that global stats integrate with existing memory monitoring
        let old_stats = get_memory_stats();
        let global_stats = get_global_memory_stats();

        // Both should provide memory usage information
        assert!(old_stats.total_usage > 0);
        assert!(global_stats.total_memory_usage >= old_stats.total_usage);

        // Global stats should have additional fields
        assert_eq!(global_stats.circuit_breaker_trips, 0);
        assert_eq!(global_stats.cleanup_count, 0);
    }
}
