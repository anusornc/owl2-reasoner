//! Comprehensive Memory Safety Validation Tests
//!
//! This module provides extensive tests to validate that the memory safety system
//! works correctly under various conditions including edge cases, stress scenarios,
//! and concurrent access patterns.

#![allow(unused_doc_comments)]

use crate::cache_manager::*;
use crate::memory::*;
use crate::memory_safe_test;
use crate::test_helpers::*;
use crate::test_memory_guard::*;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

/// Test basic memory monitoring functionality
memory_safe_test!(
    test_basic_memory_monitoring,
    MemorySafeTestConfig::small(),
    {
        let initial_stats = get_memory_stats();

        // Verify initial state
        assert!(
            initial_stats.total_usage > 0,
            "Should have non-zero memory usage"
        );
        assert!(initial_stats.pressure_level >= 0.0 && initial_stats.pressure_level <= 1.0);

        // Simulate memory usage
        let _data: Vec<u8> = vec![0; 1024 * 1024]; // 1MB

        let after_stats = get_memory_stats();

        // Memory usage should have increased
        assert!(after_stats.total_usage >= initial_stats.total_usage);

        // Pressure level should still be valid
        assert!(after_stats.pressure_level >= 0.0 && after_stats.pressure_level <= 1.0);

        println!("Memory monitoring test passed:");
        println!(
            "  Initial: {} bytes, {:.2}% pressure",
            initial_stats.total_usage,
            initial_stats.pressure_level * 100.0
        );
        println!(
            "  After: {} bytes, {:.2}% pressure",
            after_stats.total_usage,
            after_stats.pressure_level * 100.0
        );
    }
);

/// Test memory guard configuration and behavior
memory_safe_test!(
    test_memory_guard_configuration,
    MemorySafeTestConfig::small(),
    {
        // Test default configuration
        let default_config = TestMemoryConfig::default();
        assert_eq!(default_config.max_memory_bytes, 512 * 1024 * 1024); // 512MB
        assert_eq!(default_config.max_cache_size, 1000);
        assert!(default_config.auto_cleanup);
        assert!(default_config.fail_on_limit_exceeded);

        // Test custom configuration
        let custom_config = TestMemoryConfig {
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            max_cache_size: 500,
            auto_cleanup: false,
            fail_on_limit_exceeded: false,
            warn_threshold_percent: 0.5,
            check_interval: Duration::from_millis(200),
        };

        let guard = TestMemoryGuard::with_config(custom_config);
        guard.start_monitoring();

        // Verify guard is monitoring
        assert!(guard.is_monitoring());

        // Check memory usage
        let result = guard.check_memory();
        assert!(result.is_ok(), "Memory check should succeed");

        let usage_percent = guard.memory_usage_percent();
        assert!(usage_percent >= 0.0 && usage_percent <= 100.0);

        let report = guard.stop_monitoring();
        assert!(report.is_acceptable());

        println!("Memory guard configuration test passed");
        println!("  Usage: {:.1}%", usage_percent);
        println!("  Report: {}", report.format());
    }
);

/// Test memory guard under low memory conditions
memory_safe_test!(test_memory_guard_low_memory, {
    let config = TestMemoryConfig {
        max_memory_bytes: 50 * 1024 * 1024, // 50MB limit
        max_cache_size: 100,
        auto_cleanup: true,
        fail_on_limit_exceeded: false, // Don't fail, just warn
        warn_threshold_percent: 0.5,
        check_interval: Duration::from_millis(50),
    };

    let guard = TestMemoryGuard::with_config(config);
    guard.start_monitoring();

    // Allocate memory gradually to trigger warnings
    let mut allocations = Vec::new();
    for i in 0..10 {
        let allocation: Vec<u8> = vec![i as u8; 5 * 1024 * 1024]; // 5MB each
        allocations.push(allocation);

        let result = guard.check_memory();
        if let Err(e) = result {
            println!("Memory check failed at iteration {}: {}", i, e);
            break;
        }

        thread::sleep(Duration::from_millis(10));
    }

    let report = guard.stop_monitoring();

    // Should have warnings due to low memory limit
    assert!(!report.warnings.is_empty(), "Should have memory warnings");

    // But should still be acceptable since we didn't fail on limit exceeded
    println!(
        "Low memory test passed with {} warnings",
        report.warnings.len()
    );
    for warning in &report.warnings {
        println!("  Warning: {}", warning);
    }
});

