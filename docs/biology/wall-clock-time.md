# Wall-Clock Time in Biological Systems

## Core Principle

**Biological processes operate in real time, not simulation ticks.**

Ticks are the simulation's sampling frequency - how often we compute the world. But a creature's metabolism doesn't speed up because we observe it more frequently. Energy consumption, aging, gestation, and neural processing all occur at rates determined by biology, not simulation architecture.

---

## Why This Matters

### The Problem: Tick-Dependent Biology

If biological rates are tied to tick counts:

```rust
// WRONG: Tick-dependent metabolism
fn metabolism_system(mut query: Query<&mut Energy>) {
    for mut energy in query.iter_mut() {
        energy.current -= 1.0;  // Loses 1 energy per tick
    }
}
```

**At 20Hz:** Creature loses 20 energy/second
**At 60Hz:** Creature loses 60 energy/second (3× faster starvation!)

This violates thermodynamics and makes simulation behavior dependent on implementation details.

### The Solution: Wall-Clock Scaling

```rust
// CORRECT: Wall-clock scaled metabolism
fn metabolism_system(
    time: Res<Time<Fixed>>,
    mut query: Query<(&mut Energy, &MetabolismRate)>,
) {
    let dt = time.delta_seconds();  // Real time elapsed this tick
    for (mut energy, rate) in query.iter_mut() {
        energy.current -= rate.per_second * dt;
    }
}
```

**At 20Hz (dt = 0.05s):** Loses 1.0 × 0.05 = 0.05 energy/tick = 1.0 energy/second
**At 60Hz (dt = 0.0167s):** Loses 1.0 × 0.0167 = 0.0167 energy/tick = 1.0 energy/second ✅

Same real-world rate regardless of tick frequency.

---

## Biological Systems Requiring Wall-Clock Time

### Energy & Metabolism

Metabolic rate is power (energy/time), not events/tick.

```rust
pub struct MetabolismComponent {
    pub basal_rate: f32,      // Energy per second at rest (Kleiber's Law)
    pub activity_multiplier: f32,
}

fn consume_energy(metabolism: &MetabolismComponent, dt: f32) -> f32 {
    metabolism.basal_rate * metabolism.activity_multiplier * dt
}
```

**Trade-offs (DNA-driven):**
- High metabolism = more active, faster starvation
- Low metabolism = energy efficient, slower reactions

### Aging & Lifespan

Biological aging is continuous degradation, not discrete events.

```rust
pub struct AgeComponent {
    pub chronological_age: f32,  // Seconds since birth
    pub biological_age: f32,     // Accumulated wear (can exceed chronological)
    pub max_lifespan: f32,       // DNA-encoded maximum (seconds)
}

fn aging_system(time: Res<Time>, mut query: Query<&mut AgeComponent>) {
    let dt = time.delta_seconds();
    for mut age in query.iter_mut() {
        age.chronological_age += dt;
        age.biological_age += dt;  // Could scale with metabolic wear
    }
}
```

### Reproduction & Gestation

Embryonic development is biochemistry, not tick counting.

```rust
pub struct ReproductionComponent {
    pub gestation_duration: f32,    // DNA-encoded: seconds (not ticks!)
    pub conception_time: Option<f32>,
}

fn check_birth(repro: &ReproductionComponent, current_time: f32) -> bool {
    if let Some(conception) = repro.conception_time {
        let elapsed = current_time - conception;
        elapsed >= repro.gestation_duration
    } else {
        false
    }
}
```

**Scaling by size (allometric):**
```
gestation_seconds = 30 * 86400 * mass^0.25  // days converted to seconds
```

### Perception & Reaction Time

Neural signal propagation has physical time constraints.

```rust
pub struct PerceptionComponent {
    pub last_update: f32,           // Wall-clock timestamp
    pub refresh_interval: f32,      // DNA-encoded: seconds between updates
}

fn should_update_perception(perc: &PerceptionComponent, current_time: f32) -> bool {
    current_time - perc.last_update >= perc.refresh_interval
}
```

At 1000Hz tick rate, this prevents creatures from having 1ms reaction times (neurologically impossible for macroscopic organisms).

### Decision Making

Cognitive processing takes real time, not ticks.

```rust
pub struct CognitionComponent {
    pub decision_interval: f32,     // DNA-encoded: 0.1-2.0 seconds
    pub last_decision: f32,
}

fn should_reconsider(cog: &CognitionComponent, current_time: f32) -> bool {
    current_time - cog.last_decision >= cog.decision_interval
}
```

