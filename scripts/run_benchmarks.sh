#!/bin/bash

# OWL2 Reasoner Benchmark Runner Script
# This script runs all benchmarks and generates comprehensive reports

set -e

echo "🚀 Starting OWL2 Reasoner Benchmark Suite"
echo "============================================"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Build the project first
echo "📦 Building project..."
cargo build --release

echo ""
echo "🏃 Running Rust benchmarks..."
cargo bench --bench basic_benchmarks
cargo bench --bench performance_validation
cargo bench --bench scale_testing

echo ""
echo "🐍 Running Python benchmarking framework..."
cd benchmarking/framework
python benchmark_runner.py --all
cd ../..

echo ""
echo "📊 Generating comprehensive reports..."
python benchmarking/framework/generate_report.py

echo ""
echo "✅ Benchmark suite completed successfully!"
echo "📈 Results available in:"
echo "   - target/criterion/ (Rust benchmark results)"
echo "   - benchmarking/results/ (Python framework results)"