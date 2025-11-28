# Sprint 15: ECS Optimizations & Backend Performance

**Branch:** `feat/sprint-15-ecs-optimizations` (to be created)
**Status:** PLANNED
**Prerequisites:** Sprint 14 complete (Frontend GPU Interpolation)
**Duration:** 6 days

---

## Sprint Goal

Scale backend ECS simulation to 150K-200K creatures through:
1. **Uber-struct pattern** (stable archetypes, hot/cold split, cache-friendly)
2. **Vision system refactor** (remove Vec allocation bottleneck, FOV, stochastic updates)
3. **Vec2 vector math** (SIMD optimization)
4. **Parallelization** (multi-core utilization)

**Key Architecture:**
- Stable ECS archetypes (no add/remove component churn)
- Zero allocations in vision system
- Component-based timing (10-100x faster than HashMap)
- Per-creature reaction times (natural load distribution)

---

## Team

**ECS Optimization Lead:**
- **ecs-emma** - ECS architecture, data-oriented design, performance profiling
- **rusty-ron** - Backend implementation, Bevy ECS systems
- **instrumentation-ian** - Performance analysis, hardware profiling
- **architect-andy** - Architecture validation, technical standards
- **zoologist-tom** - Biological validation (FOV, reaction times)
- **pm-pam** - Sprint coordination, task breakdown

---

## Phase Overview

1. **Phase 1:** Archetype Churn Trial (Day 1 - VALIDATION)
2. **Phase 1b:** Uber-Struct Refactor (Day 2 - IF trial shows impact)
3. **Phase 2A:** Vision Split Queries (Day 3 - CRITICAL)
4. **Phase 2B:** Changed<T> Filters + Vec2 (Day 4)
5. **Phase 2C:** Parallelization (Day 5)
6. **Phase 2D:** Stochastic Vision + Performance Validation (Day 6)

---

## Performance Metrics Summary

### Baseline (Actual @ 5K Active Wanderers)
**Source:** `docs/performance/snapshots/5k_wanderers_2025-11-28T14-33-50.json`

| Metric | Value | Notes |
|--------|-------|-------|
| Total tick time | **50ms** | **AT BUDGET LIMIT** |
| Perception system | **34ms (67%)** | **CRITICAL BOTTLENECK** |
| Movement systems | 13ms (26%) | Acceptable |
| Avoidance | 3.4ms (7%) | Acceptable |
| Rotation | 0.1ms | Trivial |
| Max active creatures | **~5K** | Limited by perception O(N²) |
| IPC | 3.42 | Good (hardware efficient) |
| CPU utilization | 17% | 7 cores idle |

### The Problem: O(N²) Vision System

At 5K creatures, perception takes 34ms. This scales **quadratically**:
- 5K creatures → 34ms (5K × 5K = 25M comparisons)
- 10K creatures → ~136ms (theoretically, 4x)
- 20K creatures → ~544ms (theoretically, 16x)

**Current architecture cannot scale.** Split queries + stochastic updates are mandatory.

### Expected Gains by Phase

| Phase | Optimization | Expected Gain | Cumulative Capacity |
|-------|-------------|---------------|---------------------|
| **Baseline** | - | 50ms @ 5K | 5K active |
| **Phase 1** | Uber-struct (cache locality) | 5-10% tick reduction | 6K active |
| **Phase 2A** | Vision split queries (zero alloc) | **50-70% perception reduction** | 15-20K active |
| **Phase 2B** | Changed<T> + Vec2 SIMD | 2-3ms saved | 25K active |
| **Phase 2C** | Parallelization (8-core) | 2-3x on movement | 40-50K active |
| **Phase 2D** | Stochastic vision (10% per tick) | **10x perception reduction** | 150-200K active |

### Target Metrics (End of Sprint)

| Active Creatures | Target Tick Time | Perception Budget | Confidence |
|-----------------|------------------|-------------------|------------|
| 5K | <25ms | <10ms | 99% |
| 20K | <35ms | <15ms | 90% |
| 50K | <45ms | <25ms | 75% |
| 100K | <50ms | <30ms | 50% |
| 200K | <50ms | <30ms | 30% (stretch) |

### Key Performance Indicators (KPIs)