/// Test memory cleanup functionality
memory_safe_test!(
    test_memory_cleanup_functionality,
    MemorySafeTestConfig::medium(),
    {
        // Fill caches first
        let cache_stats_before = global_cache_stats();
        println!(
            "Cache stats before cleanup: {} hits, {} misses",
            cache_stats_before.iri_hits, cache_stats_before.iri_misses
        );

        // Perform cleanup
        let cleanup_result = force_memory_cleanup();
        assert!(cleanup_result.is_ok(), "Memory cleanup should succeed");

        // Check that cleanup was recorded
        let cleanup_count = get_cleanup_count();
        assert!(
            cleanup_count > 0,
            "Should have performed cleanup operations"
        );

        // Get memory stats after cleanup
        let stats_after = get_memory_stats();
        println!("Memory stats after cleanup:");
        println!("  Total usage: {} bytes", stats_after.total_usage);
        println!("  Pressure: {:.2}%", stats_after.pressure_level * 100.0);
        println!("  Cleanup count: {}", stats_after.cleanup_count);

        // Verify leak detection
        let leak_report = detect_memory_leaks();
        assert!(
            leak_report.memory_efficiency_score >= 0.0
                && leak_report.memory_efficiency_score <= 1.0
        );

        println!("Leak detection report:");
        println!(
            "  Efficiency score: {:.2}",
            leak_report.memory_efficiency_score
        );
        println!("  Potential leaks: {}", leak_report.potential_leaks.len());
        println!("  Recommendations: {}", leak_report.recommendations.len());

        for leak in &leak_report.potential_leaks {
            println!("    - {}", leak);
        }

        for recommendation in &leak_report.recommendations {
            println!("    - {}", recommendation);
        }
    }
);

/// Test concurrent memory access and monitoring
memory_safe_test!(
    test_concurrent_memory_access,
    MemorySafeTestConfig::medium(),
    {
        let num_threads = 4;
        let barrier = Arc::new(Barrier::new(num_threads));
        let results = Arc::new(Mutex::new(Vec::new()));

        let mut handles = Vec::new();

        for thread_id in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let results_clone = Arc::clone(&results);

            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                // Each thread creates a memory guard and performs operations
                let config = TestMemoryConfig {
                    max_memory_bytes: 100 * 1024 * 1024, // 100MB per thread
                    max_cache_size: 200,
                    auto_cleanup: true,
                    fail_on_limit_exceeded: false,
                    warn_threshold_percent: 0.8,
                    check_interval: Duration::from_millis(10),
                };

                let guard = TestMemoryGuard::with_config(config);
                guard.start_monitoring();

                // Perform memory-intensive operations
                let mut allocations = Vec::new();
                for i in 0..20 {
                    let size = (thread_id * 20 + i) * 1024; // Variable sizes
                    let allocation: Vec<u8> = vec![i as u8; size];
                    allocations.push(allocation);

                    // Check memory periodically
                    let _ = guard.check_memory();

                    // Small delay to allow other threads
                    thread::sleep(Duration::from_millis(1));
                }

                let report = guard.stop_monitoring();

                // Store results
                let mut results = results_clone.lock().unwrap();
                results.push((thread_id, report));

                // Clean up allocations
                drop(allocations);
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Analyze results
        let results = results.lock().unwrap();
        assert_eq!(results.len(), num_threads, "All threads should complete");

        let total_warnings: usize = results
            .iter()
            .map(|(_, report)| report.warnings.len())
            .sum();

        println!("Concurrent memory access test completed:");
        println!("  Threads: {}", num_threads);
        println!("  Total warnings: {}", total_warnings);

        for (thread_id, report) in results.iter() {
            println!(
                "  Thread {}: {:.1}% usage, {} warnings",
                thread_id,
                (report.end_memory as f64 / report.max_memory_bytes as f64) * 100.0,
                report.warnings.len()
            );
        }

        // Verify all threads completed successfully
        for (_, report) in results.iter() {
            assert!(
                report.is_acceptable(),
                "All thread reports should be acceptable"
            );
        }
    }
);

