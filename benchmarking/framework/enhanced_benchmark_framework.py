#!/usr/bin/env python3

"""
Enhanced OWL2 Reasoner Benchmark Framework with LUBM and SP2B Integration
Extends the existing benchmark framework with comprehensive benchmark suites
"""

import subprocess
import time
import json
import os
import sys
import statistics
import math
from pathlib import Path
from typing import Dict, List, Tuple, Any, Optional
from dataclasses import dataclass, asdict, field
import argparse
import tempfile

# Import the original framework
import sys
from pathlib import Path
sys.path.append(str(Path(__file__).parent / "established_reasoners"))
from benchmark_framework import OWL2BenchmarkFramework, BenchmarkResult

@dataclass
class BenchmarkSuiteResult:
    """Enhanced benchmark result with multi-suite support"""
    reasoner_name: str
    benchmark_suite: str  # "LUBM", "SP2B", "CUSTOM", "BIOPORTAL", "SCALABILITY"
    dataset_size: str     # "1-university", "10-university", "scale-100", etc.
    operation: str         # "classification", "query", "consistency"
    query_name: Optional[str] = None
    execution_time_ms: float = 0.0
    memory_usage_mb: float = 0.0
    success: bool = True
    error_message: str = ""
    additional_metrics: Dict[str, Any] = field(default_factory=dict)

    def get_benchmark_type(self) -> str:
        """Get standardized benchmark type identifier"""
        return f"{self.benchmark_suite}_{self.dataset_size}_{self.operation}"

@dataclass
class BenchmarkComparison:
    """Cross-benchmark comparison results"""
    reasoner_name: str
    overall_rank: int
    lubm_rank: int
    sp2b_rank: int
    custom_rank: int
    scalability_rank: int
    performance_score: float  # Weighted average performance
    scalability_score: float  # Performance across different scales
    robustness_score: float  # Success rate across different tests

