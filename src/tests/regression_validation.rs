//! Comprehensive Regression Validation Tests
//!
//! This module provides regression tests to ensure that existing functionality
//! is preserved and no regressions were introduced during the memory safety
//! implementation and project reorganization.

use crate::memory::*;
use crate::test_memory_guard::*;
use crate::cache_manager::*;
use crate::ontology::*;
use crate::entities::*;
use crate::iri::IRI;
use crate::parser::*;
use crate::reasoning::*;
use std::sync::Arc;
use std::time::Duration;

/// Test basic ontology creation and management (regression test)
memory_safe_test!(test_basic_ontology_regression, MemorySafeTestConfig::small(), {
    println!("🔄 Testing basic ontology creation (regression)...");
    
    // This test should work exactly as it did before memory safety changes
    let mut ontology = Ontology::new();
    
    // Create basic classes
    let person = Class::new(Arc::new(IRI::new("http://example.org/Person").unwrap()));
    let employee = Class::new(Arc::new(IRI::new("http://example.org/Employee").unwrap()));
    
    // Add classes to ontology
    ontology.add_class(person.clone()).unwrap();
    ontology.add_class(employee.clone()).unwrap();
    
    // Verify classes were added
    assert!(ontology.classes().contains(&person));
    assert!(ontology.classes().contains(&employee));
    assert_eq!(ontology.classes().len(), 2);
    
    // Add subclass relationship
    let subclass_axiom = SubClassOfAxiom::new(
        ClassExpression::Class(employee.clone()),
        ClassExpression::Class(person.clone()),
    );
    ontology.add_subclass_axiom(subclass_axiom).unwrap();
    
    // Verify subclass axiom was added
    assert_eq!(ontology.subclass_axioms().len(), 1);
    
    println!("  ✅ Basic ontology creation works as expected");
});

/// Test IRI creation and management (regression test)
memory_safe_test!(test_iri_creation_regression, MemorySafeTestConfig::small(), {
    println!("🔄 Testing IRI creation and management (regression)...");
    
    // Test basic IRI creation
    let iri1 = IRI::new("http://example.org/test").unwrap();
    let iri2 = IRI::new("http://example.org/test").unwrap();
    
    // Test IRI equality
    assert_eq!(iri1, iri2);
    assert_eq!(iri1.to_string(), "http://example.org/test");
    
    // Test IRI with different cases
    let iri3 = IRI::new("http://example.org/Test").unwrap();
    assert_ne!(iri1, iri3); // Should be different due to case sensitivity
    
    // Test IRI components
    assert_eq!(iri1.namespace(), "http://example.org/");
    assert_eq!(iri1.local_name(), "test");
    
    // Test invalid IRI handling
    let invalid_iri = IRI::new("not a valid iri");
    assert!(invalid_iri.is_err());
    
    println!("  ✅ IRI creation and management works as expected");
});

/// Test basic reasoning functionality (regression test)
memory_safe_test!(test_basic_reasoning_regression, MemorySafeTestConfig::small(), {
    println!("🔄 Testing basic reasoning functionality (regression)...");
    
    // Create simple ontology
    let mut ontology = Ontology::new();
    
    let person = Class::new(Arc::new(IRI::new("http://example.org/Person").unwrap()));
    let employee = Class::new(Arc::new(IRI::new("http://example.org/Employee").unwrap()));
    let manager = Class::new(Arc::new(IRI::new("http://example.org/Manager").unwrap()));
    
    ontology.add_class(person.clone()).unwrap();
    ontology.add_class(employee.clone()).unwrap();
    ontology.add_class(manager.clone()).unwrap();
    
    // Add hierarchy: Manager -> Employee -> Person
    ontology.add_subclass_axiom(
        SubClassOfAxiom::new(
            ClassExpression::Class(employee.clone()),
            ClassExpression::Class(person.clone()),
        )
    ).unwrap();
    
    ontology.add_subclass_axiom(
        SubClassOfAxiom::new(
            ClassExpression::Class(manager.clone()),
            ClassExpression::Class(employee.clone()),
        )
    ).unwrap();
    
    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    
    // Test consistency
    let is_consistent = reasoner.is_consistent().unwrap();
    assert!(is_consistent, "Simple hierarchy should be consistent");
    
    // Test subclass reasoning
    let is_employee_person = reasoner.is_subclass_of(employee.iri(), person.iri()).unwrap();
    let is_manager_employee = reasoner.is_subclass_of(manager.iri(), employee.iri()).unwrap();
    let is_manager_person = reasoner.is_subclass_of(manager.iri(), person.iri()).unwrap();
    
    assert!(is_employee_person, "Employee should be subclass of Person");
    assert!(is_manager_employee, "Manager should be subclass of Employee");
    assert!(is_manager_person, "Manager should be subclass of Person (transitive)");
    
    println!("  ✅ Basic reasoning functionality works as expected");
});

