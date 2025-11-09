# Simulation ECS Architecture Guide

This document defines the **Bevy ECS patterns and architectural decisions** for the Rust simulation backend.

## Core Principles

1. **Component Composition Over Enums**: Use components to enable behavior blending
2. **Archetype Stability**: Minimize archetype changes for performance
3. **Force Accumulation**: Systems ADD to acceleration, physics integrates
4. **DNA-Driven Parameters**: Behavior constants → DNA gene expression migration path

---

## Component Architecture: The Hybrid Pattern

We use a **three-tier component architecture** that balances ECS performance with biological state machines:

### 1. Capability Markers (Zero-Sized Types)

**Purpose:** Permanent entity capabilities, added at spawn, **never removed**.

**Pattern:**
```rust
#[derive(Component, Default)]
pub struct CanSeek;

#[derive(Component, Default)]
pub struct CanFlee;

#[derive(Component, Default)]
pub struct CanAvoidObstacles;
```

**Why:**
- Zero memory overhead (ZST)
- Enables fast archetype filtering: `Query<..., With<CanSeek>>`
- No archetype changes during gameplay (added once at spawn)
- Represents what entity CAN do, not what it IS doing

**Usage:**
```rust
// At spawn time
commands.spawn((
    Position::default(),
    Velocity::default(),
    CanSeek,          // Permanent capability
    CanFlee,          // Permanent capability
    CanAvoidObstacles, // Permanent capability
));
```

### 2. Behavioral State (Enum Component)

**Purpose:** Mutually exclusive high-level behavioral modes (state machine).

**Pattern:**
```rust
#[derive(Component, Clone, Debug)]
pub enum BehaviorState {
    Catatonic,
    Seeking { target_entity: Option<Entity> },
    Fleeing { threat_entity: Option<Entity> },
    Wandering { angle: f32 },
    Feeding { food_entity: Entity },
}
```

**Why:**
- Represents CURRENT activity (one active mode at a time)
- Mutating enum is cheap (no archetype change)
- Biological realism: High-urgency behaviors suppress low-urgency ones
- Easy state transitions: `creature.behavior = BehaviorState::Fleeing { threat }`

**Usage:**
```rust
fn seek_system(
    query: Query<(&BehaviorState, &Target, &mut Acceleration), With<CanSeek>>
) {
    for (behavior, target, mut accel) in query.iter_mut() {
        if let BehaviorState::Seeking { .. } = behavior {
            // Apply seek force
        }
    }
}
```

### 3. Data Components (Pure Data)

**Purpose:** Minimal data payloads for behaviors (coordinates, references, configuration).

**Pattern:**
```rust
#[derive(Component, Clone, Copy)]
pub struct Target {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct PerceptionData {
    pub nearby_entities: Vec<Entity>,
    pub last_update: f64,
}
```

**Why:**
- Just the facts (coordinates, IDs, cached data)
- No logic, no behavior parameters (those come from DNA)
- Lightweight, easy to add/remove if needed

---

## System Patterns

### Force Accumulation (Additive Steering)

**Principle:** Systems ADD forces to `Acceleration`, physics system integrates.

**Why:**
- Natural force blending (seek + avoid = emergent path)
- Extensible (add new behaviors without modifying existing ones)
- Biologically realistic (multiple sensory inputs → single motor output)

**Pattern:**
```rust
// Behavior systems ADD forces
fn seek_system(mut query: Query<(&Position, &Target, &mut Acceleration)>) {
    for (pos, target, mut accel) in query.iter_mut() {
        let force = calculate_seek_force(pos, target);
        accel.ax += force.x;  // ADD, don't replace
        accel.ay += force.y;
    }
}

fn obstacle_avoidance_system(mut query: Query<(&Position, &mut Acceleration)>) {
    for (pos, mut accel) in query.iter_mut() {
        let force = calculate_avoidance_force(pos);
        accel.ax += force.x;  // Accumulates with seek force
        accel.ay += force.y;
    }
}

// Movement system integrates accumulated forces (Euler integration)
fn integrate_motion_system(mut query: Query<(&mut Position, &mut Velocity, &Acceleration)>) {
    for (mut pos, mut vel, accel) in query.iter_mut() {
        vel.vx += accel.ax * dt;  // Integrate acceleration
        vel.vy += accel.ay * dt;
        pos.x += vel.vx * dt;     // Integrate velocity
        pos.y += vel.vy * dt;
        // Acceleration reset to 0 at end (not shown)
    }
}
```

