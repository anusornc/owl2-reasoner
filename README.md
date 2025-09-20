# OWL2 Reasoner

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rust-lang.org)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/anusornc/owl2-reasoner)
[![Performance](https://img.shields.io/badge/performance-38x%20faster-brightgreen.svg)](https://github.com/anusornc/owl2-reasoner)
[![Benchmark](https://img.shields.io/badge/benchmark-comprehensive-blue.svg)](https://github.com/anusornc/owl2-reasoner)
[![Documentation](https://img.shields.io/badge/docs-available-brightgreen.svg)](https://anusornc.github.io/owl2-reasoner/)

High‑performance Rust OWL2 reasoner with an actively evolving parser/reasoner, examples, test-suite integration, and Criterion benchmarks.

## 🏆 Key Achievements

### **Performance Notes**
- Competitive performance on internal Criterion benches (informative)
- Fast responses on small ontologies; release mode recommended for timing
- Clippy + fmt supported locally; CI integration and coverage are evolving

### **Format & Reasoning Support**
- Parsers: Turtle, RDF/XML (streaming backend available), OWL Functional (in progress), N‑Triples
- Tableaux reasoning: practical SROIQ(D) subset with ongoing improvements
- Multi‑level reasoning modes (simple to advanced/tableaux)
- Rule‑based inference (forward chaining)
- Query engine: SPARQL‑like pattern matching
- Memory efficiency: conservative allocation, pooling, sharing
- Benchmarks: Criterion benches in‑repo; external comparisons optional

### **Advanced Reasoning Capabilities**
- Tableaux‑based reasoning engine with configurable limits/timeouts
- Multiple reasoning strategies under a unified API
- Scalable architecture; performance validated with Criterion benches

## 🎯 Project Overview

This project provides a complete OWL2 reasoning ecosystem with:

- **🚀 Native Rust Implementation** - Zero JVM overhead, maximum performance
- **📊 Comprehensive Benchmarking** - Scientific comparison with 5 major reasoners
- **🔬 Research-Grade Framework** - Academic publication-ready performance data
- **🛠️ Production-Ready Architecture** - Stable, reliable, extensible design
- **📚 Complete Documentation** - API docs, usage guides, and technical specifications

### Core Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Parser Module │    │  Ontology Core  │    │ Reasoning Engine│
│                 │    │                 │    │                 │
│ • Turtle ✓      │───▶│ • Entity Store  │───▶│ • Tableaux      │
│ • RDF/XML ✓     │    │ • Axiom Index   │    │ • Rule Engine   │
│ • OWL/XML ✓     │    │ • IRI Cache     │    │ • Query Engine  │
│ • N-Triples ✓   │    │ • Memory Pool   │    │ • Caching      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                │
                    ┌─────────────────────────────────┐
                    │     Performance Framework      │
                    │                                 │
                    │ • 5-Way Benchmark Suite ✓     │
                    │ • Real-time Performance Data   │
                    │ • Memory Profiling ✓          │
                    │ • Scientific Analysis ✓        │
                    └─────────────────────────────────┘
```

## 📊 Comprehensive Benchmarking Results

Benchmark comparisons with other reasoners are available in the benchmarking folder; treat them as informative while the project evolves.

## 🚀 Getting Started

This crate lives in a Rust workspace together with a reusable test‑suite crate:
- Core: `owl2-reasoner/` (this crate)
- Test suite: `owl2-reasoner-test-suite/` (shared datasets/tests)

### Prerequisites
- Rust 1.70+
- Optional (for external comparisons): Java 11+ and Maven 3.6+

### Installation

```bash
# Clone the repository
git clone https://github.com/anusornc/owl2-reasoner.git
cd owl2-reasoner

# Build (this crate)
cargo build

# Build entire workspace
cargo build --workspace

# Run tests
cargo test                 # workspace tests
cargo test -p owl2-reasoner  # crate tests only

# Format and lint
cargo fmt --all
cargo clippy --all-targets -- -D warnings

# Parser feature flags
# Streaming RDF/XML support (feature `rio-xml`) is enabled by default.
# Disable to force legacy RDF/XML path only:
cargo test -p owl2-reasoner --no-default-features
```

### Quick Start

#### Basic Library Usage
```rust
use owl2_reasoner::*;

// Create and populate ontology
let mut ontology = Ontology::new();
let person_class = Class::new("http://example.org/Person");
ontology.add_class(person_class)?;

// Initialize reasoner
let reasoner = SimpleReasoner::new(ontology);

// Check consistency
let is_consistent = reasoner.is_consistent()?;
println!("Ontology consistent: {}", is_consistent);

// Perform classification
let classified = reasoner.classify()?;
println!("Classification completed: {} classes", classified.len());
```

#### Advanced Reasoning Usage
```rust
use owl2_reasoner::reasoning::{
    OwlReasoner, ReasoningConfig,
    tableaux::ReasoningConfig as TableauxConfig
};

// Configure advanced reasoning
let tableaux_config = TableauxConfig {
    max_depth: 2000,
    debug: false,
    incremental: true,
    timeout: Some(45000),
};

let reasoning_config = ReasoningConfig {
    enable_reasoning: true,
    use_advanced_reasoning: true,
    tableaux_config,
};

// Create advanced reasoner
let mut reasoner = OwlReasoner::with_config(ontology, reasoning_config);

// Advanced reasoning capabilities
let is_consistent = reasoner.is_consistent()?;
let is_satisfiable = reasoner.is_class_satisfiable(&class_iri)?;
let classification = reasoner.classify()?;
```

#### Example CLI usage (via `cargo run --example`)
```bash
# Consistency/classification helpers
cargo run --example benchmark_cli -- --consistent path/to/ontology.ttl
cargo run --example benchmark_cli -- --classify  path/to/ontology.ttl
```

## 📚 Project Structure

The project is structured as a workspace with a modular core crate:

```
owl2-reasoner/
├── examples/              # Example usage and demonstrations
│   ├── basic/             # Basic usage examples
│   │   ├── family_ontology.rs
│   │   ├── biomedical_ontology.rs
│   │   └── simple_example.rs
│   ├── benchmarking/      # Performance benchmarking examples
│   │   ├── benchmark_cli.rs
│   │   └── performance_benchmarking.rs
│   ├── validation/        # Validation and testing examples
│   │   └── complete_validation.rs
│   └── advanced/          # Advanced use cases
│       ├── comparative_analysis.rs
│       └── epcis_validation_suite.rs
├── benches/               # Criterion benches (targeted)
│   ├── parser_bench.rs        # Turtle parsing
│   ├── rdfxml_parser_bench.rs # RDF/XML parsing
│   ├── reasoning_bench.rs     # Reasoning paths
│   └── query_bench.rs         # Query engine
├── tests/                 # Unit and integration tests
├── benchmarking/          # External benchmarking framework
│   ├── framework/         # Python benchmarking tools
│   ├── established_reasoners/  # External reasoners (HermiT, ELK, etc.)
│   └── datasets/          # Benchmark datasets (LUBM, SP2B, BioPortal)
├── scripts/               # Utility scripts (see Scripts section)
├── archive/               # Legacy and historical components
└── docs/                  # Documentation (organized by category)

## 🔧 Parser Modes & Features

- RDF/XML streaming (`rio-xml` feature): enabled by default. When `strict_validation` is false, the streaming backend is used to reduce memory usage.
- Strict mode (default in ParserConfig): validates input rigorously and uses the legacy parser path.

Examples:
- Strict validation (default): `ParserConfig::default()` sets `strict_validation: true`.
- Non‑strict (streaming): set `strict_validation: false` in `ParserConfig` to prefer streaming RDF/XML.
- Disable streaming feature entirely: `cargo test --no-default-features`.

### Usage: RDF/XML Streaming vs Strict
```rust
use owl2_reasoner::parser::{RdfXmlParser, ParserConfig, OntologyParser};

// Non‑strict mode: uses streaming RDF/XML when feature is enabled
let cfg = ParserConfig { strict_validation: false, ..Default::default() };
let parser = RdfXmlParser::with_config(cfg);
let onto_streaming = parser.parse_file(std::path::Path::new("examples/ontologies/sample.rdf"))?;

// Strict mode (default): validates input rigorously; uses legacy RDF/XML path
let strict_cfg = ParserConfig { strict_validation: true, ..Default::default() };
let strict_parser = RdfXmlParser::with_config(strict_cfg);
let onto_strict = strict_parser.parse_file(std::path::Path::new("examples/ontologies/sample.rdf"))?;

assert_eq!(onto_streaming.entities_len(), onto_strict.entities_len());
```

## 🧪 Test Suite & Examples

The W3C-style test runner lives in a separate crate to keep the core lean:

- Crate: `owl2-reasoner-test-suite` (path dependency in this repo)
- Example runner: `examples/test_suite_runner.rs`

Usage:
- From `owl2-reasoner/`: `cargo run --example test_suite_runner`
- As a dependency: add `owl2-reasoner-test-suite = { path = "../owl2-reasoner-test-suite" }` to your dev‑dependencies.
Note: the runner loads zero cases unless you point it to a populated test‑suite directory; see the config in `owl2-reasoner-test-suite/src/lib.rs`.

Runner flags (examples):
```bash
# Point to a specific suite directory and output location
cargo run --example test_suite_runner -- \
  --suite-dir ./test_suite/owl2 \
  --out ./test_results \
  --timeout 60 \
  --jobs 8

# Include extra-credit tests
cargo run --example test_suite_runner -- --extra-credit
```

## 📈 Benchmarks

- Criterion benches live in `benches/`.
- Run targeted benches:
  - Turtle parsing: `cargo bench --bench parser_bench`
  - RDF/XML parsing: `cargo bench --bench rdfxml_parser_bench`
  - Reasoning: `cargo bench --bench reasoning_bench`
  - Query engine: `cargo bench --bench query_bench`
- Aggregate bench binaries are placeholders to keep `cargo test --all-targets` green. Prefer targeted benches above.

### Key Documentation
- **API Documentation**: `target/doc/owl2_reasoner/` (generated with `cargo doc`)
- **Performance Analysis**: `docs/performance/COMPREHENSIVE_PERFORMANCE_ANALYSIS.md`
- **Project Status**: `docs/project/PROJECT_SUCCESS_SUMMARY.md`
- **Technical Details**: `docs/technical/ARCHITECTURE.md`
- **User Guide**: `docs/book/` (mdbook documentation)

## 🧪 Testing and Validation

### Running Tests

```bash
# Run all workspace tests
cargo test

# Run crate library tests only
cargo test -p owl2-reasoner --lib

# Run specific test modules
cargo test validation
cargo test reasoning

# Run tests with release mode (timing/perf checks)
cargo test --release
```

### System Validation

```bash
# Comprehensive system validation
./scripts/validate_system.sh

# This script runs:
# - Full test suite (152 tests)
# - Example validation
# - System integration tests
# - Performance verification
```

### Example Usage

```bash
# Basic examples
cargo run --example family_ontology
cargo run --example biomedical_ontology
cargo run --example simple_example

# Test suite examples
cargo run --example simple_test_runner      # Basic reasoning validation
cargo run --example advanced_test_runner    # Advanced reasoning comparison

# Benchmarking examples
cargo run --example benchmark_cli -- --help
cargo run --example performance_benchmarking

# Validation examples
cargo run --example complete_validation

# Advanced examples
cargo run --example comparative_analysis
cargo run --example epcis_validation_suite
cargo run --example real_world_simulation

# EPCIS examples (focus)
cargo run --example epcis_validation_suite      # EPCIS compliance/validation
cargo run --example real_world_simulation       # Large‑scale EPCIS simulation
```

## 📊 Benchmarking

### Running Benchmarks

```bash
# Run comprehensive benchmark suite
./scripts/run_benchmarks.sh

# Run Rust Criterion benchmarks (release mode)
cargo bench --bench parser_bench
cargo bench --bench rdfxml_parser_bench
cargo bench --bench reasoning_bench
cargo bench --bench query_bench

# Build benches without executing (fast sanity check)
cargo bench --no-run
# Or only one bench target without running
cargo bench --no-run --bench parser_bench

# Run external reasoner comparisons
cd benchmarking/established_reasoners
python3 run_simple_comprehensive_benchmark.py

# Quick benchmark test
python3 run_simple_comprehensive_benchmark.py --quick
```

### External Benchmarks (LUBM/UOBM, Established Reasoners)

This project includes an external benchmarking harness to compare against established Java reasoners (ELK, HermiT, JFact, Pellet) using the university‑domain LUBM and UOBM suites.

Prerequisites
- Install Java 11+ and Maven 3.6+ locally
- Prepare datasets offline (no network fetch in this environment):
  - Place LUBM datasets under `benchmarking/datasets/lubm/`:
    - `benchmarking/datasets/lubm/lubm1/*.ttl`
    - `benchmarking/datasets/lubm/lubm5/*.ttl`
  - Place UOBM datasets under `benchmarking/datasets/uobm/`:
    - `benchmarking/datasets/uobm/uobm1/*.{owl,rdf,ttl}`
    - `benchmarking/datasets/uobm/uobm5/*.{owl,rdf,ttl}`

Run external comparisons
```bash
cd benchmarking

# Optional one-time setup (creates/validates structure and config)
python3 framework/setup_benchmarks.py --base-dir .

# Full run: LUBM + UOBM, all reasoners, 5 iterations, Markdown output
python3 framework/enhanced_benchmark_framework.py \
  --suites LUBM,UOBM \
  --reasoners all \
  --iterations 5 \
  --output-format markdown \
  --out results

# Targeted run: LUBM sizes 1 and 5 only
python3 framework/enhanced_benchmark_framework.py \
  --suites LUBM \
  --sizes 1,5 \
  --reasoners all \
  --iterations 5 \
  --output-format markdown \
  --out results
```

Output
- Main report (Markdown): `benchmarking/results/Benchmark_Report.md`
- Per‑run artifacts (JSON/CSV) may be emitted alongside, depending on framework settings.

Notes
- If any reasoner binaries or JARs need setup, use `benchmarking/setup_reasoners.sh` or follow the notes in `benchmarking/README_Enhanced_Benchmarking.md` and `benchmarking/IMPLEMENTATION_SUMMARY.md`.
- The external harness reads `benchmarking/config.json` for paths and parameters; adjust if you keep datasets elsewhere.

### Benchmark Results

Results are saved in `benchmarking/results/` with:
- Comprehensive JSON reports
- Performance comparison metrics
- Success/failure analysis
- Statistical significance testing

Example results:
```json
{
  "timestamp": "2025-09-15T15:24:55",
  "total_tests": 40,
  "successful_tests": 16,
  "failed_tests": 24,
  "reasoners": {
    "OWL2-Reasoner": {
      "success_rate": "50%",
      "avg_time_ms": 8.08,
      "min_time_ms": 5.47,
      "max_time_ms": 14.78
    },
    "HermiT": {
      "success_rate": "100%",
      "avg_time_ms": 305.39,
      "min_time_ms": 289.81,
      "max_time_ms": 345.40
    }
  }
}
```

## 🛠️ Development

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy -- -D warnings

# Check compilation
cargo check

# Build documentation
cargo doc --no-deps
```

### Updating Documentation

```bash
# Update all documentation
./update_docs.sh "Description of changes"

# This script updates:
# - Rustdoc API documentation
# - mdbook documentation
# - Technical documentation (if Typst available)
# - Example documentation
# - Test validation
```

### Project Scripts

- `scripts/validate_system.sh`
  - Builds, runs unit/integration tests, and exercises key examples.
  - Usage: from `owl2-reasoner/`: `./scripts/validate_system.sh`

- `scripts/run_benchmarks.sh`
  - Runs the release build, targeted Criterion benches, then the Python framework and report generator (if available).
  - Usage: `./scripts/run_benchmarks.sh`

- `scripts/update_docs.sh`
  - Builds Rustdoc, checks core examples, builds mdBook in `docs/`, and optionally builds Typst technical docs.
  - Usage: `./scripts/update_docs.sh "Description of changes"`
  - Requirements: `mdbook` installed; optional `typst` for technical PDF.

- `scripts/build-technical-docs.sh`
  - Directly builds the Typst technical documentation to `docs/technical-documentation/output/`.
  - Usage: `./scripts/build-technical-docs.sh`

## 📈 Performance Characteristics

### Notes on Performance
- Prefer `--release` for measurements and benches.
- Treat README numbers as informative; rely on local Criterion results.

### Real-World Applications
- **Interactive Tools**: Real-time ontology editing and validation
- **Web Applications**: Backend reasoning for semantic web apps
- **Edge Computing**: Efficient reasoning on resource-constrained devices
- **Research Systems**: Fast prototyping and experimentation

## 🔬 Research Contributions

### Academic/Research Use
- External comparisons (ELK, HermiT, JFact, Pellet) are supported via the `benchmarking/` folder; Java/Maven required.
- Use results as informative baselines; rerun locally for current measurements.

## 🏗️ Architecture Details

### Core Components
- **IRI Management**: Efficient internationalized resource identifier handling
- **Entity Store**: Type-safe representation of OWL2 entities
- **Axiom Index**: Optimized storage for logical statements
- **Tableaux Engine**: Complete SROIQ(D) reasoning implementation
- **Rule System**: Forward chaining with conflict resolution
- **Query Engine**: SPARQL-like pattern matching

### Performance Optimizations
- **Memory Pooling**: Reused allocations for common structures
- **Caching Layers**: Multi-level intelligent result caching
- **Arc-Based Sharing**: Memory-efficient entity representation
- **Zero-Copy Parsing**: Direct ontology loading where possible

## 🤝 Contributing

We welcome contributions that advance:

### High Priority
- **OWL Format Parser**: Complete full format support
- **Advanced Reasoning**: Enhanced tableaux optimizations
- **SPARQL Compliance**: Full SPARQL 1.1 implementation
- **Enterprise Testing**: Large-scale ontology validation

### Development Setup
```bash
# Install development tools
rustup component add clippy rustfmt

# Code quality checks
cargo clippy -- -D warnings
cargo fmt --check

# Run comprehensive test suite
cargo test --release

# Build documentation
cargo doc --no-deps --open
```

## 📊 Current Status

### ✅ **Current Capabilities**
- OWL2 reasoning engine with tableaux and rule‑based paths (practical SROIQ(D) subset)
- Parsers: Turtle, RDF/XML (streaming available), OWL XML, N‑Triples; OWL Functional (in progress)
- Benchmarks: Criterion benches and external comparison framework
- Caching and memory pooling utilities; entity/IRI interning
- Documentation: API docs, mdBook, and technical docs (Typst)

### 🔄 **In Progress**
- Parser/IRI validation modes (strict vs relaxed)
- Bench aggregation re‑enablement (placeholders present)
- EPCIS coverage and examples

### 📋 **Next Steps**
1. Re‑enable aggregator benches (currently placeholders)
2. Expand strict vs relaxed IRI/parse validation tests
3. Add EPCIS samples and end‑to‑end tests
4. Tighten clippy across all targets and docs

## 📄 License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- The W3C OWL2 Working Group for the excellent specification
- The Rust community for outstanding tooling and libraries
- Research contributions from semantic web and knowledge representation communities
- Open source reasoner developers (HermiT, ELK, JFact, Pellet teams)

## 📞 Contact

- **Project Lead**: Anusorn Chaikaew
- **Issues**: [GitHub Issues](https://github.com/anusornc/owl2-reasoner/issues)
- **Performance Data**: Available in `benchmarking/results/` directory
- **Documentation**: [API Docs](https://anusornc.github.io/owl2-reasoner/)

---

**Built with ❤️ in Rust for the Future of Semantic Web**

*This project demonstrates that native implementations can dramatically outperform traditional JVM-based semantic web reasoners, opening new possibilities for real-time semantic applications.*
