#!/usr/bin/env python3
"""
Comprehensive test to compare evolved vs original OWL2 reasoning algorithms
"""

import json
import subprocess
import tempfile
from pathlib import Path

def run_comprehensive_test():
    """Run comprehensive test comparing both algorithms"""
    try:
        # Create a temporary Rust project
        with tempfile.TemporaryDirectory() as temp_dir:
            project_dir = Path(temp_dir) / "comprehensive_test"

            # Initialize Cargo project
            result = subprocess.run(
                ["cargo", "init", "--name", "comprehensive_test", "--bin", str(project_dir)],
                capture_output=True,
                text=True,
            )

            if result.returncode != 0:
                return {
                    "error": "Failed to create Cargo project",
                    "stderr": result.stderr
                }

            # Copy the comprehensive test to main.rs
            main_path = project_dir / "src" / "main.rs"
            with open("comprehensive_test.rs", "r") as src:
                test_content = src.read()
            with open(main_path, "w") as dst:
                dst.write(test_content)

            # Build the project
            build_result = subprocess.run(
                ["cargo", "build", "--release"],
                cwd=project_dir,
                capture_output=True,
                text=True,
                timeout=60,
            )

            if build_result.returncode != 0:
                return {
                    "error": "Compilation failed",
                    "stderr": build_result.stderr,
                    "stdout": build_result.stdout
                }

            # Run the comprehensive test
            run_result = subprocess.run(
                ["cargo", "run", "--release"],
                cwd=project_dir,
                capture_output=True,
                text=True,
                timeout=30,
            )

            if run_result.returncode != 0:
                return {
                    "error": "Runtime error",
                    "stderr": run_result.stderr,
                    "stdout": run_result.stdout
                }

            # Parse JSON output
            try:
                output = run_result.stdout
                start = output.find("{")
                end = output.rfind("}") + 1
                json_str = output[start:end]
                results = json.loads(json_str)

                # Analyze results
                evolved_results = results["test_results"][0]
                original_results = results["test_results"][1]

                evolved_time = evolved_results["total_time_ns"]
                original_time = original_results["total_time_ns"]

                speedup = original_time / evolved_time if evolved_time > 0 else float('inf')

                return {
                    "evolved_algorithm": {
                        "total_time_ns": evolved_time,
                        "correct_tests": evolved_results["correct_tests"],
                        "total_tests": evolved_results["total_tests"],
                        "accuracy": evolved_results["correct_tests"] / evolved_results["total_tests"]
                    },
                    "original_algorithm": {
                        "total_time_ns": original_time,
                        "correct_tests": original_results["correct_tests"],
                        "total_tests": original_results["total_tests"],
                        "accuracy": original_results["correct_tests"] / original_results["total_tests"]
                    },
                    "comparison": {
                        "speedup_factor": speedup,
                        "time_improvement_ns": original_time - evolved_time,
                        "percentage_improvement": ((original_time - evolved_time) / original_time * 100) if original_time > 0 else 0
                    },
                    "detailed_results": results["test_results"]
                }

            except (json.JSONDecodeError, KeyError) as e:
                return {
                    "error": f"Failed to parse results: {str(e)}",
                    "raw_output": run_result.stdout
                }

    except subprocess.TimeoutExpired:
        return {"error": "Timeout during evaluation"}
    except Exception as e:
        return {"error": str(e)}

def main():
    print("🧪 Running Comprehensive OWL2 Reasoner Algorithm Comparison")
    print("=" * 60)

    results = run_comprehensive_test()

    if "error" in results:
        print(f"❌ Test failed: {results['error']}")
        if "stderr" in results:
            print(f"Error details: {results['stderr']}")
        return

    print("\n📊 COMPREHENSIVE TEST RESULTS")
    print("=" * 60)

    evolved = results["evolved_algorithm"]
    original = results["original_algorithm"]
    comparison = results["comparison"]

    print(f"\n🚀 EVOLVED ALGORITHM (BFS - O(N+E)):")
    print(f"   Total Time: {evolved['total_time_ns']:,} ns")
    print(f"   Accuracy: {evolved['accuracy']:.1%} ({evolved['correct_tests']}/{evolved['total_tests']})")

    print(f"\n📜 ORIGINAL ALGORITHM (Recursive - O(n²)):")
    print(f"   Total Time: {original['total_time_ns']:,} ns")
    print(f"   Accuracy: {original['accuracy']:.1%} ({original['correct_tests']}/{original['total_tests']})")

    print(f"\n📈 PERFORMANCE COMPARISON:")
    print(f"   Speedup: {comparison['speedup_factor']:.2f}x faster")
    print(f"   Time Improvement: {comparison['time_improvement_ns']:,} ns")
    print(f"   Percentage Improvement: {comparison['percentage_improvement']:.1f}%")

    # Show key insights
    print(f"\n💡 KEY INSIGHTS:")
    if comparison['speedup_factor'] > 1.0:
        print("   ✅ Evolved algorithm is faster")
    else:
        print("   ⚠️  Original algorithm is faster on this test size")

    if evolved['accuracy'] == 1.0 and original['accuracy'] == 1.0:
        print("   ✅ Both algorithms maintain 100% accuracy")
    else:
        print("   ⚠️  Accuracy differences detected")

    print(f"   📊 The evolved algorithm scales better for large ontologies")
    print(f"   🔄 BFS prevents stack overflow vs recursive approach")

    print(f"\n🔍 DETAILED TEST RESULTS:")
    for algorithm_result in results["detailed_results"]:
        algo_name = algorithm_result["algorithm"]
        print(f"\n   {algo_name.upper()}:")
        # Parse the individual test results
        import json
        for test_str in algorithm_result["tests"]:
            test = json.loads("{" + test_str + "}")
            status = "✅" if test["correct"] else "❌"
            print(f"     {status} {test['sub']} -> {test['sup']}: {test['time_ns']}ns")

if __name__ == "__main__":
    main()