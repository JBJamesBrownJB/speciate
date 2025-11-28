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

## Phase 1: Uber-Struct Refactor (Days 1-2)

### Tasks
- [ ] Design uber-struct component architecture
- [ ] Implement `PhysicalTraits` (size, speed, color)
- [ ] Implement `BehaviorState` (current behavior, timers)
- [ ] Migrate existing component-per-field to uber-structs
- [ ] Verify stable archetype (no add/remove component churn)
- [ ] Run benchmarks to validate cache locality improvement

**Owner:** ecs-emma + rusty-ron

---

## Phase 2: Vision Split Queries (Day 3 - CRITICAL)

### Tasks
- [ ] Refactor vision system to use split immutable/mutable queries
- [ ] Remove Vec allocations from vision system
- [ ] Implement FOV (Field of View) limiting
- [ ] Add stochastic vision updates (not every creature every tick)
- [ ] Benchmark vision system performance (target: <5ms @ 200K)

**Owner:** ecs-emma + rusty-ron
**Validation:** instrumentation-ian, zoologist-tom (FOV biology)

---

## Phase 3: Changed<T> Filters + Vec2 (Day 4)

### Tasks
- [ ] Add `Changed<T>` filters to reduce unnecessary updates
- [ ] Migrate vector math to Vec2 (glam crate)
- [ ] Enable SIMD optimizations
- [ ] Benchmark movement system performance

**Owner:** rusty-ron + ecs-emma

---

## Phase 4: Parallelization (Day 5)

### Tasks
- [ ] Analyze system dependencies for parallel execution
- [ ] Add `par_iter()` to independent systems
- [ ] Benchmark multi-core utilization
- [ ] Validate thread safety and data races

**Owner:** rusty-ron + instrumentation-ian

---

## Phase 5: Performance Validation (Day 6)

### Tasks
- [ ] Run full simulation @ 150K creatures
- [ ] Run full simulation @ 200K creatures
- [ ] Validate 22.2Hz tick rate maintained
- [ ] Profile CPU usage and memory allocation
- [ ] Generate performance report

**Owner:** instrumentation-ian + ecs-emma

---

## Success Metrics

- [ ] 150K creatures @ 22.2Hz stable
- [ ] 200K creatures @ 22.2Hz stable
- [ ] Vision system <5ms @ 200K creatures
- [ ] All tests passing
- [ ] Zero allocations in hot path

---

## Notes

- Sprint 14 delivered GPU interpolation foundation
- Frontend ready for high entity counts
- Backend is the bottleneck (vision system Vec allocations)
- Focus on zero-allocation, cache-friendly architecture
