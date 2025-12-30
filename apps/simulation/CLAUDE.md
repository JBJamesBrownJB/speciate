# Simulation ECS Architecture Guide

This document defines the **Bevy ECS patterns and architectural decisions** for the Rust simulation backend.

## Core Principles

1. **Component Composition Over Enums**: Use components to enable behavior blending
2. **Archetype Stability**: Minimize archetype changes for performance
3. **Force Accumulation**: Systems ADD to acceleration, physics integrates
4. **DNA-Driven Parameters**: Behavior constants → DNA gene expression migration path
5. **System Instrumentation**: All ECS systems MUST have timing instrumentation
6. **Dev-Tools Adds, Never Forks**: Feature flags must NOT create behavior differences

---

## Dev-Tools Feature Flag (CRITICAL)

**The `dev-tools` feature flag must ONLY add instrumentation/visualization code. It must NEVER change core behavior.**

### ✅ CORRECT: Dev-tools ADDS on top of production code

```rust
pub fn perception_system(...) {
    // ONE code path - runs identically in prod and dev
    for creature in creatures {
        run_perception(creature, &mut perception);
    }

    // Dev-tools ADDS visualization on top (doesn't change behavior above)
    #[cfg(feature = "dev-tools")]
    if let Some(debug_target) = debug_target.get() {
        capture_debug_snapshot(debug_target, &perception);
    }
}
```

### ❌ WRONG: Dev-tools creates FORKED behavior

```rust
pub fn perception_system(...) {
    #[cfg(feature = "dev-tools")]
    {
        // DIFFERENT code path in dev - BUG RISK!
        run_perception_with_tracking(...);
    }

    // Production code path - might behave differently!
    for creature in creatures {
        run_perception(creature, &mut perception);
    }
}
```

### Why This Matters

- **Bugs hide in forks**: A bug fixed in one path may not be fixed in the other
- **Testing gaps**: Tests may pass in dev but fail in prod (or vice versa)
- **Maintenance nightmare**: Two code paths to maintain, debug, and keep in sync
- **False confidence**: Dev visualization might show correct behavior while prod is broken

### Rule

When adding dev-tools functionality:
1. Write the core behavior ONCE (no `#[cfg]` in the core logic)
2. Dev-tools code only OBSERVES or AUGMENTS the results
3. If you need tracking data, pass a callback or collect it AFTER the core logic runs
4. Never have `#[cfg(feature = "dev-tools")]` around behavior-changing code

---

## System Instrumentation Requirements (MANDATORY)

**CRITICAL: Every new ECS system MUST be instrumented for timing.**

When creating a new system function (any function with `Query<...>` parameters that gets registered with `.add_systems()`), you MUST:

### 1. Add Timing to the System Function

```rust
pub fn my_new_system(
    // ... existing parameters ...
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "my_new_system");

    // ... system logic ...
}
```

### 2. Update SystemTimings Resource

In `apps/simulation/src/instrumentation/mod.rs`:

```rust
pub struct SystemTimings {
    // ... existing fields ...
    pub my_new_system_us: AtomicU64,  // ADD
}

impl SystemTimings {
    pub fn new() -> Self {
        Self {
            // ... existing fields ...
            my_new_system_us: AtomicU64::new(0),  // ADD
        }
    }

    pub fn time(&self, name: &str) -> TimingGuard<'_> {
        let target = match name {
            // ... existing arms ...
            "my_new_system" => &self.my_new_system_us,  // ADD
            _ => panic!("Unknown system: {}", name),
        };
        TimingGuard::new(target)
    }

    pub fn snapshot(&self) -> SystemTimingsSnapshot {
        SystemTimingsSnapshot {
            // ... existing fields ...
            my_new_system_us: self.my_new_system_us.load(Ordering::Relaxed),  // ADD
        }
    }
}

pub struct SystemTimingsSnapshot {
    // ... existing fields ...
    pub my_new_system_us: u64,  // ADD (serde renames to camelCase: myNewSystemUs)
}
```

### 3. Update TypeScript Interfaces

**Dev-UI** (`apps/dev-ui/src/types.ts`):
```typescript
export interface SystemTimingsSnapshot {
  // ... existing fields ...
  myNewSystemUs: number;  // ADD (camelCase)
}
```

