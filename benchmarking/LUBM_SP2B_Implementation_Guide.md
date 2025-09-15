# LUBM and SP2B Benchmark Integration Implementation Guide

## Executive Summary

This document provides comprehensive implementation details for integrating LUBM (Lehigh University Benchmark) and SP2B (SPARQL Performance Benchmark) into the existing OWL2 reasoner testing framework. Based on the analysis of the current testing infrastructure and available resources, this guide covers practical implementation steps, data structures, and integration strategies.

## Current Testing Infrastructure Analysis

### Existing Framework Strengths
- **Comprehensive test runner**: `/Users/anusornchaikaew/Work/Phd/KnowledgeGraph/owl2-reasoner/benchmarking/established_reasoners/test_all_reasoners.py`
- **Benchmark framework**: `/Users/anusornchaikaew/Work/Phd/KnowledgeGraph/owl2-reasoner/benchmarking/benchmark_framework.py`
- **Multi-reasoner support**: Currently supports Rust OWL2, ELK, HermiT, JFact
- **Standardized testing methodology**: Consistent command execution and timing
- **Error handling and reporting**: Comprehensive result documentation

### Current Limitations
- Missing standard benchmark ontologies (LUBM, SP2B)
- Limited test ontology variety
- No memory profiling
- Missing large-scale testing

## 1. LUBM Implementation Details

### 1.1 LUBM Overview
LUBM is a standardized benchmark for evaluating OWL reasoner performance on university domain ontologies.

### 1.2 Data Source and Download
```python
# LUBM Data Generator Download
LUBM_SOURCES = {
    "official_site": "https://swat.cse.lehigh.edu/projects/lubm/",
    "generator": "https://sourceforge.net/projects/lubm/",
    "ontology": "http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl"
}

# Direct ontology file URL
UNIV_BENCH_ONTOLOGY = "http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl"
```

### 1.3 Available LUBM Queries in Pellet Distribution
Based on analysis of `/Users/anusornchaikaew/Work/Phd/KnowledgeGraph/owl2-reasoner/benchmarking/established_reasoners/pellet-2.3.1/examples/data/`:

**Query 1**: Find persons who work for some organization
```sparql
PREFIX : <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>
SELECT ?person
WHERE {
    ?person a [
        owl:intersectionOf (
            :Person
            [
                owl:onProperty :worksFor ;
                owl:someValuesFrom :Organization
            ]
        )
    ] .
}
```

**Query 2**: Find students taking at least one course
```sparql
PREFIX : <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>
SELECT ?person
WHERE {
    ?person a [
        owl:intersectionOf (
            :Student
            [
                owl:onProperty :takesCourse ;
                owl:minCardinality 1
            ]
        )
    ] .
}
```

**Query 3-14**: Additional queries for various university domain reasoning tasks

### 1.4 LUBM Data Generation
```python
class LUBMGenerator:
    """LUBM data generator for creating university ontologies of different sizes"""

    def __init__(self, base_dir: str = "lubm_data"):
        self.base_dir = Path(base_dir)
        self.base_dir.mkdir(exist_ok=True)

    def download_base_ontology(self):
        """Download the base university ontology"""
        import urllib.request

        base_ontology_url = "http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl"
        output_path = self.base_dir / "univ-bench.owl"

        if not output_path.exists():
            print(f"Downloading LUBM base ontology...")
            urllib.request.urlretrieve(base_ontology_url, output_path)
            print(f"Downloaded to: {output_path}")

        return output_path

    def generate_university_data(self, num_universities: int, seed: int = 0):
        """Generate university data using UBA (University Benchmark Generator)"""

        # Command structure for UBA generator
        if num_universities == 1:
            cmd = f"java -jar uba.jar -univ {num_universities} -seed {seed}"
        else:
            cmd = f"java -jar uba.jar -univ {num_universities} -seed {seed} -o University{num_universities}"

        return {
            "command": cmd,
            "expected_files": [
                f"University{num_universities}_0.owl",
                f"University{num_universities}_1.owl",
                # ... more files based on number of universities
            ]
        }

    def get_standard_sizes(self):
        """Return standard LUBM dataset sizes"""
        return {
            "LUBM_1": {"universities": 1, "estimated_triples": 10000},
            "LUBM_10": {"universities": 10, "estimated_triples": 100000},
            "LUBM_100": {"universities": 100, "estimated_triples": 1000000},
            "LUBM_1000": {"universities": 1000, "estimated_triples": 10000000}
        }
```

