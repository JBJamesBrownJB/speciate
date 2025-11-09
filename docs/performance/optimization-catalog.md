# Performance Optimization Ideas

**Status**: Mix of Implemented and Planned
**Last Updated**: 2025-11-09
**Related**: [instrumentation-plan.md](./instrumentation-plan.md), [nats-optimizations.md](./nats-optimizations.md)

Catalog of performance optimization strategies for the Speciate simulation system.

---

## Current State

**Scale:** 100,000 agents @ 60 Hz
**Serialization:** MessagePack binary format (61% smaller than JSON)
**Payload Size:** 2.8-3.5 MB per frame (100K agents)
**Bandwidth:** 168-210 MB/s @ 60Hz
**NATS Limit:** 1 MB default (payloads currently exceed this)
**Architecture:** Rust Simulation → NATS → Node.js Broadcaster → WebSocket Clients

**Note**: For detailed NATS publisher implementation (non-blocking, buffer pooling, backoff), see [nats-optimizations.md](./nats-optimizations.md)

---

## Implemented Optimizations

### Frontend Viewport Culling (Grid Rendering)

**Status:** ✅ Implemented

**Problem:** Rendering full-world grid at all zoom levels was causing performance issues.

**Solution:** Viewport-based culling for grid rendering.

**Implementation Details:**
- Grid only renders lines visible in current viewport bounds
- Grid visibility threshold: Only renders when zoom >= 20 px/m
- Fixed 1m grid spacing for consistent spatial reference
- Uses Camera and Viewport classes to calculate visible world bounds
- Padding added (2× grid spacing) to ensure smooth rendering at viewport edges

**Performance Impact:**
- Rendering cost scales with viewport size, not world size
- At zoomed-out views (< 20 px/m), grid completely disabled
- Eliminates rendering of 2000km × 2000km grid lines when only 100m × 100m viewport visible

**Files:**
- `apps/portal/src/rendering/GridRenderer.ts` - Viewport culling implementation
- `apps/portal/src/domain/Camera.ts` - Camera coordinate system
- `apps/portal/src/domain/Viewport.ts` - Visible bounds calculation
- Tests: 74 tests covering grid rendering, camera, and viewport

**Future Work:**
- Apply same viewport culling strategy to creature sprites when scale increases
- Spatial sharding (see below) will extend this concept to network layer

---

## Bandwidth Reduction Ideas

### Spatial Sharding (95-98% reduction)

**Problem:** Clients don't need all agents, only those in viewport.

**Solution:** Partition world into spatial grid, publish to separate NATS subjects per region.

```
Simulation partitions world into grid cells (e.g., 32×32)
  ↓
Publishes to: agents.region.{x}.{y}
  ↓
Broadcaster/Client subscribes only to viewport regions
  ↓
Result: 10 MB/frame → 200-500 KB/frame per client
```

**Implementation:**
- Grid size: 32×32 or 64×64 cells (configurable based on agent density)
- Each cell publishes to subject: `agents.region.{x}.{y}`
- Border agents: Publish to multiple cells if near boundary
- Max agents per cell must stay under NATS 1MB limit (~33K agents @ 30 bytes/agent)

**Benefits:**
- 95-98% bandwidth reduction for typical viewports
- Foundation for horizontal broadcaster scaling
- Enables viewport culling
- Natural load distribution across regions

**Considerations:**
- Boundary handling (agents near cell edges)
- Dynamic agent density balancing
- Client needs to calculate viewport → region mapping

---

### Schema-Driven Array Serialization

  Concept: Use a JSON Schema contract file (like .proto) to define field positions. Then MessagePack can send compact
  arrays [123, "...", [...]] instead of verbose objects {"tick": 123, ...}. The schema tells the broadcaster "position 0 =
   tick, position 1 = timestamp".

  Benefit: ~20% smaller payloads while maintaining type safety through schema validation.

### Delta Updates (90-95% reduction)

**Problem:** Most agents change minimally frame-to-frame.

**Solution:** Only publish agents with "network-significant" changes.