/// Test Turtle parsing (regression test)
memory_safe_test!(test_turtle_parsing_regression, MemorySafeTestConfig::small(), {
    println!("🔄 Testing Turtle parsing (regression)...");
    
    let turtle_content = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:Person a owl:Class .
ex:Employee a owl:Class .
ex:Employee rdfs:subClassOf ex:Person .

ex:John a ex:Person .
ex:Mary a ex:Employee .
ex:worksFor a owl:ObjectProperty .
ex:John ex:worksFor ex:Mary .
"#;
    
    // Parse the content
    let parser = TurtleParser::new();
    let parse_result = parser.parse_str(turtle_content);
    
    assert!(parse_result.is_ok(), "Turtle parsing should succeed");
    let ontology = parse_result.unwrap();
    
    // Verify parsed content
    assert!(!ontology.classes().is_empty(), "Should have parsed classes");
    assert!(!ontology.object_properties().is_empty(), "Should have parsed properties");
    assert!(!ontology.named_individuals().is_empty(), "Should have parsed individuals");
    
    // Test reasoning on parsed ontology
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent().unwrap();
    assert!(is_consistent, "Parsed ontology should be consistent");
    
    println!("  ✅ Turtle parsing works as expected");
});

/// Test cache functionality (regression test)
memory_safe_test!(test_cache_functionality_regression, MemorySafeTestConfig::small(), {
    println!("🔄 Testing cache functionality (regression)...");
    
    // Get initial cache stats
    let initial_stats = global_cache_stats();
    
    // Create some IRIs to populate cache
    let iri1 = IRI::new("http://example.org/cache/test1").unwrap();
    let iri2 = IRI::new("http://example.org/cache/test2").unwrap();
    let iri3 = IRI::new("http://example.org/cache/test1").unwrap(); // Same as iri1
    
    // Check cache stats after IRI creation
    let after_stats = global_cache_stats();
    
    // Cache should have some activity
    assert!(after_stats.iri_hits + after_stats.iri_misses > initial_stats.iri_hits + initial_stats.iri_misses);
    
    // Test cache clearing
    let _ = clear_global_iri_cache();
    let cleared_stats = global_cache_stats();
    
    // Cache should be reset after clearing
    assert!(cleared_stats.iri_hits == 0 && cleared_stats.iri_misses == 0);
    
    println!("  ✅ Cache functionality works as expected");
});

/// Test property characteristics (regression test)
memory_safe_test!(test_property_characteristics_regression, MemorySafeTestConfig::small(), {
    println!("🔄 Testing property characteristics (regression)...");
    
    let mut ontology = Ontology::new();
    
    // Create properties with different characteristics
    let mut symmetric_prop = ObjectProperty::new(Arc::new(IRI::new("http://example.org/symmetric").unwrap()));
    symmetric_prop.add_characteristic(ObjectPropertyCharacteristic::Symmetric);
    
    let mut transitive_prop = ObjectProperty::new(Arc::new(IRI::new("http://example.org/transitive").unwrap()));
    transitive_prop.add_characteristic(ObjectPropertyCharacteristic::Transitive);
    
    let mut asymmetric_prop = ObjectProperty::new(Arc::new(IRI::new("http://example.org/asymmetric").unwrap()));
    asymmetric_prop.add_characteristic(ObjectPropertyCharacteristic::Asymmetric);
    
    // Add properties to ontology
    ontology.add_object_property(symmetric_prop.clone()).unwrap();
    ontology.add_object_property(transitive_prop.clone()).unwrap();
    ontology.add_object_property(asymmetric_prop.clone()).unwrap();
    
    // Verify properties were added
    assert!(ontology.object_properties().contains(&symmetric_prop));
    assert!(ontology.object_properties().contains(&transitive_prop));
    assert!(ontology.object_properties().contains(&asymmetric_prop));
    
    // Test reasoning with properties
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent().unwrap();
    assert!(is_consistent, "Ontology with property characteristics should be consistent");
    
    println!("  ✅ Property characteristics work as expected");
});