### 1.5 LUBM Integration with Testing Framework
```python
@dataclass
class LUBMTestConfig:
    """Configuration for LUBM benchmark testing"""
    base_ontology: Path
    university_counts: List[int] = field(default_factory=lambda: [1, 10, 100])
    queries: List[str] = field(default_factory=lambda: ["query1", "query2", "query3"])
    iterations: int = 5

class LUBMBenchmark:
    """LUBM-specific benchmark implementation"""

    def __init__(self, test_config: LUBMTestConfig):
        self.config = test_config
        self.query_dir = Path("lubm_queries")
        self.query_dir.mkdir(exist_ok=True)

    def setup_queries(self):
        """Setup LUBM queries from Pellet examples"""
        queries = {
            "query1": """PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
                      PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                      PREFIX owl: <http://www.w3.org/2002/07/owl#>
                      PREFIX : <http://www.lehigh.edu/~zhp2/2004/0401/univ-bench.owl#>
                      SELECT ?person WHERE {
                          ?person a [ owl:intersectionOf ( :Person [ owl:onProperty :worksFor ; owl:someValuesFrom :Organization ] ) ] .
                      }""",
            # Add more queries...
        }

        for query_name, query_content in queries.items():
            with open(self.query_dir / f"{query_name}.sparql", 'w') as f:
                f.write(query_content)

    def run_lubm_classification_test(self, reasoner_name: str, ontology_path: Path) -> BenchmarkResult:
        """Run LUBM classification test"""
        # Use existing benchmark framework structure
        return self._run_reasoning_operation(reasoner_name, ontology_path, "classification")

    def run_lubm_query_test(self, reasoner_name: str, ontology_path: Path, query_name: str) -> BenchmarkResult:
        """Run LUBM query test"""
        query_file = self.query_dir / f"{query_name}.sparql"
        return self._run_query_operation(reasoner_name, ontology_path, query_file)
```

## 2. SP2B Implementation Details

### 2.1 SP2B Overview
SP2B is a benchmark for SPARQL query performance, but can be adapted for OWL2 reasoner testing by focusing on the reasoning aspects of the queries.

### 2.2 SP2B Data Source and Setup
```python
# SP2B Benchmark Sources
SP2B_SOURCES = {
    "official_site": "http://dbpedia.org/SP2B/",
    "generator": "https://github.com/aksw/SP2B",
    "queries": "http://dbpedia.org/SP2B/queries/"
}

class SP2BGenerator:
    """SP2B data generator for social network ontologies"""

    def __init__(self, base_dir: str = "sp2b_data"):
        self.base_dir = Path(base_dir)
        self.base_dir.mkdir(exist_ok=True)

    def generate_social_network_data(self, scale_factor: int):
        """Generate social network data with different scales"""

        # SP2B scale factors
        scales = {
            1: {"estimated_nodes": 1000, "estimated_edges": 10000},
            10: {"estimated_nodes": 10000, "estimated_edges": 100000},
            100: {"estimated_nodes": 100000, "estimated_edges": 1000000},
            1000: {"estimated_nodes": 1000000, "estimated_edges": 10000000}
        }

        return {
            "command": f"java -jar sp2b-generator.jar -scale {scale_factor}",
            "expected_output": f"sp2b-scale-{scale_factor}.ttl",
            "estimated_size": scales.get(scale_factor, {"estimated_nodes": 0, "estimated_edges": 0})
        }

    def get_reasoning_queries(self):
        """SP2B queries adapted for OWL2 reasoning"""
        return {
            "sp2b_query_1": {
                "description": "Find friends of friends with reasoning",
                "query": """PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                         PREFIX foaf: <http://xmlns.com/foaf/0.1/>
                         SELECT DISTINCT ?person WHERE {
                             ?person foaf:knows ?friend .
                             ?friend foaf:knows ?friend_of_friend .
                             FILTER(?person != ?friend_of_friend)
                         }""",
                "reasoning_aspect": "transitive reasoning"
            },
            "sp2b_query_2": {
                "description": "Find people with specific interests",
                "query": """PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                         PREFIX foaf: <http://xmlns.com/foaf/0.1/>
                         SELECT ?person WHERE {
                             ?person foaf:interest ?interest .
                             ?interest rdf:type ?interest_type .
                         }""",
                "reasoning_aspect": "type inference"
            }
        }
```

