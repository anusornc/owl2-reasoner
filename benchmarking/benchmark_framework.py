#!/usr/bin/env python3
"""
Comprehensive Benchmarking Framework for OWL2 Reasoners
Provides consistent performance measurement across different reasoner implementations
"""

import subprocess
import time
import json
import os
import sys
import statistics
import tempfile
from pathlib import Path
from typing import Dict, List, Tuple, Any
from dataclasses import dataclass, asdict
import argparse

@dataclass
class BenchmarkResult:
    """Standardized benchmark result format"""
    reasoner_name: str
    ontology_file: str
    operation: str
    execution_time_ms: float
    memory_usage_mb: float
    success: bool
    error_message: str = ""
    additional_metrics: Dict[str, Any] = None

    def __post_init__(self):
        if self.additional_metrics is None:
            self.additional_metrics = {}

class OWL2BenchmarkFramework:
    """Comprehensive benchmarking framework for OWL2 reasoners"""

    def __init__(self, results_dir: str = "benchmark_results"):
        self.results_dir = Path(results_dir)
        self.results_dir.mkdir(exist_ok=True)
        self.reasoners = {}
        self.test_ontologies = self._discover_test_ontologies()

    def register_reasoner(self, name: str, command: str, setup_func=None):
        """Register a reasoner for benchmarking"""
        self.reasoners[name] = {
            'command': command,
            'setup': setup_func,
            'available': self._check_reasoner_availability(command)
        }

    def _check_reasoner_availability(self, command: str) -> bool:
        """Check if a reasoner command is available"""
        try:
            result = subprocess.run(command.split(), capture_output=True, timeout=5)
            return result.returncode == 0 or result.returncode != 127  # 127 = command not found
        except (subprocess.TimeoutExpired, FileNotFoundError):
            return False

    def _discover_test_ontologies(self) -> List[Path]:
        """Discover test ontology files"""
        test_dir = Path(__file__).parent / "test_ontologies"
        ontologies = []
        for ext in ['*.owl', '*.ttl', '*.rdf', '*.xml']:
            ontologies.extend(test_dir.glob(ext))
        return ontologies

    def run_consistency_check(self, reasoner_name: str, ontology_file: Path) -> BenchmarkResult:
        """Benchmark consistency checking operation"""
        reasoner_config = self.reasoners.get(reasoner_name)
        if not reasoner_config:
            return BenchmarkResult(
                reasoner_name=reasoner_name,
                ontology_file=str(ontology_file),
                operation="consistency_check",
                execution_time_ms=0,
                memory_usage_mb=0,
                success=False,
                error_message=f"Reasoner {reasoner_name} not registered"
            )

        start_time = time.time()
        start_memory = self._get_memory_usage()

        try:
            # Construct command based on reasoner type
            if reasoner_name == "elk":
                cmd = f"java -jar {reasoner_config['command']} -c {ontology_file}"
            elif reasoner_name == "hermit":
                cmd = f"java -jar {reasoner_config['command']} {ontology_file}"
            elif reasoner_name == "rust_owl2":
                # Our Rust implementation
                cmd = f"cargo run --example consistency_check -- {ontology_file}"
            else:
                cmd = f"{reasoner_config['command']} {ontology_file}"

            result = subprocess.run(
                cmd.split(),
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )

            end_time = time.time()
            end_memory = self._get_memory_usage()

            execution_time = (end_time - start_time) * 1000  # Convert to ms
            memory_used = end_memory - start_memory

            success = result.returncode == 0

            return BenchmarkResult(
                reasoner_name=reasoner_name,
                ontology_file=str(ontology_file),
                operation="consistency_check",
                execution_time_ms=execution_time,
                memory_usage_mb=memory_used,
                success=success,
                error_message=result.stderr if not success else "",
                additional_metrics={
                    'stdout': result.stdout,
                    'exit_code': result.returncode
                }
            )

        except subprocess.TimeoutExpired:
            return BenchmarkResult(
                reasoner_name=reasoner_name,
                ontology_file=str(ontology_file),
                operation="consistency_check",
                execution_time_ms=300000,  # 5 minutes in ms
                memory_usage_mb=0,
                success=False,
                error_message="Timeout after 5 minutes"
            )
        except Exception as e:
            return BenchmarkResult(
                reasoner_name=reasoner_name,
                ontology_file=str(ontology_file),
                operation="consistency_check",
                execution_time_ms=0,
                memory_usage_mb=0,
                success=False,
                error_message=str(e)
            )

    def run_classification_benchmark(self, reasoner_name: str, ontology_file: Path) -> BenchmarkResult:
        """Benchmark ontology classification operation"""
        # Similar structure to consistency check but for classification
        # Implementation depends on specific reasoner capabilities
        return self.run_consistency_check(reasoner_name, ontology_file)  # Placeholder

    def run_query_benchmark(self, reasoner_name: str, ontology_file: Path) -> BenchmarkResult:
        """Benchmark query processing operation"""
        # This would need specific SPARQL queries or query patterns
        # For now, use a simple instance retrieval test
        reasoner_config = self.reasoners.get(reasoner_name)
        if not reasoner_config:
            return BenchmarkResult(
                reasoner_name=reasoner_name,
                ontology_file=str(ontology_file),
                operation="query_processing",
                execution_time_ms=0,
                memory_usage_mb=0,
                success=False,
                error_message=f"Reasoner {reasoner_name} not registered"
            )

        # For our Rust implementation, use the performance_benchmarking example
        if reasoner_name == "rust_owl2":
            start_time = time.time()
            start_memory = self._get_memory_usage()

            try:
                cmd = f"cargo run --example performance_benchmarking -- {ontology_file}"
                result = subprocess.run(
                    cmd.split(),
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                end_time = time.time()
                end_memory = self._get_memory_usage()

                execution_time = (end_time - start_time) * 1000
                memory_used = end_memory - start_memory

                # Parse query performance from output
                query_time = self._parse_query_performance(result.stdout)

                return BenchmarkResult(
                    reasoner_name=reasoner_name,
                    ontology_file=str(ontology_file),
                    operation="query_processing",
                    execution_time_ms=query_time,
                    memory_usage_mb=memory_used,
                    success=result.returncode == 0,
                    error_message=result.stderr if not success else "",
                    additional_metrics={
                        'total_time': execution_time,
                        'stdout': result.stdout,
                        'exit_code': result.returncode
                    }
                )

            except Exception as e:
                return BenchmarkResult(
                    reasoner_name=reasoner_name,
                    ontology_file=str(ontology_file),
                    operation="query_processing",
                    execution_time_ms=0,
                    memory_usage_mb=0,
                    success=False,
                    error_message=str(e)
                )
        else:
            # For other reasoners, we'd need to implement query testing
            return BenchmarkResult(
                reasoner_name=reasoner_name,
                ontology_file=str(ontology_file),
                operation="query_processing",
                execution_time_ms=0,
                memory_usage_mb=0,
                success=False,
                error_message="Query benchmarking not implemented for this reasoner"
            )

    def _parse_query_performance(self, stdout: str) -> float:
        """Parse query performance metrics from Rust benchmark output"""
        try:
            for line in stdout.split('\n'):
                if "Average query time:" in line:
                    # Extract time value (e.g., "Average query time: 81.4µs")
                    time_str = line.split(":")[1].strip()
                    if 'µs' in time_str:
                        return float(time_str.replace('µs', ''))
                    elif 'ms' in time_str:
                        return float(time_str.replace('ms', '')) * 1000
        except:
            pass
        return 0

    def _get_memory_usage(self) -> float:
        """Get current memory usage in MB"""
        try:
            import psutil
            process = psutil.Process()
            return process.memory_info().rss / 1024 / 1024  # Convert to MB
        except ImportError:
            # Fallback: return 0 if psutil not available
            return 0

    def run_comprehensive_benchmark(self, iterations: int = 5) -> Dict[str, List[BenchmarkResult]]:
        """Run comprehensive benchmarks across all reasoners and ontologies"""
        all_results = {}

        print(f"Running comprehensive benchmark with {iterations} iterations...")
        print(f"Available reasoners: {list(self.reasoners.keys())}")
        print(f"Test ontologies: {[str(o) for o in self.test_ontologies]}")

        for reasoner_name in self.reasoners:
            print(f"\nBenchmarking {reasoner_name}...")
            reasoner_results = []

            if not self.reasoners[reasoner_name]['available']:
                print(f"  ❌ {reasoner_name} not available")
                continue

            for ontology_file in self.test_ontologies:
                print(f"  📁 Testing with {ontology_file.name}...")

                # Run multiple iterations for statistical significance
                consistency_results = []
                query_results = []

                for i in range(iterations):
                    print(f"    Iteration {i+1}/{iterations}...")

                    # Consistency check
                    cons_result = self.run_consistency_check(reasoner_name, ontology_file)
                    consistency_results.append(cons_result)

                    # Query processing
                    query_result = self.run_query_benchmark(reasoner_name, ontology_file)
                    query_results.append(query_result)

                    # Small delay between iterations
                    time.sleep(0.1)

                # Calculate aggregates
                reasoner_results.extend(consistency_results)
                reasoner_results.extend(query_results)

            all_results[reasoner_name] = reasoner_results

        return all_results

    def generate_report(self, results: Dict[str, List[BenchmarkResult]]):
        """Generate comprehensive benchmark report"""
        report_file = self.results_dir / f"benchmark_report_{int(time.time())}.json"

        # Convert results to serializable format
        serializable_results = {}
        for reasoner, result_list in results.items():
            serializable_results[reasoner] = [asdict(result) for result in result_list]

        with open(report_file, 'w') as f:
            json.dump(serializable_results, f, indent=2)

        # Generate summary statistics
        self._generate_summary_report(results)

        print(f"📊 Benchmark report saved to {report_file}")

    def _generate_summary_report(self, results: Dict[str, List[BenchmarkResult]]):
        """Generate human-readable summary report"""
        summary_file = self.results_dir / f"benchmark_summary_{int(time.time())}.md"

        with open(summary_file, 'w') as f:
            f.write("# OWL2 Reasoner Benchmark Summary\n\n")
            f.write(f"Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}\n\n")

            for reasoner_name, result_list in results.items():
                f.write(f"## {reasoner_name}\n\n")

                successful_results = [r for r in result_list if r.success]
                failed_results = [r for r in result_list if not r.success]

                f.write(f"- **Successful runs**: {len(successful_results)}\n")
                f.write(f"- **Failed runs**: {len(failed_results)}\n")

                if successful_results:
                    consistency_times = [r.execution_time_ms for r in successful_results if r.operation == "consistency_check"]
                    query_times = [r.execution_time_ms for r in successful_results if r.operation == "query_processing"]

                    if consistency_times:
                        f.write(f"- **Avg consistency time**: {statistics.mean(consistency_times):.2f}ms\n")
                        f.write(f"- **Min consistency time**: {min(consistency_times):.2f}ms\n")
                        f.write(f"- **Max consistency time**: {max(consistency_times):.2f}ms\n")

                    if query_times:
                        f.write(f"- **Avg query time**: {statistics.mean(query_times):.2f}ms\n")
                        f.write(f"- **Min query time**: {min(query_times):.2f}ms\n")
                        f.write(f"- **Max query time**: {max(query_times):.2f}ms\n")

                f.write("\n")

        print(f"📋 Summary report saved to {summary_file}")

def main():
    parser = argparse.ArgumentParser(description="OWL2 Reasoner Benchmark Framework")
    parser.add_argument("--iterations", type=int, default=3, help="Number of benchmark iterations")
    parser.add_argument("--reasoners", nargs="+", help="Specific reasoners to benchmark")
    parser.add_argument("--output-dir", default="benchmark_results", help="Output directory for results")

    args = parser.parse_args()

    # Initialize framework
    framework = OWL2BenchmarkFramework(args.output_dir)

    # Register reasoners (paths need to be updated to actual locations)
    framework.register_reasoner(
        "rust_owl2",
        "cargo run --example",
        lambda: subprocess.run(["cargo", "build"], cwd="..")
    )

    # Note: These paths need to be updated to actual reasoner locations
    framework.register_reasoner("elk", "java -jar elk.jar")
    framework.register_reasoner("hermit", "java -jar hermit.jar")
    framework.register_reasoner("jfact", "java -jar jfact.jar")

    # Filter reasoners if specified
    if args.reasoners:
        framework.reasoners = {k: v for k, v in framework.reasoners.items() if k in args.reasoners}

    # Run benchmarks
    results = framework.run_comprehensive_benchmark(args.iterations)

    # Generate reports
    framework.generate_report(results)

    print("\n✅ Benchmarking completed!")

if __name__ == "__main__":
    main()