#!/bin/bash
# Hardware Telemetry Analysis: Spatial Grid Cache Performance
#
# This script measures the cache impact of FxHashMap-based spatial grid
# vs the previous O(N²) brute-force approach.

set -euo pipefail

BINARY="./target/release/sim_app"
DURATION=10
RESULTS_DIR="./perf_results_$(date +%Y%m%d_%H%M%S)"

# Ensure binary is built with release optimizations
if [ ! -f "$BINARY" ]; then
    echo "ERROR: Release binary not found at $BINARY"
    echo "Build with: cargo build --release"
    exit 1
fi

mkdir -p "$RESULTS_DIR"

echo "==================================="
echo "Hardware Telemetry: Spatial Grid"
echo "Results: $RESULTS_DIR"
echo "==================================="
echo ""

# ============================================================================
# PHASE 1: HEALTH CHECK - Baseline Hardware Counters
# ============================================================================
echo "[PHASE 1] Hardware Baseline (perf stat)"
echo "----------------------------------------"

perf stat -e instructions,cycles,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses,LLC-stores,branch-misses \
    -o "$RESULTS_DIR/01_health_check.txt" \
    timeout ${DURATION}s "$BINARY" 2>&1 | tee "$RESULTS_DIR/01_stdout.log"

echo ""
echo "Health Check Analysis:"
cat "$RESULTS_DIR/01_health_check.txt"
echo ""

# Calculate IPC
IPC=$(awk '/instructions/ {inst=$1} /cycles/ {cycles=$1} END {print inst/cycles}' "$RESULTS_DIR/01_health_check.txt")
echo "IPC: $IPC"
echo ""

# ============================================================================
# PHASE 2: CACHE MISS PROFILE - L1 Data Cache
# ============================================================================
echo "[PHASE 2] L1 Cache Miss Hotspots"
echo "---------------------------------"

perf record --call-graph dwarf -e L1-dcache-load-misses \
    -o "$RESULTS_DIR/02_l1_misses.data" \
    timeout ${DURATION}s "$BINARY" > /dev/null 2>&1

perf report -i "$RESULTS_DIR/02_l1_misses.data" --stdio \
    --sort=symbol,dso --percent-limit=1 \
    > "$RESULTS_DIR/02_l1_misses_report.txt"

echo "L1 Miss Hotspots (>1% weight):"
head -n 50 "$RESULTS_DIR/02_l1_misses_report.txt"
echo ""
echo "Full report: $RESULTS_DIR/02_l1_misses_report.txt"
echo ""

# ============================================================================
# PHASE 3: LLC MISS PROFILE - Random Access Detection
# ============================================================================
echo "[PHASE 3] LLC (L3) Miss Hotspots - HashMap Random Access"
echo "---------------------------------------------------------"

perf record --call-graph dwarf -e LLC-load-misses \
    -o "$RESULTS_DIR/03_llc_misses.data" \
    timeout ${DURATION}s "$BINARY" > /dev/null 2>&1

perf report -i "$RESULTS_DIR/03_llc_misses.data" --stdio \
    --sort=symbol,dso --percent-limit=1 \
    > "$RESULTS_DIR/03_llc_misses_report.txt"

echo "LLC Miss Hotspots (>1% weight):"
head -n 50 "$RESULTS_DIR/03_llc_misses_report.txt"
echo ""
echo "Full report: $RESULTS_DIR/03_llc_misses_report.txt"
echo ""

# ============================================================================
# PHASE 4: INSTRUCTION-LEVEL PROFILE - Find Hot Loops
# ============================================================================
echo "[PHASE 4] CPU Time Profile (samply)"
echo "------------------------------------"

if command -v samply &> /dev/null; then
    samply record -o "$RESULTS_DIR/04_cpu_profile.json" \
        timeout ${DURATION}s "$BINARY" > /dev/null 2>&1
    echo "Flamegraph: $RESULTS_DIR/04_cpu_profile.json"
    echo "Open with: samply load $RESULTS_DIR/04_cpu_profile.json"