```rust
// Add ECS component to track network state
#[derive(Component)]
pub struct NetworkState {
    pub last_published_pos: (f32, f32),
    pub last_published_rotation: f32,
    pub dirty: bool,
}

// Mark agents dirty if moved >0.5 units or rotated >5 degrees
fn mark_network_dirty_system(
    mut query: Query<(&Position, &Rotation, &mut NetworkState)>
) {
    for (pos, rot, mut net_state) in query.iter_mut() {
        let pos_delta = distance(pos, &net_state.last_published_pos);
        let rot_delta = (rot.radians - net_state.last_published_rotation).abs();

        if pos_delta > 0.5 || rot_delta > 0.087 {
            net_state.dirty = true;
        }
    }
}
```

**Expected Results:**
- Typical movement: 5-10% agents dirty per frame
- Payload: 2.8 MB → 280-560 KB (90-95% reduction)

**Requirements:**
- Client-side interpolation for smooth motion
- Initial frame must include all agents (baseline)
- Periodic full-frame refresh to prevent drift

**Synergizes With:** Spatial sharding (apply deltas per region)

---

### Frame Rate Reduction (66% reduction)

**Problem:** 60 Hz network updates may be overkill.

**Solution:** Decouple simulation rate from network rate.

```
Simulation Loop: 60 Hz (physics precision)
  ↓
Change Detection (mark dirty agents)
  ↓
Network Loop: 20 Hz (client updates)
```

**Benefits:**
- 66% bandwidth reduction (60 Hz → 20 Hz)
- Client-side interpolation smooths 20 Hz updates
- Maintains 60 Hz physics precision

**Trade-offs:**
- Requires client interpolation logic
- 20-50ms additional latency
- May feel less responsive for fast-paced interactions

---

### Quaternion Compression (5-25% reduction)

**Problem:** Rotation as f32 radians (4 bytes) is overkill for 2D.

**2D Optimization:**
```rust
pub rotation: u16  // 2 bytes, quantized to 0-65535 (0.0055° precision)
// Encode: (radians / TAU * 65535.0) as u16
// Decode: (value as f32 / 65535.0) * TAU
```
Savings: 2 bytes/agent (20KB @ 10K agents, 200KB @ 100K agents)

**3D Future (smallest-3 quaternion):**
```rust
pub rotation: [i16; 3]  // 6 bytes vs 16 bytes (4× f32)
// Omit largest component, reconstruct on decode
// Precision: ~0.001 degrees
```
Savings: 10 bytes/agent if moving to 3D (1MB @ 100K agents)

**Impact:** 5% for 2D, 25% for 3D

---

### Fixed-Point Encoding (30-40% reduction)

**Problem:** Full f32 precision (4 bytes, ~7 decimal places) is visual overkill.

**Example data:** `x: 168.44235229492188` - do we really need that precision?

**Solution:** Use i16/u16 fixed-point encoding with reduced precision.

```rust
// Current: f32 (4 bytes each)
pub x: f32, y: f32, vx: f32, vy: f32, rotation: f32

// Optimized: i16/u16 fixed-point (2-3 bytes each)
pub struct AgentTransformCompressed {
    pub id: u32,

    // Position: i16 with 0.1 precision (LOCAL coordinates)
    // Perfect for 1000×1000 regions with infinite world support
    pub x_local: i16,  // actual_x * 10.0, range: ±3276.8 units
    pub y_local: i16,

    // Velocity: i16 with 0.1 precision
    pub vx_fixed: i16,  // actual_vx * 10.0, range: ±3276.8 units/sec
    pub vy_fixed: i16,

    // Rotation: u16 quantized (see Quaternion Compression above)
    pub rotation_fixed: u16,  // (radians / TAU) * 65535
}

// Encode
fn compress(agent: &AgentTransform) -> AgentTransformCompressed {
    AgentTransformCompressed {
        id: agent.id,
        x_local: (agent.x * 10.0) as i16,
        y_local: (agent.y * 10.0) as i16,
        vx_fixed: (agent.vx * 10.0) as i16,
        vy_fixed: (agent.vy * 10.0) as i16,
        rotation_fixed: ((agent.rotation / TAU) * 65535.0) as u16,
    }
}

// Decode (broadcaster/client)
fn decompress(c: &AgentTransformCompressed, region_x: u16, region_y: u16, region_size: f32) -> GlobalPosition {
    GlobalPosition {
        // Convert local coords to global: (region × size) + local
        x: (region_x as f32 * region_size) + (c.x_local as f32 / 10.0),
        y: (region_y as f32 * region_size) + (c.y_local as f32 / 10.0),
        vx: c.vx_fixed as f32 / 10.0,
        vy: c.vy_fixed as f32 / 10.0,
        rotation: (c.rotation_fixed as f32 / 65535.0) * TAU,
    }
}
```

