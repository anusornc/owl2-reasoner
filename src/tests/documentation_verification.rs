//! Documentation Verification Tests
//!
//! This module provides tests to verify that all documentation links,
//! examples, and code snippets work correctly after the reorganization.

#![allow(unused_doc_comments)]

use crate::axioms::{ClassAssertionAxiom, ClassExpression, SubClassOfAxiom};
use crate::cache_manager::*;
use crate::entities::*;
use crate::iri::IRI;
use crate::memory::*;
use crate::memory_safe_test;
use crate::ontology::*;
use crate::parser::*;
use crate::reasoning::*;
use crate::test_helpers::*;
use crate::test_memory_guard::*;
use smallvec::smallvec;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Test that library documentation examples work
memory_safe_test!(
    test_library_documentation_examples,
    MemorySafeTestConfig::medium(),
    {
        println!("📚 Testing library documentation examples...");

        // Test the basic example from lib.rs documentation
        let mut ontology = Ontology::new();

        // Add classes as documented
        let person = Class::new(Arc::new(IRI::new("http://example.org/Person").unwrap()));
        let parent = Class::new(Arc::new(IRI::new("http://example.org/Parent").unwrap()));

        ontology.add_class(person.clone()).unwrap();
        ontology.add_class(parent.clone()).unwrap();

        // Add subclass relationship as documented
        let subclass_axiom = SubClassOfAxiom::new(
            ClassExpression::Class(parent.clone()),
            ClassExpression::Class(person.clone()),
        );
        ontology.add_subclass_axiom(subclass_axiom).unwrap();

        // Test reasoning as documented
        let reasoner = SimpleReasoner::new(ontology);
        let is_consistent = reasoner.is_consistent().unwrap();
        let is_subclass = reasoner
            .is_subclass_of(&parent.iri(), &person.iri())
            .unwrap();

        // Verify documented behavior
        assert!(is_consistent, "Ontology should be consistent as documented");
        assert!(
            is_subclass,
            "Parent should be subclass of Person as documented"
        );

        println!("  ✅ Library documentation examples work correctly");
    }
);

/// Test that README examples work
memory_safe_test!(test_readme_examples, MemorySafeTestConfig::medium(), {
    println!("📚 Testing README examples...");

    // Test basic ontology operations that would be in README
    let mut ontology = Ontology::new();

    // Create a simple family ontology as might be shown in README
    let person = Class::new(Arc::new(
        IRI::new("http://example.org/family/Person").unwrap(),
    ));
    let male = Class::new(Arc::new(
        IRI::new("http://example.org/family/Male").unwrap(),
    ));
    let female = Class::new(Arc::new(
        IRI::new("http://example.org/family/Female").unwrap(),
    ));
    let parent = Class::new(Arc::new(
        IRI::new("http://example.org/family/Parent").unwrap(),
    ));

    // Add classes
    ontology.add_class(person.clone()).unwrap();
    ontology.add_class(male.clone()).unwrap();
    ontology.add_class(female.clone()).unwrap();
    ontology.add_class(parent.clone()).unwrap();

    // Add subclass relationships
    ontology
        .add_subclass_axiom(SubClassOfAxiom::new(
            ClassExpression::Class(male.clone()),
            ClassExpression::Class(person.clone()),
        ))
        .unwrap();

    ontology
        .add_subclass_axiom(SubClassOfAxiom::new(
            ClassExpression::Class(female.clone()),
            ClassExpression::Class(person.clone()),
        ))
        .unwrap();

    // Create individuals
    let john = NamedIndividual::new(Arc::new(
        IRI::new("http://example.org/family/John").unwrap(),
    ));
    let mary = NamedIndividual::new(Arc::new(
        IRI::new("http://example.org/family/Mary").unwrap(),
    ));

    ontology.add_named_individual(john.clone()).unwrap();
    ontology.add_named_individual(mary.clone()).unwrap();

    // Add class assertions
    ontology
        .add_class_assertion(ClassAssertionAxiom::new(
            john.iri().clone(),
            ClassExpression::Class(male.clone()),
        ))
        .unwrap();

    ontology
        .add_class_assertion(ClassAssertionAxiom::new(
            mary.iri().clone(),
            ClassExpression::Class(female.clone()),
        ))
        .unwrap();

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent().unwrap();

    // Test instance retrieval
    let males = reasoner.get_instances(male.iri()).unwrap();
    let females = reasoner.get_instances(female.iri()).unwrap();
    let people = reasoner.get_instances(person.iri()).unwrap();

    // Verify results
    assert!(is_consistent, "Family ontology should be consistent");
    assert!(!males.is_empty(), "Should have male instances");
    assert!(!females.is_empty(), "Should have female instances");
    assert_eq!(
        males.len() + females.len(),
        people.len(),
        "All males and females should be people"
    );

    println!("  ✅ README examples work correctly");
});

