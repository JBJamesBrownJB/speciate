# Behavior Engine Architecture

## Overview

The behavior engine implements **Reynolds steering behaviors** in a Bevy ECS architecture. Creatures exhibit complex, lifelike movement through the **force accumulation pattern**: multiple behavior systems independently calculate steering forces, which are summed into acceleration and integrated by the physics system.

This document describes the architectural principles, individual behaviors, state machines, and system ordering that power creature intelligence.

## Core Principles

### 1. Force Accumulation (Additive Steering)

**Pattern:** Behavior systems ADD forces to `Acceleration`, never replace.

```rust
// Each behavior system adds its force
fn seek_system(mut query: Query<(&Position, &Target, &mut Acceleration)>) {
    for (pos, target, mut accel) in query.iter_mut() {
        let force = calculate_seek_force(pos, target);
        accel.ax += force.x;  // ADD, don't replace
        accel.ay += force.y;
    }
}

fn avoidance_system(mut query: Query<(&Position, &mut Acceleration)>) {
    for (pos, mut accel) in query.iter_mut() {
        let force = calculate_avoidance_force(pos);
        accel.ax += force.x;  // Accumulates with seek force
        accel.ay += force.y;
    }
}

// Physics system integrates accumulated forces (Euler integration)
fn integrate_motion_system(
    mut query: Query<(&mut Position, &mut Velocity, &mut Acceleration)>,
    dt: Res<DeltaTime>,  // Injected by physics tick (33.3ms at 30Hz)
) {
    for (mut pos, mut vel, mut accel) in query.iter_mut() {
        vel.vx += accel.ax * dt.0;  // Integrate acceleration
        vel.vy += accel.ay * dt.0;
        pos.x += vel.vx * dt.0;     // Integrate velocity
        pos.y += vel.vy * dt.0;

        // Reset acceleration for next frame
        accel.ax = 0.0;
        accel.ay = 0.0;
    }
}
```

**Benefits:**
- **Natural blending:** Seek + avoid = emergent path around obstacles
- **Extensible:** Add new behaviors without modifying existing ones
- **Biologically realistic:** Multiple sensory inputs → single motor output
- **Priority through magnitude:** Stronger forces (panic > avoidance > seek) naturally dominate

### 2. Three-Tier Component Architecture

Creatures use a **hybrid ECS pattern** that balances performance with behavioral state machines:

#### Tier 1: Capability Markers (Zero-Sized Types)

**Purpose:** Permanent entity capabilities, added at spawn, **never removed**.

```rust
#[derive(Component, Default)]
pub struct CanSeek;       // Can pursue targets

#[derive(Component, Default)]
pub struct CanWander;     // Can patrol territory

#[derive(Component, Default)]
pub struct CanAvoidObstacles;  // Can dodge collisions
```

**Why ZST markers:**
- Zero memory overhead
- Fast archetype filtering: `Query<..., With<CanSeek>>`
- No archetype changes during gameplay (archetype stability)
- Represents what entity CAN do, not what it IS doing

#### Tier 2: Behavioral State (Enum Component)

**Purpose:** Mutually exclusive high-level behavioral modes (state machine).

```rust
#[derive(Component, Clone, Debug, PartialEq)]
pub enum BehaviorMode {
    Catatonic,   // Stationary, no movement
    Seeking,     // Pursuing a target
    Wandering,   // Territory patrol
    Fleeing,     // Escaping from threat (future)
    Feeding,     // Consuming resource (future)
}
```

**Why enum state:**
- Mutating enum is cheap (no archetype change)
- Biological realism: High-urgency behaviors suppress low-urgency ones
- Easy state transitions: `creature.behavior = BehaviorMode::Fleeing`
- Clear priority hierarchy (threat > hunger > exploration)

#### Tier 3: Data Components (Pure Data)

**Purpose:** Minimal data payloads for behaviors.

