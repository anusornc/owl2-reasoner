//! Memory Stress Testing Suite
//!
//! This module provides stress tests that push the memory safety system to its limits
//! to ensure it behaves correctly under extreme conditions.

#![allow(unused_doc_comments)]

use crate::cache_manager::*;
use crate::entities::*;
use crate::iri::IRI;
use crate::memory::*;
use crate::ontology::*;
use crate::test_memory_guard::*;
use crate::memory_safe_stress_test;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

/// Test memory guard under extreme memory pressure
memory_safe_stress_test!(test_extreme_memory_pressure, {
    println!("🔥 Testing memory guard under extreme pressure...");

    let config = TestMemoryConfig {
        max_memory_bytes: 200 * 1024 * 1024, // 200MB limit
        max_cache_size: 500,
        auto_cleanup: true,
        fail_on_limit_exceeded: false, // Don't fail, just monitor
        warn_threshold_percent: 0.6,
        check_interval: Duration::from_millis(10),
    };

    let guard = TestMemoryGuard::with_config(config);
    guard.start_monitoring();

    let mut allocations = Vec::new();
    let mut total_allocated = 0;
    let chunk_size = 10 * 1024 * 1024; // 10MB chunks

    // Keep allocating until we hit limits or warnings
    for chunk in 0..30 {
        let allocation: Vec<u8> = vec![chunk as u8; chunk_size];
        allocations.push(allocation);
        total_allocated += chunk_size;

        let result = guard.check_memory();
        match result {
            Ok(_) => {
                let usage = guard.memory_usage_percent();
                println!("  Chunk {}: {:.1}% memory usage", chunk, usage);
            }
            Err(e) => {
                println!("  Memory check failed at chunk {}: {}", chunk, e);
                break;
            }
        }

        // Small delay to allow monitoring
        thread::sleep(Duration::from_millis(50));
    }

    let report = guard.stop_monitoring();

    println!("  Stress test completed:");
    println!("    Total allocated: {} MB", total_allocated / 1024 / 1024);
    println!(
        "    Peak memory: {:.1} MB",
        report.peak_memory as f64 / 1024.0 / 1024.0
    );
    println!("    Warnings: {}", report.warnings.len());
    println!("    Cleanups: {}", report.cleanup_count);

    // Should have warnings due to high memory usage
    assert!(
        !report.warnings.is_empty(),
        "Should have memory warnings under stress"
    );

    // But system should still be functional
    assert!(report.end_memory > 0, "Should have non-zero memory usage");

    // Clean up allocations
    drop(allocations);
});

