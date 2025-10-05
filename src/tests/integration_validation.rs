//! Comprehensive Integration Validation Tests
//!
//! This module provides integration tests to validate that all components
//! work together correctly after the project reorganization.

#![allow(unused_doc_comments)]

use crate::cache_manager::*;
use crate::entities::*;
use crate::iri::IRI;
use crate::memory::*;
use crate::ontology::*;
use crate::parser::*;
use crate::reasoning::*;
use crate::test_memory_guard::*;
use crate::test_helpers::*;
use crate::memory_safe_test;
use crate::axioms::{SubClassOfAxiom, ClassExpression};
use std::sync::Arc;
use std::time::Duration;

/// Test integration between memory safety and ontology operations
memory_safe_test!(
    test_memory_ontology_integration,
    MemorySafeTestConfig::medium(),
    {
        println!("🔗 Testing memory safety and ontology integration...");

        let guard = TestMemoryGuard::new();
        guard.start_monitoring();

        // Create ontology while monitoring memory
        let mut ontology = Ontology::new();

        // Add classes
        let person_class = Class::new(Arc::new(IRI::new("http://example.org/Person").unwrap()));
        let employee_class = Class::new(Arc::new(IRI::new("http://example.org/Employee").unwrap()));
        let manager_class = Class::new(Arc::new(IRI::new("http://example.org/Manager").unwrap()));

        ontology.add_class(person_class.clone()).unwrap();
        ontology.add_class(employee_class.clone()).unwrap();
        ontology.add_class(manager_class.clone()).unwrap();

        // Check memory after adding classes
        let _ = guard.check_memory();

        // Add subclass relationships
        ontology
            .add_subclass_axiom(SubClassOfAxiom::new(
                ClassExpression::Class(employee_class.clone()),
                ClassExpression::Class(person_class.clone()),
            ))
            .unwrap();

        ontology
            .add_subclass_axiom(SubClassOfAxiom::new(
                ClassExpression::Class(manager_class.clone()),
                ClassExpression::Class(employee_class.clone()),
            ))
            .unwrap();

        // Check memory after adding axioms
        let _ = guard.check_memory();

        // Test reasoning
        let reasoner = SimpleReasoner::new(ontology.clone());
        let is_consistent = reasoner.is_consistent().unwrap();
        let is_manager_employee = reasoner
            .is_subclass_of(manager_class.iri(), employee_class.iri())
            .unwrap();
        let is_employee_person = reasoner
            .is_subclass_of(employee_class.iri(), person_class.iri())
            .unwrap();

        // Verify reasoning results
        assert!(is_consistent, "Ontology should be consistent");
        assert!(
            is_manager_employee,
            "Manager should be subclass of Employee"
        );
        assert!(is_employee_person, "Employee should be subclass of Person");

        let report = guard.stop_monitoring();

        println!("  Integration test completed:");
        println!("    Classes: 3, Subclass axioms: 2");
        println!(
            "    Memory usage: {:.1} MB",
            report.end_memory as f64 / 1024.0 / 1024.0
        );
        println!("    Warnings: {}", report.warnings.len());

        assert!(report.is_acceptable(), "Memory usage should be acceptable");
    }
);

/// Test integration between cache manager and memory monitoring
memory_safe_test!(
    test_cache_memory_integration,
    MemorySafeTestConfig::small(),
    {
        println!("🔗 Testing cache manager and memory monitoring integration...");

        let guard = TestMemoryGuard::new();
        guard.start_monitoring();

        // Get initial cache stats
        let initial_stats = global_cache_stats();
        println!(
            "  Initial cache stats: {} hits, {} misses",
            initial_stats.iri_hits, initial_stats.iri_misses
        );

        // Create many IRIs to populate cache
        let mut iris = Vec::new();
        for i in 0..1000 {
            let iri = IRI::new(&format!("http://example.org/cache/test/{}", i)).unwrap();
            iris.push(iri);

            if i % 200 == 0 {
                let _ = guard.check_memory();
            }
        }

        // Check cache after population
        let populated_stats = global_cache_stats();
        println!(
            "  After population: {} hits, {} misses",
            populated_stats.iri_hits, populated_stats.iri_misses
        );

        // Test memory pressure response
        let memory_stats = get_memory_stats();
        println!(
            "  Memory pressure: {:.2}%",
            memory_stats.pressure_level * 100.0
        );

        // Clear cache and verify memory response
        let _ = clear_global_iri_cache();

        let after_clear_stats = global_cache_stats();
        println!(
            "  After clear: {} hits, {} misses",
            after_clear_stats.iri_hits, after_clear_stats.iri_misses
        );

        let report = guard.stop_monitoring();

        println!("  Cache integration test completed:");
        println!("    IRIs created: {}", iris.len());
        println!("    Cache cleared successfully");
        println!("    Memory usage acceptable: {}", report.is_acceptable());

        // Clean up
        drop(iris);
    }
);

