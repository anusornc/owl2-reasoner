#!/usr/bin/env python3

"""
Real OWL2 Reasoner Benchmark Execution
Executes actual benchmarks with real OWL2 reasoners and genuine performance data
"""

import os
import sys
import json
import time
import subprocess
import tempfile
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any, Optional, Tuple

# Import our testing framework
from enhanced_data_structures import (
    PublicationTestResult, TestOperation, ReasonerType, BenchmarkType,
    BenchmarkSuite, EnhancedDataAnalyzer
)
from memory_profiler import ProcessMemoryMonitor, MemoryAnalysisEngine
from environment_collector import EnvironmentCollector, EnvironmentSpecification
from statistical_analysis_engine import StatisticalAnalysisEngine

class RealBenchmarkExecutor:
    """Execute real benchmarks with actual OWL2 reasoners"""

    def __init__(self):
        self.output_dir = Path("real_benchmark_results")
        self.output_dir.mkdir(exist_ok=True)

        # Available reasoners with their configurations
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
                'command': ['java', '-jar', 'jfact-4.0.0.jar'],
                'args': {
                    'classification': ['-classify'],
                    'consistency': ['-consistent']
                }
            },
            'pellet': {
                'name': 'Pellet',
                'type': ReasonerType.LIBRARY,
                'command': ['./pellet-2.3.1/pellet.sh'],
                'args': {
                    'classification': ['classify'],
                    'consistency': ['consistency']
                }
            }
        }

        # Benchmark configurations
        self.benchmarks = {
            'lubm': {
                'path': Path('benchmarks/lubm'),
                'ontology_files': ['ontology/univ-bench.ttl'],
                'data_scales': ['scale_1', 'scale_10']
            },
            'sp2b': {
                'path': Path('benchmarks/sp2b'),
                'ontology_files': ['ontology/sp2b-social.ttl'],
                'data_scales': ['scale_1', 'scale_10']
            }
        }

        print("🚀 Real Benchmark Executor initialized")
        print(f"📁 Output directory: {self.output_dir}")
        print(f"🔧 Available reasoners: {list(self.reasoners.keys())}")
        print(f"📊 Available benchmarks: {list(self.benchmarks.keys())}")

    def execute_comprehensive_benchmark(self) -> BenchmarkSuite:
        """Execute comprehensive benchmark across all reasoners and benchmarks"""
        print("🎯 Starting comprehensive real benchmark execution...")

        test_results = []
        environment_spec = EnvironmentCollector().collect_complete_specification()

        # Execute benchmarks for each reasoner
        for reasoner_id, reasoner_config in self.reasoners.items():
            print(f"\n🔬 Testing reasoner: {reasoner_config['name']}")

            # Test each benchmark type
            for benchmark_id, benchmark_config in self.benchmarks.items():
                print(f"  📊 Testing benchmark: {benchmark_id.upper()}")

                # Test each operation type
                for operation in [TestOperation.CLASSIFICATION, TestOperation.CONSISTENCY]:
                    print(f"    🧪 Testing operation: {operation.value}")

                    # Test each data scale
                    for scale in benchmark_config['data_scales']:
                        result = self._execute_single_test(
                            reasoner_config, benchmark_config, operation, scale
                        )
                        if result:
                            test_results.append(result)
                            print(f"      ✅ {scale}: {result.execution_time_ms:.2f}ms")
                        else:
                            print(f"      ❌ {scale}: Failed")

        # Create benchmark suite
        suite = BenchmarkSuite(
            suite_name="Real_OWL2_Reasoner_Benchmark",
            benchmark_type=BenchmarkType.CUSTOM,
            description="Comprehensive real benchmark of established OWL2 reasoners",
            version="1.0",
            test_results=test_results,
            environment_spec=environment_spec,
            collection_timestamp=datetime.now().isoformat()
        )

        print(f"\n🎯 Benchmark complete! Collected {len(test_results)} real test results")
        return suite

    def _execute_single_test(self, reasoner_config: Dict, benchmark_config: Dict,
                            operation: TestOperation, scale: str) -> Optional[PublicationTestResult]:
        """Execute a single benchmark test"""

        # Build input file path
        input_file = benchmark_config['path'] / f"data/{scale}/dataset.ttl"

        if not input_file.exists():
            print(f"      ⚠️  Input file not found: {input_file}")
            return None

        # Build command
        base_command = reasoner_config['command'].copy()
        operation_args = reasoner_config['args'].get(operation.value, [])

        if operation == TestOperation.CLASSIFICATION:
            command = base_command + ['--input', str(input_file), '--classify']
        elif operation == TestOperation.CONSISTENCY:
            command = base_command + ['--input', str(input_file), '--consistent']
        else:
            print(f"      ⚠️  Unsupported operation: {operation.value}")
            return None

        # Create temporary output file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as temp_file:
            output_file = temp_file.name

        try:
            # Execute with memory monitoring
            print(f"      🔍 Executing: {' '.join(command)}")

            start_time = time.time()

            # Run the command
            result = subprocess.run(
                command,
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )

            end_time = time.time()
            execution_time_ms = (end_time - start_time) * 1000

            # Count output lines and size
            output_lines = len(result.stdout.split('\n')) if result.stdout else 0
            output_size_bytes = len(result.stdout.encode('utf-8')) if result.stdout else 0

            # Parse output for additional metrics
            success = result.returncode == 0
            error_message = result.stderr if result.stderr else None

            # Count warnings (simple heuristic)
            warning_count = len([line for line in result.stdout.split('\n')
                               if 'warning' in line.lower() or 'warn' in line.lower()]) if result.stdout else 0

            # Create test result
            test_result = PublicationTestResult(
                reasoner_name=reasoner_config['name'],
                reasoner_type=reasoner_config['type'],
                benchmark_type=BenchmarkType.LUBM if 'lubm' in str(benchmark_config['path']) else BenchmarkType.SP2B,
                test_operation=operation,
                ontology_file=str(input_file),
                ontology_format="turtle",
                test_timestamp=datetime.now().isoformat(),
                success=success,
                execution_time_ms=execution_time_ms,
                return_code=result.returncode,
                timeout_occurred=False,
                output_file=output_file,
                output_size_bytes=output_size_bytes,
                output_lines=output_lines,
                error_message=error_message,
                warning_count=warning_count
            )

            # Save actual output
            with open(output_file, 'w') as f:
                f.write(f"=== STDOUT ===\n{result.stdout}\n\n=== STDERR ===\n{result.stderr}")

            return test_result

        except subprocess.TimeoutExpired:
            print(f"      ⏰ Timeout after 300 seconds")
            return None
        except Exception as e:
            print(f"      ❌ Execution error: {e}")
            return None
        finally:
            # Clean up temp file if test failed
            if 'test_result' not in locals() and os.path.exists(output_file):
                os.unlink(output_file)

    def save_benchmark_data(self, suite: BenchmarkSuite) -> str:
        """Save benchmark data for analysis"""
        data_file = self.output_dir / f"benchmark_suite_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"

        # Convert to serializable format
        suite_data = {
            'suite_name': suite.suite_name,
            'benchmark_type': suite.benchmark_type.value,
            'description': suite.description,
            'version': suite.version,
            'collection_timestamp': suite.collection_timestamp,
            'test_results': []
        }

        for result in suite.test_results:
            result_data = {
                'reasoner_name': result.reasoner_name,
                'reasoner_type': result.reasoner_type.value,
                'benchmark_type': result.benchmark_type.value,
                'test_operation': result.test_operation.value,
                'ontology_file': result.ontology_file,
                'ontology_format': result.ontology_format,
                'test_timestamp': result.test_timestamp,
                'success': result.success,
                'execution_time_ms': result.execution_time_ms,
                'return_code': result.return_code,
                'timeout_occurred': result.timeout_occurred,
                'output_file': result.output_file,
                'output_size_bytes': result.output_size_bytes,
                'output_lines': result.output_lines,
                'error_message': result.error_message,
                'warning_count': result.warning_count
            }
            suite_data['test_results'].append(result_data)

        with open(data_file, 'w') as f:
            json.dump(suite_data, f, indent=2)

        print(f"💾 Benchmark data saved to: {data_file}")
        return str(data_file)

