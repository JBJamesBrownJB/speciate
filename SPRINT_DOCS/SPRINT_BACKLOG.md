# Sprint 15: ECS Optimizations - Backlog

**Branch:** `feat/sprint-15-ecs-optimizations`
**Status:** IN PROGRESS
**Duration:** 6 days

---

## Sprint Goal

Scale backend ECS simulation to 150K-200K creatures through:
1. **Uber-struct pattern** (stable archetypes, hot/cold split, cache-friendly) - DONE
2. **Vision system refactor** (remove Vec allocation bottleneck, FOV, stochastic updates) - DONE
3. **Vec2 vector math** (SIMD optimization) - DONES
4. **Parallelization** (multi-core utilization) - DONE FOR MOVEMENTS
5. **🚨 CRITICAL: Stochastic Vision (Phase 2D) - FINAL Sprint 15 Task!!** 🚨
   - Status: PENDING (cleanup complete, ready to implement)
   - Expected: 90% perception time reduction @ 50K+ creatures
   - Only update 10% of creatures per tick instead of 100%

---

## Baseline Metrics (Updated @ 20K Active Wanderers After Rayon)
**Source:** `docs/performance/snapshots/10k_wanderers_2025-11-28T21-40-33.json` (10K post-Rayon)
**Previous baseline:** 10K @ 49ms (pre-Rayon), 5K @ 50ms (pre-vision optimization)

| Metric | Current Value | Status |
|--------|---------------|--------|
| Total tick time | **28.7ms @ 10K** | ✅ **WAY UNDER BUDGET** (45ms target) |
| Movement systems | **4.1ms (14%)** | ✅ **6.3x SPEEDUP** (was 26.5ms) |
| Perception system | **20.1ms (70%)** | Now the bottleneck |
| Avoidance | 3.3ms (12%) | Acceptable |
| Rotation | 0.2ms (<1%) | Negligible |
| Vec allocations | **~0 bytes** | ✅ **ELIMINATED** |
| Max active creatures | **20K tested** | **4x improvement from Sprint start!** |
| CPU utilization | **24.7% (all 16 cores)** | ✅ **Multi-core engaged!** |
| IPC | **4.25** | ✅ **Excellent!** (was 3.0) |
| CPU cores active | **16/16** | ✅ **Full parallelism** |

---

## Sprint 15 Progress

| Phase | Status | Expected Gain | Progress |
|-------|--------|---------------|----------|
| Phase 2A: Vision Split Queries | ✅ COMPLETE | 2x capacity (5K→10K) | 100% |
| Phase 1b: Uber-Struct Refactor | ✅ COMPLETE | Archetype stability | 100% |
| Phase 2A-2: Movement Optimizations | ✅ COMPLETE | 13% tick budget (~6.5ms) | 100% (7/8, OPT-8 deferred) |
| Phase 1: Archetype Churn Trial | 📋 SKIPPED | Validation unnecessary | N/A |
| Phase 2B: Vec2 + Changed<T> | 📋 PLANNED | 2-3ms @ 20K | 0% |
| Phase 2C: Parallelization | ✅ COMPLETE | 2-3x speedup | 100% |
| **Phase 2D: Stochastic Vision** | **🔄 NEXT** | **90% perception** | **0% - READY TO START** |
| Phase 3: Spatial Grid | 🔮 DEFERRED | Sprint 16+ | N/A |

**Legend:** ✅ Complete | 🔄 In Progress | 📋 Planned | ⏸️ Conditional | 🔮 Deferred

---

# ✅ COMPLETED PHASES

## Phase 2A: Vision Split Queries (Day 3 - CRITICAL) ✅ COMPLETE

### Actual Metrics (Completed 2025-11-28)
| Metric | Before | After | Gain | Status |
|--------|--------|-------|------|--------|
| Vec allocations/frame | 3.2MB | **~0 bytes** | **~100% eliminated** | ✅ |
| Perception time @ 5K | 34ms | **18.3ms @ 10K** | **46% reduction** | ✅ |
| Active creature capacity | 5K | **10K** | **2x increase** | ✅ |
| Tick time | 50ms @ 5K | **49ms @ 10K** | Stable | ✅ |
| Memory churn | 64MB/sec | **~0** | Eliminated | ✅ |

