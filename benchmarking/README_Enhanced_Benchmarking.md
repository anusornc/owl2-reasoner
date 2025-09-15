# Enhanced OWL2 Reasoner Benchmark Framework

This document provides a comprehensive guide to the enhanced OWL2 reasoner benchmark framework that integrates LUBM and SP2B benchmarks with the existing testing infrastructure.

## Overview

The enhanced benchmark framework extends the original testing infrastructure to include:
- **LUBM (Lehigh University Benchmark)**: Standardized university domain ontologies
- **SP2B (SPARQL Performance Benchmark)**: Social network data adapted for OWL2 reasoning
- **Scalability Testing**: Progressive testing across different dataset sizes
- **Comprehensive Analytics**: Statistical analysis and performance comparison

## Quick Start

### 1. Setup Benchmark Infrastructure

```bash
# Navigate to benchmarking directory
cd /Users/anusornchaikaew/Work/Phd/KnowledgeGraph/owl2-reasoner/benchmarking

# Run the setup script
python setup_benchmarks.py --base-dir benchmarking
```

This will:
- Create benchmark directories
- Download LUBM base ontology
- Setup LUBM and SP2B queries
- Create data generators
- Generate configuration file
- Create test datasets

### 2. Configure Reasoners

Edit the generated `config.json` file to match your reasoner installations:

```json
{
  "reasoners": {
    "rust_owl2": {
      "name": "Rust OWL2 Reasoner",
      "command": "cargo run --example",
      "working_dir": "../../",
      "classification_cmd": "cargo run --example classification_check --",
      "consistency_cmd": "cargo run --example consistency_check --",
      "query_cmd": "cargo run --example query_check --"
    },
    "elk": {
      "name": "ELK Reasoner",
      "command": "java -jar elk.jar",
      "classification_cmd": "java -jar elk.jar -c",
      "consistency_cmd": "java -jar elk.jar -s"
    }
  }
}
```

### 3. Run Comprehensive Benchmark

```bash
# Run all benchmarks for all configured reasoners
python enhanced_benchmark_framework.py

# Run specific reasoners
python enhanced_benchmark_framework.py --reasoners rust_owl2 elk

# Run with more iterations for statistical significance
python enhanced_benchmark_framework.py --iterations 5
```

## Benchmark Suites

### LUBM Benchmark

**Purpose**: Test performance on standardized university domain ontologies

**Datasets**:
- 1 university (~10K triples)
- 10 universities (~100K triples)
- 100 universities (~1M triples)

**Operations**:
- Classification: Hierarchical classification of university entities
- Consistency: Consistency checking of university axioms
- Query: Complex query answering on university data

**Queries**:
1. **Query 1**: Find persons working for organizations
2. **Query 2**: Find students taking courses
3. **Query 3**: Find faculty members
4. **Query 4**: Find department members
5. **Query 5**: Find university members

Example Query 1:
```sparql
PREFIX ub: <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>
SELECT ?person
WHERE {
    ?person a [
        owl:intersectionOf (
            ub:Person
            [
                owl:onProperty ub:worksFor ;
                owl:someValuesFrom ub:Organization
            ]
        )
    ] .
}
```

### SP2B Benchmark

**Purpose**: Test performance on social network data with reasoning requirements

**Datasets**:
- Scale 1 (~1K nodes, 10K edges)
- Scale 10 (~10K nodes, 100K edges)
- Scale 100 (~100K nodes, 1M edges)

**Operations**:
- Classification: Type inference in social networks
- Consistency: Consistency of social network axioms
- Query: Complex social network queries with reasoning

**Queries**:
1. **Query 1**: Friends of friends (transitive reasoning)
2. **Query 2**: Interest classification (type inference)
3. **Query 3**: Organization hierarchy (hierarchical reasoning)

Example Query 1:
```sparql
PREFIX foaf: <http://xmlns.com/foaf/0.1/>
SELECT DISTINCT ?person WHERE {
    ?person foaf:knows ?friend .
    ?friend foaf:knows ?friend_of_friend .
    FILTER(?person != ?friend_of_friend)
}
```

### Scalability Benchmark

**Purpose**: Test performance across different dataset sizes to identify scalability limits

**Datasets**:
- Small: 1K entities, 5K axioms
- Medium: 10K entities, 50K axioms
- Large: 100K entities, 500K axioms

**Operations**:
- Classification: Performance on increasingly complex ontologies
- Consistency: Scalability of consistency checking

### BioPortal Benchmark (Optional)

**Purpose**: Test performance on real-world biomedical ontologies

**Requirements**:
- BioPortal API key (set `BIOPORTAL_API_KEY` environment variable)
- Internet connection for ontology downloads

**Ontologies**:
- Gene Ontology (GO)
- SNOMED CT
- Human Disease Ontology (DOID)
- Chemical Entities of Biological Interest (CHEBI)

## Configuration

### Benchmark Configuration

The `config.json` file controls which benchmarks are enabled:

