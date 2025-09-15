"""
Specialized Query Processing Evaluator for OpenEvolve Optimization

This evaluator is designed to work with OpenEvolve to optimize query processing
algorithms for the OWL2 reasoner. It evaluates evolved programs based on query
performance, correctness, memory efficiency, and scalability.

## Key Features

- **Query Performance Testing**: Measures query execution speed and throughput
- **Correctness Validation**: Ensures query results are accurate
- **Memory Efficiency**: Tracks memory usage patterns during query processing
- **Scalability Testing**: Evaluates performance on larger datasets
- **Multi-dimensional Fitness**: Comprehensive evaluation across multiple metrics

Usage:
    python query_evaluator.py evolved_program.rs
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
class QueryEvaluationResult:
    """Result of evaluating an evolved query processing program"""
    fitness: float
    features: List[float]
    artifacts: Dict[str, Any]
    compilation_success: bool
    correctness: float
    performance: float
    memory_efficiency: float
    scalability: float

class QueryEvaluator:
    """Specialized evaluator for query processing optimization"""

    def __init__(self, benchmark_complexity: str = "medium"):
        self.benchmark_complexity = benchmark_complexity
        self.test_cases = self._generate_test_cases()
        self.temp_dir = tempfile.mkdtemp(prefix="query_eval_")

    def __del__(self):
        """Clean up temporary directory"""
        try:
            shutil.rmtree(self.temp_dir)
        except:
            pass

    def evaluate(self, program: str) -> QueryEvaluationResult:
        """
        Evaluate an evolved query processing program

        Args:
            program: Rust source code of the evolved query processor

        Returns:
            QueryEvaluationResult with fitness and metrics
        """
        print(f"Evaluating evolved query processing program...")

        # Test compilation
        if not self._compile_program(program):
            return QueryEvaluationResult(
                fitness=0.0,
                features=[0.0, 0.0, 0.0, 0.0],
                artifacts={"compilation": "failed", "error": "Compilation failed"},
                compilation_success=False,
                correctness=0.0,
                performance=0.0,
                memory_efficiency=0.0,
                scalability=0.0,
            )

        # Test query correctness
        correctness = self._test_correctness()
        if correctness < 0.8:  # Minimum correctness threshold
            return QueryEvaluationResult(
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

        # Test query performance
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
            min(1.0, 2.0 / performance) * 0.3 +   # 30% speed (target <2ms)
            min(1.0, 200.0 / memory_efficiency) * 0.2 +  # 20% memory (target <200KB)
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

        return QueryEvaluationResult(
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
            program_path = Path(self.temp_dir) / "evolved_query_processor.rs"
            with open(program_path, 'w') as f:
                f.write(program)

            # Compile with Rust
            result = subprocess.run([
                "rustc", "--edition", "2021",
                "-O",  # Optimize
                program_path,
                "-o", str(Path(self.temp_dir) / "evolved_query_processor")
            ], capture_output=True, text=True, timeout=30)

            if result.returncode != 0:
                print(f"Compilation failed: {result.stderr}")
                return False

            return True

        except Exception as e:
            print(f"Compilation error: {e}")
            return False

    def _test_correctness(self) -> float:
        """Test query correctness of the evolved algorithm"""
        test_results = []

        # Test case 1: SELECT query correctness
        test_results.append(self._run_correctness_test("select", True))

        # Test case 2: ASK query correctness
        test_results.append(self._run_correctness_test("ask", True))

        # Test case 3: CONSTRUCT query correctness
        test_results.append(self._run_correctness_test("construct", True))

        # Test case 4: DESCRIBE query correctness
        test_results.append(self._run_correctness_test("describe", True))

        # Test case 5: Complex query with joins
        test_results.append(self._run_correctness_test("complex", True))

        # Test case 6: Negative query (should return false)
        test_results.append(self._run_correctness_test("negative", False))

        correctness_score = mean(test_results)
        print(f"Query correctness score: {correctness_score:.3f}")

        return correctness_score

    def _run_correctness_test(self, test_type: str, expected_success: bool) -> float:
        """Run a single correctness test"""
        try:
            executable = Path(self.temp_dir) / "evolved_query_processor"
            result = subprocess.run([
                str(executable), "--test", test_type
            ], capture_output=True, text=True, timeout=10)

            if result.returncode != 0:
                return 0.0

            # Parse output to determine if result matches expectation
            output = result.stdout.strip()
            actual_success = "success" in output.lower()

            return 1.0 if actual_success == expected_success else 0.0

        except Exception:
            return 0.0

    def _benchmark_performance(self) -> float:
        """Benchmark query processing performance"""
        times = []

        for i in range(10):  # Run 10 iterations
            try:
                executable = Path(self.temp_dir) / "evolved_query_processor"
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
        print(f"Average query performance: {avg_time:.3f}ms")

        return avg_time

    def _measure_memory_efficiency(self) -> float:
        """Measure memory usage of the evolved query processor"""
        try:
            executable = Path(self.temp_dir) / "evolved_query_processor"

            # Use /usr/bin/time to measure memory
            result = subprocess.run([
                "/usr/bin/time", "-f", "%M",  # Maximum memory in KB
                str(executable), "--memory-test"
            ], capture_output=True, text=True, timeout=30)

            if result.returncode == 0:
                # Extract memory usage from stderr
                memory_kb = int(result.stderr.strip())
                print(f"Query processing memory usage: {memory_kb}KB")
                return memory_kb

        except Exception:
            pass

        return 10000.0  # High penalty for memory measurement failure

    def _test_scalability(self) -> float:
        """Test scalability with increasingly complex queries"""
        complexity_levels = ["small", "medium", "large"]
        times = []

        for complexity in complexity_levels:
            try:
                executable = Path(self.temp_dir) / "evolved_query_processor"
                start_time = time.time()

                result = subprocess.run([
                    str(executable), "--scalability", complexity
                ], capture_output=True, text=True, timeout=60)

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
        print(f"Query processing scalability score: {scalability:.3f}")

        return scalability

    def _generate_test_cases(self) -> List[Dict[str, Any]]:
        """Generate test cases for query evaluation"""
        return [
            {
                "name": "select",
                "query_type": "SELECT",
                "query": "SELECT ?person WHERE { ?person rdf:type Person }",
                "expected_results": 3,
                "description": "Basic SELECT query"
            },
            {
                "name": "ask",
                "query_type": "ASK",
                "query": "ASK { Alice knows Bob }",
                "expected_result": True,
                "description": "Boolean ASK query"
            },
            {
                "name": "construct",
                "query_type": "CONSTRUCT",
                "query": "CONSTRUCT { ?s ?p ?o } WHERE { ?s rdf:type Person }",
                "expected_triples": 3,
                "description": "Graph construction query"
            },
            {
                "name": "describe",
                "query_type": "DESCRIBE",
                "query": "DESCRIBE Alice",
                "expected_triples": 3,
                "description": "Resource description query"
            },
            {
                "name": "complex",
                "query_type": "SELECT",
                "query": "SELECT ?person ?company WHERE { ?person rdf:type Person ; worksAt ?company . ?company rdf:type Organization }",
                "expected_results": 2,
                "description": "Complex query with joins"
            },
            {
                "name": "negative",
                "query_type": "ASK",
                "query": "ASK { NonexistentEntity rdf:type Person }",
                "expected_result": False,
                "description": "Negative query (should return false)"
            },
        ]

    def get_evaluation_summary(self, results: List[QueryEvaluationResult]) -> Dict[str, Any]:
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

    evaluator = QueryEvaluator()
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
        print("Usage: python query_evaluator.py <program_file.rs>")
        sys.exit(1)

    program_file = sys.argv[1]

    try:
        with open(program_file, 'r') as f:
            program = f.read()

        result = evaluate(program)

        print(f"\n=== Query Processing Evaluation Results ===")
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