### System Ordering

**Critical Order:**
1. **Behavior Systems** (calculate forces, write to Acceleration)
2. **Movement Integration** (apply forces to velocity, velocity to position via Euler integration)
3. **Constraint Systems** (boundaries, collision response)
4. **Visual Systems** (rotation, NATS publishing)

**Example:**
```rust
schedule.add_systems((
    // 1. Behaviors (can run in parallel if non-overlapping queries)
    seek_system,
    flee_system,
    wander_system,
    obstacle_avoidance_system,

    // 2. Movement (MUST run after all behaviors)
    integrate_motion_system,

    // 3. Constraints (MUST run after movement)
    boundary_enforcement_system.after(integrate_motion_system),

    // 4. Visuals (can run anytime after movement)
    rotation_system,
    publish_frame_system,
));
```

---

## Entity Lifecycle Management

### Death Handling: Add Dead, Don't Remove

**Problem:** Removing components causes archetype thrashing.

**Solution:** Add `Dead` marker component, filter with `Without<Dead>`.

**Pattern:**
```rust
// Death detection
fn death_system(
    mut commands: Commands,
    query: Query<(Entity, &CreatureState), Without<Dead>>,
) {
    for (entity, state) in query.iter() {
        if state.energy <= 0.0 {
            commands.entity(entity)
                .insert(Dead { time_of_death: now })
                .insert(Corpse { biomass: state.calculate_body_mass() });
            // DON'T remove CanSeek, CanFlee, etc. - leave them!
        }
    }
}

// Living creatures filter
fn seek_system(
    query: Query<(&Position, &Target, &mut Acceleration), (With<CanSeek>, Without<Dead>)>
) {
    // Only processes living seekers
}

// Corpse decay (eventual cleanup)
fn corpse_decay_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Corpse, &Dead)>,
) {
    for (entity, mut corpse, _) in query.iter_mut() {
        corpse.biomass -= corpse.decay_rate * dt;
        if corpse.biomass <= 0.0 {
            commands.entity(entity).despawn();  // Final cleanup
        }
    }
}
```

**Why:**
- Adding `Dead` = 1 archetype change
- Removing multiple capabilities = N archetype changes
- Corpses remain as obstacles/food (ecological realism)
- Gradual cleanup (despawn after decay, not immediately)

### Spawning Entities

**Always add capabilities at spawn:**
```rust
commands.spawn((
    // Physics
    Position::default(),
    Velocity::default(),
    Acceleration::default(),

    // Capabilities (permanent)
    CanSeek,
    CanFlee,
    CanWander,
    CanAvoidObstacles,

    // State (mutable)
    BehaviorState::Wandering { angle: 0.0 },
    CreatureState::default(),

    // DNA (future)
    // DNA::from_genes(genes),
));
```

---

## DNA Integration Strategy

### Current: Hardcoded Constants

**Pattern:**
```rust
fn seek_system(query: Query<(&Position, &Target, &mut Acceleration)>) {
    const MAX_SPEED: f32 = 50.0;       // TODO: from DNA
    const ARRIVAL_RADIUS: f32 = 10.0;  // TODO: from DNA
    const MAX_FORCE: f32 = 10.0;       // TODO: from DNA

    // Use constants for now
}
```

### Future: DNA-Driven Parameters

