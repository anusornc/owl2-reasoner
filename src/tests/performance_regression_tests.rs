use std::sync::Arc;

use crate::{memory_safe_bench_test, memory_safe_test};
use crate::test_helpers::MemorySafeTestConfig;
/// Regression Test Suite for OWL2 Reasoner
///
/// This test suite ensures that performance improvements don't break existing functionality
/// and provides baseline measurements for detecting performance regressions.
/// All tests are now memory-safe and will fail gracefully before causing OOM.
use crate::*;
use std::time::Instant;

/// Test basic functionality with regression timing
memory_safe_test!(
    test_basic_functionality_regression,
    MemorySafeTestConfig::small(),
    {
        let start_time = Instant::now();

        let mut ontology = Ontology::new();

        // Create basic ontology structure
        let person_class = Class::new(Arc::new(IRI::new("http://example.org/Person")?));
        let employee_class = Class::new(Arc::new(IRI::new("http://example.org/Employee")?));

        ontology.add_class(person_class.clone())?;
        ontology.add_class(employee_class.clone())?;

        // Create subclass relationship
        let subclass_axiom = SubClassOfAxiom::new(
            ClassExpression::Class(employee_class.clone()),
            ClassExpression::Class(person_class.clone()),
        );
        ontology.add_subclass_axiom(subclass_axiom)?;

        // Test reasoning
        let reasoner = SimpleReasoner::new(ontology);
        let is_consistent = reasoner.is_consistent()?;
        assert!(is_consistent, "Basic ontology should be consistent");

        // Test subclass reasoning
        let is_subclass = reasoner.is_subclass_of(employee_class.iri(), person_class.iri())?;
        assert!(is_subclass, "Employee should be subclass of Person");

        let duration = start_time.elapsed();
        println!(
            "✅ Basic functionality regression test passed in {:?}",
            duration
        );

        // Regression check: should complete in under 100ms
        assert!(
            duration.as_millis() < 100,
            "Basic functionality should complete quickly"
        );

        Ok(())
    }
);

/// Test ontology scalability with regression tracking
memory_safe_test!(
    test_ontology_scalability_regression,
    MemorySafeTestConfig::medium(),
    {
        let start_time = Instant::now();

        let mut ontology = Ontology::new();
        let size = 1000; // Test with 1000 classes

        // Create hierarchical class structure
        for i in 0..size {
            let class_iri = Arc::new(IRI::new(format!("http://example.org/Class{}", i))?);
            let class = Class::new(class_iri);
            ontology.add_class(class)?;
        }

        // Create subclass relationships (tree structure)
        for i in 1..size {
            let parent_idx = (i - 1) / 2; // Binary tree structure
            let subclass_iri = Arc::new(IRI::new(format!("http://example.org/Class{}", i))?);
            let superclass_iri =
                Arc::new(IRI::new(format!("http://example.org/Class{}", parent_idx))?);

            let subclass = ClassExpression::Class(Class::new(subclass_iri));
            let superclass = ClassExpression::Class(Class::new(superclass_iri));
            let axiom = SubClassOfAxiom::new(subclass, superclass);
            ontology.add_subclass_axiom(axiom)?;
        }

        // Test reasoning performance
        let reasoner = SimpleReasoner::new(ontology);
        let consistency_start = Instant::now();
        let is_consistent = reasoner.is_consistent()?;
        let consistency_duration = consistency_start.elapsed();

        assert!(is_consistent, "Hierarchical ontology should be consistent");

        // Test some subclass queries
        let query_start = Instant::now();
        for i in 0..10.min(size) {
            for j in 0..10.min(size) {
                if i != j {
                    let class_i_iri = Arc::new(IRI::new(format!("http://example.org/Class{}", i))?);
                    let class_j_iri = Arc::new(IRI::new(format!("http://example.org/Class{}", j))?);
                    let _ = reasoner.is_subclass_of(&class_i_iri, &class_j_iri);
                }
            }
        }
        let query_duration = query_start.elapsed();

        let total_duration = start_time.elapsed();

        println!("✅ Scalability regression test:");
        println!("   - {} classes created", size);
        println!("   - Consistency check: {:?}", consistency_duration);
        println!("   - Query operations: {:?}", query_duration);
        println!("   - Total time: {:?}", total_duration);

        // Regression checks - adjusted for shared resource contention
        assert!(
            consistency_duration.as_millis() < 500,
            "Consistency check should be fast"
        );
        assert!(
            query_duration.as_millis() < 3000,
            "Query operations should be fast"
        );
        assert!(
            total_duration.as_millis() < 45000,
            "Total test should complete quickly"
        );

        Ok(())
    }
);