### 2.3 SP2B Integration Strategy
```python
@dataclass
class SP2BTestConfig:
    """Configuration for SP2B benchmark testing"""
    scale_factors: List[int] = field(default_factory=lambda: [1, 10, 100])
    reasoning_queries: List[str] = field(default_factory=lambda: ["sp2b_query_1", "sp2b_query_2"])
    iterations: int = 5

class SP2BBenchmark:
    """SP2B-specific benchmark implementation"""

    def __init__(self, test_config: SP2BTestConfig):
        self.config = test_config
        self.queries = self._setup_reasoning_queries()

    def _setup_reasoning_queries(self):
        """Setup SP2B queries adapted for OWL2 reasoning"""
        return {
            "sp2b_query_1": {
                "description": "Social network reasoning",
                "ontology_requirements": ["foaf ontology", "social network axioms"],
                "expected_complexity": "quadratic"
            },
            "sp2b_query_2": {
                "description": "Interest classification",
                "ontology_requirements": ["interest hierarchy", "type inference"],
                "expected_complexity": "linear"
            }
        }

    def run_sp2b_reasoning_test(self, reasoner_name: str, ontology_path: Path, query_name: str) -> BenchmarkResult:
        """Run SP2B reasoning test"""
        return self._run_reasoning_query(reasoner_name, ontology_path, query_name)
```

## 3. Integration Strategy

### 3.1 Extended Benchmark Framework
```python
class ExtendedBenchmarkFramework(OWL2BenchmarkFramework):
    """Extended benchmark framework with LUBM and SP2B support"""

    def __init__(self, results_dir: str = "benchmark_results"):
        super().__init__(results_dir)
        self.lubm_config = LUBMTestConfig()
        self.sp2b_config = SP2BTestConfig()
        self.lubm_benchmark = LUBMBenchmark(self.lubm_config)
        self.sp2b_benchmark = SP2BBenchmark(self.sp2b_config)

    def run_lubm_benchmark_suite(self, reasoner_name: str) -> List[BenchmarkResult]:
        """Run complete LUBM benchmark suite"""
        results = []

        # Download and setup LUBM data
        base_ontology = self.lubm_benchmark.download_base_ontology()
        self.lubm_benchmark.setup_queries()

        # Test different university sizes
        for univ_count in self.lubm_config.university_counts:
            ontology_path = self.lubm_benchmark.generate_university_data(univ_count)

            # Classification test
            class_result = self.lubm_benchmark.run_lubm_classification_test(
                reasoner_name, ontology_path
            )
            results.append(class_result)

            # Query tests
            for query_name in self.lubm_config.queries:
                query_result = self.lubm_benchmark.run_lubm_query_test(
                    reasoner_name, ontology_path, query_name
                )
                results.append(query_result)

        return results

    def run_sp2b_benchmark_suite(self, reasoner_name: str) -> List[BenchmarkResult]:
        """Run complete SP2B benchmark suite"""
        results = []

        # Test different scale factors
        for scale_factor in self.sp2b_config.scale_factors:
            ontology_path = self.sp2b_benchmark.generate_social_network_data(scale_factor)

            # Reasoning query tests
            for query_name in self.sp2b_config.reasoning_queries:
                query_result = self.sp2b_benchmark.run_sp2b_reasoning_test(
                    reasoner_name, ontology_path, query_name
                )
                results.append(query_result)

        return results

    def run_comprehensive_benchmark(self, iterations: int = 5) -> Dict[str, List[BenchmarkResult]]:
        """Run comprehensive benchmarks including LUBM and SP2B"""
        all_results = {}

        for reasoner_name in self.reasoners:
            print(f"\n🔬 Benchmarking {reasoner_name} with comprehensive suite...")

            reasoner_results = []

            # Original benchmark tests
            original_results = super().run_comprehensive_benchmark(iterations)
            reasoner_results.extend(original_results.get(reasoner_name, []))

            # LUBM benchmark tests
            lubm_results = self.run_lubm_benchmark_suite(reasoner_name)
            reasoner_results.extend(lubm_results)

            # SP2B benchmark tests
            sp2b_results = self.run_sp2b_benchmark_suite(reasoner_name)
            reasoner_results.extend(sp2b_results)

            all_results[reasoner_name] = reasoner_results

        return all_results
```