**Pattern:**
```rust
fn seek_system(query: Query<(&Position, &Target, &mut Acceleration, &DNA)>) {
    for (pos, target, mut accel, dna) in query.iter() {
        let max_speed = dna.express_gene("agility");      // From DNA
        let arrival_radius = dna.express_gene("precision"); // From DNA
        let max_force = dna.express_gene("strength");      // From DNA

        // DNA-driven behavior
    }
}
```

### Migration Path

1. **Sprint 6 (Now):** Hardcode constants with `// TODO: from DNA` comments
2. **Sprint 7:** Add `DNA` component with placeholder genes
3. **Sprint 8:** Implement gene expression (`dna.express_gene("agility")`)
4. **Sprint 9+:** Full DNA-driven ecosystem

**Biological Consultation:**
- ALWAYS consult zoologist-tom BEFORE adding hardcoded constants
- Log decisions in `/workspace/BIOLOGY_NOTES.md`
- Document realistic ranges and trade-offs

---

## Performance Guidelines

### Avoid Archetype Thrashing

**❌ BAD:** Adding/removing components in hot loops
```rust
// Every frame, for every entity
commands.entity(e).remove::<CanSeek>();  // Archetype change!
commands.entity(e).insert(CanFlee);      // Archetype change!
```

**✅ GOOD:** Add all capabilities at spawn, mutate state
```rust
// Once at spawn
commands.spawn((CanSeek, CanFlee, BehaviorState::Seeking));

// Every frame (cheap mutation)
creature.behavior = BehaviorState::Fleeing { threat };  // No archetype change
```

### Query Optimization

**❌ BAD:** Iterate all entities, branch on enum
```rust
Query<(&Position, &CreatureState)>  // All creatures
for (pos, state) in query.iter() {
    match state.behavior {  // Branch on every entity
        Seeking => { /* ... */ },
        Fleeing => { /* ... */ },
    }
}
```

**✅ GOOD:** Filter by capability, check state
```rust
Query<(&Position, &BehaviorState), With<CanSeek>>  // Only seekers
for (pos, behavior) in query.iter() {
    if let BehaviorState::Seeking { .. } = behavior {  // Minimal branching
        // ...
    }
}
```

### Parallel Execution

Bevy parallelizes systems automatically if queries don't conflict.

**Systems that CAN run in parallel:**
```rust
fn seek_system(query: Query<&mut Acceleration, With<CanSeek>>) { }
fn flee_system(query: Query<&mut Acceleration, With<CanFlee>>) { }
// Different archetypes → parallel execution
```

**Systems that CANNOT run in parallel:**
```rust
fn system_a(query: Query<&mut Position>) { }
fn system_b(query: Query<&mut Position>) { }
// Both mutate Position → sequential execution
```

---

## Testing Patterns

### Test Setup

```rust
impl Simulation {
    #[cfg(test)]
    pub fn world(&self) -> &World {
        &self.world
    }

    #[cfg(test)]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}
```

### Behavior Testing

```rust
#[test]
fn test_seek_behavior() {
    let mut sim = SimulationBuilder::new().build();

    // Spawn test entity
    let entity = sim.world_mut().spawn((
        Position { x: 0.0, y: 0.0 },
        Velocity::default(),
        Acceleration::default(),
        Target { x: 100.0, y: 0.0 },
        CanSeek,
        BehaviorState::Seeking { target_entity: None },
    )).id();

    // Run simulation
    for _ in 0..100 {
        sim.update();
    }

    // Verify behavior
    let pos = sim.world().get::<Position>(entity).unwrap();
    assert!(pos.x > 0.0, "Should move toward target");
}
```

---

## Common Patterns Reference

### Adding a New Behavior

1. **Consult zoologist-tom** for biological parameters
2. **Add capability marker**: `pub struct CanNewBehavior;`
3. **Add state variant**: `BehaviorState::NewBehavior { data }`
4. **Add data component** (if needed): `pub struct NewBehaviorData { ... }`
5. **Implement system**: Force accumulation pattern
6. **Register system**: Before physics integration
7. **Write tests**: TDD approach
8. **Document in BIOLOGY_NOTES.md**

### Spawning Crits with CritBuilder

