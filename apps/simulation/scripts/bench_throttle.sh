#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "=== Throttle Method Comparison Benchmark ==="
echo ""
echo "Building benchmark with optimizations..."
cargo build --release --bench throttle_comparison

echo ""
echo "Running criterion benchmark..."
cargo bench --bench throttle_comparison

echo ""
echo "Results saved to target/criterion/throttle_comparison/"
echo "Open target/criterion/report/index.html to view detailed results"
