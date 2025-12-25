#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "Building and running throttle performance test..."
echo ""

cargo run --release --example throttle_perf
