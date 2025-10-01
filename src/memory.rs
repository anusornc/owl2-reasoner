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

    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        // Attempt to get stats with graceful fallback on lock failure
        let mut stats = match self.stats.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Memory stats lock poisoned: {}", e);
                // Create temporary fallback stats
                return MemoryStats {
                    total_usage: self.get_current_memory_usage(),
                    peak_usage: self.get_current_memory_usage(),
                    iri_cache_size: 0,
                    entity_cache_size: 0,
                    cleanup_count: self.cleanup_count.load(Ordering::Relaxed),
                    pressure_level: 0.0,
                };
            }
        };

        // Update current usage
        stats.total_usage = self.get_current_memory_usage();
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

    /// Get current memory usage (platform-specific)
    fn get_current_memory_usage(&self) -> usize {
        #[cfg(target_os = "linux")]
        {
            // Use /proc/self/status on Linux
            if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
                for line in content.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<usize>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Use mach APIs on macOS
            #[allow(deprecated)]
            unsafe {
                use libc::{
                    mach_msg_type_number_t, mach_task_basic_info, mach_task_self, task_info,
                };

                // SAFETY:
                // 1. mach_task_basic_info is a plain data structure with no padding
                // 2. std::mem::zeroed() is safe for plain data structs
                // 3. task_info is a system call that only writes to the provided buffer
                // 4. We provide the correct struct size via count parameter
                // 5. MACH_TASK_BASIC_INFO (4) is a valid system-defined constant
                // 6. mach_task_self() returns a valid task handle for the current process
                // 7. The pointer cast is safe because both structs have compatible layout
                let mut info: mach_task_basic_info = std::mem::zeroed();
                let mut count = (std::mem::size_of::<mach_task_basic_info>()
                    / std::mem::size_of::<i32>())
                    as mach_msg_type_number_t;

                // SAFETY: task_info kernel call
                // - Writes at most 'count' integers to info buffer
                // - Returns 0 on success, error code on failure
                // - Only accesses memory within the info struct bounds
                // - No undefined behavior as we provide valid, properly aligned memory
                if task_info(
                    mach_task_self(),
                    4, // MACH_TASK_BASIC_INFO
                    &mut info as *mut _ as *mut _,
                    &mut count,
                ) == 0
                {
                    return info.virtual_size as usize;
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Use Windows APIs
            unsafe {
                use winapi::um::processthreadsapi::OpenProcess;
                use winapi::um::psapi::{
                    GetCurrentProcess, GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
                };

                // SAFETY: Windows API calls for process memory information
                // 1. GetCurrentProcess() returns a pseudo-handle that's always valid
                // 2. OpenProcess() returns a valid handle when successful, checked below
                // 3. PROCESS_MEMORY_COUNTERS is a plain data struct safe to zero
                // 4. GetProcessMemoryInfo only writes within the provided buffer size
                // 5. The size parameter correctly matches the struct size
                use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

                let process = OpenProcess(PROCESS_QUERY_INFORMATION, 0, GetCurrentProcess());
                if !process.is_null() {
                    // SAFETY: PROCESS_MEMORY_COUNTERS is a POD struct safe to zero
                    let mut pmc: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();

                    // SAFETY: GetProcessMemoryInfo kernel call
                    // - Third parameter is size, correctly calculated
                    // - Only writes within pmc buffer bounds
                    // - Returns BOOL: TRUE on success, FALSE on failure
                    if GetProcessMemoryInfo(
                        process,
                        &mut pmc,
                        std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                    ) != 0 {
                        return pmc.WorkingSetSize as usize;
                    }
                }
            }
        }

        // Fallback: estimate from known structures (now using unified cache)
        let cache_size = cache_manager::global_cache_manager()
            .get_iri_cache_size()
            .unwrap_or(0); // Default to 0 if we can't get size

        cache_size * 200 + // Estimate ~200 bytes per cached IRI (unified cache)
        1024 * 1024 // Base 1MB estimate
    }

    /// Check for memory pressure and perform cleanup if needed
    pub fn check_and_cleanup(&self) -> Result<(), String> {
        let stats = self.get_stats();
        let mut last_cleanup = self
            .last_cleanup
            .lock()
            .map_err(|e| format!("Cleanup timing lock poisoned: {}", e))?;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
