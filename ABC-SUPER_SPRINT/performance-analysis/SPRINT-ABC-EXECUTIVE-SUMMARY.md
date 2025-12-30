# Sprint A-B-C Performance Analysis - Executive Summary

**Date:** 2025-12-21 (Updated: 2025-12-30)
**Analyst:** claude-perf (Performance Telemetry Engineer)
**Status:** APPROVED for implementation

**Current Priority:** Phase H (Hierarchical Perception v2) - see `hierarchical-perception-v2.md`
**Updated Order:** A ✅ → 4 ✅ → C ✅ → **H** 🔄 → B ⏸️

---

## Recommendation

**PROCEED** with implementation order **A → C → B** with mandatory performance gates.

All three sprints are LOW-RISK from a performance perspective, with high expected benefits.

---

## Quick Reference

| Document | Purpose |
|----------|---------|
| `/home/dev/dev/speciate/docs/performance/analysis/sprint-abc-performance-assessment.md` | Full analysis (35 pages, detailed) |
| `/home/dev/dev/speciate/docs/performance/analysis/hardware-counter-reference.md` | Perf counter interpretation guide |
| `/home/dev/dev/speciate/apps/simulation/scripts/measure_phase_a.sh` | Phase A benchmarking script |
| `/home/dev/dev/speciate/apps/simulation/scripts/measure_phase_c.sh` | Phase C benchmarking script |
| `/home/dev/dev/speciate/apps/simulation/scripts/measure_phase_b.sh` | Phase B benchmarking script |

---

## Current Performance Baseline (360K Creatures @ 20Hz)

```
Total Tick:     29.3ms avg (58% of 50ms budget)
├─ Steering:     9.4ms (32%)
├─ Movement:     6.0ms (21%)  [Rayon: IPC 4.25, 16 cores]
├─ Perception:   5.9ms (20%)  [Rayon: IPC 1.68, memory-bound]
├─ Grid Rebuild: 4.2ms (14%)
└─ Behavior:     2.0ms (7%)

Hardware:
├─ IPC:              1.68 (memory-bound, below optimal)
├─ L1D Miss Rate:    3.4% (acceptable)
├─ LLC Miss Rate:    3.0% (high - working set exceeds L3)
└─ CPU Utilization:  45% (11 cores engaged)

Bottleneck: Perception and steering are memory-bound (IPC 1.68).
Opportunity: Early-exit, throttling, and drive simplification.
```

---

## Phase A: Dual Spatial Grid

### What It Does

1. **L1 Coarse Grid:** 30m cells (3×3 L0 cells) with BioSignature aggregation
2. **Early-Exit:** Skip L0 scan when L1 cell is "empty" (total_mass < threshold)
3. **Size Domination:** Large creatures ignore small ones (emergent from threshold)

### Performance Impact

**L1 Aggregation Overhead:**
- **Estimate:** 0.1-0.5ms at 360K creatures (sequential or Rayon)
- **Risk:** LOW (simple reduction, cache-friendly)
- **Gate:** Must be < 0.5ms

**Early-Exit Optimization:**
- **Expected:** 50%+ reduction in perception time for sparse scenarios
- **Mechanism:** Skip expensive L0 scan when no neighbors in L1 cell
- **Benefit:** IPC increases from 1.68 → 1.8-2.0 (less memory-bound)

**Size Domination:**
- **Benefit:** Giants have smaller neighbor sets (fewer below threshold)
- **Result:** Giant perception 5-10× faster than mice

### Gates

- [ ] L1 aggregation < 0.5ms at 360K creatures
- [ ] Early-exit reduces sparse perception by 50%+
- [ ] IPC increases to 1.8+ (from 1.68 baseline)
- [ ] Determinism test passes

### Measurement

```bash
cd /home/dev/dev/speciate/apps/simulation
./scripts/measure_phase_a.sh
```

**Review:** `/home/dev/dev/speciate/docs/performance/snapshots/phase-a/`

---

## Phase C: System Update Frequency

### What It Does

1. **L1 Cell Bucketing:** `if l1_cell_idx % divisor != current_bucket { skip }`
2. **Runtime Control:** Adjust divisor via IPC command (dev-ui sliders)
3. **Zero Overhead:** Fast path when `divisor=1` (full rate)

### Performance Impact

