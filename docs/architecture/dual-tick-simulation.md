# Dual-Tick Simulation Architecture

## Overview

The simulation uses a **dual-tick architecture** to separate concerns and enable massive scale:

- **30Hz Physics + Collision** (33.3ms period)
- **20Hz AI + Perception** (50ms period)
- **90Hz Frontend Rendering** (interpolated smoothing)

This architecture enables **150,000-200,000 creatures** vs ~10,000 with single-tick.

---

## Core Rationale

### Why Separate Tick Rates?

**Single-tick problem (all systems at same frequency):**
- AI perception queries are expensive (O(N) spatial lookups)
- Physics integration is cheap (just math)
- Running expensive AI at physics frequency wastes CPU
- Running cheap physics at AI frequency causes choppy motion

**Dual-tick solution:**
- Run expensive AI systems less frequently (20Hz)
- Run cheap physics systems more frequently (30Hz)
- Frontend interpolates for smooth visuals (90Hz)

### Biological Justification

The 20Hz AI tick (50ms period) models realistic biological reaction times:

- Small prey (mouse): 20-50ms reaction time ✅
- Medium predator (wolf): 50-100ms reaction time ✅
- Large herbivore (elephant): 100-300ms reaction time ✅

Creatures "commit" to steering decisions for 50ms, creating natural movement patterns rather than instant reactions. This is emergent "reflexes" via physics constraints.

---

## Architecture

### Two Separate Bevy Schedules

```rust
pub struct DualTickSimulation {
    world: World,
    physics_schedule: Schedule,  // 30Hz
    ai_schedule: Schedule,       // 20Hz

    physics_accumulator: f32,
    ai_accumulator: f32,
}
```

### Main Loop

```rust
impl DualTickSimulation {
    const PHYSICS_TICK: f32 = 1.0 / 30.0;  // 33.3ms
    const AI_TICK: f32 = 1.0 / 20.0;        // 50ms

    pub fn update(&mut self, real_delta_time: f32) {
        self.physics_accumulator += real_delta_time;
        self.ai_accumulator += real_delta_time;

        // AI tick first (calculates forces)
        while self.ai_accumulator >= Self::AI_TICK {
            self.ai_schedule.run(&mut self.world);
            self.ai_accumulator -= Self::AI_TICK;
        }

        // Physics tick (integrates forces)
        while self.physics_accumulator >= Self::PHYSICS_TICK {
            self.physics_schedule.run(&mut self.world);
            self.physics_accumulator -= Self::PHYSICS_TICK;
        }
    }
}
```

---

## System Distribution

### Physics Schedule (30Hz)

**Systems that run every 33.3ms:**

```rust
physics_schedule.add_systems((
    // Spatial grid updates incrementally with movement
    integrate_motion_with_grid_update,

    // Collision detection uses fresh grid
    detect_collisions,
    apply_collision_response,

    // Position constraints
    boundary_enforcement,

    // Visual state
    rotation_system,

    // Clean up for next tick
    reset_acceleration,
).chain());
```

