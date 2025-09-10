# Introduction

Welcome to the **OWL2 Reasoner** - a high-performance, feature-complete OWL2 reasoning engine implemented in Rust.

## What is OWL2?

OWL2 (Web Ontology Language 2) is a W3C standard for representing rich and complex knowledge about things, groups of things, and relations between things. It provides powerful reasoning capabilities that allow machines to understand and infer new knowledge from existing facts.

## Features

### 🚀 Performance
- **Lightning-fast reasoning** with Rust's zero-cost abstractions
- **Optimized data structures** for memory efficiency
- **Parallel processing** capabilities for large ontologies
- **Intelligent caching** with TTL-based expiration

### 🎯 OWL2 Compliance
- **Complete OWL2 DL profile** support
- **SROIQ(D) description logic** foundation
- **Tableaux-based reasoning algorithm**
- **Rule-based inference system**

### 🔧 Developer Experience
- **Type-safe API** leveraging Rust's type system
- **Comprehensive error handling** with detailed diagnostics
- **Extensible architecture** for custom reasoners
- **Rich documentation** and examples

### 📊 Query Capabilities
- **SPARQL-like query engine** for knowledge retrieval
- **Complex pattern matching** with optional and union patterns
- **Filter expressions** and value constraints
- **Hash join optimization** for performance

## Quick Start

```rust
use owl2_reasoner::{Ontology, IRI, SimpleReasoner};

// Create a new ontology
let mut ontology = Ontology::new();

// Add a class
let person_class = Class::new("http://example.org/Person");
ontology.add_class(person_class)?;

// Add a subclass relationship
let animal_class = Class::new("http://example.org/Animal");
let subclass_axiom = SubClassOfAxiom::new(
    ClassExpression::from(person_class.clone()),
    ClassExpression::from(animal_class),
);
ontology.add_subclass_axiom(subclass_axiom)?;

// Create a reasoner and perform inference
let reasoner = SimpleReasoner::new(ontology);
let is_consistent = reasoner.is_consistent()?;
let is_subclass = reasoner.is_subclass_of(&person_class, &animal_class)?;
```

## Architecture

The OWL2 Reasoner is built with a modular architecture:

```
┌─────────────────────────────────────────────────┐
│                Application Layer                │
├─────────────────────────────────────────────────┤
│              Query Engine                       │
│              Reasoning Engine                   │
├─────────────────────────────────────────────────┤
│              Ontology Management                │
│              Parser Interface                    │
├─────────────────────────────────────────────────┤
│              Core Types & Entities              │
└─────────────────────────────────────────────────┘
```

## Key Components

### Core Types
- **IRI Management**: Efficient IRI handling with caching
- **Entities**: Classes, properties, and individuals
- **Axioms**: Logical statements and relationships

### Ontology Management
- **Indexed Storage**: O(1) access to axioms and entities
- **Memory Management**: Arc-based sharing for efficiency
- **Import Support**: Multi-ontology reasoning

### Reasoning Engine
- **Tableaux Algorithm**: Complete and sound reasoning
- **Rule-Based Inference**: Forward and backward chaining
- **Caching System**: TTL-based result caching

### Query Engine
- **Pattern Matching**: Complex graph pattern queries
- **Join Optimization**: Hash-based join algorithms
- **Filter Expressions**: Value-based filtering

## Performance Characteristics

The OWL2 Reasoner is designed for high performance:

- **Memory Efficiency**: ~10MB base footprint + ontology size
- **Reasoning Speed**: Sub-millisecond consistency checks
- **Query Performance**: Millisecond-scale complex queries
- **Scalability**: Tested with ontologies up to 100K axioms

## Getting Help

- **Documentation**: Browse this book for comprehensive guides
- **API Reference**: Check the Rustdoc for detailed API documentation
- **Examples**: Explore the examples directory for real-world usage
- **Issues**: Report bugs or request features on GitHub

## Contributing

We welcome contributions! Please see the [Contributing Guide](developer/contributing.md) for details on how to get involved.

---

**Next**: [Getting Started](getting-started.md)