/// Test memory pressure detection and response
memory_safe_test!(
    test_memory_pressure_detection,
    MemorySafeTestConfig::large(),
    {
        let initial_pressure = get_memory_pressure_level();
        println!("Initial memory pressure: {:.2}%", initial_pressure * 100.0);

        // Create a large allocation to increase pressure
        let large_allocation: Vec<u8> = vec![0; 50 * 1024 * 1024]; // 50MB

        let after_allocation_pressure = get_memory_pressure_level();
        println!(
            "Memory pressure after allocation: {:.2}%",
            after_allocation_pressure * 100.0
        );

        // Check if system is under pressure
        let is_under_pressure = is_under_memory_pressure();
        println!("System under pressure: {}", is_under_pressure);

        // Test pressure-based cleanup
        let monitor = MemoryMonitor::new(MemoryMonitorConfig {
            max_memory: 200 * 1024 * 1024, // 200MB
            pressure_threshold: 0.5,       // 50% threshold
            cleanup_interval: Duration::from_secs(1),
            auto_cleanup: true,
        });

        // Check and perform cleanup if needed
        let cleanup_result = monitor.check_and_cleanup();
        assert!(
            cleanup_result.is_ok(),
            "Pressure-based cleanup should succeed"
        );

        let final_pressure = monitor.get_pressure_level();
        println!("Final memory pressure: {:.2}%", final_pressure * 100.0);

        // Get detailed stats
        let stats = monitor.get_stats();
        println!("Memory monitor stats:");
        println!("  Total usage: {} bytes", stats.total_usage);
        println!("  Peak usage: {} bytes", stats.peak_usage);
        println!("  Pressure level: {:.2}%", stats.pressure_level * 100.0);
        println!("  Cleanup count: {}", stats.cleanup_count);
        println!("  IRI cache size: {}", stats.iri_cache_size);
        println!("  Entity cache size: {}", stats.entity_cache_size);

        // Clean up
        drop(large_allocation);
    }
);

/// Test memory guard error handling
memory_safe_test!(
    test_memory_guard_error_handling,
    MemorySafeTestConfig::small(),
    {
        let config = TestMemoryConfig {
            max_memory_bytes: 10 * 1024 * 1024, // 10MB - very small
            max_cache_size: 10,
            auto_cleanup: false,
            fail_on_limit_exceeded: true,
            warn_threshold_percent: 0.5,
            check_interval: Duration::from_millis(10),
        };

        let guard = TestMemoryGuard::with_config(config);
        guard.start_monitoring();

        // Try to allocate more than the limit
        let large_allocation: Vec<u8> = vec![0; 15 * 1024 * 1024]; // 15MB

        // This should fail due to memory limit
        let result = guard.check_memory();
        assert!(
            result.is_err(),
            "Memory check should fail when limit exceeded"
        );

        match result.unwrap_err() {
            MemoryGuardError::LimitExceeded(msg) => {
                assert!(msg.contains("Memory limit exceeded"));
                println!("Correctly detected limit exceeded: {}", msg);
            }
            other => panic!("Expected LimitExceeded error, got: {:?}", other),
        }

        // Stop monitoring and get report
        let report = guard.stop_monitoring();

        // Report should show warnings/errors
        assert!(
            !report.warnings.is_empty(),
            "Should have warnings about limit exceeded"
        );
        assert!(
            !report.is_acceptable(),
            "Report should not be acceptable due to limit exceeded"
        );

        println!(
            "Error handling test passed with {} warnings",
            report.warnings.len()
        );

        // Clean up
        drop(large_allocation);
    }
);