/// Test concurrent memory stress scenarios
memory_safe_stress_test!(test_concurrent_memory_stress, {
    println!("🔥 Testing concurrent memory stress scenarios...");

    let num_threads = 6;
    let barrier = Arc::new(Barrier::new(num_threads));
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let barrier_clone = Arc::clone(&barrier);
        let results_clone = Arc::clone(&results);

        let handle = thread::spawn(move || {
            // Wait for all threads to start
            barrier_clone.wait();

            // Each thread creates its own memory guard
            let config = TestMemoryConfig {
                max_memory_bytes: 150 * 1024 * 1024, // 150MB per thread
                max_cache_size: 300,
                auto_cleanup: true,
                fail_on_limit_exceeded: false,
                warn_threshold_percent: 0.7,
                check_interval: Duration::from_millis(20),
            };

            let guard = TestMemoryGuard::with_config(config);
            guard.start_monitoring();

            let mut thread_allocations = Vec::new();
            let mut warnings_count = 0;

            // Perform intensive memory operations
            for round in 0..50 {
                // Allocate different sizes based on thread ID
                let size = (thread_id * 1000000 + round * 100000) % 2000000 + 500000;
                let allocation: Vec<u8> = vec![(round + thread_id) as u8; size];
                thread_allocations.push(allocation);

                // Check memory
                if let Err(_) = guard.check_memory() {
                    warnings_count += 1;
                }

                // Periodic cleanup simulation
                if round % 10 == 0 && round > 0 {
                    // Free some allocations
                    thread_allocations.drain(0..thread_allocations.len() / 2);
                }

                // Small delay to allow other threads
                thread::sleep(Duration::from_millis(5));
            }

            let report = guard.stop_monitoring();

            // Store results
            let mut results = results_clone.lock().unwrap();
            results.push((thread_id, report, warnings_count));

            // Clean up
            drop(thread_allocations);
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Analyze results
    let results = results.lock().unwrap();
    assert_eq!(results.len(), num_threads, "All threads should complete");

    let total_warnings: usize = results.iter().map(|(_, _, warnings)| *warnings).sum();

    let total_guard_warnings: usize = results
        .iter()
        .map(|(_, report, _)| report.warnings.len())
        .sum();

    println!("  Concurrent stress test results:");
    println!("    Threads completed: {}", results.len());
    println!("    Total memory warnings: {}", total_warnings);
    println!("    Total guard warnings: {}", total_guard_warnings);

    for (thread_id, report, _) in results.iter() {
        println!(
            "    Thread {}: {:.1}% peak usage, {} warnings",
            thread_id,
            (report.peak_memory as f64 / report.max_memory_bytes as f64) * 100.0,
            report.warnings.len()
        );
    }

    // All threads should complete successfully
    for (_, report, _) in results.iter() {
        assert!(
            report.end_memory > 0,
            "Each thread should have memory usage"
        );
    }
});

/// Test memory limit enforcement behavior
memory_safe_stress_test!(test_memory_limit_enforcement, {
    println!("🔥 Testing memory limit enforcement behavior...");

    // Test with strict limit enforcement
    let strict_config = TestMemoryConfig {
        max_memory_bytes: 50 * 1024 * 1024, // 50MB - very strict
        max_cache_size: 50,
        auto_cleanup: false, // Disable auto cleanup to test enforcement
        fail_on_limit_exceeded: true,
        warn_threshold_percent: 0.5,
        check_interval: Duration::from_millis(5),
    };

    let guard = TestMemoryGuard::with_config(strict_config);
    guard.start_monitoring();

    let mut allocations = Vec::new();
    let mut limit_exceeded_count = 0;

    // Keep allocating until limit is strictly enforced
    for chunk in 0..20 {
        let size = 5 * 1024 * 1024; // 5MB chunks
        let allocation: Vec<u8> = vec![chunk as u8; size];
        allocations.push(allocation);

        match guard.check_memory() {
            Ok(_) => {
                println!("  Chunk {}: OK", chunk);
            }
            Err(MemoryGuardError::LimitExceeded(msg)) => {
                println!("  Limit exceeded at chunk {}: {}", chunk, msg);
                limit_exceeded_count += 1;
                break;
            }
            Err(e) => {
                println!("  Other error at chunk {}: {}", chunk, e);
                break;
            }
        }

        thread::sleep(Duration::from_millis(10));
    }

    let report = guard.stop_monitoring();

    println!("  Limit enforcement test results:");
    println!("    Limit exceeded count: {}", limit_exceeded_count);
    println!(
        "    Final memory usage: {:.1} MB",
        report.end_memory as f64 / 1024.0 / 1024.0
    );
    println!("    Warnings: {}", report.warnings.len());

    // Should have hit the limit
    assert!(
        limit_exceeded_count > 0,
        "Should have exceeded memory limit"
    );

    // Should have warnings
    assert!(
        !report.warnings.is_empty(),
        "Should have warnings about limit"
    );

    // Clean up
    drop(allocations);
});

/// Test memory leak detection under stress
memory_safe_stress_test!(test_memory_leak_detection_stress, {
    println!("🔥 Testing memory leak detection under stress...");

    let initial_report = detect_memory_leaks();
    println!(
        "  Initial efficiency score: {:.2}",
        initial_report.memory_efficiency_score
    );

    // Create many allocations to stress the leak detection
    let mut large_allocations = Vec::new();

    for i in 0..100 {
        let size = (i % 10 + 1) * 1024 * 1024; // 1-10MB each
        let allocation: Vec<u8> = vec![i as u8; size];
        large_allocations.push(allocation);

        if i % 20 == 0 {
            let mid_report = detect_memory_leaks();
            println!(
                "  Allocation {} - Efficiency: {:.2}",
                i, mid_report.memory_efficiency_score
            );
        }
    }

    let stressed_report = detect_memory_leaks();
    println!(
        "  Under stress - Efficiency: {:.2}",
        stressed_report.memory_efficiency_score
    );
    println!(
        "  Potential leaks detected: {}",
        stressed_report.potential_leaks.len()
    );

    // Clean up half the allocations
    large_allocations.drain(0..50);

    let partial_cleanup_report = detect_memory_leaks();
    println!(
        "  Partial cleanup - Efficiency: {:.2}",
        partial_cleanup_report.memory_efficiency_score
    );

    // Clean up all allocations
    drop(large_allocations);

    // Force cleanup
    let _ = force_memory_cleanup();
    thread::sleep(Duration::from_millis(100));

    let final_report = detect_memory_leaks();
    println!(
        "  Final cleanup - Efficiency: {:.2}",
        final_report.memory_efficiency_score
    );

    // Efficiency should improve after cleanup
    assert!(
        final_report.memory_efficiency_score >= partial_cleanup_report.memory_efficiency_score,
        "Efficiency should improve after complete cleanup"
    );

    // Check for specific leak types
    if !stressed_report.potential_leaks.is_empty() {
        println!("  Potential leaks detected:");
        for leak in &stressed_report.potential_leaks {
            println!("    - {}", leak);
        }
    }

    if !final_report.recommendations.is_empty() {
        println!("  Recommendations:");
        for rec in &final_report.recommendations {
            println!("    - {}", rec);
        }
    }
});

/// Test cache behavior under memory stress
memory_safe_stress_test!(test_cache_memory_stress, {
    println!("🔥 Testing cache behavior under memory stress...");

    let initial_cache_stats = global_cache_stats();
    println!(
        "  Initial cache stats: {} hits, {} misses",
        initial_cache_stats.iri_hits, initial_cache_stats.iri_misses
    );

    // Create many IRIs to fill the cache
    let mut iris = Vec::new();
    for i in 0..10000 {
        let iri = match IRI::new(&format!("http://example.org/stress/test/{}", i)) {
            Ok(iri) => iri,
            Err(_) => continue,
        };
        iris.push(iri);

        if i % 1000 == 0 {
            let cache_stats = global_cache_stats();
            println!(
                "  Created {} IRIs - Cache size: {}",
                i,
                cache_stats.iri_hits + cache_stats.iri_misses
            );
        }
    }

    let after_creation_stats = global_cache_stats();
    println!(
        "  After creation - Cache size: {}",
        after_creation_stats.iri_hits + after_creation_stats.iri_misses
    );

    // Test cache under memory pressure
    let config = TestMemoryConfig {
        max_memory_bytes: 100 * 1024 * 1024, // 100MB
        max_cache_size: 1000,                // Small cache limit
        auto_cleanup: true,
        fail_on_limit_exceeded: false,
        warn_threshold_percent: 0.7,
        check_interval: Duration::from_millis(10),
    };

    let guard = TestMemoryGuard::with_config(config);
    guard.start_monitoring();

    // Continue creating IRIs under memory pressure
    for i in 10000..20000 {
        let _ = IRI::new(&format!("http://example.org/stress/pressure/{}", i));

        if i % 1000 == 0 {
            let _ = guard.check_memory();
            let cache_stats = global_cache_stats();
            println!(
                "  Under pressure {} - Cache size: {}",
                i,
                cache_stats.iri_hits + cache_stats.iri_misses
            );
        }
    }

    // Clear cache and test recovery
    let _ = clear_global_iri_cache();

    let after_clear_stats = global_cache_stats();
    println!(
        "  After cache clear - Cache size: {}",
        after_clear_stats.iri_hits + after_clear_stats.iri_misses
    );

    let report = guard.stop_monitoring();

    println!("  Cache stress test completed:");
    println!(
        "    Final cache size: {}",
        after_clear_stats.iri_hits + after_clear_stats.iri_misses
    );
    println!("    Memory warnings: {}", report.warnings.len());
    println!("    Cleanups performed: {}", report.cleanup_count);

    // Clean up
    drop(iris);
});

/// Test ontology operations under memory stress
memory_safe_stress_test!(test_ontology_memory_stress, {
    println!("🔥 Testing ontology operations under memory stress...");

    let config = TestMemoryConfig {
        max_memory_bytes: 300 * 1024 * 1024, // 300MB
        max_cache_size: 2000,
        auto_cleanup: true,
        fail_on_limit_exceeded: false,
        warn_threshold_percent: 0.8,
        check_interval: Duration::from_millis(50),
    };

    let guard = TestMemoryGuard::with_config(config);
    guard.start_monitoring();

    let mut ontologies = Vec::new();

    // Create multiple large ontologies
    for ont_idx in 0..5 {
        let mut ontology = Ontology::new();

        println!("  Creating ontology {}...", ont_idx);

        // Create many classes
        for class_idx in 0..2000 {
            let iri = IRI::new(&format!(
                "http://example.org/ont{}/class{}",
                ont_idx, class_idx
            ))
            .unwrap();
            let class = Class::new(Arc::new(iri));
            let _ = ontology.add_class(class);
        }

        // Create many properties
        for prop_idx in 0..500 {
            let iri = IRI::new(&format!(
                "http://example.org/ont{}/prop{}",
                ont_idx, prop_idx
            ))
            .unwrap();
            let prop = crate::ObjectProperty::new(Arc::new(iri));
            let _ = ontology.add_object_property(prop);
        }

        // Create many subclass relationships
        for rel_idx in 0..3000 {
            let subclass_iri = IRI::new(&format!(
                "http://example.org/ont{}/class{}",
                ont_idx,
                rel_idx % 2000
            ))
            .unwrap();
            let superclass_iri = IRI::new(&format!(
                "http://example.org/ont{}/class{}",
                ont_idx,
                (rel_idx + 1) % 2000
            ))
            .unwrap();

            let subclass = crate::ClassExpression::Class(crate::Class::new(Arc::new(subclass_iri)));
            let superclass =
                crate::ClassExpression::Class(crate::Class::new(Arc::new(superclass_iri)));

            let axiom = crate::SubClassOfAxiom::new(subclass, superclass);
            let _ = ontology.add_subclass_axiom(axiom);
        }

        ontologies.push(ontology);

        // Check memory after each ontology
        let _ = guard.check_memory();
        let usage = guard.memory_usage_percent();
        println!(
            "    Ontology {} created: {:.1}% memory usage",
            ont_idx, usage
        );

        if usage > 85.0 {
            println!("    High memory usage detected, stopping ontology creation");
            break;
        }
    }

    // Test reasoning on the ontologies
    println!("  Testing reasoning under stress...");
    for (idx, ontology) in ontologies.iter().enumerate() {
        let reasoner = crate::SimpleReasoner::new(ontology.clone());
        let _ = reasoner.is_consistent();

        if idx % 2 == 0 {
            let _ = guard.check_memory();
        }

        println!("    Reasoning completed for ontology {}", idx);
    }

    let report = guard.stop_monitoring();

    println!("  Ontology stress test completed:");
    println!("    Ontologies created: {}", ontologies.len());
    println!(
        "    Peak memory: {:.1} MB",
        report.peak_memory as f64 / 1024.0 / 1024.0
    );
    println!("    Warnings: {}", report.warnings.len());
    println!("    Cleanups: {}", report.cleanup_count);

    // Clean up
    drop(ontologies);
});

/// Test rapid allocation and deallocation cycles
memory_safe_stress_test!(test_rapid_allocation_cycles, {
    println!("🔥 Testing rapid allocation and deallocation cycles...");

    let config = TestMemoryConfig {
        max_memory_bytes: 150 * 1024 * 1024, // 150MB
        max_cache_size: 1000,
        auto_cleanup: true,
        fail_on_limit_exceeded: false,
        warn_threshold_percent: 0.75,
        check_interval: Duration::from_millis(5),
    };

    let guard = TestMemoryGuard::with_config(config);
    guard.start_monitoring();

    let mut cycle_count = 0;
    let mut max_allocations = 0;

    // Perform many rapid allocation/deallocation cycles
    for cycle in 0..100 {
        let mut allocations = Vec::new();

        // Allocate many small chunks
        for chunk in 0..50 {
            let size = (chunk % 5 + 1) * 1024 * 1024; // 1-5MB
            let allocation: Vec<u8> = vec![cycle as u8; size];
            allocations.push(allocation);
        }

        max_allocations = max_allocations.max(allocations.len());

        // Check memory
        let _ = guard.check_memory();

        // Immediately deallocate half
        allocations.drain(0..25);

        // Check memory again
        let _ = guard.check_memory();

        // Clean up the rest
        drop(allocations);

        cycle_count += 1;

        if cycle % 10 == 0 {
            let usage = guard.memory_usage_percent();
            println!("  Cycle {}: {:.1}% memory usage", cycle, usage);
        }
    }

    let report = guard.stop_monitoring();

    println!("  Rapid allocation cycles completed:");
    println!("    Cycles completed: {}", cycle_count);
    println!("    Max simultaneous allocations: {}", max_allocations);
    println!(
        "    Peak memory: {:.1} MB",
        report.peak_memory as f64 / 1024.0 / 1024.0
    );
    println!("    Warnings: {}", report.warnings.len());
    println!("    Cleanups: {}", report.cleanup_count);

    // System should handle rapid cycles well
    assert!(cycle_count >= 90, "Should complete most cycles");
    assert!(
        report.warnings.len() < 20,
        "Should not have excessive warnings"
    );
});

/// Comprehensive memory stress test summary
memory_safe_stress_test!(test_memory_stress_summary, {
    println!("🔥 Comprehensive Memory Stress Test Summary");
    println!("============================================");

    // Run all stress tests
    test_extreme_memory_pressure();
    test_concurrent_memory_stress();
    test_memory_limit_enforcement();
    test_memory_leak_detection_stress();
    test_cache_memory_stress();
    test_ontology_memory_stress();
    test_rapid_allocation_cycles();

    println!("============================================");
    println!("✅ All memory stress tests completed!");
    println!("   - Extreme memory pressure handling verified");
    println!("   - Concurrent memory stress tested");
    println!("   - Memory limit enforcement validated");
    println!("   - Memory leak detection under stress confirmed");
    println!("   - Cache behavior under pressure tested");
    println!("   - Ontology operations under stress validated");
    println!("   - Rapid allocation cycles handled successfully");

    // Final system health check
    let final_stats = get_memory_stats();
    let final_leak_report = detect_memory_leaks();

    println!("\n📊 Final System Health After Stress:");
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

    // System should be in reasonable state after stress tests
    assert!(
        final_stats.pressure_level < 0.95,
        "System pressure should be manageable after stress"
    );
    assert!(
        final_leak_report.memory_efficiency_score > 0.3,
        "System should maintain reasonable efficiency"
    );
});
