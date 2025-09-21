# OWL2 Reasoner Documentation

Welcome to the comprehensive documentation for the OWL2 Reasoner project.

## 📚 Documentation Structure

### **Getting Started**
- [User Guide](src/getting-started.md) - Quick start and basic usage
- [API Reference](API_REFERENCE.md) - Complete API documentation
- [Examples](src/examples/examples.md) - Usage examples and tutorials

### **Architecture & Design**
- [Architecture Overview](architecture/ARCHITECTURE.md) - System architecture and design
- [Technical Documentation](technical-documentation/README.md) - Technical implementation details
- [Performance Analysis](performance/COMPREHENSIVE_PERFORMANCE_ANALYSIS.md) - Performance characteristics

### **Integration Guides**
- [EPCIS Ecosystem Integration](guides/ECOSYSTEM_INTEGRATION.md) - Supply chain integration examples
- [Python Bindings](src/api/python-bindings.md) - Python integration guide
- [Web Services](src/api/web-services.md) - REST API documentation

### **Development & Planning**
- [Project Plans](plans/CLAUDE.md) - Development roadmap and plans
- [Git Workflow](plans/GIT_WORKFLOW.md) - Development workflow guide
- [Testing Guidelines](project/TESTING_IMPROVEMENT_PLAN.md) - Testing strategies

### **Reports & Analysis**
- [Performance Reports](reports/) - Benchmarking and validation reports
- [System Validation](reports/SYSTEM_VALIDATION_REPORT.md) - Comprehensive validation
- [Industry Analysis](performance/INDUSTRY_REASONER_ANALYSIS.md) - Industry comparison

### **Advanced Topics**
- [Neurosymbolic Reasoning](src/neurosymbolic-reasoning.md) - Advanced reasoning techniques
- [Custom Reasoners](src/advanced/custom-reasoners.md) - Extending the reasoner
- [Performance Tuning](src/advanced/performance-tuning.md) - Optimization techniques

## 🚀 Quick Links

### **Core Examples**
```bash
# Basic reasoning
cargo run --example family_ontology
cargo run --example biomedical_ontology

# Performance benchmarking
cargo bench -- basic_benchmarks

# EPCIS integration
cargo run --example epcis_validation_suite

# Ecosystem integration
cargo run --example ecosystem_integration_examples
```

### **Key Documentation**
- [Architecture](architecture/ARCHITECTURE.md) - System design and components
- [API Reference](API_REFERENCE.md) - Complete API documentation
- [EPCIS Integration](guides/ECOSYSTEM_INTEGRATION.md) - Enterprise integration
- [Performance Guide](src/advanced/performance-tuning.md) - Optimization techniques

### **Development Resources**
- [Git Workflow](plans/GIT_WORKFLOW.md) - How to contribute
- [Testing Guidelines](project/TESTING_IMPROVEMENT_PLAN.md) - Testing strategies
- [Build Instructions](src/developer/building.md) - Build and development setup

## 📊 Project Status

- ✅ **Core Implementation**: Complete OWL2 reasoning engine
- ✅ **Performance**: 56x memory efficiency improvement
- ✅ **Testing**: 241 tests passing with comprehensive validation
- ✅ **Ecosystem Integration**: Python, web services, data pipelines
- ✅ **Documentation**: Comprehensive guides and API reference

## 🔗 Related Resources

- [GitHub Repository](https://github.com/anusornc/owl2-reasoner)
- [Crates.io Package](https://crates.io/crates/owl2-reasoner)
- [API Documentation](API_REFERENCE.md)
- [Examples Gallery](src/examples/examples.md)