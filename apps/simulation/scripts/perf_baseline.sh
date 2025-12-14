#!/bin/bash
# Performance baseline measurement for Speciate simulation
# Usage: ./scripts/perf_baseline.sh [creature_count] [duration_sec]

set -e

CREATURE_COUNT=${1:-200000}
DURATION=${2:-10}
BINARY="./target/release/simulation"

echo "=================================="
echo "Speciate Performance Baseline"
echo "=================================="
echo "Creatures: $CREATURE_COUNT"
echo "Duration: ${DURATION}s"
echo "Binary: $BINARY"
echo ""

if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found at $BINARY"
    echo "Run: cargo build --release"
    exit 1
fi

if ! command -v perf &> /dev/null; then
    echo "ERROR: perf not found. Install with:"
    echo "  sudo apt-get install linux-tools-common linux-tools-generic"
    exit 1
fi

echo "=================================="
echo "TEST 1: Hardware Counters (Health Check)"
echo "=================================="
perf stat -e instructions,cycles,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses,branch-misses \
    timeout ${DURATION}s $BINARY 2>&1 | tee perf_stat.txt

echo ""
echo "=================================="
echo "TEST 2: IPC Analysis"
echo "=================================="
INSTRUCTIONS=$(grep "instructions" perf_stat.txt | awk '{print $1}' | tr -d ',')
CYCLES=$(grep "cycles" perf_stat.txt | awk '{print $1}' | tr -d ',')

if [ -n "$INSTRUCTIONS" ] && [ -n "$CYCLES" ] && [ "$CYCLES" -gt 0 ]; then
    IPC=$(echo "scale=2; $INSTRUCTIONS / $CYCLES" | bc)
    echo "IPC: $IPC"

    if (( $(echo "$IPC < 0.8" | bc -l) )); then
        echo "STATUS: MEMORY BOUND (IPC < 0.8) - CPU waiting on RAM"
    elif (( $(echo "$IPC < 2.0" | bc -l) )); then
        echo "STATUS: MODERATE (0.8 < IPC < 2.0) - Mixed workload"
    else
        echo "STATUS: EXCELLENT (IPC > 2.0) - Good SIMD/cache usage"
    fi
else
    echo "WARNING: Could not calculate IPC"
fi

echo ""
echo "=================================="
echo "TEST 3: Cache Miss Rates"
echo "=================================="
L1_LOADS=$(grep "L1-dcache-loads" perf_stat.txt | awk '{print $1}' | tr -d ',')
L1_MISSES=$(grep "L1-dcache-load-misses" perf_stat.txt | awk '{print $1}' | tr -d ',')
LLC_LOADS=$(grep "LLC-loads" perf_stat.txt | grep -v "LLC-load-misses" | awk '{print $1}' | tr -d ',')
LLC_MISSES=$(grep "LLC-load-misses" perf_stat.txt | awk '{print $1}' | tr -d ',')

if [ -n "$L1_LOADS" ] && [ -n "$L1_MISSES" ] && [ "$L1_LOADS" -gt 0 ]; then
    L1_MISS_RATE=$(echo "scale=2; ($L1_MISSES / $L1_LOADS) * 100" | bc)
    echo "L1 Miss Rate: ${L1_MISS_RATE}%"

    if (( $(echo "$L1_MISS_RATE > 5" | bc -l) )); then
        echo "  WARNING: High L1 miss rate (target < 5%)"
        echo "  ACTION: Check component sizes and access patterns"
    else
        echo "  OK: L1 miss rate within target"
    fi
fi

if [ -n "$LLC_LOADS" ] && [ -n "$LLC_MISSES" ] && [ "$LLC_LOADS" -gt 0 ]; then
    LLC_MISS_RATE=$(echo "scale=2; ($LLC_MISSES / $LLC_LOADS) * 100" | bc)
    echo "LLC Miss Rate: ${LLC_MISS_RATE}%"

    if (( $(echo "$LLC_MISS_RATE > 1" | bc -l) )); then
        echo "  WARNING: High LLC miss rate (target < 1%)"
        echo "  ACTION: Reduce random memory access (HashMaps, pointer chasing)"
    else
        echo "  OK: LLC miss rate within target"
    fi
fi

echo ""
echo "=================================="
echo "Baseline complete!"
echo "Output saved to: perf_stat.txt"
echo ""
echo "Next steps:"
echo "  1. Record hotspots: perf record --call-graph dwarf -e L1-dcache-load-misses timeout ${DURATION}s $BINARY"
echo "  2. Visualize: hotspot perf.data"
echo "  3. Compare: Run again after optimization and diff results"
echo "=================================="