else
    echo "WARNING: samply not installed. Skipping CPU profile."
    echo "Install with: cargo install samply"
fi
echo ""

# ============================================================================
# PHASE 5: PERCEPTION SYSTEM DRILL-DOWN
# ============================================================================
echo "[PHASE 5] Perception System Isolation"
echo "--------------------------------------"

# Record only perception-related functions
perf record --call-graph dwarf -e cycles:pp \
    -o "$RESULTS_DIR/05_perception.data" \
    timeout ${DURATION}s "$BINARY" > /dev/null 2>&1

perf report -i "$RESULTS_DIR/05_perception.data" --stdio \
    --sort=symbol --percent-limit=0.5 \
    | grep -E "perception|spatial|grid|query" \
    > "$RESULTS_DIR/05_perception_hotspots.txt" || true

echo "Perception/Spatial hotspots:"
cat "$RESULTS_DIR/05_perception_hotspots.txt"
echo ""

# ============================================================================
# PHASE 6: MEMORY BANDWIDTH ANALYSIS
# ============================================================================
echo "[PHASE 6] Memory Bandwidth Saturation"
echo "--------------------------------------"

perf stat -e cycles,instructions,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses,dTLB-load-misses \
    -I 1000 \
    -o "$RESULTS_DIR/06_bandwidth_timeseries.txt" \
    timeout ${DURATION}s "$BINARY" > /dev/null 2>&1

echo "Time-series memory events (per second):"
cat "$RESULTS_DIR/06_bandwidth_timeseries.txt"
echo ""

# ============================================================================
# SUMMARY & ANALYSIS
# ============================================================================
echo ""
echo "==================================="
echo "ANALYSIS SUMMARY"
echo "==================================="

# Parse key metrics
L1_MISS_RATE=$(awk '
    /L1-dcache-loads/ {loads=$1; gsub(/,/, "", loads)}
    /L1-dcache-load-misses/ {misses=$1; gsub(/,/, "", misses)}
    END {if (loads > 0) print (misses/loads)*100; else print "N/A"}
' "$RESULTS_DIR/01_health_check.txt")

LLC_MISS_RATE=$(awk '
    /LLC-loads/ {loads=$1; gsub(/,/, "", loads)}
    /LLC-load-misses/ {misses=$1; gsub(/,/, "", misses)}
    END {if (loads > 0) print (misses/loads)*100; else print "N/A"}
' "$RESULTS_DIR/01_health_check.txt")

echo "IPC:              $IPC"
echo "L1 Miss Rate:     ${L1_MISS_RATE}%"
echo "LLC Miss Rate:    ${LLC_MISS_RATE}%"
echo ""

echo "Diagnosis Rubric:"
echo "----------------"
echo "IPC < 0.8?           → Memory Bound (CPU stalled waiting on RAM)"
echo "L1 Miss > 5%?        → Poor Locality (cold data structures)"
echo "LLC Miss > 1%?       → Random Access (HashMap pointer chasing)"
echo ""

# Interpretation
if (( $(echo "$IPC < 0.8" | bc -l) )); then
    echo "ALERT: Memory bound detected (IPC=$IPC)"
fi

if (( $(echo "$L1_MISS_RATE > 5" | bc -l) )); then
    echo "ALERT: High L1 miss rate (${L1_MISS_RATE}%)"
fi

if (( $(echo "$LLC_MISS_RATE > 1" | bc -l) )); then
    echo "ALERT: High LLC miss rate (${LLC_MISS_RATE}%) - Random access pattern"
fi

echo ""
echo "Next Steps:"
echo "----------"
echo "1. Examine flamegraph: samply load $RESULTS_DIR/04_cpu_profile.json"
echo "2. Check LLC hotspots in: $RESULTS_DIR/03_llc_misses_report.txt"
echo "3. If HashMap is the culprit, consider:"
echo "   - Flat Vec2D array (dense spatial indexing)"
echo "   - BVH (Bounding Volume Hierarchy)"
echo "   - Sort-and-sweep (cache-friendly linear scan)"
echo ""
echo "Results saved to: $RESULTS_DIR"
