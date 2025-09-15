#!/usr/bin/env python3
"""
Run Rule System Evolution with OpenEvolve

This script runs the OpenEvolve optimization for rule system algorithms
to achieve competitive performance against established rule engines.
"""

import sys
import os
import json
import time
from pathlib import Path

# Add OpenEvolve to path
sys.path.insert(0, '/Users/anusornchaikaew/Work/Phd/KnowledgeGraph/openevolve')

from openevolve import OpenEvolve
from rule_evaluator import evaluate

def main():
    print("=== OpenEvolve Rule System Optimization ===")
    print("Starting Phase 3: Rule System Algorithm Evolution")
    print()

    # Load the target rule system program
    target_program_path = Path(__file__).parent / "rule_optimization_target.rs"
    evaluation_file = Path(__file__).parent / "rule_evaluator.py"

    if not target_program_path.exists():
        print(f"Error: Target program not found at {target_program_path}")
        sys.exit(1)

    if not evaluation_file.exists():
        print(f"Error: Evaluation file not found at {evaluation_file}")
        sys.exit(1)

    print(f"Loaded initial rule system program from {target_program_path}")
    print(f"Using evaluator: {evaluation_file}")
    print("Target: Sub-1ms rule execution with high accuracy")
    print()

    # Configure OpenEvolve for rule system optimization
    print("OpenEvolve Configuration:")
    print(f"  Algorithm: map_elites")
    print(f"  Population: 50 individuals")
    print(f"  Generations: 200")
    print(f"  Features: correctness, speed, memory_efficiency, scalability")
    print(f"  Timeout: 300s per evaluation")
    print(f"  Target: >90% correctness, <1ms execution time")
    print()

    # Create output directory
    output_dir = Path(__file__).parent / "rule_evolution_output"
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
    print("This will optimize rule system algorithms for:")
    print("  - Forward chaining performance")
    print("  - Pattern matching efficiency")
    print("  - Conflict resolution strategies")
    print("  - Memory usage optimization")
    print("  - Scalability with rule complexity")
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
        best_info_path = Path(__file__).parent / "rule_evolution_output" / "best" / "best_program_info.json"
        best_program_path = Path(__file__).parent / "rule_evolution_output" / "best" / "best_program.rs"

        if best_info_path.exists():
            with open(best_info_path, 'r') as f:
                best_info = json.load(f)

            metrics = best_info.get('metrics', {})
            print(f"\n=== Best Rule System Solution ===")
            print(f"Fitness: {metrics.get('fitness', 0.0):.4f}")
            print(f"Correctness: {metrics.get('correctness', 0.0):.3f}")
            print(f"Performance: {metrics.get('performance', 0.0):.3f}ms")
            print(f"Memory: {metrics.get('memory_efficiency', 0.0):.1f}KB")
            print(f"Scalability: {metrics.get('scalability', 0.0):.3f}")

            # Copy the best program
            if best_program_path.exists():
                output_file = Path(__file__).parent / "optimized_rule_system.rs"
                import shutil
                shutil.copy2(best_program_path, output_file)
                print(f"Best program saved to: {output_file}")

                # Test the optimized program
                print(f"\n=== Testing Optimized Rule System ===")
                with open(best_program_path, 'r') as f:
                    best_program = f.read()
                test_result = evaluate(best_program)
                print(f"Validation - Fitness: {test_result['fitness']:.4f}")
                print(f"Validation - Performance: {test_result['performance']:.3f}ms")
                print(f"Validation - Correctness: {test_result['correctness']:.3f}")

        # Save evolution results
        results_file = Path(__file__).parent / "rule_evolution_results.json"
        with open(results_file, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"\nEvolution results saved to: {results_file}")

        # Compare with baseline
        baseline_fitness = 0.5  # Expected baseline fitness for rule system
        achieved_fitness = metrics.get('fitness', 0.0) if 'metrics' in locals() else 0.0
        improvement = ((achieved_fitness - baseline_fitness) / baseline_fitness) * 100

        print(f"\n=== Performance Summary ===")
        print(f"Baseline fitness: {baseline_fitness:.3f}")
        print(f"Achieved fitness: {achieved_fitness:.3f}")
        print(f"Improvement: {improvement:+.1f}%")

        if achieved_fitness > baseline_fitness:
            print("✅ Rule system optimization successful!")
        else:
            print("⚠️  Further optimization may be needed")

    except Exception as e:
        print(f"Error during evolution: {e}")
        print("Evolution failed to complete successfully")
        sys.exit(1)

if __name__ == "__main__":
    main()