| KPI | Before (5K) | After (50K) | Target |
|-----|-------------|-------------|--------|
| Perception % of frame | 67% | <50% | ✓ |
| Vec allocations/frame | 3.2MB | 0 bytes | ✓ |
| Perception time | 34ms | <25ms @ 50K | ✓ |
| CPU core utilization | 17% (1 core) | 50%+ (4-8 cores) | ✓ |
| Max active creatures | 5K | 50-100K | ✓ |
| IPC | 3.42 | >3.5 | ✓ |

---

## Phase 1: Archetype Churn Trial (VALIDATION)

**Duration:** Day 1

**Goal:** Validate whether archetype fragmentation is actually a measurable performance problem before investing in uber-struct refactor.

### Trial Design

**Scenario A - Baseline (Stable Archetypes):**
- Spawn 2.5K wandering creatures
- All creatures maintain stable component composition
- No behavior transitions (no add/remove component operations)
- Capture performance snapshot

**Scenario B - Churning (Unstable Archetypes):**
- Spawn 2.5K creatures with constant behavior changes
- Creatures continuously transition between behavior states
- Forces frequent add/remove component operations
- Capture performance snapshot

### Metrics to Compare

| Metric | Scenario A (Stable) | Scenario B (Churning) | Decision |
|--------|---------------------|----------------------|----------|
| Tick time | Baseline | If significantly higher | Uber-struct needed |
| Archetype count | Stable | If growing | Memory fragmentation |
| IPC | Baseline | If lower | Cache thrashing |
| L1/L2 miss rate | Baseline | If higher | Cache pollution |

### Decision Point

- **If B >> A (>20% slower):** Proceed to Phase 1b (uber-struct refactor)
- **If B ≈ A (<10% difference):** Skip Phase 1b, proceed to Phase 2A (vision optimization)

### Implementation Notes

User to determine mechanism for forcing behavior changes in Scenario B. Options:
- Timer-based behavior transitions
- Forced state cycling (Wandering → Idle → Wandering)
- Component add/remove stress test

---

## Phase 1b: Uber-Struct Refactor (CONDITIONAL)

**Duration:** Day 2 (only if Phase 1 trial shows significant impact)

**Goal:** Stable ECS archetypes (no add/remove component churn → cache-friendly)

### Expected Metrics After Phase 1b
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Tick time @ 5K | 50ms | 45ms | 5-10% |
| Active creature capacity | 5K | 6K | +20% |
| L1 cache hit rate | Baseline | +5-10% | Measurable |
| Archetype fragmentation | High | Stable | Eliminated |

**Note:** Phase 1b provides modest gains. The real wins come from Phase 2 (vision optimization).

### Remove Catatonic Component

**Before (Component churn):**
```rust
// BAD: Adding/removing components fragments memory
commands.entity(entity).insert(Catatonic);
commands.entity(entity).remove::<Catatonic>();
```

**After (Enum in uber-struct):**
```rust
#[derive(Component)]
pub struct CreatureState {
    pub behavior: BehaviorMode,  // Enum includes Catatonic
    pub health: Health,
    pub energy: Energy,
    // ... other "warm" data (accessed frequently)
}

pub enum BehaviorMode {
    Idle,
    Wandering,
    Fleeing,
    Seeking,
    Catatonic,  // Now just an enum variant
}
```

### Hot/Cold Data Split

**Hot (every frame):**
```rust
#[derive(Component)]
pub struct Transform {
    pub position: Vec2,    // SIMD-optimized
    pub rotation: f32,
}

#[derive(Component)]
pub struct Physics {
    pub velocity: Vec2,    // SIMD-optimized
    pub acceleration: Vec2,
}
```

**Cold (infrequent):**
```rust
#[derive(Component)]
pub struct BiologyData {
    pub dna: DNA,
    pub age: f32,
    pub lineage: LineageId,
}
```

**Benefits:**
- Hot data in contiguous arrays → CPU prefetches correctly
- Cold data separate → doesn't pollute cache
- Stable archetypes → no fragmentation over time

**Success:** Benchmarks show improved L1/L2 cache hit rates

---

## Phase 2: Vision System + Comprehensive ECS Optimization

**Duration:** Days 3-5 (expanded from 2 days for thorough ECS audit)

**Goal:** Transform perception → vision WITH comprehensive ECS optimization across ALL systems

