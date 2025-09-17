# External OWL2 Reasoner Benchmark Report
==================================================
**Date:** 2025-09-16T14:12:30.411911
**Hardware:** arm64 (Darwin)

## Performance Comparison

| Reasoner | Success Rate | Avg Time (ms) | Status |
|-----------|--------------|---------------|---------|
| OWL2-Reasoner (Rust) | 100.0% | 326.64 | ✅ Working |
| ELK (Java) | 100.0% | 276.07 | ✅ Working |
| HermiT (Java) | 0.0% | 0.00 | ❌ Failed |
| JFact (Java) | 0.0% | 0.00 | ❌ Failed |

## Detailed Results

### OWL2-Reasoner (Rust)

- **Small Family**: ✅ Consistent (398.63ms)
- **Biomedical**: ✅ Consistent (287.76ms)
- **Complex Expressions**: ✅ Consistent (298.20ms)
- **Classification**: ✅ Consistent (321.96ms)

### ELK (Java)

- **Small Family**: ✅ Inconsistent (318.49ms)
- **Biomedical**: ✅ Inconsistent (247.33ms)
- **Complex Expressions**: ✅ Inconsistent (276.44ms)
- **Classification**: ✅ Inconsistent (262.03ms)

### HermiT (Java)

- **Small Family**: ❌ Unknown (0.00ms)
- **Biomedical**: ❌ Unknown (0.00ms)
- **Complex Expressions**: ❌ Unknown (0.00ms)
- **Classification**: ❌ Unknown (0.00ms)

### JFact (Java)

- **Small Family**: ❌ Unknown (0.00ms)
- **Biomedical**: ❌ Unknown (0.00ms)
- **Complex Expressions**: ❌ Unknown (0.00ms)
- **Classification**: ❌ Unknown (0.00ms)

## Performance Analysis

🏆 **Best Performance:** ELK (Java) (276.07ms average)

🦀 **OWL2-Reasoner Performance:**
- Success Rate: 100.0%
- Average Time: 326.64ms
- Speedup vs ELK (Java): 0.8x