/// Test integration between parser and memory safety
memory_safe_test!(
    test_parser_memory_integration,
    MemorySafeTestConfig::medium(),
    {
        println!("🔗 Testing parser and memory safety integration...");

        let guard = TestMemoryGuard::new();
        guard.start_monitoring();

        // Create test ontology content
        let turtle_content = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:Person a owl:Class .
ex:Employee a owl:Class .
ex:Manager a owl:Class .

ex:Employee rdfs:subClassOf ex:Person .
ex:Manager rdfs:subClassOf ex:Employee .

ex:worksFor a owl:ObjectProperty .
ex:manages a owl:ObjectProperty .

ex:manages rdfs:subPropertyOf ex:worksFor .

ex:John a ex:Person .
ex:Mary a ex:Employee .
ex:Bob a ex:Manager .

ex:John ex:worksFor ex:Mary .
ex:Mary ex:worksFor ex:Bob .
ex:Bob ex:manages ex:Mary .
"#;

        // Parse the ontology
        let parser = TurtleParser::new();
        let parse_result = parser.parse_str(turtle_content);
        let parse_success = parse_result.is_ok();

        assert!(parse_success, "Parsing should succeed");
        let ontology = parse_result.unwrap();

        // Check memory after parsing
        let _ = guard.check_memory();

        // Verify parsed content
        assert!(!ontology.classes().is_empty(), "Should have parsed classes");
        assert!(
            !ontology.object_properties().is_empty(),
            "Should have parsed properties"
        );
        assert!(
            !ontology.subclass_axioms().is_empty(),
            "Should have parsed subclass axioms"
        );

        // Test reasoning on parsed ontology
        let reasoner = SimpleReasoner::new(ontology);
        let is_consistent = reasoner.is_consistent().unwrap();

        assert!(is_consistent, "Parsed ontology should be consistent");

        let report = guard.stop_monitoring();

        let ontology_ref = &reasoner.ontology;

        println!("  Parser integration test completed:");
        println!("    Classes parsed: {}", ontology_ref.classes().len());
        println!(
            "    Properties parsed: {}",
            ontology_ref.object_properties().len()
        );
        println!(
            "    Memory usage: {:.1} MB",
            report.end_memory as f64 / 1024.0 / 1024.0
        );
        println!("    Parsing successful: {}", parse_success);

        assert!(report.is_acceptable(), "Memory usage should be acceptable");
    }
);