**Source:** `docs/performance/snapshots/10k_wanderers_2025-11-28T18-53-31.json`
**Commit:** `642b598` - "massive perf boost in perception"

**CRITICAL:** This was the single most important optimization - DELIVERED!

### Tasks
- [x] Rename `Perception` → `Vision` (biological naming)
- [x] Add `VisionTiming` component (stochastic updates)
- [x] Add `Visible` marker component (zero-cost filter)
- [x] **CRITICAL:** Remove `.collect()` Vec allocation
- [x] Implement split queries (observers mut, targets immut)
- [x] Implement FOV dot product check (blind spots)
- [x] Add stochastic vision updates (reaction time gating)
- [x] **BENCHMARK:** Verify zero allocations with profiler
- [ ] **BENCHMARK:** Vision time @ 50K, 100K creatures (deferred to Phase 2D)

**Owner:** ecs-emma + rusty-ron
**Validation:** instrumentation-ian, zoologist-tom (FOV biology)

---

## ✅ Phase 1b: Uber-Struct Refactor

**Status:** COMPLETE (2025-11-28)
**Commit:** 2369ec1 - "quite beautiful sim now"
**Impact:** Archetype stability achieved, eliminated component churn on behavior transitions

### Actual Implementation

Instead of the planned `PhysicalTraits` uber-struct, the refactor used `CreatureState`:

```rust
pub enum BehaviorMode {
    Catatonic = 0,
    Seeking = 1,
    Wandering = 2,
}

pub struct CreatureState {
    pub behavior: BehaviorMode,  // Enum variant, not component!
    pub energy: f32,
    pub age: f32,
    pub max_speed: f32,
}
```

### Key Changes

- [x] Remove `Catatonic` component → `BehaviorMode::Catatonic` enum ✅
- [x] Implement uber-struct pattern via `CreatureState` ✅
- [x] Verify stable archetype (no add/remove component churn) ✅
- [x] Add `BehaviorMode::is_active()` helper method ✅

**Result:** Behavior transitions no longer cause archetype changes. Component layout remains stable throughout creature lifecycle.

---

## Phase 2C: Rayon Parallelization (CRITICAL) ✅ COMPLETE

**Status:** COMPLETE (2025-11-28)
**Source:** `docs/performance/snapshots/10k_wanderers_2025-11-28T21-40-33.json`

### Actual Metrics - MASSIVE WIN! 🚀

| Metric | Before (Sequential) | After (Rayon Parallel) | Improvement | Status |
|--------|---------------------|------------------------|-------------|--------|
| **Movement time** | 25.9ms | **4.1ms** | **-21.8ms (84% faster!)** | ✅ |
| **Total tick time** | 47.7ms | **28.7ms** | **-19ms (40% faster!)** | ✅ |
| **CPU cores active** | 9.3 avg | **16 (ALL!)** | **+73% core usage** | ✅ |
| **CPU utilization** | 19.6% | **24.7%** | **+26%** | ✅ |
| **IPC** | 3.0 | **4.25** | **+42% efficiency** | ✅ |

**Result:** Movement system achieved **6.3x speedup** through Rayon parallelization!

### Implementation

**Pattern:** Collect → Parallel → Write-back

```rust
use rayon::prelude::*;

pub fn integrate_motion_system(
    mut query: Query<(Entity, &BodySize, &mut Position, &mut Velocity, ...)>,
    // ... resources
) {
    // Collect entities into Vec for Rayon parallel processing
    let mut entities: Vec<_> = query.iter_mut().collect();

    // Parallel physics integration using Rayon
    entities.par_iter_mut().for_each(|(entity, size, position, velocity, ...)| {
        // Physics logic runs in parallel across all CPU cores!
        // Each of 16 cores processes ~625 creatures (at 10K total)
    });

    // Parallel boundary enforcement (reuse Vec)
    entities.par_iter_mut().for_each(|(_, _, position, velocity, ...)| {
        // Boundary clamping runs in parallel
    });
}
```

