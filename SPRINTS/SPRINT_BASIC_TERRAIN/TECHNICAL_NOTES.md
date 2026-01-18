# Technical Notes: L0-Aligned Terrain

## Cell Math

```
World size: ±2500m (MAX_WORLD_SIZE = 5000, centered at origin)
L0 cell size: 20m (CELL_SIZE constant)
Cells per axis: 5000 / 20 = 250
Total cells: 250 × 250 = 62,500
Bitmap size: 62,500 bits = 7,813 bytes ≈ 8KB
```

### World ↔ Cell Conversion

```rust
const CELL_SIZE: f32 = 20.0;
const WORLD_OFFSET: f32 = 2500.0;  // Shift origin to positive space
const CELLS_PER_AXIS: u32 = 250;

fn world_to_cell(world_x: f32, world_y: f32) -> (u32, u32) {
    let cell_x = ((world_x + WORLD_OFFSET) / CELL_SIZE) as u32;
    let cell_y = ((world_y + WORLD_OFFSET) / CELL_SIZE) as u32;
    (cell_x.min(CELLS_PER_AXIS - 1), cell_y.min(CELLS_PER_AXIS - 1))
}

fn cell_to_world_center(cell_x: u32, cell_y: u32) -> (f32, f32) {
    let world_x = (cell_x as f32 * CELL_SIZE) + (CELL_SIZE / 2.0) - WORLD_OFFSET;
    let world_y = (cell_y as f32 * CELL_SIZE) + (CELL_SIZE / 2.0) - WORLD_OFFSET;
    (world_x, world_y)
}

fn cell_index(cell_x: u32, cell_y: u32) -> usize {
    (cell_y * CELLS_PER_AXIS + cell_x) as usize
}
```

---

## Movement Collision

### Approach: Check Before Move

```rust
// In movement integration
let new_x = pos.x + vel.vx * dt;
let new_y = pos.y + vel.vy * dt;

if terrain.is_blocked(new_x, new_y) {
    // Option 1: Full stop (simplest)
    // Don't update position, zero velocity
    vel.vx = 0.0;
    vel.vy = 0.0;

    // Option 2: Axis-aligned stop (allows sliding)
    // Check each axis independently - more complex
}
```

### Edge Case: Diagonal Movement

Creature at (19, 19) moving to (21, 21) - crosses cell boundary diagonally.

**Simple approach:** Check destination cell only.
- Fast, works for most cases
- May clip corner of blocked cell at high speeds

**Robust approach:** Check all cells along path.
- Use line rasterization (Bresenham or DDA)
- More expensive, needed if speeds > 20m/tick

At current speeds (~10 m/s max) and tick rate (22Hz), max movement per tick ≈ 0.45m. Well under 20m cell size, so simple approach is safe.

---

## Dual Cache Architecture

### Why Separate Caches?

| Problem with Unified Cache | Example |
|---------------------------|---------|
| Obstacles crowd out creatures | Near wall: 4 obstacle slots, only 3 creature slots left |
| Creatures crowd out obstacles | Crowded area: all 7 slots = creatures, clip through wall |
| Different update frequencies | Creatures: every N ticks. Obstacles: only on cell change |

### ObstacleCache Design

```rust
pub const MAX_PERCEIVED_OBSTACLES: usize = 4;

#[derive(Component)]
pub struct ObstacleCache {
    pub obstacles: [PerceivedObstacle; MAX_PERCEIVED_OBSTACLES],
    pub count: u8,
    pub last_cell: (u32, u32),  // Early-exit optimization
}

#[derive(Clone, Copy, Default)]
pub struct PerceivedObstacle {
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
}
```

**Why 4 slots?**
- At most 8 adjacent cells (3×3 minus self)
- But realistically only ~4 matter for avoidance (cardinal directions)
- Diagonal obstacles rarely need direct avoidance (you hit cardinals first)
- Memory: 4 × 12 bytes + 1 + 8 = ~57 bytes per creature

### Update Frequency Comparison

| Cache | Trigger | Frequency | Cost |
|-------|---------|-----------|------|
| NeighborCache | Every N ticks (2-8) | ~5-12 Hz | High (spatial query) |
| ObstacleCache | Cell boundary crossing | ~0.5-2 Hz | Low (local 3×3 scan) |

**ObstacleCache is ~10x cheaper** because:
1. Only updates when creature moves 20m (enters new cell)
2. Just checks 8 adjacent cells (no spatial query)
3. No sorting, no distance calculations beyond simple check