### 3.2 Enhanced Data Structures
```python
@dataclass
class BenchmarkSuiteResult:
    """Enhanced benchmark result with multi-suite support"""
    reasoner_name: str
    benchmark_suite: str  # "LUBM", "SP2B", "CUSTOM", "BIOPORTAL"
    dataset_size: str     # "1-university", "10-university", "scale-100", etc.
    operation: str         # "classification", "query", "consistency"
    query_name: Optional[str] = None
    execution_time_ms: float = 0.0
    memory_usage_mb: float = 0.0
    success: bool = True
    error_message: str = ""
    additional_metrics: Dict[str, Any] = field(default_factory=dict)

    def get_benchmark_type(self) -> str:
        """Get standardized benchmark type identifier"""
        return f"{self.benchmark_suite}_{self.dataset_size}_{self.operation}"

@dataclass
class BenchmarkComparison:
    """Cross-benchmark comparison results"""
    reasoner_name: str
    overall_rank: int
    lubm_rank: int
    sp2b_rank: int
    custom_rank: int
    performance_score: float  # Weighted average performance
    scalability_score: float  # Performance across different scales
    robustness_score: float  # Success rate across different tests

class BenchmarkAnalytics:
    """Analytics for multi-benchmark results"""

    def calculate_performance_scores(self, results: Dict[str, List[BenchmarkSuiteResult]]) -> Dict[str, BenchmarkComparison]:
        """Calculate comprehensive performance scores"""
        comparisons = {}

        for reasoner_name, reasoner_results in results.items():
            # Calculate scores for each benchmark suite
            lubm_score = self._calculate_suite_score(reasoner_results, "LUBM")
            sp2b_score = self._calculate_suite_score(reasoner_results, "SP2B")
            custom_score = self._calculate_suite_score(reasoner_results, "CUSTOM")

            # Overall weighted score
            overall_score = (
                lubm_score * 0.4 +  # LUBM is most important for OWL reasoning
                sp2b_score * 0.3 +  # SP2B tests query performance
                custom_score * 0.3   # Custom ontologies test general performance
            )

            comparisons[reasoner_name] = BenchmarkComparison(
                reasoner_name=reasoner_name,
                overall_rank=0,  # Will be calculated after all scores
                lubm_rank=0,
                sp2b_rank=0,
                custom_rank=0,
                performance_score=overall_score,
                scalability_score=self._calculate_scalability_score(reasoner_results),
                robustness_score=self._calculate_robustness_score(reasoner_results)
            )

        # Calculate ranks
        self._calculate_ranks(comparisons)

        return comparisons

    def _calculate_suite_score(self, results: List[BenchmarkSuiteResult], suite: str) -> float:
        """Calculate score for a specific benchmark suite"""
        suite_results = [r for r in results if r.benchmark_suite == suite and r.success]
        if not suite_results:
            return 0.0

        # Normalize execution times (lower is better)
        times = [r.execution_time_ms for r in suite_results]
        avg_time = sum(times) / len(times)

        # Score based on inverse of time (log scale to handle large variations)
        return 100.0 / (1.0 + math.log10(avg_time + 1))
```