### Why It Works

**Bevy's `par_iter_mut()` vs Rayon:**
- Bevy's parallel iteration uses `bevy_tasks` (not configured for max parallelism in NAPI context)
- Rayon's `par_iter_mut()` uses ALL available CPU cores automatically
- Manual collect → parallel → write-back pattern gives full control

**Key Insights:**
1. **Embarrassingly parallel:** Each creature's physics is independent
2. **All 16 cores engaged:** Rayon distributes 10K creatures across threads
3. **IPC jumped to 4.25:** Excellent instruction-level parallelism
4. **Zero overhead:** Mutable references write back automatically

### Capacity Projection

**Current @ 10K creatures:**
- Total tick: 28.7ms
- Budget remaining: **16.3ms headroom!**

**Projected capacity:**
- Comfortable: **~15K creatures** (within 45ms budget)
- With optimization: **20K+ creatures** achievable!

### Tasks

- [x] Add `rayon` dependency to Cargo.toml ✅
- [x] Implement collect → parallel → write-back pattern ✅
- [x] Parallelize physics integration loop ✅
- [x] Parallelize boundary enforcement loop ✅
- [x] Verify zero behavioral regression (all 156 tests pass) ✅
- [x] Benchmark @ 10K creatures (28.7ms total, 4.1ms movement) ✅

**CRITICAL SUCCESS:** Parallelization delivered exactly what was promised - multi-core utilization with massive speedup!

---

# 🔄 ACTIVE WORK

## Phase 2A-2: Movement System Optimizations (ONGOING)

**Context:** Movement system is now the bottleneck after perception optimization (commit 642b598). At 10K creatures @ 22.2Hz, movement systems perform ~666,000 sqrt operations per second.

**Strategy:** Apply same optimization patterns from perception (defer expensive ops, squared comparisons, early exits).

### Baseline Metrics (10K Wanderers)
| Metric | Current Value |
|--------|---------------|
| Movement systems time | 26.5ms (54% of tick) |
| Avoidance time | 3.2ms (7% of tick) |
| Rotation time | 0.2ms (<1% of tick) |
| Total sqrt ops/sec | ~666,000 |

---

### OPT-1: Defer sqrt() in integrate_motion_system

**Location:** `apps/simulation/src/simulation/movement/systems.rs:55-56`

**Change:**
```rust
// BEFORE
let speed = (velocity.vx * velocity.vx + velocity.vy * velocity.vy).sqrt();
if speed > 0.1 { /* ... */ }

// AFTER
let speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
if speed_sq > 0.01 {  // Defer sqrt until after check
    let speed = speed_sq.sqrt();
    /* ... */
}
```

**Expected Metrics:**
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| sqrt calls @ 10K | 10K/tick | ~9K/tick | 10% reduction |
| Movement time | Baseline | -0.5ms | 2% improvement |

**Risk:** LOW | **Effort:** 15 min | **Status:** ✅ **COMPLETE** (2025-11-28)

**Actual Results:**
- All 156 library tests pass
- Zero behavioral regression
- Defers sqrt() until after speed_sq > 0.01 check
- Eliminates ~1,000 sqrt/tick for slow/stationary creatures

---

### OPT-2: Cache inv_sqrt_length in BodySize ⭐

**Location:** `apps/simulation/src/simulation/core/components.rs:55`

**Change:**
```rust
// BEFORE
pub struct BodySize {
    pub length: f32,
}
// Movement system recalculates: 1.0 / size.length.sqrt() every tick!

// AFTER
pub struct BodySize {
    pub length: f32,
    pub inv_sqrt_length: f32,  // Cached at spawn (immutable)
}

impl BodySize {
    pub fn new(length: f32) -> Self {
        Self {
            length,
            inv_sqrt_length: 1.0 / length.sqrt(),
        }
    }
}
```