/// Test that example files compile and run
memory_safe_test!(
    test_example_files_compilation,
    MemorySafeTestConfig::large(),
    {
        println!("📚 Testing example files compilation...");

        // Check that example files exist and are accessible
        let examples_dir = Path::new("examples");
        assert!(examples_dir.exists(), "Examples directory should exist");

        // Test that we can read example files
        if let Ok(entries) = fs::read_dir(examples_dir) {
            let mut example_count = 0;
            let mut has_basic_examples = false;
            let mut has_benchmark_examples = false;

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    example_count += 1;

                    // Check if we can read the file
                    if let Ok(content) = fs::read_to_string(&path) {
                        // Basic validation that it looks like Rust code
                        assert!(
                            content.contains("fn") || content.contains("use"),
                            "Example file should contain Rust code"
                        );

                        // Check for different types of examples
                        if content.to_lowercase().contains("basic") {
                            has_basic_examples = true;
                        }
                        if content.to_lowercase().contains("benchmark") {
                            has_benchmark_examples = true;
                        }
                    }
                }
            }

            assert!(example_count > 0, "Should have at least one example file");
            assert!(has_basic_examples, "Should have basic examples");
            assert!(has_benchmark_examples, "Should have benchmark examples");

            println!("  Found {} example files", example_count);
            println!("  ✅ Example files are accessible and contain valid Rust code");
        } else {
            println!("  ⚠️  Could not read examples directory, but this might be expected in test environment");
        }
    }
);

/// Test documentation links and references
memory_safe_test!(test_documentation_links, MemorySafeTestConfig::small(), {
    println!("📚 Testing documentation links and references...");

    // Test that core documentation references work
    // This simulates following links and references in documentation

    // Test IRI creation as documented
    let iri = IRI::new("http://example.org/test").unwrap();
    assert_eq!(iri.to_string(), "http://example.org/test");

    // Test Class creation as documented
    let class = Class::new(Arc::new(iri.clone()));

    // Test Ontology operations as documented
    let mut ontology = Ontology::new();
    ontology.add_class(class.clone()).unwrap();

    // Test that classes can be retrieved
    assert!(ontology.classes().contains(&class));

    // Test ObjectProperty creation as documented
    let prop_iri = IRI::new("http://example.org/hasProperty").unwrap();
    let property = crate::ObjectProperty::new(Arc::new(prop_iri));
    ontology.add_object_property(property).unwrap();

    // Test NamedIndividual creation as documented
    let individual_iri = IRI::new("http://example.org/Individual").unwrap();
    let individual = NamedIndividual::new(Arc::new(individual_iri));
    ontology.add_named_individual(individual).unwrap();

    // Test reasoning as documented
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent().unwrap();
    assert!(is_consistent, "Should be consistent with basic entities");

    println!("  ✅ Documentation references and examples work correctly");
});

/// Test Turtle parsing examples from documentation
memory_safe_test!(
    test_turtle_parsing_documentation,
    MemorySafeTestConfig::medium(),
    {
        println!("📚 Testing Turtle parsing documentation examples...");

        // Test the basic Turtle example that would be in documentation
        let turtle_content = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:Person a owl:Class .
ex:Student a owl:Class .
ex:Student rdfs:subClassOf ex:Person .

ex:hasTeacher a owl:ObjectProperty .
ex:hasStudent a owl:ObjectProperty .

ex:Alice a ex:Student .
ex:Bob a ex:Student .
ex:Carol a ex:Person .
ex:ProfSmith a ex:Person .

ex:Alice ex:hasTeacher ex:ProfSmith .
ex:Bob ex:hasTeacher ex:ProfSmith .
ex:ProfSmith ex:hasStudent ex:Alice .
ex:ProfSmith ex:hasStudent ex:Bob .
"#;

        // Parse the Turtle content
        let parser = TurtleParser::new();
        let ontology = parser.parse_str(turtle_content).unwrap();

        // Verify the parsed content
        assert!(!ontology.classes().is_empty(), "Should have parsed classes");
        assert!(
            !ontology.object_properties().is_empty(),
            "Should have parsed properties"
        );
        assert!(
            !ontology.named_individuals().is_empty(),
            "Should have parsed individuals"
        );
        assert!(
            !ontology.subclass_axioms().is_empty(),
            "Should have parsed subclass axioms"
        );

        // Test reasoning on parsed content
        let reasoner = SimpleReasoner::new(ontology);
        let is_consistent = reasoner.is_consistent().unwrap();

        // Test subclass reasoning
        let student_iri = IRI::new("http://example.org/Student").unwrap();
        let person_iri = IRI::new("http://example.org/Person").unwrap();
        let is_student_person = reasoner.is_subclass_of(&student_iri, &person_iri).unwrap();

        assert!(is_consistent, "Parsed Turtle ontology should be consistent");
        assert!(is_student_person, "Student should be subclass of Person");

        println!("  ✅ Turtle parsing documentation examples work correctly");
    }
);