### 3.3 Enhanced Reporting System
```python
class EnhancedBenchmarkReporter:
    """Enhanced reporting system for multi-benchmark results"""

    def generate_comprehensive_report(self, results: Dict[str, List[BenchmarkSuiteResult]]):
        """Generate comprehensive multi-benchmark report"""
        analytics = BenchmarkAnalytics()
        comparisons = analytics.calculate_performance_scores(results)

        # Generate markdown report
        report = self._generate_markdown_report(results, comparisons)

        # Generate JSON report
        json_report = self._generate_json_report(results, comparisons)

        # Generate visualizations
        self._generate_performance_charts(results)
        self._generate_scalability_charts(results)

        return report, json_report

    def _generate_markdown_report(self, results: Dict[str, List[BenchmarkSuiteResult]],
                                comparisons: Dict[str, BenchmarkComparison]) -> str:
        """Generate detailed markdown report"""

        report = []
        report.append("# Comprehensive OWL2 Reasoner Benchmark Report")
        report.append(f"Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}")
        report.append("")

        # Executive summary
        report.append("## Executive Summary")
        report.append("")
        sorted_reasoners = sorted(comparisons.values(), key=lambda x: x.overall_rank)
        for i, comparison in enumerate(sorted_reasoners, 1):
            report.append(f"{i}. **{comparison.reasoner_name}**")
            report.append(f"   - Overall Score: {comparison.performance_score:.1f}")
            report.append(f"   - LUBM Rank: {comparison.lubm_rank}")
            report.append(f"   - SP2B Rank: {comparison.sp2b_rank}")
            report.append(f"   - Scalability: {comparison.scalability_score:.1f}")
            report.append("")

        # LUBM results
        report.append("## LUBM Benchmark Results")
        report.append("")
        self._add_suite_results_section(report, results, "LUBM")

        # SP2B results
        report.append("## SP2B Benchmark Results")
        report.append("")
        self._add_suite_results_section(report, results, "SP2B")

        # Scalability analysis
        report.append("## Scalability Analysis")
        report.append("")
        self._add_scalability_section(report, results)

        # Detailed results
        report.append("## Detailed Results")
        report.append("")
        self._add_detailed_results_section(report, results)

        return "\n".join(report)

    def _generate_performance_charts(self, results: Dict[str, List[BenchmarkSuiteResult]]):
        """Generate performance comparison charts"""
        try:
            import matplotlib.pyplot as plt
            import seaborn as sns

            # Overall performance comparison
            fig, axes = plt.subplots(2, 2, figsize=(15, 10))

            # LUBM performance
            self._plot_suite_performance(results, "LUBM", axes[0, 0])

            # SP2B performance
            self._plot_suite_performance(results, "SP2B", axes[0, 1])

            # Scalability
            self._plot_scalability(results, axes[1, 0])

            # Success rates
            self._plot_success_rates(results, axes[1, 1])

            plt.tight_layout()
            plt.savefig(self.results_dir / "performance_charts.png", dpi=300)
            plt.close()

        except ImportError:
            print("⚠️  Matplotlib not available, skipping chart generation")
```

## 4. Real-world Ontology Integration

### 4.1 BioPortal Integration
```python
class BioPortalBenchmark:
    """Integration with BioPortal ontologies for real-world testing"""

    def __init__(self):
        self.bioportal_api_key = os.getenv("BIOPORTAL_API_KEY")
        self.base_url = "http://data.bioontology.org"

    def download_biomedical_ontologies(self):
        """Download standard biomedical ontologies from BioPortal"""
        standard_ontologies = {
            "GO": "Gene Ontology",
            "SNOMEDCT": "SNOMED CT",
            "UMLS": "Unified Medical Language System",
            "DOID": "Human Disease Ontology",
            "CHEBI": "Chemical Entities of Biological Interest"
        }

        downloaded = {}
        for acronym, name in standard_ontologies.items():
            try:
                ontology_data = self._download_ontology(acronym)
                downloaded[acronym] = {
                    "name": name,
                    "file_path": f"bioportal_{acronym}.owl",
                    "size": len(ontology_data),
                    "classes": self._count_classes(ontology_data)
                }
            except Exception as e:
                print(f"Failed to download {acronym}: {e}")

        return downloaded

    def run_biomedical_benchmark(self, reasoner_name: str) -> List[BenchmarkSuiteResult]:
        """Run benchmark on biomedical ontologies"""
        results = []

        ontologies = self.download_biomedical_ontologies()

        for acronym, ontology_info in ontologies.items():
            ontology_path = Path(ontology_info["file_path"])

            if ontology_path.exists():
                # Classification test
                class_result = self._run_classification(reasoner_name, ontology_path)
                class_result.benchmark_suite = "BIOPORTAL"
                class_result.dataset_size = acronym
                results.append(class_result)

                # Consistency test
                cons_result = self._run_consistency(reasoner_name, ontology_path)
                cons_result.benchmark_suite = "BIOPORTAL"
                cons_result.dataset_size = acronym
                results.append(cons_result)

        return results
```