```rust
#[derive(Component, Clone, Copy)]
pub struct Target {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct WanderState {
    pub wander_angle: f32,       // Current wander direction
    pub wander_distance: f32,    // Circle projection distance
    pub wander_radius: f32,      // Circle radius
    pub angle_change: f32,       // Max angle change per tick
}

#[derive(Component)]
pub struct HomePosition {
    pub x: f32,
    pub y: f32,
}
```

**Why separate data:**
- Lightweight, easy to add/remove if needed
- No logic, just coordinates and references
- Clear separation: state vs. configuration

### 3. System Ordering

**Critical execution order ensures correct physics:**

```rust
schedule.add_systems((
    // 1. Behavior Systems (force accumulation - can run in parallel)
    behavior_transition_system,
    territory_wandering_system,
    seek_system,
    flee_system,
    avoidance_system,

    // 2. Movement Integration (MUST run after all behaviors)
    integrate_motion_system,

    // 3. Constraint Systems (MUST run after movement)
    boundary_enforcement_system.after(integrate_motion_system),

    // 4. Visual Systems (can run anytime after movement)
    rotation_system,
    snapshot_system,
));
```

**Why this order:**
1. **Behaviors first:** All behaviors calculate and accumulate forces
2. **Physics second:** Integration applies accumulated forces to position
3. **Constraints third:** Boundary clamping, collision response
4. **Visuals last:** Rotation, frontend snapshot

## Individual Behaviors

### Seeking: Goal-Directed Pursuit

**File:** `src/simulation/creatures/behaviors/seek.rs`

**Purpose:** Steer toward a target with smooth exponential deceleration and precise arrival.

**Algorithm:**
1. Calculate distance to target
2. Exponential deceleration in slow zone (gentle far out, sharp near target)
3. Pounce when close and slow (snap to target, prevent creeping)
4. Emergency brake if within arrival radius
5. Calculate steering force and ADD to acceleration

**Arrival Zones:**
- **Slow zone:** 15m (begin exponential deceleration)
- **Pounce zone:** 0.5m @ speed < 5.5 m/s (snap to target)
- **Emergency brake:** < 0.5m (hard counter-force)

**Constants (from `SEEKING`):**
```rust
pub struct SeekingConstants {
    pub max_force: f32,          // 50.0 N
    pub brake_force: f32,        // 70.0 N (emergency stop)
    pub pounce_distance: f32,    // 0.5 m (snap threshold)
    pub pounce_speed: f32,       // 5.5 m/s (max speed for snap)
    pub arrival_tolerance: f32,  // 0.5 m (stop distance)
    pub slow_zone_decay: f32,    // 1.5 (deceleration curve)
}
```

**Exponential Deceleration Formula:**
```rust
let slow_zone_distance = slow_zone - arrival_radius;
let distance_into_zone = center_distance - arrival_radius;
let ratio = distance_into_zone / slow_zone_distance;

// Exponential decay: maintains speed far out, brakes hard near target
let desired_speed = max_speed * (decay_factor * ratio).exp() / decay_factor.exp();
```

**Behavior:**
- **Far from target (> 15m):** Full speed pursuit
- **Entering slow zone (5-15m):** Exponential deceleration
- **Near target (0.5-5m):** Slow approach, ready to pounce
- **Pounce condition:** Distance < 0.5m AND speed < 5.5 m/s → Snap to target, enter Catatonic
- **Emergency brake:** Distance < 0.5m AND speed > 5.5 m/s → Hard counter-force

**Why exponential over linear:**
- Maintains speed longer (max reaction time for obstacles)
- Sharp deceleration near target (precise arrival)
- Only overshoots if too fast with insufficient distance (realistic physics)

### Wandering: Territory-Based Patrol

**File:** `src/simulation/creatures/behaviors/wander.rs`

**Purpose:** Organic territory patrol using Reynolds wandering + elastic tether homeward bias.

**Algorithm (Hybrid Force Blending):**
1. **Calculate Reynolds wandering force** (smooth random exploration)
   - Project circle ahead of creature
   - Randomly adjust wander angle
   - Calculate steering force toward point on circle perimeter
