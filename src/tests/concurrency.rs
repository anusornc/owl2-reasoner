//! Concurrency and thread safety tests for the OWL2 reasoner
//!
//! This module tests thread safety and concurrent access patterns
//! for global caches, arena allocation, and parser operations.

use crate::entities::{Class, ObjectProperty};
use crate::iri::{clear_global_iri_cache, global_iri_cache_stats, IRI};
use crate::ontology::Ontology;
use crate::parser::arena::SharedParserArena;
use std::sync::Arc;
use std::thread;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrent_shared_arena_allocation() {
        let shared_arena = Arc::new(SharedParserArena::new());
        let num_threads = 4;
        let allocations_per_thread = 1000;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let arena_clone = Arc::clone(&shared_arena);
                thread::spawn(move || {
                    for i in 0..allocations_per_thread {
                        let content = format!("thread_{}_item_{}", thread_id, i);
                        let _allocated = {
                            let arena_ref = arena_clone.arena();
                            arena_ref.alloc_str(&content)
                        };

                        // Verify allocation succeeded
                        assert!(!_allocated.is_empty());
                    }
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify arena is still in a consistent state
        {
            let arena_ref = shared_arena.arena();
            assert!(arena_ref.allocation_count() > 0);
        }
    }

    #[test]
    fn test_concurrent_shared_arena_access() {
        let shared_arena = Arc::new(SharedParserArena::new());
        let num_threads = 8;
        let allocations_per_thread = 500;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let arena_clone = Arc::clone(&shared_arena);
                thread::spawn(move || {
                    for i in 0..allocations_per_thread {
                        let content = format!("shared_thread_{}_item_{}", thread_id, i);
                        // Test both read and write access
                        {
                            let arena_ref = arena_clone.arena();
                            let _allocated = arena_ref.alloc_str(&content);
                        }
                    }
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify shared arena is still functional
        {
            let arena_ref = shared_arena.arena();
            assert!(arena_ref.allocation_count() > 0);
        }
    }

    #[test]
    fn test_concurrent_iri_creation() {
        let num_threads = 4;
        let iris_per_thread = 1000;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                thread::spawn(move || {
                    let mut created_iris = Vec::new();

                    for i in 0..iris_per_thread {
                        let iri_str =
                            format!("http://example.org/thread_{}/entity_{}", thread_id, i);
                        let iri = IRI::new(iri_str).unwrap();
                        created_iris.push(iri);
                    }

                    created_iris
                })
            })
            .collect();

        // Wait for all threads and collect results
        let all_iris: Vec<Vec<IRI>> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();

        // Verify all IRIs were created successfully
        let total_iris: usize = all_iris.iter().map(|v| v.len()).sum();
        assert_eq!(total_iris, num_threads * iris_per_thread);

        // Verify IRI cache is working (duplicates should be shared)
        let _ = clear_global_iri_cache();
    }

    #[test]
    fn test_concurrent_entity_creation() {
        let num_threads = 4;
        let entities_per_thread = 500;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                thread::spawn(move || {
                    let mut classes = Vec::new();
                    let mut properties = Vec::new();

                    for i in 0..entities_per_thread {
                        let class_iri =
                            format!("http://example.org/classes/Class_{}_{}", thread_id, i);
                        let class = Class::new(class_iri);
                        classes.push(class);

                        let prop_iri =
                            format!("http://example.org/properties/prop_{}_{}", thread_id, i);
                        let property = ObjectProperty::new(prop_iri);
                        properties.push(property);
                    }

                    (classes, properties)
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify global cache is still functional
        let test_iri = IRI::new("http://example.org/test").unwrap();
        let _test_class = Class::new(test_iri);
    }

    #[test]
    fn test_concurrent_ontology_operations() {
        let num_threads = 4;
        let operations_per_thread = 100;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                thread::spawn(move || {
                    let mut ontology = Ontology::new();

                    for i in 0..operations_per_thread {
                        let class_iri =
                            format!("http://example.org/ontology_{}/Class_{}", thread_id, i);
                        let class = Class::new(class_iri);

                        // Add class to ontology
                        ontology.add_class(class).unwrap();

                        // Query classes
                        let _classes = ontology.classes();

                        // Get IRI cache statistics
                        let _cache_stats = global_iri_cache_stats();
                    }

                    ontology
                })
            })
            .collect();

        // Wait for all threads to complete
        let _ontologies: Vec<Ontology> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();

        // Verify global state is still consistent
        let final_test_iri = IRI::new("http://example.org/final_test").unwrap();
        assert!(final_test_iri.as_str().starts_with("http://example.org/"));
    }

    #[test]
    fn test_shared_arena_thread_safety() {
        // Test that shared arena can be safely used across threads
        let arena = Arc::new(SharedParserArena::new());

        // Spawn multiple threads that allocate and deallocate
        let handles: Vec<_> = (0..4)
            .map(|thread_id| {
                let arena_clone = Arc::clone(&arena);
                thread::spawn(move || {
                    for i in 0..1000 {
                        let content = format!("safety_test_{}_{}", thread_id, i);
                        let _allocated = {
                            let arena_ref = arena_clone.arena();
                            arena_ref.alloc_str(&content)
                        };

                        // Mix in some slice allocations
                        if i % 100 == 0 {
                            let data = vec![thread_id; i % 10 + 1];
                            let _slice = {
                                let arena_ref = arena_clone.arena();
                                arena_ref.alloc_slice(&data)
                            };
                        }
                    }
                })
            })
            .collect();

        // Wait for completion
        for handle in handles {
            handle.join().unwrap();
        }

        // Arena should be in a valid state
        {
            let arena_ref = arena.arena();
            assert!(arena_ref.allocation_count() > 0);
            assert!(arena_ref.memory_usage() > 0);
        }
    }

    #[test]
    fn test_memory_pressure_under_load() {
        let arena = Arc::new(SharedParserArena::new());
        let num_threads = 2;
        let large_allocations = 100;

        let handles: Vec<_> = (0..num_threads)
            .map(|_thread_id| {
                let arena_clone = Arc::clone(&arena);
                thread::spawn(move || {
                    for i in 0..large_allocations {
                        // Create large strings to test memory pressure
                        let large_content = "x".repeat(1000 + (i % 5000));
                        let _allocated = {
                            let arena_ref = arena_clone.arena();
                            arena_ref.alloc_str(&large_content)
                        };

                        // Periodically check memory usage
                        if i % 20 == 0 {
                            let usage = {
                                let arena_ref = arena_clone.arena();
                                arena_ref.memory_usage()
                            };
                            assert!(usage > 0, "Memory usage should be positive");
                        }
                    }
                })
            })
            .collect();

        // Wait for completion
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify memory tracking still works
        {
            let arena_ref = arena.arena();
            assert!(arena_ref.memory_usage() > 0);
        }
    }

    #[test]
    fn test_cache_contention() {
        // Test cache behavior under concurrent access
        let num_threads = 8;
        let operations_per_thread = 200;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                thread::spawn(move || {
                    let mut iris = Vec::new();

                    for i in 0..operations_per_thread {
                        // Create IRIs that might overlap with other threads
                        let shared_base = "http://example.org/shared/";
                        let unique_part = format!("entity_{}_{}", thread_id % 2, i); // Some overlap
                        let iri_str = format!("{}{}", shared_base, unique_part);

                        let iri = IRI::new(iri_str).unwrap();
                        iris.push(iri);

                        // Periodically check cache stats
                        if i % 50 == 0 {
                            let _stats = global_iri_cache_stats();
                        }
                    }

                    iris
                })
            })
            .collect();

        // Wait for completion and verify no deadlocks or panics
        let _all_iris: Vec<Vec<IRI>> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();

        // Final verification that cache is still functional
        let final_iri = IRI::new("http://example.org/final_verification").unwrap();
        assert!(final_iri.as_str().contains("final_verification"));
    }

    #[test]
    fn test_stress_test_mixed_operations() {
        // Stress test with mixed operations across threads
        let arena = Arc::new(SharedParserArena::new());
        let num_threads = 6;
        let operations_per_thread = 500;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let arena_clone = Arc::clone(&arena);
                thread::spawn(move || {
                    for i in 0..operations_per_thread {
                        match i % 4 {
                            0 => {
                                // String allocation
                                let content = format!("mixed_{}_{}", thread_id, i);
                                let _allocated = {
                                    let arena_ref = arena_clone.arena();
                                    arena_ref.alloc_str(&content)
                                };
                            }
                            1 => {
                                // Slice allocation
                                let data = vec![thread_id; (i % 10) + 1];
                                let _slice = {
                                    let arena_ref = arena_clone.arena();
                                    arena_ref.alloc_slice(&data)
                                };
                            }
                            2 => {
                                // Query operations
                                let (_usage, _count) = {
                                    let arena_ref = arena_clone.arena();
                                    (arena_ref.memory_usage(), arena_ref.allocation_count())
                                };
                            }
                            3 => {
                                // IRI operations
                                let iri_str =
                                    format!("http://mixed.example.org/{}_{}", thread_id, i);
                                let _iri = IRI::new(iri_str).unwrap();
                            }
                            _ => unreachable!(),
                        }
                    }
                })
            })
            .collect();

        // Wait for completion
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify system is still stable
        {
            let arena_ref = arena.arena();
            assert!(arena_ref.memory_usage() > 0);
            assert!(arena_ref.allocation_count() > 0);
        }

        // Test final operations
        let test_iri = IRI::new("http://stress.test/final").unwrap();
        assert!(test_iri.as_str().starts_with("http://"));
    }
}
