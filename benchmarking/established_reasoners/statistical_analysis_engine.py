#!/usr/bin/env python3

"""
Publication-Ready Statistical Analysis Engine for OWL2 Reasoner Testing
Implements academic-grade statistical analysis and significance testing
"""

import statistics
import math
import json
import scipy.stats as stats
import numpy as np
from typing import Dict, List, Tuple, Optional, Any, Union
from dataclasses import dataclass, asdict
from pathlib import Path
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from datetime import datetime
import warnings
warnings.filterwarnings('ignore')

# Import our enhanced data structures
from enhanced_data_structures import (
    PublicationTestResult, BenchmarkSuite, BenchmarkType,
    TestOperation, ReasonerType, ComparativeAnalysisResult,
    EnhancedDataAnalyzer
)

@dataclass
class StatisticalResult:
    """Result of statistical analysis"""
    test_name: str
    test_type: str
    statistic: float
    p_value: float
    effect_size: float
    confidence_interval: Tuple[float, float]
    interpretation: str
    significance_level: float = 0.05

@dataclass
class PerformanceProfile:
    """Performance profile for a reasoner"""
    reasoner_name: str
    mean_performance: float
    median_performance: float
    std_deviation: float
    coefficient_of_variation: float
    performance_range: Tuple[float, float]
    outlier_count: int
    reliability_score: float
    efficiency_score: float
    overall_score: float
    rank: int

