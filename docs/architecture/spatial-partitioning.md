# Unified Spatial Partitioning Strategy

## Executive Summary

This document defines the spatial query infrastructure for the simulation, supporting **150,000-200,000 creatures** in a **1000km × 1000km world** with varied creature sizes (1-20m).

**Core Strategy:** Bucket-based spatial hash grid with 200m cells, incremental updates, and dual-tick architecture.

---

## Assumptions

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| World size | 1000km × 1000km | Core feature requirement |
| Creature size | 1-20m body length | Mouse to elephant scale |
| Perception range | 10-200m (10× body length) | DNA-driven, scales with size |
| Target population | 150,000-200,000 | Realistic for dual-tick architecture |
| Physics tick | 30Hz | Smooth motion with frontend interpolation |
| AI tick | 20Hz | Biological reaction time (50ms) |
| Frontend render | 90Hz | Interpolated smoothing |
| Cell size | 200m | 2× max perception range |

---

## Architecture Decision: Bucket Grid (200m Cells)

### Why NOT Fine-Grained 1m Cells (Occupancy Grid)

**Intuition:** "1m cells = precise lookup, O(1) per cell"

**Reality:** Cache misses kill performance.

**Perception query (200m radius):**
- 1m cells: 125,664 HashMap lookups → **20M CPU cycles** (cache thrashing)
- 200m cells: 9 HashMap lookups + 180 entity scan → **540 CPU cycles** (L1 cache)

**Performance ratio: 37,000×**

**Root cause:** HashMap lookups require random memory access (200 cycles/miss). Vec scans are sequential (1 cycle/element due to prefetching).

### Why 200m Bucket Grid Wins

1. **Fewer lookups:** 9 cells vs 125K cells
2. **Cache locality:** Vec<Entity> is contiguous memory, fits in L1 cache
3. **Minimal filtering:** ~180 candidates vs 125K cells to check
4. **Sparse world support:** FxHashMap handles 1000km world efficiently

---

## Data Structure

```rust
#[derive(Resource)]
pub struct SpatialGrid {
    cell_size: f32,                                    // 200.0
    inv_cell_size: f32,                                // 1/200 (precomputed)
    cells: FxHashMap<(i32, i32), Vec<Entity>>,         // Bucket storage
}

#[derive(Component)]
pub struct SpatialCell(pub (i32, i32));               // Tracks current cell per entity
```

### Memory Estimates (200K Creatures)

- Occupied cells: ~10,000 (200K / 20 per cell)
- FxHashMap overhead: ~320KB
- Vec storage: ~800KB (4 bytes × 200K entities)
- SpatialCell component: ~1.6MB (8 bytes × 200K)
- **Total: ~3MB** (negligible)

---

## Tick Architecture Integration

The spatial grid operates within a dual-tick simulation architecture:

- **Physics tick (30Hz):** Grid updated incrementally during movement
- **AI tick (20Hz):** Grid queried for perception
- **Frontend (90Hz):** Interpolated rendering

**Key insight:** Grid is always fresh for collision (updated every physics tick), slightly stale for perception (0-33ms old), which is acceptable.

**See:** `docs/architecture/dual-tick-simulation.md` for complete tick architecture details.

---

## Incremental Grid Updates

### Key Insight

Instead of full rebuild every frame, update grid **only when creatures change cells**.

**With 200m cells and 50 m/s max speed:**
- Distance per tick: 1.67m
- Cell crossing probability: 1.67 / 200 = **0.8%**
- Updates per tick: 200K × 0.8% = **~1,600 operations**

**133× faster than full rebuild (200K operations).**

### Implementation

```rust
fn integrate_motion_with_grid_update(
    mut grid: ResMut<SpatialGrid>,
    mut query: Query<(Entity, &mut Position, &Velocity, &mut SpatialCell)>,
    dt: Res<DeltaTime>,
) {
    for (entity, mut pos, vel, mut current_cell) in query.iter_mut() {
        // Update position
        pos.x += vel.vx * dt.0;
        pos.y += vel.vy * dt.0;

        // Check for cell change
        let new_cell = grid.position_to_cell(&pos);

        if new_cell != current_cell.0 {
            // Remove from old cell (O(K) scan, K≈20)
            grid.remove_entity(entity, current_cell.0);
            // Add to new cell (O(1) push)
            grid.insert_entity(entity, new_cell);
            current_cell.0 = new_cell;
        }
    }
}
```

### Grid Operations

```rust
impl SpatialGrid {
    #[inline]
    pub fn position_to_cell(&self, pos: &Position) -> (i32, i32) {
        (
            (pos.x * self.inv_cell_size).floor() as i32,
            (pos.y * self.inv_cell_size).floor() as i32,
        )
    }

    pub fn insert_entity(&mut self, entity: Entity, cell: (i32, i32)) {
        self.cells
            .entry(cell)
            .or_insert_with(|| Vec::with_capacity(32))
            .push(entity);
    }

    pub fn remove_entity(&mut self, entity: Entity, cell: (i32, i32)) {
        if let Some(vec) = self.cells.get_mut(&cell) {
            if let Some(idx) = vec.iter().position(|&e| e == entity) {
                vec.swap_remove(idx);  // O(1) after finding
            }
        }
    }

    pub fn query_radius(&self, center: &Position, radius: f32) -> Vec<Entity> {
        let min_cell = self.position_to_cell(&Position {
            x: center.x - radius,
            y: center.y - radius,
        });
        let max_cell = self.position_to_cell(&Position {
            x: center.x + radius,
            y: center.y + radius,
        });

        let mut results = Vec::with_capacity(256);
        for cx in min_cell.0..=max_cell.0 {
            for cy in min_cell.1..=max_cell.1 {
                if let Some(entities) = self.cells.get(&(cx, cy)) {
                    results.extend_from_slice(entities);
                }
            }
        }
        results
    }
}
```