2. **Calculate homeward seeking force** (pull toward territory center)
   - Direction to home
   - Urgency factor scales with distance from home
3. **Blend forces using sigmoid curve** based on distance from home
4. **ADD blended force to acceleration** (force accumulation pattern)

**Constants (from `TERRITORY`):**
```rust
pub struct TerritoryConstants {
    pub comfort_radius: f32,      // 10.0 m (territory core)
    pub blend_center: f32,         // 20.0 m (50% blend point)
    pub max_wander_distance: f32,  // 30.0 m (hard limit)
    pub homeward_force: f32,       // 50.0 N (strong home pull)
    pub sigmoid_steepness: f32,    // 1.5 (elastic tether smoothness)
}
```

**Force Blending Strategy:**
- **Near home (0-10m):** 90% wandering, 10% homeward (free exploration)
- **Mid-range (10-20m):** 50% wandering, 50% homeward (balanced patrol)
- **Far from home (20-30m):** 10% wandering, 90% homeward (emergency return)

**Sigmoid Blending Formula:**
```rust
fn calculate_territory_blend(distance: f32, comfort: f32, center: f32) -> f32 {
    let normalized = (distance - center) / comfort;
    let sigmoid = 1.0 / (1.0 + (-steepness * normalized).exp());
    sigmoid.clamp(0.0, 1.0)
}

fn blend_forces(wander: (f32, f32), homeward: (f32, f32), blend: f32) -> (f32, f32) {
    let x = wander.0 * (1.0 - blend) + homeward.0 * blend;
    let y = wander.1 * (1.0 - blend) + homeward.1 * blend;
    (x, y)
}
```

**Biological Rationale:**
- Animals don't wander randomly - they patrol territories with soft boundaries
- "Elastic tether" model from movement ecology research
- Composite movement strategies are the norm in territorial species
- See `docs/biology/biology-notes.md` (2025-11-08 consultation)

### Obstacle Avoidance: Collision Prevention

**File:** `src/simulation/creatures/behaviors/avoidance.rs`

**Purpose:** Steer away from nearby creatures using inverse square law repulsion.

**Algorithm:**
1. Query perception for nearby entities
2. For each neighbor:
   - Calculate edge-to-edge distance (accounting for body radii)
   - If within personal space: apply repulsion force
   - Use inverse square scaling (stronger when closer)
   - Cap at panic force if collision imminent
3. Sum all repulsion forces
4. Limit total magnitude to max_force
5. ADD to acceleration (force accumulation)

**Force Zones:**
- **Comfort zone** (distance > personal_space): No force
- **Repulsion zone** (panic_threshold < distance ≤ personal_space): Inverse square scaling
- **Panic zone** (distance ≤ panic_threshold): Maximum force (capped)

**Constants (from `STEERING`):**
```rust
pub struct SteeringConstants {
    pub avoidance_force: f32,  // 35.0 N (base repulsion)
    pub panic_force: f32,      // 90.0 N (max emergency force)
}
```

**Inverse Square Law:**
```rust
let ratio = personal_space / edge_distance;
let mut force_magnitude = avoidance_force * ratio * ratio;

// Cap at panic force if too close
if edge_distance < panic_threshold {
    force_magnitude = force_magnitude.min(panic_force);
}
```

