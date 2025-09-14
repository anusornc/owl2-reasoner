#!/usr/bin/env python3
"""
Comprehensive OWL2 Reasoner Benchmark including Custom Reasoner
Executes all 5 reasoners on LUBM and SP2B benchmarks
"""

import os
import sys
import subprocess
import time
import json
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any
import psutil
import platform

# Add the parent directory to path to import our modules
sys.path.insert(0, str(Path(__file__).parent.parent))

from enhanced_data_structures import (
    PublicationTestResult, BenchmarkSuite, ReasonerType, BenchmarkType, TestOperation,
    EnvironmentSpecification
)
from memory_profiler import CrossPlatformMemoryProfiler
from environment_collector import EnvironmentCollector

class ComprehensiveBenchmarkRunner:
    def __init__(self):
        self.results = []
        self.memory_profiler = CrossPlatformMemoryProfiler()
        self.env_collector = EnvironmentCollector()
        self.base_dir = Path(__file__).parent

        # Define all reasoners
        self.reasoners = {
            'elk': {
                'name': 'ELK',
                'type': ReasonerType.LIBRARY,
                'command': ['java', '-jar', 'elk-distribution-cli-0.6.0/elk.jar'],
                'args': {
                    'classification': ['--classify'],
                    'consistency': ['--consistent']
                }
            },
            'hermit': {
                'name': 'HermiT',
                'type': ReasonerType.LIBRARY,
                'command': ['java', '-jar', 'org.semanticweb.HermiT.jar'],
                'args': {
                    'classification': ['-c'],
                    'consistency': ['-k']
                }
            },
            'jfact': {
                'name': 'JFact',
                'type': ReasonerType.LIBRARY,
                'command': ['java', '-jar', 'jfact.jar'],
                'args': {
                    'classification': ['-c'],
                    'consistency': ['-k']
                }
            },
            'pellet': {
                'name': 'Pellet',
                'type': ReasonerType.LIBRARY,
                'command': ['java', '-jar', 'pellet.jar'],
                'args': {
                    'classification': ['-c'],
                    'consistency': ['-k']
                }
            },
            'owl2-reasoner': {
                'name': 'OWL2-Reasoner',
                'type': ReasonerType.CUSTOM,
                'command': ['./owl2-reasoner-cli'],
                'args': {
                    'classification': ['--classify'],
                    'consistency': ['--consistent']
                }
            }
        }

        # Define benchmarks
        self.benchmarks = {
            'lubm_scale_1': {
                'type': BenchmarkType.LUBM,
                'ontology': 'lubm/univ-bench.ttl',
                'data': 'lubm/data/scale_1/dataset.ttl',
                'queries': 'lubm/queries/Q1.rq'
            },
            'lubm_scale_10': {
                'type': BenchmarkType.LUBM,
                'ontology': 'lubm/univ-bench.ttl',
                'data': 'lubm/data/scale_10/dataset.ttl',
                'queries': 'lubm/queries/Q1.rq'
            },
            'sp2b_small': {
                'type': BenchmarkType.SP2B,
                'ontology': 'sp2b/social_network.ttl',
                'data': 'sp2b/data/small_dataset.ttl',
                'queries': 'sp2b/queries/Q1.rq'
            },
            'sp2b_medium': {
                'type': BenchmarkType.SP2B,
                'ontology': 'sp2b/social_network.ttl',
                'data': 'sp2b/data/medium_dataset.ttl',
                'queries': 'sp2b/queries/Q1.rq'
            }
        }

    def run_single_test(self, reasoner_key: str, benchmark_key: str, operation: TestOperation) -> TestResult:
        """Run a single test with memory profiling"""
        reasoner = self.reasoners[reasoner_key]
        benchmark = self.benchmarks[benchmark_key]

        print(f"🧪 Running {reasoner['name']} on {benchmark_key} {operation.value}...")

        # Start memory profiling
        self.memory_profiler.start_monitoring()

        start_time = time.time()

        try:
            # Prepare command
            if operation == TestOperation.CLASSIFICATION:
                cmd = reasoner['command'] + reasoner['args']['classification'] + [benchmark['ontology']]
            else:  # CONSISTENCY
                cmd = reasoner['command'] + reasoner['args']['consistency'] + [benchmark['ontology']]

            # Execute command
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=300,  # 5 minute timeout
                cwd=str(self.base_dir)
            )

            end_time = time.time()
            execution_time = end_time - start_time

            # Stop memory profiling and get metrics
            memory_profile = self.memory_profiler.stop_monitoring()

            # Create memory profile object
            mem_profile = MemoryProfile(
                peak_memory_mb=memory_profile.get('peak_memory_mb', 0),
                average_memory_mb=memory_profile.get('average_memory_mb', 0),
                memory_growth_mb=memory_profile.get('memory_growth_mb', 0),
                final_memory_mb=memory_profile.get('final_memory_mb', 0),
                sampling_interval=0.1,
                total_samples=int(execution_time / 0.1) if execution_time > 0 else 1
            )

            # Calculate efficiency metrics
            efficiency = EfficiencyMetrics(
                triples_per_second=0,  # Would need to count triples
                memory_per_triple_mb=0,  # Would need to count triples
                classification_efficiency=1.0 / execution_time if execution_time > 0 else 0,
                consistency_efficiency=1.0 / execution_time if execution_time > 0 else 0,
                overall_score=1.0 / execution_time if execution_time > 0 else 0
            )

            # Determine success
            success = result.returncode == 0

            # Create test result
            test_result = TestResult(
                reasoner_name=reasoner['name'],
                reasoner_type=reasoner['type'],
                benchmark_type=benchmark['type'],
                test_operation=operation,
                execution_time_ms=execution_time * 1000,
                success=success,
                error_message=result.stderr if result.stderr else None,
                axioms_processed=0,  # Would need to parse output
                triples_loaded=0,     # Would need to parse output
                memory_profile=mem_profile,
                efficiency_metrics=efficiency,
                timestamp=datetime.now(),
                environment_tag=platform.system()
            )

            print(f"  ✅ Completed in {execution_time:.3f}s - {'Success' if success else 'Failed'}")
            if result.stderr:
                print(f"  ⚠️  Error: {result.stderr[:200]}...")

            return test_result

        except subprocess.TimeoutExpired:
            print(f"  ⏰ Timeout after 300 seconds")
            # Clean up memory profiling
            try:
                memory_profile = self.memory_profiler.stop_monitoring()
            except:
                memory_profile = {}

            return TestResult(
                reasoner_name=reasoner['name'],
                reasoner_type=reasoner['type'],
                benchmark_type=benchmark['type'],
                test_operation=operation,
                execution_time_ms=300000,  # 5 minutes in ms
                success=False,
                error_message="Timeout after 300 seconds",
                axioms_processed=0,
                triples_loaded=0,
                memory_profile=MemoryProfile(
                    peak_memory_mb=0,
                    average_memory_mb=0,
                    memory_growth_mb=0,
                    final_memory_mb=0,
                    sampling_interval=0.1,
                    total_samples=3000
                ),
                efficiency_metrics=EfficiencyMetrics(),
                timestamp=datetime.now(),
                environment_tag=platform.system()
            )

        except Exception as e:
            print(f"  ❌ Error: {str(e)}")
            # Clean up memory profiling
            try:
                memory_profile = self.memory_profiler.stop_monitoring()
            except:
                memory_profile = {}

            return TestResult(
                reasoner_name=reasoner['name'],
                reasoner_type=reasoner['type'],
                benchmark_type=benchmark['type'],
                test_operation=operation,
                execution_time_ms=0,
                success=False,
                error_message=str(e),
                axioms_processed=0,
                triples_loaded=0,
                memory_profile=MemoryProfile(
                    peak_memory_mb=0,
                    average_memory_mb=0,
                    memory_growth_mb=0,
                    final_memory_mb=0,
                    sampling_interval=0.1,
                    total_samples=1
                ),
                efficiency_metrics=EfficiencyMetrics(),
                timestamp=datetime.now(),
                environment_tag=platform.system()
            )

    def run_comprehensive_benchmark(self) -> BenchmarkSuite:
        """Run comprehensive benchmark with all reasoners and benchmarks"""
        print("🚀 Starting Comprehensive OWL2 Reasoner Benchmark")
        print("=" * 60)

        all_results = []

        # Collect environment specification
        print("🔍 Collecting environment specification...")
        env_spec = self.env_collector.collect_complete_specification()

        # Run all combinations
        total_tests = len(self.reasoners) * len(self.benchmarks) * 2  # 2 operations per benchmark
        test_count = 0

        for reasoner_key in self.reasoners:
            for benchmark_key in self.benchmarks:
                for operation in [TestOperation.CLASSIFICATION, TestOperation.CONSISTENCY]:
                    test_count += 1
                    print(f"\n📊 Test {test_count}/{total_tests}: {reasoner_key} on {benchmark_key} ({operation.value})")

                    result = self.run_single_test(reasoner_key, benchmark_key, operation)
                    all_results.append(result)

        # Create benchmark suite
        benchmark_suite = BenchmarkSuite(
            name="Comprehensive OWL2 Reasoner Benchmark",
            description="Benchmark comparing 5 OWL2 reasoners on LUBM and SP2B benchmarks",
            results=all_results,
            environment_specification=env_spec,
            timestamp=datetime.now(),
            total_tests_run=total_tests,
            successful_tests=sum(1 for r in all_results if r.success)
        )

        print(f"\n🎯 Benchmark completed!")
        print(f"   Total tests: {total_tests}")
        print(f"   Successful: {benchmark_suite.successful_tests}")
        print(f"   Failed: {total_tests - benchmark_suite.successful_tests}")

        return benchmark_suite

    def save_results(self, benchmark_suite: BenchmarkSuite, filename: str = None):
        """Save benchmark results to file"""
        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"comprehensive_benchmark_{timestamp}.json"

        filepath = self.base_dir / "results" / filename

        # Create results directory if it doesn't exist
        filepath.parent.mkdir(exist_ok=True)

        # Convert to serializable format
        serializable_suite = {
            'name': benchmark_suite.name,
            'description': benchmark_suite.description,
            'timestamp': benchmark_suite.timestamp.isoformat(),
            'total_tests_run': benchmark_suite.total_tests_run,
            'successful_tests': benchmark_suite.successful_tests,
            'environment_specification': {
                'hardware': {
                    'cpu_model': benchmark_suite.environment_specification.hardware.cpu_model,
                    'cpu_cores': benchmark_suite.environment_specification.hardware.cpu_cores,
                    'memory_gb': benchmark_suite.environment_specification.hardware.memory_gb,
                    'architecture': benchmark_suite.environment_specification.hardware.architecture
                },
                'software': {
                    'os_name': benchmark_suite.environment_specification.software.os_name,
                    'os_version': benchmark_suite.environment_specification.software.os_version,
                    'python_version': benchmark_suite.environment_specification.software.python_version
                }
            },
            'results': []
        }

        for result in benchmark_suite.results:
            serializable_result = {
                'reasoner_name': result.reasoner_name,
                'reasoner_type': result.reasoner_type.value,
                'benchmark_type': result.benchmark_type.value,
                'test_operation': result.test_operation.value,
                'execution_time_ms': result.execution_time_ms,
                'success': result.success,
                'error_message': result.error_message,
                'axioms_processed': result.axioms_processed,
                'triples_loaded': result.triples_loaded,
                'memory_profile': {
                    'peak_memory_mb': result.memory_profile.peak_memory_mb,
                    'average_memory_mb': result.memory_profile.average_memory_mb,
                    'memory_growth_mb': result.memory_profile.memory_growth_mb,
                    'final_memory_mb': result.memory_profile.final_memory_mb,
                    'sampling_interval': result.memory_profile.sampling_interval,
                    'total_samples': result.memory_profile.total_samples
                },
                'efficiency_metrics': {
                    'triples_per_second': result.efficiency_metrics.triples_per_second,
                    'memory_per_triple_mb': result.efficiency_metrics.memory_per_triple_mb,
                    'classification_efficiency': result.efficiency_metrics.classification_efficiency,
                    'consistency_efficiency': result.efficiency_metrics.consistency_efficiency,
                    'overall_score': result.efficiency_metrics.overall_score
                },
                'timestamp': result.timestamp.isoformat(),
                'environment_tag': result.environment_tag
            }
            serializable_suite['results'].append(serializable_result)

        with open(filepath, 'w') as f:
            json.dump(serializable_suite, f, indent=2)

        print(f"💾 Results saved to: {filepath}")
        return filepath