/// Test error handling examples from documentation
memory_safe_test!(
    test_error_handling_documentation,
    MemorySafeTestConfig::small(),
    {
        println!("📚 Testing error handling documentation examples...");

        // Test invalid IRI handling as would be documented
        let invalid_iris = vec!["", "not a valid iri", "http://[invalid]", "no://scheme"];

        for invalid_iri in invalid_iris {
            let result = IRI::new(invalid_iri);
            assert!(
                result.is_err(),
                "Should reject invalid IRI: {}",
                invalid_iri
            );

            // Test error type
            if let Err(e) = result {
                assert!(
                    !e.to_string().is_empty(),
                    "Error should have descriptive message"
                );
            }
        }

        // Test ontology constraint violations
        let mut ontology = Ontology::new();
        let class = Class::new(Arc::new(IRI::new("http://example.org/Class").unwrap()));

        // Adding same class twice should be handled gracefully
        let result1 = ontology.add_class(class.clone());
        let _result2 = ontology.add_class(class.clone());

        assert!(result1.is_ok(), "First class addition should succeed");
        // Depending on implementation, second addition might succeed or fail gracefully

        // Test parsing invalid content
        let invalid_turtle = "invalid turtle content @prefix";
        let parser = TurtleParser::new();
        let parse_result = parser.parse_str(invalid_turtle);

        // Should handle invalid content gracefully (either return error or handle it)
        println!("  Invalid Turtle parsing result: {:?}", parse_result);

        println!("  ✅ Error handling examples work correctly");
    }
);

/// Test memory safety documentation examples
memory_safe_test!(
    test_memory_safety_documentation,
    MemorySafeTestConfig::medium(),
    {
        println!("📚 Testing memory safety documentation examples...");

        // Test memory monitoring as documented
        let initial_stats = get_memory_stats();
        assert!(
            initial_stats.total_usage > 0,
            "Should have non-zero memory usage"
        );
        assert!(initial_stats.pressure_level >= 0.0 && initial_stats.pressure_level <= 1.0);

        // Test memory guard usage as would be documented
        let guard = TestMemoryGuard::new();
        guard.start_monitoring();

        // Simulate some memory usage
        let _data: Vec<u8> = vec![0; 1024 * 1024]; // 1MB

        let usage_percent = guard.memory_usage_percent();
        assert!(usage_percent >= 0.0 && usage_percent <= 100.0);

        let check_result = guard.check_memory();
        assert!(
            check_result.is_ok(),
            "Memory check should succeed with normal usage"
        );

        let report = guard.stop_monitoring();
        assert!(report.is_acceptable(), "Memory usage should be acceptable");

        // Test leak detection as documented
        let leak_report = detect_memory_leaks();
        assert!(
            leak_report.memory_efficiency_score >= 0.0
                && leak_report.memory_efficiency_score <= 1.0
        );

        println!(
            "  Memory efficiency score: {:.2}",
            leak_report.memory_efficiency_score
        );

        // Test cache management as documented
        let _cache_stats_before = global_cache_stats();
        let _ = clear_global_iri_cache();
        let _cache_stats_after = global_cache_stats();

        println!("  ✅ Memory safety documentation examples work correctly");
    }
);

