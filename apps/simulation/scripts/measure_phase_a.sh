#!/usr/bin/env bash
# Performance measurement script for Phase A: Dual Grid
# Usage: ./scripts/measure_phase_a.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== Phase A Performance Measurement ==="
echo "Date: $(date -Iseconds)"
echo "Commit: $(git rev-parse --short HEAD)"
echo ""

# Ensure release build
echo "Building release binary..."
cargo build --release --quiet

BINARY="$PROJECT_ROOT/target/release/sim_app"

# Create output directory
OUTPUT_DIR="$PROJECT_ROOT/../../docs/performance/snapshots/phase-a"
mkdir -p "$OUTPUT_DIR"

# Test 1: L1 Aggregation Overhead at Scale
echo ""
echo "Test 1: L1 Aggregation Overhead"
echo "================================"
echo ""

for CREATURE_COUNT in 20000 100000 360000; do
    echo "Measuring L1 aggregation at $CREATURE_COUNT creatures..."

    # Run with perf stat to capture cache metrics
    perf stat \
        -e cycles,instructions,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses \
        -o "$OUTPUT_DIR/l1_aggregation_${CREATURE_COUNT}.perf" \
        timeout 10s "$BINARY" \
        --creatures "$CREATURE_COUNT" \
        --duration 10 \
        2>&1 | tee "$OUTPUT_DIR/l1_aggregation_${CREATURE_COUNT}.log"

    echo "Results saved to $OUTPUT_DIR/l1_aggregation_${CREATURE_COUNT}.{perf,log}"
    echo ""
done

# Test 2: Early-Exit Effectiveness (Sparse Distribution)
echo ""
echo "Test 2: Early-Exit Effectiveness (Sparse)"
echo "=========================================="
echo ""

echo "Creating sparse distribution spec (10K creatures in corner)..."
cat > /tmp/sparse_test.toml <<EOF
[[spawn]]
species = "wanderer"
min = 10000
max = 10000

[spawn.x_distribution]
min = 0
max = 1000

[spawn.y_distribution]
min = 0
max = 1000

[expectations]
# Expect early-exit to skip 90%+ of creatures (rest of world is empty)
[expectations.metrics]
early_exit_rate_min = 0.85
perception_time_max_ms = 2.0
EOF

echo "Running sparse distribution test..."
perf stat \
    -e cycles,instructions,L1-dcache-load-misses,LLC-load-misses,branch-misses \
    -o "$OUTPUT_DIR/early_exit_sparse.perf" \
    timeout 10s "$BINARY" \
    --spec /tmp/sparse_test.toml \
    2>&1 | tee "$OUTPUT_DIR/early_exit_sparse.log"

echo ""
echo "Comparing to uniform distribution baseline..."
perf stat \
    -e cycles,instructions,L1-dcache-load-misses,LLC-load-misses,branch-misses \
    -o "$OUTPUT_DIR/early_exit_uniform.perf" \
    timeout 10s "$BINARY" \
    --creatures 10000 \
    --duration 10 \
    2>&1 | tee "$OUTPUT_DIR/early_exit_uniform.log"

echo ""
echo "Early-exit comparison saved to $OUTPUT_DIR/early_exit_{sparse,uniform}.{perf,log}"
echo ""

# Test 3: Size Domination Performance
echo ""
echo "Test 3: Size Domination Performance"
echo "===================================="
echo ""

echo "Creating size domination spec (1K giants + 10K mice)..."
cat > /tmp/size_domination.toml <<EOF
[[spawn]]
species = "wanderer"
min = 1000
max = 1000
# Giants: size 5m
[spawn.size]
min = 5.0
max = 5.0

[[spawn]]
species = "wanderer"
min = 10000
max = 10000
# Mice: size 0.5m
[spawn.size]
min = 0.5
max = 0.5
EOF

echo "Running size domination test..."
perf stat \
    -e cycles,instructions,L1-dcache-load-misses,LLC-load-misses \
    -o "$OUTPUT_DIR/size_domination.perf" \
    timeout 10s "$BINARY" \
    --spec /tmp/size_domination.toml \
    2>&1 | tee "$OUTPUT_DIR/size_domination.log"

echo ""
echo "Results saved to $OUTPUT_DIR/size_domination.{perf,log}"
echo ""

# Generate summary report
echo ""
echo "=== Phase A Measurement Summary ==="
echo ""
echo "All results saved to: $OUTPUT_DIR"
echo ""
echo "Analysis checklist:"
echo "  [ ] L1 aggregation < 0.5ms at 360K creatures"
echo "  [ ] L1 cache miss rate < 1% during aggregation"
echo "  [ ] Early-exit reduces perception time by 50%+ in sparse scenario"
echo "  [ ] IPC increases from baseline (1.68) to 1.8+"
echo "  [ ] Giants have smaller neighbor sets (fewer neighbors pass threshold)"
echo ""
echo "Next steps:"
echo "  1. Review perf output files for cache miss rates"
echo "  2. Compare early_exit sparse vs uniform perception times"
echo "  3. Extract l1_aggregation_us from telemetry logs"
echo "  4. Validate against gates in performance assessment doc"
echo ""
