#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "==================================================================="
echo "  HARDWARE-LEVEL THROTTLE ANALYSIS"
echo "  Comparing: Bitwise AND vs Ticket Component vs Modulo"
echo "==================================================================="
echo ""

echo "Building release binary with optimizations..."
cargo build --release --example throttle_ecs_perf

BINARY="./target/release/examples/throttle_ecs_perf"

echo ""
echo "==================================================================="
echo "  PHASE 1: User-Space Timing"
echo "==================================================================="
$BINARY

echo ""
echo ""
echo "==================================================================="
echo "  PHASE 2: Hardware Performance Counters"
echo "==================================================================="
echo ""
echo "Key Metrics:"
echo "  - IPC (Instructions Per Cycle): Higher is better (> 1.0 ideal)"
echo "  - L1 Cache Miss Rate: Lower is better (< 5% ideal)"
echo "  - Branch Miss Rate: Lower is better (< 2% ideal)"
echo ""

perf stat -e instructions,cycles,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses,branches,branch-misses $BINARY 2>&1 | tee /tmp/throttle_perf.txt

echo ""
echo ""
echo "==================================================================="
echo "  PHASE 3: Key Metrics Extraction"
echo "==================================================================="

# Extract and calculate key metrics
INSTRUCTIONS=$(grep "instructions" /tmp/throttle_perf.txt | awk '{print $1}' | tr -d ',')
CYCLES=$(grep "cycles" /tmp/throttle_perf.txt | awk '{print $1}' | tr -d ',')
L1_LOADS=$(grep "L1-dcache-loads" /tmp/throttle_perf.txt | awk '{print $1}' | tr -d ',')
L1_MISSES=$(grep "L1-dcache-load-misses" /tmp/throttle_perf.txt | awk '{print $1}' | tr -d ',')
BRANCHES=$(grep -E "^[[:space:]]*[0-9,]+ branches" /tmp/throttle_perf.txt | awk '{print $1}' | tr -d ',')
BRANCH_MISSES=$(grep "branch-misses" /tmp/throttle_perf.txt | awk '{print $1}' | tr -d ',')

echo ""
echo "IPC (Instructions Per Cycle):"
echo "  $(echo "scale=3; $INSTRUCTIONS / $CYCLES" | bc)"
echo ""

echo "L1 Cache Miss Rate:"
if [ ! -z "$L1_LOADS" ] && [ ! -z "$L1_MISSES" ]; then
    echo "  $(echo "scale=2; 100 * $L1_MISSES / $L1_LOADS" | bc)%"
else
    echo "  N/A"
fi
echo ""

echo "Branch Miss Rate:"
if [ ! -z "$BRANCHES" ] && [ ! -z "$BRANCH_MISSES" ]; then
    echo "  $(echo "scale=2; 100 * $BRANCH_MISSES / $BRANCHES" | bc)%"
else
    echo "  N/A"
fi

echo ""
echo "==================================================================="
echo "  PHASE 4: Assembly Inspection (Optional)"
echo "==================================================================="
echo ""
echo "To inspect the actual assembly generated for each approach:"
echo ""
echo "  cargo install cargo-show-asm  # (if not installed)"
echo ""
echo "  cargo asm --release --example throttle_ecs_perf approach_a_bitwise"
echo "  cargo asm --release --example throttle_ecs_perf approach_b_ticket"
echo "  cargo asm --release --example throttle_ecs_perf approach_c_modulo"
echo ""
echo "Look for:"
echo "  - AND instruction (approach A)"
echo "  - DIV instruction (approach C - expensive!)"
echo "  - Memory loads (approach B)"
echo ""
echo "==================================================================="
echo "  ANALYSIS COMPLETE"
echo "==================================================================="
