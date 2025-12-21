#!/usr/bin/env bash
# Performance measurement script for Phase B: Drive Simplex
# Usage: ./scripts/measure_phase_b.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== Phase B Performance Measurement ==="
echo "Date: $(date -Iseconds)"
echo "Commit: $(git rev-parse --short HEAD)"
echo ""

# Ensure release build
echo "Building release binary..."
cargo build --release --quiet

BINARY="$PROJECT_ROOT/target/release/sim_app"

# Create output directory
OUTPUT_DIR="$PROJECT_ROOT/../../docs/performance/snapshots/phase-b"
mkdir -p "$OUTPUT_DIR"

# Test 1: Drive Computation Baseline
echo ""
echo "Test 1: Drive Computation Baseline"
echo "==================================="
echo ""

for CREATURE_COUNT in 20000 100000 360000; do
    echo "Measuring drive computation at $CREATURE_COUNT creatures..."

    # Focus on IPC and cache metrics
    perf stat \
        -e cycles,instructions,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses \
        -o "$OUTPUT_DIR/drive_${CREATURE_COUNT}.perf" \
        timeout 10s "$BINARY" \
        --creatures "$CREATURE_COUNT" \
        --duration 10 \
        --enable-drive-system \
        2>&1 | tee "$OUTPUT_DIR/drive_${CREATURE_COUNT}.log"

    echo ""
    echo "Results saved to $OUTPUT_DIR/drive_${CREATURE_COUNT}.{perf,log}"
    echo ""
done

# Extract IPC from perf output
echo ""
echo "IPC Summary:"
echo "------------"
for CREATURE_COUNT in 20000 100000 360000; do
    IPC=$(grep "insn per cycle" "$OUTPUT_DIR/drive_${CREATURE_COUNT}.perf" | awk '{print $1}')
    echo "$CREATURE_COUNT creatures: IPC = $IPC (target: > 1.8)"
done
echo ""

# Test 2: Drive System vs Behavior State Machine
echo ""
echo "Test 2: Drive System vs Behavior State Machine"
echo "==============================================="
echo ""

CREATURE_COUNT=360000

echo "Measuring OLD behavior state machine performance..."
git stash push -m "Temp: switch to behavior state machine"
git checkout HEAD~1  # Assume previous commit has old behavior system
cargo build --release --quiet

perf stat \
    -e cycles,instructions,L1-dcache-load-misses \
    -r 3 \
    -o "$OUTPUT_DIR/behavior_old.perf" \
    timeout 10s "$BINARY" \
    --creatures "$CREATURE_COUNT" \
    --duration 10 \
    2>&1 | tee "$OUTPUT_DIR/behavior_old.log"

git checkout -  # Return to drive system
git stash pop || true
cargo build --release --quiet

echo ""
echo "Measuring NEW drive simplex performance..."
perf stat \
    -e cycles,instructions,L1-dcache-load-misses \
    -r 3 \
    -o "$OUTPUT_DIR/behavior_new.perf" \
    timeout 10s "$BINARY" \
    --creatures "$CREATURE_COUNT" \
    --duration 10 \
    --enable-drive-system \
    2>&1 | tee "$OUTPUT_DIR/behavior_new.log"

echo ""
echo "Comparison saved to $OUTPUT_DIR/behavior_{old,new}.{perf,log}"
echo ""

# Test 3: L1 Drive Scan Microbenchmark
echo ""
echo "Test 3: L1 Drive Scan Microbenchmark"
echo "====================================="
echo ""

echo "Running isolated L1 drive scan benchmark..."
cargo bench --bench drive_bench -- --exact "l1_scan" 2>&1 | tee "$OUTPUT_DIR/drive_bench.log"

echo ""
echo "Benchmark results saved to $OUTPUT_DIR/drive_bench.log"
echo ""

# Test 4: Rayon Parallelization Validation
echo ""
echo "Test 4: Rayon Parallelization Validation"
echo "========================================="
echo ""

echo "Measuring drive system with perf record (for flamegraph)..."
perf record \
    --call-graph dwarf \
    -F 999 \
    -o "$OUTPUT_DIR/drive_parallel.perf.data" \
    timeout 10s "$BINARY" \
    --creatures "$CREATURE_COUNT" \
    --duration 10 \
    --enable-drive-system

echo ""
echo "Generating flamegraph..."
perf script -i "$OUTPUT_DIR/drive_parallel.perf.data" | \
    /usr/share/FlameGraph/stackcollapse-perf.pl | \
    /usr/share/FlameGraph/flamegraph.pl > "$OUTPUT_DIR/drive_parallel_flamegraph.svg"

