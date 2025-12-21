#!/usr/bin/env bash
# Performance measurement script for Phase C: Frequency Control
# Usage: ./scripts/measure_phase_c.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== Phase C Performance Measurement ==="
echo "Date: $(date -Iseconds)"
echo "Commit: $(git rev-parse --short HEAD)"
echo ""

# Ensure release build
echo "Building release binary..."
cargo build --release --quiet

BINARY="$PROJECT_ROOT/target/release/sim_app"

# Create output directory
OUTPUT_DIR="$PROJECT_ROOT/../../docs/performance/snapshots/phase-c"
mkdir -p "$OUTPUT_DIR"

CREATURE_COUNT=360000

# Test 1: Zero Overhead Verification (divisor=1)
echo ""
echo "Test 1: Zero Overhead Verification (divisor=1)"
echo "==============================================="
echo ""

echo "Measuring baseline (no frequency control code)..."
perf stat \
    -e cycles,instructions,L1-dcache-load-misses,branch-misses \
    -r 5 \
    -o "$OUTPUT_DIR/divisor_baseline.perf" \
    timeout 10s "$BINARY" \
    --creatures "$CREATURE_COUNT" \
    --duration 10 \
    --no-frequency-control \
    2>&1 | tee "$OUTPUT_DIR/divisor_baseline.log"

echo ""
echo "Measuring with divisor=1 (fast path)..."
perf stat \
    -e cycles,instructions,L1-dcache-load-misses,branch-misses \
    -r 5 \
    -o "$OUTPUT_DIR/divisor_1_fast.perf" \
    timeout 10s "$BINARY" \
    --creatures "$CREATURE_COUNT" \
    --duration 10 \
    --perception-divisor 1 \
    2>&1 | tee "$OUTPUT_DIR/divisor_1_fast.log"

echo ""
echo "Results saved to $OUTPUT_DIR/divisor_{baseline,1_fast}.{perf,log}"
echo "Expected: Within 2% margin of error"
echo ""

# Test 2: Throttled Performance Scaling
echo ""
echo "Test 2: Throttled Performance Scaling"
echo "======================================"
echo ""

for DIVISOR in 1 2 4 8 10; do
    EFFECTIVE_HZ=$((20 / DIVISOR))
    echo "Measuring performance at divisor=$DIVISOR (${EFFECTIVE_HZ}Hz effective)..."

    perf stat \
        -e cycles,instructions,L1-dcache-load-misses,LLC-load-misses \
        -r 3 \
        -o "$OUTPUT_DIR/divisor_${DIVISOR}.perf" \
        timeout 10s "$BINARY" \
        --creatures "$CREATURE_COUNT" \
        --duration 10 \
        --perception-divisor "$DIVISOR" \
        2>&1 | tee "$OUTPUT_DIR/divisor_${DIVISOR}.log"

    echo ""
done

echo "Scaling results saved to $OUTPUT_DIR/divisor_{1,2,4,8,10}.{perf,log}"
echo ""

# Test 3: Spatial Bucketing vs Entity Bucketing (Cache Analysis)
echo ""
echo "Test 3: Spatial Bucketing vs Entity Bucketing"
echo "=============================================="
echo ""

echo "Measuring cache behavior with L1 cell bucketing (spatial locality)..."
perf stat \
    -e L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses \
    -r 5 \
    -o "$OUTPUT_DIR/bucketing_spatial.perf" \
    timeout 10s "$BINARY" \
    --creatures "$CREATURE_COUNT" \
    --duration 10 \
    --perception-divisor 10 \
    --bucketing-method spatial \
    2>&1 | tee "$OUTPUT_DIR/bucketing_spatial.log"

echo ""
echo "Measuring cache behavior with entity ID bucketing (random)..."
perf stat \
    -e L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses \
    -r 5 \
    -o "$OUTPUT_DIR/bucketing_entity.perf" \
    timeout 10s "$BINARY" \
    --creatures "$CREATURE_COUNT" \
    --duration 10 \
    --perception-divisor 10 \
    --bucketing-method entity \
    2>&1 | tee "$OUTPUT_DIR/bucketing_entity.log"

echo ""
echo "Bucketing comparison saved to $OUTPUT_DIR/bucketing_{spatial,entity}.{perf,log}"
echo "Expected: Spatial bucketing 20-40% fewer cache misses"
echo ""

# Test 4: Determinism Verification Across Divisors
echo ""
echo "Test 4: Determinism Verification"
echo "================================="
echo ""

for DIVISOR in 1 2 4 8; do
    echo "Running determinism test with divisor=$DIVISOR..."
    cargo test --release test_deterministic_simulation_20k -- \
        --test-args "--perception-divisor $DIVISOR" \
        2>&1 | tee "$OUTPUT_DIR/determinism_divisor_${DIVISOR}.log"

    if [ $? -ne 0 ]; then
        echo "ERROR: Determinism test FAILED at divisor=$DIVISOR"
        exit 1
    fi
    echo "PASS: Determinism verified at divisor=$DIVISOR"
    echo ""
done

# Generate comparison table
echo ""
echo "=== Phase C Performance Summary ==="
echo ""

cat > "$OUTPUT_DIR/summary.md" <<'EOF'
# Phase C Frequency Control - Performance Summary

## Test 1: Zero Overhead (divisor=1)

Compare `divisor_baseline.perf` vs `divisor_1_fast.perf`:
- Cycles: Should be within 2%
- Instructions: Should be within 1%
- Branch misses: Should be negligible

## Test 2: Throttling Scaling

| Divisor | Effective Hz | Expected Perception Time | Expected Total Tick |
|---------|--------------|--------------------------|---------------------|
| 1 | 20 | 5.9ms | 29.3ms |
| 2 | 10 | 3.0ms | 26.4ms |
| 4 | 5 | 1.5ms | 24.9ms |
| 8 | 2.5 | 0.75ms | 23.8ms |
| 10 | 2 | 0.6ms | 23.6ms |

Extract actual values from `divisor_*.log` telemetry output.

## Test 3: Bucketing Method Comparison

Compare `bucketing_spatial.perf` vs `bucketing_entity.perf`:
- L1 cache miss rate: Spatial should be 20-40% lower
- LLC miss rate: Spatial should be 20-40% lower
- Explanation: Nearby creatures share L0 scan results

## Test 4: Determinism

All divisors must pass determinism test:
- [x] divisor=1
- [x] divisor=2
- [x] divisor=4
- [x] divisor=8

## Performance Gates

- [ ] Divisor=1 within 2% of baseline (zero overhead verified)
- [ ] Divisor=2 reduces perception time by ~50%
- [ ] Divisor=4 reduces perception time by ~75%
- [ ] Spatial bucketing reduces cache misses by 20%+
- [ ] Determinism passes at all divisor values
EOF

echo "Summary report generated: $OUTPUT_DIR/summary.md"
echo ""
echo "Analysis checklist:"
echo "  [ ] Zero overhead verified (divisor=1 vs baseline < 2% difference)"
echo "  [ ] Linear scaling observed (divisor=2 → 50% reduction)"
echo "  [ ] Spatial bucketing superior to entity bucketing (cache metrics)"
echo "  [ ] Determinism tests pass at all divisors"
echo ""
echo "Next steps:"
echo "  1. Extract perception_us from telemetry logs"
echo "  2. Plot scaling curve (divisor vs total_tick_us)"
echo "  3. Compare L1/LLC miss rates for bucketing methods"
echo "  4. Validate against gates in performance assessment doc"
echo ""