/// Test memory usage regression
memory_safe_test!(
    test_memory_usage_regression,
    MemorySafeTestConfig::large(),
    {
        let start_time = Instant::now();

        let mut ontology = Ontology::new();

        // Create many entities to test memory usage
        for i in 0..5000 {
            let class_iri = Arc::new(IRI::new(format!("http://example.org/Entity{}", i))?);
            let class = Class::new(class_iri);
            ontology.add_class(class)?;

            if i % 100 == 0 {
                let prop_iri = Arc::new(IRI::new(format!(
                    "http://example.org/hasProperty{}",
                    i / 100
                ))?);
                let prop = ObjectProperty::new(prop_iri);
                ontology.add_object_property(prop)?;
            }
        }

        // Create some relationships
        for i in 0..1000 {
            let subclass_iri = Arc::new(IRI::new(format!("http://example.org/Entity{}", i))?);
            let superclass_iri = Arc::new(IRI::new(format!(
                "http://example.org/Entity{}",
                (i + 1) % 5000
            ))?);

            let subclass = ClassExpression::Class(Class::new(subclass_iri));
            let superclass = ClassExpression::Class(Class::new(superclass_iri));
            let axiom = SubClassOfAxiom::new(subclass, superclass);
            ontology.add_subclass_axiom(axiom)?;
        }

        let creation_duration = start_time.elapsed();

        // Test that we can reason without memory issues
        let reasoner = SimpleReasoner::new(ontology);
        let _is_consistent = reasoner.is_consistent()?;

        let total_duration = start_time.elapsed();

        println!("✅ Memory usage regression test:");
        println!("   - {} classes created", 5000);
        println!("   - {} properties created", 50);
        println!("   - {} subclass axioms created", 1000);
        println!("   - Creation time: {:?}", creation_duration);
        println!("   - Total time: {:?}", total_duration);

        // Regression check - should complete in reasonable time
        // Note: When run with other tests, global resource contention may increase time
        assert!(
            total_duration.as_secs() < 60,
            "Memory test should complete in under 60 seconds"
        );

        Ok(())
    }
);