echo "Flamegraph saved to $OUTPUT_DIR/drive_parallel_flamegraph.svg"
echo ""

echo "Checking CPU core utilization..."
perf stat \
    -e task-clock,context-switches \
    -a \
    timeout 10s "$BINARY" \
    --creatures "$CREATURE_COUNT" \
    --duration 10 \
    --enable-drive-system \
    2>&1 | tee "$OUTPUT_DIR/drive_parallel_cpu.log"

echo ""
echo "CPU utilization saved to $OUTPUT_DIR/drive_parallel_cpu.log"
echo "Expected: All 16 cores engaged (task-clock ~16× wall time)"
echo ""

# Test 5: Emergent Behavior Validation
echo ""
echo "Test 5: Emergent Behavior Validation"
echo "====================================="
echo ""

echo "Running dispersal test (creatures should spread from dense spawn)..."
cat > /tmp/dispersal_test.toml <<EOF
[[spawn]]
species = "wanderer"
min = 10000
max = 10000

# Dense spawn in center
[spawn.x_distribution]
min = -100
max = 100

[spawn.y_distribution]
min = -100
max = 100

[expectations]
# After 1000 ticks, creatures should have dispersed
# (this is a visual test - run in portal to confirm)
[expectations.metrics]
drive_computation_us_max = 2000
avg_drive_magnitude_min = 0.1
EOF

"$BINARY" \
    --spec /tmp/dispersal_test.toml \
    --duration 60 \
    2>&1 | tee "$OUTPUT_DIR/dispersal_test.log"

echo ""
echo "Dispersal test complete. Review telemetry in log."
echo "Visual validation required: Run in portal, observe creatures spreading."
echo ""

# Generate summary report
echo ""
echo "=== Phase B Performance Summary ==="
echo ""

cat > "$OUTPUT_DIR/summary.md" <<'EOF'
# Phase B Drive Simplex - Performance Summary

## Test 1: Drive Computation Baseline

| Creature Count | Drive Time (target: < 2ms) | IPC (target: > 1.8) | L1D Miss Rate |
|----------------|----------------------------|---------------------|---------------|
| 20K | (extract from log) | (extract from perf) | (extract from perf) |
| 100K | (extract from log) | (extract from perf) | (extract from perf) |
| 360K | (extract from log) | (extract from perf) | (extract from perf) |

## Test 2: Drive vs Behavior State Machine

| Metric | Old Behavior | New Drive | Delta |
|--------|--------------|-----------|-------|
| Total Tick | (extract) | (extract) | (%) |
| Behavior/Drive Time | (extract) | (extract) | (%) |
| L1D Misses | (extract) | (extract) | (%) |

Expected: Drive system competitive or faster (simpler logic, no state machine).

## Test 3: L1 Drive Scan Microbenchmark

Extract from `drive_bench.log`:
- Time per creature (ns)
- L1 cells scanned per creature (avg)
- Cache miss rate

## Test 4: Rayon Parallelization

Check `drive_parallel_flamegraph.svg`:
- Verify rayon thread pool visible in flame graph
- Check CPU utilization: Should be ~16× (all cores engaged)

## Test 5: Emergent Behavior

Visual validation (run in portal):
- [ ] Creatures disperse from dense spawn
- [ ] Small creatures avoid large ones
- [ ] No jittering at equilibrium (resting state)

Quantitative:
- Resting rate: % of creatures with drive magnitude < threshold
- Expected: 30-50% at equilibrium

## Performance Gates

- [ ] Drive computation < 2ms at 360K creatures (parallel)
- [ ] IPC > 1.8 during drive computation (compute-bound)
- [ ] L1D miss rate < 3% (cache-friendly BioSignatures)
- [ ] Drive system competitive with old behavior system (< 2ms)
- [ ] All 16 CPU cores engaged (Rayon parallelization confirmed)
EOF

echo "Summary report generated: $OUTPUT_DIR/summary.md"
echo ""
echo "Analysis checklist:"
echo "  [ ] Drive computation < 2ms at 360K creatures"
echo "  [ ] IPC > 1.8 (compute-bound, not memory-bound)"
echo "  [ ] L1D miss rate < 3%"
echo "  [ ] Drive system faster than old behavior state machine"
echo "  [ ] Rayon parallelization confirmed (16 cores engaged)"
echo "  [ ] Emergent behaviors observed (dispersal, avoidance, equilibrium)"
echo ""
echo "Next steps:"
echo "  1. Extract drive_computation_us from telemetry logs"
echo "  2. Review flamegraph for hotspots"
echo "  3. Visual validation in portal (dispersal, avoidance)"
echo "  4. Validate against gates in performance assessment doc"
echo ""