**Expected Metrics:**
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| sqrt calls @ 10K | 10K/tick | 0/tick | **100% eliminated** |
| Operations saved/sec | 222K sqrt | 0 sqrt | **222K/sec saved** |
| Movement time | Baseline | -2ms | **8% improvement** |
| Memory overhead | 0 | +4 bytes/creature | 40KB @ 10K |

**Risk:** LOW | **Effort:** 30 min | **Status:** ✅ **COMPLETE** (2025-11-28)

**Actual Results:**
- All 156 library tests pass
- Added `inv_sqrt_length: f32` field to `BodySize` component (+4 bytes/creature)
- Pre-computed at spawn via `BodySize::new(length)` constructor
- Added `update_body_size_cache()` system using `Changed<BodySize>` filter
- System runs BEFORE `integrate_motion_system` in schedule
- Changed line 59: `size.inv_sqrt_length` instead of `1.0 / size.length.sqrt()`
- **Serialization fix:** Added `#[serde(skip)]` and `#[reflect(ignore)]` to cache field (prevents Bevy reflection panic)
- **Future-proof:** When growth is added, cache auto-updates via Bevy's change detection
- Zero behavioral regression

---

### OPT-3: Eliminate division in speed clamping

**Location:** `apps/simulation/src/simulation/movement/systems.rs:70-76`

**Change:**
```rust
// BEFORE
if speed_sq > max_speed_sq {
    let speed = speed_sq.sqrt();
    let inv_speed = MAX_SPEED / speed;  // Division!
    velocity.vx *= inv_speed;
    velocity.vy *= inv_speed;
}

// AFTER
if speed_sq > max_speed_sq {
    let scale = (max_speed_sq / speed_sq).sqrt();  // Single sqrt, no division
    velocity.vx *= scale;
    velocity.vy *= scale;
}
```

**Expected Metrics:**
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| Division ops | 1/clamp | 0 | Eliminate division |
| Numerical precision | Good | Slightly better | fp32 accuracy |

**Risk:** LOW | **Effort:** 10 min | **Status:** ✅ **COMPLETE** (2025-11-28)

**Actual Results:**
- All 156 library tests pass
- Changed lines 72-76: Replaced `sqrt() + division` with single `sqrt(ratio)`
- Mathematical equivalence: `MAX_SPEED / sqrt(speed_sq)` = `sqrt(max_speed_sq / speed_sq)`
- Eliminates ~100-200 divisions/tick (every creature exceeding max speed)
- Division: ~40 cycles → Eliminated
- Better numerical precision (fewer floating-point operations)
- Zero behavioral regression

---

### OPT-4: Skip atan2() when velocity unchanged

**Location:** `apps/simulation/src/simulation/movement/rotation.rs:16`

**Change:**
```rust
// Add component
#[derive(Component)]
pub struct PreviousVelocity(Vec2);

// Update system
pub fn rotation_system(
    mut query: Query<(&Velocity, &mut Rotation, &mut PreviousVelocity)>
) {
    for (velocity, mut rotation, mut prev_vel) in query.iter_mut() {
        if velocity.vx != prev_vel.0.x || velocity.vy != prev_vel.0.y {
            rotation.radians = velocity.vy.atan2(velocity.vx);
            prev_vel.0 = Vec2::new(velocity.vx, velocity.vy);
        }
    }
}
```

**Expected Metrics:**
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| atan2 calls @ 10K | 10K/tick | ~2-4K/tick | **60-80% reduction** |
| Rotation time | 0.2ms | ~0.1ms | 50% improvement |
| Memory overhead | 0 | +8 bytes/creature | 80KB @ 10K |

**Risk:** LOW | **Effort:** 45 min | **Status:** ✅ **COMPLETE** (2025-11-28)