class StatisticalAnalysisEngine:
    """Advanced statistical analysis for academic publication"""

    def __init__(self, significance_level: float = 0.05):
        self.significance_level = significance_level
        self.analyzer = EnhancedDataAnalyzer()
        self.results_cache = {}

    def analyze_benchmark_suite(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Perform comprehensive statistical analysis of benchmark suite"""
        print(f"📊 Performing statistical analysis for {suite.suite_name}...")

        analysis_results = {
            'basic_statistics': self._basic_statistical_analysis(suite),
            'comparative_analysis': self._comparative_analysis(suite),
            'significance_testing': self._significance_testing(suite),
            'performance_profiles': self._performance_profiling(suite),
            'reliability_analysis': self._reliability_analysis(suite),
            'scalability_analysis': self._scalability_analysis(suite),
            'outlier_analysis': self._outlier_analysis(suite),
            'correlation_analysis': self._correlation_analysis(suite),
            'publication_ready_insights': self._generate_publication_insights(suite)
        }

        # Cache results
        self.results_cache[suite.suite_name] = analysis_results

        print("✅ Statistical analysis complete")
        return analysis_results

    def _basic_statistical_analysis(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Perform basic statistical analysis"""
        print("   📈 Computing basic statistics...")

        basic_stats = {
            'summary': {
                'total_tests': len(suite.test_results),
                'successful_tests': len([r for r in suite.test_results if r.success]),
                'failed_tests': len([r for r in suite.test_results if not r.success]),
                'timeout_rate': len([r for r in suite.test_results if r.timeout_occurred]) / len(suite.test_results) * 100,
                'overall_success_rate': len([r for r in suite.test_results if r.success]) / len(suite.test_results) * 100
            }
        }

        # Group by reasoner and operation
        grouped_results = self._group_results(suite.test_results)

        for (reasoner, operation), results in grouped_results.items():
            successful_results = [r for r in results if r.success and not r.timeout_occurred]

            if successful_results:
                execution_times = [r.execution_time_ms for r in successful_results]

                key = f"{reasoner}_{operation}"
                basic_stats[key] = {
                    'count': len(successful_results),
                    'mean_ms': statistics.mean(execution_times),
                    'median_ms': statistics.median(execution_times),
                    'std_dev_ms': statistics.stdev(execution_times) if len(execution_times) > 1 else 0.0,
                    'min_ms': min(execution_times),
                    'max_ms': max(execution_times),
                    'range_ms': max(execution_times) - min(execution_times),
                    'coefficient_of_variation': statistics.stdev(execution_times) / statistics.mean(execution_times) if len(execution_times) > 1 else 0.0,
                    'percentile_25': np.percentile(execution_times, 25),
                    'percentile_75': np.percentile(execution_times, 75),
                    'iqr': np.percentile(execution_times, 75) - np.percentile(execution_times, 25)
                }

                # Memory statistics if available
                memory_results = [r for r in successful_results if r.memory_metrics]
                if memory_results:
                    peak_memories = [r.memory_metrics.peak_memory_mb for r in memory_results]
                    basic_stats[key]['memory_stats'] = {
                        'mean_peak_memory_mb': statistics.mean(peak_memories),
                        'std_dev_memory_mb': statistics.stdev(peak_memories) if len(peak_memories) > 1 else 0.0,
                        'min_memory_mb': min(peak_memories),
                        'max_memory_mb': max(peak_memories)
                    }

        return basic_stats

    def _comparative_analysis(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Perform comparative analysis between reasoners"""
        print("   🔄 Performing comparative analysis...")

        # Use the existing comparative analysis from EnhancedDataAnalyzer
        comparative_result = self.analyzer._perform_comparative_analysis(suite)

        # Enhance with additional statistical metrics
        enhanced_comparison = {
            'overall_ranking': comparative_result.overall_ranking,
            'performance_ranking': comparative_result.performance_ranking,
            'efficiency_ranking': comparative_result.efficiency_ranking,
            'reliability_ranking': comparative_result.reliability_ranking,
            'reasoner_scores': comparative_result.reasoner_comparison,
            'statistical_significance': comparative_result.statistical_significance,
            'recommendations': comparative_result.recommendations,
            'key_findings': comparative_result.key_findings,
            'pairwise_comparisons': self._pairwise_comparisons(suite)
        }

        return enhanced_comparison

    def _significance_testing(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Perform statistical significance testing"""
        print("   🧪 Performing significance testing...")

        significance_results = {}

        # Group results by operation
        operation_groups = {}
        for result in suite.test_results:
            if result.success and not result.timeout_occurred:
                if result.test_operation.value not in operation_groups:
                    operation_groups[result.test_operation.value] = {}
                if result.reasoner_name not in operation_groups[result.test_operation.value]:
                    operation_groups[result.test_operation.value][result.reasoner_name] = []
                operation_groups[result.test_operation.value][result.reasoner_name].append(result.execution_time_ms)

        # Perform significance tests for each operation
        for operation, reasoner_data in operation_groups.items():
            operation_results = {}

            reasoners = list(reasoner_data.keys())
            for i, reasoner1 in enumerate(reasoners):
                for j, reasoner2 in enumerate(reasoners[i+1:], i+1):
                    data1 = reasoner_data[reasoner1]
                    data2 = reasoner_data[reasoner2]

                    if len(data1) >= 3 and len(data2) >= 3:  # Minimum sample size
                        test_result = self._perform_statistical_test(data1, data2, reasoner1, reasoner2, operation)
                        key = f"{reasoner1}_vs_{reasoner2}_{operation}"
                        operation_results[key] = asdict(test_result)

            significance_results[operation] = operation_results

        return significance_results

    def _perform_statistical_test(self, data1: List[float], data2: List[float],
                                   reasoner1: str, reasoner2: str, operation: str) -> StatisticalResult:
        """Perform statistical test between two reasoners"""
        try:
            # Check normality assumption
            normal1 = self._check_normality(data1)
            normal2 = self._check_normality(data2)

            # Check variance homogeneity
            equal_variance = self._check_equal_variance(data1, data2)

            if normal1 and normal2 and equal_variance:
                # Use t-test
                statistic, p_value = stats.ttest_ind(data1, data2, equal_var=True)
                test_type = "Independent t-test"
            elif normal1 and normal2 and not equal_variance:
                # Use Welch's t-test
                statistic, p_value = stats.ttest_ind(data1, data2, equal_var=False)
                test_type = "Welch's t-test"
            else:
                # Use Mann-Whitney U test (non-parametric)
                statistic, p_value = stats.mannwhitneyu(data1, data2, alternative='two-sided')
                test_type = "Mann-Whitney U test"

            # Calculate effect size (Cohen's d for t-test, rank-biserial for Mann-Whitney)
            if test_type in ["Independent t-test", "Welch's t-test"]:
                effect_size = self._calculate_cohens_d(data1, data2)
            else:
                effect_size = self._calculate_rank_biserial(data1, data2, statistic)

            # Calculate confidence interval
            mean_diff = statistics.mean(data1) - statistics.mean(data2)
            ci_lower, ci_upper = self._calculate_confidence_interval(data1, data2)

            # Interpret results
            significant = p_value < self.significance_level
            if significant:
                interpretation = f"{reasoner1} is significantly {'faster' if mean_diff < 0 else 'slower'} than {reasoner2} for {operation}"
            else:
                interpretation = f"No significant difference between {reasoner1} and {reasoner2} for {operation}"

            return StatisticalResult(
                test_name=f"{reasoner1}_vs_{reasoner2}_{operation}",
                test_type=test_type,
                statistic=statistic,
                p_value=p_value,
                effect_size=effect_size,
                confidence_interval=(ci_lower, ci_upper),
                interpretation=interpretation,
                significance_level=self.significance_level
            )

        except Exception as e:
            return StatisticalResult(
                test_name=f"{reasoner1}_vs_{reasoner2}_{operation}",
                test_type="Error",
                statistic=0.0,
                p_value=1.0,
                effect_size=0.0,
                confidence_interval=(0.0, 0.0),
                interpretation=f"Statistical test failed: {str(e)}",
                significance_level=self.significance_level
            )

    def _performance_profiling(self, suite: BenchmarkSuite) -> Dict[str, PerformanceProfile]:
        """Create performance profiles for each reasoner"""
        print("   📊 Creating performance profiles...")

        profiles = {}
        reasoner_data = self._group_by_reasoner(suite.test_results)

        for reasoner, results in reasoner_data.items():
            successful_results = [r for r in results if r.success and not r.timeout_occurred]

            if successful_results:
                execution_times = [r.execution_time_ms for r in successful_results]

                # Basic performance metrics
                mean_perf = statistics.mean(execution_times)
                median_perf = statistics.median(execution_times)
                std_dev = statistics.stdev(execution_times) if len(execution_times) > 1 else 0.0
                cv = std_dev / mean_perf if mean_perf > 0 else 0.0

                # Outlier detection
                q1 = np.percentile(execution_times, 25)
                q3 = np.percentile(execution_times, 75)
                iqr = q3 - q1
                lower_bound = q1 - 1.5 * iqr
                upper_bound = q3 + 1.5 * iqr
                outliers = [t for t in execution_times if t < lower_bound or t > upper_bound]

                # Reliability score
                total_tests = len(results)
                successful_count = len(successful_results)
                reliability_score = successful_count / total_tests

                # Efficiency score (memory + throughput)
                efficiency_scores = []
                for result in successful_results:
                    if result.memory_metrics and result.triples_count > 0:
                        mem_efficiency = result.triples_count / result.memory_metrics.peak_memory_mb
                        time_efficiency = result.triples_count / (result.execution_time_ms / 1000.0)
                        efficiency_scores.append((mem_efficiency + time_efficiency) / 2)

                efficiency_score = statistics.mean(efficiency_scores) if efficiency_scores else 0.0

                # Overall score (weighted combination)
                overall_score = (
                    (1.0 / (mean_perf + 1.0)) * 0.4 +  # Inverse of time (lower = better)
                    efficiency_score * 0.3 +               # Efficiency
                    reliability_score * 0.3                # Reliability
                )

                profiles[reasoner] = PerformanceProfile(
                    reasoner_name=reasoner,
                    mean_performance=mean_perf,
                    median_performance=median_perf,
                    std_deviation=std_dev,
                    coefficient_of_variation=cv,
                    performance_range=(min(execution_times), max(execution_times)),
                    outlier_count=len(outliers),
                    reliability_score=reliability_score,
                    efficiency_score=efficiency_score,
                    overall_score=overall_score,
                    rank=0  # Will be set after ranking
                )

        # Assign ranks
        ranked_profiles = sorted(profiles.values(), key=lambda x: x.overall_score, reverse=True)
        for rank, profile in enumerate(ranked_profiles, 1):
            profile.rank = rank
            profiles[profile.reasoner_name] = profile

        return profiles

    def _reliability_analysis(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Analyze reliability metrics"""
        print("   🛡️  Analyzing reliability...")

        reliability_analysis = {}

        # Group by reasoner
        reasoner_groups = {}
        for result in suite.test_results:
            if result.reasoner_name not in reasoner_groups:
                reasoner_groups[result.reasoner_name] = []
            reasoner_groups[result.reasoner_name].append(result)

        for reasoner, results in reasoner_groups.items():
            total_tests = len(results)
            successful_tests = len([r for r in results if r.success])
            timeout_tests = len([r for r in results if r.timeout_occurred])
            error_tests = len([r for r in results if not r.success and not r.timeout_occurred])

            # Calculate reliability metrics
            success_rate = successful_tests / total_tests if total_tests > 0 else 0.0
            timeout_rate = timeout_tests / total_tests if total_tests > 0 else 0.0
            error_rate = error_tests / total_tests if total_tests > 0 else 0.0

            # Consistency analysis (coefficient of variation for successful tests)
            successful_results = [r for r in results if r.success and not r.timeout_occurred]
            cv_scores = []
            for operation in set(r.test_operation.value for r in results):
                op_results = [r.execution_time_ms for r in successful_results if r.test_operation.value == operation]
                if len(op_results) > 1:
                    cv = statistics.stdev(op_results) / statistics.mean(op_results)
                    cv_scores.append(cv)

            consistency_score = 1.0 - (statistics.mean(cv_scores) if cv_scores else 0.0)

            reliability_analysis[reasoner] = {
                'success_rate': success_rate,
                'timeout_rate': timeout_rate,
                'error_rate': error_rate,
                'consistency_score': consistency_score,
                'overall_reliability': (success_rate + consistency_score) / 2,
                'test_coverage': len(set(r.test_operation.value for r in results))
            }

        return reliability_analysis

    def _scalability_analysis(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Analyze scalability characteristics"""
        print("   📈 Analyzing scalability...")

        scalability_analysis = {}

        # Group by ontology size (triples count)
        scale_groups = {}
        for result in suite.test_results:
            if result.success and not result.timeout_occurred:
                scale_key = result.triples_count
                if scale_key not in scale_groups:
                    scale_groups[scale_key] = {}
                if result.reasoner_name not in scale_groups[scale_key]:
                    scale_groups[scale_key][result.reasoner_name] = []
                scale_groups[scale_key][result.reasoner_name].append(result)

        # Analyze scalability for each reasoner
        reasoner_names = set()
        for scale_data in scale_groups.values():
            reasoner_names.update(scale_data.keys())

        for reasoner in reasoner_names:
            scale_performance = []
            scale_memory = []

            for scale, reasoner_data in scale_groups.items():
                if reasoner in reasoner_data:
                    results = reasoner_data[reasoner]
                    if results:
                        avg_time = statistics.mean([r.execution_time_ms for r in results])
                        scale_performance.append((scale, avg_time))

                        memory_results = [r for r in results if r.memory_metrics]
                        if memory_results:
                            avg_memory = statistics.mean([r.memory_metrics.peak_memory_mb for r in memory_results])
                            scale_memory.append((scale, avg_memory))

            # Calculate complexity metrics
            time_complexity = self._calculate_complexity_exponent(scale_performance)
            memory_complexity = self._calculate_complexity_exponent(scale_memory)

            # Scalability score (lower exponent = better scalability)
            time_scalability = max(0.0, 1.0 - (time_complexity.get('exponent', 2.0) - 1.0) / 2.0)
            memory_scalability = max(0.0, 1.0 - (memory_complexity.get('exponent', 2.0) - 1.0) / 2.0)

            scalability_analysis[reasoner] = {
                'time_complexity': time_complexity,
                'memory_complexity': memory_complexity,
                'time_scalability_score': time_scalability,
                'memory_scalability_score': memory_scalability,
                'overall_scalability_score': (time_scalability + memory_scalability) / 2,
                'scale_vs_time': scale_performance,
                'scale_vs_memory': scale_memory,
                'breakpoint_analysis': self._analyze_breakpoints(scale_performance)
            }

        return scalability_analysis

    def _outlier_analysis(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Analyze outliers in test results"""
        print("   🎯 Analyzing outliers...")

        outlier_analysis = {}

        # Group by reasoner and operation
        grouped_results = self._group_results(suite.test_results)

        for (reasoner, operation), results in grouped_results.items():
            successful_results = [r for r in results if r.success and not r.timeout_occurred]

            if len(successful_results) >= 4:  # Need sufficient data points
                execution_times = [r.execution_time_ms for r in successful_results]

                # Calculate quartiles
                q1 = np.percentile(execution_times, 25)
                q3 = np.percentile(execution_times, 75)
                iqr = q3 - q1

                # Define outlier bounds
                lower_bound = q1 - 1.5 * iqr
                upper_bound = q3 + 1.5 * iqr

                # Identify outliers
                outliers = []
                for i, time in enumerate(execution_times):
                    if time < lower_bound or time > upper_bound:
                        outliers.append({
                            'index': i,
                            'execution_time_ms': time,
                            'deviation_from_median': abs(time - statistics.median(execution_times)),
                            'severity': 'mild' if (time < lower_bound * 0.5 or time > upper_bound * 1.5) else 'moderate',
                            'test_result': successful_results[i]
                        })

                key = f"{reasoner}_{operation}"
                outlier_analysis[key] = {
                    'total_tests': len(successful_results),
                    'outlier_count': len(outliers),
                    'outlier_rate': len(outliers) / len(successful_results) * 100,
                    'outliers': outliers,
                    'bounds': {
                        'lower_bound': lower_bound,
                        'upper_bound': upper_bound,
                        'q1': q1,
                        'q3': q3,
                        'iqr': iqr
                    }
                }

        return outlier_analysis

    def _correlation_analysis(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Analyze correlations between different metrics"""
        print("   🔗 Analyzing correlations...")

        correlation_analysis = {}

        # Collect data for correlation analysis
        data_points = []
        for result in suite.test_results:
            if result.success and not result.timeout_occurred:
                data_point = {
                    'execution_time_ms': result.execution_time_ms,
                    'triples_count': result.triples_count,
                    'axioms_count': result.axioms_count,
                }

                if result.memory_metrics:
                    data_point['peak_memory_mb'] = result.memory_metrics.peak_memory_mb
                    data_point['memory_efficiency'] = result.memory_efficiency_score

                data_points.append(data_point)

        if len(data_points) >= 3:  # Need sufficient data points
            # Convert to DataFrame for analysis
            df = pd.DataFrame(data_points)

            # Calculate correlation matrix
            numeric_columns = df.select_dtypes(include=[np.number]).columns
            correlation_matrix = df[numeric_columns].corr()

            correlation_analysis = {
                'correlation_matrix': correlation_matrix.to_dict(),
                'significant_correlations': self._find_significant_correlations(correlation_matrix),
                'data_points_count': len(data_points)
            }

        return correlation_analysis

    def _generate_publication_insights(self, suite: BenchmarkSuite) -> Dict[str, Any]:
        """Generate publication-ready insights and conclusions"""
        print("   💡 Generating publication insights...")

        insights = {
            'key_findings': [],
            'performance_conclusions': [],
            'methodology_notes': [],
            'limitations': [],
            'recommendations': [],
            'future_research_directions': []
        }

        # Analyze overall performance
        basic_stats = self._basic_statistical_analysis(suite)
        comparative_analysis = self._comparative_analysis(suite)

        # Key findings
        total_tests = basic_stats['summary']['total_tests']
        success_rate = basic_stats['summary']['overall_success_rate']
        insights['key_findings'].append(f"Overall success rate across {total_tests} tests: {success_rate:.1f}%")

        # Performance insights
        if comparative_analysis.get('overall_ranking'):
            top_performer = comparative_analysis['overall_ranking'][0][0]
            insights['performance_conclusions'].append(f"{top_performer} demonstrates the best overall performance")

        # Methodology insights
        insights['methodology_notes'].append("Comprehensive statistical analysis with significance testing")
        insights['methodology_notes'].append("Multiple benchmark types and scales evaluated")

        # Limitations
        insights['limitations'].append("Results specific to tested ontologies and configurations")
        insights['limitations'].append("Real-world performance may vary with different datasets")

        # Recommendations
        insights['recommendations'].append("Consider specific use case requirements when selecting reasoners")
        insights['recommendations'].append("Test with custom ontologies for production deployment")

        # Future research
        insights['future_research_directions'].append("Large-scale ontology testing (100K+ entities)")
        insights['future_research_directions'].append("Concurrency and parallel processing evaluation")

        return insights

    # Helper methods
    def _group_results(self, results: List[PublicationTestResult]) -> Dict[Tuple[str, str], List[PublicationTestResult]]:
        """Group results by reasoner and operation"""
        grouped = {}
        for result in results:
            key = (result.reasoner_name, result.test_operation.value)
            if key not in grouped:
                grouped[key] = []
            grouped[key].append(result)
        return grouped

    def _group_by_reasoner(self, results: List[PublicationTestResult]) -> Dict[str, List[PublicationTestResult]]:
        """Group results by reasoner"""
        grouped = {}
        for result in results:
            if result.reasoner_name not in grouped:
                grouped[result.reasoner_name] = []
            grouped[result.reasoner_name].append(result)
        return grouped

    def _pairwise_comparisons(self, suite: BenchmarkSuite) -> Dict[str, Dict[str, Any]]:
        """Perform pairwise comparisons between all reasoners"""
        pairwise_results = {}

        reasoner_groups = self._group_by_reasoner(suite.test_results)
        reasoner_names = list(reasoner_groups.keys())

        for i, reasoner1 in enumerate(reasoner_names):
            for j, reasoner2 in enumerate(reasoner_names[i+1:], i+1):
                key = f"{reasoner1}_vs_{reasoner2}"
                pairwise_results[key] = self._compare_reasoners_pairwise(
                    reasoner_groups[reasoner1], reasoner_groups[reasoner2]
                )

        return pairwise_results

    def _compare_reasoners_pairwise(self, results1: List[PublicationTestResult], results2: List[PublicationTestResult]) -> Dict[str, Any]:
        """Compare two reasoners pairwise"""
        successful1 = [r for r in results1 if r.success and not r.timeout_occurred]
        successful2 = [r for r in results2 if r.success and not r.timeout_occurred]

        if not successful1 or not successful2:
            return {'error': 'Insufficient successful tests for comparison'}

        times1 = [r.execution_time_ms for r in successful1]
        times2 = [r.execution_time_ms for r in successful2]

        comparison = {
            'reasoner1_stats': {
                'mean': statistics.mean(times1),
                'median': statistics.median(times1),
                'std_dev': statistics.stdev(times1) if len(times1) > 1 else 0.0,
                'count': len(times1)
            },
            'reasoner2_stats': {
                'mean': statistics.mean(times2),
                'median': statistics.median(times2),
                'std_dev': statistics.stdev(times2) if len(times2) > 1 else 0.0,
                'count': len(times2)
            },
            'performance_difference': {
                'absolute': abs(statistics.mean(times1) - statistics.mean(times2)),
                'relative': abs(statistics.mean(times1) - statistics.mean(times2)) / min(statistics.mean(times1), statistics.mean(times2)) * 100
            }
        }

        return comparison

    def _check_normality(self, data: List[float]) -> bool:
        """Check if data follows normal distribution using Shapiro-Wilk test"""
        if len(data) < 3:
            return False

        try:
            _, p_value = stats.shapiro(data)
            return p_value > 0.05
        except:
            return False

    def _check_equal_variance(self, data1: List[float], data2: List[float]) -> bool:
        """Check if two datasets have equal variance using Levene's test"""
        if len(data1) < 3 or len(data2) < 3:
            return True

        try:
            _, p_value = stats.levene(data1, data2)
            return p_value > 0.05
        except:
            return True

    def _calculate_cohens_d(self, data1: List[float], data2: List[float]) -> float:
        """Calculate Cohen's d effect size"""
        mean1, mean2 = statistics.mean(data1), statistics.mean(data2)
        std1, std2 = statistics.stdev(data1) if len(data1) > 1 else 0, statistics.stdev(data2) if len(data2) > 1 else 0

        # Pooled standard deviation
        n1, n2 = len(data1), len(data2)
        pooled_std = math.sqrt(((n1-1)*std1**2 + (n2-1)*std2**2) / (n1 + n2 - 2))

        if pooled_std == 0:
            return 0.0

        return abs(mean1 - mean2) / pooled_std

    def _calculate_rank_biserial(self, data1: List[float], data2: List[float], statistic: float) -> float:
        """Calculate rank-biserial correlation for Mann-Whitney U test"""
        n1, n2 = len(data1), len(data2)
        return statistic / (n1 * n2) - 0.5

    def _calculate_confidence_interval(self, data1: List[float], data2: List[float]) -> Tuple[float, float]:
        """Calculate 95% confidence interval for mean difference"""
        mean_diff = statistics.mean(data1) - statistics.mean(data2)

        # Standard error of the difference
        se_diff = math.sqrt(
            (statistics.variance(data1) / len(data1)) + (statistics.variance(data2) / len(data2))
        )

        # Degrees of freedom (Welch-Satterthwaite)
        df = self._welch_satterthwaite_df(data1, data2)

        # t-critical value for 95% CI
        t_critical = stats.t.ppf(0.975, df)

        # Confidence interval
        margin_of_error = t_critical * se_diff
        ci_lower = mean_diff - margin_of_error
        ci_upper = mean_diff + margin_of_error

        return (ci_lower, ci_upper)

    def _welch_satterthwaite_df(self, data1: List[float], data2: List[float]) -> float:
        """Calculate Welch-Satterthwaite degrees of freedom"""
        var1, var2 = statistics.variance(data1), statistics.variance(data2)
        n1, n2 = len(data1), len(data2)

        numerator = (var1/n1 + var2/n2)**2
        denominator = (var1**2)/(n1**2 * (n1-1)) + (var2**2)/(n2**2 * (n2-1))

        return numerator / denominator if denominator > 0 else min(n1-1, n2-1)

    def _calculate_complexity_exponent(self, scale_data: List[Tuple[float, float]]) -> Dict[str, float]:
        """Calculate complexity exponent from scale vs performance data"""
        if len(scale_data) < 2:
            return {'exponent': 1.0, 'r_squared': 0.0}

        # Log-log regression
        log_scales = [math.log(scale) for scale, _ in scale_data]
        log_values = [math.log(value) for _, value in scale_data]

        # Linear regression
        n = len(log_scales)
        sum_x = sum(log_scales)
        sum_y = sum(log_values)
        sum_xy = sum(x * y for x, y in zip(log_scales, log_values))
        sum_x2 = sum(x * x for x in log_scales)

        slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
        intercept = (sum_y - slope * sum_x) / n

        # R-squared calculation
        y_mean = sum_y / n
        ss_tot = sum((y - y_mean)**2 for y in log_values)
        ss_res = sum((y - (slope * x + intercept))**2 for x, y in zip(log_scales, log_values))
        r_squared = 1 - (ss_res / ss_tot) if ss_tot > 0 else 0.0

        return {'exponent': slope, 'r_squared': r_squared}

    def _analyze_breakpoints(self, scale_performance: List[Tuple[float, float]]) -> Dict[str, Any]:
        """Analyze performance breakpoints where performance degrades significantly"""
        if len(scale_performance) < 3:
            return {'breakpoints': [], 'analysis': 'Insufficient data'}

        breakpoints = []
        for i in range(1, len(scale_performance)):
            scale1, time1 = scale_performance[i-1]
            scale2, time2 = scale_performance[i]

            # Calculate performance degradation
            time_increase = (time2 - time1) / time1 * 100
            scale_increase = (scale2 - scale1) / scale1 * 100

            # If time increase is significantly higher than scale increase
            if time_increase > scale_increase * 2:
                breakpoints.append({
                    'scale': scale2,
                    'time': time2,
                    'degradation_factor': time_increase / scale_increase,
                    'severity': 'high' if time_increase > scale_increase * 3 else 'moderate'
                })

        return {
            'breakpoints': breakpoints,
            'analysis': f'Found {len(breakpoints)} performance breakpoints'
        }

    def _find_significant_correlations(self, correlation_matrix: Dict[str, Dict[str, float]]) -> List[Dict[str, Any]]:
        """Find statistically significant correlations"""
        significant_correlations = []

        for var1, correlations in correlation_matrix.items():
            for var2, correlation in correlations.items():
                if var1 != var2 and abs(correlation) > 0.5:  # Strong correlation threshold
                    significant_correlations.append({
                        'variable1': var1,
                        'variable2': var2,
                        'correlation': correlation,
                        'strength': 'strong' if abs(correlation) > 0.7 else 'moderate'
                    })

        return significant_correlations

    def generate_statistical_report(self, analysis_results: Dict[str, Any], suite: BenchmarkSuite) -> str:
        """Generate comprehensive statistical report"""
        report = f"""# Statistical Analysis Report
**Benchmark Suite**: {suite.suite_name}
**Analysis Date**: {datetime.now().isoformat()}
**Total Tests**: {len(suite.test_results)}

## Executive Summary
This report provides a comprehensive statistical analysis of OWL2 reasoner performance across multiple benchmarks and configurations.

## Key Findings
"""

        # Add key findings
        if 'publication_ready_insights' in analysis_results:
            for finding in analysis_results['publication_ready_insights']['key_findings']:
                report += f"- {finding}\n"

        # Add comparative analysis
        if 'comparative_analysis' in analysis_results:
            report += "\n## Performance Rankings\n"
            rankings = analysis_results['comparative_analysis'].get('overall_ranking', [])
            for i, (reasoner, score) in enumerate(rankings, 1):
                report += f"{i}. **{reasoner}**: {score:.3f}\n"

        # Add significance testing
        if 'significance_testing' in analysis_results:
            report += "\n## Statistical Significance\n"
            significance_results = analysis_results['significance_testing']
            for operation, tests in significance_results.items():
                report += f"\n### {operation}\n"
                for test_name, test_data in tests.items():
                    if isinstance(test_data, dict) and 'p_value' in test_data:
                        significant = test_data['p_value'] < 0.05
                        report += f"- {test_name}: p={test_data['p_value']:.4f} ({'Significant' if significant else 'Not significant'})\n"

        return report

def main():
    """Main interface for statistical analysis"""
    print("📊 Statistical Analysis Engine")
    print("=" * 50)

    # Example usage (would normally load from saved benchmark results)
    print("🔧 Statistical analysis engine ready for use with benchmark suite data")
    print("\nUsage:")
    print("1. Load benchmark suite results")
    print("2. Run comprehensive statistical analysis")
    print("3. Generate publication-ready reports")
    print("4. Export statistical insights")

if __name__ == "__main__":
    main()