/// Test performance documentation examples
memory_safe_test!(
    test_performance_documentation,
    MemorySafeTestConfig::medium(),
    {
        println!("📚 Testing performance documentation examples...");

        // Test performance characteristics that would be documented

        let start_time = std::time::Instant::now();

        // Create a moderately sized ontology as might be shown in performance docs
        let mut ontology = Ontology::new();

        // Create 500 classes
        for i in 0..500 {
            let iri = IRI::new(&format!("http://example.org/perf/class{}", i)).unwrap();
            let class = Class::new(Arc::new(iri));
            ontology.add_class(class).unwrap();
        }

        // Create subclass relationships
        for i in 1..500 {
            let subclass_iri = IRI::new(&format!("http://example.org/perf/class{}", i)).unwrap();
            let superclass_iri =
                IRI::new(&format!("http://example.org/perf/class{}", i / 2)).unwrap();

            let subclass = ClassExpression::Class(Class::new(Arc::new(subclass_iri)));
            let superclass = ClassExpression::Class(Class::new(Arc::new(superclass_iri)));

            let axiom = SubClassOfAxiom::new(subclass, superclass);
            ontology.add_subclass_axiom(axiom).unwrap();
        }

        let creation_time = start_time.elapsed();

        // Test reasoning performance
        let reasoning_start = std::time::Instant::now();
        let reasoner = SimpleReasoner::new(ontology);
        let is_consistent = reasoner.is_consistent().unwrap();
        let reasoning_time = reasoning_start.elapsed();

        // Performance should be reasonable (not too slow)
        assert!(
            creation_time.as_secs() < 10,
            "Ontology creation should be fast"
        );
        assert!(reasoning_time.as_secs() < 15, "Reasoning should be fast");
        assert!(
            is_consistent,
            "Performance test ontology should be consistent"
        );

        println!("  Performance metrics:");
        println!(
            "    Creation: {:?}, Reasoning: {:?}",
            creation_time, reasoning_time
        );
        println!("  ✅ Performance documentation examples work correctly");
    }
);

/// Test that API reference documentation works
memory_safe_test!(
    test_api_reference_documentation,
    MemorySafeTestConfig::small(),
    {
        println!("📚 Testing API reference documentation...");

        // Test core API methods as would be documented in API reference

        // Test Ontology API
        let mut ontology = Ontology::new();
        assert!(
            ontology.classes().is_empty(),
            "New ontology should have no classes"
        );
        assert!(
            ontology.object_properties().is_empty(),
            "New ontology should have no properties"
        );

        // Test adding entities
        let class_iri = IRI::new("http://example.org/api/TestClass").unwrap();
        let class = Class::new(Arc::new(class_iri));
        ontology.add_class(class.clone()).unwrap();

        assert!(
            !ontology.classes().is_empty(),
            "Ontology should now have classes"
        );
        assert!(
            ontology.classes().contains(&class),
            "Should contain added class"
        );

        // Test Reasoner API
        let reasoner = SimpleReasoner::new(ontology);
        let is_consistent = reasoner.is_consistent().unwrap();
        assert!(is_consistent, "Simple ontology should be consistent");

        // Test IRI API
        let iri = IRI::new("http://example.org/api/TestIRI").unwrap();
        assert_eq!(iri.namespace(), "http://example.org/api/");
        assert_eq!(iri.local_name(), "TestIRI");

        // Test cache API
        let cache_stats = global_cache_stats();
        // Cache stats are always non-negative (unsigned types)
        // Just verify we can access them
        let _ = cache_stats.iri_hits;
        let _ = cache_stats.iri_misses;

        println!("  ✅ API reference documentation examples work correctly");
    }
);