**Visual Impact Assessment:**

For 1000×1000 viewport on 1920×1080 screen (1 game unit = 1.92 pixels):
- **Position 0.1 precision:** 0.19 pixels error (sub-pixel, imperceptible)
- **Velocity 0.1 at 60Hz:** ±0.096 pixels/frame (completely smooth)
- **Rotation 0.0055°:** 0.0096 pixels at 100px radius (perfect)

**Verdict:** Visually identical to full f32 precision.

**Size Savings:**
- Current: ~30 bytes/agent (5 fields × 5 bytes + overhead)
- Compressed: ~20 bytes/agent (1×5 + 5×3 bytes)
- **Reduction: 33% per agent**
- **100K agents:** 3.0 MB → 2.0 MB (saves 1 MB/frame)

**Infinite World Support:**

Using **local coordinates + region prefixes** solves encoding constraints:

```rust
// Agents use LOCAL coordinates within their region (0-1000)
// i16 with 0.1 precision = ±3276 range (perfect for 1000×1000 regions)
pub x_local: i16  // 0-1000 local, fits easily in ±3276 range

// Global position: region prefix + local coords
// Example: agents.alpha.region.5.7 → local coords 0-1000
//          agents.alpha.region.5.8 → local coords 0-1000 (resets!)

// Infinite worlds = infinite regions, not infinite coordinate values
```

**Benefits:**
- ✅ Each region has small local coords (never huge numbers)
- ✅ i16 with 0.1 precision perfect for 1000×1000 regions
- ✅ Infinite worlds via region prefixes (no coordinate overflow)
- ✅ World prefix enables multiple game instances: "alpha", "beta", "survival_001"

**See Also:** Horizontal Broadcaster Scaling (region-based architecture)

**Testing Strategy:**
1. **Easy first step:** Add float rounding before encoding
   ```rust
   x_rounded = (x * 100.0).round() / 100.0  // Simulate 0.01 precision
   ```
   Toggle on/off to check if anyone notices visually
2. **If imperceptible:** Implement proper i16/u16 encoding
3. **Test with actual gameplay** before committing to schema change

**Combined Impact:**
- Fixed-point (33%) + Spatial sharding (95%) + Deltas (90%) = **97-99% total reduction**
- Example: 100K agents @ 60Hz
  - Baseline: 180 MB/s
  - With all optimizations: 3-6 MB/s per client

**Synergizes With:**
- Spatial sharding (natural when combined with cell-relative coords)
- Delta updates (fewer bytes per changed agent)
- MessagePack (better compression of smaller integers)

---

### Cell-Relative Coordinates (15-20% reduction)

**Problem:** Absolute coordinates require f32 (4 bytes) for world-scale precision.

**Solution:** With spatial sharding, encode positions relative to cell origin.

```rust
// Instead of:
pub x: f32  // 4 bytes, world coordinates 0-10000
pub y: f32  // 4 bytes

// Use:
pub cell_x: u8    // 1 byte, cell index 0-255
pub cell_y: u8    // 1 byte
pub local_x: u16  // 2 bytes, position within cell 0-65535
pub local_y: u16  // 2 bytes
// Total: 6 bytes instead of 8 bytes (25% reduction for position)
```

**Requirements:**
- Spatial sharding must be implemented first
- Cell size ~256 units provides 0.004 unit precision

