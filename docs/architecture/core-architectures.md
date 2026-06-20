# Core Architectures

This document indexes the foundational architectural principles that ALL features must align with. Read this first before adding new features.

---

## Quick Reference

| Architecture | Problem | Solution | Reference |
|--------------|---------|----------|-----------|
| **DNA-Driven Design** | Hardcoded traits prevent evolution | Encode primitives in DNA, behavior emerges | `docs/biology/ideas/dna-driven-design.md` |
| **Force Accumulation** | Multiple behaviors compete | All forces ADD to acceleration | `docs/architecture/behavior-engine.md` |
| **Two-Level Spatial Grid** | O(N^2) perception | L0 (20m) + L1 (60m) hierarchy | `docs/performance/done/hierarchical-spatial-grid.md` |
| **ECS Capability Markers** | Archetype thrashing | ZST markers added at spawn, never removed | `docs/architecture/ecs-optimization-playbook.md` |
| **Frequency Throttling** | Expensive cognitive systems | Entity-ID bucketing with bitwise AND | `docs/performance/done/system-update-frequency.md` |
| **Binary IPC** | JSON serialization kills FPS | Zero-copy Float32Array buffers | `docs/architecture/electron-architecture.md` |
| **Snapshot Interpolation** | 20 Hz sim looks jerky on a 60–120 Hz screen | Render ~1 tick in the past; drive α from a playout clock, never reset on arrival | `docs/architecture/snapshot-interpolation.md` |

---

## 1. DNA-Driven Design

### The Problem

Hardcoded creature traits (speed, size, perception) prevent:
- Genetic diversity and evolution
- Emergent ecological niches
- Player-observable variety

### The Architecture

**DNA encodes primitive traits. Complex behaviors EMERGE from combinations.**

| DO Encode | DON'T Encode |
|-----------|--------------|
| Physical parameters (size, speed) | "Sociality" (emerges from personal_space + flocking) |
| Simple thresholds (hunger, flee trigger) | "Intelligence" (emerges from perception + reaction time) |
| Binary flags (diurnal/nocturnal) | "Dominance" (emerges from aggression + size) |

**Trade-offs are mandatory:** Every advantage must have a cost built into physics/biology.
- Large size = higher speed BUT massive energy consumption
- High speed = escape predators BUT burns energy rapidly

**Golden Zone:** Seek optimizations that ARE the biological feature.

| Optimization | Free Biological Behavior |
|--------------|-------------------------|
| Skip small entities in perception | Size domination (giants ignore mice) |
| Skip stationary targets | Prey freeze = camouflage |
| Satiated creatures skip prey detection | Post-meal predators rest |

### Key Rules

- NEVER hardcode creature traits with magic numbers
- ALWAYS derive from individual creature DNA
- Consult zoologist-tom agent for biological bounds before adding genes
- Trade-offs must be systemic (built into physics), not arbitrary penalties

### Reference

Full details: `docs/biology/ideas/dna-driven-design.md`

---

## 2. Force Accumulation Pattern

### The Problem

Multiple behaviors compete for control (seek target vs avoid obstacle). Priority-based systems are brittle and don't blend naturally.

### The Architecture

**All steering behaviors ADD forces to Acceleration. Physics integrates the sum.**

```
Tick Flow:
1. Wander system:     accel += wander_force
2. Seek system:       accel += seek_force
3. Flee system:       accel += flee_force
4. Avoidance system:  accel += avoidance_force
5. Physics:           velocity += accel * dt; position += velocity * dt
6. Reset:             accel = (0, 0)
```

**Priority through magnitude:** Stronger forces (panic flee > obstacle avoidance > casual seek) naturally dominate without explicit priority logic.

**Benefits:**
- Natural blending: Seek + avoid = emergent path around obstacles
- Extensible: Add new behaviors without modifying existing ones
- Biologically realistic: Multiple sensory inputs, single motor output

### Key Rules

- Systems write `accel.ax += force.x`, NEVER `accel.ax = force.x`
- System order: Behaviors (parallel) → Physics Integration → Constraints
- Acceleration resets to zero at end of each tick
- Higher urgency behaviors use larger force magnitudes

### Reference

Full details: `docs/architecture/behavior-engine.md`
Implementation: `apps/simulation/src/simulation/creatures/steering/system.rs`

---

## 3. Two-Level Spatial Grid (L0/L1)

### The Problem

Naive perception is O(N^2) - every creature checks every other creature. Doesn't scale past 10K entities.