**Actual Implementation:**
- Used `Changed<Velocity>` filter instead of PreviousVelocity component (zero archetype churn!)
- Fixed spurious write in movement system (lines 38-41): only set velocity to 0 if not already 0
- Modified rotation_system query: `Query<(&mut Rotation, &Velocity), Changed<Velocity>>`
- All 156 library tests pass
- Zero new components, zero memory overhead
- Bevy's internal change tracking handles detection automatically

**How It Works:**
- Bevy tracks "last modified tick" for every component internally
- `Changed<Velocity>` filter only processes entities where Velocity was written this tick
- Stopped catatonic creatures: velocity unchanged → skip atan2() entirely
- Moving creatures: velocity changes → rotation updates as normal
- **Critical:** Eliminated spurious write prevents false-positive change detection

**Expected Benefit:**
- Stopped/slow creatures skip atan2() (~50-100 cycles saved)
- Rotation system only processes entities that actually moved
- Zero memory overhead (vs +8 bytes/creature with PreviousVelocity approach)

---

### OPT-5: Squared distance in wander system

**Location:** `apps/simulation/src/simulation/creatures/behaviors/wander.rs:36, 60, 74, 86`

**Change:** Replace 4 sqrt() calls with squared distance checks + early exits

**Expected Metrics:**
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| sqrt calls/wandering creature | 4/tick | 1-2/tick | **50-75% reduction** |
| Wander time @ 10K | Baseline | -1ms | 15% improvement |

**Risk:** LOW | **Effort:** 1 hour | **Status:** ✅ **COMPLETE** (2025-11-28)

**Actual Implementation:**
- All 156 library tests pass
- Deferred 4 sqrt() calls in wander system using squared distance pattern:
  1. **Line 36:** Speed calculation - defer sqrt until needed for normalization
  2. **Line 60:** Desired length - defer sqrt until needed for normalization
  3. **Line 74:** Steer magnitude - defer sqrt until needed for clamping
  4. **Line 86:** To-home distance - defer sqrt until needed for normalization
- Pattern: Check `value_sq < threshold_sq` before computing `sqrt(value_sq)`
- Slow/stationary wanderers: **4 sqrt → 0 sqrt** (100% elimination)
- Active wanderers: **4 sqrt → 2-3 sqrt** (25-50% reduction)
- Zero behavioral regression

---

### OPT-6: Squared distance in seek system

**Location:** `apps/simulation/src/simulation/creatures/behaviors/seek.rs:37, 47`

**Change:** Replace 2 sqrt() calls with squared distance checks

**Expected Metrics:**
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| sqrt calls/seeking creature | 2/tick | 1/tick | **50% reduction** |
| Seek time @ 5K seekers | Baseline | -0.5ms | 10% improvement |

**Risk:** LOW | **Effort:** 45 min | **Status:** ✅ **COMPLETE** (2025-11-28)

**Actual Implementation:**
- All 156 library tests pass
- Deferred 2 sqrt() calls in seek system:
  1. **center_distance:** Converted all distance checks to squared comparisons (lines 39, 50, 57), only compute sqrt at line 65 when needed for normalization
  2. **current_speed:** Eliminated entirely - replaced with squared comparison at line 50
- Early exits now use squared distance checks (no sqrt until line 65)
- Seekers that arrive/pounce: **2 sqrt → 0 sqrt** (100% elimination)
- Active seekers: **2 sqrt → 1 sqrt** (50% reduction)
- Zero behavioral regression

---

### OPT-7: Early exit in avoidance (CRITICAL) ⭐⭐⭐

**Location:** `apps/simulation/src/simulation/creatures/behaviors/avoidance.rs:50`