### Early Exit Optimization

```rust
pub fn update_obstacle_cache_system(
    terrain: Res<TerrainGrid>,
    mut query: Query<(&Position, &mut ObstacleCache)>,
) {
    for (pos, mut cache) in query.iter_mut() {
        let current_cell = terrain.world_to_cell(pos.x, pos.y);

        // EARLY EXIT: Still in same cell, cache is valid
        if current_cell == cache.last_cell {
            continue;  // Skip ~90% of creatures per tick
        }

        // Only ~10% of creatures cross cell boundaries per tick
        cache.last_cell = current_cell;
        // ... rebuild cache
    }
}
```

At 22Hz tick rate, creature moving 10 m/s crosses cell boundary every ~2 seconds = ~44 ticks. So ~2% of creatures update obstacle cache per tick.

---

## Avoidance: Dual Algorithm

### Creature Avoidance (Existing)

**Algorithm:** Time-To-Collision (TTC)
- Predict when creature will collide with neighbor
- Apply steering force inversely proportional to TTC
- Works well for moving targets

```rust
let ttc = calculate_time_to_collision(self_pos, self_vel, neighbor_pos, neighbor_vel);
let urgency = 1.0 / (ttc + 0.1);
let force = urgency * avoidance_direction;
```

### Obstacle Avoidance (NEW)