**Why this matters:** At 60Hz, a "1 tick decision" = 16ms thinking time. Unrealistic. At 1000Hz, it's 1ms. Absurd. Wall-clock ensures creatures think at biologically plausible speeds.

---

## What SHOULD Be Tick-Based

Not everything uses wall-clock time. Physics integration benefits from fixed timesteps:

### Physics Integration

```rust
fn integrate_motion(
    mut query: Query<(&mut Position, &Velocity)>,
    dt: f32,  // Fixed timestep (1/30 for 30Hz physics)
) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.vx * dt;
        pos.y += vel.vy * dt;
    }
}
```

Higher tick rates = finer integration = better numerical stability. This is a feature, not a bug.

### Collision Detection

More ticks = catch fast collisions before tunneling. Again, desirable.

### Force Application

Forces calculated per AI tick, applied per physics tick. The dual-tick architecture handles this correctly.

---

## Implementation Pattern

### Resources

```rust
#[derive(Resource)]
pub struct SimulationClock {
    pub elapsed_seconds: f32,  // Total wall-clock simulation time
}

#[derive(Resource)]
pub struct TickConfig {
    pub physics_hz: f32,  // 30.0
    pub ai_hz: f32,       // 20.0
}
```

### Delta-Time Access

```rust
fn biological_system(
    clock: Res<SimulationClock>,
    time: Res<Time<Fixed>>,
    mut query: Query<&mut BiologicalComponent>,
) {
    let dt = time.delta_seconds();  // For rate-based calculations
    let current_time = clock.elapsed_seconds;  // For timestamp comparisons

    for mut bio in query.iter_mut() {
        // Rate-based: energy -= rate * dt
        // Timestamp: if current_time - last_update >= interval
    }
}
```

---

## Testing Considerations

Wall-clock time + randomness = non-deterministic outcomes. This is intentional for emergence.

**Test approaches:**

1. **Statistical bounds:** Creature should lose 0.9-1.1 energy/second (±10% tolerance for noise)
2. **Injectable time:** Mock clock for exact control in unit tests
3. **Rate verification:** Confirm rate is independent of tick frequency

```rust
#[test]
fn test_metabolism_rate_independent_of_tick_rate() {
    let mut creature = Creature::new(energy: 100.0, metabolism_rate: 10.0);

    // Simulate 1 second at 20Hz
    for _ in 0..20 {
        creature.metabolize(dt: 0.05);
    }
    let energy_20hz = creature.energy;

    // Reset and simulate 1 second at 60Hz
    creature.energy = 100.0;
    for _ in 0..60 {
        creature.metabolize(dt: 1.0/60.0);
    }
    let energy_60hz = creature.energy;

    // Should be nearly identical (floating point tolerance)
    assert!((energy_20hz - energy_60hz).abs() < 0.001);
}
```

---

## Common Mistakes to Avoid

### 1. Per-Tick Constants

```rust
// WRONG
const ENERGY_LOSS_PER_TICK: f32 = 0.1;

// RIGHT
const ENERGY_LOSS_PER_SECOND: f32 = 2.0;
```

### 2. Tick Counters for Timing

```rust
// WRONG
struct Creature {
    ticks_since_birth: u64,  // Age in ticks (tick-rate dependent)
}

// RIGHT
struct Creature {
    birth_time: f32,  // Wall-clock timestamp
}
```

### 3. Hardcoded Intervals as Tick Counts

```rust
// WRONG
if tick_counter % 10 == 0 {  // Every 10 ticks
    update_perception();
}

// RIGHT
if current_time - last_perception >= perception_interval {
    update_perception();
    last_perception = current_time;
}
```

---

## Integration with Dual-Tick Architecture

The dual-tick system (30Hz physics, 20Hz AI) uses wall-clock time correctly:

- **AI tick (20Hz / 50ms):** Perception refresh rate models biological reaction time
- **Physics tick (30Hz / 33ms):** Motion integration uses fixed dt for stability
- **Both use wall-clock:** Energy consumption scaled by dt, timestamps for intervals

The tick rates are chosen for performance and biological realism, not as arbitrary implementation details. Wall-clock scaling ensures these rates don't distort creature behavior.

---

## Summary

**Rule:** If a biological process has a rate (per second, per minute, per day), multiply by delta-time.

**Rule:** If a biological process has a duration (gestation, reaction delay), use timestamps.

**Rule:** If a process is pure physics (integration, collision), fixed timestep is fine.

**Why:** The simulation models biology. Biology operates in real time. Ticks are our sampling rate, not the creature's clock.