**Always use CritBuilder** to create crits. This ensures proper component initialization and follows our architecture.

**Basic Spawn:**
```rust
use speciate::CritBuilder;

// Simple crit at position
let builder = CritBuilder::new()
    .at(100.0, 50.0)
    .with_all_capabilities();
let id = sim.spawn_crit(builder);

// Seeker with target
let id = sim.spawn_seeker(0.0, 0.0, 100.0, 0.0);

// Or using builder directly
let builder = CritBuilder::new()
    .at(0.0, 0.0)
    .as_seeker(100.0, 0.0);  // Includes CanSeek, Target, and Seeking behavior
let id = sim.spawn_crit(builder);
```

**Customization:**
```rust
let builder = CritBuilder::new()
    .at(50.0, 50.0)
    .with_seeking()           // Add seeking capability
    .with_avoidance()         // Add avoidance capability
    .in_behavior(BehaviorMode::Seeking)  // Set behavior state
    .with_energy(75.0)        // Custom energy
    .with_max_speed(30.0);    // Custom speed
let id = sim.spawn_crit(builder);
```

**Testing:**
```rust
#[test]
fn test_crit_behavior() {
    let mut sim = SimulationBuilder::new().build();

    // Quick test crit (all capabilities, catatonic)
    let id = sim.spawn_test_crit(0.0, 0.0);

    // Or use builder for specific config
    let builder = CritBuilder::new()
        .at(0.0, 0.0)
        .as_seeker(100.0, 0.0);
    let id = sim.spawn_crit(builder);

    // Run and verify
    for _ in 0..100 {
        sim.update(0.05);
    }
}
```

**Why CritBuilder:**
- ✅ Clear separation: building vs spawning
- ✅ All capabilities added at construction (archetype stability)
- ✅ Fluent API is discoverable and readable
- ✅ Prevents inconsistent component initialization
- ✅ Easy to extend with new capabilities

**Deprecated:**
```rust
// ❌ OLD (deprecated)
sim.spawn_creature(x, y, 0.0, 0.0);

// ✅ NEW
let builder = CritBuilder::new().at(x, y).with_all_capabilities();
sim.spawn_crit(builder);
```

### State Transitions

```rust
fn transition_system(
    mut query: Query<(&mut BehaviorState, &Energy, &Perception)>
) {
    for (mut behavior, energy, perception) in query.iter_mut() {
        // Priority hierarchy (most urgent first)
        if perception.threat_detected() {
            *behavior = BehaviorState::Fleeing { threat: perception.nearest_threat() };
        } else if energy.is_low() {
            *behavior = BehaviorState::Seeking { target: perception.nearest_food() };
        } else {
            *behavior = BehaviorState::Wandering { angle: 0.0 };
        }
    }
}
```

---

## Summary: Quick Reference

| Concept | Pattern | Why |
|---------|---------|-----|
| **Capabilities** | `CanSeek` (ZST marker) | Permanent, fast filtering, no archetype changes |
| **State** | `BehaviorState::Seeking` | Exclusive modes, cheap mutation, biological realism |
| **Data** | `Target { x, y }` | Pure data, no logic, minimal payload |
| **Forces** | `accel += force` | Additive, extensible, emergent blending |
| **Death** | Add `Dead`, don't remove | 1 archetype change, corpses as ecology |
| **DNA** | Hardcoded → gene expression | Mark TODOs, migrate in Sprint 8+ |
| **Testing** | TDD with `world()` access | Verify behavior over time, not instant state |

---

## See Also

- `/workspace/CLAUDE.md` - Project-wide principles (TDD, DNA-driven design)
- `/workspace/docs/biology/biology-notes.md` - Zoologist consultations log
- `/workspace/docs/biology/dna-driven-design.md` - DNA architecture specification
- Bevy ECS documentation: https://bevyengine.org/learn/book/

---

**Remember:** In ECS, **data IS behavior**. Components represent capabilities and state, systems express logic. Keep it simple, keep it composable, keep it DNA-driven.