**Algorithm:** Distance-based repulsion
- No TTC calculation (obstacles don't move)
- Simple inverse-distance force
- Higher base strength (obstacles don't yield)

```rust
fn calculate_obstacle_repulsion(pos: &Position, obstacle: &PerceivedObstacle) -> (f32, f32) {
    let dx = pos.x - obstacle.center_x;
    let dy = pos.y - obstacle.center_y;
    let dist = (dx * dx + dy * dy).sqrt();

    // Edge distance (how close to obstacle surface)
    let edge_dist = dist - obstacle.radius;

    // Inside obstacle - emergency escape
    if edge_dist <= 0.0 {
        let nx = dx / dist.max(0.001);
        let ny = dy / dist.max(0.001);
        return (nx * EMERGENCY_REPULSION, ny * EMERGENCY_REPULSION);
    }

    // Too far to care
    if edge_dist > OBSTACLE_AWARENESS_DIST {
        return (0.0, 0.0);
    }

    // Quadratic falloff repulsion
    let urgency = 1.0 - (edge_dist / OBSTACLE_AWARENESS_DIST);
    let force_mag = OBSTACLE_REPULSION_STRENGTH * urgency * urgency;

    let nx = dx / dist;
    let ny = dy / dist;
    (nx * force_mag, ny * force_mag)
}
```

### Tuning Constants

```rust
// Obstacle avoidance
const OBSTACLE_REPULSION_STRENGTH: f32 = 20.0;  // Higher than creature (15.0)
const OBSTACLE_AWARENESS_DIST: f32 = 15.0;       // Start avoiding at this distance
const EMERGENCY_REPULSION: f32 = 100.0;          // If inside obstacle

// For reference - creature avoidance
const CREATURE_AVOIDANCE_STRENGTH: f32 = 15.0;
const TTC_HORIZON: f32 = 3.0;  // Seconds ahead to predict
```

**Why obstacles have higher strength:**
- Obstacles don't yield (unlike creatures that also avoid you)
- Missing an obstacle = wall collision (bad)
- Missing a creature = they probably avoid you too (less bad)

---

## IPC: Terrain Buffer

### Option A: Full Bitmap (Simplest)

Send entire 8KB bitmap on init:

```rust
// Rust side
pub fn get_terrain_bitmap(&self) -> Vec<u8> {
    self.terrain.blocked.as_raw_slice().to_vec()
}
```

```typescript
// TypeScript side
const bitmap = new Uint8Array(terrain_buffer);
function isBlocked(cellX: number, cellY: number): boolean {
    const idx = cellY * 250 + cellX;
    const byteIdx = Math.floor(idx / 8);
    const bitIdx = idx % 8;
    return (bitmap[byteIdx] & (1 << bitIdx)) !== 0;
}
```

### Option B: Blocked Cell List (Sparse)

If <10% of cells blocked, list is smaller:

```rust
pub fn get_blocked_cells(&self) -> Vec<(u16, u16)> {
    self.terrain.blocked_cell_coords()  // Only blocked cells
}
```

For 1000 blocked cells: 1000 × 4 bytes = 4KB (comparable to bitmap).

**Recommendation:** Start with full bitmap for simplicity. Optimize to sparse list if terrain is mostly open.

---

## Portal Rendering

### Cliff Sprite Pool

```typescript
class TerrainLayer {
    private cliffPool: Sprite[] = [];
    private activeCliffs: Map<string, Sprite> = new Map();

    updateTerrain(blockedCells: [number, number][]) {
        // Return unused sprites to pool
        // Create/reuse sprites for blocked cells
        // Position at cell world coordinates
    }

    updateViewport(viewport: Viewport) {
        // Show only visible blocked cells
        // Hide off-screen sprites (return to pool)
    }
}
```

### Sprite Positioning

```typescript
const CELL_SIZE = 20;
const WORLD_OFFSET = 2500;

function cellToWorld(cellX: number, cellY: number): [number, number] {
    const worldX = cellX * CELL_SIZE + CELL_SIZE / 2 - WORLD_OFFSET;
    const worldY = cellY * CELL_SIZE + CELL_SIZE / 2 - WORLD_OFFSET;
    return [worldX, worldY];
}
```

### Visual Style

- Cliff/rock texture tiled at 20m×20m
- Slight variation (2-3 sprite variants) to avoid grid look
- Consider edge detection for smoother borders (Phase 2)

---

## Test Scenarios

### 1. Direct Collision

```
Creature at (0, 0) moving toward blocked cell at (40, 0)
Expected: Stop at cell edge (~20m), velocity zeroed
```

### 2. Diagonal Approach

```
Creature at (0, 0) moving toward blocked cell at (40, 40)
Expected: Avoidance steering activates, creature curves around
```

### 3. Corridor Navigation

```
Two parallel rows of blocked cells forming a corridor
Creature seeking target on other side
Expected: Finds path through corridor, doesn't clip walls
```

### 4. Cache Independence

```
Creature near wall with 7 other creatures nearby
Expected: All 7 creatures in NeighborCache, wall in ObstacleCache
         Both avoidance behaviors active simultaneously
```

### 5. Cache Update Frequency

```
Creature moving at 10 m/s
Expected: ObstacleCache updates every ~44 ticks (when crossing cell boundary)
         NeighborCache updates every 2-8 ticks (normal schedule)
```

### 6. Performance

```
20K creatures, 1000 blocked cells (~1.6% of grid)
Expected: No measurable FPS regression (<5%)
```

---

## Memory Budget

### Per-Creature Components

| Component | Size | Notes |
|-----------|------|-------|
| NeighborCache | ~170 bytes | 7 × 24 + overhead |
| ObstacleCache | ~57 bytes | 4 × 12 + 1 + 8 |
| **Delta** | **+57 bytes** | New memory per creature |

At 20K creatures: +57 × 20,000 = 1.14 MB additional memory. Negligible.

### Global Resources

| Resource | Size | Notes |
|----------|------|-------|
| TerrainGrid bitmap | ~8 KB | 62,500 bits |

---

## Future: Phase 2 Terrain Types

```rust
#[repr(u8)]
#[derive(Clone, Copy, Default)]
pub enum TerrainType {
    #[default]
    Open = 0,
    Blocked = 1,
    Slow = 2,      // 0.5x movement speed
    Dangerous = 3, // -1 energy/tick
}

pub struct TerrainGrid {
    cells: Vec<TerrainType>,  // 62,500 bytes
    // ... same dimensions
}
```

### Movement with Terrain Types

```rust
match terrain.get(new_x, new_y) {
    TerrainType::Open => { /* normal movement */ }
    TerrainType::Blocked => { /* stop */ }
    TerrainType::Slow => {
        vel.vx *= 0.5;
        vel.vy *= 0.5;
    }
    TerrainType::Dangerous => { /* allow movement, damage handled elsewhere */ }
}
```

### Energy Drain System

```rust
fn terrain_damage_system(
    terrain: Res<TerrainGrid>,
    mut query: Query<(&Position, &mut CreatureState)>,
) {
    for (pos, mut state) in query.iter_mut() {
        if terrain.get(pos.x, pos.y) == TerrainType::Dangerous {
            state.energy -= TERRAIN_DAMAGE_PER_TICK;
        }
    }
}
```

---

## Potential Optimizations

1. **L1 terrain hints** - Store "any blocked in this L1?" flag for early skip
2. **SIMD bitmap ops** - Use `u64` chunks for fast block checking
3. **Sparse storage** - If <5% blocked, use HashSet instead of bitmap
4. **Parallel obstacle cache update** - Rayon for the ~2% that need updates
