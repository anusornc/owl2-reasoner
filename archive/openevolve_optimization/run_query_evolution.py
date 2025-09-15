#!/usr/bin/env python3
"""
Run Query Processing Evolution with OpenEvolve

This script runs the OpenEvolve optimization for query processing algorithms
to achieve competitive performance against established query engines.
"""

import sys
import os
import json
import time
from pathlib import Path

# Add OpenEvolve to path
sys.path.insert(0, '/Users/anusornchaikaev/Work/Phd/KnowledgeGraph/openevolve')

from openevolve import OpenEvolve
from query_evaluator import evaluate

def main():
    print("=== OpenEvolve Query Processing Optimization ===")
    print("Starting Phase 2: Query Processing Algorithm Evolution")
    print()

    # Load the target query processing program
    target_program_path = Path(__file__).parent / "query_optimization_target.rs"
    evaluation_file = Path(__file__).parent / "query_evaluator.py"

    if not target_program_path.exists():
        print(f"Error: Target program not found at {target_program_path}")
        sys.exit(1)

    if not evaluation_file.exists():
        print(f"Error: Evaluation file not found at {evaluation_file}")
        sys.exit(1)

    print(f"Loaded initial query processing program from {target_program_path}")
    print(f"Using evaluator: {evaluation_file}")
    print("Target: Sub-2ms query execution with high accuracy")
    print()

    # Configure OpenEvolve for query processing optimization
    print("OpenEvolve Configuration:")
    print(f"  Algorithm: map_elites")
    print(f"  Population: 50 individuals")
    print(f"  Generations: 200")
    print(f"  Features: correctness, speed, memory_efficiency, scalability")
    print(f"  Timeout: 300s per evaluation")
    print(f"  Target: >90% correctness, <2ms execution time")
    print()

    # Create output directory
    output_dir = Path(__file__).parent / "query_evolution_output"
    output_dir.mkdir(exist_ok=True)

    # Initialize OpenEvolve
    print("Initializing OpenEvolve...")
    try:
        from openevolve.config import Config

        # Create config with proper field assignments
        evolution_config = Config()
        evolution_config.max_iterations = 10  # Small number for testing
        evolution_config.cascade_evaluation = False
        evolution_config.log_level = "INFO"

        evolution = OpenEvolve(
            initial_program_path=str(target_program_path),
            evaluation_file=str(evaluation_file),
            config=evolution_config,
            output_dir=str(output_dir)
        )
    except Exception as e:
        print(f"Error initializing OpenEvolve: {e}")
        print("Make sure OpenEvolve is properly installed and configured")
        sys.exit(1)

    # Run the evolution
    print("Starting evolution...")
    print("This will optimize query processing algorithms for:")
    print("  - SELECT query performance")
    print("  - ASK query responsiveness")
    print("  - CONSTRUCT query efficiency")
    print("  - Memory usage optimization")
    print("  - Scalability with dataset size")
    print()

    start_time = time.time()

    try:
        import asyncio
        results = asyncio.run(evolution.run())
        evolution_time = time.time() - start_time

        print(f"\n=== Evolution Complete ===")
        print(f"Total evolution time: {evolution_time:.1f} seconds")
        print(f"Generations completed: {len(results.get('generation_history', []))}")
        print(f"Best fitness achieved: {results.get('best_fitness', 0.0):.4f}")

        # Display detailed results
        if 'best_individual' in results:
            best = results['best_individual']
            print(f"\n=== Best Query Processing Solution ===")
            print(f"Fitness: {best.get('fitness', 0.0):.4f}")
            print(f"Correctness: {best.get('correctness', 0.0):.3f}")
            print(f"Performance: {best.get('performance', 0.0):.3f}ms")
            print(f"Memory: {best.get('memory_efficiency', 0.0):.1f}KB")
            print(f"Scalability: {best.get('scalability', 0.0):.3f}")

            # Save the best program
            best_program = best.get('program', '')
            if best_program:
                output_file = Path(__file__).parent / "optimized_query_processor.rs"
                with open(output_file, 'w') as f:
                    f.write(best_program)
                print(f"Best program saved to: {output_file}")

                # Test the optimized program
                print(f"\n=== Testing Optimized Query Processor ===")
                test_result = evaluate(best_program)
                print(f"Validation - Fitness: {test_result['fitness']:.4f}")
                print(f"Validation - Performance: {test_result['performance']:.3f}ms")
                print(f"Validation - Correctness: {test_result['correctness']:.3f}")

        # Save evolution results
        results_file = Path(__file__).parent / "query_evolution_results.json"
        with open(results_file, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"\nEvolution results saved to: {results_file}")

        # Compare with baseline
        baseline_fitness = 150.0  # Expected baseline fitness
        achieved_fitness = results.get('best_fitness', 0.0)
        improvement = ((achieved_fitness - baseline_fitness) / baseline_fitness) * 100

        print(f"\n=== Performance Summary ===")
        print(f"Baseline fitness: {baseline_fitness:.1f}")
        print(f"Achieved fitness: {achieved_fitness:.1f}")
        print(f"Improvement: {improvement:+.1f}%")

        if achieved_fitness > baseline_fitness:
            print("✅ Query processing optimization successful!")
        else:
            print("⚠️  Further optimization may be needed")

    except Exception as e:
        print(f"Error during evolution: {e}")
        print("Evolution failed to complete successfully")
        sys.exit(1)

if __name__ == "__main__":
    main()