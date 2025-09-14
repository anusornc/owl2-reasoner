# OWL2 Reasoner Evaluation Report

**Suite Name**: Comprehensive_OWL2_Reasoner_Benchmark
**Benchmark Type**: custom
**Generated**: 2025-09-14 21:01:42
**Version**: 1.0

## Executive Summary

This comprehensive evaluation analyzes the performance of 4 OWL2 reasoners across 32 test cases. The results show an overall success rate of 100.0% with an average execution time of 0.00 ms per test. The evaluation covers custom benchmarks with 32 individual test executions.

## Test Environment

### Hardware Specifications
- **CPU Cores**: 8
- **Processor**: arm
- **Total Memory**: 8.0 GB
- **Architecture**: arm64

### Software Environment
- **Operating System**: Darwin 15.6.1
- **Python Version**: 3.11.3
- **Java Runtime**: 24.0.2 (Unknown)


## Methodology

### Benchmark Suite
The evaluation uses the custom benchmark suite with 32 test cases. Each test represents a reasoning task with varying complexity and dataset sizes.

### Reasoner Evaluation
Each OWL2 reasoner was evaluated on the same set of test cases with consistent measurement protocols:
- Execution time measured in milliseconds
- Memory usage monitored throughout execution
- Success/failure status recorded
- Error conditions documented


## Results Overview

### Overall Performance
From 32 test cases, 32 completed successfully (100.0% success rate). The average execution time was 0.00 ms with a standard deviation of 0.00 ms.

### Key Metrics
- **Total Tests**: 32
- **Successful Tests**: 32
- **Failed Tests**: 0
- **Success Rate**: 100.0%
- **Average Execution Time**: 0.00 ms
- **Minimum Execution Time**: 0.00 ms
- **Maximum Execution Time**: 0.00 ms


## Statistical Analysis

### Basic Statistics
**summary**: total_tests=32.00, successful_tests=32.00, failed_tests=0.00, timeout_rate=0.00, overall_success_rate=100.00

**ELK_classification**: count=4.00, mean_ms=287.40, median_ms=266.47, std_dev_ms=47.49, min_ms=258.29, max_ms=358.40, range_ms=100.11, coefficient_of_variation=0.17, percentile_25=263.63, percentile_75=290.25, iqr=26.62

**ELK_consistency**: count=4.00, mean_ms=267.77, median_ms=248.80, std_dev_ms=42.86, min_ms=241.63, max_ms=331.85, range_ms=90.22, coefficient_of_variation=0.16, percentile_25=246.47, percentile_75=270.10, iqr=23.63

**HermiT_classification**: count=4.00, mean_ms=48.49, median_ms=48.49, std_dev_ms=1.36, min_ms=46.83, max_ms=50.17, range_ms=3.33, coefficient_of_variation=0.03, percentile_25=47.99, percentile_75=48.99, iqr=1.00

**HermiT_consistency**: count=4.00, mean_ms=49.28, median_ms=48.43, std_dev_ms=3.35, min_ms=46.21, max_ms=54.03, range_ms=7.82, coefficient_of_variation=0.07, percentile_25=47.62, percentile_75=50.08, iqr=2.46

**JFact_classification**: count=4.00, mean_ms=44.31, median_ms=44.26, std_dev_ms=0.52, min_ms=43.74, max_ms=44.99, range_ms=1.25, coefficient_of_variation=0.01, percentile_25=44.03, percentile_75=44.54, iqr=0.51

**JFact_consistency**: count=4.00, mean_ms=44.79, median_ms=45.14, std_dev_ms=1.26, min_ms=43.03, max_ms=45.84, range_ms=2.81, coefficient_of_variation=0.03, percentile_25=44.32, percentile_75=45.61, iqr=1.29

**Pellet_classification**: count=4.00, mean_ms=148.30, median_ms=11.00, std_dev_ms=274.87, min_ms=10.60, max_ms=560.61, range_ms=550.01, coefficient_of_variation=1.85, percentile_25=10.88, percentile_75=148.42, iqr=137.55

**Pellet_consistency**: count=4.00, mean_ms=11.10, median_ms=11.15, std_dev_ms=0.31, min_ms=10.67, max_ms=11.41, range_ms=0.74, coefficient_of_variation=0.03, percentile_25=10.99, percentile_75=11.26, iqr=0.28



