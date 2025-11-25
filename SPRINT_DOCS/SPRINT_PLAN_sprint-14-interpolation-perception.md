# Sprint 14: Interpolation, Vision Refactor & Data-Oriented Design

**Branch:** `feat/sprint-14-interpolation-perception`
**Status:** PLANNED
**Prerequisites:** Sprint 12 complete (Hardware Metrics Cockpit)
**Duration:** 11 days

---

## Sprint Goal

Scale to 150K-200K creatures through:
1. **22.2Hz simulation** → 60Hz interpolated rendering (smooth visuals, 3x capacity)
2. **Perception → Vision refactor** (biological naming, FOV dot product, stochastic updates)
3. **Uber-struct pattern** (stable archetypes, hot/cold split, cache-friendly)
4. **Vec2 vector math** (SIMD optimization)

**Key Architecture:**
- Single-tick 22.2Hz simulation (achieved in Sprint 13 NAPI migration)
- f64 precision SimulationTime (no drift over 24+ hours)
- Component-based timing (10-100x faster than HashMap)
- Per-creature reaction times (natural load distribution, no synchronization spikes)

---

## Team

**Phase 2 Lead (GPU Interpolation & Wiggle):**
- **shader-sarah** (Dr. Sarah Boid) - GPU/Shader specialist
  - WebGL 2.0, GLSL ES 3.0, PixiJS custom geometry
  - Organic procedural animation expert
  - Target: 60 FPS @ 1M entities, <0.5ms CPU, <0.2ms GPU per frame

**Key Collaborators:**
- **zoologist-tom** - 🔥 PRIMARY CREATIVE PARTNER for Sarah
  - Biological motion patterns, creature locomotion physics
  - Natural movement consultation (fish swimming, snake slithering)
  - Ensures visual beauty matches ecological reality
- **frontend-fanny** - PixiJS integration, TypeScript buffer management
- **rusty-ron** - Backend NAPI zero-copy buffers, snapshot format
- **architect-andy** - Performance architecture, technical standards
- **instrumentation-ian** - GPU profiling, frame time metrics
- **ecs-emma** - ECS optimization (Phases 3-4)
- **pm-pam** - Sprint coordination, task breakdown

---

## Phase Order (User Priority)

1. ✅ Tick rate validated (22.2Hz achieved in Sprint 13)
2. ✅ Frontend interpolation (60Hz position + rotation)
3. ✅ Refactor components → uber-struct pattern
4. ✅ Perception → Vision (stochastic updates, FOV, Vec2)
5. ✅ Async zoom (decouple GPU from wheel events)
6. ✅ Performance validation (150K-200K creatures)

---

## Phase 1: Validate Tick Rate (22.2Hz)

**Duration:** Day 1 (COMPLETE - Discovery)

**Goal:** Confirm 22.2Hz achieved in Sprint 13 NAPI migration

**Discovery:**
Sprint 13's NAPI-RS migration introduced hardcoded tick rate in `simulation_engine.rs`:
```rust
// apps/simulation/src/napi_addon/simulation_engine.rs:37
const TARGET_SIMULATION_HZ: f32 = 22.2;
```

This replaced the old `config.rs` approach. 22.2Hz provides:
- ~45ms per tick (vs 16.7ms @ 60Hz)
- 2.7x capacity improvement
- Sufficient for 150K-200K creature target

**Validation:**
- ✅ All systems use `DeltaTime` resource (no hardcoded assumptions)
- ✅ Tick rate confirmed at 22.2Hz in NAPI engine
- ✅ No changes needed - already optimal

**Success:** 22.2Hz stable, ready for Phase 2 interpolation

---

## Phase 2: Frontend Interpolation (60Hz) 🎮 GPU SHADER APPROACH

**Duration:** Days 2-3
**Owner:** shader-sarah (Dr. Sarah Boid)
**Status:** IN PROGRESS
**Technical Spec:** `docs/visuals/shader-smooth-and-wiggle.md`

**Goal:** GPU-accelerated 60Hz rendering with smooth position/rotation interpolation + organic wiggle animation

### Overview

