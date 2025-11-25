# Sprint 14: Interpolation, Vision Refactor & Data-Oriented Design

**Branch:** `feat/sprint-14-interpolation-perception`
**Duration:** 11 days
**Status:** IN PROGRESS

---

## Sprint Goal

**MORE DETAIL IN:** SPRINT_PLAN_sprint-14-interpolation-perception.md, check this to confirm your approach

Scale to 150K-200K creatures through:
1. 22.2Hz simulation → 60Hz interpolated rendering (achieved in Sprint 13)
2. Perception → Vision refactor (biological naming, FOV, stochastic updates)
3. Uber-struct pattern (stable archetypes, hot/cold split)
4. Vec2 vector math (SIMD optimization)

---

## Phase Checklist

### Phase 1: Validate Tick Rate (22.2Hz) - Day 1 ✅ COMPLETE
- [x] Discovered: 22.2Hz achieved in Sprint 13 NAPI migration
- [x] Validated: Hardcoded in simulation_engine.rs:37
- [x] Confirmed: All systems use DeltaTime resource
- [x] Result: 22.2Hz provides ~45ms tick budget (sufficient for 150K-200K target)

### Phase 2: Frontend Interpolation (60Hz) - Days 2-3 🎮 GPU SHADER APPROACH
**Owner:** shader-sarah (Dr. Sarah Boid - GPU/Shader Specialist)
**Status:** IN PROGRESS - GPU shader-based interpolation & organic wiggle animation
**Spec:** `docs/visuals/shader-smooth-and-wiggle.md`

#### Phase 2A: Custom PixiJS Geometry Setup
- [ ] Design interleaved Float32Array buffer layout (start/end pos/rot per entity)
- [ ] Implement custom PixiJS Geometry with instanced rendering
- [ ] Create buffer update strategy (swap prev←curr on snapshot)
- [ ] Verify zero-copy NAPI buffer integration with Rusty-Ron
- [ ] Test buffer uploads @ 200K entities

#### Phase 2B: Vertex Shader Interpolation (Kinematic Smoothing)
- [ ] Implement GLSL vertex shader with mix(aStartPos, aEndPos, uInterpolation)
- [ ] Implement rotation interpolation with shortest-path angle wrapping
- [ ] Handle edge case: rotation wraparound (350° → 10° = 20° CW, not 340° CCW)
- [ ] Handle edge case: entity spawn/despawn (buffer resizing)
- [ ] Handle edge case: extrapolation when uInterpolation > 1.0 (network lag)
- [ ] Test 60 FPS @ 1 million entities
- [ ] Profile: CPU <0.5ms per frame, GPU <0.2ms per frame
- [ ] Cross-GPU testing (Intel/NVIDIA/AMD)

#### Phase 2C: Organic Wiggle Animation
- [ ] Add uGameTime uniform for sine wave animation
- [ ] Implement procedural wiggle: sin(time - uv.y * lag) * amplitude
- [ ] Ensure tail wiggles more than head (uv.y gradient)
- [ ] (Nice-to-have) Dynamic coupling: wiggle frequency scales with velocity
- [ ] Verify ZERO performance regression vs Phase 2B
- [ ] Visual QA: creatures "swim" organically at various zoom levels

#### Phase 2D: Performance Validation & Polish
- [ ] Verify 60 FPS stable @ 1 million entities
- [ ] Confirm no visual stuttering or "rubber banding"
- [ ] Profile WebGL shader performance
- [ ] Cross-platform compatibility testing
- [ ] Collaborate with Instrumentation-Ian on GPU metrics for Dev-UI
- [ ] Document shader architecture and API

### Phase 3: Uber-Struct Refactor - Days 4-5
- [ ] Remove Catatonic component
- [ ] Add Catatonic variant to BehaviorMode enum
- [ ] Create CreatureState uber-struct
- [ ] Split hot (Transform, Physics) components
- [ ] Split cold (BiologyData) components
- [ ] Update all systems for new component structure
- [ ] Run all unit tests
- [ ] Benchmark cache hit rates

### Phase 4A: Vision Split Queries - Day 6
- [ ] Rename Perception → Vision
- [ ] Add VisionTiming component
- [ ] Add Visible marker component
- [ ] Remove Vec collection (CRITICAL)
- [ ] Implement split queries (observers + targets)
- [ ] Add FOV dot product check
- [ ] Test @ 50K creatures
- [ ] Test @ 100K creatures
- [ ] Verify zero allocations with profiler

### Phase 4B: Changed<T> Filters + Vec2 - Day 7
- [ ] Add Changed<Velocity> to rotation_system
- [ ] Audit all systems for Changed opportunities
- [ ] Replace Position with Vec2
- [ ] Replace Velocity with Vec2
- [ ] Replace Acceleration with Vec2
- [ ] Update all movement systems
- [ ] Update all vision systems
- [ ] Run all tests
- [ ] Benchmark vector math

### Phase 4C: Parallelization - Day 8
- [ ] Add par_iter_mut() to rotation_system
- [ ] Add par_iter_mut() to seek_system
- [ ] Replace thread_rng() with fastrand in wander
- [ ] Add par_iter_mut() to wander_system
- [ ] Test determinism
- [ ] Benchmark 4-core, 8-core systems
- [ ] Run @ 150K creatures
- [ ] Profile for race conditions

### Phase 4D: Performance Validation - Day 9
- [ ] Benchmark @ 50K creatures
- [ ] Benchmark @ 100K creatures
- [ ] Benchmark @ 150K creatures
- [ ] Benchmark @ 200K creatures (stretch)
- [ ] Capture hardware metrics snapshots
- [ ] Analyze bottlenecks
- [ ] Update optimization backlog docs
- [ ] Write Phase 4 completion report

### Phase 5: Final Validation - Day 11
- [ ] Visual quality check (size-based reactions)
- [ ] Test predator sneaking (FOV blind spots)
- [ ] Zoom smoothness @ 150K creatures
- [ ] Compare hardware metrics (baseline → final)
- [ ] All unit tests pass
- [ ] All integration tests pass

---

## Success Criteria

**Performance:**
- [ ] 150K creatures @ 22.2Hz sustained
- [ ] 200K creatures @ 22.2Hz (stretch)
- [ ] 60 FPS frontend rendering
- [ ] Vision <40% frame budget (was 70%)

**Behavior:**
- [ ] Size-based reaction times visible
- [ ] FOV blind spots enable sneaking
- [ ] No synchronization spikes

**Architecture:**
- [ ] Stable archetypes (no add/remove churn)
- [ ] SIMD vector math throughout
- [ ] Component-based timing (not HashMap)
- [ ] Zero Vec allocations in vision system

---

## Current Tasks

_This section will be updated as the sprint progresses._
