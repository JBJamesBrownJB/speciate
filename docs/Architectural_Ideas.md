# Architectural Ideas

Architecture concepts and patterns for the Speciate simulation system.

---

## WebSocket Broadcasting (IMPLEMENTED ✅)

**Status:** Implemented via NATS streaming pipeline

**Original Idea:** Decouple WebSocket broadcast from simulation - have the simulation write current state to a buffer, then hand to separate component for broadcasting to many clients.

**Current Implementation:**
```
Simulation tick runs (60 Hz)
  ↓
State published to NATS (MessagePack binary)
  ↓
Broadcaster service consumes from NATS
  ↓
WebSocket broadcast to thousands of clients
```

**Benefits Achieved:**
- ✅ Simulation never blocks on network I/O
- ✅ Separate hardware/scaling for broadcaster vs simulation
- ✅ NATS provides pub/sub decoupling
- ✅ MessagePack reduces payload by 61% vs JSON

**Future Enhancement:** Horizontal broadcaster scaling with region-based routing (see below)

---

## Infinite World Architecture

**Goal:** Support unlimited world size while respecting NATS 1MB payload limit and enabling efficient coordinate encoding.

**Strategy:** World prefix + local coordinate reset per region.

### Core Concept

```
┌───────────────────────────────────────────────────────────┐
│ World "alpha" (infinite)                                  │
│                                                            │
│  Region 0.0      Region 1.0      Region 2.0      ...      │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐              │
│  │ Coords: │    │ Coords: │    │ Coords: │              │
│  │ 0-1000  │    │ 0-1000  │    │ 0-1000  │  (resets!)  │
│  │ 0-1000  │    │ 0-1000  │    │ 0-1000  │              │
│  └─────────┘    └─────────┘    └─────────┘              │
│                                                            │
│  Region 0.1      Region 1.1      Region 2.1               │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐              │
│  │ 0-1000  │    │ 0-1000  │    │ 0-1000  │              │
│  │ 0-1000  │    │ 0-1000  │    │ 0-1000  │              │
│  └─────────┘    └─────────┘    └─────────┘              │
└───────────────────────────────────────────────────────────┘

Each region publishes to: agents.alpha.region.{x}.{y}
Local coordinates: 0-1000 (i16 with 0.1 precision fits perfectly)
Global coordinates: (region_x × 1000) + local_x
```

### Multiple Game Instances

Different world prefixes enable separate game worlds:

```
agents.alpha.region.*       → Main world
agents.beta.region.*        → Secondary world
agents.survival_001.*       → Player-specific instance
agents.creative_mode.*      → Creative mode world
```

**Benefits:**
- Different rulesets per world prefix
- Player isolation (PvP vs PvE)
- Parallel game modes
- A/B testing new features
- Private instances

### Coordinate System

**Local Coordinates (per region):**
```rust
pub struct AgentTransformLocal {
    pub id: u32,
    pub x_local: i16,  // 0-1000 × 10 (0.1 precision) = 0-10000
    pub y_local: i16,
    pub vx: i16,       // velocity
    pub vy: i16,
    pub rotation: u16,
}

// Example: agent at local position (450.2, 873.0)
x_local: 4502  // 450.2 × 10
y_local: 8730  // 873.0 × 10

// Range: i16 = ±32768 → ±3276.8 units with 0.1 precision
// Perfect for 1000×1000 regions with safety margin
```

**Global Coordinates (client-side):**
```rust
fn local_to_global(
    world_prefix: &str,
    region_x: u16,
    region_y: u16,
    local_x: i16,
    local_y: i16,
    region_size: f32,  // 1000.0
) -> GlobalPosition {
    GlobalPosition {
        world: world_prefix.to_string(),
        x: (region_x as f32 × region_size) + (local_x as f32 / 10.0),
        y: (region_y as f32 × region_size) + (local_y as f32 / 10.0),
    }
}

// Example: region (5, 7), local (450.2, 873.0)
// Global X = (5 × 1000) + 450.2 = 5450.2
// Global Y = (7 × 1000) + 873.0 = 7873.0
```

### Agent Migration Between Regions

When agent moves from one region to another:

```rust
// Agent in region (5, 7) at local (980.5, 500.0)
// Moves to local (1020.3, 500.0) → exceeds region boundary!

// Migration process:
1. Detect boundary crossing (local_x > 1000 or < 0)
2. Calculate new region: (6, 7)
3. Reset local coords: x = 1020.3 - 1000 = 20.3
4. Unpublish from agents.alpha.region.5.7
5. Publish to agents.alpha.region.6.7 with reset coords

// Agent now at region (6, 7), local (20.3, 500.0)
// Global position unchanged: 6020.3 (6 × 1000 + 20.3)
```

**Implementation Considerations:**
- Agents near boundaries may need to publish to 2-4 adjacent regions
- Hysteresis to prevent rapid region switching
- Smooth handoff between broadcasters

### NATS Subject Hierarchy

```
agents.{world_prefix}.region.{region_x}.{region_y}

Examples:
agents.alpha.region.0.0      → World "alpha", region (0,0)
agents.alpha.region.5.7      → World "alpha", region (5,7)
agents.beta.region.2.3       → World "beta", region (2,3)
agents.survival_001.region.10.15

Subscribers:
- Broadcasters subscribe to specific regions
- Frontend connects to broadcasters covering viewport
- Monitoring tools can wildcard: agents.alpha.region.*
```

### Player Discovery & Viewport Mapping

**Frontend viewport calculation:**
```javascript
// Player viewport: 5000-6000 x 7000-8000 (global coords)
// Region size: 1000
const viewportRegions = calculateViewportRegions(
  playerViewport,
  regionSize
);

// Result: regions (5,7), (5,8), (6,7), (6,8)
// Connect to broadcasters serving these regions
const broadcasters = [
  "ws://broadcaster-a.example.com",  // Handles regions 5.7, 5.8
  "ws://broadcaster-b.example.com",  // Handles regions 6.7, 6.8
];

// Subscribe only to visible regions
broadcasters.forEach(ws => {
  ws.subscribe("agents.alpha.region.5.7");
  ws.subscribe("agents.alpha.region.5.8");
  // etc...
});
```

### Comparison to Minecraft Architecture

Similar pattern to Minecraft's chunk system:

| Aspect | Minecraft | Speciate Infinite Worlds |
|--------|-----------|--------------------------|
| **Global coords** | Block coords ±30M | Unlimited (region prefix) |
| **Local coords** | 0-16 per chunk | 0-1000 per region |
| **Encoding** | Relative to chunk | i16 with 0.1 precision |
| **Network** | Send visible chunks | Publish to region subjects |
| **Migration** | Chunk loading/unloading | NATS subscribe/unsubscribe |
| **Scaling** | Single server per world | Horizontal broadcaster scaling |

### Scalability Analysis

**Single Region:**
- Max agents: ~33K (at 1MB NATS limit with i16 encoding)
- Area: 1000×1000 = 1M units²
- Density: 33 agents per 1000 units²

**100 Regions (10×10 grid):**
- Max agents: 3.3M
- Area: 10,000×10,000 = 100M units²
- 10 broadcaster instances (10 regions each)
- 5,000-10,000 concurrent players

**Infinite Regions:**
- Max agents: Unlimited (add regions)
- Area: Unlimited
- Broadcasters: Scale horizontally
- Players: Limited only by discovery service capacity

### Implementation Phases

**Phase 1: Single Region (Current)**
- Hardcoded region (0, 0)
- No world prefix
- Global coords = local coords
- Single broadcaster
- **Status:** Foundation in place (NATS, MessagePack)

**Phase 2: Multi-Region, Single World**
- Implement region grid (10×10)
- Add world prefix: "alpha"
- Agent migration between regions
- Multiple broadcasters (region-based routing)
- **Complexity:** 2-3 weeks

**Phase 3: Multiple Worlds**
- Support multiple world prefixes
- World selection UI
- Cross-world player tracking
- World-specific rulesets
- **Complexity:** 1-2 weeks (after Phase 2)

**Phase 4: Truly Infinite**
- Dynamic region creation
- Region persistence (database)
- Procedural generation at region boundaries
- Region cleanup (despawn empty regions)
- **Complexity:** 4-6 weeks