```json
{
  "benchmarks": {
    "lubm": {
      "enabled": true,
      "university_counts": [1, 10, 100],
      "queries": ["query1", "query2", "query3", "query4", "query5"],
      "iterations": 3
    },
    "sp2b": {
      "enabled": true,
      "scale_factors": [1, 10, 100],
      "queries": ["sp2b_query_1", "sp2b_query_2", "sp2b_query_3"],
      "iterations": 3
    },
    "scalability": {
      "enabled": true,
      "scales": ["small", "medium", "large"],
      "iterations": 3
    },
    "bioportal": {
      "enabled": false,
      "api_key": null,
      "ontologies": ["GO", "SNOMEDCT", "DOID"],
      "iterations": 3
    }
  }
}
```

### Reasoner Configuration

Each reasoner requires specific command configurations:

#### Rust OWL2 Reasoner
```json
{
  "rust_owl2": {
    "name": "Rust OWL2 Reasoner",
    "command": "cargo run --example",
    "working_dir": "../../",
    "classification_cmd": "cargo run --example classification_check --",
    "consistency_cmd": "cargo run --example consistency_check --",
    "query_cmd": "cargo run --example query_check --"
  }
}
```

#### ELK Reasoner
```json
{
  "elk": {
    "name": "ELK Reasoner",
    "command": "java -jar elk.jar",
    "classification_cmd": "java -jar elk.jar -c",
    "consistency_cmd": "java -jar elk.jar -s"
  }
}
```

#### HermiT Reasoner
```json
{
  "hermit": {
    "name": "HermiT Reasoner",
    "command": "java -jar hermit.jar",
    "classification_cmd": "java -jar hermit.jar -c",
    "consistency_cmd": "java -jar hermit.jar -k"
  }
}
```

## Output and Results

### Generated Reports

The framework generates several types of reports:

1. **Markdown Report** (`comprehensive_report_*.md`)
   - Executive summary with rankings
   - Detailed results for each benchmark suite
   - Performance tables and analysis

2. **JSON Data** (`comprehensive_results_*.json`)
   - Complete raw data for further analysis
   - Machine-readable format for automation
   - Statistical comparisons and scores

3. **Performance Charts** (if matplotlib is available)
   - Performance comparison charts
   - Scalability analysis plots
   - Success rate visualizations

### Performance Metrics

For each test, the following metrics are collected:

- **Execution Time**: Time in milliseconds for the complete operation
- **Memory Usage**: Peak memory consumption during operation
- **Success Rate**: Percentage of successful operations
- **Output Size**: Size of generated output files
- **Error Messages**: Detailed error information for failures

### Statistical Analysis

The framework provides comprehensive statistical analysis:

- **Performance Scores**: Weighted scores across different benchmarks
- **Rankings**: Overall and per-benchmark-suite rankings
- **Scalability Analysis**: Performance degradation across scales
- **Robustness Analysis**: Success rates and error patterns

Example Performance Score Calculation:
```
Overall Score = (LUBM_Score × 0.4) + (SP2B_Score × 0.3) + (Custom_Score × 0.2) + (Scalability_Score × 0.1)
```

## Directory Structure

After setup, the benchmark directory structure looks like this:

```
benchmarking/
├── config.json                              # Configuration file
├── setup_benchmarks.py                      # Setup script
├── enhanced_benchmark_framework.py          # Main benchmark framework
├── README_Enhanced_Benchmarking.md          # This documentation
├── benchmarks/                              # Benchmark data and configurations
│   ├── lubm/                               # LUBM benchmark
│   │   ├── data/                           # LUBM ontologies
│   │   │   ├── univ-bench.owl              # Base ontology
│   │   │   ├── university1.owl             # 1 university dataset
│   │   │   └── university10.owl            # 10 universities dataset
│   │   ├── queries/                        # LUBM queries
│   │   │   ├── query1.sparql               # Standard LUBM queries
│   │   │   └── query2.sparql
│   │   └── generator/                       # Data generator
│   │       └── lubm_generator.py           # LUBM data generator
│   ├── sp2b/                               # SP2B benchmark
│   │   ├── data/                           # SP2B datasets
│   │   │   ├── sp2b_scale_1.ttl            # Scale 1 dataset
│   │   │   └── sp2b_scale_10.ttl           # Scale 10 dataset
│   │   ├── queries/                        # SP2B queries
│   │   │   └── sp2b_query_1.sparql         # Social network queries
│   │   └── generator/                       # Data generator
│   │       └── sp2b_generator.py           # SP2B data generator
│   ├── scalability/                        # Scalability benchmark
│   │   └── ontologies/                     # Test ontologies
│   │       ├── scalability_small.owl       # Small scale ontology
│   │       ├── scalability_medium.owl      # Medium scale ontology
│   │       └── scalability_large.owl       # Large scale ontology
│   └── bioportal/                          # BioPortal benchmark
│       ├── ontologies/                     # Downloaded ontologies
│       └── queries/                        # BioPortal queries
├── results/                                # Benchmark results
│   ├── comprehensive_report_*.md           # Markdown reports
│   ├── comprehensive_results_*.json         # JSON data
│   └── performance_charts.png              # Performance charts
└── established_reasoners/                  # Original testing framework
    ├── test_all_reasoners.py               # Original test runner
    └── benchmark_framework.py              # Original benchmark framework
```