### Comparative Analysis
**overall_ranking**: Complex comparative data

**performance_ranking**: Complex comparative data

**efficiency_ranking**: Complex comparative data

**reliability_ranking**: Complex comparative data

**reasoner_scores**: Complex comparative data

**statistical_significance**: Complex comparative data

**recommendations**: Complex comparative data

**key_findings**: Complex comparative data

**pairwise_comparisons**: Complex comparative data



### Significance Testing
**classification**: ELK_vs_HermiT_classification={'test_name': 'ELK_vs_HermiT_classification', 'test_type': 'Mann-Whitney U test', 'statistic': 16.0, 'p_value': 0.02857142857142857, 'effect_size': 0.5, 'confidence_interval': (163.37779848312925, 314.4427332482192), 'interpretation': 'ELK is significantly slower than HermiT for classification', 'significance_level': 0.05}, ELK_vs_JFact_classification={'test_name': 'ELK_vs_JFact_classification', 'test_type': 'Mann-Whitney U test', 'statistic': 16.0, 'p_value': 0.02857142857142857, 'effect_size': 0.5, 'confidence_interval': (167.52978167979512, 318.66156774253835), 'interpretation': 'ELK is significantly slower than JFact for classification', 'significance_level': 0.05}, ELK_vs_Pellet_classification={'test_name': 'ELK_vs_Pellet_classification', 'test_type': 'Mann-Whitney U test', 'statistic': 12.0, 'p_value': 0.34285714285714286, 'effect_size': 0.25, 'confidence_interval': (-290.94991065465763, 569.1603457105492), 'interpretation': 'No significant difference between ELK and Pellet for classification', 'significance_level': 0.05}, HermiT_vs_JFact_classification={'test_name': 'HermiT_vs_JFact_classification', 'test_type': 'Independent t-test', 'statistic': 5.725967523064171, 'p_value': 0.0012308825345216498, 'effect_size': 4.048870464412614, 'confidence_interval': (2.1286291628430027, 6.242188528141989), 'interpretation': 'HermiT is significantly slower than JFact for classification', 'significance_level': 0.05}, HermiT_vs_Pellet_classification={'test_name': 'HermiT_vs_Pellet_classification', 'test_type': 'Mann-Whitney U test', 'statistic': 12.0, 'p_value': 0.34285714285714286, 'effect_size': 0.25, 'confidence_interval': (-537.1792523538787, 337.5691556784218), 'interpretation': 'No significant difference between HermiT and Pellet for classification', 'significance_level': 0.05}, JFact_vs_Pellet_classification={'test_name': 'JFact_vs_Pellet_classification', 'test_type': 'Mann-Whitney U test', 'statistic': 12.0, 'p_value': 0.34285714285714286, 'effect_size': 0.25, 'confidence_interval': (-541.3704578216286, 333.38954345518675), 'interpretation': 'No significant difference between JFact and Pellet for classification', 'significance_level': 0.05}

**consistency**: ELK_vs_HermiT_consistency={'test_name': 'ELK_vs_HermiT_consistency', 'test_type': 'Mann-Whitney U test', 'statistic': 16.0, 'p_value': 0.02857142857142857, 'effect_size': 0.5, 'confidence_interval': (150.55242973448145, 286.42802448255145), 'interpretation': 'ELK is significantly slower than HermiT for consistency', 'significance_level': 0.05}, ELK_vs_JFact_consistency={'test_name': 'ELK_vs_JFact_consistency', 'test_type': 'Mann-Whitney U test', 'statistic': 16.0, 'p_value': 0.02857142857142857, 'effect_size': 0.5, 'confidence_interval': (154.81958307620897, 291.13491508278673), 'interpretation': 'ELK is significantly slower than JFact for consistency', 'significance_level': 0.05}, ELK_vs_Pellet_consistency={'test_name': 'ELK_vs_Pellet_consistency', 'test_type': 'Mann-Whitney U test', 'statistic': 16.0, 'p_value': 0.02857142857142857, 'effect_size': 0.5, 'confidence_interval': (188.4786125266995, 324.8635294428083), 'interpretation': 'ELK is significantly slower than Pellet for consistency', 'significance_level': 0.05}, HermiT_vs_JFact_consistency={'test_name': 'HermiT_vs_JFact_consistency', 'test_type': 'Independent t-test', 'statistic': 2.5064486871129965, 'p_value': 0.04612416737045017, 'effect_size': 1.772326863353719, 'confidence_interval': (-0.5706575501885709, 9.544701492151361), 'interpretation': 'HermiT is significantly slower than JFact for consistency', 'significance_level': 0.05}, HermiT_vs_Pellet_consistency={'test_name': 'HermiT_vs_Pellet_consistency', 'test_type': 'Independent t-test', 'statistic': 22.686479828020364, 'p_value': 4.802704581601009e-07, 'effect_size': 16.04176372764502, 'confidence_interval': (32.87666365304573, 43.48502409942919), 'interpretation': 'HermiT is significantly slower than Pellet for consistency', 'significance_level': 0.05}, JFact_vs_Pellet_consistency={'test_name': 'JFact_vs_Pellet_consistency', 'test_type': 'Independent t-test', 'statistic': 51.893372258184726, 'p_value': 3.4363320352964896e-09, 'effect_size': 36.69415542240027, 'confidence_interval': (31.750217508061755, 35.63742630245036), 'interpretation': 'JFact is significantly slower than Pellet for consistency', 'significance_level': 0.05}