class LUBMBenchmark:
    """LUBM-specific benchmark implementation"""

    def __init__(self, lubm_dir: Path):
        self.lubm_dir = lubm_dir
        self.data_dir = lubm_dir / "data"
        self.query_dir = lubm_dir / "queries"
        self.generator_dir = lubm_dir / "generator"

    def run_lubm_test(self, reasoner_name: str, reasoner_config: dict,
                     university_count: int, operation: str, iterations: int = 3) -> List[BenchmarkSuiteResult]:
        """Run LUBM benchmark test"""
        results = []

        # Find the appropriate ontology file
        ontology_file = self.data_dir / f"university{university_count}.owl"

        if not ontology_file.exists():
            # Try to generate it
            self._generate_lubm_data(university_count)
            if not ontology_file.exists():
                return [BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="LUBM",
                    dataset_size=f"{university_count}-university",
                    operation=operation,
                    success=False,
                    error_message=f"Ontology file not found: {ontology_file}"
                )]

        if operation == "classification":
            return self._run_classification_test(reasoner_name, reasoner_config, ontology_file, university_count, iterations)
        elif operation == "query":
            return self._run_query_tests(reasoner_name, reasoner_config, ontology_file, university_count, iterations)
        elif operation == "consistency":
            return self._run_consistency_test(reasoner_name, reasoner_config, ontology_file, university_count, iterations)
        else:
            return [BenchmarkSuiteResult(
                reasoner_name=reasoner_name,
                benchmark_suite="LUBM",
                dataset_size=f"{university_count}-university",
                operation=operation,
                success=False,
                error_message=f"Unknown operation: {operation}"
            )]

    def _generate_lubm_data(self, university_count: int):
        """Generate LUBM data if not exists"""
        generator_script = self.generator_dir / "lubm_generator.py"
        if generator_script.exists():
            try:
                result = subprocess.run([
                    sys.executable, str(generator_script), str(university_count), str(self.data_dir)
                ], capture_output=True, text=True, timeout=300)
                if result.returncode != 0:
                    print(f"⚠️  LUBM generation failed: {result.stderr}")
            except subprocess.TimeoutExpired:
                print("⚠️  LUBM generation timed out")

    def _run_classification_test(self, reasoner_name: str, reasoner_config: dict,
                                ontology_file: Path, university_count: int, iterations: int) -> List[BenchmarkSuiteResult]:
        """Run LUBM classification test"""
        results = []

        for i in range(iterations):
            try:
                # Build command based on reasoner
                if reasoner_name == "rust_owl2":
                    cmd = reasoner_config["classification_cmd"].split() + [str(ontology_file)]
                    working_dir = reasoner_config.get("working_dir", ".")
                elif reasoner_name == "elk":
                    cmd = reasoner_config["classification_cmd"].split() + ["-i", str(ontology_file)]
                    working_dir = "."
                elif reasoner_name == "hermit":
                    cmd = reasoner_config["classification_cmd"].split() + ["-o", "/tmp/classification_output.txt", str(ontology_file)]
                    working_dir = "."
                else:
                    cmd = reasoner_config["classification_cmd"].split() + [str(ontology_file)]
                    working_dir = "."

                # Run the command
                start_time = time.time()
                start_memory = self._get_memory_usage()

                result = subprocess.run(
                    cmd,
                    cwd=working_dir,
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                end_time = time.time()
                end_memory = self._get_memory_usage()

                execution_time = (end_time - start_time) * 1000  # Convert to ms
                memory_used = end_memory - start_memory

                success = result.returncode == 0

                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="LUBM",
                    dataset_size=f"{university_count}-university",
                    operation="classification",
                    execution_time_ms=execution_time,
                    memory_usage_mb=memory_used,
                    success=success,
                    error_message=result.stderr if not success else "",
                    additional_metrics={
                        "stdout": result.stdout,
                        "exit_code": result.returncode,
                        "iteration": i + 1
                    }
                ))

            except subprocess.TimeoutExpired:
                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="LUBM",
                    dataset_size=f"{university_count}-university",
                    operation="classification",
                    execution_time_ms=300000,  # 5 minutes
                    success=False,
                    error_message="Timeout after 5 minutes",
                    additional_metrics={"iteration": i + 1}
                ))
            except Exception as e:
                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="LUBM",
                    dataset_size=f"{university_count}-university",
                    operation="classification",
                    success=False,
                    error_message=str(e),
                    additional_metrics={"iteration": i + 1}
                ))

        return results

    def _run_query_tests(self, reasoner_name: str, reasoner_config: dict,
                        ontology_file: Path, university_count: int, iterations: int) -> List[BenchmarkSuiteResult]:
        """Run LUBM query tests"""
        results = []

        # Get available queries
        query_files = list(self.query_dir.glob("query*.sparql"))
        if not query_files:
            return [BenchmarkSuiteResult(
                reasoner_name=reasoner_name,
                benchmark_suite="LUBM",
                dataset_size=f"{university_count}-university",
                operation="query",
                success=False,
                error_message="No query files found"
            )]

        for query_file in query_files:
            query_name = query_file.stem

            for i in range(iterations):
                try:
                    # For now, run classification as a proxy for query performance
                    # (since most reasoners don't have direct SPARQL support)
                    cmd = reasoner_config["classification_cmd"].split() + [str(ontology_file)]
                    working_dir = reasoner_config.get("working_dir", ".")

                    start_time = time.time()
                    start_memory = self._get_memory_usage()

                    result = subprocess.run(
                        cmd,
                        cwd=working_dir,
                        capture_output=True,
                        text=True,
                        timeout=300
                    )

                    end_time = time.time()
                    end_memory = self._get_memory_usage()

                    execution_time = (end_time - start_time) * 1000
                    memory_used = end_memory - start_memory

                    success = result.returncode == 0

                    results.append(BenchmarkSuiteResult(
                        reasoner_name=reasoner_name,
                        benchmark_suite="LUBM",
                        dataset_size=f"{university_count}-university",
                        operation="query",
                        query_name=query_name,
                        execution_time_ms=execution_time,
                        memory_usage_mb=memory_used,
                        success=success,
                        error_message=result.stderr if not success else "",
                        additional_metrics={
                            "stdout": result.stdout,
                            "exit_code": result.returncode,
                            "iteration": i + 1
                        }
                    ))

                except Exception as e:
                    results.append(BenchmarkSuiteResult(
                        reasoner_name=reasoner_name,
                        benchmark_suite="LUBM",
                        dataset_size=f"{university_count}-university",
                        operation="query",
                        query_name=query_name,
                        success=False,
                        error_message=str(e),
                        additional_metrics={"iteration": i + 1}
                    ))

        return results

    def _run_consistency_test(self, reasoner_name: str, reasoner_config: dict,
                             ontology_file: Path, university_count: int, iterations: int) -> List[BenchmarkSuiteResult]:
        """Run LUBM consistency test"""
        results = []

        for i in range(iterations):
            try:
                # Build command based on reasoner
                if reasoner_name == "rust_owl2":
                    cmd = reasoner_config["consistency_cmd"].split() + [str(ontology_file)]
                    working_dir = reasoner_config.get("working_dir", ".")
                elif reasoner_name == "elk":
                    cmd = reasoner_config["consistency_cmd"].split() + ["-i", str(ontology_file)]
                    working_dir = "."
                elif reasoner_name == "hermit":
                    cmd = reasoner_config["consistency_cmd"].split() + [str(ontology_file)]
                    working_dir = "."
                else:
                    cmd = reasoner_config["consistency_cmd"].split() + [str(ontology_file)]
                    working_dir = "."

                start_time = time.time()
                start_memory = self._get_memory_usage()

                result = subprocess.run(
                    cmd,
                    cwd=working_dir,
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                end_time = time.time()
                end_memory = self._get_memory_usage()

                execution_time = (end_time - start_time) * 1000
                memory_used = end_memory - start_memory

                success = result.returncode == 0

                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="LUBM",
                    dataset_size=f"{university_count}-university",
                    operation="consistency",
                    execution_time_ms=execution_time,
                    memory_usage_mb=memory_used,
                    success=success,
                    error_message=result.stderr if not success else "",
                    additional_metrics={
                        "stdout": result.stdout,
                        "exit_code": result.returncode,
                        "iteration": i + 1
                    }
                ))

            except Exception as e:
                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="LUBM",
                    dataset_size=f"{university_count}-university",
                    operation="consistency",
                    success=False,
                    error_message=str(e),
                    additional_metrics={"iteration": i + 1}
                ))

        return results

    def _get_memory_usage(self) -> float:
        """Get current memory usage in MB"""
        try:
            import psutil
            process = psutil.Process()
            return process.memory_info().rss / 1024 / 1024  # Convert to MB
        except ImportError:
            return 0.0

