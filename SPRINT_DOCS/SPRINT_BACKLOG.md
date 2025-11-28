# Sprint 15: ECS Optimizations - Backlog

**Branch:** `feat/sprint-15-ecs-optimizations`
**Status:** IN PROGRESS
**Duration:** 6 days

---

## Sprint Goal

Scale backend ECS simulation to 150K-200K creatures through:
1. **Uber-struct pattern** (stable archetypes, hot/cold split, cache-friendly)
2. **Vision system refactor** (remove Vec allocation bottleneck, FOV, stochastic updates)
3. **Vec2 vector math** (SIMD optimization)
4. **Parallelization** (multi-core utilization)

---

## Baseline Metrics (Actual @ 5K Active Wanderers)
**Source:** `docs/performance/snapshots/5k_wanderers_2025-11-28T14-33-50.json`

| Metric | Current Value | Problem |
|--------|---------------|---------|
| Total tick time | **50ms** | **AT BUDGET LIMIT** |
| Perception system | **34ms (67%)** | **CRITICAL BOTTLENECK** |
| Movement systems | 13ms (26%) | Acceptable |
| Avoidance | 3.4ms (7%) | Acceptable |
| Vec allocations | 3.2MB/frame | 64MB/sec memory churn |
| Max active creatures | **~5K** | Limited by O(N²) perception |
| CPU utilization | 17% (1 core) | 7 cores idle |
| IPC | 3.42 | Good (hardware efficient) |

---

## Phase 1: Archetype Churn Trial (Day 1 - VALIDATION)

### Purpose
Validate whether archetype fragmentation is a measurable performance problem before investing in uber-struct refactor.

### Trial Design
| Scenario | Description | Expected Outcome |
|----------|-------------|------------------|
| A (Stable) | 2.5K wanderers, no behavior changes | Baseline metrics |
| B (Churning) | 2.5K creatures, constant behavior transitions | Measure degradation |

### Metrics to Capture
- Tick time (ms)
- Archetype count (should grow in B if fragmentation)
- IPC (should drop in B if cache thrashing)
- L1/L2 cache miss rates

### Decision Point
- **B >> A (>20% slower):** Proceed to Phase 1b (uber-struct)
- **B ≈ A (<10% difference):** Skip to Phase 2A (vision optimization)

### Tasks
- [ ] Design Scenario A: stable 2.5K wanderers
- [ ] Design Scenario B: churning behavior mechanism (user to specify)
- [ ] Run Scenario A, capture snapshot
- [ ] Run Scenario B, capture snapshot
- [ ] Compare metrics, make decision

**Owner:** ecs-emma + instrumentation-ian

---

## Phase 1b: Uber-Struct Refactor (Day 2 - CONDITIONAL)

**Only proceed if Phase 1 trial shows >20% degradation.**

### Expected Metrics
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Tick time @ 5K | 50ms | 45ms | 5-10% |
| Active creature capacity | 5K | 6K | +20% |
| L1 cache hit rate | Baseline | +5-10% | Measurable |
| Archetype fragmentation | High | Stable | Eliminated |

**Note:** Modest gains. Real wins come from Phase 2 (vision optimization).

### Tasks
- [ ] Design uber-struct component architecture
- [ ] Implement `PhysicalTraits` (size, speed, color)
- [ ] Implement `BehaviorState` (current behavior, timers)
- [ ] Remove `Catatonic` component → `BehaviorMode::Catatonic` enum
- [ ] Migrate existing component-per-field to uber-structs
- [ ] Verify stable archetype (no add/remove component churn)
- [ ] Run benchmarks to validate cache locality improvement
- [ ] **BENCHMARK:** Capture tick time @ 5K before/after

**Owner:** ecs-emma + rusty-ron
**Validation:** instrumentation-ian (cache metrics)

---

## Phase 2A: Vision Split Queries (Day 3 - CRITICAL)

### Expected Metrics
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Vec allocations/frame | 3.2MB | 0 bytes | **100% eliminated** |
| Perception time @ 5K | 34ms | ~15ms | **50-60% reduction** |
| Perception time @ 20K | N/A (>100ms) | ~25ms | Now possible |
| Active creature capacity | 6K | 15-20K | **3x increase** |
| Memory churn | 64MB/sec | 0 | GC eliminated |

**CRITICAL:** This is the single most important optimization.

### Tasks
- [ ] Rename `Perception` → `Vision` (biological naming)
- [ ] Add `VisionTiming` component (stochastic updates)
- [ ] Add `Visible` marker component (zero-cost filter)
- [ ] **CRITICAL:** Remove `.collect()` Vec allocation
- [ ] Implement split queries (observers mut, targets immut)
- [ ] Implement FOV dot product check (blind spots)
- [ ] Add stochastic vision updates (reaction time gating)
- [ ] **BENCHMARK:** Verify zero allocations with profiler
- [ ] **BENCHMARK:** Vision time @ 50K, 100K creatures

**Owner:** ecs-emma + rusty-ron
**Validation:** instrumentation-ian, zoologist-tom (FOV biology)

---

## Phase 2B: Changed<T> Filters + Vec2 (Day 4)

### Expected Metrics
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Rotation iterations @ 20K | 20K/frame | 2-4K/frame | **80-90% reduction** |
| Vector math speed | Scalar f32 | SIMD Vec2 | 10-20% faster |
| Tick time @ 20K | ~35ms | ~32ms | 2-3ms saved |
| Active creature capacity | 15-20K | 25K | +25% |

