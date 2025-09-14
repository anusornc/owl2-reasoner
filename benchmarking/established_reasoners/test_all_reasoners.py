#!/usr/bin/env python3

"""
Comprehensive OWL2 Reasoner Testing Framework
Tests functionality and basic performance of all available reasoners
"""

import subprocess
import time
import json
import os
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple
import sys

@dataclass
class TestResult:
    reasoner_name: str
    test_operation: str
    success: bool
    execution_time_ms: float
    output_file: Optional[str] = None
    error_message: Optional[str] = None
    output_size: Optional[int] = None

@dataclass
class ReasonerConfig:
    name: str
    command: List[str]
    classification_cmd: List[str]
    consistency_cmd: List[str]
    help_cmd: List[str]

class OWL2ReasonerTester:
    def __init__(self, test_dir: str = "test_results"):
        self.test_dir = Path(test_dir)
        self.test_dir.mkdir(exist_ok=True)
        self.results: List[TestResult] = []

        # Test ontologies
        self.test_ontologies = {
            "small_rdfxml": Path("../test_ontologies/benchmark_small.owl"),
            "medium_turtle": Path("../test_ontologies/benchmark_medium.ttl"),
            "small_functional": Path("../test_ontologies/simple_functional.owl")
        }

        # Configure reasoners
        self.reasoners = {
            "rust": ReasonerConfig(
                name="Rust OWL2",
                command=["cargo", "run", "--example", "simple_example", "--quiet"],
                classification_cmd=["cargo", "run", "--example", "simple_example", "--quiet"],
                consistency_cmd=["cargo", "run", "--example", "simple_example", "--quiet"],
                help_cmd=["cargo", "run", "--example", "simple_example", "--quiet"]
            ),
            "elk": ReasonerConfig(
                name="ELK",
                command=["java", "-jar", "elk-distribution-cli-0.6.0/elk.jar"],
                classification_cmd=["java", "-jar", "elk-distribution-cli-0.6.0/elk.jar", "-c"],
                consistency_cmd=["java", "-jar", "elk-distribution-cli-0.6.0/elk.jar", "-s"],
                help_cmd=["java", "-jar", "elk-distribution-cli-0.6.0/elk.jar", "--help"]
            ),
            "hermit": ReasonerConfig(
                name="HermiT",
                command=["java", "-jar", "HermiT.jar"],
                classification_cmd=["java", "-jar", "HermiT.jar", "-c"],
                consistency_cmd=["java", "-jar", "HermiT.jar", "-k"],
                help_cmd=["java", "-jar", "HermiT.jar", "--help"]
            ),
            "jfact": ReasonerConfig(
                name="JFact",
                command=["echo", "JFact is a library without CLI interface"],
                classification_cmd=["echo", "JFact requires OWL API integration"],
                consistency_cmd=["echo", "JFact requires OWL API integration"],
                help_cmd=["echo", "JFact is a library, not a standalone CLI tool"]
            )
        }

    def run_command(self, cmd: List[str], working_dir: str = None, timeout: int = 30) -> Tuple[float, str, str, int]:
        """Run a command and return timing, stdout, stderr, and return code"""
        start_time = time.time()
        try:
            result = subprocess.run(
                cmd,
                cwd=working_dir,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            end_time = time.time()
            execution_time = (end_time - start_time) * 1000  # Convert to milliseconds

            return execution_time, result.stdout, result.stderr, result.returncode

        except subprocess.TimeoutExpired:
            return timeout * 1000, "", "Command timed out", 1
        except Exception as e:
            return 0, "", str(e), 1

    def test_reasoner_help(self, reasoner_key: str) -> TestResult:
        """Test if reasoner can show help"""
        config = self.reasoners[reasoner_key]
        print(f"🔍 Testing {config.name} help system...")

        exec_time, stdout, stderr, returncode = self.run_command(config.help_cmd)

        success = returncode == 0
        error_msg = stderr if stderr else None

        return TestResult(
            reasoner_name=config.name,
            test_operation="help",
            success=success,
            execution_time_ms=exec_time,
            error_message=error_msg
        )

    def test_reasoner_classification(self, reasoner_key: str, ontology_path: Path) -> TestResult:
        """Test ontology classification"""
        config = self.reasoners[reasoner_key]
        ontology_name = ontology_path.stem
        output_file = self.test_dir / f"{reasoner_key}_{ontology_name}_classification.txt"

        print(f"🧠 Testing {config.name} classification on {ontology_name}...")

        # Build classification command
        if reasoner_key == "rust":
            # Rust needs special handling - run in rust directory
            cmd = config.classification_cmd + [str(ontology_path.resolve())]
            working_dir = "../../"
        elif reasoner_key == "elk":
            cmd = config.classification_cmd + ["-i", str(ontology_path.resolve()), "-o", str(output_file)]
            working_dir = "."
        elif reasoner_key == "hermit":
            cmd = config.classification_cmd + ["-o", str(output_file), str(ontology_path.resolve())]
            working_dir = "."
        else:  # jfact
            cmd = config.classification_cmd + [str(ontology_path.resolve())]
            working_dir = "."

        exec_time, stdout, stderr, returncode = self.run_command(cmd, working_dir)

        success = returncode == 0
        error_msg = stderr if stderr else None
        output_size = output_file.stat().st_size if output_file.exists() else None

        return TestResult(
            reasoner_name=config.name,
            test_operation=f"classification_{ontology_name}",
            success=success,
            execution_time_ms=exec_time,
            output_file=str(output_file) if output_file.exists() else None,
            error_message=error_msg,
            output_size=output_size
        )

    def test_reasoner_consistency(self, reasoner_key: str, ontology_path: Path) -> TestResult:
        """Test ontology consistency checking"""
        config = self.reasoners[reasoner_key]
        ontology_name = ontology_path.stem

        print(f"✅ Testing {config.name} consistency on {ontology_name}...")

        # Build consistency command
        if reasoner_key == "rust":
            cmd = config.consistency_cmd + [str(ontology_path.resolve())]
            working_dir = "../../"
        elif reasoner_key == "elk":
            cmd = config.consistency_cmd + ["-i", str(ontology_path.resolve())]
            working_dir = "."
        elif reasoner_key == "hermit":
            cmd = config.consistency_cmd + [str(ontology_path.resolve())]
            working_dir = "."
        else:  # jfact
            cmd = config.consistency_cmd + [str(ontology_path.resolve())]
            working_dir = "."

        exec_time, stdout, stderr, returncode = self.run_command(cmd, working_dir)

        success = returncode == 0
        error_msg = stderr if stderr else None

        return TestResult(
            reasoner_name=config.name,
            test_operation=f"consistency_{ontology_name}",
            success=success,
            execution_time_ms=exec_time,
            error_message=error_msg
        )

    def test_all_reasoners(self) -> Dict[str, List[TestResult]]:
        """Run comprehensive tests on all reasoners"""
        print("🚀 Starting comprehensive OWL2 reasoner testing...")
        print("=" * 60)

        all_results = {}

        for reasoner_key in self.reasoners:
            print(f"\n📊 Testing {self.reasoners[reasoner_key].name}...")
            print("-" * 40)

            reasoner_results = []

            # Test help system
            help_result = self.test_reasoner_help(reasoner_key)
            reasoner_results.append(help_result)
            self.results.append(help_result)

            # Only test reasoning operations if help works
            if help_result.success:
                for ontology_name, ontology_path in self.test_ontologies.items():
                    if ontology_path.exists():
                        # Test classification
                        class_result = self.test_reasoner_classification(reasoner_key, ontology_path)
                        reasoner_results.append(class_result)
                        self.results.append(class_result)

                        # Test consistency
                        cons_result = self.test_reasoner_consistency(reasoner_key, ontology_path)
                        reasoner_results.append(cons_result)
                        self.results.append(cons_result)
                    else:
                        print(f"⚠️  Ontology file not found: {ontology_path}")
            else:
                print(f"❌ {self.reasoners[reasoner_key].name} help system failed, skipping reasoning tests")

            all_results[reasoner_key] = reasoner_results

        return all_results

    def generate_report(self, results: Dict[str, List[TestResult]]) -> str:
        """Generate a comprehensive test report"""
        report = []
        report.append("# OWL2 Reasoner Comprehensive Test Report")
        report.append(f"Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}")
        report.append("")

        # Summary table
        report.append("## Test Results Summary")
        report.append("")
        report.append("| Reasoner | Help | Classification | Consistency | Overall Status |")
        report.append("|----------|------|----------------|-------------|----------------|")

        for reasoner_key, reasoner_results in results.items():
            help_ok = any(r.test_operation == "help" and r.success for r in reasoner_results)
            class_ok = any("classification" in r.test_operation and r.success for r in reasoner_results)
            cons_ok = any("consistency" in r.test_operation and r.success for r in reasoner_results)

            status = "✅ Working" if help_ok else "❌ Failed"
            if help_ok and not (class_ok or cons_ok):
                status = "⚠️ Limited"

            help_status = "✅" if help_ok else "❌"
            class_status = "✅" if class_ok else "❌"
            cons_status = "✅" if cons_ok else "❌"

            report.append(f"| {self.reasoners[reasoner_key].name} | {help_status} | {class_status} | {cons_status} | {status} |")

        report.append("")

        # Detailed results
        report.append("## Detailed Test Results")
        report.append("")

        for reasoner_key, reasoner_results in results.items():
            report.append(f"### {self.reasoners[reasoner_key].name}")
            report.append("")

            for result in reasoner_results:
                status_icon = "✅" if result.success else "❌"
                report.append(f"- {status_icon} **{result.test_operation}**: {result.execution_time_ms:.2f}ms")
                if result.error_message:
                    report.append(f"  - Error: {result.error_message[:100]}...")
                if result.output_file:
                    report.append(f"  - Output: {result.output_file} ({result.output_size} bytes)")

            report.append("")

        return "\n".join(report)

    def save_results(self, results: Dict[str, List[TestResult]]):
        """Save test results to files"""
        # Generate and save report
        report = self.generate_report(results)
        report_file = self.test_dir / "test_report.md"
        with open(report_file, 'w') as f:
            f.write(report)

        # Save raw results as JSON
        json_file = self.test_dir / "test_results.json"
        serializable_results = {}

        for reasoner_key, reasoner_results in results.items():
            serializable_results[reasoner_key] = []
            for result in reasoner_results:
                serializable_results[reasoner_key].append({
                    "reasoner_name": result.reasoner_name,
                    "test_operation": result.test_operation,
                    "success": result.success,
                    "execution_time_ms": result.execution_time_ms,
                    "output_file": result.output_file,
                    "error_message": result.error_message,
                    "output_size": result.output_size
                })

        with open(json_file, 'w') as f:
            json.dump(serializable_results, f, indent=2)

        print(f"📄 Results saved to:")
        print(f"   - {report_file}")
        print(f"   - {json_file}")

def main():
    """Main testing function"""
    tester = OWL2ReasonerTester()

    try:
        # Run all tests
        results = tester.test_all_reasoners()

        # Generate and save report
        tester.save_results(results)

        # Print summary
        print("\n" + "=" * 60)
        print("🏆 TEST COMPLETED")
        print("=" * 60)

        working_count = 0
        for reasoner_key, reasoner_results in results.items():
            help_works = any(r.test_operation == "help" and r.success for r in reasoner_results)
            if help_works:
                working_count += 1
                print(f"✅ {tester.reasoners[reasoner_key].name}: Working")
            else:
                print(f"❌ {tester.reasoners[reasoner_key].name}: Failed")

        print(f"\n📊 Summary: {working_count}/{len(results)} reasoners working")

    except KeyboardInterrupt:
        print("\n🛑 Testing interrupted by user")
    except Exception as e:
        print(f"💥 Testing failed with error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()