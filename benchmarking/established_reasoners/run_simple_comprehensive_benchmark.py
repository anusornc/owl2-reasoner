#!/usr/bin/env python3
"""
Simple Comprehensive OWL2 Reasoner Benchmark
Runs all 5 reasoners and collects timing data
"""

import os
import sys
import subprocess
import time
import json
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any
import platform

class SimpleBenchmarkRunner:
    def __init__(self):
        self.results = []
        self.base_dir = Path(__file__).parent

        # Define all reasoners
        self.reasoners = {
            'elk': {
                'name': 'ELK',
                'command': ['java', '-jar', 'elk-distribution-cli-0.6.0/elk.jar'],
                'args': {
                    'classification': ['--classify', '--input'],
                    'consistency': ['--consistent', '--input']
                }
            },
            'hermit': {
                'name': 'HermiT',
                'command': ['java', '-cp', 'org.semanticweb.HermiT.jar:HermiT/project/lib/owlapi-3.4.3.jar:HermiT/project/lib/axiom-1.2.8.jar:HermiT/project/lib/commons-logging-1.1.1.jar', 'org.semanticweb.HermiT.cli.CommandLine'],
                'args': {
                    'classification': ['-c'],
                    'consistency': ['-k']
                }
            },
            'jfact': {
                'name': 'JFact',
                'command': ['java', '-jar', 'jfact-4.0.0.jar'],
                'args': {
                    'classification': [''],
                    'consistency': ['']
                }
            },
            'pellet': {
                'name': 'Pellet',
                'command': ['./pellet-2.3.1/pellet.sh'],
                'args': {
                    'classification': ['classify'],
                    'consistency': ['consistency']
                }
            },
            'owl2-reasoner': {
                'name': 'OWL2-Reasoner',
                'command': ['./owl2-reasoner-cli'],
                'args': {
                    'classification': ['--classify'],
                    'consistency': ['--consistent']
                }
            }
        }

        # Define benchmarks
        self.benchmarks = {
            'test_simple_ttl': {
                'ontology': 'test_simple.ttl',
                'description': 'Simple test ontology (Turtle)'
            },
            'test_simple_owl': {
                'ontology': 'test_simple.owl',
                'description': 'Simple test ontology (OWL)'
            },
            'lubm_base_ttl': {
                'ontology': 'lubm/univ-bench.ttl',
                'description': 'LUBM base ontology (Turtle)'
            },
            'lubm_base_owl': {
                'ontology': 'lubm_base.owl',
                'description': 'LUBM base ontology (OWL)'
            }
        }

    def run_single_test(self, reasoner_key: str, benchmark_key: str, operation: str) -> Dict[str, Any]:
        """Run a single test"""
        reasoner = self.reasoners[reasoner_key]
        benchmark = self.benchmarks[benchmark_key]

        print(f"🧪 Running {reasoner['name']} on {benchmark_key} {operation}...")

        start_time = time.time()

        try:
            # Prepare command
            if operation == 'classification':
                cmd = reasoner['command'] + reasoner['args']['classification'] + [benchmark['ontology']]
            else:  # consistency
                cmd = reasoner['command'] + reasoner['args']['consistency'] + [benchmark['ontology']]

            # Execute command
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=60,  # 1 minute timeout
                cwd=str(self.base_dir)
            )

            end_time = time.time()
            execution_time = end_time - start_time

            # Determine success
            success = result.returncode == 0

            test_result = {
                'reasoner_name': reasoner['name'],
                'reasoner_key': reasoner_key,
                'benchmark_key': benchmark_key,
                'benchmark_description': benchmark['description'],
                'operation': operation,
                'execution_time_seconds': execution_time,
                'execution_time_ms': execution_time * 1000,
                'success': success,
                'return_code': result.returncode,
                'error_message': result.stderr if result.stderr else None,
                'output': result.stdout[:500] if result.stdout else None,  # Truncate long output
                'timestamp': datetime.now().isoformat(),
                'platform': platform.system()
            }

            print(f"  ✅ Completed in {execution_time:.3f}s - {'Success' if success else 'Failed'}")
            if result.stderr:
                print(f"  ⚠️  Error: {result.stderr[:200]}...")

            return test_result

        except subprocess.TimeoutExpired:
            print(f"  ⏰ Timeout after 60 seconds")
            return {
                'reasoner_name': reasoner['name'],
                'reasoner_key': reasoner_key,
                'benchmark_key': benchmark_key,
                'benchmark_description': benchmark['description'],
                'operation': operation,
                'execution_time_seconds': 60,
                'execution_time_ms': 60000,
                'success': False,
                'return_code': -1,
                'error_message': 'Timeout after 60 seconds',
                'output': None,
                'timestamp': datetime.now().isoformat(),
                'platform': platform.system()
            }

        except Exception as e:
            print(f"  ❌ Error: {str(e)}")
            return {
                'reasoner_name': reasoner['name'],
                'reasoner_key': reasoner_key,
                'benchmark_key': benchmark_key,
                'benchmark_description': benchmark['description'],
                'operation': operation,
                'execution_time_seconds': 0,
                'execution_time_ms': 0,
                'success': False,
                'return_code': -1,
                'error_message': str(e),
                'output': None,
                'timestamp': datetime.now().isoformat(),
                'platform': platform.system()
            }

    def run_comprehensive_benchmark(self) -> List[Dict[str, Any]]:
        """Run comprehensive benchmark with all reasoners and benchmarks"""
        print("🚀 Starting Comprehensive OWL2 Reasoner Benchmark")
        print("=" * 60)

        all_results = []

        # Collect environment info
        env_info = {
            'platform': platform.system(),
            'platform_version': platform.version(),
            'machine': platform.machine(),
            'processor': platform.processor(),
            'python_version': platform.python_version(),
            'timestamp': datetime.now().isoformat()
        }

        # Run all combinations
        total_tests = len(self.reasoners) * len(self.benchmarks) * 2  # 2 operations per benchmark
        print(f"📊 Total tests to run: {total_tests} ({len(self.reasoners)} reasoners × {len(self.benchmarks)} benchmarks × 2 operations)")
        test_count = 0

        for reasoner_key in self.reasoners:
            for benchmark_key in self.benchmarks:
                for operation in ['classification', 'consistency']:
                    test_count += 1
                    print(f"\n📊 Test {test_count}/{total_tests}: {reasoner_key} on {benchmark_key} ({operation})")

                    result = self.run_single_test(reasoner_key, benchmark_key, operation)
                    all_results.append(result)

        # Create benchmark summary
        benchmark_summary = {
            'benchmark_name': 'Comprehensive OWL2 Reasoner Benchmark',
            'description': 'Benchmark comparing 5 OWL2 reasoners',
            'environment': env_info,
            'total_tests': total_tests,
            'successful_tests': sum(1 for r in all_results if r['success']),
            'failed_tests': sum(1 for r in all_results if not r['success']),
            'results': all_results,
            'timestamp': datetime.now().isoformat()
        }

        print(f"\n🎯 Benchmark completed!")
        print(f"   Total tests: {total_tests}")
        print(f"   Successful: {benchmark_summary['successful_tests']}")
        print(f"   Failed: {benchmark_summary['failed_tests']}")

        return benchmark_summary

    def save_results(self, benchmark_summary: Dict[str, Any], filename: str = None):
        """Save benchmark results to file"""
        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"comprehensive_benchmark_{timestamp}.json"

        filepath = self.base_dir / "results" / filename

        # Create results directory if it doesn't exist
        filepath.parent.mkdir(exist_ok=True)

        with open(filepath, 'w') as f:
            json.dump(benchmark_summary, f, indent=2)

        print(f"💾 Results saved to: {filepath}")
        return filepath

    def print_summary(self, benchmark_summary: Dict[str, Any]):
        """Print summary statistics"""
        print(f"\n📈 Summary Statistics:")

        reasoner_stats = {}
        for result in benchmark_summary['results']:
            reasoner_name = result['reasoner_name']
            if reasoner_name not in reasoner_stats:
                reasoner_stats[reasoner_name] = {'total': 0, 'success': 0, 'total_time': 0, 'successful_times': []}

            reasoner_stats[reasoner_name]['total'] += 1
            if result['success']:
                reasoner_stats[reasoner_name]['success'] += 1
                reasoner_stats[reasoner_name]['total_time'] += result['execution_time_ms']
                reasoner_stats[reasoner_name]['successful_times'].append(result['execution_time_ms'])

        # Print stats for each reasoner
        for reasoner, stats in reasoner_stats.items():
            success_rate = (stats['success'] / stats['total']) * 100
            avg_time = stats['total_time'] / stats['success'] if stats['success'] > 0 else 0

            print(f"   {reasoner}:")
            print(f"     Tests: {stats['success']}/{stats['total']} ({success_rate:.1f}%)")
            print(f"     Avg Time: {avg_time:.2f}ms")
            if stats['successful_times']:
                print(f"     Min Time: {min(stats['successful_times']):.2f}ms")
                print(f"     Max Time: {max(stats['successful_times']):.2f}ms")

        # Find best performer
        best_reasoner = None
        best_avg_time = float('inf')

        for reasoner, stats in reasoner_stats.items():
            if stats['success'] > 0:
                avg_time = stats['total_time'] / stats['success']
                if avg_time < best_avg_time:
                    best_avg_time = avg_time
                    best_reasoner = reasoner

        if best_reasoner:
            print(f"\n🏆 Best Performer: {best_reasoner} (avg: {best_avg_time:.2f}ms)")

def main():
    """Main execution function"""
    runner = SimpleBenchmarkRunner()

    try:
        # Run comprehensive benchmark
        benchmark_summary = runner.run_comprehensive_benchmark()

        # Save results
        results_file = runner.save_results(benchmark_summary)

        # Print summary
        runner.print_summary(benchmark_summary)

        print(f"\n🎉 Comprehensive benchmark completed successfully!")
        print(f"📊 Results file: {results_file}")

        return benchmark_summary

    except KeyboardInterrupt:
        print("\n⚠️  Benchmark interrupted by user")
        return None
    except Exception as e:
        print(f"\n❌ Benchmark failed: {str(e)}")
        return None

if __name__ == "__main__":
    main()