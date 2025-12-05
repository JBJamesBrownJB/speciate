#!/bin/bash
# A/B Comparison: Brute-Force vs Spatial Grid Perception
#
# This script compares hardware metrics between two perception implementations
# to quantify the cache performance difference.

set -euo pipefail

DURATION=10
RESULTS_DIR="./comparison_$(date +%Y%m%d_%H%M%S)"

mkdir -p "$RESULTS_DIR"

echo "=========================================="
echo "A/B Comparison: Perception Implementations"
echo "=========================================="
echo ""

# Check if both branches exist
CURRENT_BRANCH=$(git branch --show-current)
echo "Current branch: $CURRENT_BRANCH"
echo ""

# Function to build and profile
profile_implementation() {
    local BRANCH=$1
    local LABEL=$2
    local OUTPUT_PREFIX="$RESULTS_DIR/${LABEL}"

    echo "----------------------------------------"
    echo "Testing: $LABEL (branch: $BRANCH)"
    echo "----------------------------------------"

    # Checkout branch
    git checkout "$BRANCH" 2>&1 | grep -v "Already on" || true

    # Clean build
    cargo build --release 2>&1 | tail -n 5

    # Hardware counters
    echo "Running perf stat..."
    perf stat -e instructions,cycles,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses \
        -o "${OUTPUT_PREFIX}_stats.txt" \
        timeout ${DURATION}s ./target/release/sim_app 2>&1 | tail -n 10 > "${OUTPUT_PREFIX}_stdout.log"

    # Parse metrics
    local IPC=$(awk '/instructions/ {inst=$1; gsub(/,/, "", inst)} /cycles/ {cyc=$1; gsub(/,/, "", cyc)} END {printf "%.3f", inst/cyc}' "${OUTPUT_PREFIX}_stats.txt")
    local L1_MISS=$(awk '/L1-dcache-loads/ {loads=$1; gsub(/,/, "", loads)} /L1-dcache-load-misses/ {miss=$1; gsub(/,/, "", miss)} END {printf "%.2f", (miss/loads)*100}' "${OUTPUT_PREFIX}_stats.txt")
    local LLC_MISS=$(awk '/LLC-loads/ {loads=$1; gsub(/,/, "", loads)} /LLC-load-misses/ {miss=$1; gsub(/,/, "", miss)} END {printf "%.2f", (miss/loads)*100}' "${OUTPUT_PREFIX}_stats.txt")

    echo "Results:"
    echo "  IPC:          $IPC"
    echo "  L1 Miss Rate: ${L1_MISS}%"
    echo "  LLC Miss Rate: ${LLC_MISS}%"
    echo ""

    # Save parsed metrics
    cat > "${OUTPUT_PREFIX}_summary.txt" <<EOF
Branch: $BRANCH
Label: $LABEL
IPC: $IPC
L1_Miss_Rate: ${L1_MISS}%
LLC_Miss_Rate: ${LLC_MISS}%
EOF

    # Cache miss hotspot
    echo "Recording LLC misses..."
    perf record --call-graph dwarf -e LLC-load-misses \
        -o "${OUTPUT_PREFIX}_llc.data" \
        timeout ${DURATION}s ./target/release/sim_app > /dev/null 2>&1

    perf report -i "${OUTPUT_PREFIX}_llc.data" --stdio \
        --sort=symbol --percent-limit=1 \
        > "${OUTPUT_PREFIX}_llc_hotspots.txt"

    echo "Complete. Results in ${OUTPUT_PREFIX}_*.txt"
    echo ""
}

# ============================================================================
# Profile both implementations
# ============================================================================

# Profile current implementation (spatial grid)
profile_implementation "$CURRENT_BRANCH" "spatial_grid"

# Profile brute-force (if branch exists)
if git rev-parse --verify main > /dev/null 2>&1; then
    profile_implementation "main" "brute_force"
else
    echo "WARNING: main branch not found. Skipping brute-force comparison."
    echo "Please specify the branch with the O(N²) implementation."
fi