class SP2BBenchmark:
    """SP2B-specific benchmark implementation"""

    def __init__(self, sp2b_dir: Path):
        self.sp2b_dir = sp2b_dir
        self.data_dir = sp2b_dir / "data"
        self.query_dir = sp2b_dir / "queries"
        self.generator_dir = sp2b_dir / "generator"

    def run_sp2b_test(self, reasoner_name: str, reasoner_config: dict,
                     scale_factor: int, operation: str, iterations: int = 3) -> List[BenchmarkSuiteResult]:
        """Run SP2B benchmark test"""
        results = []

        # Find the appropriate dataset file
        dataset_file = self.data_dir / f"sp2b_scale_{scale_factor}.ttl"

        if not dataset_file.exists():
            # Try to generate it
            self._generate_sp2b_data(scale_factor)
            if not dataset_file.exists():
                return [BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="SP2B",
                    dataset_size=f"scale-{scale_factor}",
                    operation=operation,
                    success=False,
                    error_message=f"Dataset file not found: {dataset_file}"
                )]

        if operation == "classification":
            return self._run_classification_test(reasoner_name, reasoner_config, dataset_file, scale_factor, iterations)
        elif operation == "query":
            return self._run_query_tests(reasoner_name, reasoner_config, dataset_file, scale_factor, iterations)
        elif operation == "consistency":
            return self._run_consistency_test(reasoner_name, reasoner_config, dataset_file, scale_factor, iterations)
        else:
            return [BenchmarkSuiteResult(
                reasoner_name=reasoner_name,
                benchmark_suite="SP2B",
                dataset_size=f"scale-{scale_factor}",
                operation=operation,
                success=False,
                error_message=f"Unknown operation: {operation}"
            )]

    def _generate_sp2b_data(self, scale_factor: int):
        """Generate SP2B data if not exists"""
        generator_script = self.generator_dir / "sp2b_generator.py"
        if generator_script.exists():
            try:
                result = subprocess.run([
                    sys.executable, str(generator_script), str(scale_factor), str(self.data_dir)
                ], capture_output=True, text=True, timeout=300)
                if result.returncode != 0:
                    print(f"⚠️  SP2B generation failed: {result.stderr}")
            except subprocess.TimeoutExpired:
                print("⚠️  SP2B generation timed out")

    def _run_classification_test(self, reasoner_name: str, reasoner_config: dict,
                                dataset_file: Path, scale_factor: int, iterations: int) -> List[BenchmarkSuiteResult]:
        """Run SP2B classification test"""
        results = []

        for i in range(iterations):
            try:
                cmd = reasoner_config["classification_cmd"].split() + [str(dataset_file)]
                working_dir = reasoner_config.get("working_dir", ".")

                start_time = time.time()
                start_memory = self._get_memory_usage()

                result = subprocess.run(
                    cmd,
                    cwd=working_dir,
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                end_time = time.time()
                end_memory = self._get_memory_usage()

                execution_time = (end_time - start_time) * 1000
                memory_used = end_memory - start_memory

                success = result.returncode == 0

                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="SP2B",
                    dataset_size=f"scale-{scale_factor}",
                    operation="classification",
                    execution_time_ms=execution_time,
                    memory_usage_mb=memory_used,
                    success=success,
                    error_message=result.stderr if not success else "",
                    additional_metrics={
                        "stdout": result.stdout,
                        "exit_code": result.returncode,
                        "iteration": i + 1
                    }
                ))

            except Exception as e:
                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="SP2B",
                    dataset_size=f"scale-{scale_factor}",
                    operation="classification",
                    success=False,
                    error_message=str(e),
                    additional_metrics={"iteration": i + 1}
                ))

        return results

    def _run_query_tests(self, reasoner_name: str, reasoner_config: dict,
                        dataset_file: Path, scale_factor: int, iterations: int) -> List[BenchmarkSuiteResult]:
        """Run SP2B query tests"""
        results = []

        # Get available queries
        query_files = list(self.query_dir.glob("sp2b_query_*.sparql"))
        if not query_files:
            return [BenchmarkSuiteResult(
                reasoner_name=reasoner_name,
                benchmark_suite="SP2B",
                dataset_size=f"scale-{scale_factor}",
                operation="query",
                success=False,
                error_message="No query files found"
            )]

        for query_file in query_files:
            query_name = query_file.stem

            for i in range(iterations):
                try:
                    # Run classification as a proxy for query performance
                    cmd = reasoner_config["classification_cmd"].split() + [str(dataset_file)]
                    working_dir = reasoner_config.get("working_dir", ".")

                    start_time = time.time()
                    start_memory = self._get_memory_usage()

                    result = subprocess.run(
                        cmd,
                        cwd=working_dir,
                        capture_output=True,
                        text=True,
                        timeout=300
                    )

                    end_time = time.time()
                    end_memory = self._get_memory_usage()

                    execution_time = (end_time - start_time) * 1000
                    memory_used = end_memory - start_memory

                    success = result.returncode == 0

                    results.append(BenchmarkSuiteResult(
                        reasoner_name=reasoner_name,
                        benchmark_suite="SP2B",
                        dataset_size=f"scale-{scale_factor}",
                        operation="query",
                        query_name=query_name,
                        execution_time_ms=execution_time,
                        memory_usage_mb=memory_used,
                        success=success,
                        error_message=result.stderr if not success else "",
                        additional_metrics={
                            "stdout": result.stdout,
                            "exit_code": result.returncode,
                            "iteration": i + 1
                        }
                    ))

                except Exception as e:
                    results.append(BenchmarkSuiteResult(
                        reasoner_name=reasoner_name,
                        benchmark_suite="SP2B",
                        dataset_size=f"scale-{scale_factor}",
                        operation="query",
                        query_name=query_name,
                        success=False,
                        error_message=str(e),
                        additional_metrics={"iteration": i + 1}
                    ))

        return results

    def _run_consistency_test(self, reasoner_name: str, reasoner_config: dict,
                             dataset_file: Path, scale_factor: int, iterations: int) -> List[BenchmarkSuiteResult]:
        """Run SP2B consistency test"""
        results = []

        for i in range(iterations):
            try:
                cmd = reasoner_config["consistency_cmd"].split() + [str(dataset_file)]
                working_dir = reasoner_config.get("working_dir", ".")

                start_time = time.time()
                start_memory = self._get_memory_usage()

                result = subprocess.run(
                    cmd,
                    cwd=working_dir,
                    capture_output=True,
                    text=True,
                    timeout=300
                )

                end_time = time.time()
                end_memory = self._get_memory_usage()

                execution_time = (end_time - start_time) * 1000
                memory_used = end_memory - start_memory

                success = result.returncode == 0

                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="SP2B",
                    dataset_size=f"scale-{scale_factor}",
                    operation="consistency",
                    execution_time_ms=execution_time,
                    memory_usage_mb=memory_used,
                    success=success,
                    error_message=result.stderr if not success else "",
                    additional_metrics={
                        "stdout": result.stdout,
                        "exit_code": result.returncode,
                        "iteration": i + 1
                    }
                ))

            except Exception as e:
                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="SP2B",
                    dataset_size=f"scale-{scale_factor}",
                    operation="consistency",
                    success=False,
                    error_message=str(e),
                    additional_metrics={"iteration": i + 1}
                ))

        return results

    def _get_memory_usage(self) -> float:
        """Get current memory usage in MB"""
        try:
            import psutil
            process = psutil.Process()
            return process.memory_info().rss / 1024 / 1024  # Convert to MB
        except ImportError:
            return 0.0

