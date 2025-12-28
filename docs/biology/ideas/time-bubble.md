# Time Bubble - Accelerated Evolution Zones

**Status:** IDEA (Technical Feasibility Analysis)
**Author:** bevy-ecs-bishop
**Date:** 2025-12-28

## Concept

A player-placed beacon creates a spherical region where simulation runs at 1000x speed. Creatures can freely cross the boundary - when entering, they transfer to the bubble simulation; when exiting, they return to the main simulation.

## Use Cases

1. **Accelerated Evolution Experiments** - Watch 1000 generations evolve in real-time
2. **Safe Haven Breeding Grounds** - Combined with barrier thumpers for protected speciation
3. **Player-Driven Speciation Events** - Create isolated populations that diverge rapidly

---

## Technical Feasibility Assessment

### Architecture: Dual Bevy Worlds

**Verdict: FEASIBLE but requires careful design**

Bevy supports multiple `World` instances. The current architecture in `/home/dev/dev/speciate/apps/simulation/src/simulation/core/simulation.rs` already wraps a `World`:

```rust
pub struct Simulation {
    pub(crate) world: World,
    schedule: Schedule,
    assets_path: Option<std::path::PathBuf>,
}
```

A Time Bubble would instantiate a second `Simulation`:

```rust
pub struct TimeBubble {
    simulation: Simulation,
    center: Vec2,
    radius: f32,
    time_multiplier: f32,  // e.g., 1000.0
    main_sim_tick_at_creation: u64,
}
```

**Performance Note:** Two Bevy Worlds are independent - schedules run sequentially unless we spawn threads. At 1000x, the bubble sim would need to tick 1000 times per main sim tick. This is CPU-bound but achievable if:
- Bubble creature count is capped (e.g., 1000 creatures max)
- Bubble runs in a dedicated thread
- Main sim tick budget is respected

---

## Key Challenges

### 1. Entity Identity Preservation

**Problem:** Bevy `Entity` IDs are World-local. Entity(42) in World A is NOT Entity(42) in World B.

**Solution:** Use `CritId` (already exists in `/home/dev/dev/speciate/apps/simulation/src/simulation/creatures/components/identity.rs:7`):

```rust
pub struct CritId(pub u32);
```

This is a stable, simulation-global identifier. When transferring:
1. Serialize creature by `CritId` from source World
2. Despawn from source World
3. Spawn in destination World with same `CritId`

**Implementation:**

```rust
pub struct CreatureTransferPacket {
    crit_id: CritId,
    position: Position,
    velocity: Velocity,
    body_size: BodySize,
    rotation: Rotation,
    dna: Dna,
    state: CreatureState,
    brain: Brain,
    // ... all serializable components
}

impl CreatureTransferPacket {
    pub fn extract_from(world: &World, entity: Entity) -> Self {
        // Read all components, create packet
    }

    pub fn spawn_into(self, world: &mut World) -> Entity {
        // Spawn with all components
    }
}
```

**Key Insight:** Components are already `Serialize`/`Deserialize` (for save/load). This infrastructure exists.

### 2. Time Synchronization at Boundary Crossing

**Problem:** When a creature exits the bubble after 1000x ticks, what "time" does it return to?

**Two Models:**

#### Model A: Absolute Time (Simpler)
- Bubble runs ahead in absolute time
- Creature exits into "the future" relative to main sim
- Main sim doesn't know about bubble's future - just accepts the aged creature
- **Risk:** Time paradoxes if bubble creatures affect main sim in ways that should have propagated

#### Model B: Relative Time (Complex but Correct)
- Bubble is a "pocket dimension" - its time is meaningless to main sim
- Creature's `age` field is scaled on exit: `main_age = bubble_age / time_multiplier`
- Evolution/mutation counts are real, but "perceived time" is compressed
- **Benefit:** No paradoxes, cleaner mental model

**Recommendation:** Model B (Relative Time). The bubble is a hyperspace - biological changes are real, but calendar time is not.

```rust
fn transfer_out_of_bubble(
    creature: &mut CreatureTransferPacket,
    bubble: &TimeBubble,
    main_tick: u64,
) {
    // Creature spent N bubble ticks inside
    let bubble_ticks_spent = bubble.current_tick - creature.entered_at_bubble_tick;

    // Convert to main-sim equivalent (compressed)
    let main_equivalent_ticks = bubble_ticks_spent / bubble.time_multiplier as u64;

    // Age is real (mutations happened), but we don't advance main sim time
    // Creature exits at current main_tick, not main_tick + bubble_ticks_spent
}
```

### 3. Archetype Stability During Transfer

**Problem:** If source and destination Worlds have different archetypes registered, component insertion could fail or fragment.

**Solution:** Both Worlds use identical `SimulationBuilder` initialization:

```rust
impl TimeBubble {
    pub fn new(center: Vec2, radius: f32, time_multiplier: f32) -> Self {
        // Use same builder as main sim - identical archetype registration
        let simulation = SimulationBuilder::new()
            .with_default_systems()  // Same systems
            .build();

        Self { simulation, center, radius, time_multiplier }
    }
}
```

**Guarantee:** If both Worlds are built identically, archetypes will match. No fragmentation on transfer.

### 4. Boundary Detection and Transfer Trigger

**Problem:** How do we detect creatures crossing the bubble boundary?

**Solution:** Spatial query in main sim's movement system:

```rust
fn bubble_boundary_system(
    mut commands: Commands,
    bubbles: Res<ActiveTimeBubbles>,
    creatures: Query<(Entity, &CritId, &Position), With<CreatureState>>,
    mut transfer_queue: ResMut<BubbleTransferQueue>,
) {
    for (entity, crit_id, pos) in creatures.iter() {
        for bubble in bubbles.iter() {
            let distance = pos.distance_to(bubble.center);

            if distance < bubble.radius {
                // Inside bubble - queue for transfer
                transfer_queue.enter_bubble.push((entity, *crit_id, bubble.id));
            }
        }
    }
}
```

**Optimization:** This is O(N * B) where N = creatures, B = bubbles. If B is small (1-3 bubbles), this is fine. For many bubbles, use spatial hashing to find nearby bubbles first.

### 5. Visual Representation

**Problem:** How does the frontend show bubble contents?

**Two Options:**

#### Option A: Sampled Snapshots
- Bubble sim emits snapshots at 1/1000th rate (so frontend sees ~60fps equivalent)
- Frontend renders bubble contents in a "time-lapse" style
- Creatures appear to move 1000x faster visually

#### Option B: Abstracted View
- Don't render individual creatures inside bubble
- Show aggregate stats: population count, average size, generation count
- "Time Crystal" visualization with swirling particles
- On exit, creatures "materialize" fully rendered

**Recommendation:** Option B for MVP. Rendering 1000 ticks worth of movement is visually chaotic. An abstracted view is more readable and cheaper.

---

## Performance Analysis

### CPU Budget

Main sim target: 20Hz (50ms per tick)
Main sim creature budget: 20K creatures

**Bubble Scenario:**
- Bubble contains 500 creatures
- Bubble runs 1000x = 1000 ticks per main tick
- Bubble tick time estimate: 0.05ms per tick (500 creatures is fast)
- Total bubble time: 50ms per main tick

**Problem:** Bubble consumes entire tick budget.

**Solution:** Dedicated thread for bubble simulation:

```rust
pub struct TimeBubbleManager {
    bubble_thread: JoinHandle<()>,
    command_tx: Sender<BubbleCommand>,
    state_rx: Receiver<BubbleState>,
}
```

Bubble runs in parallel with main sim. Main sim doesn't wait for bubble - they communicate via channels.

### Memory

Two Worlds = two sets of ECS storage. At 500 bubble creatures:
- Archetype storage: ~100KB
- Component data: ~500 creatures * ~500 bytes = 250KB
- Total overhead: <1MB

**Verdict:** Negligible memory cost.

---

## Golden Zone Analysis

**Does this optimization match real biology?**

| Aspect | Biological Parallel | Golden Zone? |
|--------|---------------------|--------------|
| Isolated population | Island biogeography | YES |
| Accelerated generations | Bacterial evolution (fast reproducing) | YES |
| Boundary crossing | Migration between ecosystems | YES |
| Time dilation | No real equivalent | Neutral (fantasy element) |

**Verdict:** Partial Golden Zone. The isolation and migration aspects are biologically authentic. Time dilation is a fantasy mechanic but doesn't break biological logic - it's more like "watching bacteria in a petri dish" vs "watching elephants in the wild."

---

## Suggested Architecture

```
Main Thread                    Bubble Thread
    |                              |
    v                              v
[Main Simulation]            [Bubble Simulation]
    |                              |
    +-- tick() ------------------>-+-- tick() x 1000
    |                              |
    +-- boundary_check() ----------+
    |                              |
    +-- transfer_out() <----------+-- exit_queue
    |                              |
    +-- transfer_in() ------------>+-- enter_queue
    |                              |
    +-- snapshot() ----------------|-- (no direct snapshot)
    |                              |
    v                              v
[Frontend]                   [Aggregate Stats Only]
```

### Data Structures

```rust
// Shared between threads via channels
pub enum BubbleCommand {
    TransferIn(CreatureTransferPacket),
    SetTimeMultiplier(f32),
    Shutdown,
}

pub enum BubbleEvent {
    TransferOut(CreatureTransferPacket),
    Stats(BubbleStats),
}

pub struct BubbleStats {
    creature_count: u32,
    average_size: f32,
    generation_count: u32,
    total_births: u64,
    total_deaths: u64,
}
```

---

## Implementation Phases

### Phase 1: Single-Threaded Prototype
- Second `Simulation` instance
- Manual tick multiplier (10x for testing)
- Basic transfer packet serialization
- No boundary detection (manual trigger)

### Phase 2: Threaded Bubble
- Spawn bubble simulation on dedicated thread
- Channel-based communication
- Proper 1000x tick rate

### Phase 3: Boundary System
- Spatial detection of boundary crossings
- Automatic transfer in/out
- Time scaling on exit

### Phase 4: Frontend Integration
- Aggregate stats display
- Bubble visualization (abstract)
- Entry/exit particle effects

---

## Open Questions for Design Review

1. **Creature Relationships:** If creature A is chasing creature B, and A enters bubble but B doesn't - what happens to A's target? (Answer: Clear target on transfer, let AI re-acquire)

2. **Reproduction in Bubble:** Do bubble offspring get new `CritId`s from a shared counter? (Answer: Yes, use AtomicU32 shared between threads)

3. **Multiple Bubbles:** Can bubbles overlap? Can a creature be in two bubbles? (Answer: No overlap allowed - spatial constraint)

4. **Bubble Lifespan:** Fixed duration or player-controlled? (Design decision)

5. **Resource Cost:** What does it cost to place a bubble? (Gameplay balance)

---

## Conclusion

**Technical Feasibility: HIGH**

The dual-simulation architecture is sound. Key risks are:
1. Transfer packet completeness (must capture ALL creature state)
2. Thread synchronization for transfer queues
3. CPU budget management if bubble creature count grows

**Recommended Next Step:** Build Phase 1 prototype with 10x multiplier, validate transfer packet round-trip preserves creature behavior.