### Open Questions

1. **Discovery Service:**
   - How do clients discover which broadcaster serves which regions?
   - Static config vs dynamic service registry?
   - Load balancing when multiple broadcasters serve same region?

2. **Agent Migration:**
   - How to handle agents that move very fast (cross multiple regions per frame)?
   - Publish to 4 adjacent regions for boundary agents?
   - Hysteresis threshold to prevent region flapping?

3. **World Persistence:**
   - Do regions persist when all players leave?
   - How to save/restore region state?
   - Procedural generation vs player-created content?

4. **Broadcaster Routing:**
   - Static assignment: broadcaster A always handles regions 0-9?
   - Dynamic assignment: least-loaded broadcaster takes new region?
   - Failover: what happens when broadcaster crashes?

---

## Border Coherence & Ghost Entities

**Goal:** Enable agents in different simulations to see and react to each other across region boundaries.

### The Problem

When the world is partitioned into separate simulations, agents near boundaries become isolated:

```
┌─────────────────┐ ┌─────────────────┐
│ Simulation A    │ │ Simulation B    │
│ (region 0,0)    │ │ (region 1,0)    │
│                 │ │                 │
│    Agent 1 →    |→| ← Agent 2       │
│    (x=950)      │ │   (x=50)        │
│                 │ │                 │
└─────────────────┘ └─────────────────┘
         ↑ Border at x=1000 ↑

Problem: Agents are only 100 units apart, but:
❌ Agent 1 can't see Agent 2 (different simulation)
❌ Agent 2 can't see Agent 1 (different simulation)
❌ Can't flee from predators across boundaries
❌ Can't flock/swarm across regions
❌ Player sees "invisible wall" behavior
```

**Impact:** Breaks immersion and simulation realism at every region boundary.

---

### Solution: Ghost Entity Pattern

**Concept:** Each simulation publishes "ghost entities" for agents near borders. Neighboring simulations receive these ghosts as **read-only** agents that local agents can see and react to.

```
┌─────────────────────────────────────────────────┐
│ Simulation A (region 0,0)                       │
│                                                  │
│   Full Zone (0-950)     Border Zone (950-1000)  │
│   ┌───────────────┐     ┌────────┐             │
│   │ Local agents  │     │ Agent 1│  (x=950)    │
│   │ Full physics  │     │ Ghost? │             │
│   └───────────────┘     └────────┘             │
│                              │                   │
│   Publishes to NATS:         │                  │
│   ghosts.from.0.0.to.1.0 ────┘                  │
└──────────────────────────────────────────────────┘
                                │
                    NATS (ghost stream)
                                │
                                ▼
┌──────────────────────────────────────────────────┐
│ Simulation B (region 1,0)                        │
│                                                   │
│   Border Zone (0-50)      Full Zone (50-1000)   │
│   ┌────────┐              ┌───────────────┐     │
│   │ Ghost 1│ (read-only)  │ Agent 2       │     │
│   │ (x=950)│ ◄────────────│ Can SEE Ghost!│     │
│   └────────┘              └───────────────┘     │
│       ↑                                          │
│   Subscribes to:                                 │
│   ghosts.from.0.0.to.1.0                        │
└──────────────────────────────────────────────────┘

Now: Agent 2 can see Agent 1 (as ghost) and react!
✅ Flee from predators across boundaries
✅ Flock/swarm seamlessly
✅ No invisible walls
```

---

### NATS Subject Design

**Ghost Streams** (border agents published at 10 Hz):
```
ghosts.from.{src_region_x}.{src_region_y}.to.{dest_region_x}.{dest_region_y}

Examples:
ghosts.from.0.0.to.1.0    → Sim (0,0) sends eastward ghosts to Sim (1,0)
ghosts.from.0.0.to.0.1    → Sim (0,0) sends northward ghosts to Sim (0,1)
ghosts.from.5.7.to.6.7    → Sim (5,7) sends eastward ghosts to Sim (6,7)
ghosts.from.5.7.to.4.7    → Sim (5,7) sends westward ghosts to Sim (4,7)
```

**8-Neighbor Subscriptions:**