This phase uses **GPU vertex shaders** instead of CPU-based JavaScript interpolation to achieve:
- 60 FPS @ 1 million entities
- <0.5ms CPU overhead per frame
- <0.2ms GPU overhead for interpolation shader
- Zero visual stuttering or "rubber banding"
- Organic procedural animation (wiggle) at near-zero cost

**Key Innovation:** Move interpolation math from CPU (12M ops/sec) to GPU (parallel execution on all entities simultaneously).

### Phase 2A: Custom PixiJS Geometry Setup

**Goal:** Create instanced rendering infrastructure with interleaved attribute buffers.

**Implementation:**
```typescript
// Interleaved Float32Array buffer layout per entity:
// [ startX, startY, endX, endY, startRot, endRot, uvX, uvY ]
const FLOATS_PER_VERTEX = 8;

// Custom PixiJS Geometry
const geometry = new Geometry()
  .addAttribute('aStartPos', buffer, 2, false, FLOAT, stride, 0)
  .addAttribute('aEndPos', buffer, 2, false, FLOAT, stride, 8)
  .addAttribute('aStartRot', buffer, 1, false, FLOAT, stride, 16)
  .addAttribute('aEndRot', buffer, 1, false, FLOAT, stride, 20)
  .addAttribute('aTextureCoord', buffer, 2, false, FLOAT, stride, 24);
```

**Update Strategy:**
- On Server Tick (22.2Hz): Copy `end` data to `start`, load new server data into `end`, reset `uInterpolation` to 0
- On Render Tick (60Hz): Increment `uInterpolation` based on `deltaMS / 45ms`

**Collaboration:** Frontend-Fanny (PixiJS integration), Rusty-Ron (NAPI buffer format validation)

### Phase 2B: Vertex Shader Interpolation (Kinematic Smoothing)

**Goal:** Perfectly smooth linear movement masking 22.2Hz server updates.

**GLSL Vertex Shader:**
```glsl
// Attributes per entity
attribute vec2 aStartPos;
attribute vec2 aEndPos;
attribute float aStartRot;
attribute float aEndRot;
attribute vec2 aTextureCoord;

// Uniforms (updated every frame)
uniform float uInterpolation;  // 0.0 to 1.0
uniform mat3 uProjection;

void main() {
  // Position interpolation
  vec2 worldPos = mix(aStartPos, aEndPos, uInterpolation);

  // Rotation interpolation (shortest path, handles 350°→10° wraparound)
  float rotation = shortestPathAngle(aStartRot, aEndRot, uInterpolation);

  // Apply rotation + project
  vec2 rotatedPos = rotate(aLocalPos, rotation);
  gl_Position = vec4((uProjection * vec3(worldPos + rotatedPos, 1.0)).xy, 0.0, 1.0);
}
```

**Edge Cases:**
- ✅ Rotation wrapping (350° → 10° = 20° CW, not 340° CCW)
- ✅ Entity spawn/despawn (buffer resizing)
- ✅ Extrapolation when `uInterpolation > 1.0` (network lag)

**Collaboration:** Architect-Andy (performance validation), Instrumentation-Ian (GPU profiling)

### Phase 2C: Organic Wiggle Animation

**Goal:** Inject "life" using procedural vertex deformation (fish swimming, snake slithering).

**Enhanced Vertex Shader:**
```glsl
uniform float uGameTime;

void main() {
  // Calculate movement speed for dynamic coupling
  float moveSpeed = length(aEndPos - aStartPos) / 0.045;  // pixels/sec

  // Wiggle algorithm (in local space, before world transform)
  float lagFactor = 3.0;  // Tail lags behind head
  float amplitude = 5.0 * (moveSpeed / 100.0);  // Scale with speed
  float wiggleOffset = sin(uGameTime * 2.0 - aTextureCoord.y * lagFactor) * amplitude;

  vec2 localPos = aLocalVertexPos;
  localPos.x += wiggleOffset * aTextureCoord.y;  // head fixed, tail wiggles

  // ... rest of interpolation shader (position, rotation)
}
```

**Success Criteria:**
- Creatures appear to "swim" organically
- Tail lags behind head (S-curve motion)
- Wiggle intensity correlates with movement speed
- **ZERO performance regression** vs Phase 2B

**Collaboration:** Zoologist-Tom (🔥 biological motion patterns, natural locomotion physics)