**Zero Overhead Claim:**
- **Tested:** Divisor=1 vs baseline (no frequency code)
- **Expected:** Within 2% margin of error
- **Method:** Use fast path (early return) to avoid modulo overhead

**Throttling Scaling:**

| Divisor | Effective Hz | Perception Time | Total Tick | Creatures Supported |
|---------|--------------|-----------------|------------|---------------------|
| 1 | 20 | 5.9ms | 29.3ms | 360K (current) |
| 2 | 10 | 3.0ms | 26.4ms | 500K |
| 4 | 5 | 1.5ms | 24.9ms | 700K |
| 10 | 2 | 0.6ms | 23.6ms | 1M+ |

**Spatial Bucketing Benefit:**
- Nearby creatures update together (L1 cell coherence)
- Shared L0 cache hits (creatures in same L1 cell scan same 9 L0 cells)
- **Expected:** 20-40% fewer cache misses vs entity-based bucketing

### Gates

- [ ] Divisor=1 within 2% of baseline (zero overhead)
- [ ] Divisor=2 reduces perception time by 50%
- [ ] Divisor=4 reduces perception time by 75%
- [ ] Determinism passes at all divisor values

### Measurement

```bash
cd /home/dev/dev/speciate/apps/simulation
./scripts/measure_phase_c.sh
```

**Review:** `/home/dev/dev/speciate/docs/performance/snapshots/phase-c/`

---

## Phase B: Simple Drive Simplex

### What It Does

1. **Remove BehaviorMode:** Delete state machine (Catatonic, Wandering, Seeking, Fleeing)
2. **L1 Drive System:** Compute repulsion/attraction from L1 BioSignatures
3. **Emergent Behavior:** Resting, wandering, fleeing emerge from drive gradients

### Performance Impact

**Drive Computation Cost:**
- **Complexity:** Per creature, scan ~35 L1 cells (perception_range ÷ 30m)
- **Operations:** 35 cells × 12 ops = 420 ops per creature
- **Estimate:** 360K × 420 ops = 151M ops → 1.5ms (parallel, 16 cores)
- **Target:** < 2ms (competitive with current behavior 2.0ms)

**L1 Scan Characteristics:**
- **Cache-Friendly:** BioSignatures are small (8 bytes: total_mass + max_size)
- **Expected IPC:** 2.0+ (compute-bound, not memory-bound)
- **Expected L1D Miss Rate:** < 3%

**Simplification Benefit:**
- No state transitions (remove branching)
- No wandering timer/random direction (simpler logic)
- Potentially faster than current behavior system

### Gates

- [ ] Drive computation < 2ms at 360K creatures
- [ ] IPC > 1.8 (compute-bound)
- [ ] L1D miss rate < 3%
- [ ] Rayon parallelization engaged (16 cores)
- [ ] Emergent behaviors validated (dispersal, avoidance, equilibrium)

### Measurement

```bash
cd /home/dev/dev/speciate/apps/simulation
./scripts/measure_phase_b.sh
```

**Review:** `/home/dev/dev/speciate/docs/performance/snapshots/phase-b/`

---

## Risk Assessment

### High-Risk Items (Require Pre-Implementation Validation)

**None.** All three phases are low-risk.

### Medium-Risk Items (Benchmark Early)

**Phase B: Drive Computation Scalability**
- **Risk:** L1 drive scan could be memory-bound (poor IPC, high cache misses)
- **Mitigation:** Benchmark L1 scan in isolation BEFORE removing behavior state machine
- **Abort Criteria:** If drive > 3ms at 360K, keep behavior state machine

### Low-Risk Items