**Portal** (`apps/portal/src/types/GameState.ts`):
```typescript
export interface SystemTimingsSnapshot {
  // ... existing fields ...
  myNewSystemUs: number;  // ADD (camelCase)
}
```

### 4. Update SystemTimingsPanel

In `apps/dev-ui/src/components/SystemTimingsPanel.tsx`:
- Add new canvas ref: `const myNewSystemCanvasRef = useRef<HTMLCanvasElement>(null);`
- Add new history ref: `const myNewSystemHistoryRef = useRef<SparklineData>({ history: [], maxHistory: 120 });`
- Update history in useEffect: `updateHistory(myNewSystemHistoryRef.current, timings.myNewSystemUs);`
- Render sparkline in useEffect
- Add entry to `timingEntries` array: `{ name: 'my_new_system', valueUs: timings.myNewSystemUs, canvasRef: myNewSystemCanvasRef }`

### Why This Matters

- **Visibility**: Every system's performance is tracked in dev-ui sparklines
- **Bottleneck Detection**: Auto-sorted display shows slowest systems first
- **Zero Production Cost**: Feature-gated, compiles to nothing in release builds
- **Debugging**: Instant insight when performance degrades

### Verification

After adding instrumentation:
1. `cargo build --features dev-tools` (must compile)
2. `cargo build --no-default-features` (must compile without instrumentation)
3. `npm run build` in dev-ui (TypeScript must compile)
4. Run simulation and verify sparkline appears in dev-ui

**See**: `docs/testing/metrics/README.md` for complete instrumentation guide

---

## Naming Conventions

### Creature vs. Crit

The codebase uses both "Creature" and "Crit" intentionally with distinct meanings:

**"Crit" (Informal, Identifiers):**
- `CritId` - Entity identifier (short, used frequently in queries)
- `CritBuilder` - Fluent builder API (short, used in spawn calls)
- **Rationale:** Brevity for high-frequency usage, friendlier feel

**"Creature" (Formal, Complete States):**
- `CreatureState` - Full biological state (energy, age, etc.)
- `CreatureSnapshot` - IPC data structure
- `CreatureSpawnRequest` - Spawn configuration
- `spawn_creature()` - Public API function
- **Rationale:** Clarity for complete data structures, formal API surface

**Pattern:** Use "Crit" for lightweight identifiers/builders, "Creature" for stateful components and public APIs.

**Migration:** No mass renaming planned. This is an intentional dual-naming convention, not technical debt.

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

## Save State Component Requirements (MANDATORY)

**CRITICAL: When adding new components to `CritBundle`, you MUST handle save-state serialization.**

### Two Patterns for Components

**Pattern 1: Serializable Components**
- Simple data types (scalars, small structs)
- Register in type registry (`simulation/core/simulation.rs`)
- Example: `Position`, `Velocity`, `CritId`, `BehaviorMode`

```rust
// In SimulationBuilder::new()
type_registry.register::<NewComponent>();
```

**Pattern 2: Runtime-Reconstructed Components**
- Fixed-size arrays (serialization issues)
- Entity references (stale after deserialize)
- Cache/computed data (not worth persisting)
- Add reconstruction in `from_save_state()` (`persistence/snapshot.rs`)
- Add a test in `persistence/snapshot.rs`
- Example: `Perception`, `NeighborCache`, `L1Vision`

```rust
// In Simulation::from_save_state()
use crate::simulation::perception::NewComponent;
let entities_needing_component: Vec<Entity> = simulation
    .world
    .query_filtered::<Entity, Without<NewComponent>>()
    .iter(&simulation.world)
    .filter(|e| simulation.world.get::<CritId>(*e).is_some())
    .collect();

for entity in entities_needing_component {
    simulation
        .world
        .entity_mut(entity)
        .insert(NewComponent::new());
}
```

### Checklist for New Components

When adding a component to `CritBundle`:

- [ ] **Decide pattern**: Serializable OR reconstructed?
- [ ] **Serializable**: Add `type_registry.register::<T>()` in `simulation.rs`
- [ ] **Reconstructed**: Add reconstruction code in `from_save_state()`
- [ ] **Add test**: `test_save_state_reconstructs_<component>` in `snapshot.rs`
- [ ] **Run tests**: `cargo test save_state`

