# Legacy Tests

This directory contains legacy test files that may have compatibility issues with the current codebase structure.

## ⚠️ Compatibility Issues

The tests in this directory were written for an older version of the OWL2 Reasoner API and may not compile or run correctly with the current codebase.

## Legacy Test Files

### `test_tokenization.rs`
**Status:** ❌ Incompatible
**Issues:** 
- Missing proper module imports
- References outdated tokenizer API
- Incompatible with current parser structure

**Recommended Action:** 
- Archive for reference
- Functionality is covered by current test suite in `src/tests/`

### `test_tokenizer.rs`
**Status:** ❌ Incompatible  
**Issues:**
- Outdated import paths
- References deprecated API methods
- Missing required dependencies

**Recommended Action:**
- Archive for reference
- Consider reimplementing if needed with current API

## Migration Options

### Option 1: Fix Legacy Tests
To make these tests compatible with the current API:

1. Update import paths to match current module structure
2. Replace deprecated API calls with current equivalents
3. Update test structure to use current testing patterns
4. Add memory-safe testing protections

### Option 2: Replace with New Tests
Create new tests that cover the same functionality:

1. Identify the test scenarios covered by legacy tests
2. Implement equivalent tests using current API
3. Place in appropriate test modules (`src/tests/`)
4. Use memory-safe testing patterns

### Option 3: Archive
If the functionality is adequately covered by existing tests:

1. Move files to this legacy directory
2. Add documentation about what they tested
3. Remove from active test suite

## Current Test Coverage

The legacy test functionality is largely covered by:

- **Tokenizer Tests:** `src/tests/debug_tokenizer_test.rs`
- **Parser Tests:** `src/tests/negative_tests.rs`
- **Integration Tests:** `src/tests/integration_tests.rs`
- **Performance Tests:** `src/tests/performance_regression_tests.rs`

## Recommendations

Based on the analysis, it's recommended to:

1. **Archive** the current legacy tests in this directory
2. **Document** what functionality they covered
3. **Verify** that current test suite adequately covers the same scenarios
4. **Remove** the legacy tests from the root directory to avoid confusion

## Future Considerations

If specific functionality from the legacy tests is needed:

1. Check if it's covered by current tests
2. If not, implement new tests using current API
3. Use memory-safe testing patterns
4. Place in appropriate test module

## History

These tests were originally written for early versions of the OWL2 Reasoner and have been kept for historical reference and to ensure no test coverage was lost during API evolution.