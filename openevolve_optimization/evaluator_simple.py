"""
Simple evaluator for OWL2 Reasoner optimization example
Based on the working rust_adaptive_sort evaluator
"""

import json
import subprocess
import tempfile
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

            # Create main.rs with a simple test
            main_content = """
use owl2_test::ReasoningEngine;

fn main() {
    // Create a simple test case
    let mut engine = ReasoningEngine::new();

    // Add some test data
    engine.add_class("A".to_string());
    engine.add_class("B".to_string());
    engine.add_class("C".to_string());
    engine.add_subclass_relation("A".to_string(), "B".to_string());
    engine.add_subclass_relation("B".to_string(), "C".to_string());

    // Test basic functionality
    let test_results = vec![
        ("A", "B", true),
        ("B", "C", true),
        ("A", "C", true),  // Transitive test
        ("C", "A", false),  // Reverse test
    ];

    let mut total_time = 0;
    let mut correct_count = 0;

    for (sub, sup, expected) in &test_results {
        let start = std::time::Instant::now();
        let result = engine.is_subclass_of(sub, sup);
        let elapsed = start.elapsed();
        total_time += elapsed.as_nanos();

        if result == *expected {
            correct_count += 1;
        }
    }

    let avg_time = total_time / test_results.len() as u128;
    let correctness = correct_count as f64 / test_results.len() as f64;

    // Calculate scores
    let performance_score = if avg_time > 0 {
        1_000_000.0 / avg_time as f64
    } else {
        0.0
    };

    let overall_score = if correctness > 0.8 {
        0.6 * correctness + 0.4 * performance_score.min(1.0)
    } else {
        0.0 // Must be mostly correct
    };

    println!("{{\\"correctness\\": {}, \\"performance_score\\": {}, \\"overall_score\\": {}, \\"avg_time_ns\\": {}, \\"correct_tests\\": {}, \\"total_tests\\": {}}}", correctness, performance_score, overall_score, avg_time, correct_count, test_results.len());
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
                        "score": results["overall_score"],
                        "compile_success": 1.0,
                        "correctness": results["correctness"],
                        "performance": results["performance_score"],
                        "avg_time_ns": results["avg_time_ns"],
                    },
                    artifacts={
                        "correct_tests": results["correct_tests"],
                        "total_tests": results["total_tests"],
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