Each simulation subscribes to ghosts FROM all 8 neighbors:
```
Simulation at region (5, 7) subscribes to:
- ghosts.from.4.7.to.5.7  (west)
- ghosts.from.6.7.to.5.7  (east)
- ghosts.from.5.6.to.5.7  (south)
- ghosts.from.5.8.to.5.7  (north)
- ghosts.from.4.6.to.5.7  (southwest diagonal)
- ghosts.from.6.6.to.5.7  (southeast diagonal)
- ghosts.from.4.8.to.5.7  (northwest diagonal)
- ghosts.from.6.8.to.5.7  (northeast diagonal)
```

---

### Border Zone Implementation

**Key Concepts:**

**1. Border Zone Definition:**
- Define a threshold distance from region edges (e.g., 50 units)
- Agents within this threshold are "border agents"
- Corner agents may be in multiple border zones simultaneously (publish to 2-4 neighbors)
- Configuration: region size (1000×1000), border threshold (50 units), update rate (10 Hz)

**2. Ghost Entity State:**
- Minimal read-only state: id, position, velocity, rotation, source region
- Ghost entities in local simulation:
  - ✅ CAN be seen by local agents (for AI vision/decisions)
  - ✅ CAN collide with local agents (simplified collision)
  - ❌ CANNOT be modified (read-only)
  - ❌ CANNOT be damaged/interacted with (only source simulation has authority)

**3. Publishing Border Agents:**
- System runs at 10 Hz (not 60 Hz) to reduce overhead
- Find all agents in border zones each update
- Group agents by destination neighbor
- Batch publish to NATS subjects per neighbor
- Serialize using MessagePack for efficiency

**4. Receiving Ghost Entities:**
- Subscribe to ghost streams from all 8 neighbors at startup
- Spawn async tasks to receive ghost updates
- Despawn stale ghosts from each source region
- Spawn fresh ghost entities with updated state
- Mark entities with Ghost component to distinguish from local agents

---

### Agent Migration Protocol

When an agent crosses from one simulation to another, full ownership must transfer:

**Migration Subject Pattern:**
```
migration.{agent_id}.from.{src_region_x}.{src_region_y}.to.{dest_region_x}.{dest_region_y}

Example:
migration.42.from.0.0.to.1.0
```

**Migration Flow (6 Steps):**

1. **Source simulation detects boundary crossing**
   - Agent position exceeds region bounds (e.g., x > 1000)
   - Serialize full agent state (all ECS components: DNA, health, etc.)
   - Reset local coordinates for destination region

2. **Publish migration message to NATS**
   - Subject: migration.{agent_id}.from.{src}.to.{dest}
   - Payload: Full agent state + new local coordinates

3. **Mark agent as "migrating" in source simulation**
   - Don't simulate the agent (freeze physics)
   - Continue publishing as ghost (visible to neighbors until transfer completes)

4. **Destination simulation receives migration**
   - Spawn agent entity with full state
   - Apply local coordinates in destination region

5. **Destination sends ACK to source**
   - Subject: migration_ack.{agent_id}.from.{dest}
   - Confirms successful spawn

6. **Source receives ACK and despawns agent**
   - Remove agent entity from source simulation
   - Migration complete (agent now owned by destination)

**Edge Cases:**

| Scenario | Solution |
|----------|----------|
| Agent migrates BACK before ACK | Use migration ID (timestamp + agent_id), ignore stale |
| Dest simulation crashes during migration | Timeout (5 sec), respawn in source sim |
| Both sims think they own agent | Consensus: lowest region ID wins, other despawns |
| Agent moves very fast (crosses 2+ regions/frame) | Multi-hop migration (chain of migrations) |

---

### Performance Analysis

**Ghost Update Bandwidth:**
```
Per simulation:
- 8 neighbors
- 10 Hz update rate
- Border zone: 50 units (5% of 1000×1000 region)
- 100K agents × 5% = 5K border agents
- Payload: 5K × 20 bytes/agent = 100 KB per message

Total bandwidth:
8 neighbors × 10 Hz × 100 KB = 8 MB/s per simulation
```

**Optimizations:**