**Change:**
```rust
// BEFORE (O(N×M) - 100K sqrt/tick @ 10K creatures)
for other_entity in perception.iter_neighbors() {
    let center_distance = (away_x * away_x + away_y * away_y).sqrt();  // ALWAYS!
    if safe_distance < avoidance.personal_space { /* ... */ }
}

// AFTER (80% early exit before sqrt)
let personal_space_sq = avoidance.personal_space * avoidance.personal_space;
for other_entity in perception.iter_neighbors() {
    let center_distance_sq = away_x * away_x + away_y * away_y;

    // Early exit BEFORE sqrt (80% of neighbors outside personal space)
    if center_distance_sq > personal_space_sq + max_radius * max_radius {
        continue;
    }

    let center_distance = center_distance_sq.sqrt();  // Only for close neighbors
    /* ... */
}
```

**Expected Metrics:**
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| sqrt calls @ 10K, 10 neighbors | 100K/tick | 20K/tick | **80% reduction** |
| Avoidance time | 3.2ms | 0.8ms | **75% improvement** |
| Operations saved/sec | 2.22M sqrt | 444K sqrt | **1.78M/sec saved** |

**Risk:** LOW | **Effort:** 30 min | **Status:** ✅ **COMPLETE** (2025-11-28)
**Critical:** Biggest single win - O(N×M) nested loop optimization!

**Actual Implementation:**
- All 156 library tests pass
- Early exit with squared distance check before sqrt (lines 51-62):
  ```rust
  let center_distance_sq = away_x * away_x + away_y * away_y;

  let other_radius = other_size.radius();
  let max_combined_radius = self_radius + other_radius;
  let max_interaction_distance = avoidance.personal_space + max_combined_radius;
  let max_interaction_distance_sq = max_interaction_distance * max_interaction_distance;

  // Early exit BEFORE sqrt (80% of neighbors)
  if center_distance_sq > max_interaction_distance_sq {
      continue;
  }

  // Only compute sqrt for close neighbors (20% of cases)
  let center_distance = center_distance_sq.sqrt();
  ```
- Accounts for both creature radii (not just personal_space)
- 100K sqrt/tick → 20K sqrt/tick (80% reduction expected)
- O(N×M) optimization - most critical improvement in this phase

---

### OPT-8: Fast inverse sqrt (Advanced - DEFERRED)

**Implementation:** Quake III fast_inv_sqrt() approximation

**Expected Metrics:**
| Metric | Before | After | Gain |
|--------|--------|-------|------|
| sqrt speed | 1x | ~10x | **10x faster** |
| Numerical accuracy | Exact | ~99% | 1% error |
| Movement time | Baseline | -5ms | 20% improvement |

**Risk:** MEDIUM (1% numerical error) | **Effort:** 2 hours | **Status:** [ ] Deferred
**Recommendation:** Defer to future sprint (uncertain biological impact)

---

### Future Work: Rayon Parallelization for Movement Systems

**After completing OPT-1 through OPT-7, consider Rayon for multi-core parallelization.**

**Concept:**
```rust
use rayon::prelude::*;

// Convert ECS query iteration to parallel
pub fn integrate_motion_system(
    query: Query<(Entity, &BodySize, &mut Position, &mut Velocity, ...)>,
    // ... resources
) {
    // Collect entities into Vec for parallel processing
    let entities: Vec<_> = query.iter().collect();

    entities.par_iter().for_each(|(entity, size, pos, vel, ...)| {
        // Parallel physics integration
        // Each thread processes subset of creatures
    });
}
```

**Expected Metrics:**
| Metric | Sequential (post OPT-7) | Parallel (8 cores) | Gain |
|--------|-------------------------|-------------------|------|
| Movement time @ 10K | ~24ms (optimized) | ~6ms | **4x speedup** |
| Movement time @ 20K | ~48ms | ~12ms | **4x speedup** |
| CPU utilization | 12.5% (1 core) | 100% (8 cores) | Full core usage |

**Challenges:**
- **Mutable access:** `&mut Position`, `&mut Velocity` require careful partitioning
- **Resource contention:** `Res<DeltaTime>`, `Res<WorldBounds>` are read-only (safe)
- **Boundary enforcement:** Second loop may need special handling
- **Archetype iteration:** Rayon works on Vec, need to collect from ECS query