**Why inverse square:**
- Mimics looming threat perception (angular size grows as 1/distance²)
- Gentle repulsion far out, sharp avoidance close in
- Biologically realistic (animals don't use linear gradients)
- See `docs/biology/biology-notes.md` (2025-11-07 zoologist consultation)

**Multiple Obstacles:**
- Forces from all neighbors are summed
- Total force is capped at `max_force` to prevent overwhelming acceleration
- Prevents physics instability when surrounded

### Fleeing: Threat Escape (Future)

**File:** `src/simulation/creatures/behaviors/flee.rs`

**Status:** Stub implementation, not yet active.

**Planned Behavior:**
- Steer directly away from threat entity
- Higher force than seeking (flee_force = 20.0 N)
- Override other behaviors (survival priority)
- Transition back to wandering when threat > perception range

## Behavioral State Machine

**File:** `src/simulation/creatures/behaviors/transitions.rs`

### Current State Machine (Simplified)

```
┌─────────────┐
│  Catatonic  │  (Stationary, no movement)
└─────────────┘
      │
      ↓
┌─────────────┐
│   Seeking   │  (Pursuing target)
└─────────────┘
      │
      ↓
┌─────────────┐
│  Wandering  │  (Territory patrol)
└─────────────┘
```

**Current Transitions:**
- **Catatonic:** No auto-transitions, externally controlled
- **Seeking:** No auto-transitions (when target reached, manually set to Catatonic)
- **Wandering:** Energy consumption (0.01/tick), no auto-transitions yet

### Future State Machine (Full A-Life)

**Planned hierarchy (urgency-based):**
1. **Fleeing** (survival - highest priority)
2. **Feeding** (hunger - high priority)
3. **Seeking** (goal pursuit - moderate priority)
4. **Wandering** (exploration - low priority)
5. **Resting** (energy recovery - low priority)

**Planned Transitions:**
```rust
// Priority-based state selection
if perception.threat_detected() {
    creature.behavior = BehaviorMode::Fleeing;
} else if energy < 30.0 && perception.food_nearby() {
    creature.behavior = BehaviorMode::Feeding;
} else if energy < 50.0 {
    creature.behavior = BehaviorMode::Resting;
} else if perception.goal_detected() {
    creature.behavior = BehaviorMode::Seeking;
} else {
    creature.behavior = BehaviorMode::Wandering;
}
```

**Energy Costs (planned):**
- Catatonic: 0.0/tick (stationary)
- Resting: -0.02/tick (recovery)
- Wandering: 0.01/tick (slow patrol)
- Seeking: 0.03/tick (active pursuit)
- Fleeing: 0.05/tick (high-energy escape)
- Feeding: -0.1/tick (rapid recovery)

**TODO:** Migrate all constants to DNA genes (Future DNA system)

## Integration with Perception

Behaviors rely on the **perception system** to detect nearby entities and threats.

**Perception Component:**
```rust
pub struct Perception {
    pub nearby: Vec<Entity>,      // Entities within perception range
    pub range: f32,                // Detection radius
    pub last_update: f64,          // Cache timestamp
}
```

**Perception Range Formula:**
```rust
perception_range = body_length × perception_multiplier
// Default: 1m creature × 10.0 = 10m detection range
```

**System Ordering:**
```
perception::update_perception_system  →  behavior systems  →  physics
```

**Why perception first:**
- Behaviors need fresh neighbor data
- Avoidance requires up-to-date positions
- State transitions may depend on threat detection

**Perception Performance:**
- Planned: 50m bucket grid with FxHash (O(N) average case)
- See: `SPRINTS/spatial-grid/SPRINT_PLAN.md` for implementation plan
- See `src/simulation/perception/systems.rs`

## Force Hierarchy & Priority

**Biological Principle:** Survival > goals > exploration

**Force Magnitudes:**
1. **Panic:** 90.0 N (emergency collision avoidance)
2. **Homeward:** 50.0 N (territory return)
3. **Seeking:** 50.0 N (goal pursuit)
4. **Avoidance:** 35.0 N (base repulsion)
5. **Seeking (general):** 10.0 N (low-priority seek)
6. **Wander:** 5.0 N (exploration)

**Why this hierarchy:**
- High-urgency forces naturally dominate through magnitude
- No need for explicit priority system (emergent from physics)
- Creatures can still navigate around obstacles while fleeing (panic > flee > seek)
- Biological realism: animals trade off goals vs. safety

**Example Scenario:**
```
Creature pursuing food (seek = 50N)
Obstacle detected (avoid = 35N inverse square)
Close to collision (panic = 90N cap)

Result: Panic force dominates, creature swerves around obstacle
After dodge: Avoidance drops to 0N, seek resumes, creature reaches food
```

## System Performance

### Current Architecture: Single-Tick Simulation

The simulation runs all systems at a single tick rate:
- **~22Hz Tick Rate** - All systems (physics, AI, perception) run together
- **Frontend Interpolation** - 60+ FPS visuals via lerp between frames

**Note:** Dual-tick was explored and abandoned. See `docs/archive/dual-tick/` for rationale.

**Physics Tick Budget (30Hz):**
- Grid updates: ~3ms (incremental only)
- Collision detection: ~10ms
- Motion integration: ~5ms
- Total: ~18ms ✅ (46% headroom)

**AI Tick Budget (20Hz):**
- Perception queries: ~40ms (100K creatures)
- Behavior transition: ~2ms
- Force calculation: ~8ms
- Total: ~50ms ⚠️ (at budget limit)

**Bottleneck:** Perception system (O(N²) brute force). Optimization planned via 50m bucket grid - see `SPRINTS/spatial-grid/SPRINT_PLAN.md`.

### Scalability Targets

| Creature Count | Physics | AI | Strategy |
|----------------|---------|-----|----------|
| 0-10,000       | 30 Hz   | 20 Hz | Current dual-tick implementation |
| 10,000-50,000  | 30 Hz   | 20 Hz | Spatial grid (see `SPRINTS/spatial-grid/SPRINT_PLAN.md`) |
| 50,000-100,000 | 30 Hz   | 20 Hz | Parallel queries, SIMD distance calc |
| 100,000-200,000| 30 Hz   | 20 Hz | Optimized dual-tick (target) |
| 200,000+       | 30 Hz   | 10 Hz | LOD simulation (partial update) |

**Target: 150,000-200,000 creatures** with dual-tick architecture.

### Future Optimizations

**Implemented Optimizations:**
1. **Spatial Grid:** 200m bucket grid with FxHash (O(N) queries)
2. **Dual-Tick Architecture:** 30Hz physics / 20Hz AI (implemented)
3. **Incremental Updates:** Only update grid on cell changes

**Planned Optimizations:**
1. **ECS Parallelization:** Bevy's `par_iter_mut()` for perception
2. **SIMD Physics:** Vectorized distance calculations
3. **LOD Simulation:** Full sim near player, statistical sim distant
4. **GPU Compute:** Perception queries on GPU (research phase)

## DNA Integration (Future)

**Current:** Hardcoded constants (`SEEKING`, `TERRITORY`, `STEERING`)

**Future:** DNA-driven gene expression

**Migration Path:**
```rust
// Phase 1 (Current): Hardcoded constants
const MAX_SPEED: f32 = 50.0;

// Phase 2 (Planned): DNA component added
pub struct DNA {
    pub genes: HashMap<String, f32>,
}

// Phase 3 (Future): Gene expression
fn seek_system(query: Query<(&DNA, &Target, &mut Acceleration)>) {
    for (dna, target, mut accel) in query.iter() {
        let max_speed = dna.express_gene("agility");
        let max_force = dna.express_gene("strength");
        // DNA-driven behavior
    }
}
```

**Planned Gene Mappings:**

| Constant                    | DNA Gene                  | Range      |
|-----------------------------|---------------------------|------------|
| `MAX_SPEED`                 | `agility`                 | 20-80 m/s  |
| `SEEKING.max_force`         | `strength`                | 10-100 N   |
| `SEEKING.arrival_tolerance` | `precision`               | 0.1-2.0 m  |
| `TERRITORY.comfort_radius`  | `comfort_radius_multiplier` | 5-50 m   |
| `TERRITORY.homeward_force`  | `territory_attachment`    | 10-100 N   |
| `STEERING.avoidance_force`  | `caution`                 | 10-70 N    |
| `PERCEPTION.perception_multiplier` | `perception`       | 3-20×      |

**Trade-offs:**
- Large + fast = high energy consumption (starves faster)
- High perception = detect threats early BUT cognitive overload in cluttered terrain
- High aggression = secure resources BUT fight injuries and energy waste

**Goal:** Create viable ecological niches, not perfect balance. Every strategy succeeds somewhere, fails elsewhere.

**See:** `docs/biology/dna-driven-design.md` for full specification

## Testing Strategy

### Unit Tests

**Each behavior system has dedicated tests:**
- `seek.rs`: Arrival, pounce, emergency brake, overshoot prevention
- `wander.rs`: Territory blending, sigmoid curve, force interpolation
- `avoidance.rs`: Inverse square scaling, panic cap, multiple obstacles
- `transitions.rs`: Aging, energy consumption, state persistence

**Run tests:**
```bash
cd apps/simulation
cargo test
```

### Integration Tests

**Full simulation scenarios:**
- Seeker reaches target eventually
- Seeker slows down near target
- Seeker avoids obstacle in path
- Wandering creatures stay within territory
- Multiple creatures don't overlap (avoidance)

**Location:** `apps/simulation/src/simulation/tests/behavior_tests.rs`

### Test-Driven Development (TDD)

**MANDATORY workflow (from CLAUDE.md):**
1. Run `cargo test` before ANY code change
2. Write test FIRST if adding new functionality
3. Run tests IMMEDIATELY after change
4. If tests fail, revert or fix immediately

**Why this matters:**
- Tests exist to catch breaking changes
- Prevents regressions (e.g., removing null checks that break code)
- Validates assumptions (e.g., exponential deceleration actually stops)

## File Structure

```
src/simulation/creatures/behaviors/
├── mod.rs                 # Module exports
├── transitions.rs         # State machine
├── seek.rs                # Goal pursuit
├── wander.rs              # Territory patrol
├── avoidance.rs           # Collision prevention
└── flee.rs                # Threat escape (future)

src/simulation/movement/
├── constants.rs           # SEEKING, TERRITORY, STEERING constants
├── systems.rs             # Physics integration (Euler)
├── rotation.rs            # Visual heading updates
└── noise.rs               # Locomotion variability (Perlin)

src/simulation/perception/
├── components.rs          # Perception, AvoidanceBehavior
└── systems.rs             # Spatial awareness updates
```

## Summary: Quick Reference

| Concept                  | Implementation                           | Why                                         |
|--------------------------|------------------------------------------|---------------------------------------------|
| **Force Accumulation**   | `accel += force` (ADDitive)              | Emergent blending, extensible, realistic    |
| **Capabilities**         | `CanSeek`, `CanWander` (ZST)             | Permanent, fast filtering, archetype stable |
| **State**                | `BehaviorMode::Seeking` (enum)           | Exclusive modes, cheap mutation, priority   |
| **Data**                 | `Target`, `WanderState` (structs)        | Pure data, no logic, minimal payload        |
| **System Ordering**      | Behaviors → Physics → Constraints        | Correct force application and integration   |
| **Force Hierarchy**      | Panic > Homeward > Seek > Avoid > Wander | Biological priority, emergent behavior      |
| **Perception**           | Spatial awareness, O(N²) brute-force     | Feeds neighbor data to avoidance            |
| **DNA Integration**      | Hardcoded → gene expression (future)     | Genetic diversity, evolution, player breeding |

---

**Remember:** In this architecture, **forces ARE behavior**. Creatures don't have scripts - they have sensory inputs (perception) and motor outputs (forces). Complex behaviors emerge from simple force combinations.

**See Also:**
- `/workspace/CLAUDE.md` - Project-wide principles (TDD, DNA-driven design)
- `/workspace/apps/simulation/CLAUDE.md` - ECS patterns and architectural standards
- `/workspace/docs/biology/biology-notes.md` - Zoologist consultations log
- `/workspace/docs/biology/dna-driven-design.md` - DNA architecture specification
