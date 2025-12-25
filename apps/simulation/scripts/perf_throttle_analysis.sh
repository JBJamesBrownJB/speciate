#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "=== Hardware-Level Throttle Comparison ==="
echo ""

echo "Building release binary..."
cargo build --release --example throttle_perf

BINARY="./target/release/examples/throttle_perf"

echo ""
echo "=== 1. Basic Timing (User-Space) ==="
$BINARY

echo ""
echo ""
echo "=== 2. Hardware Counters (perf stat) ==="
echo ""
echo "Running with IPC, cache misses, branch prediction..."

perf stat -e instructions,cycles,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses,branch-misses,branches $BINARY 2>&1 | grep -E "instructions|cycles|L1-dcache|LLC|branch|Performance counter stats|seconds time elapsed"

echo ""
echo ""
echo "=== 3. Detailed Assembly Analysis ==="
echo ""
echo "To see the actual assembly generated:"
echo "  cargo asm --release --example throttle_perf bitwise_throttle"
echo "  cargo asm --release --example throttle_perf modulo_throttle"
echo "  cargo asm --release --example throttle_perf ticket_throttle"
echo ""
echo "(Install cargo-asm with: cargo install cargo-show-asm)"
echo ""
echo "=== Analysis Complete ==="