### Why This Matters

Save-state enables:
- Game save/load
- Crash recovery
- Testing (reproducible scenarios)

Missing components = broken restore = frustrated users.

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

### Test Anti-Patterns (NEVER DO)

**❌ NEVER hardcode expected values derived from constants:**

```rust
// BAD: Hardcodes 1.25, breaks when PANIC_THRESHOLD_RATIO changes
assert_eq!(panic, 1.25);

// GOOD: Uses the actual constant, tests the relationship
assert_eq!(panic, personal_space * PANIC_THRESHOLD_RATIO);
```

**Why this matters:**
- Constants are tuning parameters that WILL change during development
- Hardcoded values create false test failures when constants are adjusted
- Tests should verify *behavior and relationships*, not specific numeric outputs

**✅ DO test:**
- Relationships between values (`panic < personal_space`)
- Behavior over time (`pos.x > initial_x` after seeking)
- Invariants (`neighbor_count <= MAX_NEIGHBORS`)
- Edge cases (empty input, boundary conditions)

**❌ DON'T test:**
- Exact values derived from tunable constants
- Implementation details that may change
- Specific numeric outputs that depend on arbitrary parameters

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

## Electron Integration (Phase 1)

### Overview

Phase 1 uses **Electron** to bundle the Rust simulation subprocess with the TypeScript/PixiJS frontend. The simulation runs locally as a child process communicating via stdio (no network, no separate server).

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  ELECTRON APPLICATION                        │
├──────────────────────────┬───────────────────────────────────┤
│  RUST SUBPROCESS         │  FRONTEND (PixiJS)               │
│  (Bevy ECS)              │  (Renderer Process)              │
│                          │                                   │
│  Simulation Loop:        │  app.ticker (60 FPS):            │
│  • AI & Decision Making  │  • Receive state-update events   │
│  • Steering Behaviors    │  • Update sprite positions       │
│  • Physics Integration   │  • Render frame                  │
│                          │                                   │
│  stdout MessagePack:     │                                   │
│  • Serialize GameState   │  Main Process:                   │
│  • 4-byte length prefix  │  • Read stdout frames            │
│  • Binary payload   ─────┼──> Decode MessagePack            │
│  • Stream at tick rate   │  • Forward to renderer           │
└──────────────────────────┴───────────────────────────────────┘
```

### stdio MessagePack Protocol

**Problem:** Need efficient IPC between Rust subprocess and Electron main process.

**Solution:** Length-prefixed MessagePack frames over stdout/stdin.

**Protocol Format:**
```
┌─────────────┬─────────────────────────────┐
│   4 bytes   │      N bytes                │
│  (u32 BE)   │   (MessagePack payload)     │
│   Length    │      GameState              │
└─────────────┴─────────────────────────────┘
```

**Rust (Simulation) - Write:**
```rust
use rmp_serde;
use std::io::{self, Write};

fn write_frame(state: &GameState) -> io::Result<()> {
    // Serialize to MessagePack
    let payload = rmp_serde::to_vec(state)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Write 4-byte length prefix (big-endian)
    let len = payload.len() as u32;
    io::stdout().write_all(&len.to_be_bytes())?;

    // Write payload
    io::stdout().write_all(&payload)?;
    io::stdout().flush()?;

    Ok(())
}
```

**Electron (Main Process) - Read:**
```javascript
const msgpack = require('msgpack-lite');

simulationProcess.stdout.on('data', (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);

    while (buffer.length >= 4) {
        const frameLength = buffer.readUInt32BE(0);
        if (buffer.length < 4 + frameLength) break;

        const payload = buffer.slice(4, 4 + frameLength);
        buffer = buffer.slice(4 + frameLength);

        const state = msgpack.decode(payload);
        mainWindow.webContents.send('state-update', state);
    }
});
```

**Benefits:**
- **Simple:** No shared memory complexity
- **Efficient:** Binary serialization, ~1KB per frame
- **Lock-free:** Subprocess writes, main process reads (no coordination)
- **Full precision:** f32 coordinates, no quantization

### Event Streaming (Future)

For significant events (death, reproduction), we can extend the protocol with event frames:

```rust
enum Frame {
    State(GameState),
    Event(GameEvent),
}