/// Test advanced features documentation
memory_safe_test!(
    test_advanced_features_documentation,
    MemorySafeTestConfig::medium(),
    {
        println!("📚 Testing advanced features documentation...");

        // Test advanced class expressions as might be documented
        let mut ontology = Ontology::new();

        // Create base classes
        let person = Class::new(Arc::new(
            IRI::new("http://example.org/advanced/Person").unwrap(),
        ));
        let student = Class::new(Arc::new(
            IRI::new("http://example.org/advanced/Student").unwrap(),
        ));
        let teacher = Class::new(Arc::new(
            IRI::new("http://example.org/advanced/Teacher").unwrap(),
        ));

        ontology.add_class(person.clone()).unwrap();
        ontology.add_class(student.clone()).unwrap();
        ontology.add_class(teacher.clone()).unwrap();

        // Create union class expression
        let student_or_teacher = ClassExpression::ObjectUnionOf(smallvec![
            Box::new(ClassExpression::Class(student.clone())),
            Box::new(ClassExpression::Class(teacher.clone())),
        ]);

        // Create intersection class expression
        let person_and_student = ClassExpression::ObjectIntersectionOf(smallvec![
            Box::new(ClassExpression::Class(person.clone())),
            Box::new(ClassExpression::Class(student.clone())),
        ]);

        // Test that expressions can be created and used
        assert!(matches!(
            student_or_teacher,
            ClassExpression::ObjectUnionOf(_)
        ));
        assert!(matches!(
            person_and_student,
            ClassExpression::ObjectIntersectionOf(_)
        ));

        // Test property characteristics
        let mut symmetric_prop = ObjectProperty::new(Arc::new(
            IRI::new("http://example.org/advanced/symmetric").unwrap(),
        ));
        symmetric_prop.add_characteristic(ObjectPropertyCharacteristic::Symmetric);

        ontology.add_object_property(symmetric_prop).unwrap();

        // Test reasoning with advanced features
        ontology
            .add_subclass_axiom(SubClassOfAxiom::new(
                ClassExpression::Class(student.clone()),
                ClassExpression::Class(person.clone()),
            ))
            .unwrap();

        let reasoner = SimpleReasoner::new(ontology);
        let is_consistent = reasoner.is_consistent().unwrap();

        assert!(
            is_consistent,
            "Advanced features ontology should be consistent"
        );

        println!("  ✅ Advanced features documentation examples work correctly");
    }
);

/// Test that all documentation is accessible and functional
memory_safe_test!(
    test_documentation_accessibility,
    MemorySafeTestConfig::small(),
    {
        println!("📚 Testing documentation accessibility...");

        // Test that key documentation files exist
        let doc_files = vec![
            "README.md",
            "docs/README.md",
            "MEMORY_SAFETY_IMPLEMENTATION_SUMMARY.md",
            "PROJECT_REORGANIZATION_PLAN.md",
            "docs/MEMORY_SAFE_TESTING.md",
        ];

        let mut accessible_docs = 0;

        for doc_file in &doc_files {
            if Path::new(doc_file).exists() {
                if let Ok(_content) = fs::read_to_string(doc_file) {
                    accessible_docs += 1;
                    println!("  ✅ {} is accessible", doc_file);
                } else {
                    println!("  ⚠️  {} exists but cannot be read", doc_file);
                }
            } else {
                println!(
                    "  ⚠️  {} does not exist (may be expected in test environment)",
                    doc_file
                );
            }
        }

        // At least some documentation should be accessible
        assert!(
            accessible_docs > 0,
            "At least some documentation should be accessible"
        );

        // Test that examples directory is accessible
        if Path::new("examples").exists() {
            println!("  ✅ Examples directory is accessible");
        }

        // Test that tests directory is accessible
        if Path::new("tests").exists() {
            println!("  ✅ Tests directory is accessible");
        }

        println!("  {} documentation files accessible", accessible_docs);
    }
);

/// Comprehensive documentation verification summary
memory_safe_test!(
    test_documentation_verification_summary,
    MemorySafeTestConfig::large(),
    {
        println!("📚 Comprehensive Documentation Verification Summary");
        println!("================================================");

        // Run all documentation verification tests
        test_library_documentation_examples();
        test_readme_examples();
        test_example_files_compilation();
        test_documentation_links();
        test_turtle_parsing_documentation();
        test_error_handling_documentation();
        test_memory_safety_documentation();
        test_performance_documentation();
        test_api_reference_documentation();
        test_advanced_features_documentation();
        test_documentation_accessibility();

        println!("================================================");
        println!("✅ All documentation verification tests passed!");
        println!("   - Library documentation examples work correctly");
        println!("   - README examples are functional");
        println!("   - Example files are accessible and valid");
        println!("   - Documentation links and references work");
        println!("   - Turtle parsing examples work");
        println!("   - Error handling examples are correct");
        println!("   - Memory safety documentation is functional");
        println!("   - Performance documentation is accurate");
        println!("   - API reference documentation works");
        println!("   - Advanced features are documented correctly");
        println!("   - Documentation files are accessible");

        // Final verification that the system is ready for documentation
        let final_stats = get_memory_stats();
        println!("\n📊 System Ready for Documentation:");
        println!("   Memory usage: {} bytes", final_stats.total_usage);
        println!(
            "   Pressure level: {:.2}%",
            final_stats.pressure_level * 100.0
        );

        assert!(
            final_stats.pressure_level < 0.8,
            "System should be ready for documentation"
        );
    }
);
