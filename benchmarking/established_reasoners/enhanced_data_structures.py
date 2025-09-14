#!/usr/bin/env python3

"""
Enhanced Data Structures for Publication-Ready OWL2 Reasoner Testing
Implements academic-grade metrics and statistical analysis capabilities
"""

import statistics
import json
from typing import Dict, List, Tuple, Optional, Any, Union
from dataclasses import dataclass, asdict
from datetime import datetime
from enum import Enum
import math
from pathlib import Path

# Import our new components
from memory_profiler import MemoryMetrics, MemoryAnalysisEngine
from environment_collector import EnvironmentSpecification

class BenchmarkType(Enum):
    """Types of benchmarks for publication-ready testing"""
    CUSTOM = "custom"
    LUBM = "lubm"
    SP2B = "sp2b"
    SCALABILITY = "scalability"
    REAL_WORLD = "real_world"

class ReasonerType(Enum):
    """Types of reasoners for proper categorization"""
    CLI_TOOL = "cli_tool"
    LIBRARY = "library"
    FRAMEWORK = "framework"

class TestOperation(Enum):
    """Standardized test operations for consistency"""
    CLASSIFICATION = "classification"
    CONSISTENCY = "consistency"
    QUERY_ANSWERING = "query_answering"
    HELP_SYSTEM = "help_system"
    MEMORY_PROFILING = "memory_profiling"

@dataclass
class PublicationTestResult:
    """Enhanced test result with publication-grade metrics"""
    # Basic identification
    reasoner_name: str
    reasoner_type: ReasonerType
    benchmark_type: BenchmarkType
    test_operation: TestOperation
    ontology_file: str
    ontology_format: str
    test_timestamp: str

    # Success and execution
    success: bool
    execution_time_ms: float
    return_code: int
    timeout_occurred: bool

    # Output metrics
    output_file: Optional[str] = None
    output_size_bytes: Optional[int] = None
    output_lines: Optional[int] = None
    error_message: Optional[str] = None
    warning_count: int = 0

    # Memory profiling (new)
    memory_metrics: Optional[MemoryMetrics] = None
    memory_efficiency_score: float = 0.0
    memory_stability_score: float = 0.0
    memory_scalability_score: float = 0.0

    # Ontology characteristics
    triples_count: int = 0
    axioms_count: int = 0
    classes_count: int = 0
    properties_count: int = 0
    individuals_count: int = 0

    # Performance metrics
    throughput_triples_per_second: float = 0.0
    throughput_axioms_per_second: float = 0.0
    time_per_triple_microseconds: float = 0.0
    time_per_axiom_microseconds: float = 0.0

    # Statistical analysis
    statistical_significance: float = 0.0
    confidence_interval_95: Optional[Tuple[float, float]] = None
    outlier_detected: bool = False
    coefficient_of_variation: float = 0.0

    # Benchmark-specific metadata
    benchmark_metadata: Optional[Dict[str, Any]] = None

    def __post_init__(self):
        """Calculate derived metrics after initialization"""
        self._calculate_derived_metrics()
        self._calculate_memory_scores()

    def _calculate_derived_metrics(self):
        """Calculate performance metrics from basic data"""
        if self.execution_time_ms > 0 and self.triples_count > 0:
            self.throughput_triples_per_second = (self.triples_count / (self.execution_time_ms / 1000.0))
            self.time_per_triple_microseconds = (self.execution_time_ms * 1000.0) / self.triples_count

        if self.execution_time_ms > 0 and self.axioms_count > 0:
            self.throughput_axioms_per_second = (self.axioms_count / (self.execution_time_ms / 1000.0))
            self.time_per_axiom_microseconds = (self.execution_time_ms * 1000.0) / self.axioms_count

    def _calculate_memory_scores(self):
        """Calculate memory performance scores"""
        if self.memory_metrics:
            analysis = MemoryAnalysisEngine.analyze_efficiency(self.memory_metrics)
            self.memory_efficiency_score = analysis.get('memory_efficiency_triples_per_mb', 0.0)
            self.memory_stability_score = analysis.get('memory_stability', 0.0)
            self.memory_scalability_score = analysis.get('scalability_score', 0.0)