### Expected Metrics After Phase 2A (Vision Split Queries)
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Vec allocations/frame | 3.2MB | 0 bytes | **100% eliminated** |
| Perception time @ 5K | 34ms | ~15ms | **50-60% reduction** |
| Perception time @ 20K | N/A (>100ms) | ~25ms | Now possible |
| Active creature capacity | 6K | 15-20K | **3x increase** |
| Memory churn | 64MB/sec | 0 | GC eliminated |

**Critical:** This is the single most important optimization. Without it, nothing else matters.

### Critical Context

**Current bottleneck identified:** Perception system allocates **3.2MB Vec every frame** @ 200K creatures
```rust
// CURRENT INEFFICIENT PATTERN (apps/simulation/src/simulation/perception/systems.rs:17-20)
let creatures: Vec<(Entity, Position, BodySize)> = query
    .iter()
    .map(|(entity, pos, size, _, _)| (entity, *pos, *size))
    .collect();  // ❌ 3.2MB allocation, 20x per second = 64MB/sec
```

**Why this exists:** Bevy borrow-checker prevents simultaneous mutable + immutable access to same query

**The solution:** Split queries (different component sets = no borrow conflict)

### Phase 2A: Vision Split Queries (Day 3 - CRITICAL)

**Morning: Rename & Add Components**

1. Rename `Perception` → `Vision` (biological naming)
   ```rust
   // apps/simulation/src/simulation/vision/components.rs
   pub struct Vision {
       range: f32,           // Sight distance
       fov: f32,             // Field of view (radians, e.g., PI)
       nearby: Vec<Entity>,  // Visible entities
   }
   ```

2. Add `VisionTiming` component (stochastic updates)
   ```rust
   #[derive(Component)]
   pub struct VisionTiming {
       pub reaction_time_ms: u16,  // 50-5000ms (size-based)
       pub last_update: f64,        // f64 for precision
       pub spawn_offset: f32,       // Random -1.0 to 0.0
   }
   ```

3. Add `Visible` marker component (zero-cost filter)
   ```rust
   #[derive(Component, Default)]
   pub struct Visible;  // Zero-sized type (no memory cost)
   ```

**Afternoon: CRITICAL FIX - Split Queries**

4. **Remove Vec collection** (THE KEY OPTIMIZATION):
   ```rust
   // ❌ DELETE THIS (apps/simulation/src/simulation/perception/systems.rs:17-20)
   let creatures: Vec<(Entity, Position, BodySize)> = query.iter().collect();

   // ✅ REPLACE WITH SPLIT QUERIES
   pub fn update_vision_system(
       sim_time: Res<SimulationTime>,
       // Observers: mutable (write to Vision/VisionTiming)
       mut observers: Query<(
           Entity,
           &Position,
           &Rotation,
           &BodySize,
           &mut Vision,
           &mut VisionTiming,
           &CreatureState,
       )>,
       // Targets: immutable (read Position/BodySize only)
       targets: Query<(Entity, &Position, &BodySize), With<Visible>>,
   ) {
       // NO .collect() - direct ECS iteration!
       // Zero allocations, Bevy handles archetype queries efficiently
   }
   ```

5. **Implement FOV dot product check** (blind spots)

```rust
pub fn update_vision_system(
    sim_time: Res<SimulationTime>,
    mut observers: Query<(
        Entity,
        &Position,
        &Rotation,
        &BodySize,
        &mut Vision,
        &mut VisionTiming,
        &CreatureState,
    )>,
    targets: Query<(Entity, &Position, &BodySize), With<Visible>>,
) {
    let current_time = sim_time.elapsed_seconds();

    // ✅ NO .collect() CALL - iterate observers mutably
    for (me, my_pos, my_rot, my_size, mut vision, mut timing, state) in observers.iter_mut() {
        // Skip inactive creatures
        if !state.behavior.is_active() {
            continue;
        }

        // Stochastic gating: only update if reaction time elapsed
        let reaction_sec = timing.reaction_time_ms as f64 / 1000.0;
        let effective_last = timing.last_update + timing.spawn_offset as f64;
        if (current_time - effective_last) < reaction_sec {
            continue;  // Skip (load balancing)
        }

        vision.nearby.clear();
        let self_radius = my_size.radius();

        // ✅ Direct iteration over targets (no allocation)
        for (target_entity, target_pos, target_size) in targets.iter() {
            if target_entity == me { continue; }

            // Distance check with body size consideration
            let dx = target_pos.x - my_pos.x;
            let dy = target_pos.y - my_pos.y;
            let center_dist_sq = dx * dx + dy * dy;

            let other_radius = target_size.radius();
            let combined_radii = self_radius + other_radius;

            if center_dist_sq > (vision.range + combined_radii).powi(2) {
                continue;  // Too far
            }

            // FOV check (dot product blind spot)
            let to_target = Vec2::new(dx, dy);
            let forward = Vec2::new(my_rot.cos(), my_rot.sin());
            let dot = forward.dot(to_target.normalize());
            let angle = dot.acos();

            if angle > vision.fov / 2.0 {
                continue;  // Outside field of view
            }

            vision.nearby.push(target_entity);
        }

        timing.last_update = current_time;
        timing.spawn_offset = 0.0;  // Clear after first update
    }
}
```

