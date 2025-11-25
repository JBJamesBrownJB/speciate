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

1. **Phase 1:** Uber-Struct Refactor (Days 1-2)
2. **Phase 2:** Vision Split Queries (Day 3 - CRITICAL)
3. **Phase 3:** Changed<T> Filters + Vec2 (Day 4)
4. **Phase 4:** Parallelization (Day 5)
5. **Phase 5:** Performance Validation (Day 6)

---

## Phase 1: Uber-Struct Refactor

**Duration:** Days 1-2

**Goal:** Stable ECS archetypes (no add/remove component churn → cache-friendly)

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

## Future Work (Sprint 16+)

- DNA-driven `neural_speed` gene (0.5-2.0 multiplier, costs energy²)
- Spatial grid for O(1) vision queries (if 200K fails)
- Metabolic brain cost (fast reactions = high energy drain)
- Viewport culling (only update visible creatures)
- Variable LOD based on zoom level

---

## References

- **Sprint 14:** Frontend GPU interpolation (prerequisite)
- **Sprint 13:** NAPI-RS migration (zero-copy buffers)
- **Sprint 12:** Hardware Metrics Cockpit
- **Biology notes:** `docs/biology/biology-notes.md`
- **Optimization backlog:** `docs/performance/optimization-backlog.md`
