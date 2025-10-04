# Standalone Tests

This directory contains standalone test scripts that can be run independently of the main test suite.

## Running Standalone Tests

### JSON-LD Compliance Test

```bash
# Run the JSON-LD compliance test
cargo run --bin test_jsonld_compliance

# Or compile and run directly
rustc test_jsonld_compliance.rs -L target/debug/deps
./test_jsonld_compliance
```

## Test Descriptions

### JSON-LD Compliance Test (`test_jsonld_compliance.rs`)

Tests compliance with W3C JSON-LD 1.1 standard examples including:
- Basic @context and @id usage
- @vocab functionality
- Nested @context handling
- @graph with multiple nodes
- @set and @list collections
- Blank node processing
- Language tag support
- Datatype coercion
- @reverse (inverse properties)
- @index functionality

## Memory Safety

All standalone tests are designed to be memory-safe and will not cause OOM errors. They use the same memory management system as the main test suite.

## Adding New Standalone Tests

When adding new standalone tests:

1. Place the test file in this directory
2. Ensure it's memory-safe using the test helpers
3. Update this README with a description
4. Add compilation instructions if needed

## Legacy Tests

Legacy tests that may have compatibility issues are located in `../legacy/`. These are kept for reference but may not work with the current API.