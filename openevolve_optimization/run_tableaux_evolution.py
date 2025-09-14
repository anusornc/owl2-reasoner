#!/usr/bin/env python3
"""
OpenEvolve runner for tableaux algorithm optimization with corrected baseline
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from openevolve import run_evolution
from openevolve.config import Config, LLMModelConfig

def main():
    print("🚀 Starting OpenEvolve for tableaux algorithm optimization (Corrected Baseline)...")
    print("=" * 70)

    # Create Google AI LLM configuration
    google_llm = LLMModelConfig(
        name="gemini-2.5-flash",
        api_key="AIzaSyDjhnGSNOZWlc6D64fC8MNhDK8o3RE6lj4",
        api_base="https://generativelanguage.googleapis.com/v1beta/openai/",
        temperature=0.7,
        max_tokens=4000,
        timeout=120,
        retries=3,
        retry_delay=5,
        weight=1.0,
        system_message="""You are an expert Rust programmer specializing in algorithm optimization for OWL2 reasoning systems.

Your task is to evolve the tableaux reasoning algorithm for optimal performance, memory efficiency, and scalability.

Focus on:
1. Advanced blocking strategies to minimize backtracking
2. Dependency-directed backtracking for efficient contradiction resolution
3. Parallel processing opportunities for independent tableaux branches
4. Memory-efficient data structures and caching strategies
5. Early pruning of non-productive search paths
6. Optimized rule application ordering

Return only the complete, optimized Rust code."""
    )

    # Create configuration
    config = Config()
    config.max_iterations = 50  # Reasonable number of iterations for testing
    config.llm.models = [google_llm]
    config.llm.temperature = 0.6  # Lower temperature for more focused changes
    config.llm.max_tokens = 3000  # Reduced to prevent overly long responses
    config.llm.timeout = 180  # Increased timeout for complex generations

    # Database settings
    config.database.population_size = 30  # Smaller population for better quality control
    config.database.num_islands = 4  # Multiple islands for diversity
    config.database.feature_dimensions = ["correctness_score", "performance_score", "memory_efficiency_score", "scalability_score"]

    # Set maximum code length to prevent overflow
    config.max_code_length = 8000  # Prevent extremely long programs

    # Evaluation settings
    config.evaluator.timeout = 180
    config.evaluator.parallel_evaluations = 2  # Reduced to avoid timeout issues

    print("🔧 Configuration created successfully")
    print("🎯 Starting evolution process...")
    print(f"📊 Target: Beat baseline score of 264.04")
    print(f"📈 Current baseline: Perfect correctness (1.0), 477ns avg time")
    print("=" * 70)

    try:
        # Run the evolution
        result = run_evolution(
            initial_program="initial_program.rs",
            evaluator="evaluator.py",
            config=config,
            iterations=50,
            output_dir="openevolve_output_corrected",
            cleanup=False
        )

        print("\n" + "=" * 70)
        print("🎯 Evolution Complete!")
        print("=" * 70)

        # Print results
        print(f"📊 Best Score: {result.best_score:.4f}")
        print(f"🏆 Best Program saved to: {result.output_dir}/best/")
        print(f"📁 Evolution output: {result.output_dir}")

        # Print detailed metrics
        if result.metrics:
            print(f"\n📈 Detailed Metrics:")
            for key, value in result.metrics.items():
                print(f"   {key}: {value}")

        # Compare with baseline
        baseline_score = 264.04
        improvement = ((result.best_score - baseline_score) / baseline_score) * 100
        print(f"\n📊 Baseline Comparison:")
        print(f"   Baseline Score: {baseline_score:.4f}")
        print(f"   Evolved Score: {result.best_score:.4f}")
        print(f"   Improvement: {improvement:+.2f}%")

        if result.best_score > baseline_score:
            print("   ✅ Evolution successful - improvement achieved!")
        else:
            print("   ⚠️  No improvement - further optimization needed")

    except Exception as e:
        print(f"❌ Evolution failed with error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()