def main():
    """Main execution function"""
    runner = ComprehensiveBenchmarkRunner()

    try:
        # Run comprehensive benchmark
        benchmark_suite = runner.run_comprehensive_benchmark()

        # Save results
        results_file = runner.save_results(benchmark_suite)

        print(f"\n🎉 Comprehensive benchmark completed successfully!")
        print(f"📊 Results file: {results_file}")

        # Print summary statistics
        print(f"\n📈 Summary Statistics:")
        reasoner_stats = {}
        for result in benchmark_suite.results:
            if result.reasoner_name not in reasoner_stats:
                reasoner_stats[result.reasoner_name] = {'total': 0, 'success': 0, 'total_time': 0}

            reasoner_stats[result.reasoner_name]['total'] += 1
            if result.success:
                reasoner_stats[result.reasoner_name]['success'] += 1
                reasoner_stats[result.reasoner_name]['total_time'] += result.execution_time_ms

        for reasoner, stats in reasoner_stats.items():
            success_rate = (stats['success'] / stats['total']) * 100
            avg_time = stats['total_time'] / stats['success'] if stats['success'] > 0 else 0
            print(f"   {reasoner}: {stats['success']}/{stats['total']} tests ({success_rate:.1f}%) - Avg: {avg_time:.2f}ms")

        return benchmark_suite

    except KeyboardInterrupt:
        print("\n⚠️  Benchmark interrupted by user")
        return None
    except Exception as e:
        print(f"\n❌ Benchmark failed: {str(e)}")
        return None

if __name__ == "__main__":
    main()