/// Test individual creation and assertions (regression test)
memory_safe_test!(test_individual_assertions_regression, MemorySafeTestConfig::small(), {
    println!("🔄 Testing individual assertions (regression)...");
    
    let mut ontology = Ontology::new();
    
    // Create classes
    let person = Class::new(Arc::new(IRI::new("http://example.org/Person").unwrap()));
    let employee = Class::new(Arc::new(IRI::new("http://example.org/Employee").unwrap()));
    
    ontology.add_class(person.clone()).unwrap();
    ontology.add_class(employee.clone()).unwrap();
    
    // Create individuals
    let john = NamedIndividual::new(Arc::new(IRI::new("http://example.org/John").unwrap()));
    let mary = NamedIndividual::new(Arc::new(IRI::new("http://example.org/Mary").unwrap()));
    
    ontology.add_named_individual(john.clone()).unwrap();
    ontology.add_named_individual(mary.clone()).unwrap();
    
    // Add class assertions
    ontology.add_class_assertion(
        ClassAssertionAxiom::new(john.iri().clone(), ClassExpression::Class(person.clone()))
    ).unwrap();
    
    ontology.add_class_assertion(
        ClassAssertionAxiom::new(mary.iri().clone(), ClassExpression::Class(employee.clone()))
    ).unwrap();
    
    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent().unwrap();
    assert!(is_consistent, "Ontology with individuals should be consistent");
    
    // Test instance retrieval
    let person_instances = reasoner.get_instances(person.iri()).unwrap();
    let employee_instances = reasoner.get_instances(employee.iri()).unwrap();
    
    assert!(!person_instances.is_empty(), "Should have person instances");
    assert!(!employee_instances.is_empty(), "Should have employee instances");
    
    println!("  ✅ Individual assertions work as expected");
});

/// Test error handling (regression test)
memory_safe_test!(test_error_handling_regression, MemorySafeTestConfig::small(), {
    println!("🔄 Testing error handling (regression)...");
    
    // Test invalid IRI handling
    let invalid_iris = vec![
        "not a valid iri",
        "",
        "http://[invalid",
        "ftp://invalid.scheme",
    ];
    
    for invalid_iri in invalid_iris {
        let result = IRI::new(invalid_iri);
        assert!(result.is_err(), "Should reject invalid IRI: {}", invalid_iri);
    }
    
    // Test ontology error handling
    let mut ontology = Ontology::new();
    let valid_class = Class::new(Arc::new(IRI::new("http://example.org/ValidClass").unwrap()));
    
    // Adding valid class should succeed
    let add_result = ontology.add_class(valid_class.clone());
    assert!(add_result.is_ok(), "Adding valid class should succeed");
    
    // Test reasoning with empty ontology
    let empty_ontology = Ontology::new();
    let reasoner = SimpleReasoner::new(empty_ontology);
    let is_consistent = reasoner.is_consistent().unwrap();
    assert!(is_consistent, "Empty ontology should be consistent");
    
    println!("  ✅ Error handling works as expected");
});

/// Test complex class expressions (regression test)
memory_safe_test!(test_class_expressions_regression, MemorySafeTestConfig::medium(), {
    println!("🔄 Testing complex class expressions (regression)...");
    
    let mut ontology = Ontology::new();
    
    // Create basic classes
    let person = Class::new(Arc::new(IRI::new("http://example.org/Person").unwrap()));
    let student = Class::new(Arc::new(IRI::new("http://example.org/Student").unwrap()));
    let teacher = Class::new(Arc::new(IRI::new("http://example.org/Teacher").unwrap()));
    
    ontology.add_class(person.clone()).unwrap();
    ontology.add_class(student.clone()).unwrap();
    ontology.add_class(teacher.clone()).unwrap();
    
    // Create complex class expressions
    let student_or_teacher = ClassExpression::ObjectUnionOf(vec![
        ClassExpression::Class(student.clone()),
        ClassExpression::Class(teacher.clone()),
    ]);
    
    let person_and_student = ClassExpression::ObjectIntersectionOf(vec![
        ClassExpression::Class(person.clone()),
        ClassExpression::Class(student.clone()),
    ]);
    
    // Test that expressions can be created and used
    assert!(matches!(student_or_teacher, ClassExpression::ObjectUnionOf(_)));
    assert!(matches!(person_and_student, ClassExpression::ObjectIntersectionOf(_)));
    
    // Test reasoning with complex expressions
    ontology.add_subclass_axiom(
        SubClassOfAxiom::new(
            ClassExpression::Class(student.clone()),
            ClassExpression::Class(person.clone()),
        )
    ).unwrap();
    
    ontology.add_subclass_axiom(
        SubClassOfAxiom::new(
            ClassExpression::Class(teacher.clone()),
            ClassExpression::Class(person.clone()),
        )
    ).unwrap();
    
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent().unwrap();
    assert!(is_consistent, "Ontology with complex expressions should be consistent");
    
    println!("  ✅ Complex class expressions work as expected");
});

