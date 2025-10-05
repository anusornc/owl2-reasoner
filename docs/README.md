# OWL2 Reasoner Documentation

Welcome to the comprehensive documentation for the OWL2 Reasoner project, featuring advanced memory safety capabilities and organized project structure.

## 📚 Documentation Structure

### **🛡️ Memory Safety & Testing**
- [Memory-Safe Testing Guide](MEMORY_SAFE_TESTING.md) - Comprehensive testing guidelines
- [Memory Safety Implementation](reports/MEMORY_SAFETY_IMPLEMENTATION_SUMMARY.md) - Implementation details
- [Memory Safety Benchmarks](benches/memory_safety_benchmarks.rs) - Performance impact analysis

### **📖 mdBook Documentation**
- [Interactive Guide](src/SUMMARY.md) - Complete book-style documentation
  - [Memory Safety Implementation](src/memory-safety-implementation.md) - Deep dive into memory safety
  - [Memory-Safe Testing](src/memory-safe-testing.md) - Testing patterns and best practices
  - [Architecture Overview](src/architecture.md) - System design and components
  - [Performance Optimization](src/performance-optimization.md) - Optimization techniques

### **🚀 Getting Started**
- [User Guide](src/getting-started.md) - Quick start and basic usage
- [API Reference](API_REFERENCE.md) - Complete API documentation
- [Examples](../examples/) - Usage examples and tutorials

### **🏗️ Architecture & Design**
- [Architecture Overview](architecture/ARCHITECTURE.md) - System architecture and design
- [Performance Analysis](performance/COMPREHENSIVE_PERFORMANCE_ANALYSIS.md) - Performance characteristics

### **🔗 Integration Guides**
- [EPCIS Ecosystem Integration](EPCIS_ECOSYSTEM_INTEGRATION.md) - Supply chain integration examples

### **📊 Reports & Analysis**
- [Analysis Reports](reports/) - Comprehensive analysis and status reports
  - [Code Analysis Report](reports/CODE_ANALYSIS_REPORT.md)
  - [Production Readiness](reports/PRODUCTION_READINESS_SUMMARY.md)
  - [Memory Safety Implementation](reports/MEMORY_SAFETY_IMPLEMENTATION_SUMMARY.md)
- [Performance Analysis](performance/) - Benchmarking and optimization

### **🛠️ Development & Planning**
- [Project Management](project/) - Project-related documentation
- [Test Suite Documentation](../tests/README.md) - Comprehensive testing information

## 🚀 Quick Links

### **Memory-Safe Testing Examples**
```bash
# Run all memory-safe tests
cargo test --lib

# Run with verbose memory reporting
OWL2_TEST_VERBOSE=1 cargo test --lib

# Run memory safety validation
cargo test memory_safety_validation --lib

# Run memory safety benchmarks
cargo bench --bench memory_safety_benchmarks
```

### **Core Examples**
```bash
# Basic reasoning
cargo run --example family_ontology
cargo run --example biomedical_ontology

# Performance benchmarking
cargo bench -- basic_benchmarks

# EPCIS integration
cargo run --example epcis_validation_suite
```

### **Key Documentation**
- [🛡️ Memory-Safe Testing Guide](MEMORY_SAFE_TESTING.md) - Comprehensive testing patterns
- [📖 mdBook Guide](src/SUMMARY.md) - Interactive documentation
- [🏗️ Architecture](architecture/ARCHITECTURE.md) - System design and components
- [📊 Performance Analysis](performance/COMPREHENSIVE_PERFORMANCE_ANALYSIS.md) - Performance characteristics
- [🔧 API Reference](API_REFERENCE.md) - Complete API documentation

### **Development Resources**
- [🧪 Test Suite Documentation](../tests/README.md) - Memory-safe testing information
- [📋 Project Management](project/) - Project organization and planning

## 🔗 Related Resources

- [GitHub Repository](https://github.com/anusornc/owl2-reasoner)
- [Crates.io Package](https://crates.io/crates/owl2-reasoner)
- [🛡️ Memory-Safe Testing Guide](MEMORY_SAFE_TESTING.md)
- [📖 Interactive Documentation (mdBook)](src/SUMMARY.md)
- [📊 Performance Analysis](performance/COMPREHENSIVE_PERFORMANCE_ANALYSIS.md)
- [🧪 Test Suite Documentation](../tests/README.md)