@dataclass
class BenchmarkSuite:
    """Complete benchmark suite with multiple tests and statistical analysis"""
    suite_name: str
    benchmark_type: BenchmarkType
    description: str
    version: str
    test_results: List[PublicationTestResult]
    environment_spec: Optional[EnvironmentSpecification] = None
    collection_timestamp: str = ""
    statistical_summary: Optional[Dict[str, Any]] = None

    def __post_init__(self):
        """Generate statistical summary after initialization"""
        if not self.collection_timestamp:
            self.collection_timestamp = datetime.now().isoformat()
        self._generate_statistical_summary()

    def _generate_statistical_summary(self):
        """Generate comprehensive statistical summary"""
        if not self.test_results:
            return

        # Group by reasoner and operation
        grouped_results = {}
        for result in self.test_results:
            key = (result.reasoner_name, result.test_operation.value)
            if key not in grouped_results:
                grouped_results[key] = []
            grouped_results[key].append(result)

        summary = {
            'total_tests': len(self.test_results),
            'successful_tests': len([r for r in self.test_results if r.success]),
            'failed_tests': len([r for r in self.test_results if not r.success]),
            'timeout_tests': len([r for r in self.test_results if r.timeout_occurred]),
            'performance_by_reasoner': {},
            'efficiency_analysis': {},
            'comparative_analysis': {}
        }

        # Analyze each reasoner-operation combination
        for (reasoner, operation), results in grouped_results.items():
            if reasoner not in summary['performance_by_reasoner']:
                summary['performance_by_reasoner'][reasoner] = {}

            successful_results = [r for r in results if r.success and not r.timeout_occurred]
            if successful_results:
                execution_times = [r.execution_time_ms for r in successful_results]

                summary['performance_by_reasoner'][reasoner][operation] = {
                    'count': len(successful_results),
                    'mean_time_ms': statistics.mean(execution_times),
                    'median_time_ms': statistics.median(execution_times),
                    'std_dev_ms': statistics.stdev(execution_times) if len(execution_times) > 1 else 0,
                    'min_time_ms': min(execution_times),
                    'max_time_ms': max(execution_times),
                    'coefficient_of_variation': statistics.stdev(execution_times) / statistics.mean(execution_times) if len(execution_times) > 1 else 0
                }

                # Memory analysis if available
                memory_results = [r for r in successful_results if r.memory_metrics]
                if memory_results:
                    peak_memories = [r.memory_metrics.peak_memory_mb for r in memory_results]
                    summary['performance_by_reasoner'][reasoner][operation]['memory_analysis'] = {
                        'mean_peak_memory_mb': statistics.mean(peak_memories),
                        'mean_efficiency_score': statistics.mean([r.memory_efficiency_score for r in memory_results]),
                        'mean_stability_score': statistics.mean([r.memory_stability_score for r in memory_results])
                    }

        self.statistical_summary = summary

    def get_top_performers(self, metric: str = 'mean_time_ms', operation: Optional[str] = None) -> List[Tuple[str, float]]:
        """Get top performing reasoners by metric"""
        performers = []

        for reasoner, operations in self.statistical_summary['performance_by_reasoner'].items():
            if operation:
                if operation in operations:
                    value = operations[operation].get(metric, float('inf'))
                    performers.append((reasoner, value))
            else:
                # Average across all operations
                values = []
                for op_data in operations.values():
                    if metric in op_data:
                        values.append(op_data[metric])
                if values:
                    performers.append((reasoner, statistics.mean(values)))

        # Sort by value (lower is better for time metrics)
        performers.sort(key=lambda x: x[1])
        return performers

    def generate_performance_scores(self) -> Dict[str, float]:
        """Generate overall performance scores for each reasoner"""
        scores = {}

        for reasoner in self.statistical_summary['performance_by_reasoner'].keys():
            performance_score = 0.0
            efficiency_score = 0.0
            reliability_score = 0.0

            operations = self.statistical_summary['performance_by_reasoner'][reasoner]

            # Performance score (inverse of execution time)
            time_scores = []
            for op_data in operations.values():
                if 'mean_time_ms' in op_data:
                    # Convert time to score (lower time = higher score)
                    time_score = 1000.0 / (op_data['mean_time_ms'] + 1.0)
                    time_scores.append(time_score)

            if time_scores:
                performance_score = statistics.mean(time_scores)

            # Efficiency score (memory + throughput)
            efficiency_scores = []
            for op_data in operations.values():
                if 'memory_analysis' in op_data:
                    mem_analysis = op_data['memory_analysis']
                    efficiency = mem_analysis.get('mean_efficiency_score', 0.0)
                    efficiency_scores.append(efficiency)

            if efficiency_scores:
                efficiency_score = statistics.mean(efficiency_scores) / 100.0  # Normalize

            # Reliability score (success rate)
            total_tests = len([r for r in self.test_results if r.reasoner_name == reasoner])
            successful_tests = len([r for r in self.test_results if r.reasoner_name == reasoner and r.success])
            if total_tests > 0:
                reliability_score = successful_tests / total_tests

            # Overall score (weighted average)
            overall_score = (performance_score * 0.4 + efficiency_score * 0.3 + reliability_score * 0.3)
            scores[reasoner] = overall_score

        return scores

