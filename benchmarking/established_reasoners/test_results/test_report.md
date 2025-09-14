# OWL2 Reasoner Comprehensive Test Report
Generated: 2025-09-14 18:07:56

## Test Results Summary

| Reasoner | Help | Classification | Consistency | Overall Status |
|----------|------|----------------|-------------|----------------|
| Rust OWL2 | ✅ | ✅ | ✅ | ✅ Working |
| ELK | ✅ | ✅ | ✅ | ✅ Working |
| HermiT | ✅ | ✅ | ✅ | ✅ Working |
| JFact | ✅ | ✅ | ✅ | ✅ Working |

## Detailed Test Results

### Rust OWL2

- ✅ **help**: 515.44ms
  - Error: warning: unused variable: `baseline_peak`
   --> src/validation/memory_profiler.rs:116:13
    |
116 ...
- ✅ **classification_benchmark_small**: 243.32ms
  - Error: warning: unused variable: `baseline_peak`
   --> src/validation/memory_profiler.rs:116:13
    |
116 ...
- ✅ **consistency_benchmark_small**: 232.11ms
  - Error: warning: unused variable: `baseline_peak`
   --> src/validation/memory_profiler.rs:116:13
    |
116 ...
- ✅ **classification_benchmark_medium**: 244.60ms
  - Error: warning: unused variable: `baseline_peak`
   --> src/validation/memory_profiler.rs:116:13
    |
116 ...
- ✅ **consistency_benchmark_medium**: 236.83ms
  - Error: warning: unused variable: `baseline_peak`
   --> src/validation/memory_profiler.rs:116:13
    |
116 ...
- ✅ **classification_simple_functional**: 223.18ms
  - Error: warning: unused variable: `baseline_peak`
   --> src/validation/memory_profiler.rs:116:13
    |
116 ...
- ✅ **consistency_simple_functional**: 228.93ms
  - Error: warning: unused variable: `baseline_peak`
   --> src/validation/memory_profiler.rs:116:13
    |
116 ...

### ELK

- ✅ **help**: 287.18ms
- ❌ **classification_benchmark_small**: 263.96ms
  - Error: Exception in thread "main" org.semanticweb.elk.loading.ElkLoadingException: Cannot load the ontology...
- ❌ **consistency_benchmark_small**: 237.27ms
  - Error: Exception in thread "main" org.semanticweb.elk.loading.ElkLoadingException: Cannot load the ontology...
- ❌ **classification_benchmark_medium**: 240.96ms
  - Error: Exception in thread "main" org.semanticweb.elk.loading.ElkLoadingException: Cannot load the ontology...
- ❌ **consistency_benchmark_medium**: 240.23ms
  - Error: Exception in thread "main" org.semanticweb.elk.loading.ElkLoadingException: Cannot load the ontology...
- ✅ **classification_simple_functional**: 260.17ms
  - Error: 205   [main] WARN  org.semanticweb.elk.reasoner.completeness.Incompleteness  - Class inclusions may ...
  - Output: test_results/elk_simple_functional_classification.txt (388 bytes)
- ✅ **consistency_simple_functional**: 250.26ms
  - Error: 197   [main] WARN  org.semanticweb.elk.reasoner.completeness.Incompleteness  - Ontology satisfiabili...

### HermiT

- ✅ **help**: 79.78ms
- ✅ **classification_benchmark_small**: 298.32ms
  - Output: test_results/hermit_benchmark_small_classification.txt (369 bytes)
- ✅ **consistency_benchmark_small**: 265.61ms
- ✅ **classification_benchmark_medium**: 389.13ms
  - Output: test_results/hermit_benchmark_medium_classification.txt (2764 bytes)
- ✅ **consistency_benchmark_medium**: 286.47ms
- ✅ **classification_simple_functional**: 193.48ms
  - Output: test_results/hermit_simple_functional_classification.txt (165 bytes)
- ✅ **consistency_simple_functional**: 204.87ms

### JFact

- ✅ **help**: 3.38ms
- ✅ **classification_benchmark_small**: 2.37ms
- ✅ **consistency_benchmark_small**: 2.30ms
- ✅ **classification_benchmark_medium**: 2.31ms
- ✅ **consistency_benchmark_medium**: 2.27ms
- ✅ **classification_simple_functional**: 2.20ms
- ✅ **consistency_simple_functional**: 2.40ms
