# Speciate Terminology Glossary

**Version:** 1.0.0
**Last Updated:** Sprint 6
**Status:** Active

---

## Purpose

This glossary defines the official terminology used throughout the Speciate codebase, documentation, and communication. Consistent terminology helps maintain code clarity and reduces confusion across the development team.

---

## Core Entities

### Crit / Crits
**Definition:** Autonomous creatures/agents in the simulation.

**Etymology:** Short for "critter" - a friendly, informal term for creatures.

**Usage:**
- Singular: "crit"
- Plural: "crits"
- **Examples:**
  - "Each crit has a unique ID"
  - "The simulation manages 1000 crits"
  - "Crit behavior is driven by steering forces"

**Previously:** agents, creatures, boids

**Code References:**
- `CritId` - Unique identifier component
- `CritTransform` - Position, velocity, and rotation snapshot
- `crit_systems.rs` - ECS systems file

---

### Portal
**Definition:** The web-based client application where players interact with the simulation.

**Usage:**
- Always capitalized as "Portal" when referring to the application
- **Examples:**
  - "The Portal renders crits at 60 FPS"
  - "Players connect to the simulation via the Portal"
  - "Portal receives updates from the Broadcaster"

**Previously:** UI, frontend

**Code References:**
- `apps/portal/` - Portal application directory
- `@simulation/portal` - NPM package name

---

## Technical Terms

### CritId
**Type:** Component (Rust), number (TypeScript)

**Definition:** Stable, unique identifier for each crit. Assigned at spawn time and never changes.

**Rust:** `CritId(u32)`
**TypeScript:** `number` (uint32 range)

**Usage:**
```rust
// Rust
CritId(42)

// TypeScript
const critId: number = 42;
```

---

### CritTransform
**Type:** Struct (Rust), Interface (TypeScript)

**Definition:** Complete state snapshot of a crit including position, velocity, and rotation.

**Fields:**
- `id` - Unique identifier (u32/number)
- `x`, `y` - World position (f32/number)
- `vx`, `vy` - Velocity in units/second (f32/number)
- `rotation` - Rotation in radians, 0 to 2π (f32/number)

**Usage:**
```rust
// Rust
CritTransform {
    id: 1,
    x: 45.23,
    y: 78.91,
    vx: 2.15,
    vy: -0.87,
    rotation: 1.57,
}
```

```typescript
// TypeScript
const crit: CritTransform = {
  id: 1,
  x: 45.23,
  y: 78.91,
  vx: 2.15,
  vy: -0.87,
  rotation: 1.57,
};
```

---

### Behavior Mode
**Type:** Enum

**Definition:** State machine state for crit behavior.

**Values:**
- `Wandering` - Random exploration (default)
- `Fleeing` - Escaping from danger
- `Feeding` - Consuming resources
- `Resting` - Energy restoration

**Usage:**
```rust
pub enum BehaviorMode {
    Wandering,
    Fleeing,
    Feeding,
    Resting,
}
```

---

## System Components

### Broadcaster
**Definition:** WebSocket service that streams simulation frames from NATS to connected Portal clients.

**Responsibility:** Real-time distribution of simulation state at ~20 Hz.

**Tech Stack:** Node.js, TypeScript, NATS.js, WebSocket

**Location:** `apps/broadcaster/`

---

### Simulation
**Definition:** Server-authoritative ECS simulation engine running at 20 Hz.

**Responsibility:** Single source of truth for all game state. Physics, crit behaviors, and state updates.

**Tech Stack:** Rust, Bevy ECS, async-nats, tokio

**Location:** `apps/simulation/`

---

## NATS Messaging

### Subject: `speciate.crits.transform`
**Definition:** NATS pub/sub subject for streaming crit state updates.

**Publisher:** Simulation (Rust)
**Subscribers:** Broadcaster (TypeScript)

**Format:** MessagePack binary serialization
**Frequency:** ~20 Hz (20 messages per second)

**Message Structure:**
```typescript
interface SimulationFrame {
  tick: number;              // Simulation tick counter
  timestamp: string;         // ISO 8601 timestamp (UTC)
  crits: CritTransform[];    // Array of crit states
}
```

---

## Steering Behaviors (Nature of Code)

### Wander
**Definition:** Smooth random exploration using steering forces. Projects a circle ahead of the crit and picks a random point on the circumference.

**Parameters:**
- `wander_distance` - How far ahead to project (default: 50.0)
- `wander_radius` - Circle size (default: 25.0)
- `angle_change` - Max angle delta per frame (default: 0.15 radians)

---

### Separation
**Definition:** Collision avoidance through mutual repulsion. Crits steer away from nearby neighbors.

**Parameters:**
- `separation_radius` - Distance at which separation engages (default: 15.0)
- `separation_force` - Strength of repulsion (default: 2.0)

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

**Use Case:** Efficient crit-to-crit interaction checks without O(n²) full-collection scanning.

**Cell Size:** 2-3x the typical interaction radius

---

## Legacy Terms (Deprecated)

| **Deprecated** | **Current** | **Changed In** |
|----------------|-------------|----------------|
| Agent/Agents   | Crit/Crits  | Sprint 6       |
| AgentId        | CritId      | Sprint 6       |
| AgentTransform | CritTransform | Sprint 6     |
| UI             | Portal      | Sprint 6       |
| `apps/ui/`     | `apps/portal/` | Sprint 6    |
| `agent_systems.rs` | `crit_systems.rs` | Sprint 6 |
| `speciate.agents.transform` | `speciate.crits.transform` | Sprint 6 |

---

## Communication Guidelines

### Do ✅
- Use "crit" for individual creatures
- Use "crits" for the plural
- Use "Portal" (capitalized) for the web application
- Use "CritTransform" for state snapshots
- Use "CritId" for identifiers

### Don't ❌
- Use "agent" or "creature" (legacy terms)
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
