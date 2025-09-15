"""
Enhanced evaluator for OWL2 Reasoner optimization - Competitive Edition
Comprehensive testing with large ontologies and competitive benchmarks
"""

import json
import subprocess
import tempfile
import time
import psutil
import os
from pathlib import Path

from openevolve.evaluation_result import EvaluationResult


def evaluate(program_path: str) -> EvaluationResult:
    """
    Evaluate an OWL2 reasoning algorithm implementation.
    """
    try:
        # Create a temporary Rust project
        with tempfile.TemporaryDirectory() as temp_dir:
            project_dir = Path(temp_dir) / "owl2_test"

            # Initialize Cargo project
            result = subprocess.run(
                ["cargo", "init", "--name", "owl2_test", "--lib", str(project_dir)],
                capture_output=True,
                text=True,
            )

            if result.returncode != 0:
                return EvaluationResult(
                    metrics={"score": 0.0, "compile_success": 0.0},
                    artifacts={"error": "Failed to create Cargo project", "stderr": result.stderr},
                )

            # Copy the program to src/lib.rs
            lib_path = project_dir / "src" / "lib.rs"
            with open(program_path, "r") as src:
                lib_content = src.read()
            with open(lib_path, "w") as dst:
                dst.write(lib_content)

            # Add required dependencies to Cargo.toml
            cargo_toml_path = project_dir / "Cargo.toml"
            with open(cargo_toml_path, "r") as f:
                cargo_content = f.read()

            # Add psutil and serde dependencies if not present
            if "psutil" not in cargo_content:
                cargo_content = cargo_content.replace(
                    "[dependencies]\n",
                    "[dependencies]\npsutil = \"3.3\"\nserde = { version = \"1.0\", features = [\"derive\"] }\nserde_json = \"1.0\"\n"
                )

            with open(cargo_toml_path, "w") as f:
                f.write(cargo_content)

            # Create main.rs with comprehensive competitive testing
            main_content = """
use owl2_test::ReasoningEngine;
use std::time::Instant;
use std::collections::HashMap;

fn main() {
    let mut engine = ReasoningEngine::new();

    // === SCENARIO 1: Basic Hierarchy (4 nodes) ===
    engine.add_class("A".to_string());
    engine.add_class("B".to_string());
    engine.add_class("C".to_string());
    engine.add_class("D".to_string());
    engine.add_subclass_relation("A".to_string(), "B".to_string());
    engine.add_subclass_relation("B".to_string(), "C".to_string());
    engine.add_subclass_relation("C".to_string(), "D".to_string());

    // === SCENARIO 2: Deep Hierarchy (100 nodes) ===
    for i in 0..100 {
        let class_name = format!("Deep_{}", i);
        engine.add_class(class_name.clone());
        if i > 0 {
            let parent_name = format!("Deep_{}", i - 1);
            engine.add_subclass_relation(class_name, parent_name);
        }
    }

    // === SCENARIO 3: Wide Hierarchy (1000 nodes) ===
    let root = "Wide_Root".to_string();
    engine.add_class(root.clone());
    for i in 0..1000 {
        let class_name = format!("Wide_Child_{}", i);
        engine.add_class(class_name.clone());
        engine.add_subclass_relation(class_name, root.clone());
    }

    // === SCENARIO 4: Complex DAG (500 nodes) ===
    for i in 0..50 {
        for j in 0..10 {
            let class_name = format!("DAG_{}_{}", i, j);
            engine.add_class(class_name.clone());
            if i > 0 {
                let parent1 = format!("DAG_{}_{}", i - 1, j);
                let parent2 = format!("DAG_{}_{}", (i - 1) / 2, j);
                engine.add_subclass_relation(class_name.clone(), parent1);
                if i > 1 {
                    engine.add_subclass_relation(class_name, parent2);
                }
            }
        }
    }

    // === COMPREHENSIVE TEST SUITE ===
    let test_cases = vec![
        // Basic hierarchy tests
        ("A", "B", true, "basic_direct"),
        ("B", "C", true, "basic_direct"),
        ("A", "C", true, "basic_transitive"),
        ("A", "D", true, "basic_long_transitive"),
        ("C", "A", false, "basic_reverse"),

        // Deep hierarchy tests
        ("Deep_0", "Deep_50", true, "deep_transitive"),
        ("Deep_25", "Deep_75", true, "deep_transitive"),
        ("Deep_99", "Deep_0", false, "deep_reverse"),

        // Wide hierarchy tests
        ("Wide_Child_0", "Wide_Root", true, "wide_direct"),
        ("Wide_Child_500", "Wide_Root", true, "wide_direct"),
        ("Wide_Child_999", "Wide_Root", true, "wide_direct"),

        // DAG tests
        ("DAG_10_0", "DAG_5_0", true, "dag_direct"),
        ("DAG_20_5", "DAG_10_5", true, "dag_direct"),
        ("DAG_30_0", "DAG_15_0", true, "dag_transitive"),

        // Edge cases
        ("NonExistent", "A", false, "nonexistent_sub"),
        ("A", "NonExistent", false, "nonexistent_sup"),
        ("A", "A", true, "self_reflexive"),
    ];

    // === PERFORMANCE TESTING ===
    let mut total_time_ns = 0u128;
    let mut correct_count = 0;
    let mut test_results = Vec::new();

    // Warm-up phase
    for _ in 0..10 {
        let _ = engine.is_subclass_of("A", "B");
    }

    // Main testing phase
    for (sub, sup, expected, category) in &test_cases {
        let start = Instant::now();
        let result = engine.is_subclass_of(sub, sup);
        let elapsed = start.elapsed().as_nanos();

        total_time_ns += elapsed;

        let is_correct = result == *expected;
        if is_correct {
            correct_count += 1;
        }

        test_results.push((
            sub.to_string(), sup.to_string(),
            expected.to_string(), result.to_string(),
            is_correct, elapsed, category.to_string()
        ));
    }

    // === MEMORY USAGE MEASUREMENT ===
    let process = psutil::Process::current();
    let memory_usage_bytes = process.memory_info().rss();

    // === SCALABILITY TEST ===
    let large_ontology_start = Instant::now();

    // Add large ontology for scalability testing
    for i in 0..10000 {
        let class_name = format!("Large_{}", i);
        engine.add_class(class_name.clone());
        if i > 0 && i % 100 == 0 {
            let parent_name = format!("Large_{}", i - 100);
            engine.add_subclass_relation(class_name, parent_name);
        }
    }

    // Perform scalability test queries
    for i in (0..10000).step_by(1000) {
        let _ = engine.is_subclass_of(&format!("Large_{}", i), &format!("Large_{}", i + 500));
    }

    let large_ontology_time = large_ontology_start.elapsed().as_nanos();

    // === CALCULATE METRICS ===
    let total_tests = test_cases.len() as f64;
    let correctness = correct_count as f64 / total_tests;

    let avg_time_ns = total_time_ns / test_cases.len() as u128;
    let avg_time_ms = avg_time_ns as f64 / 1_000_000.0;

    // Performance scoring (higher is better)
    let performance_score = if avg_time_ns > 0 {
        // Target: < 0.1ms = 1000 points, scale down from there
        (100_000.0 / avg_time_ns as f64).min(1000.0)
    } else {
        1000.0
    };

    // Memory efficiency scoring (lower is better)
    let memory_per_entity = if test_cases.len() > 0 {
        memory_usage_bytes / test_cases.len()
    } else {
        memory_usage_bytes
    };
    let memory_efficiency_score = if memory_per_entity > 0 {
        // Target: < 200 bytes/entity = 100 points, scale down from there
        (20000.0 / memory_per_entity as f64).min(100.0)
    } else {
        100.0
    };

    // Scalability scoring (lower time is better)
    let scalability_score = if large_ontology_time > 0 {
        // Target: < 100ms for large ontology = 100 points
        (100_000_000.0 / large_ontology_time as f64).min(100.0)
    } else {
        100.0
    };

    // Overall competitive score
    let competitive_score = if correctness >= 0.99 {
        0.35 * correctness +
        0.30 * (performance_score / 1000.0) +
        0.20 * (memory_efficiency_score / 100.0) +
        0.15 * (scalability_score / 100.0)
    } else {
        0.0 // Must be 99% correct
    };

    // === OUTPUT RESULTS ===
    println!("{{\
        \\"correctness\\": {:.4}, \
        \\"performance_score\\": {:.2}, \
        \\"memory_efficiency_score\\": {:.2}, \
        \\"scalability_score\\": {:.2}, \
        \\"competitive_score\\": {:.4}, \
        \\"avg_time_ns\\": {}, \
        \\"avg_time_ms\\": {:.6}, \
        \\"memory_usage_bytes\\": {}, \
        \\"memory_per_entity\\": {}, \
        \\"large_ontology_time_ns\\": {}, \
        \\"correct_tests\\": {}, \
        \\"total_tests\\": {}, \
        \\"test_results\\": {}\
    }}",
        correctness,
        performance_score,
        memory_efficiency_score,
        scalability_score,
        competitive_score,
        avg_time_ns,
        avg_time_ms,
        memory_usage_bytes,
        memory_per_entity,
        large_ontology_time,
        correct_count,
        test_cases.len(),
        serde_json::to_string(&test_results).unwrap_or_else(|_| "[]".to_string())
    );
}
"""
            main_path = project_dir / "src" / "main.rs"
            with open(main_path, "w") as f:
                f.write(main_content)

            # Build the project
            build_result = subprocess.run(
                ["cargo", "build", "--release"],
                cwd=project_dir,
                capture_output=True,
                text=True,
                timeout=60,
            )

            if build_result.returncode != 0:
                return EvaluationResult(
                    metrics={
                        "score": 0.0,
                        "compile_success": 0.0,
                        "correctness": 0.0,
                        "performance": 0.0,
                        "memory_efficiency": 0.0,
                        "scalability": 0.0,
                        "avg_time_ns": 0,
                        "avg_time_ms": 0.0,
                        "memory_usage_bytes": 0,
                        "memory_per_entity": 0,
                        "large_ontology_time_ns": 0,
                    },
                    artifacts={
                        "error": "Compilation failed",
                        "stderr": build_result.stderr,
                        "stdout": build_result.stdout,
                    },
                )

            # Run the test
            run_result = subprocess.run(
                ["cargo", "run", "--release"],
                cwd=project_dir,
                capture_output=True,
                text=True,
                timeout=30,
            )

            if run_result.returncode != 0:
                return EvaluationResult(
                    metrics={
                        "score": 0.0,
                        "compile_success": 1.0,
                        "correctness": 0.0,
                        "performance": 0.0,
                        "memory_efficiency": 0.0,
                        "scalability": 0.0,
                        "avg_time_ns": 0,
                        "avg_time_ms": 0.0,
                        "memory_usage_bytes": 0,
                        "memory_per_entity": 0,
                        "large_ontology_time_ns": 0,
                    },
                    artifacts={"error": "Runtime error", "stderr": run_result.stderr},
                )

            # Parse JSON output
            try:
                output = run_result.stdout
                start = output.find("{")
                end = output.rfind("}") + 1
                json_str = output[start:end]

                results = json.loads(json_str)

                return EvaluationResult(
                    metrics={
                        "score": results["competitive_score"],
                        "compile_success": 1.0,
                        "correctness": results["correctness"],
                        "performance": results["performance_score"],
                        "memory_efficiency": results["memory_efficiency_score"],
                        "scalability": results["scalability_score"],
                        "avg_time_ns": results["avg_time_ns"],
                        "avg_time_ms": results["avg_time_ms"],
                        "memory_usage_bytes": results["memory_usage_bytes"],
                        "memory_per_entity": results["memory_per_entity"],
                        "large_ontology_time_ns": results["large_ontology_time_ns"],
                    },
                    artifacts={
                        "correct_tests": results["correct_tests"],
                        "total_tests": results["total_tests"],
                        "test_results": results["test_results"],
                        "build_output": build_result.stdout,
                    },
                )

            except (json.JSONDecodeError, KeyError) as e:
                return EvaluationResult(
                    metrics={
                        "score": 0.0,
                        "compile_success": 1.0,
                        "correctness": 0.0,
                        "performance": 0.0,
                        "memory_efficiency": 0.0,
                        "scalability": 0.0,
                        "avg_time_ns": 0,
                        "avg_time_ms": 0.0,
                        "memory_usage_bytes": 0,
                        "memory_per_entity": 0,
                        "large_ontology_time_ns": 0,
                    },
                    artifacts={
                        "error": f"Failed to parse results: {str(e)}",
                        "stdout": run_result.stdout,
                    },
                )

    except subprocess.TimeoutExpired:
        return EvaluationResult(
            metrics={
                "score": 0.0,
                "compile_success": 0.0,
                "correctness": 0.0,
                "performance": 0.0,
                "memory_efficiency": 0.0,
                "scalability": 0.0,
                "avg_time_ns": 0,
                "avg_time_ms": 0.0,
                "memory_usage_bytes": 0,
                "memory_per_entity": 0,
                "large_ontology_time_ns": 0,
            },
            artifacts={"error": "Timeout during evaluation"},
        )
    except Exception as e:
        return EvaluationResult(
            metrics={
                "score": 0.0,
                "compile_success": 0.0,
                "correctness": 0.0,
                "performance": 0.0,
                "memory_efficiency": 0.0,
                "scalability": 0.0,
                "avg_time_ns": 0,
                "avg_time_ms": 0.0,
                "memory_usage_bytes": 0,
                "memory_per_entity": 0,
                "large_ontology_time_ns": 0,
            },
            artifacts={"error": str(e)},
        )


# For testing
if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1:
        result = evaluate(sys.argv[1])
        print(f"Score: {result.metrics['score']:.4f}")
        print(f"Correctness: {result.metrics['correctness']:.4f}")
        print(f"Performance: {result.metrics['performance']:.4f}")