1. **Reduce Update Rate:** 10 Hz instead of 60 Hz → **6x reduction**
2. **Delta Updates:** Only publish changed ghosts → **90% reduction**
3. **Adaptive Border Zone:** Narrow zone when sparse → **50% reduction**

**Optimized: ~800 KB/s per simulation** (acceptable!)

**Migration Bandwidth:**
```
Rare events:
- ~1% agents migrate per second (fast-moving agents)
- 100K × 1% = 1K migrations/sec
- Full state: ~500 bytes/agent (all components)
- 1K × 500 bytes = 500 KB/s

Total (ghosts + migrations): ~1.3 MB/s per simulation
```

---

### Comparison to Industry Architectures

**Eve Online (Single-Shard MMO):**
- All players in one universe
- Dynamic load balancing (move solar systems between nodes)
- "Reinforced nodes" for massive battles
- **Speciate approach:** Simpler! Static region assignment avoids complex rebalancing

**World of Warcraft (Realm Sharding):**
- Separate realms, historically no cross-realm interaction
- Modern: Dynamic sharding with "phasing" (similar to ghosts!)
- Cross-realm zones use ghost-like entities
- **Speciate approach:** Very similar to WoW's cross-realm zones

**Minecraft (Multi-Server):**
- Separate servers, no agent interaction across servers
- BungeeCord: Teleport players, but chunks don't sync
- No ghost entities (hard boundary at server edge)
- **Speciate approach:** Superior! Ghost entities enable true seamless world

**Second Life (Region-Based):**
- World divided into regions (256m × 256m)
- Agent "visibility" across borders via neighbor queries
- Migration protocol similar to this design
- **Speciate approach:** Nearly identical pattern (proven successful!)

---

## Hexagonal vs Square Regions: Architecture Analysis

**Question:** Should Speciate use hexagonal regions instead of square regions?

### Overview