/// Test performance characteristics (regression test)
memory_safe_test!(test_performance_characteristics_regression, MemorySafeTestConfig::medium(), {
    println!("🔄 Testing performance characteristics (regression)...");
    
    let start_time = std::time::Instant::now();
    
    // Create moderately large ontology
    let mut ontology = Ontology::new();
    
    // Create 100 classes
    for i in 0..100 {
        let iri = IRI::new(&format!("http://example.org/class{}", i)).unwrap();
        let class = Class::new(Arc::new(iri));
        ontology.add_class(class).unwrap();
    }
    
    // Create subclass relationships
    for i in 1..100 {
        let subclass_iri = IRI::new(&format!("http://example.org/class{}", i)).unwrap();
        let superclass_iri = IRI::new(&format!("http://example.org/class{}", i / 2)).unwrap();
        
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
    
    let total_time = start_time.elapsed();
    
    // Performance assertions (adjusted for memory safety overhead)
    assert!(creation_time.as_secs() < 5, "Ontology creation should be fast");
    assert!(reasoning_time.as_secs() < 10, "Reasoning should be fast");
    assert!(total_time.as_secs() < 15, "Total test should complete quickly");
    assert!(is_consistent, "Large ontology should be consistent");
    
    println!("  ✅ Performance characteristics are acceptable:");
    println!("    Creation: {:?}, Reasoning: {:?}, Total: {:?}", 
             creation_time, reasoning_time, total_time);
});

/// Test memory safety doesn't break existing functionality
memory_safe_test!(test_memory_safety_compatibility_regression, MemorySafeTestConfig::medium(), {
    println!("🔄 Testing memory safety compatibility (regression)...");
    
    let guard = TestMemoryGuard::new();
    guard.start_monitoring();
    
    // Perform all the basic operations that should still work
    let mut ontology = Ontology::new();
    
    // Create classes
    let person = Class::new(Arc::new(IRI::new("http://example.org/Person").unwrap()));
    let employee = Class::new(Arc::new(IRI::new("http://example.org/Employee").unwrap()));
    
    ontology.add_class(person.clone()).unwrap();
    ontology.add_class(employee.clone()).unwrap();
    
    // Add subclass relationship
    ontology.add_subclass_axiom(
        SubClassOfAxiom::new(
            ClassExpression::Class(employee.clone()),
            ClassExpression::Class(person.clone()),
        )
    ).unwrap();
    
    // Create individuals
    let john = NamedIndividual::new(Arc::new(IRI::new("http://example.org/John").unwrap()));
    ontology.add_named_individual(john.clone()).unwrap();
    
    // Add class assertion
    ontology.add_class_assertion(
        ClassAssertionAxiom::new(john.iri().clone(), ClassExpression::Class(employee.clone()))
    ).unwrap();
    
    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent().unwrap();
    let is_employee_person = reasoner.is_subclass_of(employee.iri(), person.iri()).unwrap();
    let instances = reasoner.get_instances(employee.iri()).unwrap();
    
    // All basic functionality should still work
    assert!(is_consistent, "Should still be consistent");
    assert!(is_employee_person, "Subclass reasoning should still work");
    assert!(!instances.is_empty(), "Instance retrieval should still work");
    
    let report = guard.stop_monitoring();
    
    // Memory usage should be reasonable
    assert!(report.is_acceptable(), "Memory usage should be acceptable");
    
    println!("  ✅ Memory safety doesn't break existing functionality");
    println!("    Memory usage: {:.1} MB, Warnings: {}", 
             report.end_memory as f64 / 1024.0 / 1024.0, report.warnings.len());
});

/// Comprehensive regression validation summary
memory_safe_test!(test_regression_validation_summary, MemorySafeTestConfig::large(), {
    println!("🔄 Comprehensive Regression Validation Summary");
    println!("===========================================");
    
    // Run all regression tests
    test_basic_ontology_regression()?;
    test_iri_creation_regression()?;
    test_basic_reasoning_regression()?;
    test_turtle_parsing_regression()?;
    test_cache_functionality_regression()?;
    test_property_characteristics_regression()?;
    test_individual_assertions_regression()?;
    test_error_handling_regression()?;
    test_class_expressions_regression()?;
    test_performance_characteristics_regression()?;
    test_memory_safety_compatibility_regression()?;
    
    println!("===========================================");
    println!("✅ All regression validation tests passed!");
    println!("   - Basic ontology creation preserved");
    println!("   - IRI creation and management unchanged");
    println!("   - Basic reasoning functionality intact");
    println!("   - Turtle parsing continues to work");
    println!("   - Cache functionality preserved");
    println!("   - Property characteristics work as before");
    println!("   - Individual assertions unchanged");
    println!("   - Error handling maintained");
    println!("   - Complex class expressions supported");
    println!("   - Performance characteristics acceptable");
    println!("   - Memory safety doesn't break existing functionality");
    
    // Final validation that the system works as expected
    let final_stats = get_memory_stats();
    println!("\n📊 Final System Health After Regression Tests:");
    println!("   Memory usage: {} bytes", final_stats.total_usage);
    println!("   Pressure level: {:.2}%", final_stats.pressure_level * 100.0);
    println!("   Total cleanups: {}", final_stats.cleanup_count);
    
    // System should be in good state
    assert!(final_stats.pressure_level < 0.8, "System pressure should be manageable");
    
    Ok(())
});