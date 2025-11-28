# Sprint 16: Spatial Grid - Rationale

## When to Run This Sprint

Sprint 16 is **CONDITIONAL** based on Sprint 15 validation results.

### Trigger Conditions

**Trigger A - Performance Bottleneck (MANDATORY):**
- Sprint 15 fails to achieve 150K creatures @ <45ms tick time
- **Reason:** Algorithmic complexity (O(N²)) is the bottleneck
- **Action:** Spatial grid is REQUIRED to break O(N²) → O(N×k)

**Trigger B - Scaling Headroom (RECOMMENDED):**
- Perception still >40% of frame budget @ 150K after all Sprint 15 optimizations
- **Reason:** Limited scaling headroom for future features
- **Action:** Spatial grid provides 833x reduction in comparisons

**Trigger C - Stretch Goal (OPTIONAL):**
- Want to achieve 200K+ creatures for marketing/gameplay reasons
- **Reason:** Current architecture tops out at 150K even with all optimizations
- **Action:** Spatial grid enables 200K+ with comfortable headroom

### Skip Conditions

**Skip A - Sprint 15 Exceeded Expectations:**
- Sprint 15 achieves 200K creatures @ <45ms tick time
- Perception <30% of frame budget
- **Decision:** Defer spatial grid to Sprint 17+, focus on features (organic shaders)

**Skip B - Sufficient for Current Milestone:**
- Current milestone only requires 100K creatures
- Sprint 15 comfortably achieves 100K @ <40ms
- **Decision:** Defer optimization, ship gameplay features first

---

## The O(N²) Problem

### Mathematical Reality

**Current Perception Algorithm:**
```
For each creature (N):
    For each other creature (N-1):
        Calculate distance
        Check if in range
        Add to neighbor list
```

**Complexity:** O(N²) - quadratic scaling

**Concrete Numbers:**

| Creatures | Total Comparisons | Sequential Time | 8-Core Parallel | Budget (45ms) |
|-----------|------------------|-----------------|-----------------|---------------|
| 5,000 | 25,000,000 | 34ms | ~5ms | ✅ Fits |
| 10,000 | 100,000,000 | ~136ms | ~17ms | ⚠️ Tight |
| 25,000 | 625,000,000 | ~850ms | ~106ms | ❌ 2.4x over |
| 50,000 | 2,500,000,000 | ~3,400ms | ~425ms | ❌ 9.4x over |
| 100,000 | 10,000,000,000 | ~13,600ms | ~1,700ms | ❌ 38x over |
| 150,000 | 22,500,000,000 | ~30,600ms | ~3,825ms | ❌ **85x over** |

### Why Parallelization Alone Isn't Enough

**Rayon (8 cores) provides 2-4x speedup, not 100x.**

Even with perfect parallelization:
- 150K creatures = 3,825ms (85x over budget)
- Need 85x improvement, not 4x

**Root cause:** Quadratic complexity (N²) scales exponentially, linear speedup (8 cores) can't keep up.

**Solution:** Fix the algorithm FIRST (O(N²) → O(N×k)), THEN parallelize.

---

## Why Spatial Grid is the Solution

### Algorithmic Improvement

**Current:** Check every creature against every other creature
- 150K × 150K = 22.5 billion comparisons

**With Spatial Grid:** Only check creatures in nearby cells
- 150K × 180 avg neighbors = 27 million comparisons
- **Reduction: 833x fewer operations**

### How It Works

**Concept:** Partition the world into 200m × 200m grid cells.

**Query Pattern:**
```
1. Calculate which grid cell you're in: O(1)
2. Check 9 nearby cells (3×3 grid): O(9)
3. Scan ~20 entities per cell: O(180)
4. Distance filter those entities: O(180)

Total: O(1 + 9 + 180 + 180) = O(~370) per creature
vs Current: O(150,000) per creature
```

**Result:** O(N × k) where k ≈ 180, not O(N²)

### Performance Projection

**Spatial Grid + Rayon (8 cores):**

| Creatures | Comparisons | Time | vs O(N²) | Budget Headroom |
|-----------|-------------|------|----------|-----------------|
| 5,000 | 900,000 | ~0.5ms | 68x faster | 90x under budget |
| 25,000 | 4,500,000 | ~2ms | 425x faster | 22x under budget |
| 50,000 | 9,000,000 | ~3ms | 1,133x faster | 15x under budget |
| 150,000 | 27,000,000 | ~7ms | 4,371x faster | **6.4x under budget** |
| 200,000 | 36,000,000 | ~10ms | 8,500x faster | **4.5x under budget** |

**With Sprint 15's stochastic vision (10% per tick):**
- 150K creatures: ~0.7ms perception (**64x headroom!**)
- 200K creatures: ~1.0ms perception (**45x headroom!**)

---

## Alternative Considered: Just Add More Rayon

**Why not just throw more CPU cores at the problem?**

**Math:**
- Current 8-core: 150K @ 3,825ms
- 64-core server: 150K @ ~478ms (still 10.6x over budget!)
- Would need **680 cores** to hit 45ms budget

**Reality:**
- Most consumer CPUs: 4-16 cores
- Server costs prohibitive for game simulation
- Doesn't solve the fundamental algorithmic problem

**Conclusion:** Spatial grid is 10-100x more effective than adding cores.

---

## Sprint 15 vs Sprint 16: What's the Difference?

### Sprint 15 Focus: Zero-Allocation + Cache-Friendly