- Phase A: L1 aggregation (simple reduction, cache-friendly)
- Phase A: Early-exit (pure optimization, doesn't change behavior)
- Phase C: Zero overhead (fast path ensures negligible cost)
- Phase C: Throttling (trivial modulo check, linear scaling)

---

## Expected Outcomes

### Performance Improvement Timeline

**After Phase A:**
- Sparse scenarios: 50% faster perception (early-exit)
- IPC: 1.68 → 1.8-2.0 (less memory-bound)
- Creature capacity: 360K stable, potential for 400K

**After Phase C:**
- With divisor=2: 500K creatures @ 26ms total tick
- With divisor=4: 700K creatures @ 25ms total tick
- Runtime control: Adjust Hz live via dev-ui

**After Phase B:**
- Simpler behavior logic (no state machine)
- Competitive or faster drive system (< 2ms)
- Emergent complexity from simple rules

### Combined Impact (All Phases)

| Scenario | Current | After A+C+B | Improvement |
|----------|---------|-------------|-------------|
| 360K creatures (divisor=1) | 29.3ms | ~27ms | 8% faster |
| 360K creatures (divisor=2) | 29.3ms | ~24ms | 18% faster |
| 500K creatures (divisor=2) | N/A | ~33ms | NEW capability |
| 700K creatures (divisor=4) | N/A | ~35ms | NEW capability |

---

## Critical Path

### Implementation Order: A → C → B

**Why This Order:**

1. **A first:** Provides L1 infrastructure needed by both C and B
2. **C before B:** Proves throttling works without complex behavior changes
3. **B last:** Requires L1 grid (from A) and benefits from throttling (from C)

### Mandatory Gates

**Phase A (MUST PASS):**
1. L1 aggregation < 0.5ms at 360K
2. Early-exit reduces sparse perception by 50%+
3. Determinism test passes

**Phase C (MUST PASS):**
1. Zero overhead verified (divisor=1 within 2% of baseline)
2. Linear scaling confirmed (divisor=2 → 50% reduction)
3. Determinism passes at all divisors

**Phase B (MUST PASS):**
1. Drive computation < 2ms at 360K
2. IPC > 1.8 (compute-bound)
3. Emergent behaviors validated (visual confirmation)

### Abort Criteria

**Phase A:**
- If L1 aggregation > 1ms → Reconsider L1 grid design

**Phase B:**
- If drive computation > 3ms → Keep behavior state machine as fallback

**Phase C:**
- (No abort criteria - zero overhead guaranteed by fast path)

---

## Measurement Workflow

### Before Implementation (Baseline)

```bash
# Capture current performance
cd /home/dev/dev/speciate/apps/simulation
perf stat -e cycles,instructions,L1-dcache-load-misses,LLC-load-misses \
  timeout 10s ./target/release/sim_app --creatures 360000

# Save baseline
cp docs/performance/snapshots/NOW.json docs/performance/snapshots/baseline-pre-abc.json
```

### During Implementation (Per Phase)

```bash
# Phase A
./scripts/measure_phase_a.sh
# Review: docs/performance/snapshots/phase-a/summary.md

# Phase C
./scripts/measure_phase_c.sh
# Review: docs/performance/snapshots/phase-c/summary.md

# Phase B
./scripts/measure_phase_b.sh
# Review: docs/performance/snapshots/phase-b/summary.md
```

### After Implementation (Validation)

```bash
# Full spec suite
cargo test --release --package simulation --test specs

# Determinism validation
cargo test test_deterministic_simulation_20k

# Visual smoke test
cd apps/portal && npm run dev
# Spawn 10K creatures, run for 60 seconds, observe behavior
```

---

## Hardware Counter Quick Reference

**IPC (Instructions Per Cycle):**
- Current: 1.68 (memory-bound)
- Target: 1.8-2.0 (Phase A), 2.0+ (Phase B)
- Interpretation: < 1.5 = memory-bound, > 2.5 = compute-bound

**L1D Miss Rate:**
- Current: 3.4% (acceptable)
- Target: < 3% (Phase B), < 1% (Phase A aggregation)
- Interpretation: < 3% = good, > 5% = poor locality

**LLC Miss Rate:**
- Current: 3.0% (high - working set > L3)
- Target: 2.5% (Phase A early-exit benefit)
- Interpretation: < 2% = good, > 5% = heavy DRAM traffic

**Full Reference:** `/home/dev/dev/speciate/docs/performance/analysis/hardware-counter-reference.md`

---

## Dev-UI Instrumentation Needs

### Phase A

**L1 Grid Overlay (Portal):**
- G key cycles: Off → L0 → L1 → Heatmap
- Heatmap: Color intensity = total_mass
- Overlay: L1 cell boundaries (30m grid)

**Metrics (Dev-UI):**
- `l1_aggregation_us`: Time for L0 → L1 reduction
- `early_exit_rate`: % creatures skipping L0 scan
- `l1_cells_non_empty`: Count of populated L1 cells

### Phase C

**Frequency Control Panel (Dev-UI):**
- Slider: `perception_divisor` (1-20)
- Display: Effective Hz = 20 / divisor
- Live update: Sparkline changes immediately
- Show: `creatures_processed_this_tick` / `total_creatures`

### Phase B

**Drive Visualization (Portal):**
- Arrow overlay: Drive direction
- Color code: Magnitude (red=strong, green=weak)
- Debug mode: L1 cells scanned

**Metrics (Dev-UI):**
- `drive_computation_us`: L1 force computation time
- `avg_l1_cells_per_creature`: Average scanned
- `resting_creatures`: Count with zero drive

---

## Collaboration Notes

### For rusty-ron (Simulation Logic Owner)

**Phase A:**
- Implement L1 aggregation system (reduce L0 → L1 every tick)
- Add early-exit check before L0 scan (threshold comparison)
- Add perception_threshold to Perception component

**Phase C:**
- Add FreqConfig resource
- Implement bucketing in perception/behavior/steering systems
- Add fast path for divisor=1 (zero overhead)

**Phase B:**
- Create DriveState component
- Implement L1 drive computation (repulsion + attraction)
- Remove BehaviorMode enum and wandering system

### For ecs-eddy (ECS Architecture Owner)

**Phase A:**
- Validate L1 aggregation archetype impact (new system added)
- Review component layout for perception_threshold

**Phase C:**
- Validate throttling doesn't break determinism
- Review query iteration with bucketing (spatial coherence)

**Phase B:**
- Validate removal of BehaviorMode component
- Review DriveState addition (new component)

### For You (Integration Owner)

**Questions to Ask:**
1. "What is the L1 aggregation time at 360K?" (must be < 0.5ms)
2. "Does early-exit work in sparse scenarios?" (50%+ reduction?)
3. "Is divisor=1 truly zero overhead?" (within 2% of baseline?)
4. "What is the drive computation time?" (must be < 2ms)
5. "Do emergent behaviors look right?" (visual validation)

**Reject If:**
- Any performance gate fails
- Determinism test breaks
- IPC decreases (optimization made things worse)

---

## Final Validation Checklist

Before merging to main:

**Phase A:** ✅ COMPLETE
- [x] L1 aggregation < 0.5ms at 360K
- [x] Early-exit reduces sparse perception by 50%+
- [x] IPC increases from 1.68 → 1.8+
- [x] Determinism test passes
- [x] Portal shows L1 heatmap correctly

**Phase C:** ✅ COMPLETE
- [x] Divisor=1 within 2% of baseline
- [x] Divisor=2 reduces perception by 50%
- [x] Divisor=4 reduces perception by 75%
- [x] Determinism passes at all divisors
- [x] Dev-UI sliders control frequency live

**Phase H (Hierarchical Perception v2):** 🔄 IN PROGRESS
- [ ] L2 grid infrastructure (90m cells)
- [ ] Pattern iteration helper (cells_from_pattern)
- [ ] L2 scan phase with biosig classification
- [ ] Early-exit cascade (L2 → L1 → L0)
- [ ] 70%+ L0 cell reduction in sparse scenarios
- [ ] Determinism test passes

**Phase B:** ⏸️ ON HOLD (pending Phase H)
- [ ] Drive computation < 2ms at 360K
- [ ] IPC > 1.8 (compute-bound)
- [ ] L1D miss rate < 3%
- [ ] Rayon engages 16 cores
- [ ] Creatures disperse (visual)
- [ ] Small avoid large (visual)
- [ ] No jittering at equilibrium (visual)
- [ ] BehaviorMode enum deleted
- [ ] Wandering system deleted

---

## Conclusion

**All three sprints are APPROVED from a performance perspective.**

**Confidence:**
- Phase A: HIGH (simple aggregation, proven early-exit)
- Phase C: HIGH (trivial bucketing, zero-cost guarantee)
- Phase B: MEDIUM-HIGH (need to validate L1 scan, but expected to be compute-bound)

**Expected Result:** 18% tick time reduction with throttling (divisor=2), enabling 500K creatures.

**Critical Success Factor:** Follow measurement scripts, validate gates, reject if performance degrades.

---

**Questions?** Consult the detailed assessment or run measurement scripts.

**Ready to proceed.** 🚀