### The Architecture

**Two-level hierarchy with classification-based early exit.**

| Level | Cell Size | Purpose | Data |
|-------|-----------|---------|------|
| **L0** | 20m | Fine perception, collision | Entity positions, velocities |
| **L1** | 60m (3x3 L0) | Strategic classification | BioSignature: total_mass, max_size, creature_count |

**Query flow:**
1. Check L1 cell classification (Threat/Prey/Empty)
2. If Empty → skip all L0 cells in that region
3. If Threat/Prey → query L0 cells for detailed entity data

**Double-buffered:** Read from front buffer, write to back buffer, swap at tick end. Prevents read-write conflicts during parallel execution.

**Size domination:** Creatures ignore entities below 5% of their body mass (threshold stored in perception component).

### Key Rules

- Always check L1 before querying L0 (early exit optimization)
- BioSignature aggregation runs after L0 rebuild, before buffer swap
- Fixed-size arrays in components (no Vec allocations per tick)
- Portal visualizes grid with G key: Off → L0 → L1

### Reference

Full details: `docs/performance/done/hierarchical-spatial-grid.md`
Implementation: `apps/simulation/src/simulation/spatial/`

---

## 4. ECS Capability Markers

### The Problem

Adding/removing components causes archetype changes. At 100K+ entities, archetype thrashing destroys cache performance.

### The Architecture

**Three-tier component model:**

| Tier | Type | Lifetime | Example |
|------|------|----------|---------|
| 1. Capability Markers | Zero-sized types (ZST) | Added at spawn, NEVER removed | `CanSeek`, `CanFlee`, `CanWander` |
| 2. Behavioral State | Enum component | Mutated freely | `BehaviorMode::Wandering` |
| 3. Data Components | Pure data | Added/removed as needed | `Target { x, y }` |

**Capability markers enable fast query filtering:**
```rust
Query<..., With<CanSeek>>  // Only entities that can seek
Query<..., Without<Dead>>  // Exclude dead entities
```

**Archetype stability:** Entities keep same archetype throughout lifetime. State changes via enum mutation, not component add/remove.

### Key Rules

- Capabilities: Add ALL at spawn, NEVER remove
- State changes: Mutate `BehaviorMode` enum, don't change components
- Death handling (planned — no mortality system exists yet): when added, use a deferred `Dead` marker instead of despawning in the hot path. See `docs/biology/ideas/mortality.md`
- Query with `With<>`/`Without<>` for filtering, not component presence

### Reference

Full details: `docs/architecture/ecs-optimization-playbook.md`
Implementation: `apps/simulation/src/simulation/creatures/components/capabilities.rs`

---

## 5. Frequency Throttling

### The Problem

Cognitive systems (perception, behavior decisions) are expensive. Not every creature needs updates every tick.

### The Architecture

**Entity-ID bucketing with bitwise AND optimization.**

```rust
// Power-of-2 divisors only (2, 4, 8)
let should_process = (entity_id & (divisor - 1)) == (tick & (divisor - 1));
```

**Why bitwise AND:** Modulo (~30 cycles) vs AND (~1 cycle).

**Why minimum divisor is 2:** Cache line false sharing at divisor=1 causes performance variance across Rayon workers.

| System | Throttled? | Rationale |
|--------|------------|-----------|
| Perception | Yes (2, 4, 8) | Stale data acceptable (reaction time) |
| Behavior Transition | Yes (2, 4, 8) | Decision-making, not physics |
| Steering | NO | Throttling caused jerky movement |
| Movement Integration | NO | Physics requires every-tick |
| Grid Rebuild | NO | Perception accuracy depends on current positions |

### Key Rules

- Power-of-2 divisors ONLY (2, 4, 8)
- Minimum divisor is 2 (no "full rate" option)
- NEVER throttle physics integration
- Entity-ID based (not position-based) to avoid visual artifacts

### Reference

Full details: `docs/performance/done/system-update-frequency.md`
Implementation: `apps/simulation/src/simulation/core/frequency_throttle.rs`

---

## 6. Binary IPC Pattern

### The Problem

JSON serialization between Rust simulation and TypeScript frontend kills FPS. `serde_json::to_string()` + `JSON.parse()` costs 5-20ms even for small payloads.

### The Architecture

**Zero-copy Float32Array buffers via NAPI-RS.**

| Data Type | Format | Frequency |
|-----------|--------|-----------|
| Creature positions | Float32Array | Every tick |
| L1 heatmap data | Float32Array | Every tick |
| Config changes | JSON | < 1Hz |
| Save/load | JSON | On demand |