**Impact:** 15-20% payload reduction (position is largest field)

---

## Scaling Architecture Ideas

### Horizontal Broadcaster Scaling (Geospatial Sharding)

**Problem:** Single broadcaster bottleneck. NATS 1MB limit constrains agent density.

**Solution:** Each broadcaster instance specializes in specific geographic regions. World prefixes enable infinite worlds and multiple game instances.

```
┌──────────────────────────────────────────────────────┐
│ Simulation (Rust)                                    │
│ - Partitions world into regions (e.g., 1000×1000)   │
│ - Uses LOCAL coordinates (0-1000) per region        │
│ - Publishes to: agents.{prefix}.region.{x}.{y}      │
│ - Enforces max agents/region (~33K @ 1MB limit)     │
└───────┬──────────┬──────────┬────────────────────────┘
        │          │          │
        ▼          ▼          ▼
     NATS Subjects (with world prefixes):
  agents.alpha.region.0.0  .alpha.region.0.1  .alpha.region.1.0
  agents.beta.region.5.7   .beta.region.5.8   (different world!)
  agents.survival_001.region.2.3              (another instance)
        │         │         │
        ▼         ▼         ▼
┌──────────┐ ┌──────────┐ ┌──────────┐
│Broadcast │ │Broadcast │ │Broadcast │
│   A      │ │   B      │ │   C      │
│(0-3)     │ │(4-7)     │ │(8-11)    │
│Port 3001 │ │Port 3002 │ │Port 3003 │
└────┬─────┘ └────┬─────┘ └────┬─────┘
     │            │            │
     └────────────┴────────────┘
                  │
        ┌─────────▼──────────┐
        │ Frontend Clients   │
        │ - Calculate which  │
        │   regions visible  │
        │ - Connect to       │
        │   relevant         │
        │   broadcasters     │
        └────────────────────┘
```

**Key Features:**
- **World Prefixes:** Enable multiple game instances ("alpha", "beta", "survival_001")
- **Local Coordinates:** Each region uses coords 0-1000 (resets per region)
- **Infinite Worlds:** Add more regions infinitely, coordinates stay small
- **Broadcaster Specialization:** Each instance handles specific regions
- **Agent Density Awareness:** Max ~33K agents/region to fit 1MB NATS limit
- **Dynamic Client Routing:** Frontend connects to broadcasters based on viewport
- **Horizontal Scaling:** Add more broadcasters for more regions

**Coordinate System:**
```rust
// Global position = (region_x × region_size) + local_x
// Example: region 5.7, local 450.2
// Global X = (5 × 1000) + 450.2 = 5450.2

// Agent in agents.alpha.region.5.7:
pub x_local: i16 = 4502  // 450.2 × 10 (0.1 precision)
pub y_local: i16 = 8730  // 873.0 × 10

// Agent in agents.alpha.region.5.8:
pub x_local: i16 = 1250  // 125.0 × 10 (coordinates RESET!)
pub y_local: i16 = 2340  // 234.0 × 10

// Same local coord space (0-1000), different regions
// Infinite world = add regions, not bigger numbers!
```

**Benefits:**
- ✅ **Infinite worlds:** Unlimited regions, coordinates never overflow
- ✅ **Multiple instances:** Different world prefixes = separate game worlds
- ✅ **Respects 1MB NATS limit:** Local coords (i16) fit perfectly in 1MB
- ✅ **Horizontal scaling:** Add broadcasters, not bigger messages
- ✅ **Bandwidth efficiency:** Each broadcaster only gets relevant data (90% reduction)
- ✅ **Client efficiency:** Only connects to visible regions
- ✅ **Fault tolerance:** One broadcaster down = one region offline (not whole world)
- ✅ **Load distribution:** Popular regions get dedicated broadcasters
- ✅ **Coordinate encoding:** i16 with 0.1 precision perfect for 1000×1000 regions

**Capacity Example:**
- 100 regions, 10 broadcaster instances (10 regions each)
- Each broadcaster: 50-100 WebSocket clients
- Total capacity: 5,000-10,000 concurrent players