### Tasks
- [ ] Add `Changed<Velocity>` filter to rotation system
- [ ] Audit all systems for `Changed<T>` opportunities
- [ ] Migrate `Position { x, y }` → `Position(Vec2)`
- [ ] Migrate `Velocity { vx, vy }` → `Velocity(Vec2)`
- [ ] Migrate `Acceleration { ax, ay }` → `Acceleration(Vec2)`
- [ ] Update all vector math to use glam Vec2
- [ ] Verify SIMD optimization in compiler output
- [ ] **BENCHMARK:** Compare rotation system iterations before/after
- [ ] **BENCHMARK:** Vector operation microbenchmarks

**Owner:** rusty-ron + ecs-emma

---

## Phase 2C: Parallelization (Day 5)

### Expected Metrics
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| CPU cores utilized | 1 | 4-8 | **4-8x potential** |
| Realistic speedup | 1x | 2-3x | Amdahl's law |
| Movement systems time @ 25K | ~13ms | ~5ms | 5-8ms saved |
| Tick time @ 25K | ~32ms | ~25ms | 20% reduction |
| Active creature capacity | 25K | 40-50K | **+60-100%** |

**Note:** Parallelization helps movement but NOT perception (O(N²)).

### Tasks
- [ ] Analyze system dependencies for parallel execution
- [ ] Add `par_iter_mut()` to rotation system
- [ ] Add `par_iter_mut()` to seek system
- [ ] Add `par_iter_mut()` to wander system
- [ ] Replace `thread_rng()` with `fastrand` (thread-safe)
- [ ] Test determinism (parallel order shouldn't matter)
- [ ] **BENCHMARK:** Measure speedup on 4-core, 8-core CPUs
- [ ] Profile for race conditions

**Owner:** rusty-ron + instrumentation-ian

**Cannot parallelize (entity lookups):**
- Vision system (O(N²) interactions)
- Avoidance system (queries other entities)

---

## Phase 2D: Stochastic Vision + Validation (Day 6)

### Expected Metrics After Stochastic Vision
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Vision updates/tick | 100% creatures | ~10% creatures | **10x reduction** |
| Perception time @ 50K | ~25ms | ~3ms | **90% reduction** |
| Active creature capacity | 40-50K | 100-200K | **2-4x increase** |

### Target Metrics (Final)
| Active Creatures | Target Tick | Perception Budget | Confidence |
|-----------------|-------------|-------------------|------------|
| 5K | <25ms | <10ms | 99% |
| 20K | <35ms | <15ms | 90% |
| 50K | <45ms | <20ms | 75% |
| 100K | <50ms | <25ms | 50% |
| 200K | <50ms | <30ms | 30% (stretch) |

### Tasks
- [ ] **BENCHMARK:** Full simulation @ 50K (baseline comparison)
- [ ] **BENCHMARK:** Full simulation @ 100K
- [ ] **BENCHMARK:** Full simulation @ 150K (primary target)
- [ ] **BENCHMARK:** Full simulation @ 200K (stretch goal)
- [ ] Capture hardware metrics snapshot (IPC, cache rates)
- [ ] Validate 22.2Hz tick rate maintained
- [ ] Profile remaining bottlenecks
- [ ] Generate performance report
- [ ] Update `docs/performance/optimization-backlog.md`

**Owner:** instrumentation-ian + ecs-emma

---

## Success Metrics Summary

### Performance KPIs
| KPI | Baseline (5K) | Target | Phase |
|-----|---------------|--------|-------|
| Max active creatures | 5K | 50-100K | All |
| Perception % of frame | 67% | <50% | 2A+2D |
| Vec allocations/frame | 3.2MB | 0 | 2A |
| Perception time @ 5K | 34ms | <10ms | 2A+2D |
| CPU utilization | 17% | 50%+ | 2C |
| IPC | 3.42 | >3.5 | 1+2 |

### Cumulative Capacity by Phase (Active Creatures)
```
Baseline:     ██ 5K (at budget limit!)
Phase 1:      ██ 6K (+20%)
Phase 2A:     ████████ 15-20K (+200%) ← CRITICAL
Phase 2B:     ██████████ 25K (+25%)
Phase 2C:     ████████████████ 40-50K (+60-100%)
Phase 2D:     ████████████████████████████████████████ 100-200K (+100-300%) ← Stochastic vision
```

---

## Biological Validation (zoologist-tom)

### Size-Based Reaction Times
| Creature Size | Reaction Time | Updates/sec | Behavior |
|---------------|---------------|-------------|----------|
| 0.5m (small) | 68ms | ~15/sec | Twitchy prey |
| 1.0m (medium) | 100ms | ~10/sec | Baseline |
| 5.0m (large) | 500ms | ~2/sec | Deliberate |
| 10.0m (huge) | 1632ms | ~0.6/sec | Ponderous |

### FOV Validation
- [ ] Verify blind spots (entities outside FOV not detected)
- [ ] Test predator sneaking from behind
- [ ] Balance FOV width (start 180°, tune from gameplay)

---

## Notes

- Sprint 14 delivered GPU interpolation (165 FPS achieved)
- Frontend ready for high entity counts (200K+)
- Backend is the bottleneck (vision Vec allocations)
- Focus: zero-allocation, cache-friendly, parallel architecture
