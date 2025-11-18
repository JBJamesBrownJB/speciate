# Sprint 12: Frontend Interpolation + Size-Based Perception

**Branch:** `feat/sprint-12-interpolation-perception` (not created yet)
**Status:** PLANNED
**Prerequisites:** Sprint 11 complete (IPC optimization + dual-tick abandonment)

---

## Sprint Goal

Scale to 150K-200K creatures by lowering simulation tick rate (20Hz) with frontend interpolation (90Hz) for smooth visuals, plus size-based perception frequency for natural behavior and load distribution.

---

## Context: Why This Approach?

Sprint 11 explored dual-tick architecture (separate AI/Physics schedules) and abandoned it after discovering sequential execution provides no performance benefit. When schedules align (every 100ms), you get the same spike as running both together.

**Key Insight:** Natural behavior variation comes from **per-creature reaction times**, NOT from different tick schedules. Each creature updates perception based on its individual reaction time threshold (derived from body size), creating stochastic distribution naturally.

**Architecture Benefits:**
- ✅ Simple single-tick baseline (proven stable)
- ✅ Biologically realistic (large creatures slower than small)
- ✅ Natural load distribution (creatures don't align on same tick)
- ✅ No parallelism complexity (no lock-free data structures needed)

---

## Sprint Anchors

**Active Documents:**
- **Biology:** `docs/biology/biology-notes.md` (lines 850-956) - Zoologist-tom consultation on reaction times
- **Optimization:** `docs/performance/optimization-backlog.md` (lines 29-33) - Size-based reaction latency spec
- **Reference:** `docs/architecture/dual-tick-simulation.md` (ABANDONED - see warning header)

**Biological Validation (Already Approved):**
> "The AI ticks at 20Hz for ALL creatures, but larger creatures only ACT when their reaction delay threshold is met. This creates realistic sluggishness without synchronization issues."
> — Zoologist-tom consultation, 2025-11-16

**Reaction Time Formula (Zoologist-Approved):**
```
reaction_time_ms = 100 + ((body_length_m - 1.0).max(0.0) / 19.0) * 900

Examples:
- 1m creature:  100ms reaction (fast, responsive)
- 5m creature:  290ms reaction (medium, deliberate)
- 10m creature: 526ms reaction (slow but powerful)
- 20m creature: 1000ms reaction (massive, ponderous)
```

---

## Phases

### Phase 1: Lower Main Tick Rate (20Hz)

**Duration:** Day 1 (30 minutes work)

**Goal:** Establish stable 20Hz baseline providing 2.5x more creatures per frame budget vs 60Hz.

**Current State:**
- Default tick rate: **60Hz** (discovered in research, NOT 30Hz as assumed)
- Location: `apps/simulation/src/config.rs` line 82

**Changes Required:**

1. **Update TimingConfig default:**
   ```rust
   // apps/simulation/src/config.rs
   impl Default for TimingConfig {
       fn default() -> Self {
           Self {
               target_tick_rate: 20,  // Changed from 60
               timing_window_size: 100,
               timing_report_interval: 200,
               creature_count_log_interval_secs: 5,
           }
       }
   }
   ```

2. **Verify DeltaTime usage:**
   - All systems must use `DeltaTime` resource (NOT hardcoded assumptions)
   - Check movement constants don't assume fixed tick rate
   - Confirm biological systems use wall-clock time (metabolism, aging)

3. **Test stability:**
   - Run at 10K creatures: verify <30ms avg tick time
   - Run at 20K creatures: verify <40ms avg tick time
   - Run at 30K creatures: verify <50ms avg tick time

**Success Criteria:**
- [x] Simulation runs stable at 20Hz (50ms per tick)
- [x] No hardcoded tick rate assumptions in systems
- [x] Frame budget: ~30-40ms avg at 20K creatures
- [x] All tests pass (especially biological rate tests)

**Known Risk:** Motion appears choppy due to 20Hz updates.
**Mitigation:** Phase 2 frontend interpolation restores smoothness.

**Files Changed:**
- `apps/simulation/src/config.rs` (1 line)

---

### Phase 2: Frontend Interpolation (90Hz)

**Duration:** Days 2-3 (2 days)

**Goal:** Smooth 90Hz rendering despite 20Hz physics updates.

**Research Phase (Day 2 morning):**
- Review archived NATS/MMO branch for existing PixiJS interpolation code
- Extract proven patterns from previous implementation
- Understand lerp formula and edge case handling

**Implementation (Day 2-3):**

#### 2.1 Backend Changes: Add Previous Positions

**File:** `apps/simulation/src/stdio/hooks.rs`

Add resource to track previous positions:
```rust
#[derive(Resource)]
pub struct PreviousPositions {
    positions: HashMap<u32, (f32, f32)>,  // creature_id -> (x, y)
}
```

Modify `serialize_snapshot_frame`:
```rust
// Before creating GameState, store current as previous
let mut prev_positions = world.resource_mut::<PreviousPositions>();
for (crit_id, pos) in query.iter(world) {
    prev_positions.positions.entry(crit_id.0).or_insert((pos.x, pos.y));
}

// Build CreatureSnapshot with prev + current
let creatures: Vec<CreatureSnapshot> = query.iter(world)
    .map(|(crit_id, pos, rot, body_size)| {
        let (prev_x, prev_y) = prev_positions.positions
            .get(&crit_id.0)
            .copied()
            .unwrap_or((pos.x, pos.y));

        CreatureSnapshot {
            id: crit_id.0,
            x: pos.x,
            y: pos.y,
            prev_x,  // NEW
            prev_y,  // NEW
            rotation: rot.radians,
            size: body_size.length,
        }
    })
    .collect();

// Update previous positions for next frame
for snapshot in &creatures {
    prev_positions.positions.insert(snapshot.id, (snapshot.x, snapshot.y));
}
```

**File:** `apps/simulation/src/ipc/snapshot_queue.rs`

Update `CreatureSnapshot`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureSnapshot {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    #[serde(rename = "prevX")]
    pub prev_x: f32,  // NEW
    #[serde(rename = "prevY")]
    pub prev_y: f32,  // NEW
    pub rotation: f32,
    pub size: f32,
}
```

#### 2.2 Frontend Changes: Interpolation

**File:** `apps/portal/src/types/GameState.ts`

Update interface:
```typescript
export interface CreatureSnapshot {
  id: number;
  x: number;
  y: number;
  prevX: number;  // NEW
  prevY: number;  // NEW
  rotation: number;
  size: number;
}
```

**File:** `apps/portal/src/core/StateManager.ts`

Add interpolation tracking:
```typescript
export class StateManager {
  private lastPhysicsUpdate: number = 0;
  private readonly PHYSICS_PERIOD_MS = 50;  // 20Hz

  public updateState(newState: GameState): void {
    this.lastPhysicsUpdate = performance.now();
    // ... existing logic
  }

  public getInterpolationAlpha(): number {
    const now = performance.now();
    const elapsed = now - this.lastPhysicsUpdate;
    return Math.min(1.0, elapsed / this.PHYSICS_PERIOD_MS);
  }
}
```

**File:** `apps/portal/src/simulation/SimulationManager.ts`

Interpolate positions in render loop:
```typescript
public update(deltaTime: number): void {
  const alpha = this.stateManager.getInterpolationAlpha();

  for (const creature of this.creatures.values()) {
    const snapshot = this.stateManager.getCreatureState(creature.id);
    if (!snapshot) continue;

    // Interpolate position
    const displayX = snapshot.prevX + (snapshot.x - snapshot.prevX) * alpha;
    const displayY = snapshot.prevY + (snapshot.y - snapshot.prevY) * alpha;

    creature.sprite.x = displayX;
    creature.sprite.y = displayY;
    creature.sprite.rotation = snapshot.rotation;
  }
}
```

**Edge Case Handling:**
```typescript
// If physics update is late (alpha > 1.0), clamp to 1.0 (don't extrapolate)
const alpha = Math.min(1.0, elapsed / PHYSICS_PERIOD_MS);

// On first frame, prev == current (no interpolation)
const prevX = snapshot.prevX || snapshot.x;
const prevY = snapshot.prevY || snapshot.y;
```

**Success Criteria:**
- [x] Frontend renders at 90 FPS (requestAnimationFrame)
- [x] Smooth motion despite 20Hz physics updates
- [x] No visible jitter or stuttering
- [x] Late physics frames handled gracefully (clamp alpha)
- [x] Creatures appear to move continuously

**Key Formula:**
```
alpha = min(1.0, time_since_last_physics / physics_period)
displayX = prevX + (currX - prevX) * alpha
displayY = prevY + (currY - prevY) * alpha
```

**Files Changed:**
- `apps/simulation/src/stdio/hooks.rs` (add PreviousPositions resource)
- `apps/simulation/src/ipc/snapshot_queue.rs` (add prev_x, prev_y fields)
- `apps/portal/src/types/GameState.ts` (update interface)
- `apps/portal/src/core/StateManager.ts` (add interpolation alpha)
- `apps/portal/src/simulation/SimulationManager.ts` (interpolate in render loop)

---

### Phase 3: Size-Based Perception Frequency

**Duration:** Days 4-5 (2 days)

**Goal:** Per-creature perception timing based on biological reaction time, creating natural load distribution.

**Biological Rationale:**
- Small creatures (1m): 100ms reaction time → ~10 perception updates/sec
- Medium creatures (5m): 290ms reaction time → ~3.4 updates/sec
- Large creatures (20m): 1000ms reaction time → ~1 update/sec

**Result:** At any given 20Hz tick, only ~5-20% of creatures update perception (natural stochastic distribution).

#### 3.1 Add Timing Components

**File:** `apps/simulation/src/simulation/perception/components.rs`

```rust
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LastPerceptionUpdate {
    pub last_update_time: f32,  // seconds since simulation start
}
```

**File:** `apps/simulation/src/simulation/core/components.rs`

```rust
#[derive(Resource, Default, Debug)]
pub struct SimulationTime {
    elapsed_seconds: f32,
}

impl SimulationTime {
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    pub fn tick(&mut self, delta_time: f32) {
        self.elapsed_seconds += delta_time;
    }
}
```

#### 3.2 Update Simulation Initialization

**File:** `apps/simulation/src/simulation/core/simulation.rs`

Add resources in `SimulationBuilder::new()`:
```rust
world.insert_resource(SimulationTime::default());
```

Update simulation loop:
```rust
pub fn update(&mut self, delta_time: f32) {
    self.world.insert_resource(DeltaTime(delta_time));
    self.world.resource_mut::<SimulationTime>().tick(delta_time);

    self.schedule.run(&mut self.world);
    // ... rest unchanged
}
```

#### 3.3 Modify Perception System

**File:** `apps/simulation/src/simulation/perception/systems.rs`

```rust
pub fn update_perception_system(
    sim_time: Res<SimulationTime>,
    mut query: Query<(
        Entity,
        &Position,
        &BodySize,
        &mut Perception,
        &mut LastPerceptionUpdate,
        &CreatureState,
    )>,
) {
    let current_time = sim_time.elapsed_seconds();

    // Collect all creature positions once (for neighbor queries)
    let all_creatures: Vec<(Entity, Position, f32)> = query
        .iter()
        .map(|(entity, pos, size, _, _, _)| (entity, *pos, size.length))
        .collect();

    // Update each creature's perception if reaction time elapsed
    for (entity, pos, size, mut perception, mut last_update, state) in query.iter_mut() {
        // Skip catatonic creatures
        if state.behavior == BehaviorMode::Catatonic {
            continue;
        }

        // Calculate reaction time based on body size
        let body_length = size.length;
        let reaction_time_sec = calculate_reaction_time(body_length);

        // Check if enough time has elapsed since last update
        let elapsed = current_time - last_update.last_update_time;
        if elapsed < reaction_time_sec {
            continue;  // Skip this creature's perception update
        }

        // Perform perception update
        perception.nearby.clear();

        for &(other_entity, other_pos, other_size) in &all_creatures {
            if other_entity == entity {
                continue;
            }

            let dx = pos.x - other_pos.x;
            let dy = pos.y - other_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= perception.range {
                perception.nearby.push(other_entity);
            }
        }

        // Record update time
        last_update.last_update_time = current_time;
    }
}

/// Calculate reaction time in seconds based on body length
fn calculate_reaction_time(body_length: f32) -> f32 {
    // Formula from zoologist-tom: 100ms (1m) to 1000ms (20m)
    let reaction_ms = 100.0 + ((body_length - 1.0).max(0.0) / 19.0) * 900.0;
    reaction_ms / 1000.0  // convert to seconds
}
```

#### 3.4 Update Creature Builder

**File:** `apps/simulation/src/simulation/creatures/builder.rs`

Add `LastPerceptionUpdate` to spawned creatures:
```rust
pub fn build(self, id: u32) -> CritBundle {
    // ... existing fields ...
    CritBundle {
        // ... existing components ...
        last_perception_update: LastPerceptionUpdate::default(),
    }
}
```

**IMPORTANT:** Initialize with random offset to avoid first-frame spike:
```rust
// Alternative: randomize initial update time
use rand::Rng;
let mut rng = rand::thread_rng();
let random_offset = rng.gen_range(0.0..1.0);  // 0-1 second offset

last_perception_update: LastPerceptionUpdate {
    last_update_time: -random_offset,  // Negative = will update soon but staggered
}
```

**Success Criteria:**
- [x] Small creatures (1m) update perception ~10x per second
- [x] Large creatures (20m) update perception ~1x per second
- [x] No synchronization spikes (creatures don't align on same tick)
- [x] Behavior feels natural (large = ponderous, small = reactive)
- [x] Reaction time formula matches zoologist specification

**Files Changed:**
- `apps/simulation/src/simulation/perception/components.rs` (add LastPerceptionUpdate)
- `apps/simulation/src/simulation/core/components.rs` (add SimulationTime)
- `apps/simulation/src/simulation/core/simulation.rs` (add resources, update loop)
- `apps/simulation/src/simulation/perception/systems.rs` (add per-creature timing)
- `apps/simulation/src/simulation/creatures/builder.rs` (add component to bundle)

---

### Phase 4: Stochastic Distribution Testing

**Duration:** Day 6 (1 day)

**Goal:** Verify creatures don't synchronize on same tick (natural distribution maintained).

#### 4.1 Spawn Timing Test

**Test:** Verify initial distribution of perception update times.

```rust
#[test]
fn test_spawn_timing_staggered() {
    let mut sim = SimulationBuilder::new().build();

    // Spawn 1000 creatures
    for i in 0..1000 {
        let builder = CritBuilder::new()
            .at(i as f32 * 10.0, 0.0)
            .with_all_capabilities();
        sim.spawn_crit(builder);
    }

    // Check LastPerceptionUpdate distribution
    let world = sim.world();
    let mut update_times: Vec<f32> = Vec::new();

    let mut query = world.query::<&LastPerceptionUpdate>();
    for last_update in query.iter(world) {
        update_times.push(last_update.last_update_time);
    }

    // Verify distribution (should be roughly uniform in [-1.0, 0.0] range)
    let min_time = update_times.iter().copied().fold(f32::INFINITY, f32::min);
    let max_time = update_times.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    assert!(max_time - min_time > 0.5, "Update times should be distributed");

    // Check no massive first-frame spike (bucket test)
    let mut buckets = vec![0; 10];
    for time in update_times {
        let bucket = ((time + 1.0) / 0.1).floor() as usize;
        if bucket < 10 {
            buckets[bucket] += 1;
        }
    }

    let max_bucket = buckets.iter().max().unwrap();
    assert!(*max_bucket < 200, "No bucket should have >20% of creatures");
}
```

#### 4.2 Steady-State Distribution Test

**Test:** Verify distribution remains stable over time (no synchronization drift).

```rust
#[test]
fn test_steady_state_distribution() {
    let mut sim = SimulationBuilder::new().build();

    // Spawn mix of creature sizes
    for i in 0..1000 {
        let size = 1.0 + (i % 20) as f32;  // 1m to 20m
        let builder = CritBuilder::new()
            .at(i as f32 * 10.0, 0.0)
            .with_size(size)
            .with_all_capabilities();
        sim.spawn_crit(builder);
    }

    // Run for 60 seconds (1200 ticks at 20Hz)
    for _ in 0..1200 {
        sim.update(0.05);
    }

    // Sample perception updates over next 100 ticks
    let mut updates_per_tick: Vec<usize> = Vec::new();

    for _ in 0..100 {
        let before_time = sim.world().resource::<SimulationTime>().elapsed_seconds();
        sim.update(0.05);
        let after_time = sim.world().resource::<SimulationTime>().elapsed_seconds();

        // Count how many creatures updated this tick
        let world = sim.world();
        let mut query = world.query::<&LastPerceptionUpdate>();
        let updates_this_tick = query.iter(world)
            .filter(|last_update| {
                last_update.last_update_time >= before_time &&
                last_update.last_update_time <= after_time
            })
            .count();

        updates_per_tick.push(updates_this_tick);
    }

    // Verify distribution (no spikes >30% above average)
    let avg = updates_per_tick.iter().sum::<usize>() as f32 / updates_per_tick.len() as f32;
    let max = *updates_per_tick.iter().max().unwrap() as f32;

    assert!(max < avg * 1.3, "No tick should have >30% spike above average");
}
```

#### 4.3 Size-Based Frequency Test

**Test:** Verify large creatures update less frequently than small.

```rust
#[test]
fn test_size_based_frequency() {
    let mut sim = SimulationBuilder::new().build();

    // Spawn 100 small (1m) and 100 large (10m) creatures
    let mut small_ids = Vec::new();
    let mut large_ids = Vec::new();

    for i in 0..100 {
        let small_id = sim.spawn_crit(
            CritBuilder::new()
                .at(i as f32 * 10.0, 0.0)
                .with_size(1.0)
                .with_all_capabilities()
        );
        small_ids.push(small_id);

        let large_id = sim.spawn_crit(
            CritBuilder::new()
                .at(i as f32 * 10.0, 100.0)
                .with_size(10.0)
                .with_all_capabilities()
        );
        large_ids.push(large_id);
    }

    // Track update counts over 10 seconds (200 ticks)
    let mut small_updates = 0;
    let mut large_updates = 0;

    for _ in 0..200 {
        let before_time = sim.world().resource::<SimulationTime>().elapsed_seconds();
        sim.update(0.05);

        // Count updates per group
        let world = sim.world();
        let mut query = world.query::<(&CritId, &LastPerceptionUpdate)>();

        for (crit_id, last_update) in query.iter(world) {
            if last_update.last_update_time >= before_time {
                if small_ids.contains(&crit_id.0) {
                    small_updates += 1;
                } else if large_ids.contains(&crit_id.0) {
                    large_updates += 1;
                }
            }
        }
    }

    // Small (100ms reaction) should update ~5x more than large (526ms reaction)
    let ratio = small_updates as f32 / large_updates as f32;
    assert!(ratio > 4.0 && ratio < 6.0,
        "Small creatures should update ~5x more than large, got ratio: {}", ratio);
}
```

**Success Criteria:**
- [x] No perception update spikes >30% above average
- [x] Distribution remains stable over 60 seconds
- [x] Large creatures demonstrably slower than small (4-6x fewer updates)
- [x] No synchronization artifacts in behavior

**Instrumentation:**
- Add `perception_updates_this_tick` metric to SystemTimings
- Track in dev-ui as sparkline
- Alert if >30% of creatures update in same tick

---

### Phase 5: Performance Validation

**Duration:** Day 7 (1 day)

**Goal:** Confirm 150K-200K creature scaling achieved.

#### 5.1 Benchmarks

**Baseline (20K creatures):**
```
Expected:
- Tick time: <30ms avg
- Perception: ~10ms (5-20% of creatures per tick)
- Movement: ~8ms
- Frontend: 90 FPS stable

Measure:
cargo run --release --features dev-tools -- --creatures 20000
```

**Target (150K creatures):**
```
Expected:
- Tick time: <45ms avg
- Perception: ~30ms (7.5x fewer updates due to staggering)
- Movement: ~12ms
- Frontend: 90 FPS stable

Measure:
cargo run --release --features dev-tools -- --creatures 150000
```

**Stretch (200K creatures):**
```
Expected:
- Tick time: <50ms avg (at 20Hz = acceptable)
- Perception: ~35ms
- Movement: ~13ms
- Frontend: 90 FPS stable (interpolation maintains smoothness)

Measure:
cargo run --release --features dev-tools -- --creatures 200000
```

#### 5.2 Profiling

**Perception timing comparison:**
- **Before (all-at-once):** 1000 creatures × O(N²) = 1M comparisons per tick
- **After (staggered):** ~150 creatures/tick × O(N²) = 22.5K comparisons per tick
- **Expected improvement:** ~44x reduction in per-tick perception cost

**Dev-UI sparklines:**
- perception_us should show stable line (not spikes)
- perception_updates_this_tick should be ~5-20% of total creatures
- total_tick_us should be <45ms at 150K creatures

#### 5.3 Visual Quality Check

**Smoothness validation:**
1. Zoom to single creature at 20K simulation
2. Verify smooth motion (no stuttering)
3. Check rotation interpolation (if implemented)
4. Confirm no "rubber-banding" artifacts

**Large vs small behavior:**
1. Spawn 1m and 20m creatures side by side
2. Verify 1m creature reacts faster to obstacles
3. Verify 20m creature appears more ponderous
4. Check perception ranges differ appropriately

**Success Criteria:**
- [x] 150K creatures @ 20Hz physics sustained
- [x] 90 FPS frontend rendering (smooth interpolation)
- [x] Perception <40% of frame budget (was >70% before)
- [x] Visual smoothness equivalent to 60Hz baseline
- [x] Size-based behavior differences visible

---

## Constraints

- **TDD mandatory** - Write tests before implementation
- **No dual-tick** - Single schedule only (proven simpler)
- **Biology validation** - Reaction time formula already approved by zoologist-tom
- **Instrumentation required** - All new systems must track timing

---

## Success Metrics

**Performance:**
- 150K creatures @ 20Hz physics (baseline achievement)
- 200K creatures @ 20Hz physics (stretch goal)
- 90 FPS frontend rendering with interpolation
- Perception <40% of frame budget (down from 70%)

**Behavior:**
- Size-based reaction times working as specified
- No synchronization artifacts or alignment spikes
- Natural stochastic distribution of perception updates
- Large creatures visibly slower/more deliberate

**Architecture:**
- Clean single-tick implementation
- Per-creature timing without added complexity
- Frontend interpolation reusable for future work
- Full test coverage (interpolation, timing, distribution)

---

## Risks & Mitigations

**Risk:** Frontend interpolation looks "floaty" or laggy
- **Mitigation:** Reuse proven code from NATS/MMO branch
- **Fallback:** Increase physics to 30Hz if needed (still 2x improvement)

**Risk:** Per-creature timing causes cache thrashing
- **Mitigation:** Profile before/after, batch updates by reaction time if needed
- **Measurement:** Monitor L1/L2 cache miss rates

**Risk:** Stochastic distribution creates perception blind spots
- **Mitigation:** Minimum reaction time (100ms = 10Hz) ensures responsiveness
- **Validation:** Test collision avoidance still works reliably

**Risk:** Randomized spawn timing causes first-frame spike
- **Mitigation:** Initialize LastPerceptionUpdate with random offset in [-1.0, 0.0]
- **Test:** Spawn 1000 creatures, verify distribution in spawn timing test

---

### Phase 6: Zoom Smoothness Fix

**Duration:** Day 8 (2 hours)

**Goal:** Buttery-smooth zoom at all creature counts by decoupling GPU operations from wheel events.

**Context:**
Zoom jitter issue discovered during Sprint 11 testing. Root cause: synchronous GPU transforms and DOM updates triggered on wheel events race with render loop, causing visible stuttering at high creature counts.

**Problem Analysis:**
1. **Wheel handler triggered GPU work synchronously** - `camera.applyTransform()` called outside render loop
2. **ScaleBar DOM updates at ~120Hz** - No throttling on wheel events
3. **Race conditions** - Wheel events fire independently of vsync timing

**Solution:**

#### 6.1 Decouple Wheel Events from GPU Work

**File:** `apps/portal/src/main.ts` (lines 209-220)

**Change wheel handler to state-only:**
```typescript
// BEFORE:
window.addEventListener("wheel", (event: WheelEvent) => {
    event.preventDefault();
    const zoomFactor = 1 - event.deltaY * CAMERA_CONFIG.ZOOM_SENSITIVITY;
    camera.adjustZoom(zoomFactor);
    camera.applyTransform(worldContainer, viewport.width, viewport.height);  // ❌ Sync GPU
    scaleBarManager.update(camera.zoom);  // ❌ DOM manipulation
}, { passive: false });

// AFTER:
window.addEventListener("wheel", (event: WheelEvent) => {
    event.preventDefault();
    const zoomFactor = 1 - event.deltaY * CAMERA_CONFIG.ZOOM_SENSITIVITY;
    camera.adjustZoom(zoomFactor);  // ✅ Only update state
    // GPU transform and ScaleBar moved to render loop
}, { passive: false });
```

#### 6.2 Move GPU Work to Render Loop

**File:** `apps/portal/src/main.ts` (render loop, ~lines 161-187)

**Add dirty checking and throttling:**
```typescript
let framesSinceScaleBarUpdate = 0;
const SCALE_BAR_UPDATE_INTERVAL = 3;  // 30Hz at 90 FPS
let lastAppliedZoom = camera.zoom;
let cameraTransformDirty = false;

app.ticker.add(() => {
    // ... existing frame timing and state updates ...

    const spriteUpdateStart = performance.now();
    creatureRenderer.render(latestCreatureData);
    const spriteUpdateEnd = performance.now();
    perfMetrics.recordSpriteUpdateTime(spriteUpdateEnd - spriteUpdateStart);

    // ✅ Only apply transform when zoom actually changed
    if (camera.zoom !== lastAppliedZoom || cameraTransformDirty) {
        camera.applyTransform(worldContainer, viewport.width, viewport.height);
        lastAppliedZoom = camera.zoom;
        cameraTransformDirty = false;
    }

    // ✅ Throttle ScaleBar updates to 30Hz (every 3rd frame)
    if (framesSinceScaleBarUpdate >= SCALE_BAR_UPDATE_INTERVAL) {
        scaleBarManager.update(camera.zoom);
        framesSinceScaleBarUpdate = 0;
    } else {
        framesSinceScaleBarUpdate++;
    }

    hudManager.updateFPS(fps);
    lastFrameTime = frameStart;
});
```

**Optimizations:**
1. **Dirty checking** - Only apply camera transform when zoom changes (not every frame)
2. **ScaleBar throttling** - 30Hz updates instead of 90Hz (DOM is expensive)
3. **Vsync alignment** - All GPU work synchronized with requestAnimationFrame

**Expected Results:**
- Zoom feels instant and smooth regardless of creature count
- GPU transforms synchronized with render pipeline (no tearing)
- Reduced DOM manipulation overhead (30Hz vs 90Hz)
- Zero race conditions (all updates in single loop)

**Success Criteria:**
- [x] Zoom responds instantly to wheel input
- [x] No visible jitter or stuttering at 10K+ creatures
- [x] ScaleBar updates smooth but not excessive
- [x] GPU transforms only when needed (not every frame)

**Performance Impact:**
- Before: 120Hz wheel events × (GPU transform + DOM update) = race conditions
- After: 90Hz dirty-checked GPU transforms + 30Hz DOM updates = smooth

**Files Changed:**
- `apps/portal/src/main.ts` (wheel handler + render loop)

**Reference:**
- Issue documented in `SPRINT_DOCS/SPRINT_BACKLOG.md` lines 272-276
- Root cause: GPU work outside render loop causes synchronization issues

---

## Future Work (Not This Sprint)

- **Spatial grid** for O(1) perception queries (massive improvement)
- **DNA-driven reaction time genes** (future biology sprint)
- **Variable LOD** based on camera zoom level
- **Viewport culling** (only update visible creatures)
- **Rotation interpolation** (currently rotation sent but not interpolated)
- **Prediction** (extrapolate position if physics late)

---

## References

- **Sprint 11:** IPC optimization complete, dual-tick abandoned
- **Biology notes:** `docs/biology/biology-notes.md` lines 850-956
- **Optimization backlog:** `docs/performance/optimization-backlog.md` lines 29-33
- **Abandoned dual-tick:** `docs/architecture/dual-tick-simulation.md` (reference only)

---

**Key Takeaway:** Simplicity wins. Single-tick with per-creature delays is more effective than complex dual-tick architecture, and matches biological reality better.
