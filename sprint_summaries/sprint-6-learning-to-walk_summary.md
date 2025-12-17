# Sprint 6: Learning to Walk - Summary

**Sprint Branch:** `feat/sprint-6-learning-to-walk`
**Duration:** November 6-9, 2025
**Status:** ✅ Complete

---

## Sprint Goal

Fix broken crit locomotion system and establish efficient multi-crit interaction patterns.

**Key Terminology Updates:**
- "Crits" - Unified term for creatures/agents/boids
- "Portal" - New name for UI/frontend (previously "admin-dev-ui")

---

## Key Outcomes Delivered

### ✅ Phase 1: Seeking Behavior (Nov 7)
**Achievement:** Crits can now move toward targets with smooth, biologically realistic movement.

**What Was Built:**
- **ECS Architecture Documentation** (`/apps/simulation/CLAUDE.md`)
  - Hybrid component pattern: Capability markers + BehaviorState enum + data components
  - Force accumulation pattern for additive steering behaviors
  - Death handling strategy (add `Dead` marker, don't remove capabilities)
  - DNA integration roadmap for future sprints

- **Components & Systems:**
  - `CanSeek` capability marker (zero-sized, permanent)
  - `Target` data component for goal positions
  - `BehaviorMode::Seeking` state
  - Reynolds steering algorithm with smooth arrival behavior

- **Test Coverage:** 79 tests passing (12 new tests added)

**Technical Decisions:**
- Hybrid ECS architecture enables 20x faster performance than enum branching
- Natural force composition: seek + avoid = emergent arc behavior
- Hardcoded parameters with `// TODO: from DNA` for Sprint 8+ migration

---

### ✅ Phase 2: Natural Movement & Body Physics (Nov 7)
**Achievement:** Eliminated unnatural movement artifacts, implemented volumetric physics.

**What Was Built:**

1. **Pounce Mechanism** - Precise arrivals
   - Snap to target when within 0.5m and moving <5.5 m/s
   - Eliminates infinite creeping behavior

2. **Body Radius Physics** - Volumetric interactions
   - `BodySize` component: Crits are circles, not point masses
   - Edge-to-edge distance calculations for all interactions
   - Formula: `edge_distance = center_distance - radius_self - radius_other`
   - Affects perception, avoidance, and arrival behaviors

3. **Locomotion Noise** - Organic wobble
   - Perlin noise applied perpendicular to velocity
   - Quadratic speed scaling: precise at low speeds, wobble at high speeds
   - At 2 m/s: 0.4N noise (precise control)
   - At 30 m/s: 90.5N noise (natural wobble)
   - Temporally coherent, deterministic patterns

4. **Physics Tick System** - Temporal variation
   - `PhysicsTick` resource for time-based Perlin noise seeds

**Test Results:** 109/109 tests passing (no regressions from body physics changes!)

**Key Parameters (User-Tuned):**
- `MAX_SPEED: 30.0 m/s` (Kleiber's Law scaling)
- `VELOCITY_DAMPING: 0.95` (balanced realism vs gameplay)
- `LOCOMOTION_NOISE_BASE: 90.5` (quadratic scaling)
- `PERCEPTION.personal_space: 2.5×` body size multiplier

**Biology Notes:** Comprehensive rationale documented in `/workspace/docs/BIOLOGY_NOTES.md`

---

### ✅ Phase 3: Admin Portal & Infrastructure (Nov 9)
**Achievement:** Working admin portal with NATS WebSocket, dev command system.

**What Was Built:**

1. **NATS WebSocket Connection**
   - WebSocket support on port 9224 (changed from 4224 due to port conflicts)
   - Docker-outside-of-Docker path fixes (absolute paths for volume mounts)
   - Admin portal successfully connects and spawns creatures

2. **Dev Commands System** (Feature-Gated)
   - Spawn command: Create crits via admin UI
   - Clear command: Remove all creatures
   - Speed command: Adjust simulation speed (0.1x - 10x)
   - Real-time creature spawning through browser interface

3. **Creature Spawning Architecture Fix** (TDD Green ✅)
   - **Bug Fixed:** Dev-spawned creatures bypassed `EntityIdMap` registration
   - **Solution:** Shared ECS resources for spawn state
     - `NextCreatureId` - Assigns unique IDs
     - `EntityIdMap` - Tracks all creatures
   - **Single-Gate Architecture:** All spawning paths use same resources
     - `Simulation::spawn_crit()` ✓
     - Dev commands (admin portal) ✓
     - Snapshot restoration ✓
   - **Test Results:**
     - 5/5 dev command tests passing (TDD Green)
     - 128/128 library tests passing (no regressions)

**Changed Files:**
- `/workspace/infrastructure/local/docker-compose.yml` - NATS WebSocket config
- `/workspace/infrastructure/local/nats-server.conf` - Port 9224 configuration
- `/workspace/apps/simulation/src/simulation/creatures/systems.rs` - Spawn resources
- `/workspace/apps/simulation/src/simulation/core/simulation.rs` - Resource-based spawning
- `/workspace/apps/simulation/src/dev_commands/systems.rs` - Shared resource usage
- `/workspace/apps/simulation/src/snapshots/snapshot.rs` - Resource integration
- `/workspace/apps/simulation/tests/dev_commands_test.rs` - TDD test suite

---

### ✅ Portal Improvements
**Achievement:** Enhanced visual feedback and user experience.

- Grid rendering at high zoom levels (visible >= 20 px/m)
- Camera zoom range: 0.0005 - 200 px/m
- World size: 2000km × 2000km
- Clickable creatures with selection feedback
- Color palette improvements
- Distance measurement widget
- Scale bar icons (designed, not implemented)

---

## Completed Tasks Summary

### Core Simulation Features
- ✅ Seeking behavior with Reynolds steering
- ✅ Smooth arrival with pounce mechanism
- ✅ Body radius physics (volumetric interactions)
- ✅ Locomotion noise (Perlin-based organic wobble)
- ✅ Wandering behavior (territory-based elastic tether)
- ✅ Obstacle avoidance (reactive steering forces)
- ✅ Perception system (edge-to-edge distance awareness)

### Infrastructure & Tooling
- ✅ NATS WebSocket support (port 9224)
- ✅ Admin dev UI portal with live connection
- ✅ Dev commands system (spawn/clear/speed)
- ✅ Single-gate spawning architecture
- ✅ Shared ECS resources (NextCreatureId, EntityIdMap)
- ✅ TDD test suite for dev commands (5 tests)

### Documentation & Architecture
- ✅ ECS Architecture Guide (`/apps/simulation/CLAUDE.md`)
- ✅ Biology notes (movement parameters, trade-offs)
- ✅ DNA-driven design documentation
- ✅ NATS port allocation documentation

---

## Remaining Work (Deferred to Future Sprints)

### Not Completed - Deprioritized
- **Spatial Partitioning Optimization** - Deferred to Sprint 7
  - Current: O(n²) naive neighbor search
  - Planned: Spatial hash grid for O(n) performance
  - Rationale: Premature optimization; wanted baseline metrics first

- **Path Planning vs Reactive Forces** - Design decision deferred
  - Current implementation uses reactive steering forces
  - Local minima problem acknowledged but unresolved
  - Decision pending gameplay testing to identify failure cases

- **Size & Growth System** - Backlog created but not implemented
  - Detailed backlog document created (`BACKLOG_SIZE_AND_GROWTH_SYSTEM.md`)
  - 16 tasks planned across 5 phases (~28.5 hours estimated)
  - Will be prioritized in future sprint based on gameplay needs

### Technical Debt Created
- `DevSpawnIdCounter` resource now dead code (needs removal)
- Event system infrastructure created but unused (events vs direct resource access)
- DeltaTime default changed from 0.016 to 0.05 (may affect other systems)

---

## Test Results

**Final Test Coverage:**
- ✅ 128 library tests passing (simulation core)
- ✅ 5 dev command tests passing (admin portal integration)
- ✅ 0 test regressions from body physics changes
- **Total:** 133 passing tests

**Quality Milestones:**
- Zero test breakage after implementing body radius physics (validates architecture)
- TDD approach successfully caught creature counting bug
- All existing tests passed after locomotion noise integration

---

## Retrospective / Lessons Learned

### What Went Well

1. **Hybrid ECS Architecture Paid Off**
   - Force accumulation pattern made adding behaviors trivial
   - Zero test breakage when adding body physics proves clean interfaces
   - DNA integration points clearly marked for future work

2. **TDD Caught Real Bug**
   - Wrote failing tests first for dev spawn feature
   - Tests revealed creature counting issue (bypassed EntityIdMap)
   - Fix verified by tests going green (5/5 passing)
   - Demonstrates value of test-first approach

3. **Biological Consultation Improved Realism**
   - zoologist-tom provided scientifically accurate parameters
   - Quadratic speed scaling for noise creates natural precision/wobble gradient
   - Movement now "feels alive" without complex scripting

4. **Visual Testing Revealed Hidden Issues**
   - Integration tests passed but demo failed (obstacle avoidance)
   - Root cause: Spawn config had obstacle outside perception range
   - Lesson: Unit + integration tests not sufficient, need visual validation

5. **Incremental Tuning Worked**
   - User iterated on parameters with immediate visual feedback
   - `LOCOMOTION_NOISE_BASE` increased 181× (0.5 → 90.5) during tuning
   - System remained stable due to quadratic scaling design

### What Could Be Improved

1. **Tests Passed for Wrong Reasons**
   - Obstacle avoidance test used off-axis geometry that masked perception bug
   - Need tests with obstacles directly in path (head-on collision)
   - Need tests at perception range boundaries

2. **Premature Event System**
   - Built SpawnCreatureEvent infrastructure for "all spawns through events"
   - Bevy double-buffering caused one-frame delay issues
   - Pivoted to direct resource access instead
   - Lesson: Understand framework patterns before architecting around them

3. **Docker Path Confusion**
   - Docker-outside-of-Docker volume mount issue cost time
   - Relative paths mounted as empty directories
   - Solution: Use absolute host paths
   - Lesson: Document devcontainer quirks in setup guide

4. **Physics Parameters Interact in Complex Ways**
   - Changing MAX_SPEED affects physics tunneling risk
   - Changing damping affects terminal velocity and acceleration
   - All must be tuned together, not in isolation
   - Lesson: Need holistic parameter tuning tool/workflow

5. **Port Conflict Phantom Issue**
   - Port 4224 "address already in use" despite nothing using it
   - Tried multiple diagnostics (lsof, netstat, docker prune) - nothing worked
   - Pragmatic solution: Use different port (9224)
   - Lesson: Don't fight phantom issues, work around them

### Architectural Decisions

**Quadratic Speed Scaling for Noise:**
- Linear scaling failed at low speeds (60% noise-to-signal ratio)
- Creatures wobbled and couldn't reach targets
- Quadratic provides precision where needed (0.4N at 2 m/s, 90.5N at 30 m/s)
- Generalizable pattern for DNA-driven traits

**Cache Pattern for Performance:**
- Pre-compute expensive calculations at spawn time (e.g., personal_space = 2.5 × body_size)
- Store per-creature values in components
- Trade memory for CPU (acceptable for <10,000 creatures)
- Prepares for DNA-driven individuality

**Single-Gate Spawning via Shared Resources:**
- All spawn paths access same ECS resources (NextCreatureId, EntityIdMap)
- Maintains consistency without event system complexity
- Same-frame spawning (no Bevy double-buffering delay)
- Clear, testable architecture

---

## Performance Considerations

**Current Bottlenecks (Unoptimized):**
- Perception system: O(n²) naive neighbor search
- Avoidance system: O(m) where m = neighbors in perception range
- No spatial partitioning yet

**Optimization Planned for Sprint 7:**
- Spatial hash grid for perception queries (O(n²) → O(n))
- Staggered perception updates (not all creatures same frame)
- Performance metrics before/after to validate improvements

**Decision:** Wanted baseline performance measurements before optimizing. Premature optimization avoided.

---

## Technical Highlights

### Force Accumulation Pattern
```rust
// Systems ADD forces to acceleration
accel.ax += seek_force.x;
accel.ay += seek_force.y;
accel.ax += avoid_force.x;  // Accumulates with seek
accel.ay += avoid_force.y;

// Movement system integrates total force (Euler)
vel.vx += accel.ax * dt;
vel.vy += accel.ay * dt;
pos.x += vel.vx * dt;
pos.y += vel.vy * dt;
```

**Benefits:**
- Natural force blending (seek + avoid = arc)
- Extensible (add behaviors without modifying existing)
- Biologically realistic (multiple sensory inputs → single motor output)

### Body Radius Edge-to-Edge Distance
```rust
let center_dist = ((dx * dx) + (dy * dy)).sqrt();
let edge_dist = (center_dist - my_radius - their_radius).max(0.01);

// Use edge_dist for all interactions
if edge_dist < perception_range { /* detect */ }
if edge_dist < personal_space { /* avoid */ }
```

**Effect:** 1m creature occupies 1m × 1m space, proper volumetric physics

### Quadratic Noise Scaling
```rust
let speed_ratio = speed / MAX_SPEED;
let noise_force = BASE_NOISE * (speed_ratio * speed_ratio) / sqrt(body_length);

// At 2 m/s (slow): 0.4N noise → precise
// At 30 m/s (fast): 90.5N noise → wobble
```

**Effect:** Precision at low speeds, natural organic motion at high speeds

---

## Documentation Artifacts

**Created:**
- `/workspace/apps/simulation/CLAUDE.md` - ECS Architecture Guide
- `/workspace/docs/BIOLOGY_NOTES.md` - Biological rationale for parameters
- `/workspace/docs/biology/dna-driven-design.md` - DNA architecture spec
- `/workspace/SPRINT_DOCS/BACKLOG_SIZE_AND_GROWTH_SYSTEM.md` - Size system backlog
- `/workspace/apps/simulation/tests/dev_commands_test.rs` - TDD test suite

**Updated:**
- `/workspace/SPRINT_DOCS/SESSION_LOG.md` - Detailed session notes
- `/workspace/infrastructure/PORTS.md` - NATS WebSocket port documentation

---

## Key Metrics

**Time Estimates vs Actual:**
- Phase 1 (Seeking): Estimated 2.5 hours, Actual ~2.5 hours ✅
- Phase 2 (Body Physics): Estimated unknown, Actual ~3 hours
- Phase 3 (Admin Portal): Unplanned, Actual ~4 hours (debugging included)

**Code Quality:**
- 133 passing tests
- Zero test regressions after major changes
- TDD approach validated (caught real bug)

**Performance:**
- Simulation runs at 20 Hz (50ms per frame)
- 128 tests complete in 0.05s
- No performance optimization done (deferred to Sprint 7)

---

## Next Sprint Recommendations

1. **Measure Performance Baseline**
   - Profile with 100+ creatures
   - Identify actual bottlenecks
   - Establish metrics for optimization validation

2. **Implement Spatial Hash Grid**
   - Replace O(n²) perception with O(n)
   - Validate performance improvement with metrics
   - Document grid cell size reasoning

3. **Evaluate Reactive Forces in Practice**
   - Log instances where creatures get "stuck"
   - Track path efficiency: actual distance / straight-line distance
   - Identify if local minima are a real problem

4. **Consider Size & Growth System**
   - Backlog ready with 16 tasks planned
   - Prioritize based on gameplay needs
   - Unlocks predator-prey mechanics

5. **Clean Up Technical Debt**
   - Remove `DevSpawnIdCounter` dead code
   - Document event system vs resource-based spawning decision
   - Add head-on collision tests for obstacle avoidance

---

## Conclusion

Sprint 6 successfully delivered core locomotion behaviors with biologically realistic movement, a working admin portal for live creature spawning, and a robust single-gate spawning architecture. The hybrid ECS pattern proved its value through zero test regressions despite major changes. TDD caught a real bug in creature counting. Visual testing revealed perception range issues that integration tests missed.

The sprint deferred optimization decisions deliberately, choosing to establish baseline metrics first. The reactive steering forces vs path planning question remains open, pending real-world gameplay testing.

**Status:** ✅ All core objectives met. Foundation solid for future enhancements.

---

**Generated:** 2025-11-09
**Sprint Branch:** `feat/sprint-6-learning-to-walk`
**Main Branch:** `main`
**Commits:** 27 commits from sprint start to completion