---

## Performance Projections

### Per-Tick Costs (30Hz Physics)

| Creatures | Grid Updates | Collision Detection | Total Physics | Status |
|-----------|-------------|---------------------|---------------|--------|
| 10,000 | 0.3ms | 1ms | 1.3ms | ✅ Easy |
| 50,000 | 1.5ms | 5ms | 6.5ms | ✅ Comfortable |
| 100,000 | 3ms | 10ms | 13ms | ✅ Good |
| 200,000 | 6ms | 20ms | 26ms | ⚠️ Budget limit |

### Per-Tick Costs (20Hz AI)

| Creatures | Perception Queries | Steering Calc | Total AI | Status |
|-----------|-------------------|---------------|----------|--------|
| 10,000 | 4ms | 1ms | 5ms | ✅ Easy |
| 50,000 | 20ms | 5ms | 25ms | ✅ Good |
| 100,000 | 40ms | 10ms | 50ms | ⚠️ Budget limit |
| 200,000 | 80ms | 20ms | 100ms | ❌ Over budget |

**Conservative target: 150,000 creatures**
**Optimistic target: 200,000 creatures**

---

## Consumer Systems

All these systems use the single shared spatial grid:

| System | Tick Rate | Query Pattern | Range |
|--------|-----------|--------------|-------|
| **Collision Detection** | 30Hz | Check pairs in same/adjacent cells | 0-40m |
| **Perception** | 20Hz | Radius query, filter by distance | 10-200m |
| **Avoidance Forces** | 20Hz | Uses perception results | N/A |
| **Viewport Culling** | 90Hz (frontend) | Rectangle query | Variable |

---

## Future Optimizations

### Phase 1 (Current Target): Dual-Tick (150-200K)
- 30Hz physics + collision
- 20Hz AI + perception
- 200m bucket grid with incremental updates
- Frontend interpolation at 90Hz

### Phase 2 (Future): Parallel Perception
- Bevy parallel queries with `par_iter_mut()`
- SIMD distance calculations
- Thread-local query buffers
- Target: 300K creatures

### Phase 3 (Ambitious): LOD Simulation
- Near player: Full simulation (10K-50K)
- Regional: Simplified AI (50K-100K)
- Background: Statistical aggregation (rest of 1M)
- Required for 1M creature goal

### Phase 4 (Research): GPU Compute
- Spatial queries on GPU
- Entire perception system parallel
- WebGPU/compute shaders
- True 1M creature simulation

---

## Why FxHash (Not Default HashMap)

Standard Rust HashMap uses SipHash (cryptographically secure, slow).
FxHash is 2-5× faster for integer keys like (i32, i32).

```toml
# Cargo.toml
[dependencies]
rustc-hash = "2.0"
```

```rust
use rustc_hash::FxHashMap;
```

At 10K+ cell lookups per frame, this makes measurable difference.

---

## Bevy Integration

### Resource Pattern

```rust
#[derive(Resource)]
pub struct SpatialGrid { ... }

// Exclusive access during physics tick
fn physics_system(mut grid: ResMut<SpatialGrid>) { ... }

// Shared access during AI tick (multiple systems can read)
fn perception_system(grid: Res<SpatialGrid>) { ... }
```

### System Ordering

```rust
app.add_systems(Update, (
    // Physics schedule (30Hz internal)
    physics_tick_system,

    // AI schedule (20Hz internal)
    ai_tick_system,
).chain());
```

The tick management happens inside wrapper systems that track accumulators.

---

## References

- **Collision system:** `docs/gameplay/critter-collision-system.md`
- **Behavior engine:** `docs/architecture/behavior-engine.md`
- **Biology notes:** `docs/biology/biology-notes.md`
- **Dual-tick rationale:** Based on biological reaction times (50ms) and physics stability (33ms)

---

## Key Decisions Log

**2025-11-16: Bucket Grid over Occupancy Grid**
- 200m cells with Vec<Entity> vs 1m cells with single Entity
- Cache locality wins: 540 cycles vs 20M cycles per query
- 37,000× performance difference due to L1 cache hits

**2025-11-16: Incremental Updates over Full Rebuild**
- Track cell per entity (SpatialCell component)
- Update only on cell boundary crossing (~0.8% of creatures per tick)
- 133× fewer grid operations

**2025-11-16: Dual-Tick Architecture**
- 30Hz physics + collision (fresh grid, smooth motion)
- 20Hz AI + perception (biologically realistic reaction time)
- 90Hz frontend interpolation (visual smoothness)
- Enables 150-200K creatures vs 10K single-tick