### Phase 2D: Performance Validation & Polish

**Metrics:**
- 60 FPS stable @ 1 million entities (Chrome DevTools)
- CPU <0.5ms per frame (profiled)
- GPU <0.2ms per frame (WebGL profiler)
- Zero visual artifacts at 1x-10x zoom
- Cross-GPU compatibility (Intel/NVIDIA/AMD)

**Deliverables:**
- Working shader implementation in `apps/portal/`
- GPU performance metrics integrated into Dev-UI (not Portal!)
- Documentation of shader architecture
- Demo at 200K creatures with smooth, organic motion

**Success:** Buttery-smooth 60 FPS rendering with creatures that swim, wiggle, and move like living organisms

---

## Phase 3: Uber-Struct Refactor

**Duration:** Days 4-5

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

## Phase 4: Vision System + Comprehensive ECS Optimization

**Duration:** Days 6-9 (expanded from 2 days for thorough ECS audit)

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

### Phase 4A: Vision Split Queries (Day 6 - CRITICAL)

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

### Phase 4B: Changed<T> Filters + Vec2 Migration (Day 7)

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

### Phase 4C: Parallelization (Day 8)

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

### Phase 4D: Performance Validation (Day 9)

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
   - Do we need spatial grid (Sprint 14)?

**Evening: Documentation**

7. Update `docs/performance/optimization-backlog.md`
8. Write Phase 4 completion report
9. Document remaining bottlenecks for Sprint 14

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

---

## Phase 5: Performance Validation

**Duration:** Day 11

**Goal:** 150K-200K creatures achieved

### Benchmarks

**Baseline (20K):**
- Tick time: <30ms avg (well under 45ms budget @ 22.2Hz)
- Vision: ~10ms (5-20% creatures per tick)
- Movement: ~8ms
- Frontend: 60 FPS stable

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
5. Zoom smoothness at 150K creatures

### Hardware Metrics

Use cockpit to capture snapshots:
- **Baseline:** Before tick rate change
- **Post-interpolation:** After Phase 2
- **Post-refactor:** After uber-struct changes
- **Final:** All optimizations active

Compare IPC, L1/L2 cache miss rates, frame times.

**Success:**
- 150K creatures @ 20Hz sustained
- 60 FPS frontend (smooth interpolation)
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

**Integration Tests:**
- [ ] 20K creatures stable at 20Hz
- [ ] Interpolation smooth at 60 FPS
- [ ] Large creatures visibly slower
- [ ] Zoom smooth at 150K creatures

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
- [ ] 60 FPS frontend rendering
- [ ] Vision <40% frame budget

**Behavior:**
- [ ] Size-based reaction times visible
- [ ] FOV blind spots enable sneaking
- [ ] No synchronization spikes

**Architecture:**
- [ ] Stable archetypes (no add/remove churn)
- [ ] SIMD vector math throughout
- [ ] Component-based timing (not HashMap)

---

## Risks & Mitigations

**Risk:** Frontend interpolation looks floaty
- **Mitigation:** Linear lerp only (no easing), test with 20K first
- **Fallback:** Increase to 30Hz physics if needed (currently 22.2Hz)

**Risk:** Uber-struct refactor breaks existing systems
- **Mitigation:** TDD - write tests first, refactor incrementally
- **Rollback:** Git branch isolation

**Risk:** FOV blind spots create exploits
- **Mitigation:** Balance FOV width (start with 180°, tune based on gameplay)
- **Validation:** Playtesting with zoologist-tom validation

---

## Future Work (Sprint 14+)

- DNA-driven `neural_speed` gene (0.5-2.0 multiplier, costs energy²)
- Spatial grid for O(1) vision queries (if 200K fails)
- Metabolic brain cost (fast reactions = high energy drain)
- Viewport culling (only update visible creatures)
- Variable LOD based on zoom level

---

## References

- **Sprint 11:** IPC optimization, dual-tick abandoned
- **Sprint 12:** Hardware Metrics Cockpit complete
- **Biology notes:** `docs/biology/biology-notes.md` (lines 850-956)
- **Optimization backlog:** `docs/performance/optimization-backlog.md` (lines 29-33)
