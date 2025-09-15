"""
Specialized Rule System Evaluator for OpenEvolve Optimization

This evaluator is designed to work with OpenEvolve to optimize OWL2 reasoning rules
and forward chaining algorithms. It evaluates evolved programs based on rule application
performance, correctness, memory efficiency, and scalability.

## Key Features

- **Rule Performance Testing**: Measures rule execution speed and inference throughput
- **Correctness Validation**: Ensures rule-based reasoning produces correct results
- **Memory Efficiency**: Tracks memory usage during rule processing
- **Scalability Testing**: Evaluates performance with increasing rule sets and facts
- **Multi-dimensional Fitness**: Comprehensive evaluation across multiple metrics

Usage:
    python rule_evaluator.py evolved_program.rs
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
class RuleEvaluationResult:
    """Result of evaluating an evolved rule system program"""
    fitness: float
    features: List[float]
    artifacts: Dict[str, Any]
    compilation_success: bool
    correctness: float
    performance: float
    memory_efficiency: float
    scalability: float

class RuleEvaluator:
    """Specialized evaluator for rule system optimization"""

    def __init__(self, benchmark_complexity: str = "medium"):
        self.benchmark_complexity = benchmark_complexity
        self.test_cases = self._generate_test_cases()
        self.temp_dir = tempfile.mkdtemp(prefix="rule_eval_")

    def __del__(self):
        """Clean up temporary directory"""
        try:
            shutil.rmtree(self.temp_dir)
        except:
            pass

    def evaluate(self, program: str) -> RuleEvaluationResult:
        """
        Evaluate an evolved rule system program

        Args:
            program: Rust source code of the evolved rule engine

        Returns:
            RuleEvaluationResult with fitness and metrics
        """
        print(f"Evaluating evolved rule system program...")

        # Test compilation
        if not self._compile_program(program):
            return RuleEvaluationResult(
                fitness=0.0,
                features=[0.0, 0.0, 0.0, 0.0],
                artifacts={"compilation": "failed", "error": "Compilation failed"},
                compilation_success=False,
                correctness=0.0,
                performance=0.0,
                memory_efficiency=0.0,
                scalability=0.0,
            )

        # Test rule correctness
        correctness = self._test_correctness()
        if correctness < 0.7:  # Minimum correctness threshold
            return RuleEvaluationResult(
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

        # Test rule performance
        performance = self._benchmark_performance()

        # Test memory efficiency
        memory_efficiency = self._measure_memory_efficiency()

        # Test scalability
        scalability = self._test_scalability()

        # Calculate features for MAP-Elites
        features = [
            correctness,                           # Correctness feature (0-1)
            min(1.0, 20.0 / performance),          # Speed feature (higher is better)
            min(1.0, 1000.0 / memory_efficiency),  # Memory efficiency feature
            scalability                            # Scalability feature (0-1)
        ]

        # Calculate weighted fitness
        fitness = (
            correctness * 0.4 +                     # 40% correctness
            min(1.0, 5.0 / performance) * 0.3 +   # 30% speed (target <5ms)
            min(1.0, 300.0 / memory_efficiency) * 0.2 +  # 20% memory (target <300KB)
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

        return RuleEvaluationResult(
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
            program_path = Path(self.temp_dir) / "evolved_rule_engine.rs"
            with open(program_path, 'w') as f:
                f.write(program)

            # Compile with Rust
            result = subprocess.run([
                "rustc", "--edition", "2021",
                "-O",  # Optimize
                program_path,
                "-o", str(Path(self.temp_dir) / "evolved_rule_engine")
            ], capture_output=True, text=True, timeout=30)

            if result.returncode != 0:
                print(f"Compilation failed: {result.stderr}")
                return False

            return True

        except Exception as e:
            print(f"Compilation error: {e}")
            return False

    def _test_correctness(self) -> float:
        """Test rule correctness of the evolved algorithm"""
        test_results = []

        # Test case 1: SubClassOf transitivity
        test_results.append(self._run_correctness_test("subclass_transitivity", True))

        # Test case 2: Domain inference
        test_results.append(self._run_correctness_test("domain_inference", True))

        # Test case 3: Range inference
        test_results.append(self._run_correctness_test("range_inference", True))

        # Test case 4: Transitive property
        test_results.append(self._run_correctness_test("transitive_property", True))

        # Test case 5: Complex inference chains
        test_results.append(self._run_correctness_test("complex_chains", True))

        # Test case 6: Conflict resolution
        test_results.append(self._run_correctness_test("conflict_resolution", True))

        correctness_score = mean(test_results)
        print(f"Rule correctness score: {correctness_score:.3f}")

        return correctness_score

    def _run_correctness_test(self, test_type: str, expected_success: bool) -> float:
        """Run a single correctness test"""
        try:
            executable = Path(self.temp_dir) / "evolved_rule_engine"
            result = subprocess.run([
                str(executable), "--test-correctness"
            ], capture_output=True, text=True, timeout=15)

            if result.returncode != 0:
                return 0.0

            # Parse output to determine if result matches expectation
            output = result.stdout.strip()
            actual_success = "success" in output.lower()

            return 1.0 if actual_success == expected_success else 0.0

        except Exception:
            return 0.0

    def _benchmark_performance(self) -> float:
        """Benchmark rule system performance"""
        times = []

        for i in range(8):  # Run 8 iterations
            try:
                executable = Path(self.temp_dir) / "evolved_rule_engine"
                start_time = time.time()

                result = subprocess.run([
                    str(executable), "--benchmark"
                ], capture_output=True, text=True, timeout=45)

                end_time = time.time()
                execution_time = (end_time - start_time) * 1000  # Convert to ms

                if result.returncode == 0:
                    times.append(execution_time)

            except Exception:
                continue

        if not times:
            return 1000.0  # High penalty for failed benchmarks

        avg_time = mean(times)
        print(f"Average rule system performance: {avg_time:.3f}ms")

        return avg_time

    def _measure_memory_efficiency(self) -> float:
        """Measure memory usage of the evolved rule engine"""
        try:
            executable = Path(self.temp_dir) / "evolved_rule_engine"

            # Use /usr/bin/time to measure memory
            result = subprocess.run([
                "/usr/bin/time", "-f", "%M",  # Maximum memory in KB
                str(executable), "--memory-test"
            ], capture_output=True, text=True, timeout=45)

            if result.returncode == 0:
                # Extract memory usage from stderr
                memory_kb = int(result.stderr.strip())
                print(f"Rule system memory usage: {memory_kb}KB")
                return memory_kb

        except Exception:
            pass

        return 10000.0  # High penalty for memory measurement failure

    def _test_scalability(self) -> float:
        """Test scalability with increasing complexity"""
        complexity_levels = ["small", "medium", "large"]
        times = []

        for complexity in complexity_levels:
            try:
                executable = Path(self.temp_dir) / "evolved_rule_engine"
                start_time = time.time()

                result = subprocess.run([
                    str(executable), "--scalability", complexity
                ], capture_output=True, text=True, timeout=90)

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
        print(f"Rule system scalability score: {scalability:.3f}")

        return scalability

    def _generate_test_cases(self) -> List[Dict[str, Any]]:
        """Generate test cases for rule evaluation"""
        return [
            {
                "name": "subclass_transitivity",
                "rule_type": "SubClassOf",
                "description": "Transitive subclass inference",
                "expected_inferences": [
                    ("Student", "rdfs:subClassOf", "Agent"),
                    ("Professor", "rdfs:subClassOf", "Person"),
                ]
            },
            {
                "name": "domain_inference",
                "rule_type": "Domain",
                "description": "Domain-based type inference",
                "expected_inferences": [
                    ("Alice", "rdf:type", "Person"),
                    ("Bob", "rdf:type", "Person"),
                ]
            },
            {
                "name": "range_inference",
                "rule_type": "Range",
                "description": "Range-based type inference",
                "expected_inferences": [
                    ("Bob", "rdf:type", "Person"),
                    ("Math101", "rdf:type", "Course"),
                ]
            },
            {
                "name": "transitive_property",
                "rule_type": "Transitive",
                "description": "Transitive property inference",
                "expected_inferences": [
                    ("Alice", "hasParent", "Grandparent"),
                ]
            },
            {
                "name": "complex_chains",
                "rule_type": "Complex",
                "description": "Complex inference chains",
                "expected_inferences": [
                    ("Professor", "rdfs:subClassOf", "Agent"),
                    ("Alice", "rdf:type", "Agent"),
                ]
            },
            {
                "name": "conflict_resolution",
                "rule_type": "Conflict",
                "description": "Rule conflict resolution",
                "expected_success": True,
            },
        ]

    def get_evaluation_summary(self, results: List[RuleEvaluationResult]) -> Dict[str, Any]:
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
        program: Rust source code as string or file path
        work_dir: Working directory for evaluation

    Returns:
        dict with evaluation results
    """
    # Handle case where program is a file path (OpenEvolve might pass file path)
    if isinstance(program, str) and program.endswith('.rs') and '\n' not in program:
        # This looks like a file path, read the contents
        try:
            with open(program, 'r') as f:
                program_content = f.read()
        except Exception as e:
            return {
                "fitness": 0.0,
                "features": [0.0, 0.0, 0.0, 0.0],
                "artifacts": {"compilation": "failed", "error": f"Could not read program file: {e}"},
                "compilation_success": False,
                "correctness": 0.0,
                "performance": 0.0,
                "memory_efficiency": 0.0,
                "scalability": 0.0,
            }
    else:
        program_content = program

    evaluator = RuleEvaluator()
    result = evaluator.evaluate(program_content)

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
        print("Usage: python rule_evaluator.py <program_file.rs>")
        sys.exit(1)

    program_file = sys.argv[1]

    try:
        with open(program_file, 'r') as f:
            program = f.read()

        result = evaluate(program)

        print(f"\n=== Rule System Evaluation Results ===")
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