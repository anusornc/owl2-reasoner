"""
Evaluator for OWL2 Reasoner optimization example
"""

import asyncio
import json
import os
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Dict, Any, List

import numpy as np

from openevolve.evaluation_result import EvaluationResult


def evaluate(program_path: str) -> EvaluationResult:
    """
    Evaluate an OWL2 reasoning algorithm implementation.

    Tests the algorithm on various reasoning tasks to measure:
    - Correctness (logical correctness)
    - Performance (speed)
    - Scalability (handling larger ontologies)
    - Memory efficiency
    """
    import subprocess
import tempfile
import time

def evaluate(program_path: str) -> EvaluationResult:
    """
    Evaluate an OWL2 reasoning algorithm implementation.

    Tests the algorithm on various reasoning tasks to measure:
    - Correctness (logical correctness)
    - Performance (speed)
    - Scalability (handling larger ontologies)
    - Memory efficiency
    """
    try:
        # Create a temporary Rust project
        with tempfile.TemporaryDirectory() as temp_dir:
            project_dir = Path(temp_dir) / "owl2_reasoner_test"

            # Initialize Cargo project
            result = subprocess.run(
                ["cargo", "init", "--name", "owl2_reasoner_test", "--lib", str(project_dir)],
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

            # Create main.rs with comprehensive benchmark code
            main_content = """
use owl2_reasoner_test::{run_reasoning_benchmark, BenchmarkTestCase};

fn main() {
    // Create test cases that simulate realistic OWL2 reasoning scenarios

    // Test Case 1: Basic hierarchy reasoning
    let basic_cases = vec![
        BenchmarkTestCase {
            sub_class: "lung_cancer".to_string(),
            super_class: "disease".to_string(),
            expected_result: true,
        },
        BenchmarkTestCase {
            sub_class: "cancer".to_string(),
            super_class: "disease".to_string(),
            expected_result: true,
        },
        BenchmarkTestCase {
            sub_class: "disease".to_string(),
            super_class: "lung_cancer".to_string(),
            expected_result: false,
        },
    ];

    // Test Case 2: Deep hierarchy reasoning
    let deep_hierarchy_cases = vec![
        BenchmarkTestCase {
            sub_class: "specific_protein".to_string(),
            super_class: "entity".to_string(),
            expected_result: true,
        },
        BenchmarkTestCase {
            sub_class: "protein".to_string(),
            super_class: "entity".to_string(),
            expected_result: true,
        },
        BenchmarkTestCase {
            sub_class: "protein".to_string(),
            super_class: "gene".to_string(),
            expected_result: false,
        },
    ];

    // Test Case 3: Large hierarchy reasoning (scalability test)
    let mut large_cases = Vec::new();
    for i in 0..100 {
        large_cases.push(BenchmarkTestCase {
            sub_class: format!("class_{}", i),
            super_class: format!("class_{}", (i + 1) % 100),
            expected_result: i % 10 != 0, // Some true, some false
        });
    }

    // Test Case 4: Realistic biomedical ontology simulation
    let biomedical_cases = vec![
        // Disease hierarchy
        BenchmarkTestCase {
            sub_class: "non_small_cell_lung_carcinoma".to_string(),
            super_class: "lung_cancer".to_string(),
            expected_result: true,
        },
        BenchmarkTestCase {
            sub_class: "lung_cancer".to_string(),
            super_class: "carcinoma".to_string(),
            expected_result: true,
        },
        BenchmarkTestCase {
            sub_class: "carcinoma".to_string(),
            super_class: "cancer".to_string(),
            expected_result: true,
        },
        BenchmarkTestCase {
            sub_class: "cancer".to_string(),
            super_class: "disease".to_string(),
            expected_result: true,
        },
        // Gene hierarchy
        BenchmarkTestCase {
            sub_class: "egfr".to_string(),
            super_class: "receptor_tyrosine_kinase".to_string(),
            expected_result: true,
        },
        BenchmarkTestCase {
            sub_class: "receptor_tyrosine_kinase".to_string(),
            super_class: "tyrosine_kinase".to_string(),
            expected_result: true,
        },
        BenchmarkTestCase {
            sub_class: "tyrosine_kinase".to_string(),
            super_class: "kinase".to_string(),
            expected_result: true,
        },
        BenchmarkTestCase {
            sub_class: "kinase".to_string(),
            super_class: "enzyme".to_string(),
            expected_result: true,
        },
        // Cross-domain relationships (should be false)
        BenchmarkTestCase {
            sub_class: "egfr".to_string(),
            super_class: "lung_cancer".to_string(),
            expected_result: false,
        },
    ];

    // Combine all test cases
    let all_test_cases = vec![
        ("basic", basic_cases),
        ("deep_hierarchy", deep_hierarchy_cases),
        ("large_scale", large_cases),
        ("biomedical", biomedical_cases),
    ];

    let mut all_results = Vec::new();
    let mut all_times = Vec::new();
    let mut all_correctness = Vec::new();

    for (test_name, cases) in all_test_cases {
        let results = run_reasoning_benchmark(cases);
        all_results.push((test_name, results.clone()));

        all_times.extend(results.times);
        all_correctness.extend(results.correctness);
    }

    // Calculate overall metrics
    let total_correct = all_correctness.iter().filter(|&&c| c).count();
    let correctness_score = total_correct as f64 / all_correctness.len() as f64;

    // Calculate performance score (inverse of average time)
    let avg_time: f64 = if all_times.is_empty() {
        0.0
    } else {
        all_times.iter().sum::<f64>() / all_times.len() as f64
    };

    // Normalize performance score (lower time = higher score)
    let performance_score = 1_000_000.0 / (1_000_000.0 + avg_time);

    // Calculate scalability score from large scale test
    let scalability_score = all_results.iter()
        .find(|(name, _)| *name == "large_scale")
        .map(|(_, results)| results.scalability_score)
        .unwrap_or(0.0);

    // Calculate memory efficiency score
    let memory_efficiency_score = all_results.iter()
        .find(|(name, _)| *name == "large_scale")
        .map(|(_, results)| results.memory_efficiency_score)
        .unwrap_or(0.0);

    // Overall score calculation
    // Correctness is mandatory (weight: 50%)
    // Performance is important (weight: 30%)
    // Scalability and memory efficiency (weight: 20%)
    let overall_score = if correctness_score < 0.9 {
        0.0 // Must be at least 90% correct
    } else {
        0.5 * correctness_score +
        0.3 * performance_score +
        0.1 * scalability_score +
        0.1 * memory_efficiency_score
    };

    // Output results as JSON
    println!("{{");
    println!("  \\"correctness_score\\": {},", correctness_score);
    println!("  \\"performance_score\\": {},", performance_score);
    println!("  \\"scalability_score\\": {},", scalability_score);
    println!("  \\"memory_efficiency_score\\": {},", memory_efficiency_score);
    println!("  \\"overall_score\\": {},", overall_score);
    println!("  \\"avg_time_ns\\": {},", avg_time);
    println!("  \\"total_tests\\": {},", all_correctness.len());
    println!("  \\"correct_tests\\": {}", total_correct);
    println!("}}");
}
"""
            main_path = project_dir / "src" / "main.rs"
            with open(main_path, "w") as f:
                f.write(main_content)

            # Add dependencies to Cargo.toml
            cargo_toml_path = project_dir / "Cargo.toml"
            with open(cargo_toml_path, "r") as f:
                cargo_content = f.read()

            # Add any additional dependencies if needed
            cargo_content = cargo_content.replace(
                "[dependencies]\n",
                "[dependencies]\n# For optimized performance\ncrossbeam = \"0.8\"\nrayon = \"1.8\"\n"
            )

            with open(cargo_toml_path, "w") as f:
                f.write(cargo_content)

            # Build the project
            build_result = subprocess.run(
                ["cargo", "build", "--release"],
                cwd=project_dir,
                capture_output=True,
                text=True,
                timeout=60,
            )

            if build_result.returncode != 0:
                # Extract compilation errors
                return EvaluationResult(
                    metrics={
                        "score": 0.0,
                        "compile_success": 0.0,
                        "correctness_score": 0.0,
                        "performance_score": 0.0,
                        "scalability_score": 0.0,
                        "memory_efficiency_score": 0.0,
                    },
                    artifacts={
                        "error": "Compilation failed",
                        "stderr": build_result.stderr,
                        "stdout": build_result.stdout,
                    },
                )

            # Run the benchmark
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
                        "correctness_score": 0.0,
                        "performance_score": 0.0,
                        "scalability_score": 0.0,
                        "memory_efficiency_score": 0.0,
                    },
                    artifacts={"error": "Runtime error", "stderr": run_result.stderr},
                )

            # Parse JSON output
            try:
                # Find JSON in output (between first { and last })
                output = run_result.stdout
                start = output.find("{")
                end = output.rfind("}") + 1
                json_str = output[start:end]

                results = json.loads(json_str)

                return EvaluationResult(
                    metrics={
                        "score": results["overall_score"],
                        "compile_success": 1.0,
                        "correctness_score": results["correctness_score"],
                        "performance_score": results["performance_score"],
                        "scalability_score": results["scalability_score"],
                        "memory_efficiency_score": results["memory_efficiency_score"],
                        "avg_time_ns": results["avg_time_ns"],
                    },
                    artifacts={
                        "total_tests": results["total_tests"],
                        "correct_tests": results["correct_tests"],
                        "build_output": build_result.stdout,
                        "run_output": output,
                    },
                )

            except (json.JSONDecodeError, KeyError) as e:
                return EvaluationResult(
                    metrics={
                        "score": 0.0,
                        "compile_success": 1.0,
                        "correctness_score": 0.0,
                        "performance_score": 0.0,
                        "scalability_score": 0.0,
                        "memory_efficiency_score": 0.0,
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
                "correctness_score": 0.0,
                "performance_score": 0.0,
                "scalability_score": 0.0,
                "memory_efficiency_score": 0.0,
            },
            artifacts={"error": "Timeout during evaluation"},
        )
    except Exception as e:
        return EvaluationResult(
            metrics={
                "score": 0.0,
                "compile_success": 0.0,
                "correctness_score": 0.0,
                "performance_score": 0.0,
                "scalability_score": 0.0,
                "memory_efficiency_score": 0.0,
            },
            artifacts={"error": str(e), "type": "evaluation_error"},
        )


# For testing
if __name__ == "__main__":
    import sys

    if len(sys.argv) > 1:
        result = asyncio.run(evaluate(sys.argv[1]))
        print(f"Overall Score: {result.metrics['score']:.4f}")
        print(f"Correctness: {result.metrics['correctness_score']:.4f}")
        print(f"Performance: {result.metrics['performance_score']:.4f}")
        print(f"Scalability: {result.metrics['scalability_score']:.4f}")
        print(f"Memory Efficiency: {result.metrics['memory_efficiency_score']:.4f}")
        print(f"Avg Time (ns): {result.metrics['avg_time_ns']:.2f}")