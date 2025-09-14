"""
Specialized Tableaux Evaluator for OpenEvolve Optimization

This evaluator is designed to work with OpenEvolve to optimize the tableaux reasoning
algorithm for the OWL2 reasoner. It evaluates evolved programs based on correctness,
performance, memory efficiency, and scalability.

## Key Features

- **Correctness Testing**: Ensures logical correctness is maintained
- **Performance Benchmarking**: Measures reasoning speed and throughput
- **Memory Efficiency**: Tracks memory usage patterns
- **Scalability Testing**: Evaluates performance on larger ontologies
- **Comprehensive Metrics**: Multi-dimensional fitness evaluation

Usage:
    python tableaux_evaluator.py evolved_program.rs
"""

import sys
import os
import time
import subprocess
import json
import tempfile
import shutil
from pathlib import Path
from dataclasses import dataclass
from typing import List, Dict, Any, Optional, Tuple
from statistics import mean, stdev

@dataclass
class EvaluationResult:
    """Result of evaluating an evolved tableaux program"""
    fitness: float
    features: List[float]
    artifacts: Dict[str, Any]
    compilation_success: bool
    correctness: float
    performance: float
    memory_efficiency: float
    scalability: float

class TableauxEvaluator:
    """Specialized evaluator for tableaux algorithm optimization"""

    def __init__(self, benchmark_complexity: str = "medium"):
        self.benchmark_complexity = benchmark_complexity
        self.test_cases = self._generate_test_cases()
        self.temp_dir = tempfile.mkdtemp(prefix="tableaux_eval_")

    def __del__(self):
        """Clean up temporary directory"""
        try:
            shutil.rmtree(self.temp_dir)
        except:
            pass

    def evaluate(self, program: str) -> EvaluationResult:
        """
        Evaluate an evolved tableaux program

        Args:
            program: Rust source code of the evolved tableaux algorithm

        Returns:
            EvaluationResult with fitness and metrics
        """
        print(f"Evaluating evolved tableaux program...")

        # Test compilation
        if not self._compile_program(program):
            return EvaluationResult(
                fitness=0.0,
                features=[0.0, 0.0, 0.0, 0.0],
                artifacts={"compilation": "failed", "error": "Compilation failed"},
                compilation_success=False,
                correctness=0.0,
                performance=0.0,
                memory_efficiency=0.0,
                scalability=0.0,
            )

        # Test reasoning correctness
        correctness = self._test_correctness()
        if correctness < 0.8:  # Minimum correctness threshold
            return EvaluationResult(
                fitness=correctness * 0.5,  # Heavy penalty for incorrectness
                features=[correctness, 0.0, 0.0, 0.0],
                artifacts={
                    "compilation": "success",
                    "correctness": correctness,
                    "performance": "failed_correctness",
                },
                compilation_success=True,
                correctness=correctness,
                performance=0.0,
                memory_efficiency=0.0,
                scalability=0.0,
            )

        # Test performance
        performance = self._benchmark_performance()

        # Test memory efficiency
        memory_efficiency = self._measure_memory_efficiency()

        # Test scalability
        scalability = self._test_scalability()

        # Calculate features for MAP-Elites
        features = [
            correctness,                           # Correctness feature (0-1)
            min(1.0, 10.0 / performance),          # Speed feature (higher is better)
            min(1.0, 1000.0 / memory_efficiency),  # Memory efficiency feature
            scalability                            # Scalability feature (0-1)
        ]

        # Calculate weighted fitness
        fitness = (
            correctness * 0.4 +                     # 40% correctness
            min(1.0, 5.0 / performance) * 0.3 +    # 30% speed (target <5ms)
            min(1.0, 500.0 / memory_efficiency) * 0.2 +  # 20% memory (target <500KB)
            scalability * 0.1                      # 10% scalability
        )

        artifacts = {
            "compilation": "success",
            "correctness": correctness,
            "performance_ms": performance,
            "memory_kb": memory_efficiency,
            "scalability": scalability,
            "features": features,
            "fitness": fitness,
        }

        print(f"Evaluation complete - Fitness: {fitness:.3f}")
        print(f"  Correctness: {correctness:.3f}")
        print(f"  Performance: {performance:.3f}ms")
        print(f"  Memory: {memory_efficiency:.1f}KB")
        print(f"  Scalability: {scalability:.3f}")

        return EvaluationResult(
            fitness=fitness,
            features=features,
            artifacts=artifacts,
            compilation_success=True,
            correctness=correctness,
            performance=performance,
            memory_efficiency=memory_efficiency,
            scalability=scalability,
        )

    def _compile_program(self, program: str) -> bool:
        """Compile the evolved Rust program"""
        try:
            # Write program to temporary file
            program_path = Path(self.temp_dir) / "evolved_tableaux.rs"
            with open(program_path, 'w') as f:
                f.write(program)

            # Compile with Rust
            result = subprocess.run([
                "rustc", "--edition", "2021",
                "-O",  # Optimize
                program_path,
                "-o", str(Path(self.temp_dir) / "evolved_tableaux")
            ], capture_output=True, text=True, timeout=30)

            if result.returncode != 0:
                print(f"Compilation failed: {result.stderr}")
                return False

            return True

        except Exception as e:
            print(f"Compilation error: {e}")
            return False

    def _test_correctness(self) -> float:
        """Test logical correctness of the evolved algorithm"""
        test_results = []

        # Test case 1: Simple satisfiability
        test_results.append(self._run_correctness_test("simple", True))

        # Test case 2: Intersection satisfiability
        test_results.append(self._run_correctness_test("intersection", True))

        # Test case 3: Union satisfiability
        test_results.append(self._run_correctness_test("union", True))

        # Test case 4: Existential satisfiability
        test_results.append(self._run_correctness_test("existential", True))

        # Test case 5: Contradiction detection
        test_results.append(self._run_correctness_test("contradiction", False))

        # Test case 6: Complex satisfiability
        test_results.append(self._run_correctness_test("complex", True))

        correctness_score = mean(test_results)
        print(f"Correctness score: {correctness_score:.3f}")

        return correctness_score

    def _run_correctness_test(self, test_type: str, expected_satisfiable: bool) -> float:
        """Run a single correctness test"""
        try:
            executable = Path(self.temp_dir) / "evolved_tableaux"
            result = subprocess.run([
                str(executable), "--test", test_type
            ], capture_output=True, text=True, timeout=10)

            if result.returncode != 0:
                return 0.0

            # Parse output to determine if result matches expectation
            output = result.stdout.strip()
            actual_satisfiable = "satisfiable: true" in output.lower()

            return 1.0 if actual_satisfiable == expected_satisfiable else 0.0

        except Exception:
            return 0.0

    def _benchmark_performance(self) -> float:
        """Benchmark reasoning performance"""
        times = []

        for i in range(10):  # Run 10 iterations
            try:
                executable = Path(self.temp_dir) / "evolved_tableaux"
                start_time = time.time()

                result = subprocess.run([
                    str(executable), "--benchmark"
                ], capture_output=True, text=True, timeout=30)

                end_time = time.time()
                execution_time = (end_time - start_time) * 1000  # Convert to ms

                if result.returncode == 0:
                    times.append(execution_time)

            except Exception:
                continue

        if not times:
            return 1000.0  # High penalty for failed benchmarks

        avg_time = mean(times)
        print(f"Average performance: {avg_time:.3f}ms")

        return avg_time

    def _measure_memory_efficiency(self) -> float:
        """Measure memory usage of the evolved algorithm"""
        try:
            executable = Path(self.temp_dir) / "evolved_tableaux"

            # Use /usr/bin/time to measure memory
            result = subprocess.run([
                "/usr/bin/time", "-f", "%M",  # Maximum memory in KB
                str(executable), "--memory-test"
            ], capture_output=True, text=True, timeout=30)

            if result.returncode == 0:
                # Extract memory usage from stderr
                memory_kb = int(result.stderr.strip())
                print(f"Memory usage: {memory_kb}KB")
                return memory_kb

        except Exception:
            pass

        return 10000.0  # High penalty for memory measurement failure

    def _test_scalability(self) -> float:
        """Test scalability with increasingly complex problems"""
        complexity_levels = ["small", "medium", "large"]
        times = []

        for complexity in complexity_levels:
            try:
                executable = Path(self.temp_dir) / "evolved_tableaux"
                start_time = time.time()

                result = subprocess.run([
                    str(executable), "--scalability", complexity
                ], capture_output=True, text_output=True, timeout=60)

                end_time = time.time()
                execution_time = end_time - start_time

                if result.returncode == 0:
                    times.append(execution_time)

            except Exception:
                times.append(1000.0)  # High penalty for timeout

        # Calculate scalability as ability to handle increasing complexity
        if len(times) < 2:
            return 0.0

        # Lower time ratios indicate better scalability
        time_ratios = [times[i] / times[0] for i in range(1, len(times))]
        avg_ratio = mean(time_ratios)

        # Convert to scalability score (lower ratios = better scalability)
        scalability = max(0.0, 1.0 - (avg_ratio - 1.0))
        print(f"Scalability score: {scalability:.3f}")

        return scalability

    def _generate_test_cases(self) -> List[Dict[str, Any]]:
        """Generate test cases for evaluation"""
        return [
            {
                "name": "simple",
                "concept": "Person",
                "expected_satisfiable": True,
                "description": "Simple named concept"
            },
            {
                "name": "intersection",
                "concept": "Person AND Parent",
                "expected_satisfiable": True,
                "description": "Intersection of two concepts"
            },
            {
                "name": "union",
                "concept": "Person OR Organization",
                "expected_satisfiable": True,
                "description": "Union of two concepts"
            },
            {
                "name": "existential",
                "concept": "hasChild SOME Person",
                "expected_satisfiable": True,
                "description": "Existential restriction"
            },
            {
                "name": "contradiction",
                "concept": "Person AND (NOT Person)",
                "expected_satisfiable": False,
                "description": "Direct contradiction"
            },
            {
                "name": "complex",
                "concept": "Person AND (hasChild SOME Doctor) AND (NOT Student)",
                "expected_satisfiable": True,
                "description": "Complex concept with multiple restrictions"
            },
        ]

    def get_evaluation_summary(self, results: List[EvaluationResult]) -> Dict[str, Any]:
        """Generate summary of multiple evaluations"""
        if not results:
            return {}

        successful_results = [r for r in results if r.compilation_success]

        summary = {
            "total_evaluations": len(results),
            "successful_compilations": len(successful_results),
            "compilation_success_rate": len(successful_results) / len(results),
        }

        if successful_results:
            summary.update({
                "avg_fitness": mean(r.fitness for r in successful_results),
                "avg_correctness": mean(r.correctness for r in successful_results),
                "avg_performance": mean(r.performance for r in successful_results),
                "avg_memory": mean(r.memory_efficiency for r in successful_results),
                "avg_scalability": mean(r.scalability for r in successful_results),
                "best_fitness": max(r.fitness for r in successful_results),
                "best_correctness": max(r.correctness for r in successful_results),
                "best_performance": min(r.performance for r in successful_results),
            })

        return summary

