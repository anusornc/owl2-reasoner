#!/usr/bin/env python3
"""
OpenEvolve Evaluator for Subclass Checking Optimization

Specialized evaluator for evolving SimpleReasoner's subclass checking algorithm
from O(n²) DFS to O(N+E) BFS with better performance characteristics.
"""

import json
import subprocess
import tempfile
import time
import statistics
from pathlib import Path
from typing import Dict, List, Tuple, Any
import sys
import os

# Add current directory to Python path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

class EvaluationResult:
    """Result of evaluating an evolved subclass checking program"""

    def __init__(self,
                 execution_time: float,
                 compilation_success: bool,
                 test_pass_rate: float,
                 memory_efficiency: float,
                 scalability_score: float,
                 correctness: bool,
                 error_message: str = ""):
        self.execution_time = execution_time
        self.compilation_success = compilation_success
        self.test_pass_rate = test_pass_rate
        self.memory_efficiency = memory_efficiency
        self.scalability_score = scalability_score
        self.correctness = correctness
        self.error_message = error_message

    def to_dict(self) -> Dict[str, Any]:
        return {
            'execution_time': self.execution_time,
            'compilation_success': self.compilation_success,
            'test_pass_rate': self.test_pass_rate,
            'memory_efficiency': self.memory_efficiency,
            'scalability_score': self.scalability_score,
            'correctness': self.correctness,
            'error_message': self.error_message
        }

def evaluate_subclass_algorithm(program_path: str) -> EvaluationResult:
    """
    Evaluate an evolved subclass checking algorithm

    Args:
        program_path: Path to the evolved Rust program

    Returns:
        EvaluationResult with performance metrics
    """
    try:
        with tempfile.TemporaryDirectory() as temp_dir:
            project_dir = Path(temp_dir) / "subclass_test"

            # Initialize Cargo project
            result = subprocess.run(
                ["cargo", "init", "--name", "subclass_test", "--lib", str(project_dir)],
                capture_output=True, text=True, timeout=30
            )

            if result.returncode != 0:
                return EvaluationResult(
                    execution_time=float('inf'),
                    compilation_success=False,
                    test_pass_rate=0.0,
                    memory_efficiency=0.0,
                    scalability_score=0.0,
                    correctness=False,
                    error_message=f"Cargo init failed: {result.stderr}"
                )

            # Copy the evolved program
            src_dir = project_dir / "src"
            evolved_program = Path(program_path)

            if not evolved_program.exists():
                return EvaluationResult(
                    execution_time=float('inf'),
                    compilation_success=False,
                    test_pass_rate=0.0,
                    memory_efficiency=0.0,
                    scalability_score=0.0,
                    correctness=False,
                    error_message=f"Program file not found: {program_path}"
                )

            # Read and prepare the evolved program
            with open(evolved_program, 'r') as f:
                evolved_code = f.read()

            # Create the main lib.rs file
            lib_rs_path = src_dir / "lib.rs"
            with open(lib_rs_path, 'w') as f:
                f.write(evolved_code)

            # Program is self-contained, no additional dependencies needed

            # Compilation test
            start_time = time.time()
            compile_result = subprocess.run(
                ["cargo", "check"],
                cwd=project_dir,
                capture_output=True, text=True, timeout=60
            )
            compilation_time = time.time() - start_time

            if compile_result.returncode != 0:
                return EvaluationResult(
                    execution_time=compilation_time,
                    compilation_success=False,
                    test_pass_rate=0.0,
                    memory_efficiency=0.0,
                    scalability_score=0.0,
                    correctness=False,
                    error_message=f"Compilation failed: {compile_result.stderr}"
                )

            # Build and run performance tests
            start_time = time.time()
            run_result = subprocess.run(
                ["cargo", "run", "--release"],
                cwd=project_dir,
                capture_output=True, text=True, timeout=120
            )
            execution_time = time.time() - start_time

            # Parse output for performance metrics
            test_pass_rate = 0.8  # Default if parsing fails
            memory_efficiency = 0.5  # Default
            scalability_score = 0.5  # Default
            correctness = False

            if run_result.returncode == 0:
                output = run_result.stdout
                correctness = True

                # Parse performance metrics from output
                try:
                    for line in output.split('\n'):
                        if "Performance time:" in line:
                            perf_time = float(line.split(':')[-1].strip().split(' ')[0])
                            # Convert time to score (lower is better)
                            test_pass_rate = max(0.0, 1.0 - (perf_time / 1000000.0))  # Normalize to 0-1
                        elif "Estimated memory usage:" in line:
                            mem_usage = float(line.split(':')[-1].strip().split(' ')[0])
                            # Convert memory to score (lower is better)
                            memory_efficiency = max(0.0, 1.0 - (mem_usage / 10000.0))  # Normalize
                        elif "All tests completed successfully" in line:
                            test_pass_rate = 1.0
                except (ValueError, IndexError):
                    # Use default values if parsing fails
                    pass

                # Run stress test for scalability
                try:
                    stress_result = subprocess.run(
                        ["cargo", "test", "--release", "stress_test"],
                        cwd=project_dir,
                        capture_output=True, text=True, timeout=60
                    )
                    if stress_result.returncode == 0:
                        scalability_score = 1.0
                    else:
                        scalability_score = 0.3
                except subprocess.TimeoutExpired:
                    scalability_score = 0.1
            else:
                return EvaluationResult(
                    execution_time=execution_time,
                    compilation_success=True,
                    test_pass_rate=0.0,
                    memory_efficiency=0.0,
                    scalability_score=0.0,
                    correctness=False,
                    error_message=f"Execution failed: {run_result.stderr}"
                )

            # Run cargo test for additional correctness validation
            try:
                test_result = subprocess.run(
                    ["cargo", "test", "--release"],
                    cwd=project_dir,
                    capture_output=True, text=True, timeout=60
                )

                if test_result.returncode == 0:
                    test_output = test_result.stdout
                    if "test result: ok" in test_output:
                        test_pass_rate = 1.0
                else:
                    test_pass_rate *= 0.7  # Penalty for test failures
            except (subprocess.TimeoutExpired, subprocess.SubprocessError):
                test_pass_rate *= 0.5  # Penalty for test timeouts/errors

            return EvaluationResult(
                execution_time=execution_time,
                compilation_success=True,
                test_pass_rate=test_pass_rate,
                memory_efficiency=memory_efficiency,
                scalability_score=scalability_score,
                correctness=correctness,
                error_message=""
            )

    except subprocess.TimeoutExpired:
        return EvaluationResult(
            execution_time=120.0,  # Timeout value
            compilation_success=False,
            test_pass_rate=0.0,
            memory_efficiency=0.0,
            scalability_score=0.0,
            correctness=False,
            error_message="Evaluation timed out"
        )
    except Exception as e:
        return EvaluationResult(
            execution_time=float('inf'),
            compilation_success=False,
            test_pass_rate=0.0,
            memory_efficiency=0.0,
            scalability_score=0.0,
            correctness=False,
            error_message=f"Evaluation error: {str(e)}"
        )

