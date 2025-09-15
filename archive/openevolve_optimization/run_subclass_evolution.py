#!/usr/bin/env python3
"""
Run OpenEvolve for SimpleReasoner subclass checking optimization
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from openevolve import run_evolution
from subclass_config_proper import create_config

def main():
    print("🚀 Starting OpenEvolve for SimpleReasoner subclass checking optimization...")
    print("=" * 60)

    # Create proper configuration
    config = create_config()

    # Run the evolution
    result = run_evolution(
        initial_program="subclass_initial_program.rs",
        evaluator="subclass_evaluator.py",
        config=config,
        iterations=30,
        output_dir="subclass_evolution_output",
        cleanup=False
    )

    print("\n" + "=" * 60)
    print("🎯 Evolution Complete!")
    print("=" * 60)

    # Print results
    print(f"📊 Best Score: {result.best_score:.4f}")
    print(f"🏆 Best Program saved to: {result.output_dir}/best/")
    print(f"📁 Evolution output: {result.output_dir}")

    # Print detailed metrics
    if result.metrics:
        print(f"\n📈 Detailed Metrics:")
        for key, value in result.metrics.items():
            print(f"  {key}: {value}")

    # Show the best code snippet
    if result.best_code:
        print(f"\n💻 Best Algorithm (first 20 lines):")
        best_lines = result.best_code.split('\n')[:20]
        for i, line in enumerate(best_lines, 1):
            print(f"  {i:2d}: {line}")
        if len(result.best_code.split('\n')) > 20:
            print("  ... (truncated)")

    print(f"\n✅ Evolution completed successfully!")
    print(f"🔍 Check the output directory for detailed results: {result.output_dir}")

if __name__ == "__main__":
    main()