## Advanced Usage

### Custom Benchmark Suites

To add custom benchmark suites:

1. **Create benchmark directory structure**:
   ```bash
   mkdir -p benchmarks/custom/{data,queries,generator}
   ```

2. **Add custom ontologies** to `benchmarks/custom/data/`

3. **Create custom queries** in `benchmarks/custom/queries/`

4. **Extend the framework** in `enhanced_benchmark_framework.py`

### Memory Profiling

To enable memory profiling, install psutil:

```bash
pip install psutil
```

The framework will automatically collect memory usage metrics.

### Parallel Execution

For faster execution on multi-core systems, you can modify the framework to run benchmarks in parallel. Add this import and modify the run loop:

```python
from concurrent.futures import ThreadPoolExecutor

def run_parallel_benchmarks(self, reasoners: List[str], iterations: int):
    """Run benchmarks in parallel"""
    with ThreadPoolExecutor(max_workers=len(reasoners)) as executor:
        futures = []
        for reasoner_name in reasoners:
            future = executor.submit(self.run_reasoner_benchmarks, reasoner_name, iterations)
            futures.append(future)

        all_results = []
        for future in futures:
            all_results.extend(future.result())

    return all_results
```

### Continuous Integration

The framework can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
name: Benchmark Tests
on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Setup Python
      uses: actions/setup-python@v2
      with:
        python-version: '3.9'
    - name: Install dependencies
      run: pip install rdflib psutil matplotlib
    - name: Run benchmarks
      run: |
        cd benchmarking
        python setup_benchmarks.py
        python enhanced_benchmark_framework.py --reasoners rust_owl2 --iterations 3
    - name: Upload results
      uses: actions/upload-artifact@v2
      with:
        name: benchmark-results
        path: benchmarking/results/
```

## Troubleshooting

### Common Issues

1. **Reasoner not found**:
   - Verify reasoner paths in `config.json`
   - Ensure reasoner executables are in PATH or provide full paths

2. **Timeout errors**:
   - Increase timeout values in the framework
   - Check for infinite loops or very large datasets

3. **Memory issues**:
   - Reduce dataset sizes in configuration
   - Increase available memory or use swap space

4. **Missing dependencies**:
   ```bash
   pip install rdflib psutil matplotlib scipy
   ```

5. **LUBM/SP2B data generation fails**:
   - Check Python version (requires 3.6+)
   - Verify write permissions in benchmark directories

### Debug Mode

Enable debug output by adding `--verbose` flag:

```bash
python enhanced_benchmark_framework.py --verbose
```

This will provide detailed logging of each benchmark execution.

## Performance Optimization Tips

### For Benchmark Framework

1. **Use SSD storage** for faster dataset loading
2. **Increase memory** to avoid swapping during large benchmarks
3. **Disable unnecessary services** to reduce system noise
4. **Use consistent environment** for reproducible results

### For Reasoner Testing

1. **Warm up reasoners** with initial runs
2. **Use multiple iterations** for statistical significance
3. **Monitor system resources** during testing
4. **Document environment** specifications (CPU, memory, OS)

## Publication Guidelines

### For Academic Papers

When using this framework for academic publications, include:

1. **Framework version**: Specify the exact version used
2. **Configuration details**: Include relevant config settings
3. **Environment specifications**: Hardware and software details
4. **Statistical methodology**: Number of iterations, significance testing
5. **Data availability**: Make raw results available

### Example Citation

```
@software{owl2_benchmark_framework,
  title={Enhanced OWL2 Reasoner Benchmark Framework with LUBM and SP2B Integration},
  author={Your Name},
  year={2024},
  url={https://github.com/your-repo/owl2-reasoner-benchmark},
  note={Version 1.0}
}
```

### Reproducibility Checklist

- [ ] Include configuration file
- [ ] Document environment specifications
- [ ] Provide raw benchmark data
- [ ] Specify framework version
- [ ] Include statistical analysis details
- [ ] Document any custom modifications

## Contributing

### Adding New Benchmarks

1. Create benchmark directory structure
2. Implement benchmark class following existing patterns
3. Add configuration options
4. Update documentation
5. Add tests if applicable

### Reporting Issues

When reporting issues, please include:

- Framework version
- Configuration file
- Complete error messages
- Environment details
- Steps to reproduce

## License

This framework is part of the OWL2 Reasoner project and follows the same license terms.

## Support

For questions or support:
- Check the documentation
- Review example configurations
- Examine generated reports
- Check the troubleshooting section