@dataclass
class ComparativeAnalysisResult:
    """Result of comparative analysis between reasoners"""
    reasoner_comparison: Dict[str, Dict[str, Any]]
    performance_ranking: List[Tuple[str, float]]
    efficiency_ranking: List[Tuple[str, float]]
    reliability_ranking: List[Tuple[str, float]]
    overall_ranking: List[Tuple[str, float]]
    statistical_significance: Dict[str, float]
    recommendations: List[str]
    key_findings: List[str]

class EnhancedDataAnalyzer:
    """Advanced data analysis for publication-ready results"""

    def __init__(self):
        self.analysis_methods = {
            'performance': self._analyze_performance,
            'efficiency': self._analyze_efficiency,
            'reliability': self._analyze_reliability,
            'scalability': self._analyze_scalability,
            'comparative': self._perform_comparative_analysis
        }

    def analyze_benchmark_suite(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Perform comprehensive analysis of benchmark suite"""
        analysis = {}

        for method_name, method_func in self.analysis_methods.items():
            try:
                analysis[method_name] = method_func(suite)
            except Exception as e:
                print(f"Analysis error in {method_name}: {e}")
                analysis[method_name] = {'error': str(e)}

        return analysis

    def _analyze_performance(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Analyze performance metrics"""
        return {
            'top_performers': suite.get_top_performers('mean_time_ms'),
            'slowest_reasoners': suite.get_top_performers('mean_time_ms')[::-1],
            'consistency_analysis': self._analyze_consistency(suite),
            'outlier_analysis': self._detect_outliers(suite)
        }

    def _analyze_efficiency(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Analyze efficiency metrics"""
        efficiency_data = {}

        for reasoner, operations in suite.statistical_summary['performance_by_reasoner'].items():
            memory_scores = []
            throughput_scores = []

            for op_data in operations.values():
                if 'memory_analysis' in op_data:
                    mem_analysis = op_data['memory_analysis']
                    memory_scores.append(mem_analysis.get('mean_efficiency_score', 0.0))

                # Calculate throughput from execution time and ontology size
                if 'mean_time_ms' in op_data:
                    # This is a simplified throughput calculation
                    throughput_score = 1000.0 / op_data['mean_time_ms']  # operations per second
                    throughput_scores.append(throughput_score)

            efficiency_data[reasoner] = {
                'memory_efficiency': statistics.mean(memory_scores) if memory_scores else 0.0,
                'throughput_efficiency': statistics.mean(throughput_scores) if throughput_scores else 0.0,
                'overall_efficiency': (statistics.mean(memory_scores) if memory_scores else 0.0) * 0.5 +
                                   (statistics.mean(throughput_scores) if throughput_scores else 0.0) * 0.5
            }

        return efficiency_data

    def _analyze_reliability(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Analyze reliability metrics"""
        reliability_data = {}

        for reasoner in suite.statistical_summary['performance_by_reasoner'].keys():
            reasoner_results = [r for r in suite.test_results if r.reasoner_name == reasoner]

            total_tests = len(reasoner_results)
            successful_tests = len([r for r in reasoner_results if r.success])
            timeout_tests = len([r for r in reasoner_results if r.timeout_occurred])
            error_tests = len([r for r in reasoner_results if not r.success and not r.timeout_occurred])

            reliability_data[reasoner] = {
                'success_rate': successful_tests / total_tests if total_tests > 0 else 0.0,
                'timeout_rate': timeout_tests / total_tests if total_tests > 0 else 0.0,
                'error_rate': error_tests / total_tests if total_tests > 0 else 0.0,
                'stability_score': 1.0 - (len([r for r in reasoner_results if r.coefficient_of_variation > 0.5]) / total_tests) if total_tests > 0 else 0.0
            }

        return reliability_data

    def _analyze_scalability(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Analyze scalability metrics"""
        scalability_data = {}

        # Group by ontology size/scale
        scale_groups = {}
        for result in suite.test_results:
            scale_key = result.triples_count  # Use triples count as scale indicator
            if scale_key not in scale_groups:
                scale_groups[scale_key] = {}
            if result.reasoner_name not in scale_groups[scale_key]:
                scale_groups[scale_key][result.reasoner_name] = []
            scale_groups[scale_key][result.reasoner_name].append(result)

        # Analyze scalability for each reasoner
        for reasoner in suite.statistical_summary['performance_by_reasoner'].keys():
            scale_performance = []
            scale_memory = []

            for scale, reasoner_results in scale_groups.items():
                if reasoner in reasoner_results:
                    successful_results = [r for r in reasoner_results[reasoner] if r.success]
                    if successful_results:
                        avg_time = statistics.mean([r.execution_time_ms for r in successful_results])
                        scale_performance.append((scale, avg_time))

                        if successful_results[0].memory_metrics:
                            avg_memory = statistics.mean([r.memory_metrics.peak_memory_mb for r in successful_results])
                            scale_memory.append((scale, avg_memory))

            # Calculate scalability metrics
            time_complexity = self._calculate_complexity(scale_performance)
            memory_complexity = self._calculate_complexity(scale_memory)

            scalability_data[reasoner] = {
                'time_complexity': time_complexity,
                'memory_complexity': memory_complexity,
                'scalability_score': 1.0 - (time_complexity.get('exponent', 2.0) - 1.0) / 2.0,
                'performance_vs_scale': scale_performance,
                'memory_vs_scale': scale_memory
            }

        return scalability_data

    def _perform_comparative_analysis(self, suite: BenchmarkSuite) -> ComparativeAnalysisResult:
        """Perform comparative analysis between reasoners"""
        performance_scores = suite.generate_performance_scores()
        efficiency_analysis = self._analyze_efficiency(suite)
        reliability_analysis = self._analyze_reliability(suite)

        # Generate rankings
        performance_ranking = sorted(performance_scores.items(), key=lambda x: x[1], reverse=True)
        efficiency_ranking = sorted(efficiency_analysis.items(), key=lambda x: x[1]['overall_efficiency'], reverse=True)
        reliability_ranking = sorted(reliability_analysis.items(), key=lambda x: x[1]['success_rate'], reverse=True)

        # Overall ranking (weighted average)
        overall_scores = {}
        for reasoner in performance_scores.keys():
            overall_scores[reasoner] = (
                performance_scores[reasoner] * 0.4 +
                efficiency_analysis[reasoner]['overall_efficiency'] * 0.3 +
                reliability_analysis[reasoner]['success_rate'] * 0.3
            )

        overall_ranking = sorted(overall_scores.items(), key=lambda x: x[1], reverse=True)

        # Statistical significance testing (simplified)
        significance_results = self._perform_significance_testing(suite)

        # Generate recommendations
        recommendations = self._generate_recommendations(
            performance_ranking, efficiency_ranking, reliability_ranking, overall_ranking
        )

        # Key findings
        key_findings = self._extract_key_findings(suite)

        return ComparativeAnalysisResult(
            reasoner_comparison={
                'performance': performance_scores,
                'efficiency': efficiency_analysis,
                'reliability': reliability_analysis,
                'overall': overall_scores
            },
            performance_ranking=performance_ranking,
            efficiency_ranking=efficiency_ranking,
            reliability_ranking=reliability_ranking,
            overall_ranking=overall_ranking,
            statistical_significance=significance_results,
            recommendations=recommendations,
            key_findings=key_findings
        )

    def _analyze_consistency(self, suite: BenchmarkSuite) -> Dict[str, float]:
        """Analyze performance consistency"""
        consistency_data = {}

        for reasoner, operations in suite.statistical_summary['performance_by_reasoner'].items():
            consistency_scores = []

            for op_data in operations.values():
                if 'coefficient_of_variation' in op_data:
                    cv = op_data['coefficient_of_variation']
                    # Lower CV = higher consistency
                    consistency_score = 1.0 / (1.0 + cv)
                    consistency_scores.append(consistency_score)

            consistency_data[reasoner] = statistics.mean(consistency_scores) if consistency_scores else 0.0

        return consistency_data

    def _detect_outliers(self, suite: BenchmarkSuite) -> Dict[str, List[str]]:
        """Detect outliers in test results"""
        outliers = {}

        for (reasoner, operation), results in self._group_results(suite.test_results).items():
            if len(results) > 3:  # Need sufficient data points
                execution_times = [r.execution_time_ms for r in results if r.success]
                if execution_times:
                    q1 = statistics.quantiles(execution_times, n=4)[0] if len(execution_times) >= 4 else min(execution_times)
                    q3 = statistics.quantiles(execution_times, n=4)[2] if len(execution_times) >= 4 else max(execution_times)
                    iqr = q3 - q1
                    lower_bound = q1 - 1.5 * iqr
                    upper_bound = q3 + 1.5 * iqr

                    outlier_tests = [
                        r.test_operation.value for r in results
                        if r.success and (r.execution_time_ms < lower_bound or r.execution_time_ms > upper_bound)
                    ]

                    if outlier_tests:
                        key = f"{reasoner}_{operation}"
                        outliers[key] = outlier_tests

        return outliers

    def _calculate_complexity(self, scale_data: List[Tuple[int, float]]) -> Dict[str, float]:
        """Calculate complexity exponent from scale vs performance data"""
        if len(scale_data) < 2:
            return {'exponent': 1.0, 'r_squared': 0.0}

        # Log-log regression to find complexity exponent
        log_scales = [math.log(scale) for scale, _ in scale_data]
        log_times = [math.log(time) for _, time in scale_data]

        # Simple linear regression on log-transformed data
        n = len(log_scales)
        sum_x = sum(log_scales)
        sum_y = sum(log_times)
        sum_xy = sum(x * y for x, y in zip(log_scales, log_times))
        sum_x2 = sum(x * x for x in log_scales)

        slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
        intercept = (sum_y - slope * sum_x) / n

        # Calculate R-squared
        y_mean = sum_y / n
        ss_tot = sum((y - y_mean) ** 2 for y in log_times)
        ss_res = sum((y - (slope * x + intercept)) ** 2 for x, y in zip(log_scales, log_times))
        r_squared = 1 - (ss_res / ss_tot) if ss_tot > 0 else 0.0

        return {'exponent': slope, 'r_squared': r_squared}

    def _group_results(self, results: List[PublicationTestResult]) -> Dict[Tuple[str, str], List[PublicationTestResult]]:
        """Group results by reasoner and operation"""
        grouped = {}
        for result in results:
            key = (result.reasoner_name, result.test_operation.value)
            if key not in grouped:
                grouped[key] = []
            grouped[key].append(result)
        return grouped

    def _perform_significance_testing(self, suite: BenchmarkSuite) -> Dict[str, float]:
        """Perform statistical significance testing (simplified)"""
        # This is a simplified significance test
        # In practice, you'd use more sophisticated statistical tests
        significance = {}

        reasoner_groups = self._group_results(suite.test_results)
        reasoner_names = list(set(r.reasoner_name for r in suite.test_results))

        for i, reasoner1 in enumerate(reasoner_names):
            for j, reasoner2 in enumerate(reasoner_names[i+1:], i+1):
                key = f"{reasoner1}_vs_{reasoner2}"

                # Get execution times for successful tests
                times1 = []
                times2 = []

                for (r, op), results in reasoner_groups.items():
                    if r == reasoner1:
                        times1.extend([res.execution_time_ms for res in results if res.success])
                    elif r == reasoner2:
                        times2.extend([res.execution_time_ms for res in results if res.success])

                if times1 and times2:
                    # Simplified t-test approximation
                    mean1, mean2 = statistics.mean(times1), statistics.mean(times2)
                    std1, std2 = statistics.stdev(times1) if len(times1) > 1 else 0, statistics.stdev(times2) if len(times2) > 1 else 0
                    n1, n2 = len(times1), len(times2)

                    # Pooled standard deviation
                    if n1 > 1 and n2 > 1:
                        pooled_std = math.sqrt(((n1-1)*std1**2 + (n2-1)*std2**2) / (n1 + n2 - 2))
                        t_stat = abs(mean1 - mean2) / (pooled_std * math.sqrt(1/n1 + 1/n2))
                        # Simplified p-value approximation
                        p_value = 2 * (1 - self._t_cdf(abs(t_stat), n1 + n2 - 2))
                        significance[key] = p_value
                    else:
                        significance[key] = 0.05  # Default

        return significance

    def _t_cdf(self, t: float, df: int) -> float:
        """Simplified t-distribution CDF"""
        # This is a very rough approximation
        # In practice, use scipy.stats.t.cdf
        return 1.0 / (1.0 + math.exp(-0.7 * t * (1 + 0.1 * t**2 / df)))

    def _generate_recommendations(self, perf_rank, eff_rank, rel_rank, overall_rank) -> List[str]:
        """Generate recommendations based on analysis"""
        recommendations = []

        # Overall winner
        if overall_rank:
            winner = overall_rank[0][0]
            recommendations.append(f"{winner} shows the best overall performance and is recommended for general use.")

        # Performance specialist
        if perf_rank and perf_rank[0][0] != winner:
            perf_specialist = perf_rank[0][0]
            recommendations.append(f"{perf_specialist} offers the best raw performance for speed-critical applications.")

        # Efficiency specialist
        if eff_rank and eff_rank[0][0] != winner:
            eff_specialist = eff_rank[0][0]
            recommendations.append(f"{eff_specialist} provides the best memory efficiency for resource-constrained environments.")

        # Reliability specialist
        if rel_rank and rel_rank[0][0] != winner:
            rel_specialist = rel_rank[0][0]
            recommendations.append(f"{rel_specialist} demonstrates the highest reliability for mission-critical applications.")

        # General recommendations
        recommendations.append("Consider your specific use case requirements when selecting a reasoner.")
        recommendations.append("Test with your own ontologies for the most accurate performance assessment.")

        return recommendations

    def _extract_key_findings(self, suite: BenchmarkSuite) -> List[str]:
        """Extract key findings from the analysis"""
        findings = []

        # Performance range
        all_times = [r.execution_time_ms for r in suite.test_results if r.success]
        if all_times:
            findings.append(f"Performance ranges from {min(all_times):.1f}ms to {max(all_times):.1f}ms across all reasoners.")

        # Success rates
        total_tests = len(suite.test_results)
        successful_tests = len([r for r in suite.test_results if r.success])
        success_rate = successful_tests / total_tests * 100 if total_tests > 0 else 0
        findings.append(f"Overall success rate across all reasoners: {success_rate:.1f}%.")

        # Memory usage
        memory_results = [r for r in suite.test_results if r.memory_metrics and r.success]
        if memory_results:
            peak_memories = [r.memory_metrics.peak_memory_mb for r in memory_results]
            findings.append(f"Peak memory usage ranges from {min(peak_memories):.1f}MB to {max(peak_memories):.1f}MB.")

        # Format compatibility
        format_success = {}
        for result in suite.test_results:
            if result.ontology_format not in format_success:
                format_success[result.ontology_format] = {'total': 0, 'success': 0}
            format_success[result.ontology_format]['total'] += 1
            if result.success:
                format_success[result.ontology_format]['success'] += 1

        for format_name, counts in format_success.items():
            if counts['total'] > 0:
                format_rate = counts['success'] / counts['total'] * 100
                findings.append(f"{format_name} format success rate: {format_rate:.1f}%.")

        return findings

# Utility functions for data export
def export_benchmark_suite(suite: BenchmarkSuite, output_dir: str = "results"):
    """Export benchmark suite in multiple formats"""
    output_path = Path(output_dir)
    output_path.mkdir(exist_ok=True)

    # JSON export
    json_file = output_path / f"{suite.suite_name}_results.json"
    with open(json_file, 'w') as f:
        json.dump(asdict(suite), f, indent=2, default=str)

    # Markdown summary
    md_file = output_path / f"{suite.suite_name}_summary.md"
    with open(md_file, 'w') as f:
        f.write(f"# {suite.suite_name} Results\n\n")
        f.write(f"**Benchmark Type**: {suite.benchmark_type.value}\n")
        f.write(f"**Total Tests**: {len(suite.test_results)}\n")
        f.write(f"**Successful Tests**: {len([r for r in suite.test_results if r.success])}\n")
        f.write(f"**Collection Time**: {suite.collection_timestamp}\n\n")

        # Performance rankings
        scores = suite.generate_performance_scores()
        if scores:
            f.write("## Performance Rankings\n\n")
            for reasoner, score in sorted(scores.items(), key=lambda x: x[1], reverse=True):
                f.write(f"1. **{reasoner}**: {score:.3f}\n")

    print(f"✅ Benchmark suite exported:")
    print(f"   📊 JSON: {json_file}")
    print(f"   📄 Summary: {md_file}")

def main():
    """Example usage of enhanced data structures"""
    print("🔧 Enhanced Data Structures for Publication-Ready Testing")
    print("=" * 60)

    # Example: Create a test result
    test_result = PublicationTestResult(
        reasoner_name="Rust OWL2",
        reasoner_type=ReasonerType.FRAMEWORK,
        benchmark_type=BenchmarkType.CUSTOM,
        test_operation=TestOperation.CLASSIFICATION,
        ontology_file="test.owl",
        ontology_format="RDF/XML",
        test_timestamp=datetime.now().isoformat(),
        success=True,
        execution_time_ms=250.5,
        return_code=0,
        timeout_occurred=False,
        triples_count=1000,
        axioms_count=500,
        classes_count=50,
        properties_count=25,
        individuals_count=200
    )

    print(f"✅ Created test result: {test_result.reasoner_name} - {test_result.test_operation.value}")
    print(f"📊 Throughput: {test_result.throughput_triples_per_second:.1f} triples/sec")
    print(f"⏱️  Time per triple: {test_result.time_per_triple_microseconds:.2f} μs")

if __name__ == "__main__":
    main()