# Return to original branch
git checkout "$CURRENT_BRANCH" 2>&1 | grep -v "Already on" || true

# ============================================================================
# Generate comparison report
# ============================================================================

echo "=========================================="
echo "COMPARISON REPORT"
echo "=========================================="
echo ""

if [ -f "$RESULTS_DIR/brute_force_summary.txt" ] && [ -f "$RESULTS_DIR/spatial_grid_summary.txt" ]; then
    # Parse both summaries
    BF_IPC=$(grep "^IPC:" "$RESULTS_DIR/brute_force_summary.txt" | awk '{print $2}')
    BF_L1=$(grep "^L1_Miss_Rate:" "$RESULTS_DIR/brute_force_summary.txt" | awk '{print $2}')
    BF_LLC=$(grep "^LLC_Miss_Rate:" "$RESULTS_DIR/brute_force_summary.txt" | awk '{print $2}')

    SG_IPC=$(grep "^IPC:" "$RESULTS_DIR/spatial_grid_summary.txt" | awk '{print $2}')
    SG_L1=$(grep "^L1_Miss_Rate:" "$RESULTS_DIR/spatial_grid_summary.txt" | awk '{print $2}')
    SG_LLC=$(grep "^LLC_Miss_Rate:" "$RESULTS_DIR/spatial_grid_summary.txt" | awk '{print $2}')

    echo "Metric                | Brute-Force | Spatial Grid | Change"
    echo "----------------------|-------------|--------------|--------"
    printf "IPC                   | %-11s | %-12s | " "$BF_IPC" "$SG_IPC"
    if (( $(echo "$SG_IPC < $BF_IPC" | bc -l) )); then
        echo "⬇️ WORSE"
    else
        echo "⬆️ BETTER"
    fi

    printf "L1 Miss Rate          | %-11s | %-12s | " "$BF_L1" "$SG_L1"
    if (( $(echo "${SG_L1%\%} > ${BF_L1%\%}" | bc -l) )); then
        echo "⬇️ WORSE"
    else
        echo "⬆️ BETTER"
    fi

    printf "LLC Miss Rate         | %-11s | %-12s | " "$BF_LLC" "$SG_LLC"
    if (( $(echo "${SG_LLC%\%} > ${BF_LLC%\%}" | bc -l) )); then
        echo "⬇️ WORSE"
    else
        echo "⬆️ BETTER"
    fi

    echo ""
    echo "Interpretation:"
    echo "--------------"
    if (( $(echo "$SG_IPC < $BF_IPC" | bc -l) )); then
        echo "❌ Spatial Grid has LOWER IPC → CPU is more stalled"
    fi

    if (( $(echo "${SG_LLC%\%} > ${BF_LLC%\%}" | bc -l) )); then
        echo "❌ Spatial Grid has HIGHER LLC miss rate → Random access (HashMap)"
    fi

    echo ""
    echo "Recommendation:"
    echo "--------------"
    if (( $(echo "$SG_IPC < $BF_IPC" | bc -l) )) && (( $(echo "${SG_LLC%\%} > ${BF_LLC%\%}" | bc -l) )); then
        echo "The spatial grid is SLOWER due to cache thrashing."
        echo "Consider:"
        echo "  1. Switch to flat 2D array (O(1) cell lookup, no hashing)"
        echo "  2. Increase cell size to reduce HashMap queries"
        echo "  3. Revert to brute-force for small populations (<5K entities)"
    else
        echo "Spatial grid shows improvement or equivalent performance."
    fi
else
    echo "WARNING: Missing summary files. Cannot generate comparison."
fi

echo ""
echo "Full results saved to: $RESULTS_DIR"
echo ""
echo "Next steps:"
echo "  1. Review flamegraphs:"
echo "     samply record timeout 10s ./target/release/sim_app"
echo ""
echo "  2. Examine LLC hotspots:"
echo "     cat $RESULTS_DIR/spatial_grid_llc_hotspots.txt"
echo ""
echo "  3. Check perception function timings in Dev UI"