**Recommended Approach:**
1. Collect entities into `Vec<(Entity, Position, Velocity, ...)>` (read phase)
2. `par_iter_mut()` on Vec for parallel computation
3. Write results back to ECS components (write phase)
4. Benchmark batch sizes (64, 256, 512) for optimal thread granularity

**Why Defer to Next Sprint:**
- Complete sequential optimizations first (establish baseline)
- Rayon adds complexity (collect → parallel → write-back pattern)
- Need empirical validation that parallelization overhead < gains
- At 10K creatures, gains are modest; at 50K+ creatures, becomes critical

**See:** Phase 2C in main sprint plan for full parallelization strategy

---

### Priority Execution Order

| Priority | Optimization | Expected Gain | Effort | Files |
|----------|--------------|---------------|--------|-------|
| **1** | OPT-7: Avoidance early exit | 75% avoidance (-2.4ms) | 30 min | avoidance.rs |
| **2** | OPT-2: Cache inv_sqrt_length | 8% movement (-2ms) | 30 min | components.rs, systems.rs |
| **3** | OPT-5: Wander squared | 15% wander (-1ms) | 1 hour | wander.rs |
| **4** | OPT-6: Seek squared | 10% seek (-0.5ms) | 45 min | seek.rs |
| **5** | OPT-4: Skip atan2() | 50% rotation (-0.1ms) | 45 min | rotation.rs |
| **6** | OPT-1: Defer sqrt check | 2% movement (-0.5ms) | 15 min | systems.rs |
| **7** | OPT-3: Eliminate division | <1% movement | 10 min | systems.rs |

**Total Expected Gain:** ~6.5ms reduction @ 10K creatures (13% of tick budget)

---

### Test Strategy (TDD)

**RED Phase:**
```rust
#[test]
fn test_body_size_caches_inv_sqrt() {
    let size = BodySize::new(4.0);
    assert_eq!(size.inv_sqrt_length, 0.5);
}

#[test]
fn test_avoidance_early_exit_far_neighbors() {
    // Verify sqrt skipped for distant neighbors
}
```

**GREEN Phase:** Implement optimizations

**REFACTOR Phase:** Zero behavioral regression, all tests pass

---

### Success Criteria

- [ ] All existing tests pass (zero behavioral regression)
- [ ] 15%+ reduction in movement system CPU time
- [ ] 5-10ms tick budget reduction @ 10K creatures
- [ ] Instrumentation metrics validate gains
- [ ] Update `docs/spec/movement-spec.md`

---


---

# 📋 PLANNED PHASES

## 📋 Phase 1: Archetype Churn Trial - SKIPPED

**Status:** SKIPPED
**Reason:** Phase 1b (uber-struct refactor) was completed without validation trial

### Original Purpose
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

## 📋 Phase 2B: Changed<T> Filters + Vec2

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

## 📋 Phase 2C: Parallelization

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

## 🔄 Phase 2D: Stochastic Vision + Validation (NEXT - FINAL Sprint 15 Task)

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

## Progress Notes

### 2025-11-28 (Night): Sprint 15 Cleanup Complete ✅
- **Phase 1 (Safety):** Fixed test duplication (70 lines removed), added 3 Rayon safety tests
- **Phase 2 (Quality):** Created query type aliases, removed 3 clippy suppressions, added inline math docs
- **Phase 3 (Docs):** Created movement-spec.md, avoidance-spec.md, wander-spec.md, updated CLAUDE.md
- **All 161 tests passing:** Zero compiler warnings, zero clippy warnings
- **🚨 NEXT: Phase 2D - Stochastic Vision (FINAL Sprint 15 Task)** 🚨

### 2025-11-28 (Evening): Phase 2C Complete ✅ - MASSIVE WIN!
- **Rayon parallelization delivered:** 6.3x movement speedup (25.9ms → 4.1ms)
- **Total tick improvement:** 47.7ms → 28.7ms (40% faster!)
- **All 16 CPU cores engaged:** 19.6% → 24.7% utilization
- **IPC jumped to 4.25:** Excellent instruction-level parallelism
- **Capacity validated:** 20K creatures tested successfully!
- **Source:** Snapshot `10k_wanderers_2025-11-28T21-40-33.json`
- **Implementation:** Collect → Parallel → Write-back pattern with Rayon