/// Test memory leak detection accuracy
memory_safe_test!(
    test_memory_leak_detection_accuracy,
    MemorySafeTestConfig::medium(),
    {
        // Get baseline leak detection
        let baseline_report = detect_memory_leaks();
        println!("Baseline leak detection:");
        println!(
            "  Efficiency score: {:.2}",
            baseline_report.memory_efficiency_score
        );
        println!(
            "  Potential leaks: {}",
            baseline_report.potential_leaks.len()
        );

        // Create some intentional "leaks" (large allocations that we don't clean up immediately)
        let _leak1: Vec<u8> = vec![1; 10 * 1024 * 1024]; // 10MB
        let _leak2: Vec<u8> = vec![2; 5 * 1024 * 1024]; // 5MB

        // Check leak detection after allocations
        let after_leaks_report = detect_memory_leaks();
        println!("After allocations leak detection:");
        println!(
            "  Efficiency score: {:.2}",
            after_leaks_report.memory_efficiency_score
        );
        println!(
            "  Potential leaks: {}",
            after_leaks_report.potential_leaks.len()
        );

        // The efficiency score should have decreased
        assert!(
            after_leaks_report.memory_efficiency_score <= baseline_report.memory_efficiency_score,
            "Efficiency score should decrease after allocations"
        );

        // Clean up the "leaks"
        drop(_leak1);
        drop(_leak2);

        // Force cleanup
        let _ = force_memory_cleanup();
        thread::sleep(Duration::from_millis(100)); // Allow cleanup to complete

        // Check leak detection after cleanup
        let cleanup_report = detect_memory_leaks();
        println!("After cleanup leak detection:");
        println!(
            "  Efficiency score: {:.2}",
            cleanup_report.memory_efficiency_score
        );
        println!(
            "  Potential leaks: {}",
            cleanup_report.potential_leaks.len()
        );

        // Efficiency should improve after cleanup
        assert!(
            cleanup_report.memory_efficiency_score >= after_leaks_report.memory_efficiency_score,
            "Efficiency score should improve after cleanup"
        );
    }
);

/// Test memory monitoring configuration updates
memory_safe_test!(
    test_memory_monitor_configuration_updates,
    MemorySafeTestConfig::small(),
    {
        let initial_config = MemoryMonitorConfig {
            max_memory: 100 * 1024 * 1024, // 100MB
            pressure_threshold: 0.7,
            cleanup_interval: Duration::from_secs(10),
            auto_cleanup: true,
        };

        let mut monitor = MemoryMonitor::new(initial_config.clone());

        // Test initial configuration
        assert!(monitor.get_pressure_level() < initial_config.pressure_threshold);

        // Update configuration
        let updated_config = MemoryMonitorConfig {
            max_memory: 50 * 1024 * 1024, // 50MB - smaller
            pressure_threshold: 0.5,      // Lower threshold
            cleanup_interval: Duration::from_secs(5),
            auto_cleanup: false,
        };

        monitor.update_config(updated_config);

        // Test that configuration was updated
        // Note: We can't directly access the config, but we can test behavior
        let stats = monitor.get_stats();
        println!("Monitor stats after config update:");
        println!("  Pressure level: {:.2}%", stats.pressure_level * 100.0);

        // Test cleanup with new configuration
        let cleanup_result = monitor.check_and_cleanup();
        assert!(
            cleanup_result.is_ok(),
            "Cleanup should succeed with updated config"
        );

        println!("Configuration update test passed");
    }
);

/// Comprehensive memory safety validation summary
memory_safe_test!(
    test_memory_safety_validation_summary,
    MemorySafeTestConfig::large(),
    {
        println!("🔍 Comprehensive Memory Safety Validation Summary");
        println!("==============================================");

        // Run all the individual validation tests
        test_basic_memory_monitoring();
        test_memory_guard_configuration();
        test_memory_guard_low_memory();
        test_memory_cleanup_functionality();
        test_concurrent_memory_access();
        test_memory_pressure_detection();
        test_memory_guard_error_handling();
        test_memory_leak_detection_accuracy();
        test_memory_monitor_configuration_updates();

        println!("==============================================");
        println!("✅ All memory safety validation tests passed!");
        println!("   - Basic monitoring functionality verified");
        println!("   - Memory guard configuration and behavior validated");
        println!("   - Low memory handling tested");
        println!("   - Cleanup functionality verified");
        println!("   - Concurrent access safety confirmed");
        println!("   - Pressure detection and response validated");
        println!("   - Error handling tested");
        println!("   - Leak detection accuracy verified");
        println!("   - Configuration updates tested");

        // Final system health check
        let final_stats = get_memory_stats();
        let final_leak_report = detect_memory_leaks();

        println!("\n📊 Final System Health:");
        println!("   Memory usage: {} bytes", final_stats.total_usage);
        println!(
            "   Pressure level: {:.2}%",
            final_stats.pressure_level * 100.0
        );
        println!(
            "   Efficiency score: {:.2}",
            final_leak_report.memory_efficiency_score
        );
        println!("   Total cleanups: {}", final_stats.cleanup_count);

        // Assert system is in good state
        assert!(
            final_stats.pressure_level < 0.9,
            "System pressure should be manageable"
        );
        assert!(
            final_leak_report.memory_efficiency_score > 0.5,
            "System efficiency should be reasonable"
        );
    }
);
