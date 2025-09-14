#!/usr/bin/env python3
"""
Comprehensive test suite for Phase 1 & Phase 2 components
Validating memory profiling, environment collection, data structures,
LUBM/SP2B benchmarks, and statistical analysis capabilities
"""

import os
import sys
import time
import json
import tempfile
import shutil
from pathlib import Path
from typing import Dict, List, Any

# Import our testing framework components
from enhanced_data_structures import (
    PublicationTestResult, TestOperation, ReasonerType, BenchmarkType,
    BenchmarkSuite, EnhancedDataAnalyzer
)
from memory_profiler import CrossPlatformMemoryProfiler, MemoryAnalysisEngine
from environment_collector import EnvironmentCollector, EnvironmentSpecification
from setup_lubm_benchmark import LUBMSetup
from setup_sp2b_benchmark import SP2BSetup
from statistical_analysis_engine import StatisticalAnalysisEngine

class Phase1Phase2Tester:
    """Comprehensive tester for Phase 1 & Phase 2 components"""

    def __init__(self):
        self.test_results = []
        self.test_directory = Path(tempfile.mkdtemp(prefix="owl2_test_"))
        print(f"🧪 Testing directory: {self.test_directory}")

    def run_comprehensive_tests(self) -> Dict[str, Any]:
        """Run all Phase 1 & Phase 2 tests"""
        print("🚀 Starting comprehensive Phase 1 & Phase 2 testing...")

        # Test 1: Memory Profiling System
        memory_test_result = self.test_memory_profiling()
        self.test_results.append(("Memory Profiling", memory_test_result))

        # Test 2: Environment Collection
        env_test_result = self.test_environment_collection()
        self.test_results.append(("Environment Collection", env_test_result))

        # Test 3: Enhanced Data Structures
        data_test_result = self.test_enhanced_data_structures()
        self.test_results.append(("Enhanced Data Structures", data_test_result))

        # Test 4: LUBM Benchmark Setup
        lubm_test_result = self.test_lubm_benchmark()
        self.test_results.append(("LUBM Benchmark", lubm_test_result))

        # Test 5: SP2B Benchmark Setup
        sp2b_test_result = self.test_sp2b_benchmark()
        self.test_results.append(("SP2B Benchmark", sp2b_test_result))

        # Test 6: Statistical Analysis Engine
        stats_test_result = self.test_statistical_analysis()
        self.test_results.append(("Statistical Analysis", stats_test_result))

        # Generate comprehensive report
        report = self.generate_test_report()

        # Cleanup
        self.cleanup()

        return report

    def test_memory_profiling(self) -> Dict[str, Any]:
        """Test memory profiling system"""
        print("\n📊 Testing Memory Profiling System...")

        try:
            # Initialize memory profiler
            profiler = CrossPlatformMemoryProfiler(sampling_interval=0.05)

            # Test basic functionality - mock process info for testing
            print("  📈 Testing basic memory monitoring...")
            process_info = {"pid": os.getpid(), "name": "test_process"}
            profiler.start_monitoring(process_info)
            time.sleep(0.5)  # Monitor for 500ms
            memory_metrics = profiler.stop_monitoring()

            # Validate memory data
            if not memory_metrics.memory_timeline:
                return {"status": "FAILED", "error": "No memory data collected"}

            if len(memory_metrics.memory_timeline) < 5:
                return {"status": "FAILED", "error": "Insufficient memory samples"}

            # Test memory analysis
            analysis_engine = MemoryAnalysisEngine()
            analysis = analysis_engine.analyze_efficiency(memory_metrics)

            required_fields = ['peak_memory_mb', 'average_memory_mb', 'memory_efficiency_triples_per_mb']
            for field in required_fields:
                if field not in analysis:
                    return {"status": "FAILED", "error": f"Missing analysis field: {field}"}

            print(f"  ✅ Memory profiling successful - Peak: {analysis['peak_memory_mb']:.2f} MB")

            return {
                "status": "PASSED",
                "samples_collected": len(memory_metrics.memory_timeline),
                "peak_memory_mb": analysis['peak_memory_mb'],
                "average_memory_mb": analysis['average_memory_mb'],
                "efficiency_score": analysis['memory_efficiency_triples_per_mb']
            }

        except Exception as e:
            return {"status": "FAILED", "error": str(e)}

    def test_environment_collection(self) -> Dict[str, Any]:
        """Test environment specification collection"""
        print("\n🖥️  Testing Environment Collection...")

        try:
            collector = EnvironmentCollector()
            spec = collector.collect_complete_specification()

            # Validate required fields
            required_fields = ['cpu_cores', 'processor', 'total_memory', 'os_name', 'python_version']
            for field in required_fields:
                if not hasattr(spec, field):
                    return {"status": "FAILED", "error": f"Missing field: {field}"}

            # Validate hardware specs
            if not hasattr(spec, 'cpu_cores') or spec.cpu_cores <= 0:
                return {"status": "FAILED", "error": "Invalid CPU cores"}

            if not hasattr(spec, 'processor') or not spec.processor:
                return {"status": "FAILED", "error": "Missing processor info"}

            # Test basic environment data
            if not spec.cpu_cores or not spec.processor:
                return {"status": "FAILED", "error": "Invalid environment data"}

            print(f"  ✅ Environment collection successful - CPU: {spec.cpu_cores} cores, Memory: {spec.total_memory}")

            return {
                "status": "PASSED",
                "cpu_cores": spec.cpu_cores,
                "total_memory": spec.total_memory,
                "processor": spec.processor,
                "python_version": spec.python_version
            }

        except Exception as e:
            return {"status": "FAILED", "error": str(e)}

    def test_enhanced_data_structures(self) -> Dict[str, Any]:
        """Test enhanced data structures"""
        print("\n🏗️  Testing Enhanced Data Structures...")

        try:
            # Test PublicationTestResult creation
            from datetime import datetime
            test_result = PublicationTestResult(
                reasoner_name="TestReasoner",
                reasoner_type=ReasonerType.LIBRARY,
                benchmark_type=BenchmarkType.LUBM,
                test_operation=TestOperation.CLASSIFICATION,
                ontology_file="test.ttl",
                ontology_format="turtle",
                test_timestamp=datetime.now().isoformat(),
                success=True,
                execution_time_ms=150.5,
                return_code=0,
                timeout_occurred=False,
                output_file="test_output.txt",
                output_size_bytes=1024,
                output_lines=50,
                error_message=None,
                warning_count=0
            )

            # Validate basic fields
            if test_result.execution_time_ms <= 0:
                return {"status": "FAILED", "error": "Invalid execution time"}

            # Test BenchmarkSuite
            from datetime import datetime
            suite = BenchmarkSuite(
                suite_name="TestSuite",
                benchmark_type=BenchmarkType.CUSTOM,
                description="Test benchmark suite",
                version="1.0",
                test_results=[test_result],
                environment_spec=None,
                collection_timestamp=datetime.now().isoformat()
            )

            if suite.statistical_summary['total_tests'] != 1:
                return {"status": "FAILED", "error": "Suite counting failed"}

            success_rate = suite.statistical_summary['successful_tests'] / suite.statistical_summary['total_tests']
            if success_rate != 1.0:
                return {"status": "FAILED", "error": "Success rate calculation failed"}

            print(f"  ✅ Enhanced data structures successful - Execution: {test_result.execution_time_ms:.2f}ms, Success: {test_result.success}")

            return {
                "status": "PASSED",
                "execution_time_ms": test_result.execution_time_ms,
                "success": test_result.success,
                "success_rate": success_rate
            }

        except Exception as e:
            return {"status": "FAILED", "error": str(e)}

    def test_lubm_benchmark(self) -> Dict[str, Any]:
        """Test LUBM benchmark setup"""
        print("\n🎓 Testing LUBM Benchmark Setup...")

        try:
            # Create default configuration
            from setup_lubm_benchmark import LUBMConfiguration
            config = LUBMConfiguration()
            setup = LUBMSetup(config)

            # Test small scale setup
            print("  📚 Setting up small LUBM benchmark...")
            success = setup.setup_complete_benchmark()

            if not success:
                return {"status": "FAILED", "error": "LUBM setup failed"}

            # Validate files created in default location
            lubm_dir = Path(config.output_dir)
            required_files = ['ontology/univ-bench.ttl', 'data/scale_1/dataset.ttl', 'queries/Q1.rq']
            for file in required_files:
                if not (lubm_dir / file).exists():
                    return {"status": "FAILED", "error": f"Missing file: {file}"}

            # Validate content
            with open(lubm_dir / 'ontology/univ-bench.ttl', 'r') as f:
                ontology_content = f.read()
                if 'univ-bench:' not in ontology_content:
                    return {"status": "FAILED", "error": "Invalid ontology content"}

            with open(lubm_dir / 'data/scale_1/dataset.ttl', 'r') as f:
                data_content = f.read()
                if len(data_content.strip()) == 0:
                    return {"status": "FAILED", "error": "Empty data file"}

            print(f"  ✅ LUBM benchmark successful - {config.scales} scales")

            return {
                "status": "PASSED",
                "scales": config.scales,
                "files_created": required_files
            }

        except Exception as e:
            return {"status": "FAILED", "error": str(e)}

    def test_sp2b_benchmark(self) -> Dict[str, Any]:
        """Test SP2B benchmark setup"""
        print("\n🌐 Testing SP2B Benchmark Setup...")

        try:
            # Create default configuration
            from setup_sp2b_benchmark import SP2BConfiguration
            config = SP2BConfiguration()
            setup = SP2BSetup(config)

            # Test small scale setup
            print("  🕸️  Setting up small SP2B benchmark...")
            success = setup.setup_complete_benchmark()

            if not success:
                return {"status": "FAILED", "error": "SP2B setup failed"}

            # Validate files created in default location
            sp2b_dir = Path(config.output_dir)
            required_files = ['ontology/sp2b-social.ttl', 'data/scale_1/dataset.ttl', 'queries/Q1_Transitive.rq']
            for file in required_files:
                if not (sp2b_dir / file).exists():
                    return {"status": "FAILED", "error": f"Missing file: {file}"}

            # Validate content
            with open(sp2b_dir / 'ontology/sp2b-social.ttl', 'r') as f:
                ontology_content = f.read()
                if 'sp2b:' not in ontology_content:
                    return {"status": "FAILED", "error": "Invalid ontology content"}

            print(f"  ✅ SP2B benchmark successful - {config.scales} scales")

            return {
                "status": "PASSED",
                "scales": config.scales,
                "files_created": required_files
            }

        except Exception as e:
            return {"status": "FAILED", "error": str(e)}

    def test_statistical_analysis(self) -> Dict[str, Any]:
        """Test statistical analysis engine"""
        print("\n📈 Testing Statistical Analysis Engine...")

        try:
            engine = StatisticalAnalysisEngine()

            # Create test benchmark suite with multiple results
            from datetime import datetime
            test_results = []
            for i in range(10):
                result = PublicationTestResult(
                    reasoner_name=f"Reasoner{i%2 + 1}",
                    reasoner_type=ReasonerType.LIBRARY,
                    benchmark_type=BenchmarkType.LUBM,
                    test_operation=TestOperation.CLASSIFICATION,
                    ontology_file=f"test_{i}.ttl",
                    ontology_format="turtle",
                    test_timestamp=datetime.now().isoformat(),
                    success=True,
                    execution_time_ms=100 + (i * 10) + (hash(f"Reasoner{i%2 + 1}") % 50),  # Add some variation
                    return_code=0,
                    timeout_occurred=False,
                    output_file=f"output_{i}.txt",
                    output_size_bytes=1024,
                    output_lines=50,
                    error_message=None,
                    warning_count=0
                )
                test_results.append(result)

            suite = BenchmarkSuite(
                suite_name="TestSuite",
                benchmark_type=BenchmarkType.CUSTOM,
                description="Test suite for statistical analysis",
                version="1.0",
                test_results=test_results,
                environment_spec=None,
                collection_timestamp=datetime.now().isoformat()
            )

            # Test basic statistical analysis
            analysis = engine.analyze_benchmark_suite(suite)

            # Validate required analysis components
            required_components = ['basic_statistics', 'comparative_analysis', 'significance_testing']
            for component in required_components:
                if component not in analysis:
                    return {"status": "FAILED", "error": f"Missing analysis component: {component}"}

            # Validate basic statistics
            basic_stats = analysis['basic_statistics']
            # Check for execution time statistics under reasoner-operation keys
            execution_time_keys = [key for key in basic_stats.keys() if key.endswith('_classification')]
            if not execution_time_keys:
                return {"status": "FAILED", "error": "Missing execution time statistics"}

            # Validate that execution time stats are properly structured
            sample_key = execution_time_keys[0]
            required_time_fields = ['mean_ms', 'median_ms', 'std_dev_ms']
            for field in required_time_fields:
                if field not in basic_stats[sample_key]:
                    return {"status": "FAILED", "error": f"Missing execution time field: {field}"}

            # Test comparative analysis
            comp_analysis = analysis['comparative_analysis']
            # Check for key comparative analysis components
            required_comp_keys = ['overall_ranking', 'reasoner_scores', 'statistical_significance']
            for key in required_comp_keys:
                if key not in comp_analysis:
                    return {"status": "FAILED", "error": f"Missing comparative analysis component: {key}"}

            print(f"  ✅ Statistical analysis successful - Analyzed {len(test_results)} test results")

            return {
                "status": "PASSED",
                "test_results_analyzed": len(test_results),
                "execution_time_stats": basic_stats[sample_key],
                "comparative_results": len(comp_analysis['reasoner_scores'])
            }

        except Exception as e:
            return {"status": "FAILED", "error": str(e)}

    def generate_test_report(self) -> Dict[str, Any]:
        """Generate comprehensive test report"""
        print("\n📋 Generating Test Report...")

        passed_tests = [name for name, result in self.test_results if result['status'] == 'PASSED']
        failed_tests = [name for name, result in self.test_results if result['status'] == 'FAILED']

        report = {
            'test_summary': {
                'total_tests': len(self.test_results),
                'passed_tests': len(passed_tests),
                'failed_tests': len(failed_tests),
                'success_rate': len(passed_tests) / len(self.test_results)
            },
            'detailed_results': dict(self.test_results),
            'failed_tests_details': [(name, result) for name, result in self.test_results if result['status'] == 'FAILED'],
            'infrastructure_readiness': {
                'phase1_complete': len(passed_tests) >= 5,  # At least 5 core components working
                'phase2_complete': len([t for t in passed_tests if t in ['LUBM Benchmark', 'SP2B Benchmark']]) == 2,
                'ready_for_phase3': len(failed_tests) == 0
            }
        }

        # Save report to file
        report_file = self.test_directory / "test_report.json"
        with open(report_file, 'w') as f:
            json.dump(report, f, indent=2, default=str)

        print(f"  📊 Test Report saved to: {report_file}")
        print(f"  🎯 Success Rate: {report['test_summary']['success_rate']:.1%}")
        print(f"  ✅ Passed: {len(passed_tests)}, ❌ Failed: {len(failed_tests)}")

        return report

    def cleanup(self):
        """Clean up test directory"""
        try:
            shutil.rmtree(self.test_directory)
            print(f"🧹 Cleaned up test directory: {self.test_directory}")
        except Exception as e:
            print(f"⚠️  Cleanup warning: {e}")