**Evening: Testing & Validation**

6. Benchmark @ 50K, 100K creatures
7. Verify zero allocations with profiler
8. Test FOV blind spots (predator sneaking)

**Expected Impact:**
- **Before:** 3.2MB allocation every frame (64MB/sec @ 20Hz)
- **After:** Zero allocations, better cache locality
- **Gain:** 8-15ms per frame @ 200K creatures

---

### Phase 2B: Changed<T> Filters + Vec2 Migration (Day 4)

### Expected Metrics After Phase 2B
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Rotation iterations @ 20K | 20K/frame | 2-4K/frame | **80-90% reduction** |
| Vector math operations | Scalar f32 | SIMD Vec2 | 10-20% faster |
| Tick time @ 20K | ~35ms | ~32ms | 2-3ms saved |
| Active creature capacity | 15-20K | 25K | +25% |

**Morning: Add Changed Filters**

**Issue:** Systems process ALL entities even when nothing changed

1. **Rotation system optimization:**
   ```rust
   // Before: processes all 200K creatures every frame
   pub fn rotation_system(
       mut query: Query<(&mut Rotation, &Velocity)>,
   )

   // After: only creatures where Velocity changed
   pub fn rotation_system(
       mut query: Query<(&mut Rotation, &Velocity), Changed<Velocity>>,
   ) {
       // 80-90% reduction in iterations
   }
   ```

2. **Audit all systems for Changed opportunities:**
   - Integration motion: `Changed<Acceleration>`?
   - Behavior transition: `Or<(Changed<Vision>, Changed<CreatureState>)>`?

**Afternoon: Vec2 SIMD Migration**

3. Replace all raw f32 pairs with Bevy Vec2:
   ```rust
   // Before (verbose, no SIMD)
   pub struct Position { pub x: f32, pub y: f32 }
   let dx = other.x - self.x;
   let dy = other.y - self.y;
   let distance = (dx * dx + dy * dy).sqrt();

   // After (concise, SIMD-optimized)
   pub struct Position(pub Vec2);
   let to_target = other.0 - self.0;
   let distance = to_target.length();  // CPU does X+Y in one cycle
   ```

4. **Files to migrate:**
   - `apps/simulation/src/simulation/components.rs`
   - `apps/simulation/src/simulation/movement/*.rs`
   - `apps/simulation/src/simulation/vision/*.rs`

**Evening: Testing**

5. Run all tests (verify Vec2 conversions work)
6. Benchmark vector math (confirm SIMD speedup)
7. Check compiler output for vectorization

**Expected Impact:**
- Changed filters: 3-5ms reduction @ 200K
- Vec2 SIMD: 10-20% speedup on vector operations

---

### Phase 2C: Parallelization (Day 5)

### Expected Metrics After Phase 2C
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| CPU cores utilized | 1 | 4-8 | **4-8x potential** |
| Realistic speedup | 1x | 2-3x | Amdahl's law |
| Movement systems time @ 25K | ~13ms | ~5ms | 5-8ms saved |
| Tick time @ 25K | ~32ms | ~25ms | 20% reduction |
| Active creature capacity | 25K | 40-50K | **+60-100%** |

**⚠️ CRITICAL WARNING: Rayon Cannot Fix O(N²) Perception**