class ScalabilityBenchmark:
    """Scalability testing implementation"""

    def __init__(self, scalability_dir: Path):
        self.scalability_dir = scalability_dir
        self.data_dir = scalability_dir / "ontologies"

    def run_scalability_test(self, reasoner_name: str, reasoner_config: dict,
                           scale_name: str, operation: str, iterations: int = 3) -> List[BenchmarkSuiteResult]:
        """Run scalability test"""
        results = []

        # Find the appropriate ontology file
        ontology_file = self.data_dir / f"scalability_{scale_name}.owl"

        if not ontology_file.exists():
            return [BenchmarkSuiteResult(
                reasoner_name=reasoner_name,
                benchmark_suite="SCALABILITY",
                dataset_size=scale_name,
                operation=operation,
                success=False,
                error_message=f"Ontology file not found: {ontology_file}"
            )]

        if operation == "classification":
            return self._run_classification_test(reasoner_name, reasoner_config, ontology_file, scale_name, iterations)
        elif operation == "consistency":
            return self._run_consistency_test(reasoner_name, reasoner_config, ontology_file, scale_name, iterations)
        else:
            return [BenchmarkSuiteResult(
                reasoner_name=reasoner_name,
                benchmark_suite="SCALABILITY",
                dataset_size=scale_name,
                operation=operation,
                success=False,
                error_message=f"Unknown operation: {operation}"
            )]

    def _run_classification_test(self, reasoner_name: str, reasoner_config: dict,
                                ontology_file: Path, scale_name: str, iterations: int) -> List[BenchmarkSuiteResult]:
        """Run scalability classification test"""
        results = []

        for i in range(iterations):
            try:
                cmd = reasoner_config["classification_cmd"].split() + [str(ontology_file)]
                working_dir = reasoner_config.get("working_dir", ".")

                start_time = time.time()
                start_memory = self._get_memory_usage()

                result = subprocess.run(
                    cmd,
                    cwd=working_dir,
                    capture_output=True,
                    text=True,
                    timeout=600  # Longer timeout for scalability tests
                )

                end_time = time.time()
                end_memory = self._get_memory_usage()

                execution_time = (end_time - start_time) * 1000
                memory_used = end_memory - start_memory

                success = result.returncode == 0

                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="SCALABILITY",
                    dataset_size=scale_name,
                    operation="classification",
                    execution_time_ms=execution_time,
                    memory_usage_mb=memory_used,
                    success=success,
                    error_message=result.stderr if not success else "",
                    additional_metrics={
                        "stdout": result.stdout,
                        "exit_code": result.returncode,
                        "iteration": i + 1
                    }
                ))

            except subprocess.TimeoutExpired:
                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="SCALABILITY",
                    dataset_size=scale_name,
                    operation="classification",
                    execution_time_ms=600000,  # 10 minutes
                    success=False,
                    error_message="Timeout after 10 minutes",
                    additional_metrics={"iteration": i + 1}
                ))
            except Exception as e:
                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="SCALABILITY",
                    dataset_size=scale_name,
                    operation="classification",
                    success=False,
                    error_message=str(e),
                    additional_metrics={"iteration": i + 1}
                ))

        return results

    def _run_consistency_test(self, reasoner_name: str, reasoner_config: dict,
                             ontology_file: Path, scale_name: str, iterations: int) -> List[BenchmarkSuiteResult]:
        """Run scalability consistency test"""
        results = []

        for i in range(iterations):
            try:
                cmd = reasoner_config["consistency_cmd"].split() + [str(ontology_file)]
                working_dir = reasoner_config.get("working_dir", ".")

                start_time = time.time()
                start_memory = self._get_memory_usage()

                result = subprocess.run(
                    cmd,
                    cwd=working_dir,
                    capture_output=True,
                    text=True,
                    timeout=600
                )

                end_time = time.time()
                end_memory = self._get_memory_usage()

                execution_time = (end_time - start_time) * 1000
                memory_used = end_memory - start_memory

                success = result.returncode == 0

                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="SCALABILITY",
                    dataset_size=scale_name,
                    operation="consistency",
                    execution_time_ms=execution_time,
                    memory_usage_mb=memory_used,
                    success=success,
                    error_message=result.stderr if not success else "",
                    additional_metrics={
                        "stdout": result.stdout,
                        "exit_code": result.returncode,
                        "iteration": i + 1
                    }
                ))

            except Exception as e:
                results.append(BenchmarkSuiteResult(
                    reasoner_name=reasoner_name,
                    benchmark_suite="SCALABILITY",
                    dataset_size=scale_name,
                    operation="consistency",
                    success=False,
                    error_message=str(e),
                    additional_metrics={"iteration": i + 1}
                ))

        return results

    def _get_memory_usage(self) -> float:
        """Get current memory usage in MB"""
        try:
            import psutil
            process = psutil.Process()
            return process.memory_info().rss / 1024 / 1024  # Convert to MB
        except ImportError:
            return 0.0

