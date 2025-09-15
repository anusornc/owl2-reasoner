#!/usr/bin/env python3
"""
Simple OpenEvolve runner for subclass checking optimization
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from openevolve import run_evolution
from openevolve.config import Config, LLMModelConfig

def main():
    print("🚀 Starting OpenEvolve for SimpleReasoner subclass checking optimization...")
    print("=" * 60)

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

Your task is to evolve the subclass checking algorithm from an inefficient O(n²) DFS implementation
to an optimal O(N+E) BFS implementation with better performance characteristics.

Key optimization targets:
1. Replace manual DFS stack with efficient BFS queue using VecDeque
2. Use HashSet for cycle detection and visited tracking
3. Eliminate redundant computations
4. Add memoization for repeated queries
5. Improve memory efficiency
6. Handle edge cases gracefully

The algorithm must:
- Maintain 100% correctness
- Handle cycles without infinite loops
- Support equivalent classes relationships
- Be memory efficient for large ontologies
- Have clear, well-documented code

IMPORTANT: Keep the code CONCISE and under 300 lines. Focus on core algorithm improvements only.
Focus on algorithmic improvements, not just micro-optimizations.
Return only the complete, optimized Rust code."""
    )

    # Create configuration
    config = Config()
    config.max_iterations = 100  # Increased iterations for more evolution time
    config.llm.models = [google_llm]
    config.llm.temperature = 0.6  # Lower temperature for more focused changes
    config.llm.max_tokens = 3000  # Reduced to prevent overly long responses
    config.llm.timeout = 180  # Increased timeout for complex generations

    # Database settings
    config.database.population_size = 50  # Reduced population for better quality control
    config.database.num_islands = 2  # Reduced islands for more focused evolution
    config.database.feature_dimensions = ["complexity", "diversity", "score"]  # Built-in features

    # Set maximum code length to prevent overflow
    config.max_code_length = 8000  # Prevent extremely long programs

    # Evaluation settings
    config.evaluator.timeout = 180
    config.evaluator.parallel_evaluations = 2  # Reduced to avoid timeout issues

    print("🔧 Configuration created successfully")
    print("🎯 Starting evolution process...")

    try:
        # Run the evolution
        result = run_evolution(
            initial_program="subclass_initial_program.rs",
            evaluator="subclass_evaluator.py",
            config=config,
            iterations=100,
            output_dir="subclass_evolution_output_v2",
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

    except Exception as e:
        print(f"❌ Evolution failed with error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()