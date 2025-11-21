# Sprint 13: Interpolation, Vision Refactor & Data-Oriented Design

**Branch:** `feat/sprint-14-interpolation-perception`
**Duration:** 11 days
**Status:** IN PROGRESS

---

## Sprint Goal

**MORE DETAIL IN:** SPRINT_PLAN_sprint-14-interpolation-perception.md, check this to confirm your approach

Scale to 150K-200K creatures through:
1. 20Hz simulation → 60Hz interpolated rendering
2. Perception → Vision refactor (biological naming, FOV, stochastic updates)
3. Uber-struct pattern (stable archetypes, hot/cold split)
4. Vec2 vector math (SIMD optimization)

---

## Phase Checklist

### Phase 1: Lower Main Tick Rate (20Hz) - Day 1
- [ ] Change target_tick_rate to 20Hz in config
- [ ] Validate all systems use DeltaTime resource
- [ ] Test 10K creatures: <30ms avg tick
- [ ] Test 20K creatures: <40ms avg tick

### Phase 2: Frontend Interpolation (60Hz) - Days 2-3
- [ ] Add PreviousPositions resource to backend
- [ ] Update CreatureSnapshot with prev_x, prev_y, prev_rotation
- [ ] Implement cleanup system for despawned creatures
- [ ] Add interpolation alpha calculation to StateManager
- [ ] Implement position interpolation in render loop
- [ ] Implement rotation interpolation (handle wraparound)
- [ ] Test 60 FPS rendering stability
- [ ] Memory leak test

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
- [ ] 150K creatures @ 20Hz sustained
- [ ] 200K creatures @ 20Hz (stretch)
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