class BenchmarkAnalytics:
    """Analytics for multi-benchmark results"""

    def calculate_performance_scores(self, results: List[BenchmarkSuiteResult]) -> Dict[str, BenchmarkComparison]:
        """Calculate comprehensive performance scores"""
        # Group results by reasoner
        reasoner_results = {}
        for result in results:
            if result.reasoner_name not in reasoner_results:
                reasoner_results[result.reasoner_name] = []
            reasoner_results[result.reasoner_name].append(result)

        # Calculate scores for each reasoner
        comparisons = {}
        for reasoner_name, reasoner_result_list in reasoner_results.items():
            # Calculate scores for each benchmark suite
            lubm_score = self._calculate_suite_score(reasoner_result_list, "LUBM")
            sp2b_score = self._calculate_suite_score(reasoner_result_list, "SP2B")
            custom_score = self._calculate_suite_score(reasoner_result_list, "CUSTOM")
            scalability_score = self._calculate_suite_score(reasoner_result_list, "SCALABILITY")

            # Overall weighted score
            overall_score = (
                lubm_score * 0.4 +  # LUBM is most important for OWL reasoning
                sp2b_score * 0.3 +  # SP2B tests query performance
                custom_score * 0.2 +  # Custom ontologies test general performance
                scalability_score * 0.1  # Scalability tests
            )

            comparisons[reasoner_name] = BenchmarkComparison(
                reasoner_name=reasoner_name,
                overall_rank=0,  # Will be calculated after all scores
                lubm_rank=0,
                sp2b_rank=0,
                custom_rank=0,
                scalability_rank=0,
                performance_score=overall_score,
                scalability_score=self._calculate_scalability_score(reasoner_result_list),
                robustness_score=self._calculate_robustness_score(reasoner_result_list)
            )

        # Calculate ranks
        self._calculate_ranks(comparisons)

        return comparisons

    def _calculate_suite_score(self, results: List[BenchmarkSuiteResult], suite: str) -> float:
        """Calculate score for a specific benchmark suite"""
        suite_results = [r for r in results if r.benchmark_suite == suite and r.success]
        if not suite_results:
            return 0.0

        # Normalize execution times (lower is better)
        times = [r.execution_time_ms for r in suite_results]
        avg_time = sum(times) / len(times)

        # Score based on inverse of time (log scale to handle large variations)
        return 100.0 / (1.0 + math.log10(avg_time + 1))

    def _calculate_scalability_score(self, results: List[BenchmarkSuiteResult]) -> float:
        """Calculate scalability score based on performance across different scales"""
        scalability_results = [r for r in results if r.benchmark_suite == "SCALABILITY" and r.success]
        if not scalability_results:
            return 0.0

        # Group by scale
        scale_times = {}
        for result in scalability_results:
            scale = result.dataset_size
            if scale not in scale_times:
                scale_times[scale] = []
            scale_times[scale].append(result.execution_time_ms)

        # Calculate how well performance scales (lower degradation is better)
        if len(scale_times) < 2:
            return 50.0  # Default score for single scale

        # Calculate degradation ratio
        scales = sorted(scale_times.keys())
        avg_times = [sum(scale_times[scale]) / len(scale_times[scale]) for scale in scales]

        # Calculate performance degradation
        degradation_ratios = []
        for i in range(1, len(avg_times)):
            if avg_times[i-1] > 0:
                degradation_ratio = avg_times[i] / avg_times[i-1]
                degradation_ratios.append(degradation_ratio)

        if not degradation_ratios:
            return 50.0

        avg_degradation = sum(degradation_ratios) / len(degradation_ratios)

        # Score based on degradation (lower is better)
        return max(0, 100 - (avg_degradation - 1) * 50)

    def _calculate_robustness_score(self, results: List[BenchmarkSuiteResult]) -> float:
        """Calculate robustness score based on success rate"""
        if not results:
            return 0.0

        successful_results = [r for r in results if r.success]
        success_rate = len(successful_results) / len(results)

        return success_rate * 100.0

    def _calculate_ranks(self, comparisons: Dict[str, BenchmarkComparison]):
        """Calculate ranks for each metric"""
        reasoners = list(comparisons.keys())

        # Overall rank
        sorted_by_overall = sorted(reasoners, key=lambda x: comparisons[x].performance_score, reverse=True)
        for rank, reasoner in enumerate(sorted_by_overall, 1):
            comparisons[reasoner].overall_rank = rank

        # LUBM rank
        sorted_by_lubm = sorted(reasoners, key=lambda x: self._calculate_suite_score([r for r in reasoners[x] if r.benchmark_suite == "LUBM"], "LUBM"), reverse=True)
        for rank, reasoner in enumerate(sorted_by_lubm, 1):
            comparisons[reasoner].lubm_rank = rank

        # SP2B rank
        sorted_by_sp2b = sorted(reasoners, key=lambda x: self._calculate_suite_score([r for r in reasoners[x] if r.benchmark_suite == "SP2B"], "SP2B"), reverse=True)
        for rank, reasoner in enumerate(sorted_by_sp2b, 1):
            comparisons[reasoner].sp2b_rank = rank

        # Custom rank
        sorted_by_custom = sorted(reasoners, key=lambda x: self._calculate_suite_score([r for r in reasoners[x] if r.benchmark_suite == "CUSTOM"], "CUSTOM"), reverse=True)
        for rank, reasoner in enumerate(sorted_by_custom, 1):
            comparisons[reasoner].custom_rank = rank

        # Scalability rank
        sorted_by_scalability = sorted(reasoners, key=lambda x: self._calculate_suite_score([r for r in reasoners[x] if r.benchmark_suite == "SCALABILITY"], "SCALABILITY"), reverse=True)
        for rank, reasoner in enumerate(sorted_by_scalability, 1):
            comparisons[reasoner].scalability_rank = rank

