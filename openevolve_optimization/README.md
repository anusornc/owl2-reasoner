# OWL2 Reasoner Evolution with OpenEvolve

This example demonstrates how to use OpenEvolve to evolve high-performance OWL2 reasoning algorithms in Rust. The evolutionary process starts with a basic O(n²) subclass reasoning implementation and discovers optimized algorithms that can handle large ontologies efficiently.

## What This Evolves

The evolution targets key OWL2 reasoning operations:

1. **Subclass Reasoning**: Optimize hierarchical reasoning from O(n²) to O(n log n) or better
2. **Memory Efficiency**: Reduce memory overhead through better data structures
3. **Scalability**: Handle large ontologies (10K+ entities) efficiently
4. **Adaptive Algorithms**: Develop hybrid reasoning strategies

## Files

- `initial_program.rs`: Starting implementation with basic O(n²) reasoning
- `evaluator.py`: Comprehensive evaluator that tests reasoning correctness and performance
- `config.yaml`: Configuration optimized for reasoning algorithm evolution
- `README.md`: This documentation

## Prerequisites

### System Dependencies
1. **Rust Toolchain**: Install from [rustup.rs](https://rustup.rs/)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Cargo**: Comes with Rust installation

### Python Dependencies
```bash
pip install -r requirements.txt
```

### OpenEvolve
Clone OpenEvolve and ensure it's in your Python path or accessible.

## Usage

### Basic Evolution
```bash
cd openevolve_optimization
python ../../openevolve/openevolve-run.py initial_program.rs evaluator.py --config config.yaml --iterations 100
```

### Extended Evolution (Recommended)
```bash
# Run for more iterations to discover better algorithms
python ../../openevolve/openevolve-run.py initial_program.rs evaluator.py --config config.yaml --iterations 200

# Monitor progress with checkpoints
python ../../openevolve/openevolve-run.py initial_program.rs evaluator.py --config config.yaml --iterations 500 --checkpoint-interval 25
```

## Test Scenarios

The evaluator tests evolved algorithms on:

1. **Basic Hierarchy**: Simple subclass relationships
2. **Deep Hierarchy**: Multi-level inheritance chains
3. **Large Scale**: 100+ entity hierarchies (scalability test)
4. **Biomedical Ontologies**: Realistic disease and gene hierarchies

## Performance Metrics

- **Correctness**: Logical reasoning accuracy (must be >95%)
- **Performance**: Operation speed (nanoseconds per query)
- **Scalability**: Performance consistency with increasing ontology size
- **Memory Efficiency**: Memory usage per entity

## Expected Evolution Timeline

- **Iterations 1-50**: Basic optimizations (indexing, caching)
- **Iterations 50-100**: Algorithm improvements (topological sort, bit vectors)
- **Iterations 100-200**: Advanced strategies (parallel processing, hybrid algorithms)
- **Iterations 200+**: Specialized optimizations (domain-specific patterns)

## Integration with Main Project

After evolution, successful algorithms can be integrated back into the main OWL2 reasoner:

1. Extract evolved reasoning logic from `EVOLVE-BLOCK-START` to `EVOLVE-BLOCK-END`
2. Replace current `compute_subclass_of` in `src/reasoning/simple.rs`
3. Update data structures and memory management
4. Run comprehensive tests to ensure correctness

## Examples of Expected Optimizations

The evolution might discover:

1. **Topological Sort**: Replace recursive calls with linear-time topological sorting
2. **Bit Vector Indexing**: Use bit vectors for efficient set operations
3. **Memoization**: Intelligent caching of intermediate results
4. **Parallel Processing**: Utilize multiple cores for independent operations
5. **Hybrid Algorithms**: Combine tableaux and rule-based reasoning optimally

## Monitoring Evolution

Monitor the evolution process by checking:

- **Score progression**: Should show steady improvement
- **Correctness**: Must remain above 95%
- **Performance**: Should decrease (faster operations)
- **Scalability**: Should improve with larger test cases

## Troubleshooting

### Common Issues
1. **Compilation errors**: Check Rust syntax and dependencies
2. **Runtime errors**: Ensure all test cases are handled correctly
3. **Performance regression**: Increase iterations or adjust fitness function
4. **Memory issues**: Monitor memory usage during large-scale tests

### Getting Help
Check the OpenEvolve documentation for detailed configuration options and troubleshooting guides.

## Next Steps

After successful evolution, consider:

1. **Profile Integration**: Integrate evolved algorithms with profiling tools
2. **Extended Evolution**: Evolve other reasoning operations (consistency, satisfiability)
3. **Domain Specialization**: Create domain-specific evolutionary targets
4. **Architecture Evolution**: Evolve entire reasoner architecture patterns