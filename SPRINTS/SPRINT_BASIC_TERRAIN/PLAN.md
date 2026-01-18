# Sprint: Basic Terrain (L0-Aligned Obstacles)

**Status:** Planning
**Branch:** `feat/sprint-XX-basic-terrain` (TBD)
**Prerequisite:** Sprint 15 complete (ECS optimizations)

---

## Goal

Add impassable terrain obstacles aligned to L0 cells (20m). Creatures perceive and avoid obstacles independently from other creatures. Minimal performance impact by leveraging existing spatial grid architecture.

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Obstacle granularity | L0 cell (20m) | Aligns with spatial grid, cheap lookup |
| Storage | Bitmap (~8KB) | Minimal memory, O(1) lookup |
| Dynamic | Yes | Supports future CA terrain |
| Collision response | Stop | Simple, avoidance handles smooth navigation |
| **Perception** | **Separate ObstacleCache** | Prevents obstacles crowding out creature awareness |
| Visual | Cliff sprite | Simple placeholder, conveys impassability |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      TERRAIN SYSTEM                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  TerrainGrid Resource (Bevy)                                    │
│  ├─ blocked: BitVec (250×250 = 62,500 bits)                     │
│  ├─ width: u32 (cells per axis)                                 │
│  ├─ offset: f32 (world origin offset, e.g., -2500.0)            │
│  └─ cell_size: f32 (20.0, matches L0)                           │
│                                                                  │
│  Methods:                                                        │
│  ├─ is_blocked(world_x, world_y) -> bool                        │
│  ├─ set_blocked(cell_x, cell_y, blocked: bool)                  │
│  ├─ world_to_cell(world_x, world_y) -> (u32, u32)               │
│  └─ cell_to_world_center(cell_x, cell_y) -> (f32, f32)          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                   DUAL CACHE ARCHITECTURE                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  NeighborCache (existing)          ObstacleCache (NEW)          │
│  ├─ 7 slots for creatures          ├─ 4 slots for obstacles     │
│  ├─ Updated every N ticks          ├─ Updated on cell change    │
│  ├─ TTC-based avoidance            ├─ Distance-based repulsion  │
│  └─ Tracks moving entities         └─ Tracks static terrain     │
│                                                                  │
│  Biological Parallel:                                            │
│  ├─ Social tracking (predators,    ├─ Spatial memory (terrain,  │
│  │   prey, conspecifics)           │   landmarks, obstacles)    │
│  └─ Active, attention-limited      └─ Passive, always available │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Why Separate Caches?

### Problem: Unified Cache Competition

If obstacles and creatures share the same 7-slot `NeighborCache`:

| Scenario | Problem |
|----------|---------|
| Creature near wall | 3-4 obstacle slots consumed, can't see predator |
| Crowded area near cliff | Obstacles pushed out, creature clips through |
| Corridor navigation | All slots = walls, blind to threats |

### Solution: Parallel Cognitive Systems

Real animals have distinct neural systems for:
- **Spatial memory** (hippocampus) - Terrain, obstacles, landmarks
- **Social tracking** (different regions) - Conspecifics, predators, prey

A deer doesn't "forget" where the cliff is because it noticed wolves. These are parallel systems.

### Performance Benefit

| Cache | Update Frequency | Reason |
|-------|------------------|--------|
| NeighborCache | Every N ticks | Creatures move unpredictably |
| ObstacleCache | Only on cell change | Terrain is static |

Obstacle cache updates are **much cheaper** - only triggered when creature enters new L0 cell.

---

## System Changes

### 1. TerrainGrid Resource

**Location:** `apps/simulation/src/simulation/terrain/mod.rs` (new)

```
TerrainGrid
├─ blocked: BitVec or [u64; 977]
├─ width: 250 (for ±2500m world, 20m cells)
├─ Methods for world↔cell conversion
└─ Serializable for save/load
```

**Key insight:** Grid covers `[-2500, +2500]` in both axes = 5000m / 20m = 250 cells per axis.

### 2. Movement System Modification

**Location:** `apps/simulation/src/simulation/movement/systems.rs`

**Current:** Clamp position to world bounds after integration.