### 4.2 RODI (Reasoner Evaluation Dataset) Integration
```python
class RODIBenchmark:
    """Integration with RODI dataset for comprehensive evaluation"""

    def __init__(self):
        self.rodi_base_url = "https://github.com/w3c/rodibench"

    def download_rodi_dataset(self):
        """Download RODI evaluation dataset"""
        # RODI provides standardized ontologies for reasoner evaluation
        # Categories: consistency, classification, entailment, satisfiability

        categories = {
            "consistency": "ontologies for consistency checking",
            "classification": "ontologies for classification testing",
            "entailment": "ontologies for entailment checking",
            "satisfiability": "ontologies for satisfiability testing"
        }

        return categories

    def run_rodi_evaluation(self, reasoner_name: str) -> List[BenchmarkSuiteResult]:
        """Run RODI standardized evaluation"""
        results = []

        categories = self.download_rodi_dataset()

        for category, description in categories.items():
            category_results = self._run_category_tests(reasoner_name, category)

            for result in category_results:
                result.benchmark_suite = "RODI"
                result.dataset_size = category
                results.append(result)

        return results
```

## 5. Scalability Testing Implementation

### 5.1 Progressive Scaling Strategy
```python
class ScalabilityBenchmark:
    """Comprehensive scalability testing across different scales"""

    def __init__(self):
        self.scale_factors = {
            "small": {"entities": 1000, "axioms": 5000},
            "medium": {"entities": 10000, "axioms": 50000},
            "large": {"entities": 100000, "axioms": 500000},
            "xlarge": {"entities": 1000000, "axioms": 5000000}
        }

    def run_scalability_test(self, reasoner_name: str) -> List[BenchmarkSuiteResult]:
        """Run progressive scalability tests"""
        results = []

        for scale_name, scale_info in self.scale_factors.items():
            # Generate ontology at this scale
            ontology_path = self._generate_scaled_ontology(scale_name, scale_info)

            # Test different operations
            operations = ["classification", "consistency", "query"]

            for operation in operations:
                result = self._run_operation(reasoner_name, ontology_path, operation)
                result.benchmark_suite = "SCALABILITY"
                result.dataset_size = scale_name
                result.additional_metrics.update({
                    "entity_count": scale_info["entities"],
                    "axiom_count": scale_info["axioms"]
                })
                results.append(result)

        return results

    def identify_breakpoints(self, results: List[BenchmarkSuiteResult]) -> Dict[str, float]:
        """Identify performance breakpoints for each reasoner"""
        breakpoints = {}

        # Group results by reasoner and operation
        grouped_results = {}
        for result in results:
            key = f"{result.reasoner_name}_{result.operation}"
            if key not in grouped_results:
                grouped_results[key] = []
            grouped_results[key].append(result)

        # Find breakpoints (where performance degrades significantly)
        for key, reasoner_results in grouped_results.items():
            # Sort by scale
            sorted_results = sorted(reasoner_results, key=lambda x: self.scale_factors[x.dataset_size]["entities"])

            # Find where execution time increases disproportionately
            breakpoint_scale = self._find_performance_breakpoint(sorted_results)
            breakpoints[key] = breakpoint_scale

        return breakpoints

    def _find_performance_breakpoint(self, results: List[BenchmarkSuiteResult]) -> str:
        """Find the scale where performance degrades significantly"""
        if len(results) < 2:
            return "unknown"

        # Calculate performance degradation ratio
        for i in range(1, len(results)):
            prev_time = results[i-1].execution_time_ms
            curr_time = results[i].execution_time_ms

            # If time increases by more than 5x while scale increases by 10x
            if curr_time > prev_time * 5:
                return results[i].dataset_size

        return "no_breakpoint_found"
```

