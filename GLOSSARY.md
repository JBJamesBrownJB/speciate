# Speciate Terminology Glossary

**Version:** 2.0.0
**Last Updated:** Sprint 9
**Status:** Active

---

## Purpose

This glossary defines the official terminology used throughout the Speciate codebase, documentation, and communication. Consistent terminology helps maintain code clarity and reduces confusion across the development team.

---

## Core Entities

### Creature / Creatures
**Definition:** Autonomous agents in the simulation with emergent A-Life behavior.

**Usage:**
- Singular: "creature"
- Plural: "creatures"
- **Examples:**
  - "Each creature has a unique ID"
  - "The simulation manages 1000 creatures"
  - "Creature behavior is driven by steering forces and DNA"

**Previously:** agents, crits, boids

**Code References:**
- `CreatureId` - Unique identifier component
- `CreatureSnapshot` - Position, velocity, and state snapshot
- `creature_systems.rs` - ECS systems file

---

### Portal
**Definition:** The Electron desktop frontend where players interact with the simulation.

**Usage:**
- Always capitalized as "Portal" when referring to the application
- **Examples:**
  - "The Portal renders creatures at 60 FPS"
  - "Portal receives state updates via stdio IPC"
  - "Portal uses PixiJS for WebGL rendering"

**Previously:** UI, frontend

**Code References:**
- `apps/portal/` - Portal application directory
- `ElectronIPCClient` - IPC interface

---

## Technical Terms

### CreatureId
**Type:** Component (Rust), number (TypeScript)

**Definition:** Stable, unique identifier for each creature. Assigned at spawn time and never changes.

**Rust:** `Entity` (Bevy ECS), exposed as `u32`
**TypeScript:** `number` (uint32 range)

**Usage:**
```rust
// Rust
entity.index() // Returns u32 ID

// TypeScript
const creatureId: number = 42;
```

---

### CreatureSnapshot
**Type:** Struct (Rust), Interface (TypeScript)

**Definition:** Complete state snapshot of a creature including position, heading, and vital stats.

**Fields:**
- `id` - Unique identifier (u32/number)
- `x`, `y` - World position (f32/number)
- `heading` - Direction in radians, 0 to 2π (f32/number)
- `body_radius` - Physical size (f32/number)
- `energy` - Current energy level (f32/number)

**Usage:**
```rust
// Rust
CreatureSnapshot {
    id: 1,
    x: 45.23,
    y: 78.91,
    heading: 1.57,
    body_radius: 2.0,
    energy: 85.5,
}
```

```typescript
// TypeScript
const creature: CreatureSnapshot = {
  id: 1,
  x: 45.23,
  y: 78.91,
  heading: 1.57,
  body_radius: 2.0,
  energy: 85.5,
};
```

---

### Behavior Mode
**Type:** Enum

**Definition:** State machine state for creature behavior.

**Values:**
- `Wandering` - Random exploration (default)
- `Seeking` - Moving toward target
- `Fleeing` - Escaping from danger
- `Feeding` - Consuming resources (future)

**Usage:**
```rust
pub enum BehaviorMode {
    Wandering,
    Seeking,
    Fleeing,
    Feeding,
}
```

---

## System Components

### Simulation
**Definition:** Bevy ECS simulation engine running as Rust subprocess.

**Responsibility:** Single source of truth for all game state. Physics, creature behaviors, and state updates.

**Tech Stack:** Rust, Bevy ECS

**Location:** `apps/simulation/`

**IPC:** Writes MessagePack frames to stdout at configured tick rate

---

## Steering Behaviors (Nature of Code)

### Wander
**Definition:** Smooth random exploration using steering forces. Projects a circle ahead of the creature and picks a random point on the circumference.

**Parameters:**
- Territory-based with elastic tether to home point
- Perlin noise for organic movement

---

### Avoidance
**Definition:** Collision avoidance through obstacle detection and steering. Creatures detect obstacles ahead and steer around them.

**Parameters:**
- Detection distance based on creature speed
- Raycasting for obstacle detection

---

## Architecture Patterns

### ECS (Entity Component System)
**Definition:** Data-oriented architecture pattern where:
- **Entities** are unique IDs
- **Components** are data structs (Position, Velocity, etc.)
- **Systems** are pure functions that process components

**Library:** Bevy ECS

---

### Spatial Hash
**Definition:** Grid-based spatial partitioning for O(1) neighbor queries.

**Use Case:** Efficient creature-to-creature interaction checks without O(n²) full-collection scanning.

**Cell Size:** 2-3x the typical interaction radius

---

## Legacy Terms (Deprecated)

| **Deprecated** | **Current** | **Changed In** |
|----------------|-------------|----------------|
| Agent/Agents   | Creature/Creatures | Sprint 6 |
| Crit/Crits     | Creature/Creatures | Sprint 9 |
| AgentId        | CreatureId  | Sprint 6       |
| CritId         | CreatureId  | Sprint 9       |
| AgentTransform | CreatureSnapshot | Sprint 6  |
| CritTransform  | CreatureSnapshot | Sprint 9  |
| UI             | Portal      | Sprint 6       |
| `apps/ui/`     | `apps/portal/` | Sprint 6    |

---

## Communication Guidelines

### Do ✅
- Use "creature" for individual creatures
- Use "creatures" for the plural
- Use "Portal" (capitalized) for the Electron application
- Use "CreatureSnapshot" for state snapshots
- Use "CreatureId" for identifiers

### Don't ❌
- Use "agent", "crit", or "critter" (legacy terms)
- Use "UI" when referring to the Portal application (use "Portal" instead)
- Mix old and new terminology in the same context

---

## Questions or Additions?

This glossary is a living document. If you encounter unclear terminology or need to propose new terms, please:
1. Check this glossary first
2. Consult the team if unclear
3. Update this document with team consensus

**Maintainer:** Development Team
**Review Cycle:** Per sprint or as needed
