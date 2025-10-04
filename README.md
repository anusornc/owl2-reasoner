# OWL2 Reasoner

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rust-lang.org)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/anusornc/owl2-reasoner)
[![Tests](https://img.shields.io/badge/tests-314%20passing-brightgreen.svg)](https://github.com/anusornc/owl2-reasoner)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/anusornc/owl2-reasoner)

**A comprehensive OWL2 reasoning engine implemented in Rust with multi-format parsing support and advanced tableaux algorithms.**

## 🎯 Current Status

- ✅ **314/314 tests passing** (comprehensive test suite)
- ✅ **Multi-format parser support** (Turtle, RDF/XML, OWL/XML, N-Triples)
- ✅ **Advanced reasoning engine** with tableaux-based SROIQ(D) algorithm
- ✅ **EPCIS integration** for supply chain ontology processing
- ✅ **Production-ready codebase** with proper error handling

## 🏆 Key Features

### **Core OWL2 Capabilities**
- **Complete OWL2 entity support**: Classes, properties, individuals, annotations
- **Advanced axiom handling**: SubClassOf, EquivalentClasses, DisjointClasses, property characteristics
- **Tableaux reasoning engine**: Implementation of SROIQ(D) description logic
- **Multiple reasoning strategies**: Simple reasoner for basic operations, advanced tableaux for complex reasoning
- **Consistency checking**: Detect contradictions in ontologies
- **Classification**: Compute class hierarchies and entailments

### **Parser Suite**
- **Turtle parser**: Full support for RDF/Turtle format
- **RDF/XML parser**: Dual-mode with streaming (rio-xml) and legacy parsing
- **OWL/XML parser**: Support for OWL2 XML serialization
- **N-Triples parser**: Basic RDF triple format
- **JSON-LD parser**: JavaScript Object Notation for Linked Data format with context expansion
- **EPCIS parser**: GS1 EPCIS 2.0 standard support for supply chain ontologies
- **Blank node support**: Comprehensive anonymous individual handling across all formats

### **Performance Optimizations**
- **Memory-efficient design**: Arena-based allocation with automatic cleanup
- **Multi-threaded processing**: Rayon-based parallel tableaux reasoning
- **Multi-layered caching**: LRU eviction, hot data caching, and compressed storage
- **Profile-optimized reasoning**: Specialized algorithms for EL, QL, and RL profiles
- **Concurrent access**: DashMap-based thread-safe operations for caching and IRI management

## 🚀 Getting Started

### Installation

```bash
# Clone the repository
git clone https://github.com/anusornc/owl2-reasoner.git
cd owl2-reasoner

# Build the project
cargo build --release

# Run tests
cargo test

# Generate documentation
cargo doc --no-deps

# Build with optional features (if needed)
cargo build --features web-service
```

### Basic Usage

```rust
use owl2_reasoner::*;

// Create an ontology
let mut ontology = Ontology::new();

// Add classes
let person_class = Class::new("http://example.org/Person".to_string());
let animal_class = Class::new("http://example.org/Animal".to_string());
ontology.add_class(person_class.clone())?;
ontology.add_class(animal_class.clone())?;

// Add subclass relationship
let subclass_axiom = SubClassOfAxiom::new(
    ClassExpression::Class(person_class.clone()),
    ClassExpression::Class(animal_class.clone()),
);
ontology.add_subclass_axiom(subclass_axiom)?;

// Create reasoner and check consistency
let mut reasoner = SimpleReasoner::new(ontology);
let is_consistent = reasoner.is_consistent()?;
println!("Ontology is consistent: {}", is_consistent);
```

### Advanced Reasoning

```rust
use owl2_reasoner::reasoning::tableaux::{TableauxReasoner, ReasoningConfig};

// Configure advanced reasoning
let config = ReasoningConfig {
    max_depth: 1000,
    timeout: Some(30000),
    debug: false,
};

// Create advanced reasoner
let mut tableaux_reasoner = TableauxReasoner::with_config(&ontology, config);

// Perform advanced reasoning
let consistency_result = tableaux_reasoner.is_consistent()?;
let classification_result = tableaux_reasoner.classify()?;
```

### JSON-LD Integration

```rust
use owl2_reasoner::parser::JsonLdParser;

// Parse JSON-LD data
let parser = JsonLdParser::new();
let json_ld_content = r#"
{
    "@context": {
        "@vocab": "http://example.org/",
        "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
        "owl": "http://www.w3.org/2002/07/owl#"
    },
    "@graph": [
        {
            "@id": "Person",
            "@type": "owl:Class"
        },
        {
            "@id": "John",
            "@type": ["http://example.org/Person"],
            "http://example.org/name": "John Doe"
        }
    ]
}
"#;

let ontology = parser.parse_str(json_ld_content)?;

// Perform reasoning on JSON-LD data
let mut reasoner = SimpleReasoner::new(ontology);
let is_consistent = reasoner.is_consistent()?;
```

### EPCIS Integration

```rust
use owl2_reasoner::epcis_parser::*;

// Parse EPCIS data
let parser = EPCISDocumentParser::default();
let events = parser.parse_xml_str(epcis_xml_content)?;

// Convert to OWL2 ontology
let ontology = parser.to_ontology(&events)?;

// Perform reasoning on EPCIS data
let mut reasoner = SimpleReasoner::new(ontology);
let is_consistent = reasoner.is_consistent()?;
```

## 📊 Project Structure

```
owl2-reasoner/
├── src/
│   ├── iri.rs              # IRI management and caching
│   ├── entities.rs         # OWL2 entities (classes, properties, individuals)
│   ├── axioms/             # OWL2 axioms and logical statements
│   ├── ontology.rs         # Ontology structure and management
│   ├── reasoning/          # Reasoning algorithms (tableaux, rules, query)
│   ├── parser/             # Multi-format OWL2 parsers (Turtle, RDF/XML, OWL/XML, N-Triples, JSON-LD)
│   ├── epcis_parser.rs     # EPCIS document processing
│   ├── python_bindings.rs  # Python interface (PyO3 - add to dependencies for Python support)
│   └── web_service.rs      # REST API interface (optional feature)
├── examples/               # Usage examples and demonstrations
├── benches/               # Performance benchmarks
├── tests/                 # Comprehensive test suite
└── docs/                  # Documentation
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test reasoning
cargo test parser
cargo test json_ld
cargo test epcis

# Run tests with release mode
cargo test --release
```

### Test Coverage
- **314 comprehensive tests** covering all major functionality
- **Parser validation** across all supported formats
- **Reasoning correctness** with known ontologies
- **Error handling** and edge cases
- **Performance regression** prevention
- **Memory safety** and concurrency testing

## 📈 Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmarks
cargo bench --bench parser_bench
cargo bench --bench reasoning_bench
cargo bench --bench query_bench

# Run benchmarks without execution (build check)
cargo bench --no-run
```

### Available Benchmarks
- **Parser performance**: Multi-format parsing speed
- **Reasoning performance**: Tableaux algorithm efficiency
- **Query performance**: Pattern matching and lookup
- **Memory usage**: Allocation and caching efficiency
- **Scalability**: Large ontology handling

## 🔧 Features

### OWL2 Profiles
- **EL Profile**: Optimized for large, simple ontologies
- **QL Profile**: Query answering with tractable reasoning
- **RL Profile**: Rule-based reasoning with limited expressivity
- **Full OWL2**: Complete SROIQ(D) description logic

### Advanced Features
- **Blank node handling**: Anonymous individuals and complex graph patterns
- **Cardinality restrictions**: Min, max, and exact cardinality axioms
- **Property chains**: Complex property relationships
- **Nominal reasoning**: Individual equality and inequality
- **Dependency-directed backtracking**: Smart conflict resolution

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

### Project Scripts

- `./scripts/update_docs.sh` - Update documentation (builds Rustdoc, tests examples, builds mdbook)
- `./scripts/validate_system.sh` - Comprehensive system validation
- `./scripts/run_benchmarks.sh` - Execute benchmark suite
- `./scripts/run_validation.sh` - Run validation tests
- `./scripts/build-technical-docs.sh` - Build technical documentation (requires Typst)
- `./scripts/analyze_tableaux_performance.rs` - Analyze tableaux reasoning performance

## 📚 Documentation

### Documentation Structure

- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation
- **[Reports](docs/reports/)** - Analysis reports and status summaries
  - [Code Analysis Report](docs/reports/CODE_ANALYSIS_REPORT.md)
  - [Production Readiness](docs/reports/PRODUCTION_READINESS_SUMMARY.md)
  - [Memory Safety Implementation](docs/reports/MEMORY_SAFETY_IMPLEMENTATION_SUMMARY.md)
- **[Plans](docs/plans/)** - Strategic planning documents
  - [Modularization Strategy](docs/plans/MODULARIZATION_STRATEGY.md)
- **[Project Management](docs/project/)** - Project-related documentation
  - [Agents Configuration](docs/project/AGENTS.md)
  - [Development TODOs](docs/project/TODOS.md)
- **[User Guides](examples/)** - Step-by-step tutorials and examples
- **[Technical Documentation](docs/technical-documentation/)** - Architecture and algorithms
- **[Performance Analysis](docs/BENCHMARKING.md)** - Benchmarking results and optimization
- **[EPCIS Integration](docs/EPCIS_ECOSYSTEM_INTEGRATION.md)** - Supply chain ontology processing

### Generated Documentation
- **API Reference**: Generated Rustdoc (`cargo doc --open`)
- **Memory Safe Testing**: [Testing Guidelines](docs/MEMORY_SAFE_TESTING.md)

## 🤝 Contributing

We welcome contributions that improve:

### High Priority
- **Parser robustness**: Edge case handling and error recovery
- **Performance optimization**: Memory usage and computation speed
- **Test coverage**: Additional test cases and validation
- **Documentation**: Examples, tutorials, and API documentation

### Development Setup
```bash
# Install required tools
rustup component add clippy rustfmt

# Verify code quality
cargo clippy -- -D warnings
cargo fmt --check

# Run comprehensive tests
cargo test --release

# Build and view documentation
cargo doc --no-deps --open
```

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
- Contributors to the semantic web and knowledge representation communities
- Open source reasoner developers for their pioneering work

## 📞 Contact

- **Project Lead**: Anusorn Chaikaew
- **Issues**: [GitHub Issues](https://github.com/anusornc/owl2-reasoner/issues)
- **Source Code**: [GitHub Repository](https://github.com/anusornc/owl2-reasoner)
- **Documentation**: [API Documentation](https://docs.rs/owl2-reasoner/)

---

**Built with ❤️ in Rust for the Future of Semantic Web**

*A high-performance, memory-safe OWL2 reasoning engine that brings semantic web capabilities to native applications.*