**Characteristics:**
- Mutates Position, Velocity, Acceleration
- Updates spatial grid incrementally (not full rebuild)
- Handles collision detection and response
- No perception queries (uses grid, doesn't search it)

### AI Schedule (20Hz)

**Systems that run every 50ms:**

```rust
ai_schedule.add_systems((
    // Spatial queries (expensive)
    update_perception_system,

    // Behavior state machine
    behavior_transition_system,

    // Steering force calculation
    territory_wandering_system,
    flee_system,
    seek_system,
    avoidance_system,
).chain());
```

**Characteristics:**
- Queries spatial grid (perception lookups)
- Decides behavioral state (seek, flee, wander)
- Calculates steering forces → writes to Acceleration
- Forces persist until next AI tick

---

## Force Accumulation Semantics

### The Key Question: What Happens to Forces Between AI Ticks?

**Decision:** Forces persist until next AI tick overwrites them.

**Timeline example:**

```
Time:    0ms      33ms     66ms     100ms    133ms    166ms
AI:      ★                           ★
Physics: ●         ●        ●        ●         ●        ●
         |____same force____|        |___new force___|
```

**At AI tick (★):**
1. Perception queries spatial grid
2. Behavior transition decides state
3. Steering systems OVERWRITE acceleration with new forces
4. Forces persist for next 2-3 physics ticks

**At Physics tick (●):**
1. Read current acceleration (from last AI tick)
2. Integrate: velocity += acceleration * dt
3. Integrate: position += velocity * dt
4. Reset acceleration to zero (for next physics tick)

**Wait, reset to zero?** Yes, but AI only runs every 50ms, so:

**Corrected pattern:**

```rust
// AI tick: Calculate desired force
fn seek_system(mut query: Query<(&Position, &Target, &mut DesiredForce)>) {
    for (pos, target, mut force) in query.iter_mut() {
        force.0 = calculate_seek_force(pos, target);
    }
}

// Physics tick: Apply stored force
fn apply_forces_system(
    mut query: Query<(&DesiredForce, &mut Acceleration)>,
) {
    for (desired, mut accel) in query.iter_mut() {
        accel.ax = desired.0.x;
        accel.ay = desired.0.y;
    }
}
```

This separates "desired force" (persists) from "current acceleration" (integrated and reset).

---

## Performance Benefits

### Single-Tick vs Dual-Tick (100K Creatures)

| Metric | Single-Tick (20Hz all) | Dual-Tick (30Hz/20Hz) |
|--------|------------------------|------------------------|
| Grid operations | 20 × 100K = 2M/sec | 30 × 1.6K = 48K/sec |
| Perception queries | 20 × 100K = 2M/sec | 20 × 100K = 2M/sec |
| Physics integration | 20 × 100K = 2M/sec | 30 × 100K = 3M/sec |
| **Total ops/sec** | **6M** | **5M** |
| Motion smoothness | Choppy (50ms steps) | Smooth (33ms + interpolation) |

**The win isn't operation count** - it's that physics runs MORE frequently while AI runs LESS frequently, optimizing for each concern.

### Tick Budget Allocation

**Physics tick (33.3ms budget):**
- Grid updates: ~3ms (incremental, only cell changes)
- Collision detection: ~10ms
- Motion integration: ~5ms
- Total: ~18ms ✅ (46% headroom)

**AI tick (50ms budget):**
- Perception queries: ~40ms
- Behavior transition: ~2ms
- Force calculation: ~8ms
- Total: ~50ms ⚠️ (at budget limit)

AI is the bottleneck, but only runs at 20Hz.

---

## Frontend Interpolation

### Why 90Hz Rendering with 30Hz Physics?

30Hz physics produces position snapshots every 33.3ms. Without interpolation:
- Creatures "teleport" 33ms apart
- Looks choppy, especially at high speeds

With interpolation:
- Frontend maintains previous and current position
- Renders at 90Hz (11.1ms), interpolating between snapshots
- Smooth motion despite lower physics rate

### Implementation

```typescript
class CreatureRenderer {
    previousPosition: Vec2;
    currentPosition: Vec2;
    interpolationFactor: number = 0;

    onPhysicsSnapshot(newPosition: Vec2) {
        this.previousPosition = this.currentPosition;
        this.currentPosition = newPosition;
        this.interpolationFactor = 0;
    }

    update(dt: number) {
        const PHYSICS_TICK_PERIOD = 1 / 30;  // 33.3ms
        this.interpolationFactor = Math.min(1, this.interpolationFactor + dt / PHYSICS_TICK_PERIOD);
    }

    getDisplayPosition(): Vec2 {
        return lerp(this.previousPosition, this.currentPosition, this.interpolationFactor);
    }
}
```

### Extrapolation Warning

Interpolation assumes you have BOTH previous and current positions. If physics tick is late:
- Option A: Hold previous position (small stutter)
- Option B: Extrapolate using velocity (smooth but can overshoot)

Recommendation: Option A for simplicity. 30Hz is fast enough that minor stutters are imperceptible.

---

## Implementation Checklist

### Phase 1: Separate Schedules

- [ ] Create `physics_schedule: Schedule`
- [ ] Create `ai_schedule: Schedule`
- [ ] Add accumulator timing in main loop
- [ ] Move systems to appropriate schedules

### Phase 2: Force Persistence

- [ ] Add `DesiredForce` component (or similar)
- [ ] AI systems write to DesiredForce
- [ ] Physics systems read DesiredForce → Acceleration
- [ ] Test force continuity across ticks

### Phase 3: Frontend Interpolation

- [ ] Store previous + current position per creature
- [ ] Interpolate during render tick
- [ ] Handle late physics ticks gracefully

### Phase 4: Testing

- [ ] Unit tests for accumulator timing
- [ ] Integration tests for force persistence
- [ ] Performance benchmarks at 10K, 50K, 100K creatures
- [ ] Visual smoothness validation

---

## Alternatives Considered

### Three-Tick (90Hz Physics, 60Hz Collision, 20Hz AI)

**Pros:** More frequent physics, dedicated collision tick
**Cons:** More complex, marginal benefit over 30Hz

**Verdict:** Overkill. 30Hz physics with interpolation is smooth enough.

### Variable AI Tick by Creature Size

**Pros:** DNA-driven reaction times (small=fast, large=slow)
**Cons:** Multiple AI schedules, complex synchronization

**Verdict:** Future optimization. Fixed 20Hz for all creatures is simpler to implement and debug.

### Continuous (Frame-Rate Dependent)

**Pros:** No fixed tick, just run as fast as possible
**Cons:** Non-deterministic, hard to replay, variable creature behavior

**Verdict:** Rejected. Fixed ticks enable determinism and replays.

---

## References

- **Spatial grid:** `docs/architecture/spatial-partitioning.md`
- **Force model:** `docs/architecture/behavior-engine.md`
- **Collision system:** `docs/gameplay/critter-collision-system.md`
- **Biology notes:** `docs/biology/biology-notes.md` (reaction time justification)

---

## Key Decisions Log

**2025-11-16: Dual-Tick Architecture Adopted**
- 30Hz physics (smooth motion, collision detection)
- 20Hz AI (perception queries, behavior decisions)
- 90Hz frontend (interpolated rendering)
- Biological justification: 50ms reaction time is realistic
- Scale: Enables 150K-200K creatures (15× improvement over single-tick)

**2025-11-16: Force Persistence Model**
- AI calculates desired forces, stores in component
- Physics reads desired forces, applies to acceleration
- Forces persist across 1-2 physics ticks until next AI tick
- Creates smooth, committed movement patterns