**New:** Before world-bounds clamp, check if target cell is blocked:
- If blocked → clamp to cell edge (don't enter blocked cell)
- Velocity zeroed in blocked direction (stop, don't slide)

```
integrate_position()
├─ new_x = pos.x + vel.vx * dt
├─ new_y = pos.y + vel.vy * dt
├─ IF terrain.is_blocked(new_x, new_y):
│   ├─ Clamp to current cell edge
│   └─ Zero velocity component into obstacle
└─ Apply world bounds clamp (existing)
```

### 3. ObstacleCache Component (NEW)

**Location:** `apps/simulation/src/simulation/terrain/components.rs` (new)

```rust
/// Perceived obstacles from terrain grid.
/// Separate from NeighborCache to prevent obstacles crowding out creature awareness.
/// Updated only when creature changes L0 cell (cheap).
#[derive(Component)]
pub struct ObstacleCache {
    pub obstacles: [PerceivedObstacle; MAX_PERCEIVED_OBSTACLES],
    pub count: u8,
    pub last_cell: (u32, u32),  // Track which cell we computed for
}

pub struct PerceivedObstacle {
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,  // Cell half-diagonal (~14m)
}

pub const MAX_PERCEIVED_OBSTACLES: usize = 4;  // Max adjacent blocked cells in any direction
```

**Why 4 slots?** A creature can have at most 4 adjacent blocked cells that matter for navigation (the 4 cardinal directions). Diagonal obstacles are less critical for avoidance.

### 4. Obstacle Perception System (NEW)

**Location:** `apps/simulation/src/simulation/terrain/systems.rs`

```rust
/// Update obstacle cache when creature enters new L0 cell.
/// Much cheaper than creature perception - only runs on cell change.
pub fn update_obstacle_cache_system(
    terrain: Res<TerrainGrid>,
    mut query: Query<(&Position, &mut ObstacleCache)>,
) {
    for (pos, mut cache) in query.iter_mut() {
        let current_cell = terrain.world_to_cell(pos.x, pos.y);

        // Early exit if still in same cell
        if current_cell == cache.last_cell {
            continue;
        }

        cache.last_cell = current_cell;
        cache.count = 0;

        // Scan 3x3 neighborhood for blocked cells
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; }  // Skip self

                let check_x = current_cell.0 as i32 + dx;
                let check_y = current_cell.1 as i32 + dy;

                if terrain.is_blocked_cell(check_x, check_y) {
                    if cache.count < MAX_PERCEIVED_OBSTACLES as u8 {
                        let (cx, cy) = terrain.cell_to_world_center(check_x, check_y);
                        cache.obstacles[cache.count as usize] = PerceivedObstacle {
                            center_x: cx,
                            center_y: cy,
                            radius: CELL_HALF_DIAGONAL,
                        };
                        cache.count += 1;
                    }
                }
            }
        }
    }
}
```

### 5. Avoidance System: Dual Cache Handling

**Location:** `apps/simulation/src/simulation/creatures/steering/avoidance.rs`

**Current:** TTC-based avoidance using `NeighborCache`.

**New:** Handle both caches with appropriate algorithms:

```rust
pub fn avoidance_system(
    mut query: Query<(
        &Position,
        &Velocity,
        &NeighborCache,      // Creatures (existing)
        &ObstacleCache,      // Obstacles (NEW)
        &mut Acceleration,
    )>,
) {
    for (pos, vel, neighbors, obstacles, mut accel) in query.iter_mut() {
        // 1. Creature avoidance (existing TTC-based logic)
        for i in 0..neighbors.count as usize {
            let neighbor = &neighbors.neighbors[i];
            let ttc = calculate_time_to_collision(pos, vel, neighbor);
            if ttc < TTC_HORIZON {
                let avoidance_force = calculate_ttc_avoidance(pos, vel, neighbor, ttc);
                accel.ax += avoidance_force.0;
                accel.ay += avoidance_force.1;
            }
        }

        // 2. Obstacle avoidance (NEW - distance-based, higher priority)
        for i in 0..obstacles.count as usize {
            let obstacle = &obstacles.obstacles[i];
            let (force_x, force_y) = calculate_obstacle_repulsion(pos, obstacle);
            accel.ax += force_x;
            accel.ay += force_y;
        }
    }
}

fn calculate_obstacle_repulsion(pos: &Position, obstacle: &PerceivedObstacle) -> (f32, f32) {
    let dx = pos.x - obstacle.center_x;
    let dy = pos.y - obstacle.center_y;
    let dist = (dx * dx + dy * dy).sqrt();

    let edge_dist = dist - obstacle.radius;
    if edge_dist <= 0.0 {
        // Inside obstacle - emergency push
        let nx = dx / dist.max(0.001);
        let ny = dy / dist.max(0.001);
        return (nx * EMERGENCY_REPULSION, ny * EMERGENCY_REPULSION);
    }

    if edge_dist > OBSTACLE_AWARENESS_DIST {
        return (0.0, 0.0);  // Too far to care
    }

    // Inverse distance repulsion
    let urgency = 1.0 - (edge_dist / OBSTACLE_AWARENESS_DIST);
    let force_mag = OBSTACLE_REPULSION_STRENGTH * urgency * urgency;  // Quadratic falloff

    let nx = dx / dist;
    let ny = dy / dist;
    (nx * force_mag, ny * force_mag)
}
```

### 6. Frontend: Terrain Rendering

**Location:** `apps/portal/src/rendering/` (new TerrainLayer)

**IPC:** Send blocked cell positions to frontend on:
- Initial load
- Terrain change (future CA system)

**Rendering:**
- Create sprite pool for cliff tiles
- Position at blocked cell centers
- Simple tiled texture (cliff/rock)
- Z-index below creatures, above background

**Optimization:** Only render visible blocked cells (viewport culling).

### 7. IPC: Terrain Data

**Location:** `apps/simulation/src/napi_addon/`

**New buffer:** `terrain_buffer: Uint8Array` or similar
- Compact representation of blocked cells
- Send full grid on init (~8KB, one-time)
- Send delta updates on terrain change (future)

**Alternative:** If terrain is static per session, bake into world generation and send once at startup.

---

## Implementation Phases

### Phase 1: Core Terrain (This Sprint)

1. **TerrainGrid resource** - Bitmap storage, world↔cell conversion
2. **ObstacleCache component** - 4-slot cache for nearby obstacles
3. **Movement blocking** - Stop at blocked cells
4. **Obstacle perception system** - Update cache on cell change
5. **Avoidance steering** - Distance-based obstacle repulsion (separate from creature TTC)
6. **Basic IPC** - Send terrain to frontend
7. **Cliff sprite rendering** - Simple visual representation
8. **Test world** - Hardcoded obstacles for testing

**Acceptance Criteria:**
- [ ] Creatures stop at blocked cells (no clipping through)
- [ ] Creatures steer around obstacles before collision
- [ ] Obstacle avoidance doesn't interfere with creature awareness (separate caches)
- [ ] Blocked cells visible as cliff sprites in portal
- [ ] No performance regression at 20K creatures

### Phase 2: Terrain Types (Future Sprint)

Evolve from bitmap to terrain type enum:

```rust
#[repr(u8)]
enum TerrainType {
    Open = 0,
    Blocked = 1,    // Impassable (cliff, water)
    Slow = 2,       // Movement penalty (mud, vegetation)
    Dangerous = 3,  // Damage over time (lava, toxic)
}
```

**Storage:** 250 × 250 bytes = ~63KB (still trivial)

**New systems:**
- Slow terrain: Movement speed multiplier
- Dangerous terrain: Energy drain per tick in cell
- DNA trait: Terrain preference (evolutionary pressure)

### Phase 3: Cellular Automata (Future)

Dynamic terrain changes based on CA rules:
- Grass regrowth after grazing
- Fire spread through forest
- Water flow / erosion

See: `docs/terrain/cellular-automata-terrain.md`

---

## File Structure

```
apps/simulation/src/simulation/
├─ terrain/
│   ├─ mod.rs           # TerrainGrid resource, methods
│   ├─ components.rs    # ObstacleCache, PerceivedObstacle (NEW)
│   ├─ constants.rs     # TERRAIN_CELL_SIZE, MAX_PERCEIVED_OBSTACLES
│   └─ systems.rs       # update_obstacle_cache_system (NEW)
├─ movement/
│   └─ systems.rs       # Modified: obstacle collision
└─ creatures/steering/
    └─ avoidance.rs     # Modified: dual-cache avoidance

apps/portal/src/
├─ rendering/
│   └─ TerrainLayer.ts  # New: cliff sprite rendering
└─ types/
    └─ GameState.ts     # Modified: terrain data types
```

---

## Golden Zone Opportunities

| Optimization | Biological Behavior |
|--------------|---------------------|
| Skip perception in blocked cells | No creatures there anyway |
| Update obstacle cache only on cell change | Spatial memory is persistent |
| Separate caches = parallel cognition | Real animals have distinct spatial vs social systems |
| Obstacle avoidance = survival pressure | Evolution favors spatial awareness |

---

## Open Questions

1. **World generation** - How to place initial obstacles? Random? Noise-based? Manual editor?
2. **Creature spawn** - Ensure creatures don't spawn in blocked cells
3. **Target validation** - Seeking targets in blocked cells should be rejected
4. **Edge behavior** - Sliding along obstacle edges vs hard stop?

---

## Dependencies

- Sprint 15 complete (ECS optimizations, Rayon parallelization)
- L0 spatial grid architecture stable
- Perception/avoidance systems working

---

## Estimated Scope

| Component | Effort |
|-----------|--------|
| TerrainGrid resource | Small |
| ObstacleCache component | Small |
| Movement blocking | Small |
| Obstacle perception system | Small |
| Avoidance modification | Medium |
| IPC terrain data | Small |
| Portal rendering | Medium |
| Testing & validation | Medium |

---

## References

- `apps/simulation/src/simulation/spatial/constants.rs` - L0 CELL_SIZE = 20.0
- `apps/simulation/src/simulation/spatial/grid.rs` - Spatial grid implementation
- `apps/simulation/src/simulation/perception/systems.rs` - Neighbor detection
- `apps/simulation/src/simulation/creatures/steering/avoidance.rs` - TTC avoidance
- `docs/terrain/cellular-automata-terrain.md` - Future CA terrain vision
- `docs/biology/ideas/collision-physics.md` - Collision concepts (not yet implemented)