**Pattern:**
```
Rust: fill_buffer(mut buffer: Float32Array) → writes directly to shared memory
Electron: Passes buffer to renderer via IPC
TypeScript: Reads Float32Array directly (no parsing)
```

**Double-buffered:** Front buffer for reading, back buffer for writing. Lock-free access.

### Key Rules

- Per-tick data: MUST use binary buffers (Float32Array)
- Low-frequency data (< 1Hz): JSON acceptable
- NEVER `serde_json` on hot path
- Complex nested structures: JSON acceptable if infrequent

### Reference

Full details: `docs/architecture/electron-architecture.md`
Implementation: `apps/simulation/src/napi_addon/simulation_engine.rs`

---

## 7. Snapshot Interpolation (smooth motion across the seam)

### The Problem

The sim commits positions at 20 Hz (every 50 ms); the screen redraws at 60–120 Hz. Showing the latest snapshot and restarting the slide on each arrival turns the jittery NAPI-seam delivery into visible **snap** (gap < 50 ms) and **freeze** (gap > 50 ms) — the high-population jitter bug, even with CPU headroom to spare.

### The Architecture

**Treat the NAPI seam as a tiny network: render in the past.** Buffer snapshots, render ~1 tick behind, and drive the interpolation α from a real-time playout clock — never reset α when a snapshot arrives (arrival only appends to the buffer).

```
snapshots:  A ──── B ──── C (latest)
render clock = now − 1 tick:     ●  interpolate B→C here
α = clock / tickInterval   (rolls over between pairs; never reset on arrival)
```

- **Always a snapshot ahead** → the slide never stalls at 1.0 waiting for data.
- **Underrun holds** at the newest position (no extrapolation/overshoot).
- **Match by creature id** across snapshots (new id → start = end; departed → dropped).
- **GC ring:** each snapshot is copied into a pre-allocated SoA pool slot (no per-tick allocation) — pre-allocate-and-reuse, like the rest of the hot path.

### Key Rules

- NEVER reset the interpolation α on snapshot arrival (the core invariant)
- Render in the past (≈1 tick) so the buffer is always ≥1 deep
- Never allocate per-snapshot on the hot path — fill pooled SoA slots
- Verify with the dev-ui **Stall frames** metric (drive to ~0%)

### Reference

Full details: `docs/architecture/snapshot-interpolation.md`
Implementation: `apps/portal/src/rendering/SnapshotInterpolator.ts`, `InterpolatedCreatureRenderer.ts`

---

## Enforcement Checklist

### Before Adding a New Feature

- [ ] **DNA Check:** Does this add creature traits?
  - YES → Derive from DNA, not hardcoded constants
  - Consult zoologist-tom for biological bounds

- [ ] **Force Check:** Does this affect movement?
  - YES → ADD to acceleration (`accel += force`), never replace
  - Follow system ordering (behaviors → physics → constraints)

- [ ] **Grid Check:** Does this use spatial data?
  - YES → Use L0/L1 hierarchy appropriately
  - Check L1 classification before querying L0

- [ ] **ECS Check:** Does this add components?
  - Capabilities → Add at spawn, never remove (ZST markers)
  - State → Use enum mutation, not component add/remove

- [ ] **Throttle Check:** Is this expensive/cognitive?
  - YES → Consider frequency throttling (power-of-2 divisor)
  - NO for physics integration

- [ ] **IPC Check:** Does this cross Rust↔TypeScript boundary?
  - High frequency → Binary buffers (Float32Array)
  - Low frequency → JSON acceptable

### Before Creating a PR

- [ ] Consulted zoologist-tom for biological parameters (if applicable)
- [ ] Ran `cargo test` and `npm test`
- [ ] Force accumulation pattern followed (no `accel = force`)
- [ ] Capability markers added at spawn only
- [ ] IPC using appropriate format

---

## Architecture Decision Records (ADR)

Archived decisions documenting what was tried and abandoned:

| Decision | Outcome | Location |
|----------|---------|----------|
| Dual-tick simulation | Abandoned - no parallelism benefit | `docs/archive/dual-tick/README.md` |
| stdio MessagePack IPC | Replaced by NAPI-RS - 10x faster | `docs/archive/stdio/README.md` |
| Perception frame skip | Abandoned - worse than throttling | `docs/archive/perception-skip/README.md` |

---

*Last updated: 2026-06-20*
