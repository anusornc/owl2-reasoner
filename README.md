# OWL2 Reasoner

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rust-lang.org)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/anusornc/owl2-reasoner)

A foundational OWL2 reasoning engine implemented in Rust, focused on educational use and small to medium knowledge graph applications.

## 🎯 Project Overview

This project provides a basic implementation of OWL2 reasoning capabilities with:

- **Core OWL2 ontology representation** with IRI management and caching
- **Multi-format parser support** (Turtle, RDF/XML, OWL/XML, N-Triples)
- **Basic reasoning engine** with simple consistency checking and classification
- **Memory profiling tools** for performance analysis
- **Performance measurement framework** with honest, empirical results

## 🏗️ Architecture

### Core Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Parser Module │    │  Ontology Core  │    │ Reasoning Engine│
│                 │    │                 │    │                 │
│ • Turtle ✓      │───▶│ • Entity Store  │───▶│ • SimpleReasoner│
│ • N-Triples ✓   │    │ • Axiom Index   │    │ • Basic Caching │
│ • RDF/XML ⚠     │    │ • IRI Cache     │    │ • Simple Logic  │
│ • OWL/XML ⚠     │    │ • Basic Storage │    │ • Basic Rules   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                │
                    ┌─────────────────┐
                    │  Performance    │
                    │                 │
                    │ • Memory Profiler│
                    │ • Benchmarks    │
                    │ • Measurement   │
                    └─────────────────┘
```

### Parser Support Status

- **✅ Working**: Turtle, N-Triples
- **⚠️ Limited**: RDF/XML, OWL/XML (basic support, may have issues)

## 📊 Performance Measurement

The project provides honest performance measurement tools based on actual testing:

### Measured Performance Characteristics
- **Response times**: 0.024-55.3ms (depends on ontology size and complexity)
- **Memory usage**: 150-390 bytes per entity (conservative estimates)
- **Reasoning speed**: ~77,000 subclass checks per second
- **Scale testing**: Tested up to 5,000 entities with linear scaling

### Available Performance Tools
```bash
# Basic performance measurement
cargo run --example complete_validation

# Scale testing (100-5000 entities)
cargo run --example scale_test_simple

# Real-world ontology testing
cargo run --example real_world_test

# Complex axiom structure testing
cargo run --example complex_axiom_test

# Enhanced memory profiling
cargo run --example enhanced_memory_profiling

# Comparative analysis
cargo run --example comparative_analysis
```

### Performance Notes
- **Honest measurements**: All results are from actual implementation testing
- **Conservative estimates**: Memory usage includes safety margins
- **Realistic scope**: Tested with small to medium ontologies only
- **No guarantees**: Performance may vary with different use cases
- **Educational focus**: Designed for learning, not production use

## 🧪 Testing

### Test Coverage
- **Unit Tests**: Core component validation
- **Integration Tests**: End-to-end workflows
- **Parser Tests**: Multi-format compatibility
- **Reasoning Tests**: Logic validation

### Running Tests
```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test categories
cargo test unit_tests
cargo test integration_tests
```

## 🛠️ Usage

### Basic Example

```rust
use owl2_reasoner::*;

// Create a simple ontology
let mut ontology = Ontology::new();

// Add classes
let person_class = Class::new("http://example.org/Person");
ontology.add_class(person_class)?;

let student_class = Class::new("http://example.org/Student");
ontology.add_class(student_class)?;

// Add subclass relationship
let sub_class = ClassExpression::Class(Class::new("http://example.org/Student")?);
let super_class = ClassExpression::Class(Class::new("http://example.org/Person")?);
let subclass_axiom = SubClassOfAxiom::new(sub_class, super_class);
ontology.add_subclass_axiom(subclass_axiom)?;

// Initialize reasoner
let reasoner = SimpleReasoner::new(ontology);

// Check consistency
let is_consistent = reasoner.is_consistent()?;
println!("Ontology is consistent: {}", is_consistent);

// Perform subclass reasoning
let student_iri = IRI::new("http://example.org/Student")?;
let person_iri = IRI::new("http://example.org/Person")?;
let is_subclass = reasoner.is_subclass_of(&student_iri, &person_iri)?;
println!("Student is subclass of Person: {}", is_subclass);
```

### Parsing OWL2 Files

```rust
use owl2_reasoner::parser::TurtleParser;