def main():
    """Main test execution"""
    print("🧪 Phase 1 & Phase 2 Comprehensive Testing")
    print("=" * 50)

    tester = Phase1Phase2Tester()
    report = tester.run_comprehensive_tests()

    print("\n" + "=" * 50)
    print("🏁 TEST SUMMARY")
    print("=" * 50)

    summary = report['test_summary']
    print(f"Total Tests: {summary['total_tests']}")
    print(f"Passed: {summary['passed_tests']}")
    print(f"Failed: {summary['failed_tests']}")
    print(f"Success Rate: {summary['success_rate']:.1%}")

    readiness = report['infrastructure_readiness']
    print(f"\n📊 Infrastructure Readiness:")
    print(f"  Phase 1 Complete: {readiness['phase1_complete']}")
    print(f"  Phase 2 Complete: {readiness['phase2_complete']}")
    print(f"  Ready for Phase 3: {readiness['ready_for_phase3']}")

    if report['failed_tests_details']:
        print(f"\n❌ Failed Tests:")
        for name, result in report['failed_tests_details']:
            print(f"  - {name}: {result['error']}")

    if readiness['ready_for_phase3']:
        print(f"\n🎉 Infrastructure is ready for Phase 3 (Report Generation)!")
        return 0
    else:
        print(f"\n⚠️  Infrastructure needs fixes before proceeding to Phase 3")
        return 1

if __name__ == "__main__":
    sys.exit(main())