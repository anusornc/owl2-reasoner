#!/usr/bin/env python3
"""
Real External OWL2 Reasoner Benchmark
===================================

This script provides a proper external benchmarking system that:
1. Tests our OWL2-Reasoner against established Java reasoners
2. Uses the same test ontologies for all reasoners
3. Measures actual reasoning performance (consistency checking, classification)
4. Provides real performance comparison data

Usage: python3 real_external_benchmark.py
"""

import subprocess
import time
import json
import os
import sys
from pathlib import Path
from datetime import datetime
import statistics

class ExternalBenchmarkRunner:
    def __init__(self):
        self.results = {
            "timestamp": datetime.now().isoformat(),
            "hardware": {
                "platform": subprocess.run(["uname", "-m"], capture_output=True, text=True).stdout.strip(),
                "os": subprocess.run(["uname", "-s"], capture_output=True, text=True).stdout.strip()
            },
            "reasoners": {},
            "test_ontologies": []
        }

        # Test ontologies for benchmarking
        self.test_ontologies = [
            {
                "name": "Small Family",
                "file": "test_suite/family_test.ttl",
                "description": "Simple family relationships ontology"
            },
            {
                "name": "Biomedical",
                "file": "test_suite/biomedical_test.ttl",
                "description": "Biomedical domain ontology"
            },
            {
                "name": "Complex Expressions",
                "file": "test_suite/complex_expressions.ttl",
                "description": "Complex class expressions ontology"
            },
            {
                "name": "Classification",
                "file": "test_suite/classification_test.rdf",
                "description": "Class hierarchy classification"
            }
        ]

        # Reasoner configurations
        self.reasoners = {
            "owl2-reasoner": {
                "name": "OWL2-Reasoner (Rust)",
                "command": self.test_owl2_reasoner,
                "enabled": True
            },
            "elk": {
                "name": "ELK (Java)",
                "command": self.test_elk_reasoner,
                "enabled": True
            },
            "hermit": {
                "name": "HermiT (Java)",
                "command": self.test_hermit_reasoner,
                "enabled": True
            },
            "jfact": {
                "name": "JFact (Java)",
                "command": self.test_jfact_reasoner,
                "enabled": True
            }
        }

    def test_owl2_reasoner(self, ontology_file):
        """Test our OWL2-Reasoner with consistency checking"""
        try:
            start_time = time.time()

            # Run our reasoner with the specific ontology
            result = subprocess.run([
                "cargo", "run", "--example", "consistency_benchmark",
                "--", ontology_file
            ], capture_output=True, text=True, timeout=30)

            end_time = time.time()
            execution_time = (end_time - start_time) * 1000  # Convert to milliseconds

            if result.returncode == 0:
                # Parse output for consistency result and timing
                output = result.stdout
                if "✅ Consistent" in output:
                    return {
                        "success": True,
                        "consistent": True,
                        "execution_time_ms": execution_time,
                        "output": output.strip()
                    }
                elif "❌ Inconsistent" in output:
                    return {
                        "success": True,
                        "consistent": False,
                        "execution_time_ms": execution_time,
                        "output": output.strip()
                    }

            return {
                "success": False,
                "error": result.stderr or result.stdout,
                "execution_time_ms": execution_time
            }

        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "error": "Timeout (30s)",
                "execution_time_ms": 30000
            }
        except Exception as e:
            return {
                "success": False,
                "error": str(e),
                "execution_time_ms": 0
            }

    def test_elk_reasoner(self, ontology_file):
        """Test ELK reasoner"""
        try:
            elk_jar = "benchmarking/established_reasoners/elk-distribution-cli-0.6.0/elk.jar"

            if not os.path.exists(elk_jar):
                return {
                    "success": False,
                    "error": f"ELK JAR not found: {elk_jar}",
                    "execution_time_ms": 0
                }

            start_time = time.time()

            # Test consistency checking with ELK
            result = subprocess.run([
                "java", "-jar", elk_jar,
                "-i", ontology_file,
                "-s"  # consistency check
            ], capture_output=True, text=True, timeout=30)

            end_time = time.time()
            execution_time = (end_time - start_time) * 1000

            # ELK returns 0 for consistent, 1 for inconsistent
            return {
                "success": True,
                "consistent": result.returncode == 0,
                "execution_time_ms": execution_time,
                "output": result.stdout.strip(),
                "error": result.stderr.strip() if result.stderr else None
            }

        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "error": "Timeout (30s)",
                "execution_time_ms": 30000
            }
        except Exception as e:
            return {
                "success": False,
                "error": str(e),
                "execution_time_ms": 0
            }

    def test_hermit_reasoner(self, ontology_file):
        """Test HermiT reasoner"""
        try:
            hermit_jar = "benchmarking/established_reasoners/org.semanticweb.HermiT.jar"

            if not os.path.exists(hermit_jar):
                return {
                    "success": False,
                    "error": f"HermiT JAR not found: {hermit_jar}",
                    "execution_time_ms": 0
                }

            start_time = time.time()

            # Test with HermiT (requires OWLAPI dependencies)
            result = subprocess.run([
                "java", "-cp", hermit_jar,
                "org.semanticweb.HermiT.cli.CommandLine",
                "-i", ontology_file
            ], capture_output=True, text=True, timeout=30)

            end_time = time.time()
            execution_time = (end_time - start_time) * 1000

            if result.returncode == 0:
                return {
                    "success": True,
                    "consistent": True,  # HermiT returns 0 if consistent
                    "execution_time_ms": execution_time,
                    "output": result.stdout.strip()
                }
            else:
                return {
                    "success": False,
                    "error": result.stderr or "HermiT execution failed",
                    "execution_time_ms": execution_time
                }

        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "error": "Timeout (30s)",
                "execution_time_ms": 30000
            }
        except Exception as e:
            return {
                "success": False,
                "error": str(e),
                "execution_time_ms": 0
            }

    def test_jfact_reasoner(self, ontology_file):
        """Test JFact reasoner"""
        try:
            jfact_jar = "benchmarking/established_reasoners/jfact-4.0.0.jar"

            if not os.path.exists(jfact_jar):
                return {
                    "success": False,
                    "error": f"JFact JAR not found: {jfact_jar}",
                    "execution_time_ms": 0
                }

            start_time = time.time()

            # JFact testing - try basic functionality
            result = subprocess.run([
                "java", "-cp", jfact_jar,
                "uk.ac.manchester.cs.jfact.JFact",
                ontology_file
            ], capture_output=True, text=True, timeout=30)

            end_time = time.time()
            execution_time = (end_time - start_time) * 1000

            return {
                "success": result.returncode == 0,
                "consistent": result.returncode == 0,
                "execution_time_ms": execution_time,
                "output": result.stdout.strip(),
                "error": result.stderr.strip() if result.stderr else None
            }

        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "error": "Timeout (30s)",
                "execution_time_ms": 30000
            }
        except Exception as e:
            return {
                "success": False,
                "error": str(e),
                "execution_time_ms": 0
            }

    def run_benchmark(self):
        """Run the complete benchmark suite"""
        print("🚀 External OWL2 Reasoner Benchmark")
        print("===================================")
        print(f"Test Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"Hardware: {self.results['hardware']['platform']}")
        print(f"OS: {self.results['hardware']['os']}")
        print()

        # Test each reasoner with each ontology
        for reasoner_key, reasoner_config in self.reasoners.items():
            if not reasoner_config["enabled"]:
                continue

            print(f"🧪 Testing {reasoner_config['name']}...")
            reasoner_results = {
                "name": reasoner_config["name"],
                "tests": [],
                "summary": {
                    "total_tests": 0,
                    "successful_tests": 0,
                    "average_time_ms": 0,
                    "success_rate": 0
                }
            }

            execution_times = []

            for ontology in self.test_ontologies:
                if not os.path.exists(ontology["file"]):
                    print(f"   ⚠️  Skipping {ontology['name']} - file not found")
                    continue

                print(f"   📄 Testing with {ontology['name']}...")

                # Test multiple runs for average
                runs = []
                for run in range(3):  # 3 runs for averaging
                    result = reasoner_config["command"](ontology["file"])
                    runs.append(result)
                    time.sleep(0.1)  # Small delay between runs

                # Calculate average performance
                successful_runs = [r for r in runs if r["success"]]

                if successful_runs:
                    avg_time = statistics.mean([r["execution_time_ms"] for r in successful_runs])
                    consistent_result = successful_runs[0]["consistent"]  # Use first successful result

                    test_result = {
                        "ontology": ontology["name"],
                        "file": ontology["file"],
                        "successful": len(successful_runs) > 0,
                        "consistent": consistent_result if len(successful_runs) > 0 else None,
                        "average_time_ms": avg_time,
                        "success_rate": len(successful_runs) / len(runs),
                        "runs": runs
                    }

                    execution_times.append(avg_time)
                    reasoner_results["summary"]["successful_tests"] += 1
                else:
                    test_result = {
                        "ontology": ontology["name"],
                        "file": ontology["file"],
                        "successful": False,
                        "consistent": None,
                        "average_time_ms": 0,
                        "success_rate": 0,
                        "runs": runs
                    }

                reasoner_results["tests"].append(test_result)
                reasoner_results["summary"]["total_tests"] += 1

                # Show individual result
                if test_result["successful"]:
                    status = "✅" if test_result["consistent"] else "❌"
                    print(f"      {status} {test_result['average_time_ms']:.2f}ms")
                else:
                    print(f"      ❌ Failed")

            # Calculate summary
            if reasoner_results["summary"]["total_tests"] > 0:
                reasoner_results["summary"]["success_rate"] = (
                    reasoner_results["summary"]["successful_tests"] /
                    reasoner_results["summary"]["total_tests"]
                ) * 100

                if execution_times:
                    reasoner_results["summary"]["average_time_ms"] = statistics.mean(execution_times)

            self.results["reasoners"][reasoner_key] = reasoner_results

            print(f"   📊 Summary: {reasoner_results['summary']['successful_tests']}/{reasoner_results['summary']['total_tests']} tests passed")
            if execution_times:
                print(f"   ⏱️  Average time: {reasoner_results['summary']['average_time_ms']:.2f}ms")
            print()

        return self.results

    def generate_report(self):
        """Generate comprehensive benchmark report"""
        report = []
        report.append("# External OWL2 Reasoner Benchmark Report")
        report.append("=" * 50)
        report.append(f"**Date:** {self.results['timestamp']}")
        report.append(f"**Hardware:** {self.results['hardware']['platform']} ({self.results['hardware']['os']})")
        report.append("")

        # Performance comparison table
        report.append("## Performance Comparison")
        report.append("")
        report.append("| Reasoner | Success Rate | Avg Time (ms) | Status |")
        report.append("|-----------|--------------|---------------|---------|")

        for reasoner_key, reasoner_data in self.results["reasoners"].items():
            success_rate = reasoner_data["summary"]["success_rate"]
            avg_time = reasoner_data["summary"]["average_time_ms"]
            status = "✅ Working" if success_rate > 50 else "⚠️ Issues" if success_rate > 0 else "❌ Failed"

            report.append(f"| {reasoner_data['name']} | {success_rate:.1f}% | {avg_time:.2f} | {status} |")

        report.append("")

        # Detailed results
        report.append("## Detailed Results")
        report.append("")

        for reasoner_key, reasoner_data in self.results["reasoners"].items():
            report.append(f"### {reasoner_data['name']}")
            report.append("")

            for test in reasoner_data["tests"]:
                status_icon = "✅" if test["successful"] else "❌"
                consistency = "Consistent" if test["consistent"] else "Inconsistent" if test["consistent"] is not None else "Unknown"

                report.append(f"- **{test['ontology']}**: {status_icon} {consistency} ({test['average_time_ms']:.2f}ms)")

            report.append("")

        # Performance analysis
        report.append("## Performance Analysis")
        report.append("")

        # Find the best performing reasoner
        working_reasoners = {
            k: v for k, v in self.results["reasoners"].items()
            if v["summary"]["success_rate"] > 50
        }

        if working_reasoners:
            best_reasoner = min(working_reasoners.items(),
                             key=lambda x: x[1]["summary"]["average_time_ms"])

            report.append(f"🏆 **Best Performance:** {best_reasoner[1]['name']} "
                         f"({best_reasoner[1]['summary']['average_time_ms']:.2f}ms average)")

            # Compare OWL2-Reasoner specifically
            if "owl2-reasoner" in working_reasoners:
                owl2_result = working_reasoners["owl2-reasoner"]
                report.append(f"")
                report.append(f"🦀 **OWL2-Reasoner Performance:**")
                report.append(f"- Success Rate: {owl2_result['summary']['success_rate']:.1f}%")
                report.append(f"- Average Time: {owl2_result['summary']['average_time_ms']:.2f}ms")

                # Compare with best
                if best_reasoner[0] != "owl2-reasoner":
                    speedup = best_reasoner[1]["summary"]["average_time_ms"] / owl2_result["summary"]["average_time_ms"]
                    report.append(f"- Speedup vs {best_reasoner[1]['name']}: {speedup:.1f}x")
                else:
                    report.append("- 🏆 Fastest reasoner in this benchmark")

        return "\n".join(report)

    def save_results(self, filename="external_benchmark_results.json"):
        """Save benchmark results to JSON file"""
        with open(filename, 'w') as f:
            json.dump(self.results, f, indent=2)
        print(f"📄 Results saved to: {filename}")

def main():
    """Main benchmark execution"""
    runner = ExternalBenchmarkRunner()

    # Run benchmark
    results = runner.run_benchmark()

    # Generate and save report
    report = runner.generate_report()

    # Save results
    runner.save_results()

    # Print report
    print("\n" + "="*50)
    print("📊 BENCHMARK RESULTS")
    print("="*50)
    print(report)

    # Save report to file
    with open("external_benchmark_report.md", 'w') as f:
        f.write(report)
    print(f"\n📄 Detailed report saved to: external_benchmark_report.md")

if __name__ == "__main__":
    main()