**The Math:**
| Creatures | Comparisons | Sequential | Parallel (8 cores) | 45ms Budget |
|-----------|-------------|------------|--------------------| ------------|
| 5K | 25M | 34ms | ~5ms | ✅ Fits |
| 25K | 625M | ~850ms | ~106ms | ❌ Exceeds |
| 150K | 22.5B | ~30,600ms | ~3,825ms | ❌ Catastrophic |

**Even with perfect 8-core parallelization, 150K creatures would take 3,825ms (85x over budget).**

**Root Cause:** The perception system is O(n²). Rayon gives you a 2-4x multiplier, but doesn't change algorithmic complexity.

**Solution:** Spatial grid FIRST (O(n²) → O(n×k)), THEN Rayon on top. See Phase 3 (deferred to Sprint 16+).

**Phase 2C Scope (Movement Systems Only):**
This phase parallelizes **pure steering systems** (seek, flee, wander, rotation) which DO benefit from Rayon because they're independent per-entity calculations. Perception remains sequential O(n²) until spatial grid is implemented.

**Morning: Add par_iter_mut() to Pure Systems**

**Current state:** ZERO systems use parallelization

**Candidates (safe to parallelize):**

1. **Rotation system** (pure math, no lookups):
   ```rust
   use rayon::prelude::*;

   pub fn rotation_system(
       mut query: Query<(&mut Rotation, &Velocity), Changed<Velocity>>,
   ) {
       query.par_iter_mut().for_each(|(mut rotation, velocity)| {
           if velocity.vx != 0.0 || velocity.vy != 0.0 {
               rotation.radians = velocity.vy.atan2(velocity.vx);
           }
       });
   }
   ```

2. **Seek system** (independent steering):
   ```rust
   pub fn seek_system(
       mut query: Query<(...), With<CanSeek>>,
   ) {
       query.par_iter_mut().for_each(|(pos, mut accel, ...)| {
           // Pure steering math, no entity lookups
       });
   }
   ```

3. **Wander system** (territory navigation):
   ```rust
   pub fn territory_wandering_system(
       mut query: Query<(...), With<CanWander>>,
   ) {
       // Replace thread_rng() with fastrand (thread-safe)
       query.par_iter_mut().for_each(|(mut accel, mut wander, ...)| {
           let random_offset = fastrand::f32();  // Thread-safe RNG
           // ... rest of logic
       });
   }
   ```

**Afternoon: Testing & Benchmarking**

