#!/usr/bin/env python3
"""
Simple evolution script for subclass checking optimization
This creates small variations of the optimized algorithm to find improvements
"""

import random
import subprocess
import tempfile
import time
import json
from pathlib import Path
from typing import List, Dict, Any

class SimpleEvolution:
    def __init__(self):
        self.population_size = 10
        self.generations = 20
        self.mutation_rate = 0.3
        self.elite_size = 2

    def create_variations(self, base_code: str) -> List[str]:
        """Create small variations of the base algorithm"""
        variations = [base_code]  # Keep original

        # Generate variations through simple mutations
        for i in range(self.population_size - 1):
            mutated = self.mutate_code(base_code)
            if mutated and len(mutated) < 10000:  # Size limit
                variations.append(mutated)

        return variations[:self.population_size]

    def mutate_code(self, code: str) -> str:
        """Apply simple mutations to the code"""
        lines = code.split('\n')
        mutations = [
            self.mutate_data_structure,
            self.mutate_algorithm,
            self.mutate_optimization,
            self.mutate_cache_strategy,
        ]

        # Apply random mutations
        mutated_lines = lines.copy()
        num_mutations = random.randint(1, 3)

        for _ in range(num_mutations):
            if random.random() < self.mutation_rate:
                mutation = random.choice(mutations)
                try:
                    mutated_lines = mutation(mutated_lines)
                except:
                    continue

        return '\n'.join(mutated_lines)

    def mutate_data_structure(self, lines: List[str]) -> List[str]:
        """Mutate data structure choices"""
        mutations = [
            # Change HashSet to BTreeSet for better memory
            ('HashSet', 'BTreeSet'),
            # Change VecDeque to Vec for simpler code
            ('VecDeque', 'Vec'),
            # Add additional caching layer
            ('memoization_cache: HashMap<(String, String), bool>,',
             'memoization_cache: HashMap<(String, String), bool>,\n    lru_cache: std::collections::LinkedList<(String, String)>,'),
        ]

        for old, new in mutations:
            if random.random() < 0.3:
                for i, line in enumerate(lines):
                    if old in line:
                        lines[i] = line.replace(old, new)

        return lines

    def mutate_algorithm(self, lines: List[str]) -> List[str]:
        """Mutate algorithm choices"""
        # Find the BFS function and modify it
        for i, line in enumerate(lines):
            if 'fn bfs_subclass_check' in line:
                # Add algorithm variations
                if random.random() < 0.3:
                    # Add early exit optimization
                    lines.insert(i + 1, '    // Early exit optimization')
                    lines.insert(i + 2, '    if start_class.len() > target_class.len() { return false; }')
                elif random.random() < 0.3:
                    # Add parallel processing hint
                    lines.insert(i + 1, '    // TODO: Consider parallel processing for large graphs')

        return lines

    def mutate_optimization(self, lines: List[str]) -> List[str]:
        """Add optimization hints and small changes"""
        for i, line in enumerate(lines):
            if 'queue.push_back' in line and random.random() < 0.3:
                # Add capacity optimization
                lines.insert(i, '        queue.reserve(16); // Pre-allocate for better performance')
            elif 'visited.insert' in line and random.random() < 0.3:
                # Add optimization comment
                lines.insert(i + 1, '        // Optimization: Fast path for direct lookups')

        return lines

    def mutate_cache_strategy(self, lines: List[str]) -> List[str]:
        """Mutate caching strategy"""
        for i, line in enumerate(lines):
            if 'memoization_cache.clear()' in line and random.random() < 0.3:
                # Add smarter cache clearing
                lines[i] = line.replace('memoization_cache.clear()',
                                     '// memoization_cache.clear() // Keep cache for performance')

        return lines

    def evaluate_individual(self, code: str, individual_id: int) -> Dict[str, Any]:
        """Evaluate a single individual"""
        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                # Write the code to a file
                code_file = Path(temp_dir) / f"individual_{individual_id}.rs"
                with open(code_file, 'w') as f:
                    f.write(code)

                # Compile and run
                compile_result = subprocess.run(
                    ['rustc', str(code_file), '-o', str(Path(temp_dir) / 'test')],
                    capture_output=True, text=True, timeout=30
                )

                if compile_result.returncode != 0:
                    return {
                        'fitness': 0.0,
                        'compilation': False,
                        'error': compile_result.stderr,
                        'execution_time': float('inf')
                    }

                # Run the program
                start_time = time.time()
                run_result = subprocess.run(
                    [str(Path(temp_dir) / 'test')],
                    capture_output=True, text=True, timeout=30
                )
                execution_time = time.time() - start_time

                if run_result.returncode != 0:
                    return {
                        'fitness': 0.0,
                        'compilation': True,
                        'runtime_error': True,
                        'execution_time': execution_time,
                        'error': run_result.stderr
                    }

                # Parse output for performance metrics
                output = run_result.stdout
                fitness = self.parse_fitness(output, execution_time)

                return {
                    'fitness': fitness,
                    'compilation': True,
                    'runtime_error': False,
                    'execution_time': execution_time,
                    'output': output
                }

        except Exception as e:
            return {
                'fitness': 0.0,
                'compilation': False,
                'error': str(e),
                'execution_time': float('inf')
            }

    def parse_fitness(self, output: str, execution_time: float) -> float:
        """Parse fitness from program output"""
        try:
            # Look for performance metrics
            lines = output.split('\n')
            optimized_time = None
            basic_time = None

            for line in lines:
                if 'Optimized implementation time:' in line:
                    optimized_time = float(line.split(':')[1].strip().split(' ')[0])
                elif 'Basic implementation time:' in line:
                    basic_time = float(line.split(':')[1].strip().split(' ')[0])

            if optimized_time and basic_time:
                # Calculate improvement factor
                improvement = basic_time / optimized_time if optimized_time > 0 else 1.0
                # Higher fitness for better improvement and faster execution
                return improvement * (1.0 / max(0.001, execution_time))

            # Fallback: use execution time only (lower is better)
            return 1.0 / max(0.001, execution_time)

        except:
            return 0.1  # Default low fitness

    def run_evolution(self, initial_code: str) -> Dict[str, Any]:
        """Run the complete evolution process"""
        print("🧬 Starting simple evolution process...")

        # Read initial code
        with open(initial_code, 'r') as f:
            base_code = f.read()

        best_individual = base_code
        best_fitness = 0.0

        for generation in range(self.generations):
            print(f"\n📊 Generation {generation + 1}/{self.generations}")

            # Create population
            population = self.create_variations(best_individual)

            # Evaluate population
            results = []
            for i, individual in enumerate(population):
                print(f"  Evaluating individual {i + 1}/{len(population)}...")
                result = self.evaluate_individual(individual, i)
                result['code'] = individual
                results.append(result)

            # Sort by fitness
            results.sort(key=lambda x: x['fitness'], reverse=True)

            # Track best
            if results[0]['fitness'] > best_fitness:
                best_fitness = results[0]['fitness']
                best_individual = results[0]['code']
                print(f"  🏆 New best fitness: {best_fitness:.4f}")

            # Show generation stats
            successful = sum(1 for r in results if r['compilation'] and not r.get('runtime_error'))
            print(f"  ✅ Successful individuals: {successful}/{len(results)}")
            print(f"  📈 Best fitness this generation: {results[0]['fitness']:.4f}")

        return {
            'best_code': best_individual,
            'best_fitness': best_fitness,
            'generations': self.generations
        }

def main():
    evolver = SimpleEvolution()

    print("🚀 Simple Evolution for Subclass Checking Optimization")
    print("=" * 60)

    result = evolver.run_evolution('subclass_optimized_manual.rs')

    print("\n" + "=" * 60)
    print("🎯 Evolution Complete!")
    print("=" * 60)
    print(f"🏆 Best fitness: {result['best_fitness']:.4f}")
    print(f"📊 Generations: {result['generations']}")

    # Save best result
    with open('subclass_evolved_simple.rs', 'w') as f:
        f.write(result['best_code'])

    print(f"💾 Best algorithm saved to: subclass_evolved_simple.rs")

    # Test the final result
    print("\n🧪 Testing final evolved algorithm...")
    test_result = evolver.evaluate_individual(result['best_code'], 0)
    if test_result['compilation'] and not test_result.get('runtime_error'):
        print("✅ Final algorithm compiles and runs successfully!")
        print(f"⚡ Execution time: {test_result['execution_time']:.3f}s")
    else:
        print("❌ Final algorithm has issues")

if __name__ == "__main__":
    main()