class EnhancedBenchmarkFramework:
    """Enhanced benchmark framework with LUBM and SP2B support"""

    def __init__(self, config_file: str = "benchmarking/config.json"):
        self.config_file = Path(config_file)
        self.config = self._load_config()

        # Initialize directories
        self.results_dir = Path(self.config["output"]["results_dir"])
        self.results_dir.mkdir(exist_ok=True)

        # Initialize benchmark suites
        benchmarks_dir = Path(config_file).parent / "benchmarks"
        self.lubm_benchmark = LUBMBenchmark(benchmarks_dir / "lubm")
        self.sp2b_benchmark = SP2BBenchmark(benchmarks_dir / "sp2b")
        self.scalability_benchmark = ScalabilityBenchmark(benchmarks_dir / "scalability")

        # Initialize analytics
        self.analytics = BenchmarkAnalytics()

    def _load_config(self) -> dict:
        """Load configuration file"""
        if not self.config_file.exists():
            raise FileNotFoundError(f"Configuration file not found: {self.config_file}")

        with open(self.config_file, 'r') as f:
            return json.load(f)

    def run_comprehensive_benchmark(self, reasoners: List[str] = None, iterations: int = 3) -> List[BenchmarkSuiteResult]:
        """Run comprehensive benchmarks across all reasoners and benchmark suites"""
        all_results = []

        # Filter reasoners if specified
        if reasoners is None:
            reasoners = list(self.config["reasoners"].keys())

        print(f"🚀 Running comprehensive benchmark with {iterations} iterations...")
        print(f"📊 Reasoners: {reasoners}")

        for reasoner_name in reasoners:
            print(f"\n🔬 Benchmarking {reasoner_name}...")
            reasoner_config = self.config["reasoners"][reasoner_name]

            # Run LUBM benchmarks
            if self.config["benchmarks"]["lubm"]["enabled"]:
                print("   📊 Running LUBM benchmarks...")
                lubm_results = self._run_lubm_benchmarks(reasoner_name, reasoner_config, iterations)
                all_results.extend(lubm_results)

            # Run SP2B benchmarks
            if self.config["benchmarks"]["sp2b"]["enabled"]:
                print("   📊 Running SP2B benchmarks...")
                sp2b_results = self._run_sp2b_benchmarks(reasoner_name, reasoner_config, iterations)
                all_results.extend(sp2b_results)

            # Run scalability benchmarks
            if self.config["benchmarks"]["scalability"]["enabled"]:
                print("   📊 Running scalability benchmarks...")
                scalability_results = self._run_scalability_benchmarks(reasoner_name, reasoner_config, iterations)
                all_results.extend(scalability_results)

        return all_results

    def _run_lubm_benchmarks(self, reasoner_name: str, reasoner_config: dict, iterations: int) -> List[BenchmarkSuiteResult]:
        """Run LUBM benchmarks"""
        results = []
        lubm_config = self.config["benchmarks"]["lubm"]

        for university_count in lubm_config["university_counts"]:
            print(f"      🏫 Testing {university_count} university dataset...")

            # Classification test
            class_results = self.lubm_benchmark.run_lubm_test(
                reasoner_name, reasoner_config, university_count, "classification", iterations
            )
            results.extend(class_results)

            # Consistency test
            cons_results = self.lubm_benchmark.run_lubm_test(
                reasoner_name, reasoner_config, university_count, "consistency", iterations
            )
            results.extend(cons_results)

            # Query tests
            query_results = self.lubm_benchmark.run_lubm_test(
                reasoner_name, reasoner_config, university_count, "query", iterations
            )
            results.extend(query_results)

        return results

    def _run_sp2b_benchmarks(self, reasoner_name: str, reasoner_config: dict, iterations: int) -> List[BenchmarkSuiteResult]:
        """Run SP2B benchmarks"""
        results = []
        sp2b_config = self.config["benchmarks"]["sp2b"]

        for scale_factor in sp2b_config["scale_factors"]:
            print(f"      📈 Testing scale {scale_factor} dataset...")

            # Classification test
            class_results = self.sp2b_benchmark.run_sp2b_test(
                reasoner_name, reasoner_config, scale_factor, "classification", iterations
            )
            results.extend(class_results)

            # Consistency test
            cons_results = self.sp2b_benchmark.run_sp2b_test(
                reasoner_name, reasoner_config, scale_factor, "consistency", iterations
            )
            results.extend(cons_results)

            # Query tests
            query_results = self.sp2b_benchmark.run_sp2b_test(
                reasoner_name, reasoner_config, scale_factor, "query", iterations
            )
            results.extend(query_results)

        return results

    def _run_scalability_benchmarks(self, reasoner_name: str, reasoner_config: dict, iterations: int) -> List[BenchmarkSuiteResult]:
        """Run scalability benchmarks"""
        results = []
        scalability_config = self.config["benchmarks"]["scalability"]

        for scale_name in scalability_config["scales"]:
            print(f"      📊 Testing {scale_name} scale dataset...")

            # Classification test
            class_results = self.scalability_benchmark.run_scalability_test(
                reasoner_name, reasoner_config, scale_name, "classification", iterations
            )
            results.extend(class_results)

            # Consistency test
            cons_results = self.scalability_benchmark.run_scalability_test(
                reasoner_name, reasoner_config, scale_name, "consistency", iterations
            )
            results.extend(cons_results)

        return results

    def generate_reports(self, results: List[BenchmarkSuiteResult]):
        """Generate comprehensive benchmark reports"""
        print("📊 Generating benchmark reports...")

        # Calculate performance scores
        comparisons = self.analytics.calculate_performance_scores(results)

        # Generate markdown report
        markdown_report = self._generate_markdown_report(results, comparisons)
        markdown_file = self.results_dir / f"comprehensive_report_{int(time.time())}.md"
        with open(markdown_file, 'w') as f:
            f.write(markdown_report)

        # Generate JSON report
        json_data = {
            "metadata": {
                "generated_at": time.strftime('%Y-%m-%d %H:%M:%S'),
                "total_results": len(results),
                "reasoners_tested": list(set(r.reasoner_name for r in results))
            },
            "results": [asdict(result) for result in results],
            "comparisons": {name: asdict(comp) for name, comp in comparisons.items()}
        }

        json_file = self.results_dir / f"comprehensive_results_{int(time.time())}.json"
        with open(json_file, 'w') as f:
            json.dump(json_data, f, indent=2)

        print(f"✅ Reports generated:")
        print(f"   📄 Markdown report: {markdown_file}")
        print(f"   📊 JSON data: {json_file}")

        # Print summary
        self._print_summary(comparisons)

    def _generate_markdown_report(self, results: List[BenchmarkSuiteResult], comparisons: Dict[str, BenchmarkComparison]) -> str:
        """Generate comprehensive markdown report"""
        report = []
        report.append("# Comprehensive OWL2 Reasoner Benchmark Report")
        report.append(f"Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}")
        report.append("")

        # Executive summary
        report.append("## Executive Summary")
        report.append("")
        sorted_reasoners = sorted(comparisons.values(), key=lambda x: x.overall_rank)
        for i, comparison in enumerate(sorted_reasoners, 1):
            report.append(f"{i}. **{comparison.reasoner_name}**")
            report.append(f"   - Overall Score: {comparison.performance_score:.1f}")
            report.append(f"   - LUBM Rank: {comparison.lubm_rank}")
            report.append(f"   - SP2B Rank: {comparison.sp2b_rank}")
            report.append(f"   - Scalability Score: {comparison.scalability_score:.1f}")
            report.append(f"   - Robustness Score: {comparison.robustness_score:.1f}")
            report.append("")

        # LUBM results
        report.append("## LUBM Benchmark Results")
        report.append("")
        lubm_results = [r for r in results if r.benchmark_suite == "LUBM"]
        self._add_suite_results_section(report, lubm_results, "LUBM")

        # SP2B results
        report.append("## SP2B Benchmark Results")
        report.append("")
        sp2b_results = [r for r in results if r.benchmark_suite == "SP2B"]
        self._add_suite_results_section(report, sp2b_results, "SP2B")

        # Scalability results
        report.append("## Scalability Benchmark Results")
        report.append("")
        scalability_results = [r for r in results if r.benchmark_suite == "SCALABILITY"]
        self._add_suite_results_section(report, scalability_results, "Scalability")

        return "\n".join(report)

    def _add_suite_results_section(self, report: List[str], results: List[BenchmarkSuiteResult], suite_name: str):
        """Add results section for a benchmark suite"""
        if not results:
            report.append(f"No results for {suite_name} benchmark.")
            return

        # Group by reasoner and operation
        reasoner_ops = {}
        for result in results:
            key = f"{result.reasoner_name}_{result.operation}"
            if key not in reasoner_ops:
                reasoner_ops[key] = []
            reasoner_ops[key].append(result)

        # Create summary table
        report.append("### Performance Summary")
        report.append("")
        report.append("| Reasoner | Operation | Avg Time (ms) | Success Rate |")
        report.append("|----------|-----------|---------------|--------------|")

        for key, op_results in reasoner_ops.items():
            reasoner_name, operation = key.split('_', 1)
            successful_results = [r for r in op_results if r.success]
            avg_time = sum(r.execution_time_ms for r in successful_results) / len(successful_results) if successful_results else 0
            success_rate = len(successful_results) / len(op_results) * 100

            report.append(f"| {reasoner_name} | {operation} | {avg_time:.1f} | {success_rate:.1f}% |")

        report.append("")

    def _print_summary(self, comparisons: Dict[str, BenchmarkComparison]):
        """Print summary of results"""
        print("\n" + "=" * 60)
        print("🏆 BENCHMARK RESULTS SUMMARY")
        print("=" * 60)

        sorted_reasoners = sorted(comparisons.values(), key=lambda x: x.overall_rank)

        for i, comparison in enumerate(sorted_reasoners, 1):
            print(f"{i}. {comparison.reasoner_name}")
            print(f"   Overall Score: {comparison.performance_score:.1f}")
            print(f"   LUBM Rank: {comparison.lubm_rank}")
            print(f"   SP2B Rank: {comparison.sp2b_rank}")
            print(f"   Scalability: {comparison.scalability_score:.1f}")
            print(f"   Robustness: {comparison.robustness_score:.1f}")
            print()

