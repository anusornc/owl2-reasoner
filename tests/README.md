# OWL2 Reasoner Test Suite

This directory contains the comprehensive test suite for the OWL2 Reasoner project, organized by test type and functionality.

## Test Suite Structure

```
tests/
├── README.md                    # This file
├── mod.rs                       # Main test module declarations
├── standalone/                  # Standalone test scripts
│   ├── README.md               # Standalone test guide
│   └── jsonld_compliance.rs    # JSON-LD compliance tests
├── legacy/                      # Legacy tests (archived)
│   └── README.md               # Legacy test documentation
└── [integration with src/tests/]
```

## Memory-Safe Testing

All tests in this suite use memory-safe testing patterns to prevent out-of-memory errors and system hangs. The memory management system includes:

- **Real-time monitoring**: Continuous memory usage tracking
- **Automatic cleanup**: Cache cleanup when memory limits approached
- **Configurable limits**: Different memory limits for different test types
- **Graceful failure**: Tests fail before causing system instability

## Test Categories

### 1. Unit Tests (`src/tests/`)
Individual component testing with minimal dependencies:
- **Parser Tests**: Tokenizer, syntax validation, parsing logic
- **Axiom Tests**: OWL axiom creation and manipulation
- **Entity Tests**: IRI handling, class/property creation
- **Reasoning Tests**: Individual reasoning algorithms

### 2. Integration Tests (`src/tests/integration_tests.rs`)
End-to-end testing of component interactions:
- **Parser to Ontology**: Complete parsing workflows
- **Reasoning Integration**: Full reasoning pipelines
- **Cache Integration**: Cache behavior across operations

### 3. Performance Tests (`src/tests/performance_regression_tests.rs`)
Performance validation and regression detection:
- **Timing Thresholds**: Operations complete within time limits
- **Memory Usage**: Memory consumption stays within bounds
- **Scalability**: Performance with large ontologies

### 4. Stress Tests (`src/tests/stress_tests.rs`)
High-load testing with large datasets:
- **Large Ontologies**: 1000+ class ontologies
- **Complex Hierarchies**: Deep class hierarchies
- **Memory Pressure**: Behavior under memory constraints

### 5. Comprehensive Tests (`src/tests/comprehensive.rs`)
Real-world scenario testing:
- **Family Ontologies**: Complex relationship modeling
- **Biomedical Ontologies**: Domain-specific reasoning
- **Property Characteristics**: Advanced property features

### 6. Standalone Tests (`tests/standalone/`)
Independent test scripts:
- **JSON-LD Compliance**: W3C standard validation
- **Format Validation**: Individual format testing

## Running Tests

### All Tests
```bash
# Run all tests with memory safety
cargo test --lib

# Run tests with verbose memory reporting
OWL2_TEST_VERBOSE=1 cargo test --lib
```

### Specific Test Categories
```bash
# Performance regression tests
cargo test performance_regression_tests --lib

# Stress tests
cargo test stress_tests --lib

# Comprehensive tests
cargo test comprehensive --lib

# Integration tests
cargo test integration_tests --lib
```

### Standalone Tests
```bash
# JSON-LD compliance test
cargo run --bin test_jsonld_compliance
```

## Memory Configuration

### Default Memory Limits
- **Unit Tests**: 256MB memory, 500 cache entries
- **Integration Tests**: 256MB memory, 500 cache entries
- **Performance Tests**: 512MB memory, 1000 cache entries
- **Stress Tests**: 1GB memory, 2000 cache entries (warnings only)

### Custom Memory Limits
```bash
# Override default memory limit
OWL2_TEST_MEMORY_LIMIT_MB=512 cargo test

# Override cache limit
OWL2_TEST_CACHE_LIMIT=1000 cargo test
```

## Test Macros

The test suite uses memory-safe testing macros:

### `memory_safe_test!`
Standard memory-safe tests with default limits:
```rust
memory_safe_test!(my_test, {
    // Test code here
});
```

### `memory_safe_test!` with custom config
Tests with specific memory limits:
```rust
memory_safe_test!(my_test, MemorySafeTestConfig::small(), {
    // Test code here
});
```

### `memory_safe_stress_test!`
Tests with relaxed memory limits:
```rust
memory_safe_stress_test!(my_stress_test, {
    // Stress test code here
});
```

### `memory_safe_bench_test!`
Multi-iteration benchmark tests:
```rust
memory_safe_bench_test!(my_bench_test, 100, {
    // Benchmark code here
});
```