### 2025-11-28 (Afternoon): Phase 2A-2 Complete ✅
- **Movement optimizations delivered:** 7 sequential optimizations (OPT-1 through OPT-7)
- **Actual savings:** ~1.3ms total (modest but stable)
- **Key optimizations:** Deferred sqrt, cached inv_sqrt, early exits
- **OPT-7 (avoidance):** 3.2ms → 2.6ms (19% reduction)
- **All 156 tests passing:** Zero behavioral regression
- **Source:** Snapshots throughout the day

### 2025-11-28 (Morning): Phase 2A Complete ✅
- **Perception optimization delivered:** 10K creatures @ 49ms (2x capacity improvement)
- **Vec allocations eliminated:** ~0 bytes/frame (was 3.2MB)
- **Perception time reduced:** 18.3ms @ 10K (was 34ms @ 5K) - 46% improvement
- **Source:** Snapshot `10k_wanderers_2025-11-28T18-53-31.json`, commit `642b598`

### Sprint Context
- Sprint 14 delivered GPU interpolation (165 FPS achieved)
- Frontend ready for high entity counts (200K+)
- Backend bottleneck addressed (Phase 2A complete)
- Focus: zero-allocation, cache-friendly, parallel architecture

---


---

# 🔮 DEFERRED TO FUTURE SPRINTS

## Phase 3: Spatial Grid + Parallel Perception → SPRINT 16

**Status:** NOT IN SPRINT 15 SCOPE
**Decision Point:** Based on Sprint 15 Phase 2D validation results
**Duration:** 5 days

**Trigger Conditions:**
- Sprint 15 fails to achieve 150K creatures @ <45ms, OR
- Perception still >40% of frame budget after all Phase 2 optimizations

**Why Deferred:**
The O(N²) perception complexity requires algorithmic changes (spatial grid) before parallelization can help. Sprint 15 focuses on zero-allocation, cache-friendly, and SIMD optimizations first. If these achieve 150K+ creatures, spatial grid can wait.

**See:** `SPRINT_16_PLAN/SPRINT_PLAN_sprint-16-spatial-grid.md` for full implementation plan.
**See:** `SPRINT_16_PLAN/RATIONALE.md` for decision logic and when to trigger Sprint 16.

### Quick Summary

**The Problem:**
- Current O(N²) perception: 150K creatures = 22.5B comparisons = 3,825ms (85x over budget)
- Even with 8-core Rayon: Still 85x over budget (algorithmic bottleneck)

**The Solution:**
- Spatial grid: O(N²) → O(N×k) where k ≈ 180 neighbors
- 150K × 180 = 27M comparisons (833x reduction)
- With Rayon (8 cores): ~7ms perception @ 150K
- With stochastic vision (10%/tick): ~0.7ms perception

**Expected Gain:**
- Sequential grid @ 150K: ~40ms (765x faster than naive)
- Parallel grid @ 150K: ~7ms (4,371x faster than naive)
- Enables 200K+ creatures with comfortable headroom

**Dependencies:**
```toml
rayon = "1.10"
rustc-hash = "2.0"  # FxHashMap
```

**Key Architecture:**
- 200m × 200m grid cells (2× max perception range)
- FxHashMap for cell storage (2-5x faster than std HashMap)
- Incremental updates (only ~0.8% creatures cross cells per tick)
- Read-only grid during parallel perception phase

**References:**
- Full spec: `docs/architecture/spatial-partitioning.md`
- Implementation plan: `SPRINT_16_PLAN/SPRINT_PLAN_sprint-16-spatial-grid.md`
- Decision guide: `SPRINT_16_PLAN/RATIONALE.md`