// Parse Turtle format
let parser = TurtleParser::new();
let ontology = parser.parse_file("ontology.ttl")?;

// Use with reasoner
let reasoner = SimpleReasoner::new(ontology);
```

### Performance Measurement

```rust
use owl2_reasoner::validation::memory_profiler::EntitySizeCalculator;

// Measure entity sizes
let class = Class::new("http://example.org/Class");
let size = EntitySizeCalculator::estimate_class_size(&class);
println!("Estimated class size: {} bytes", size);

// Use reasoner with cache statistics
let reasoner = SimpleReasoner::new(ontology);
reasoner.warm_up_caches()?;
let stats = reasoner.get_cache_stats();
println!("Cache hit rate: {:.1}%", stats.hit_rate() * 100.0);
```

## 🔬 Features

### Core OWL2 Support
- **Classes and Properties**: Basic entity representation
- **Axioms**: Subclass, equivalence, disjointness relationships
- **Individuals**: Instance-level reasoning
- **IRI Management**: Efficient internationalized resource identifier handling

### Reasoning Capabilities
- **Consistency Checking**: Basic ontology consistency validation
- **Classification**: Subclass reasoning with transitive closure
- **Satisfiability**: Basic class satisfiability checking
- **Instance Retrieval**: Get instances of classes

### Memory Management
- **IRI Interning**: Automatic deduplication of resource identifiers
- **Arc-based Sharing**: Memory-efficient entity sharing
- **Cache System**: Multi-layered caching for reasoning results

### Performance Tools
- **Entity Size Calculator**: Conservative memory estimation
- **Memory Profiler**: Basic memory usage analysis
- **Benchmarking**: Performance measurement framework

## 🚀 Getting Started

### Prerequisites
- Rust 1.70+
- Cargo (Rust package manager)

### Installation
```bash
git clone https://github.com/yourusername/owl2-reasoner.git
cd owl2-reasoner
cargo build --release
```

### Running Examples
```bash
# Basic usage example
cargo run --example simple_example

# Performance measurement
cargo run --example complete_validation

# Biomedical ontology example
cargo run --example biomedical_ontology
```

## 📚 Documentation

- **API Documentation**: `cargo doc --open`
- **Examples**: See `examples/` directory
- **Benchmarks**: See `benches/` directory

## 🔬 Research Context

This project serves as a foundation for exploring:

- **Rust for Semantic Web**: Applying modern systems programming to semantic web technologies
- **Memory-Efficient Reasoning**: Investigating optimal data structures for OWL2 processing
- **Performance Measurement**: Developing honest performance evaluation methodologies
- **Type-Safe Ontologies**: Leveraging Rust's type system for semantic web correctness

### Current Limitations
- **Tableaux Algorithm**: Basic implementation, not optimized for complex ontologies
- **Rule Engine**: Foundational rules only
- **SPARQL Query**: Basic pattern matching, not full SPARQL 1.1 compliance
- **Profile Support**: Basic OWL2 profile validation
- **Performance**: Suitable for small to medium ontologies

## 🤝 Contributing

We welcome contributions focused on:

- **Core Reasoning**: Improving tableaux algorithm and rule engine
- **Performance**: Optimizing memory usage and reasoning speed
- **Standards Compliance**: Better OWL2 specification coverage
- **Documentation**: Improving examples and API documentation

### Development Setup
```bash
# Install development tools
rustup component add clippy rustfmt

# Run code quality checks
cargo clippy -- -D warnings
cargo fmt --check

# Build documentation
cargo doc --no-deps
```

## 📄 License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- The W3C OWL2 Working Group for the OWL2 specification
- The Rust community for excellent tooling and libraries
- Research contributions from semantic web and knowledge representation communities

## 📞 Contact

- **Project Lead**: Anusorn Chaikaew
- **Issues**: [GitHub Issues](https://github.com/anusornc/owl2-reasoner/issues)

---

**Built with ❤️ in Rust for the Semantic Web**