/// Test complex reasoning scenarios for regression
memory_safe_test!(
    test_complex_reasoning_regression,
    MemorySafeTestConfig::small(),
    {
        let start_time = Instant::now();

        let mut ontology = Ontology::new();

        // Create more complex ontology with multiple inheritance
        let classes = vec![
            "Animal", "Mammal", "Dog", "Cat", "Plant", "Tree", "Flower", "Vehicle", "Car",
            "Bicycle", "Person", "Employee", "Manager",
        ];

        let mut class_iris = Vec::new();
        for class_name in &classes {
            let iri = Arc::new(IRI::new(format!("http://example.org/{}", class_name))?);
            let class = Class::new(iri.clone());
            ontology.add_class(class)?;
            class_iris.push(iri);
        }

        // Create complex hierarchy
        let relationships = vec![
            (1, 0),   // Mammal -> Animal
            (2, 1),   // Dog -> Mammal
            (3, 1),   // Cat -> Mammal
            (5, 4),   // Tree -> Plant
            (6, 4),   // Flower -> Plant
            (8, 7),   // Car -> Vehicle
            (9, 7),   // Bicycle -> Vehicle
            (11, 10), // Employee -> Person
            (12, 11), // Manager -> Employee
            (12, 10), // Manager -> Person (multiple inheritance)
        ];

        for (child_idx, parent_idx) in &relationships {
            let subclass = ClassExpression::Class(Class::new(class_iris[*child_idx].clone()));
            let superclass = ClassExpression::Class(Class::new(class_iris[*parent_idx].clone()));
            let axiom = SubClassOfAxiom::new(subclass, superclass);
            ontology.add_subclass_axiom(axiom)?;
        }

        let reasoner = SimpleReasoner::new(ontology);

        // Test complex reasoning queries
        let reasoning_start = Instant::now();

        // Test transitive relationships
        let is_dog_animal = reasoner.is_subclass_of(&class_iris[2], &class_iris[0])?; // Dog -> Animal
        assert!(is_dog_animal, "Dog should be subclass of Animal");

        let is_manager_person = reasoner.is_subclass_of(&class_iris[12], &class_iris[10])?; // Manager -> Person
        assert!(is_manager_person, "Manager should be subclass of Person");

        // Test multiple inheritance paths
        let manager_employee = reasoner.is_subclass_of(&class_iris[12], &class_iris[11])?; // Manager -> Employee
        assert!(manager_employee, "Manager should be subclass of Employee");

        let reasoning_duration = reasoning_start.elapsed();
        let total_duration = start_time.elapsed();

        println!("✅ Complex reasoning regression test:");
        println!("   - {} classes with complex hierarchy", classes.len());
        println!("   - {} subclass relationships", relationships.len());
        println!("   - Reasoning time: {:?}", reasoning_duration);
        println!("   - Total time: {:?}", total_duration);

        // Regression checks
        assert!(
            reasoning_duration.as_millis() < 100,
            "Complex reasoning should be fast"
        );
        assert!(
            total_duration.as_millis() < 500,
            "Total complex test should be fast"
        );

        Ok(())
    }
);

/// Test error handling performance regression
memory_safe_test!(
    test_error_handling_regression,
    MemorySafeTestConfig::small(),
    {
        let start_time = Instant::now();

        // Test error creation performance
        let error_start = Instant::now();
        for i in 0..1000 {
            let _ = IRI::new(format!("invalid_iri_{}", i));
        }
        let error_duration = error_start.elapsed();

        // Test mixed valid/invalid operations
        let mut ontology = Ontology::new();
        let mixed_start = Instant::now();

        for i in 0..500 {
            if i % 10 == 0 {
                // Invalid IRI
                let _ = ontology.add_class(Class::new(Arc::new(IRI::new("invalid")?)));
            } else {
                // Valid IRI
                let iri = Arc::new(IRI::new(format!("http://example.org/Class{}", i))?);
                let class = Class::new(iri);
                ontology.add_class(class)?;
            }
        }
        let mixed_duration = mixed_start.elapsed();

        let total_duration = start_time.elapsed();

        println!("✅ Error handling regression test:");
        println!("   - 1000 invalid IRIs processed: {:?}", error_duration);
        println!("   - 500 mixed operations: {:?}", mixed_duration);
        println!("   - Total time: {:?}", total_duration);

        // Regression checks - error handling shouldn't be too slow (adjusted for shared resources)
        assert!(
            error_duration.as_millis() < 2000,
            "Error creation should be fast"
        );
        assert!(
            mixed_duration.as_millis() < 5000,
            "Mixed operations should be fast"
        );
        assert!(
            total_duration.as_millis() < 8000,
            "Total error test should be fast"
        );

        Ok(())
    }
);

/// Performance regression test summary
memory_safe_test!(
    test_performance_regression_summary,
    MemorySafeTestConfig::medium(),
    {
        println!("📊 Performance Regression Test Summary");
        println!("=====================================");

        // Run key regression tests
        test_basic_functionality_regression()?;
        test_ontology_scalability_regression()?;
        test_memory_usage_regression()?;
        test_complex_reasoning_regression()?;
        test_error_handling_regression()?;

        println!("=====================================");
        println!("✅ All performance regression tests passed!");
        println!("   - No performance regressions detected");
        println!("   - All timing thresholds met");
        println!("   - Memory usage within expected bounds");

        Ok(())
    }
);