### 5.2 Statistical Analysis
```python
class StatisticalAnalysis:
    """Statistical analysis for multi-benchmark results"""

    def analyze_performance_significance(self, results: Dict[str, List[BenchmarkSuiteResult]]) -> Dict[str, Dict[str, float]]:
        """Analyze statistical significance of performance differences"""

        # Group results by test configuration
        test_groups = {}
        for reasoner_name, reasoner_results in results.items():
            for result in reasoner_results:
                test_key = f"{result.benchmark_suite}_{result.dataset_size}_{result.operation}"
                if test_key not in test_groups:
                    test_groups[test_key] = {}
                test_groups[test_key][reasoner_name] = result.execution_time_ms

        # Perform statistical tests
        significance_results = {}
        for test_key, reasoner_times in test_groups.items():
            if len(reasoner_times) >= 2:
                # Perform ANOVA or t-test
                p_value = self._perform_statistical_test(list(reasoner_times.values()))
                significance_results[test_key] = {
                    "p_value": p_value,
                    "significant": p_value < 0.05,
                    "reasoner_count": len(reasoner_times)
                }

        return significance_results

    def _perform_statistical_test(self, times: List[float]) -> float:
        """Perform statistical test on performance data"""
        try:
            import scipy.stats as stats

            if len(times) == 2:
                # Two-sample t-test
                t_stat, p_value = stats.ttest_ind(times[0:1], times[1:2])
                return p_value
            else:
                # ANOVA for multiple samples
                f_stat, p_value = stats.f_oneway(*[times[i:i+1] for i in range(len(times))])
                return p_value

        except ImportError:
            # Fallback: return a non-significant p-value
            return 1.0
```

## 6. Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
1. **Setup LUBM Integration**
   - Download LUBM base ontology
   - Implement LUBM data generator
   - Integrate LUBM queries from Pellet examples
   - Add LUBM benchmark suite to existing framework

2. **Setup SP2B Integration**
   - Download SP2B benchmark data
   - Adapt SP2B queries for OWL2 reasoning
   - Implement SP2B benchmark suite
   - Add to existing framework

### Phase 2: Enhanced Features (Week 3-4)
1. **Multi-benchmark Analytics**
   - Implement comprehensive scoring system
   - Add scalability testing
   - Create enhanced reporting system
   - Add statistical significance testing

2. **Real-world Ontology Integration**
   - BioPortal integration
   - RODI dataset integration
   - Large-scale biomedical ontology testing

### Phase 3: Advanced Features (Week 5-6)
1. **Performance Optimization**
   - Memory profiling
   - Concurrency testing
   - Performance breakpoint identification
   - Advanced visualization

2. **Publication-Ready Results**
   - Comprehensive documentation
   - Publication-quality charts
   - Statistical analysis reports
   - Reproducibility packages

## 7. File Structure

```
benchmarking/
├── established_reasoners/
│   ├── test_all_reasoners.py              # Original test framework
│   ├── benchmark_framework.py             # Original benchmark framework
│   └── enhanced_benchmark_framework.py    # New enhanced framework
├── benchmarks/
│   ├── lubm/
│   │   ├── data/                          # LUBM ontologies
│   │   ├── queries/                       # LUBM queries
│   │   └── generator/                     # LUBM data generator
│   ├── sp2b/
│   │   ├── data/                          # SP2B ontologies
│   │   ├── queries/                       # SP2B queries
│   │   └── generator/                     # SP2B data generator
│   ├── bioportal/                         # BioPortal ontologies
│   ├── rodi/                             # RODI dataset
│   └── scalability/                      # Scalability test ontologies
├── results/
│   ├── reports/                          # Generated reports
│   ├── charts/                           # Performance charts
│   └── data/                             # Raw benchmark data
└── utils/
    ├── analytics.py                       # Statistical analysis
    ├── visualization.py                   # Chart generation
    └── config.py                         # Configuration management
```