def evaluate(program_path: str) -> Dict[str, Any]:
    """
    Main evaluation function for OpenEvolve

    Args:
        program_path: Path to the evolved program

    Returns:
        Dictionary with evaluation results and features
    """
    result = evaluate_subclass_algorithm(program_path)

    # Create feature vector for MAP-Elites
    features = [
        result.test_pass_rate,           # Correctness feature
        result.memory_efficiency,       # Memory efficiency feature
        result.scalability_score,        # Scalability feature
        min(1.0, result.execution_time / 10.0),  # Speed feature (normalized)
        1.0 if result.compilation_success else 0.0,  # Compilation success feature
    ]

    # Overall fitness score
    fitness = (
        result.test_pass_rate * 0.4 +          # 40% weight on correctness
        result.memory_efficiency * 0.2 +      # 20% weight on memory
        result.scalability_score * 0.2 +       # 20% weight on scalability
        (1.0 - min(1.0, result.execution_time / 10.0)) * 0.2  # 20% weight on speed
    )

    if not result.correctness or not result.compilation_success:
        fitness *= 0.1  # Heavy penalty for incorrect or non-compiling programs

    return {
        'fitness': fitness,
        'features': features,
        'metadata': {
            'execution_time': result.execution_time,
            'compilation_success': result.compilation_success,
            'test_pass_rate': result.test_pass_rate,
            'memory_efficiency': result.memory_efficiency,
            'scalability_score': result.scalability_score,
            'correctness': result.correctness,
            'error_message': result.error_message
        }
    }

def main():
    """Main evaluation function for standalone testing"""
    if len(sys.argv) != 2:
        print("Usage: python subclass_evaluator.py <program_path>")
        sys.exit(1)

    program_path = sys.argv[1]
    result = evaluate(program_path)

    print("Evaluation Results:")
    print(f"Fitness: {result['fitness']:.4f}")
    print(f"Features: {result['features']}")
    print(f"Metadata: {result['metadata']}")

if __name__ == "__main__":
    main()