**Implementation Requirements:**
- **Simulation:**
  - Partition world into regions with local coordinates (0-1000)
  - Enforce max agent density per region (~33K agents @ 1MB limit)
  - Publish to: `agents.{world_prefix}.region.{x}.{y}`
  - Handle agent migration between regions (local coord reset)
- **Broadcaster:**
  - Subscribe to specific world prefix + region subjects
  - Convert local coords to global for frontend: `(region_x × size) + local_x`
- **Frontend:**
  - Discovery service: map `{world_prefix}.{region_x}.{region_y}` → broadcaster URLs
  - Calculate viewport → region mapping
  - Manage multiple WebSocket connections (one per visible region)
  - Handle viewport spanning multiple regions
  - Convert global coords back to local for display/interaction

**Trade-offs:**
- Complexity: Clients manage multiple connections
- Discovery: Need broadcaster URL mapping service
- Coordination: Ensure agent density limits per region

---

### Corner Barrier Optimization (50% Ghost Bandwidth Reduction)

**Problem:** With region-based partitioning, each simulation must maintain ghost entities from all 8 neighbors (N, S, E, W, NE, NW, SE, SW) for border coherence. This creates substantial NATS subscription and bandwidth overhead.

**Solution:** Strategically place impassable terrain (mountains, rocks, canyons) at region corners, eliminating the need for diagonal neighbor ghost subscriptions.

```
Current (8 neighbors):              Optimized (4 neighbors):
┌─────┬─────┬─────┐                ┌─────┬─────┬─────┐
│ NW  │  N  │ NE  │                │ XXX │  N  │ XXX │
├─────┼─────┼─────┤                ├─────┼─────┼─────┤
│  W  │THIS │  E  │                │  W  │THIS │  E  │
├─────┼─────┼─────┤                ├─────┼─────┼─────┤
│ SW  │  S  │ SE  │                │ XXX │  S  │ XXX │
└─────┴─────┴─────┘                └─────┴─────┴─────┘
8 NATS subscriptions               4 NATS subscriptions
```

**Ghost Bandwidth Impact:**
```
Current:
- 8 neighbors × 10 Hz × 100 KB/message = 8 MB/s per simulation
- With optimizations: ~800 KB/s per simulation

With Corner Barriers:
- 4 neighbors × 10 Hz × 100 KB/message = 4 MB/s per simulation
- With optimizations: ~400 KB/s per simulation
- **50% reduction in ghost entity bandwidth**
```

**Implementation Strategy:**

1. **Terrain System Required First:**
   - Collision detection for impassable terrain
   - Terrain generation/placement system
   - Agent pathfinding awareness of barriers

2. **Corner Barrier Placement:**
   - Place barriers in corners of each region (~15-20% of region area)
   - Varied sizes: some regions large mountain ranges, others small rock formations
   - Biome-appropriate: deserts → canyons, forests → dense thickets, oceans → reefs

3. **NATS Subject Simplification:**
   ```rust
   // Old: 8 subscriptions
   ghosts.from.{x-1}.{y-1}.to.{x}.{y}  (SW diagonal) ❌ Eliminated
   ghosts.from.{x+1}.{y-1}.to.{x}.{y}  (SE diagonal) ❌ Eliminated
   ghosts.from.{x-1}.{y+1}.to.{x}.{y}  (NW diagonal) ❌ Eliminated
   ghosts.from.{x+1}.{y+1}.to.{x}.{y}  (NE diagonal) ❌ Eliminated

   // New: 4 subscriptions
   ghosts.from.{x-1}.{y}.to.{x}.{y}    (W)  ✅ Keep
   ghosts.from.{x+1}.{y}.to.{x}.{y}    (E)  ✅ Keep
   ghosts.from.{x}.{y-1}.to.{x}.{y}    (S)  ✅ Keep
   ghosts.from.{x}.{y+1}.to.{x}.{y}    (N)  ✅ Keep
   ```