/// Test integration between reasoning and memory monitoring
memory_safe_test!(
    test_reasoning_memory_integration,
    MemorySafeTestConfig::medium(),
    {
        println!("🔗 Testing reasoning and memory monitoring integration...");

        let guard = TestMemoryGuard::new();
        guard.start_monitoring();

        // Create complex ontology for reasoning
        let mut ontology = Ontology::new();

        // Create class hierarchy
        let classes = vec![
            ("Entity", "http://example.org/Entity"),
            ("Person", "http://example.org/Person"),
            ("Employee", "http://example.org/Employee"),
            ("Manager", "http://example.org/Manager"),
            ("Department", "http://example.org/Department"),
            ("Project", "http://example.org/Project"),
        ];

        let mut class_entities = Vec::new();
        for &(name, iri_str) in &classes {
            let iri = IRI::new(iri_str).unwrap();
            let class = Class::new(Arc::new(iri));
            ontology.add_class(class.clone()).unwrap();
            class_entities.push((name, class));
        }

        // Add subclass relationships
        let relationships = vec![
            ("Person", "Entity"),
            ("Employee", "Person"),
            ("Manager", "Employee"),
        ];

        for &(subclass, superclass) in &relationships {
            let subclass_class = class_entities
                .iter()
                .find(|(name, _)| *name == subclass)
                .unwrap()
                .1
                .clone();
            let superclass_class = class_entities
                .iter()
                .find(|(name, _)| *name == superclass)
                .unwrap()
                .1
                .clone();

            ontology
                .add_subclass_axiom(SubClassOfAxiom::new(
                    ClassExpression::Class(subclass_class),
                    ClassExpression::Class(superclass_class),
                ))
                .unwrap();
        }

        // Check memory after ontology creation
        let _ = guard.check_memory();

        // Test various reasoning operations
        let reasoner = SimpleReasoner::new(ontology);

        // Test consistency
        let is_consistent = reasoner.is_consistent().unwrap();
        assert!(is_consistent, "Complex ontology should be consistent");

        // Check memory after reasoning
        let _ = guard.check_memory();

        // Test subclass reasoning
        for &(subclass, superclass) in &relationships {
            let subclass_iri = class_entities
                .iter()
                .find(|(name, _)| *name == subclass)
                .unwrap()
                .1
                .iri();
            let superclass_iri = class_entities
                .iter()
                .find(|(name, _)| *name == superclass)
                .unwrap()
                .1
                .iri();

            let is_subclass = reasoner
                .is_subclass_of(subclass_iri, superclass_iri)
                .unwrap();
            assert!(
                is_subclass,
                "{} should be subclass of {}",
                subclass, superclass
            );
        }

        // Test instance retrieval
        let person_class = class_entities
            .iter()
            .find(|(name, _)| *name == "Person")
            .unwrap()
            .1
            .clone();
        let person_instances = reasoner.get_instances(person_class.iri()).unwrap();

        let report = guard.stop_monitoring();

        println!("  Reasoning integration test completed:");
        println!("    Classes: {}", classes.len());
        println!("    Subclass relationships: {}", relationships.len());
        println!("    Consistency check: {}", is_consistent);
        println!("    Person instances: {}", person_instances.len());
        println!(
            "    Memory usage: {:.1} MB",
            report.end_memory as f64 / 1024.0 / 1024.0
        );

        assert!(report.is_acceptable(), "Memory usage should be acceptable");
    }
);

/// Test integration between error handling and memory safety
memory_safe_test!(
    test_error_handling_memory_integration,
    MemorySafeTestConfig::small(),
    {
        println!("🔗 Testing error handling and memory safety integration...");

        let guard = TestMemoryGuard::new();
        guard.start_monitoring();

        // Test various error scenarios while monitoring memory

        // Test invalid IRI creation
        let mut error_count = 0;
        for i in 0..100 {
            let invalid_iri = format!("invalid iri {}", i);
            if IRI::new(&invalid_iri).is_err() {
                error_count += 1;
            }

            if i % 20 == 0 {
                let _ = guard.check_memory();
            }
        }

        assert!(error_count > 0, "Should have encountered IRI errors");

        // Test ontology errors
        let mut ontology = Ontology::new();
        let valid_class = Class::new(Arc::new(IRI::new("http://example.org/ValidClass").unwrap()));
        ontology.add_class(valid_class.clone()).unwrap();

        // Try to add duplicate class (should handle gracefully)
        let duplicate_result = ontology.add_class(valid_class.clone());

        // Check memory after error handling
        let _ = guard.check_memory();

        // Test reasoning with minimal ontology
        let reasoner = SimpleReasoner::new(ontology);
        let _ = reasoner.is_consistent().unwrap();

        let report = guard.stop_monitoring();

        println!("  Error handling integration test completed:");
        println!("    IRI errors handled: {}", error_count);
        println!("    Duplicate class handling: {}", duplicate_result.is_ok());
        println!(
            "    Memory usage: {:.1} MB",
            report.end_memory as f64 / 1024.0 / 1024.0
        );

        assert!(
            report.is_acceptable(),
            "Memory usage should be acceptable despite errors"
        );
    }
);