4. Test determinism (parallel order shouldn't matter)
5. Benchmark 4-core, 8-core, 16-core CPUs
6. Add `#[cfg(feature = "parallel")]` for debugging if needed

**Evening: Integration**

7. Run full simulation @ 150K creatures
8. Measure actual speedup (expect 2-3x on 8-core)
9. Profile for race conditions (unlikely with pure steering)

**Expected Impact:**
- Theoretical: 3-4x on 8-core (75% parallel efficiency)
- Realistic: 2-3x (overhead + Amdahl's law)
- Gain: 5-10ms per frame @ 200K creatures

**Cannot parallelize:**
- Vision system (O(N²) interactions, sequential)
- Avoidance system (queries other entities)
- Integrate motion (boundary enforcement needs care)

---

### Phase 2D: Performance Validation (Day 6)

**Morning: Benchmarks**

1. Run @ 50K creatures (baseline)
2. Run @ 100K creatures (midpoint)
3. Run @ 150K creatures (target)
4. Run @ 200K creatures (stretch)

**Afternoon: Hardware Metrics**

5. Capture snapshots with cockpit:
   - IPC improvement (1.2 → 1.5+?)
   - L1/L2 cache miss rate reduction
   - Frame time breakdown by system

6. Analyze bottlenecks:
   - Is vision still >40% of frame budget?
   - Which system is now the bottleneck?
   - Do we need spatial grid (Sprint 16)?

**Evening: Documentation**

7. Update `docs/performance/optimization-backlog.md`
8. Write Phase 2 completion report
9. Document remaining bottlenecks for Sprint 16

**Success Criteria:**
- ✅ Zero Vec allocations (profiler verified)
- ✅ 80%+ reduction in rotation iterations (Changed<T>)
- ✅ 2x+ speedup on parallel systems (8-core)
- ✅ Vision <40% frame budget (was 70%)
- ✅ 150K creatures @ <45ms tick
- ✅ 200K creatures @ <50ms tick (stretch)

---

### ECS Optimization Summary

**What We're Fixing:**

| Issue | Current | Optimized | Impact |
|-------|---------|-----------|--------|
| Vec allocation | 3.2MB/frame | 0 bytes | 8-15ms @ 200K |
| Rotation waste | 200K/frame | 20K-40K/frame | 3-5ms @ 200K |
| Single-core | 1 thread | 8 threads | 5-10ms @ 200K |
| Scalar math | f32 pairs | Vec2 SIMD | 10-20% speedup |

**Total Expected Gain:** 30-40% frame time reduction @ 200K creatures

**Biological Benefits:**
- 0.5m creature: 68ms reaction → ~15 updates/sec (prey reflexes)
- 1m creature: 100ms reaction → ~10 updates/sec (baseline)
- 10m creature: 1632ms reaction → ~0.6 updates/sec (ponderous)
- FOV blind spots enable sneaking gameplay

---

## Phase 3: Final Validation

**Duration:** Day 6 (combined with Phase 2D)

**Goal:** 150K-200K creatures achieved with all ECS optimizations

### Benchmarks

**Baseline (20K):**
- Tick time: <30ms avg (well under 45ms budget @ 22.2Hz)
- Vision: ~10ms (5-20% creatures per tick)
- Movement: ~8ms
- Frontend: 60 FPS stable (from Sprint 14)

**Target (150K):**
- Tick time: <45ms avg (at 22.2Hz budget)
- Vision: ~30ms (staggered updates = 7.5x fewer per tick)
- Movement: ~12ms
- Frontend: 60 FPS stable

**Stretch (200K):**
- Tick time: <50ms avg (slightly over budget, acceptable at 22.2Hz)
- Vision: ~35ms
- Movement: ~13ms
- Frontend: 60 FPS stable

### Visual Quality Check

1. Spawn 0.5m and 10m creatures side by side
2. Verify small reacts faster (visibly)
3. Verify large appears ponderous
4. Test predator sneaking from behind (FOV blind spot)
5. Zoom smoothness at 150K creatures (GPU interpolation from Sprint 14)

### Hardware Metrics

Use cockpit to capture snapshots:
- **Baseline:** Before Sprint 15
- **Post-uber-struct:** After Phase 1
- **Post-vision:** After Phase 2A
- **Final:** All optimizations active

Compare IPC, L1/L2 cache miss rates, frame times.

**Success:**
- 150K creatures @ 22.2Hz sustained
- 60 FPS frontend (Sprint 14 GPU interpolation)
- Vision <40% frame budget (was 70%)
- Cache hit rates improved (uber-struct validation)

---

## Testing Requirements

**Unit Tests:**
- [ ] Spawn timing staggered (no first-frame spike)
- [ ] Steady-state distribution (no synchronization over time)
- [ ] Size-based frequency (large 5x slower than small)
- [ ] Memory leak prevention (despawn cleanup)
- [ ] FOV blind spots (entities outside FOV not detected)
- [ ] Vec2 math (distance, normalize, dot product)
- [ ] BehaviorMode::Catatonic replaces Catatonic component
- [ ] Uber-struct refactor doesn't change behavior

**Integration Tests:**
- [ ] 20K creatures stable at 22.2Hz
- [ ] Large creatures visibly slower reactions
- [ ] 150K creatures @ <45ms tick time
- [ ] Zero allocations in vision system (profiler verified)

---

## Component Reorganization Summary

### Files to Rename/Refactor

**Perception → Vision:**
- `apps/simulation/src/simulation/perception/` → `vision/`
- `perception/components.rs` → `vision/components.rs`
- `perception/systems.rs` → `vision/systems.rs`
- `Perception` struct → `Vision` struct
- `update_perception_system` → `update_vision_system`

**Uber-Struct Pattern:**
- Remove `Catatonic` component → `BehaviorMode::Catatonic` enum
- Split components into hot (Transform, Physics) and cold (BiologyData)
- Add `CreatureState` uber-struct for warm data

**Vec2 Migration:**
- `Position { x, y }` → `Position(Vec2)`
- `Velocity { vx, vy }` → `Velocity(Vec2)`
- `Acceleration { ax, ay }` → `Acceleration(Vec2)`

---

## Success Metrics

**Performance:**
- [ ] 150K creatures @ 22.2Hz (HIGH confidence: 90%)
- [ ] 200K creatures @ 22.2Hz (MEDIUM confidence: 60%)
- [ ] Vision <40% frame budget (was 70%)
- [ ] Zero Vec allocations in vision system

**Behavior:**
- [ ] Size-based reaction times visible
- [ ] FOV blind spots enable sneaking
- [ ] No synchronization spikes

**Architecture:**
- [ ] Stable archetypes (no add/remove churn)
- [ ] SIMD vector math throughout
- [ ] Component-based timing (not HashMap)
- [ ] Multi-core parallelization on pure systems

---

## Risks & Mitigations

**Risk:** Uber-struct refactor breaks existing systems
- **Mitigation:** TDD - write tests first, refactor incrementally
- **Rollback:** Git branch isolation

**Risk:** FOV blind spots create exploits
- **Mitigation:** Balance FOV width (start with 180°, tune based on gameplay)
- **Validation:** Playtesting with zoologist-tom validation

**Risk:** Vec2 migration introduces subtle bugs
- **Mitigation:** Comprehensive unit tests for all vector operations
- **Testing:** Compare outputs before/after migration

**Risk:** Parallelization introduces race conditions
- **Mitigation:** Only parallelize pure systems (no entity lookups)
- **Testing:** Determinism tests (same seed = same output)

---

## Phase 3: Spatial Grid + Parallel Perception (DEFERRED TO SPRINT 16+)

**Status:** NOT INCLUDED IN SPRINT 15

**Trigger:** If Phase 2D validation shows perception still bottleneck at 150K creatures

**Duration:** 5 days

**See:** `SPRINT_16_PLAN/SPRINT_PLAN_sprint-16-spatial-grid.md` for complete implementation plan.
**See:** `SPRINT_16_PLAN/RATIONALE.md` for decision logic.

### Quick Context

**The Problem:**
Even with perfect 8-core Rayon parallelization, O(N²) perception fails at scale:
- 150K creatures: 22.5B comparisons → 3,825ms (85x over budget)
- Solution: Spatial grid FIRST (O(N²) → O(N×k)), THEN Rayon
- Expected result: 150K @ 7ms perception, enabling 200K+ creatures

**Why Deferred:**
Sprint 15 focuses on zero-allocation and cache-friendly patterns (split queries, Vec2 SIMD, stochastic vision). If these succeed at 150K+, spatial grid can wait for Sprint 16+.

**Key Metrics (if implemented):**

| Metric | Before (O(N²)) | After (Grid + Rayon) | Improvement |
|--------|----------------|----------------------|-------------|
| Perception @ 150K | ~30,600ms | ~7ms | 4,371x faster |
| Max creatures @ 45ms | ~5K | 200K+ | 40x capacity |

**Dependencies:**
- `rayon = "1.10"`
- `rustc-hash = "2.0"` (FxHashMap)

**Architecture:**
- 200m × 200m grid cells (2× max perception range)
- Complexity: O(N²) → O(N×k) where k ≈ 180 neighbors
- Reduction: 833x fewer operations @ 150K

**References:**
- Full spec: `docs/architecture/spatial-partitioning.md`
- Implementation plan: `SPRINT_16_PLAN/SPRINT_PLAN_sprint-16-spatial-grid.md` (TDD breakdown, benchmarking strategy, success criteria)

---

## Future Work

**Sprint 16 (DECISION POINT):**
- **Option A:** Organic Shader Animation (if Phase 2D achieves 150K @ <45ms)
- **Option B:** Spatial Grid Implementation (if Phase 2D fails to hit target)

**Sprint 17+ (Advanced Optimizations):**
- Hierarchical spatial grid (quadtree for very large worlds)
- Spatial hash optimization (perfect hashing for fixed world size)
- GPU-accelerated perception (compute shaders for massive scale)
- Viewport culling (only update on-screen creatures)
- DNA-driven `neural_speed` gene (0.5-2.0 multiplier, costs energy²)
- Metabolic brain cost (fast reactions = high energy drain)
- Variable LOD based on zoom level

---

## References

- **Sprint 14:** Frontend GPU interpolation (prerequisite)
- **Sprint 13:** NAPI-RS migration (zero-copy buffers)
- **Sprint 12:** Hardware Metrics Cockpit
- **Biology notes:** `docs/biology/biology-notes.md`
- **Optimization backlog:** `docs/performance/optimization-backlog.md`