// Future enhancement - not yet implemented
```

### Persistence (Phase 1: None)

**Phase 1 Strategy:** In-memory only. No database, no save files.

- World resets on app close
- Focus on gameplay, not persistence
- Faster iteration (no migration headaches)

**Phase 2 Strategy:** SQLite for save/load.

---

## Tick Architecture

### Current Implementation (Single-Tick)

The simulation runs at a **hardcoded tick rate of 22.2Hz** (implemented in Sprint 13 NAPI migration). This provides ~45ms per tick, allowing for 150K-200K creature scaling. Future work will add frontend interpolation for smooth 60Hz visuals.

**Current Architecture:**
```rust
pub struct Simulation {
    world: World,
    schedule: Schedule,  // All systems run together
}

impl Simulation {
    pub fn update(&mut self, delta_time: f32) {
        self.world.insert_resource(DeltaTime(delta_time));
        self.schedule.run(&mut self.world);
    }
}
```

**System Order (Sequential):**
1. **Perception** - Query spatial grid for nearby entities
2. **Behavior Transition** - Update behavioral state machine
3. **Steering Behaviors** - Calculate seek, flee, wander, avoidance forces (accumulate into Acceleration)
4. **Motion Integration** - Velocity += Acceleration, Position += Velocity (Euler)
5. **Rotation** - Update creature orientation based on velocity
6. **Reset Acceleration** - Clear acceleration for next tick
7. **Snapshot** - Create state for frontend (IPC via background writer thread)

### Tick Rate Implementation (Sprint 13)

**Current Status:** 22.2Hz hardcoded in `napi_addon/simulation_engine.rs`

**Implementation:**
```rust
// apps/simulation/src/napi_addon/simulation_engine.rs:37
const TARGET_SIMULATION_HZ: f32 = 22.2;
```

This replaced the old configurable approach from `config.rs` used by the deprecated stdio runner.

**Benefits:**
- ~45ms per tick (2.7x improvement vs 60Hz)
- Sufficient for 150K-200K creature target
- Simpler architecture (no runtime configuration)

### Future Direction: Frontend Interpolation (Sprint 14)

**Goal:** Smooth 60Hz visuals while maintaining 22.2Hz physics simulation.

**Planned Approach:**
- Frontend interpolates between 22.2Hz physics updates to 60Hz display
- `lerp(prev_pos, curr_pos, alpha)` where `alpha = time_since_last_tick / 45ms`
- Position AND rotation interpolation to prevent sliding artifacts

**Why not dual-tick?** Dual-tick architecture (separate AI/Physics schedules) was explored and abandoned in Sprint 11. Sequential execution on single thread provides no benefit - when schedules align, you still get the same spike. Only true parallelism (separate threads) would help, which requires major architectural changes.

**See Sprint 14 plan for interpolation implementation details.**

### IPC Optimization (Sprint 13 - Completed)

**Current Implementation:** NAPI-RS zero-copy double-buffer

**Architecture:**
- Zero-copy position buffers (direct TypeScript access to Rust memory)
- Double-buffering (simulation writes to back buffer, frontend reads from front)
- ThreadsafeFunction for telemetry events
- No serialization overhead (<1ms buffer swap)

**Performance Improvement:**
- Old (stdio MessagePack): ~30ms frame serialization overhead
- New (NAPI zero-copy): <1ms for buffer swap
- **Result:** 97% reduction in IPC overhead

**See:** `docs/architecture/napi-architecture.md` for implementation details

---

## See Also

- `/workspace/CLAUDE.md` - Project-wide principles (TDD, DNA-driven design, Electron IPC patterns)
- `/workspace/docs/biology/biology-notes.md` - Zoologist consultations log
- `/workspace/docs/biology/dna-driven-design.md` - DNA architecture specification
- `/workspace/docs/architecture/electron-architecture.md` - Electron IPC patterns and desktop build
- Bevy ECS documentation: https://bevyengine.org/learn/book/
- Electron documentation: https://www.electronjs.org/

---

**Remember:** In ECS, **data IS behavior**. Components represent capabilities and state, systems express logic. Keep it simple, keep it composable, keep it DNA-driven.