**Hexagonal grids** (like Uber's H3 system) offer elegant mathematical properties:
- All 6 neighbors equidistant (no diagonal distance problem)
- Better approximation of circular coverage
- 30% less perimeter for same area (reduced edge effects)
- Used successfully in turn-based strategy games (Civilization V/VI)

However, for Speciate's real-time A-Life simulation architecture, **the trade-offs don't justify the complexity.**

---

### Key Trade-offs Comparison

| Aspect | Square Regions | Hexagonal Regions | Winner |
|--------|----------------|-------------------|--------|
| **Neighbors** | 8 (4 edge + 4 diagonal) | 6 (all equidistant) | Hexagonal (25% fewer) |
| **Distance Uniformity** | Two classes: 1.0 and 1.414 | Single class: all equal | Hexagonal |
| **Coordinate System** | Simple (x, y) | Axial/Cube coords | Square (intuitive) |
| **Implementation Complexity** | ~500-1000 LOC | ~1500-2500 LOC | Square (2-3x simpler) |
| **Border Detection** | 4 simple comparisons | 6 distance-to-line calculations | Square |
| **Screen Alignment** | Perfect (rectangular display) | Complex (viewport transform) | Square |
| **Cache Performance** | Dense 2D array possible | Sparse HashMap required | Square |
| **CPU Performance** | ~5-10 ns neighbor lookup | ~15-30 ns neighbor lookup | Square (2-3x faster) |
| **Ghost Bandwidth** | 8 neighbors × 10 Hz | 6 neighbors × 10 Hz | Hexagonal (25% less) |

---

### Benefits of Hexagons

**1. Fewer Neighbors (6 vs 8):**
- 25% reduction in NATS subscriptions
- Simpler neighbor graph for debugging
- ~20% reduction in ghost entity bandwidth

**2. Uniform Distances:**
- All neighbors equidistant → no diagonal bias
- More consistent ghost visibility range
- Eliminates pathfinding exploits (diagonal movement faster)

**3. Better Border Shape:**
- Hexagonal borders more circular → fewer corner cases
- Reduces likelihood of agents in multiple border zones

**4. Industry Proven:**
- Uber H3: Production-grade hexagonal spatial indexing
- Strategy games: Civilization, Endless Legend use hexagons successfully

---

### Drawbacks of Hexagons

**1. Implementation Complexity (2-3x more code):**
- Requires axial/cube coordinate systems (not simple x, y)
- Complex neighbor finding algorithms
- Harder viewport-to-hex conversions (rectangular screen → hexagonal world)
- Custom NATS subject naming (can't use simple region.{x}.{y})

**2. Performance Overhead:**
- Coordinate conversions add CPU cycles (~2-3x slower neighbor lookups)
- Sparse storage required (HashMap) → worse cache locality
- No dense 2D array optimization

**3. Developer Ergonomics:**
- Steeper learning curve (axial coordinates non-intuitive)
- Harder debugging (can't just print "x=5, y=7")
- Requires specialized visualization tools

**4. Player UX:**
- Rectangular displays don't align with hexagonal regions
- Minimap would show hexagonal boundaries (unusual for simulation games)
- Coordinates not intuitive to display to players

---

### Recommendation: **Retain Square Regions**

**Rationale:**

1. **Genre Mismatch:** Hexagons excel in **turn-based strategy games** where tactical fairness and balanced movement are critical. Speciate is a **real-time A-Life simulation** where agents use continuous velocity vectors, not grid-based movement. The primary hexagon advantage (fair tactical movement) doesn't apply.

2. **Complexity Not Justified:** 25% bandwidth savings (~2 MB/s → 1.6 MB/s) doesn't justify 2-3x implementation complexity. The existing square grid architecture already handles 100K+ agents at 20 Hz.

3. **Simpler = Better:** Square grids are intuitive, fast, and maintainable. Junior developers can contribute immediately without learning specialized coordinate systems.

4. **Screen Alignment:** Rectangular displays, rectangular UI, rectangular viewports → square regions feel natural.

---

### When Would Hexagons Be Worth It?

Hexagons **would** be justified if Speciate had:

1. **Turn-Based Tactical Combat:** Players control units on a grid (like XCOM, Fire Emblem)
2. **Grid-Based Movement:** Units move in discrete steps (not continuous velocity)
3. **Strategy/Territory Mechanics:** Players claim hexagonal territories
4. **No Real-Time Constraints:** Turn-based allows expensive coordinate math

**None of these apply to Speciate.**

---

### Alternative: Hybrid Approach

If organic/circular visual aesthetics are desired:

**Keep square grid for simulation**, but:
- Render hexagonal overlay for visual effect (client-side only)
- Use Voronoi diagrams for organic territory boundaries (decoupled from simulation grid)
- Implement circular influence radii (not tied to grid shape)

This achieves visual benefits without architectural complexity.

---

### References

- **Uber H3:** https://h3geo.org/ (hexagonal hierarchical spatial index)
- **Red Blob Games:** https://www.redblobgames.com/grids/hexagons/ (definitive hexagonal grid guide)
- **Industry Use:** Civilization V/VI, Endless Legend (strategy games benefit from hexagons)

---

## Inbound Commands (Future)

**Question:** How do player commands reach the simulation?

**Options:**

1. **NATS Request/Reply:**
   - Player action → Frontend → NATS request → Simulation
   - Simulation processes command, publishes state update
   - **Pro:** Uses existing NATS infrastructure
   - **Con:** Adds latency, may slow simulation tick

2. **HTTP API to Simulation:**
   - Direct HTTP endpoint on simulation server
   - Player action → Frontend → HTTP POST → Simulation
   - **Pro:** Simple, low latency
   - **Con:** Couples frontend to simulation instances

3. **Command Queue (NATS Subject):**
   - Simulation subscribes to: `commands.{world_prefix}.region.{x}.{y}`
   - Processes command queue each tick
   - **Pro:** Decoupled, can batch commands
   - **Con:** Needs command validation/authentication

**Recommended:** Option 3 (Command Queue via NATS) for consistency with architecture.

---

## Notes

- All coordinate examples assume 1000×1000 region size (configurable)
- i16 with 0.1 precision chosen for balance of range vs accuracy
- World prefixes enable multi-tenancy (separate game instances per prefix)
- See `/workspace/docs/Performance_Ideas.md` for bandwidth optimizations that synergize with this architecture