def evaluate(program, work_dir=None):
    """
    OpenEvolve-compatible evaluation function

    Args:
        program: Rust source code as string
        work_dir: Working directory for evaluation

    Returns:
        dict with evaluation results
    """
    evaluator = TableauxEvaluator()
    result = evaluator.evaluate(program)

    return {
        "fitness": result.fitness,
        "features": result.features,
        "artifacts": result.artifacts,
        "compilation_success": result.compilation_success,
        "correctness": result.correctness,
        "performance": result.performance,
        "memory_efficiency": result.memory_efficiency,
        "scalability": result.scalability,
    }

def main():
    """Main entry point for standalone evaluation"""
    if len(sys.argv) != 2:
        print("Usage: python tableaux_evaluator.py <program_file.rs>")
        sys.exit(1)

    program_file = sys.argv[1]

    try:
        with open(program_file, 'r') as f:
            program = f.read()

        result = evaluate(program)

        print(f"\n=== Evaluation Results ===")
        print(f"Fitness: {result['fitness']:.3f}")
        print(f"Correctness: {result['correctness']:.3f}")
        print(f"Performance: {result['performance']:.3f}ms")
        print(f"Memory: {result['memory_efficiency']:.1f}KB")
        print(f"Scalability: {result['scalability']:.3f}")

        # Save results to JSON
        results_file = program_file.replace('.rs', '_results.json')
        with open(results_file, 'w') as f:
            json.dump(result, f, indent=2)

        print(f"\nResults saved to: {results_file}")

    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()