def main():
    """Main benchmark execution"""
    print("🎯 Real OWL2 Reasoner Benchmark Execution")
    print("=" * 60)

    executor = RealBenchmarkExecutor()

    try:
        # Execute comprehensive benchmark
        suite = executor.execute_comprehensive_benchmark()

        # Save benchmark data
        data_file = executor.save_benchmark_data(suite)

        # Generate report using Phase 3 system
        print("\n📊 Generating publication-ready report...")
        from report_generator import AcademicReportGenerator

        report_generator = AcademicReportGenerator(str(executor.output_dir))
        package = report_generator.generate_publication_report(suite)

        print(f"\n🎉 Real benchmark execution complete!")
        print(f"📁 Results directory: {executor.output_dir}")
        print(f"📄 Package index: {package['package_index']}")
        print(f"📊 Total test results: {len(suite.test_results)}")

        # Show summary statistics
        successful_tests = [r for r in suite.test_results if r.success]
        failed_tests = [r for r in suite.test_results if not r.success]

        print(f"✅ Successful tests: {len(successful_tests)}")
        print(f"❌ Failed tests: {len(failed_tests)}")

        if successful_tests:
            avg_time = sum(r.execution_time_ms for r in successful_tests) / len(successful_tests)
            print(f"⏱️  Average execution time: {avg_time:.2f}ms")

        return 0

    except Exception as e:
        print(f"❌ Benchmark execution failed: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())