def main():
    """Main function"""
    parser = argparse.ArgumentParser(description="Enhanced OWL2 Reasoner Benchmark Framework")
    parser.add_argument("--config", default="benchmarking/config.json", help="Configuration file path")
    parser.add_argument("--reasoners", nargs="+", help="Specific reasoners to benchmark")
    parser.add_argument("--iterations", type=int, default=3, help="Number of iterations")
    parser.add_argument("--setup", action="store_true", help="Run setup first")

    args = parser.parse_args()

    # Run setup if requested
    if args.setup:
        print("🔧 Running benchmark setup...")
        setup_script = Path(args.config).parent / "setup_benchmarks.py"
        if setup_script.exists():
            result = subprocess.run([sys.executable, str(setup_script)], capture_output=True, text=True)
            if result.returncode == 0:
                print("✅ Setup completed successfully")
            else:
                print(f"❌ Setup failed: {result.stderr}")
                sys.exit(1)
        else:
            print(f"❌ Setup script not found: {setup_script}")
            sys.exit(1)

    # Initialize framework
    try:
        framework = EnhancedBenchmarkFramework(args.config)
    except FileNotFoundError as e:
        print(f"❌ Configuration error: {e}")
        print("   Run with --setup to create configuration")
        sys.exit(1)

    # Run comprehensive benchmark
    results = framework.run_comprehensive_benchmark(args.reasoners, args.iterations)

    # Generate reports
    framework.generate_reports(results)

    print("✅ Enhanced benchmark completed!")

if __name__ == "__main__":
    main()