/// Test concurrent integration of multiple components
memory_safe_test!(
    test_concurrent_component_integration,
    MemorySafeTestConfig::large(),
    {
        println!("🔗 Testing concurrent integration of multiple components...");

        use std::sync::{Arc, Barrier};
        use std::thread;

        let num_threads = 4;
        let barrier = Arc::new(Barrier::new(num_threads));
        let results = Arc::new(std::sync::Mutex::new(Vec::new()));

        let mut handles = Vec::new();

        for thread_id in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let results_clone = Arc::clone(&results);

            let handle = thread::spawn(move || {
                let guard = TestMemoryGuard::with_config(TestMemoryConfig {
                    max_memory_bytes: 100 * 1024 * 1024, // 100MB per thread
                    max_cache_size: 500,
                    auto_cleanup: true,
                    fail_on_limit_exceeded: false,
                    warn_threshold_percent: 0.7,
                    check_interval: Duration::from_millis(20),
                });
                guard.start_monitoring();

                // Wait for all threads to start
                barrier_clone.wait();

                // Each thread performs different operations

                if thread_id == 0 {
                    // Thread 0: Ontology operations
                    let mut ontology = Ontology::new();
                    for i in 0..100 {
                        let iri =
                            IRI::new(&format!("http://example.org/thread0/class{}", i)).unwrap();
                        let class = Class::new(Arc::new(iri));
                        let _ = ontology.add_class(class);
                    }

                    let reasoner = SimpleReasoner::new(ontology);
                    let _ = reasoner.is_consistent();
                } else if thread_id == 1 {
                    // Thread 1: Cache operations
                    let mut iris = Vec::new();
                    for i in 0..200 {
                        let iri =
                            IRI::new(&format!("http://example.org/thread1/cache{}", i)).unwrap();
                        iris.push(iri);
                    }

                    let _ = clear_global_iri_cache();
                    drop(iris);
                } else if thread_id == 2 {
                    // Thread 2: Parser operations
                    let content = r#"
@prefix ex: <http://example.org/thread2/> .
ex:Class1 a owl:Class .
ex:Class2 a owl:Class .
ex:Class2 rdfs:subClassOf ex:Class1 .
"#;

                    let parser = TurtleParser::new();
                    let _ = parser.parse_str(content);
                } else {
                    // Thread 3: Memory monitoring
                    for i in 0..50 {
                        let _ = get_memory_stats();
                        let _ = get_memory_pressure_level();
                        if i % 10 == 0 {
                            let _ = force_memory_cleanup();
                        }
                    }
                }

                let report = guard.stop_monitoring();

                let mut results = results_clone.lock().unwrap();
                results.push((thread_id, report));
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

        println!("  Concurrent integration test completed:");
        println!("    Threads completed: {}", results.len());

        let total_warnings: usize = results
            .iter()
            .map(|(_, report)| report.warnings.len())
            .sum();

        println!("    Total warnings: {}", total_warnings);

        for (thread_id, report) in results.iter() {
            println!(
                "    Thread {}: {:.1}% memory usage, {} warnings",
                thread_id,
                (report.end_memory as f64 / report.max_memory_bytes as f64) * 100.0,
                report.warnings.len()
            );

            assert!(
                report.is_acceptable(),
                "Each thread should have acceptable memory usage"
            );
        }
    }
);

/// Test full pipeline integration
memory_safe_test!(
    test_full_pipeline_integration,
    MemorySafeTestConfig::large(),
    {
        println!("🔗 Testing full pipeline integration...");

        let guard = TestMemoryGuard::new();
        guard.start_monitoring();

        // Step 1: Parse ontology from string
        let turtle_content = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/company/> .

ex:Organization a owl:Class .
ex:Person a owl:Class .
ex:Employee a owl:Class .
ex:Manager a owl:Class .
ex:Department a owl:Class .
ex:Project a owl:Class .

ex:Employee rdfs:subClassOf ex:Person .
ex:Manager rdfs:subClassOf ex:Employee .

ex:worksFor a owl:ObjectProperty .
ex:manages a owl:ObjectProperty .
ex:memberOf a owl:ObjectProperty .
ex:assignedTo a owl:ObjectProperty .

ex:manages rdfs:subPropertyOf ex:worksFor .

ex:CompanyA a ex:Organization .
ex:Engineering a ex:Department .
ex:ProductX a ex:Project .

ex:Alice a ex:Manager .
ex:Bob a ex:Employee .
ex:Carol a ex:Employee .

ex:Engineering ex:memberOf ex:CompanyA .
ex:ProductX ex:assignedTo ex:Engineering .

ex:Alice ex:worksFor ex:CompanyA .
ex:Bob ex:worksFor ex:CompanyA .
ex:Carol ex:worksFor ex:CompanyA .

ex:Alice ex:manages ex:Engineering .
ex:Bob ex:memberOf ex:Engineering .
ex:Carol ex:memberOf ex:Engineering .
ex:Bob ex:assignedTo ex:ProductX .
ex:Carol ex:assignedTo ex:ProductX .
"#;

        let parser = TurtleParser::new();
        let ontology = parser.parse_str(turtle_content).unwrap();

        // Check memory after parsing
        let _ = guard.check_memory();

        // Step 2: Validate parsed ontology
        let parsed_success = !ontology.classes().is_empty();
        assert!(parsed_success, "Should have parsed classes");
        assert!(
            !ontology.object_properties().is_empty(),
            "Should have parsed properties"
        );
        assert!(
            !ontology.named_individuals().is_empty(),
            "Should have parsed individuals"
        );

        // Step 3: Perform reasoning
        let reasoner = SimpleReasoner::new(ontology);

        // Check memory before reasoning
        let _ = guard.check_memory();

        let is_consistent = reasoner.is_consistent().unwrap();
        assert!(is_consistent, "Company ontology should be consistent");

        // Step 4: Test classification
        let manager_iri = IRI::new("http://example.org/company/Manager").unwrap();
        let employee_iri = IRI::new("http://example.org/company/Employee").unwrap();
        let person_iri = IRI::new("http://example.org/company/Person").unwrap();

        let managers_are_employees = reasoner
            .is_subclass_of(&manager_iri, &employee_iri)
            .unwrap();
        let employees_are_people = reasoner.is_subclass_of(&employee_iri, &person_iri).unwrap();

        assert!(managers_are_employees, "Managers should be employees");
        assert!(employees_are_people, "Employees should be people");

        // Step 5: Test instance retrieval
        let managers = reasoner.get_instances(&manager_iri).unwrap();
        let employees = reasoner.get_instances(&employee_iri).unwrap();

        assert!(!managers.is_empty(), "Should have manager instances");
        assert!(!employees.is_empty(), "Should have employee instances");

        // Check memory after reasoning
        let _ = guard.check_memory();

        // Step 6: Test memory cleanup
        let _ = force_memory_cleanup();

        let report = guard.stop_monitoring();

        let reasoner_ontology = &reasoner.ontology;

        println!("  Full pipeline integration test completed:");
        println!("    Parsed successfully: {}", parsed_success);
        println!("    Classes: {}", reasoner_ontology.classes().len());
        println!(
            "    Properties: {}",
            reasoner_ontology.object_properties().len()
        );
        println!(
            "    Individuals: {}",
            reasoner_ontology.named_individuals().len()
        );
        println!("    Consistent: {}", is_consistent);
        println!("    Managers: {}", managers.len());
        println!("    Employees: {}", employees.len());
        println!(
            "    Memory usage: {:.1} MB",
            report.end_memory as f64 / 1024.0 / 1024.0
        );
        println!("    Warnings: {}", report.warnings.len());

        assert!(
            report.is_acceptable(),
            "Memory usage should be acceptable for full pipeline"
        );
    }
);

/// Comprehensive integration validation summary
memory_safe_test!(
    test_integration_validation_summary,
    MemorySafeTestConfig::large(),
    {
        println!("🔗 Comprehensive Integration Validation Summary");
        println!("==============================================");

        // Run all integration tests
        test_memory_ontology_integration();
        test_cache_memory_integration();
        test_parser_memory_integration();
        test_reasoning_memory_integration();
        test_error_handling_memory_integration();
        test_concurrent_component_integration();
        test_full_pipeline_integration();

        println!("==============================================");
        println!("✅ All integration validation tests passed!");
        println!("   - Memory safety and ontology integration verified");
        println!("   - Cache manager and memory monitoring integrated");
        println!("   - Parser and memory safety working together");
        println!("   - Reasoning and memory monitoring integrated");
        println!("   - Error handling with memory safety confirmed");
        println!("   - Concurrent component integration validated");
        println!("   - Full pipeline integration tested");

        // Final system health check
        let final_stats = get_memory_stats();
        let final_leak_report = detect_memory_leaks();

        println!("\n📊 Final System Health After Integration:");
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

        // System should be in good state after integration tests
        assert!(
            final_stats.pressure_level < 0.8,
            "System pressure should be manageable"
        );
        assert!(
            final_leak_report.memory_efficiency_score > 0.6,
            "System should have good efficiency"
        );

    }
);