4. **Border Detection Simplification:**
   - Only check 4 edges (not 8 directions)
   - Eliminates corner agents being in 4 border zones simultaneously
   - Cleaner logic, fewer edge cases

**Additional Benefits:**

**1. Natural World Design:**
- Mountain ranges between regions feel organic (not artificial grid)
- Procedurally generated to look natural and varied
- Provides visual landmarks ("that mountain marks the region boundary")
- Can become memorable gameplay elements

**2. Ecological Barriers:**
- Creates distinct biomes per region (natural separation)
- Unique corner ecosystems (mountain flora/fauna, canyon creatures)
- Resource clustering near barriers (ore in mountains, water in canyons)

**3. Migration Simplification:**
- Only 4 possible migration directions (not 8)
- No diagonal boundary crossings to handle
- Simpler agent state machine

**4. Gameplay Elements:**
- Natural chokepoints for agent movement/migration
- Strategic value: safe zones in corner-protected regions
- Exploration: passes through mountains to reach new regions

**Trade-offs:**

**1. Reduced Playable Area:**
- ~15-20% of each region becomes impassable corners
- Net playable area: 80-85% of region
- **Mitigation:** Increase region size slightly (1000×1000 → 1150×1150 for same playable area)

**2. Movement Constraints:**
- Agents cannot move diagonally across region boundaries
- Must route around corner barriers
- **Mitigation:** Corners far from region centers (most movement unaffected)

**3. Pattern Recognition:**
- Players might notice grid if all regions have identical corner mountains
- **Mitigation:** Vary corner size, shape, and biome (some large, some small, some with passes)

**Variations & Enhancements:**

**1. Partial Permeability:**
- Some corners have narrow mountain passes (1-5% passage rate)
- Occasional diagonal crossing, but low traffic (90-95% reduction still achieved)
- Feels more organic than hard barriers

**2. Dynamic Barriers:**
- Seasonal changes: mountain passes open in summer, close in winter
- Gameplay events: earthquakes create new passes, avalanches close old ones

**3. Biome-Appropriate Design:**
```
Desert regions:   Canyon corners (red rock formations)
Forest regions:   Dense thicket corners (impassable vegetation)
Ocean regions:    Coral reef corners (underwater barriers)
Arctic regions:   Glacier corners (ice formations)
Volcanic regions: Lava flow corners (deadly barriers)
```

**Recommended Approach:**

1. **Phase 1:** Implement basic terrain/collision system
2. **Phase 2:** Add corner barriers with simple rectangular placement
3. **Phase 3:** Measure ghost bandwidth reduction (should be ~50%)
4. **Phase 4:** Enhance with varied shapes, biome-appropriate visuals
5. **Phase 5:** Add partial permeability for organic feel (optional)

**Synergizes With:**
- Spatial Sharding (natural complement to region architecture)
- Delta Updates (fewer neighbors = fewer updates to track)
- Horizontal Broadcaster Scaling (reduced cross-region traffic)
- Procedural generation (corners generated with terrain)

**Priority:** **Medium** (implement after terrain system, before ghost entity optimization becomes bottleneck)

---

### Rust Broadcaster (Experimental)

**Hypothesis:** Rust broadcaster may outperform Node.js for high-scale scenarios.

**Current Stack:**
```
Rust Simulation → [MessagePack] → NATS → [MessagePack] → Node.js Broadcaster → [JSON] → Clients
```

**Experimental Stack:**
```
Rust Simulation → [bincode] → NATS → [bincode] → Rust Broadcaster → [JSON] → Clients
```

**Comparison:**

| Metric | Node.js | Rust (tokio-tungstenite) |
|--------|---------|--------------------------|
| Connections/instance | ~10K | ~50K |
| Latency (p99) | 2-5ms | 0.5-2ms |
| Memory | 50MB + 10KB/client | 5MB + 2KB/client |
| CPU | Single-threaded | Multi-core work-stealing |
| Payload (with bincode) | 2.8 MB | 2.4 MB (14% smaller) |

**Benefits:**
- Smaller payloads (bincode: 14% reduction vs MessagePack)
- 2x faster serialization
- 30% fewer allocations
- 5x more connections per instance
- Multi-core utilization