**Optimizations:**
- Remove Vec allocations in perception (3.2MB/frame → 0)
- Split queries (no borrow-checker Vec collection)
- SIMD Vec2 vector math
- Changed<T> filters (skip unchanged entities)
- Rayon on movement systems (not perception)
- Stochastic vision updates (10% per tick)

**Expected Gain:** 5K → 150K creatures (30x capacity)
**Complexity:** Still O(N²), but with all constant-factor improvements applied

### Sprint 16 Focus: Algorithmic Complexity

**Optimization:**
- Spatial grid: O(N²) → O(N×k)
- Rayon on perception (now that complexity is fixed)

**Expected Gain:** 150K → 200K+ creatures (40x capacity vs baseline)
**Complexity:** O(N×k) - linear scaling instead of quadratic

### Why Not Both in One Sprint?

1. **Validation:** Need to measure Sprint 15 results before committing to Sprint 16
2. **Risk Management:** If Sprint 15 achieves 200K, spatial grid is unnecessary complexity
3. **Focus:** Each sprint has clear, testable goal (zero-allocation vs algorithmic change)
4. **TDD Workflow:** Smaller, incremental changes easier to test and validate

---

## Decision Timeline

### End of Sprint 15 (Phase 2D Validation)

**Measure:**
- Max creatures @ <45ms tick time
- Perception % of frame budget
- Hardware metrics (IPC, cache hit rates, CPU utilization)

**Decide:**

| Sprint 15 Result | Decision | Rationale |
|-----------------|----------|-----------|
| <100K @ 45ms | ❌ → Sprint 16 MANDATORY | Critical blocker, must fix algorithm |
| 100-150K @ 45ms | ⚠️ → Sprint 16 RECOMMENDED | Meets minimum, but tight for features |
| 150-200K @ 45ms | ✅ → Sprint 16 OPTIONAL | Good headroom, can defer if needed |
| >200K @ 45ms | 🎉 → Defer Sprint 16 | Exceeded goal, focus on gameplay |

**Additional Factors:**
- Perception >40% of frame? → Lean toward Sprint 16
- Perception <30% of frame? → Lean toward deferring
- Upcoming features CPU-heavy? → Need headroom, run Sprint 16

---

## What Happens if We Skip Sprint 16?

### Scenario A: Sprint 15 Success (150K+)

**Pros:**
- Faster time to market (ship features instead)
- Simpler codebase (no spatial grid complexity)
- Sufficient for initial launch (150K is impressive)

**Cons:**
- Hard cap at ~150K creatures (can't scale beyond)
- Tight CPU budget for future features
- May need Sprint 16 later anyway if adding CPU-heavy features

**Recommendation:** Ship features, monitor performance, revisit in 2-3 sprints if needed.

### Scenario B: Sprint 15 Partial (100-150K)

**Pros:**
- Meets minimum viable scale
- Can still ship with 100K creatures

**Cons:**
- Limited headroom for future features
- May struggle with complex scenarios (dense populations)
- Will likely need Sprint 16 before major features

**Recommendation:** Evaluate vs roadmap - if next 3 sprints are UI/gameplay (low CPU), defer. If adding features like combat/reproduction (high CPU), run Sprint 16 now.

### Scenario C: Sprint 15 Underperforms (<100K)

**Pros:** (None - this is a blocker)

**Cons:**
- Can't meet scale requirements
- Perception still dominates frame budget
- No headroom for features

**Recommendation:** Sprint 16 is MANDATORY. Algorithmic bottleneck must be fixed.

---

## Long-Term Architecture

### Current Plan (Sprints 15-16)

```
Baseline: 5K creatures @ 50ms (O(N²) naive)
    ↓
Sprint 15: 150K creatures @ 45ms (O(N²) optimized)
    ↓
Sprint 16: 200K creatures @ 45ms (O(N×k) spatial grid + Rayon)
```

### Future Scaling (Sprint 17+)

**If spatial grid + Rayon still insufficient:**

**Sprint 17: Stochastic Grid Updates**
- 10% creatures update vision per tick
- 200K × 10% × 180 neighbors = 3.6M comparisons
- Target: 500K+ creatures

**Sprint 18: Hierarchical Grid (Quadtree)**
- Adaptive cell sizing based on density
- Better for massive worlds (10,000km × 10,000km)
- Target: 1M+ creatures (regional simulation)

**Sprint 19: GPU Acceleration**
- Compute shader for perception
- Entire system parallel on GPU
- Target: 5M+ creatures (MMO-scale)

**Current Phase:** We're at Sprint 15-16 (foundation). Future optimizations build on this.

---

## References

**Architecture:**
- `docs/architecture/spatial-partitioning.md` - Full specification
- `SPRINT_16_PLAN/SPRINT_PLAN_sprint-16-spatial-grid.md` - Implementation plan

**Sprint Context:**
- `SPRINT_DOCS/SPRINT_PLAN_sprint-15-ecs-optimizations.md` - Prerequisite sprint

**Performance:**
- `docs/performance/optimization-backlog.md` - Ongoing tracking

---

## Summary

**Sprint 16 is for when Sprint 15 isn't enough.**

- **Trigger:** Perception bottleneck remains after all zero-allocation optimizations
- **Goal:** Break O(N²) → O(N×k) via spatial grid
- **Result:** 150K-200K creatures with comfortable headroom
- **Effort:** 5 days (TDD implementation + validation)

**Decision is data-driven:** Wait for Sprint 15 Phase 2D validation results, then decide based on actual measurements, not projections.