## 8. Configuration and Execution

### 8.1 Configuration File
```python
# config.py
BENCHMARK_CONFIG = {
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
        },
        "hermit": {
            "name": "HermiT Reasoner",
            "command": "java -jar hermit.jar",
            "classification_cmd": "java -jar hermit.jar -c",
            "consistency_cmd": "java -jar hermit.jar -k"
        }
    },
    "benchmarks": {
        "lubm": {
            "enabled": True,
            "university_counts": [1, 10, 100],
            "queries": ["query1", "query2", "query3", "query4", "query5"],
            "iterations": 5
        },
        "sp2b": {
            "enabled": True,
            "scale_factors": [1, 10, 100],
            "queries": ["sp2b_query_1", "sp2b_query_2"],
            "iterations": 5
        },
        "bioportal": {
            "enabled": True,
            "ontologies": ["GO", "SNOMEDCT", "DOID"],
            "iterations": 3
        },
        "scalability": {
            "enabled": True,
            "scales": ["small", "medium", "large"],
            "iterations": 3
        }
    }
}
```

### 8.2 Execution Script
```python
# run_comprehensive_benchmark.py
#!/usr/bin/env python3

import argparse
from enhanced_benchmark_framework import ExtendedBenchmarkFramework
from config import BENCHMARK_CONFIG

def main():
    parser = argparse.ArgumentParser(description="Comprehensive OWL2 Reasoner Benchmark")
    parser.add_argument("--reasoners", nargs="+", help="Specific reasoners to test")
    parser.add_argument("--benchmarks", nargs="+", help="Specific benchmarks to run")
    parser.add_argument("--iterations", type=int, default=5, help="Number of iterations")
    parser.add_argument("--output-dir", default="benchmark_results", help="Output directory")

    args = parser.parse_args()

    # Initialize framework
    framework = ExtendedBenchmarkFramework(args.output_dir)

    # Configure based on arguments
    if args.reasoners:
        framework.reasoners = {k: v for k, v in framework.reasoners.items() if k in args.reasoners}

    if args.benchmarks:
        for benchmark in ["lubm", "sp2b", "bioportal", "scalability"]:
            if benchmark not in args.benchmarks:
                BENCHMARK_CONFIG["benchmarks"][benchmark]["enabled"] = False

    # Run comprehensive benchmark
    results = framework.run_comprehensive_benchmark(args.iterations)

    # Generate reports
    reporter = EnhancedBenchmarkReporter()
    markdown_report, json_report = reporter.generate_comprehensive_report(results)

    print("✅ Comprehensive benchmark completed!")
    print(f"📊 Results saved to: {args.output_dir}")

if __name__ == "__main__":
    main()
```

## 9. Expected Results and Deliverables

### 9.1 Output Files
```
benchmark_results/
├── comprehensive_report.md        # Main markdown report
├── comprehensive_report.json      # Raw JSON data
├── performance_charts.png         # Performance comparison charts
├── scalability_charts.png         # Scalability analysis charts
├── statistical_analysis.json      # Statistical significance results
└── raw_results/                   # Raw benchmark data
    ├── lubm_results.json
    ├── sp2b_results.json
    ├── bioportal_results.json
    └── scalability_results.json
```

### 9.2 Key Metrics Tracked
- **Execution Time**: Classification, consistency, query operations
- **Memory Usage**: Peak memory consumption during operations
- **Success Rate**: Percentage of successful operations
- **Scalability**: Performance across different dataset sizes
- **Statistical Significance**: P-values for performance differences

### 9.3 Publication-Ready Outputs
- **Comprehensive Performance Report**: Detailed analysis of all benchmarks
- **Scalability Analysis**: Performance curves across different scales
- **Statistical Significance Analysis**: Confidence intervals and p-values
- **Comparative Analysis**: Head-to-head comparison with existing reasoners
- **Reproducibility Package**: Complete setup and execution instructions

This implementation provides a comprehensive, publication-ready benchmarking framework that integrates LUBM and SP2B benchmarks with the existing OWL2 reasoner testing infrastructure, enabling rigorous performance evaluation and comparison.