**Trade-offs:**
- Loses JavaScript ecosystem (harder debugging)
- Bincode only works for Rust-to-Rust (not browser)
- Must maintain two codebases
- Higher implementation complexity

**Decision:** Defer until scaling beyond 100K+ agents or needing 50K+ connections/broadcaster.

---

## Simulation Performance Ideas

### ECS Query Optimization

**Current:** Systems iterate all entities even when unchanged.

**Optimizations:**
1. **Query Filters:** Use `Changed<>` and `With<>` to skip static entities
2. **Memory Layout:** Add `#[repr(C, align(16))]` for cache locality and SIMD
3. **Parallel Queries:** Use `par_iter()` for multi-core systems

**Expected Impact:** 25-30% simulation throughput improvement

**Risk:** Parallel queries must maintain determinism for networked simulation

---

### Batch WebSocket Broadcasting

**Current:** Individual message sends per client (if applicable).

**Optimization:**
- Serialize frame once
- Share serialized buffer across all clients via `Arc<Bytes>`
- Zero-copy broadcasting

**Impact:**
- 90% fewer allocations
- 40-50% reduction in GC pressure
- More stable frame times

---

## Serialization Format Comparison

**Reference data for 100K agents:**

| Format | Size | Speed | Cross-Language | Notes |
|--------|------|-------|----------------|-------|
| **JSON** | 10.0 MB | Baseline | ✅ Excellent | Human-readable, verbose |
| **MessagePack** | 2.8-3.5 MB | ~2x faster | ✅ Excellent | **Current choice**, npm libs available |
| **bincode** | 2.4-2.8 MB | ~3-5x faster | ❌ Rust-only | Only for Rust ↔ Rust |
| **Protobuf** | 2.5-3.2 MB | ~2.5x faster | ✅ Good | Schema-based, code generation |
| **FlatBuffers** | 3.0-4.0 MB | Zero-copy | ✅ Good | Complex schema, best for reading |

**Current Choice:** MessagePack for cross-language compatibility (Rust ↔ Node.js ↔ Browser)

---

## Combined Impact Examples

**Baseline (Current):**
- 100K agents @ 60Hz
- Payload: 2.8 MB/frame
- Bandwidth: 168 MB/s

**Spatial Sharding + Delta Updates:**
- Payload per client: 200-500 KB/frame (viewport only, deltas only)
- Bandwidth per client: 12-30 MB/s
- **98% reduction**

**+ Frame Rate Reduction (20 Hz):**
- Bandwidth per client: 4-10 MB/s
- **99.4% reduction from baseline**

**+ Quaternion + Cell-Relative:**
- Additional 20-25% reduction
- Final: 3-8 MB/s per client

---

## Monitoring & Benchmarking

**Needed Measurements:**
```bash
cargo bench --bench systems         # ECS system performance
cargo flamegraph --bin speciate      # CPU profiling

# NATS monitoring
curl http://localhost:8222/varz | jq '.mem, .in_msgs, .in_bytes'
```

**Benchmark Targets:**
- Entity iteration speed
- Serialization throughput (agents/second)
- WebSocket broadcast latency (p50, p95, p99)
- Frame publishing latency

---

## Key Insights

1. **Spatial sharding + delta updates** provide the biggest bang for buck (98-99% reduction)
2. **Horizontal broadcaster scaling** solves both NATS limits and client distribution
3. **Compression tricks** (quaternion, cell-relative) are minor gains after spatial optimization
4. **Rust broadcaster** only worthwhile at extreme scale (500K+ agents, 50K+ connections)
5. **Client-side interpolation** is essential for most optimizations to work smoothly

---

## Notes

- All bandwidth numbers assume MessagePack serialization (current implementation)
- NATS default max_payload: 1 MB (configurable to 8-64 MB if needed)
- Current agent size: ~30 bytes (id: 4, x: 5, y: 5, vx: 5, vy: 5, rotation: 5, overhead: 1)
- Test all optimizations with actual gameplay before committing to architecture