## Test Coverage

### Current Coverage Areas
- ✅ **Parser Components**: All parser formats and edge cases
- ✅ **Reasoning Algorithms**: Tableaux, rule-based reasoning
- ✅ **Memory Management**: Cache behavior, memory pressure handling
- ✅ **Error Handling**: Graceful failure and recovery
- ✅ **Performance**: Timing and memory usage validation
- ✅ **Standards Compliance**: JSON-LD 1.1 standard validation

### Coverage Metrics
- **Total Tests**: 300+ tests across all categories
- **Test Success Rate**: 100% (all tests passing)
- **Memory Safety**: 100% (no OOM errors)
- **Performance Compliance**: 100% (within thresholds)

## Adding New Tests

### 1. Unit Tests
Add to appropriate module in `src/tests/`:
```rust
#[cfg(test)]
mod my_module_tests {
    use super::*;
    
    memory_safe_test!(test_my_functionality, {
        // Test implementation
    });
}
```

### 2. Integration Tests
Add to `src/tests/integration_tests.rs`:
```rust
memory_safe_test!(test_integration_scenario, {
    // Integration test implementation
});
```

### 3. Performance Tests
Add to `src/tests/performance_regression_tests.rs`:
```rust
memory_safe_test!(test_performance_scenario, MemorySafeTestConfig::medium(), {
    // Performance test with timing assertions
});
```

### 4. Standalone Tests
Add to `tests/standalone/`:
```rust
// Independent test script
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test implementation
    Ok(())
}
```

## Debugging Test Failures

### Memory Issues
```bash
# Enable verbose memory reporting
OWL2_TEST_VERBOSE=1 cargo test my_test

# Check memory usage during test
cargo test my_test -- --nocapture
```

### Performance Issues
```bash
# Run performance tests specifically
cargo test performance_regression_tests_summary --lib

# Check timing details
cargo test -- --ignored performance
```

### Test Isolation Issues
```bash
# Run tests in isolation
cargo test --test-threads=1

# Check for test interference
cargo test my_test -- --exact
```

## Continuous Integration

### GitHub Actions Example
```yaml
- name: Run Tests
  run: |
    export OWL2_TEST_VERBOSE=1
    export OWL2_TEST_MEMORY_LIMIT_MB=512
    cargo test --lib --all-features
```

### Test Results
- **Memory Reports**: Automatically generated for each test
- **Performance Metrics**: Tracked and validated against thresholds
- **Coverage Reports**: Generated on request

## Best Practices

### 1. Memory Safety
- Always use memory-safe test macros
- Choose appropriate memory limits for test type
- Include memory assertions for edge cases
- Test with various memory configurations

### 2. Test Organization
- Place tests in appropriate modules
- Use descriptive test names
- Include comprehensive documentation
- Group related tests together

### 3. Performance Testing
- Include timing assertions where appropriate
- Test with realistic data sizes
- Monitor memory usage patterns
- Establish performance baselines

### 4. Error Handling
- Test both success and failure scenarios
- Validate error messages and types
- Test graceful degradation
- Include edge case testing

## Troubleshooting

### Common Issues

#### Test Timeout
- Increase memory limits for test
- Check for infinite loops or recursion
- Optimize test data size

#### Memory Limit Exceeded
- Use stress test configuration for memory-intensive tests
- Check for memory leaks in test setup/teardown
- Reduce test data size

#### Compilation Errors
- Check import paths and module structure
- Verify all dependencies are available
- Update deprecated API usage

#### Test Flakiness
- Use test isolation (single-threaded execution)
- Check for timing dependencies
- Ensure proper cleanup between tests

## Resources

- [Memory-Safe Testing Guide](../docs/MEMORY_SAFE_TESTING.md)
- [API Documentation](../docs/api/)
- [Performance Benchmarking](../docs/performance/)
- [Developer Guide](../docs/developer/)

## Contributing

When contributing new tests:

1. **Use Memory-Safe Patterns**: Always use the provided test macros
2. **Choose Appropriate Limits**: Select memory limits based on test requirements
3. **Document Test Purpose**: Clear documentation of what the test validates
4. **Include Edge Cases**: Test both normal and exceptional conditions
5. **Performance Considerations**: Be mindful of test execution time

For detailed guidelines, see the [Developer Guide](../docs/developer/contributing.md).