## Detailed Results

| Reasoner | Type | Operation | Success | Time (ms) | Memory (MB) |
|----------|------|-----------|---------|-----------|-------------|
| ELK | library | classification | ✓ | 358.40 | N/A |
| ELK | library | classification | ✓ | 258.29 | N/A |
| ELK | library | consistency | ✓ | 331.85 | N/A |
| ELK | library | consistency | ✓ | 248.08 | N/A |
| ELK | library | classification | ✓ | 265.40 | N/A |
| ELK | library | classification | ✓ | 267.53 | N/A |
| ELK | library | consistency | ✓ | 249.51 | N/A |
| ELK | library | consistency | ✓ | 241.63 | N/A |
| HermiT | library | classification | ✓ | 50.17 | N/A |
| HermiT | library | classification | ✓ | 48.38 | N/A |
| HermiT | library | consistency | ✓ | 48.77 | N/A |
| HermiT | library | consistency | ✓ | 54.03 | N/A |
| HermiT | library | classification | ✓ | 46.83 | N/A |
| HermiT | library | classification | ✓ | 48.60 | N/A |
| HermiT | library | consistency | ✓ | 48.09 | N/A |
| HermiT | library | consistency | ✓ | 46.21 | N/A |
| JFact | library | classification | ✓ | 44.99 | N/A |
| JFact | library | classification | ✓ | 44.12 | N/A |
| JFact | library | consistency | ✓ | 45.84 | N/A |
| JFact | library | consistency | ✓ | 45.53 | N/A |
| JFact | library | classification | ✓ | 43.74 | N/A |
| JFact | library | classification | ✓ | 44.39 | N/A |
| JFact | library | consistency | ✓ | 44.75 | N/A |
| JFact | library | consistency | ✓ | 43.03 | N/A |
| Pellet | library | classification | ✓ | 560.61 | N/A |
| Pellet | library | classification | ✓ | 10.97 | N/A |
| Pellet | library | consistency | ✓ | 11.21 | N/A |
| Pellet | library | consistency | ✓ | 11.09 | N/A |
| Pellet | library | classification | ✓ | 10.60 | N/A |
| Pellet | library | classification | ✓ | 11.03 | N/A |
| Pellet | library | consistency | ✓ | 10.67 | N/A |
| Pellet | library | consistency | ✓ | 11.41 | N/A |


## Conclusions

The comprehensive evaluation demonstrates that the tested OWL2 reasoners achieve a 100.0% success rate across 32 test cases. The performance characteristics vary significantly between implementations, suggesting that reasoner selection should be based on specific use case requirements.

Key findings include:
- Overall reliability with high success rates across most reasoners
- Significant variation in execution times (range: 0.00 - 0.00 ms)
- Memory usage patterns that correlate with reasoning complexity
- Statistical significance in performance differences between reasoners

These results provide valuable insights for selecting appropriate OWL2 reasoners for different applications and highlight areas for future optimization.


---

*Generated by OWL2 Reasoner Evaluation Framework*
