#!/usr/bin/env python3
"""
Proper OpenEvolve configuration for subclass checking optimization
"""

from openevolve.config import Config, LLMModelConfig

def create_config():
    """Create proper OpenEvolve configuration"""

    # Google AI configuration
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
1. Replace manual DFS stack with efficient BFS queue
2. Use proper data structures (VecDeque, HashSet)
3. Eliminate redundant computations
4. Add cycle detection and memoization
5. Improve memory efficiency
6. Handle edge cases gracefully

The algorithm must:
- Maintain 100% correctness
- Handle cycles without infinite loops
- Support equivalent classes relationships
- Be memory efficient for large ontologies
- Have clear, well-documented code

Focus on algorithmic improvements, not just micro-optimizations.
Return only the complete, optimized Rust code."""
    )

    # Create config
    config = Config()
    config.llm.models = [google_llm]
    config.evolution.max_iterations = 30
    config.evolution.population_size = 8
    config.evolution.mutation_rate = 0.4
    config.evolution.crossover_rate = 0.3
    config.evolution.elitism_rate = 0.2

    # Database settings
    config.database.feature_dimensions = 5
    config.database.feature_bins = [10, 8, 8, 10, 2]
    config.database.num_islands = 4
    config.database.migration_interval = 10

    # Evaluation settings
    config.evaluation.timeout_seconds = 120
    config.evaluation.parallel_evaluations = True
    config.evaluation.max_concurrent_evaluations = 4

    # Output settings
    config.output.save_best_program = True
    config.output.save_diverse_programs = True
    config.output.log_fitness_progress = True
    config.output.log_feature_progress = True
    config.output.visualization_data = True

    return config

if __name__ == "__main__":
    config = create_config()
    import yaml
    print(yaml.dump(config.__dict__, default_flow_style=False))