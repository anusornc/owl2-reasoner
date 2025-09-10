#!/bin/bash

# Documentation Update Script
# Usage: ./update_docs.sh "Description of changes"

set -e

DESCRIPTION=${1:-"Documentation update"}

echo "📚 Updating OWL2 Reasoner Documentation"
echo "📝 Changes: $DESCRIPTION"

# 1. Update Rustdoc documentation
echo "🔧 Building Rustdoc documentation..."
cargo doc --no-deps

# 2. Build and test examples
echo "🧪 Testing examples..."
cargo check --example simple_example
cargo check --example family_ontology  
cargo check --example biomedical_ontology
cargo check --example performance_benchmarking

# 3. Build mdbook documentation
echo "📖 Building mdbook documentation..."
mdbook build docs

# 4. Run tests to ensure everything works
echo "✅ Running tests..."
cargo test

# 5. Show documentation locations
echo ""
echo "📚 Documentation Generated Successfully!"
echo ""
echo "📖 mdbook Documentation:"
echo "   file://$(pwd)/docs/book/index.html"
echo ""
echo "🛠️ Rustdoc API Documentation:"
echo "   file://$(pwd)/target/doc/owl2_reasoner/index.html"
echo ""
echo "💻 Examples:"
echo "   $(pwd)/examples/"
echo ""
echo "🔄 Next steps:"
echo "   1. Review generated documentation"
echo "   2. Test new features manually"
echo "   3. Commit changes with: git commit -m \"docs: $DESCRIPTION\""
echo "   4. Push to repository"