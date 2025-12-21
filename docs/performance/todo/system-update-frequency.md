# Per-System Update Frequency Control

## Overview

Add runtime-adjustable update frequency for cognitive ECS systems via dev-ui controls. Each adjustable system gets its own configurable frequency divisor, displayed in Hz.

**Important:** Only cognitive/decision-making systems can be frequency-controlled. Physics systems (movement, spatial grid rebuild) must run every tick.

---

## Solution: Spatial Grid Cell Bucketing (Zero Overhead)

### Failed Approach: Per-Entity UpdateSlice

The original plan used a per-entity `UpdateSlice` component with double modulus filtering:

```rust
// FAILED: 7ms overhead (29% regression from 24ms baseline)
let mut entities: Vec<_> = query
    .iter_mut()
    .filter(|(.., slice)| slice.id % active == current)
    .collect();
```

**Why it failed:**
- O(N) iteration over ALL 360k entities just to filter
- Filter runs before parallel work, killing performance
- Added 4 bytes per entity for the component

### Winning Approach: Spatial Grid Cell Bucketing

**Key Insight**: The spatial grid already partitions entities into cells. Use `cell_index % divisor` as the bucket - zero new data structures needed!

```rust
pub fn update_perception_system(
    tick: Res<PhysicsTick>,
    config: Res<FreqConfig>,
    grid: Res<DoubleBufferedSpatialGrid>,
    mut query: Query<(&Position, &Rotation, &Perception, &mut NeighborCache, ...)>,
) {
    let divisor = config.perception_divisor as usize;

    // HOT PATH: divisor=1 means process all cells (full rate)
    if divisor == 1 {
        // Existing code path - unchanged
        let mut entities: Vec<_> = query.iter_mut().collect();
        entities.par_iter_mut().for_each(|...| { /* work */ });
        return;
    }

    // THROTTLED PATH: Process only cells in current bucket
    let grid_ref = grid.read_grid();
    let current_bucket = (tick.get() as usize) % divisor;
    let total_cells = grid_ref.allocated_cells();

    // Collect entities from cells in current bucket
    let mut bucket_entities: Vec<Entity> = Vec::new();
    for cell_idx in (current_bucket..total_cells).step_by(divisor) {
        for proxy in grid_ref.get_cell_proxies(cell_idx) {
            bucket_entities.push(proxy.entity);
        }
    }

    // Process only bucket entities
    bucket_entities.par_iter().for_each(|&entity| {
        if let Ok((pos, rot, perc, mut cache, ...)) = query.get_mut(entity) {
            // Perception logic for this entity
        }
    });
}
```

### Performance Comparison

| Approach | Full Rate (divisor=1) | Throttled | Memory/Entity |
|----------|----------------------|-----------|---------------|
| UpdateSlice (failed) | +7ms | +7ms | +4 bytes |
| Entity.index() | 0ms | +2ms | 0 |
| **Spatial Grid Cells** | **0ms** | **~0.1ms** | **0** |

### Why Spatial Grid Wins

1. **Cell indices are sequential** (0, 1, 2, ...) - `step_by(divisor)` is O(cells), not O(entities)
2. **Cells are already grouped in memory** - cache-friendly iteration
3. **Grid is already rebuilt every tick** - no extra maintenance
4. **Skip empty cells naturally** - no wasted iteration

### Spatial Distribution Bonus

Cell-based bucketing has a hidden advantage for perception: nearby creatures are in nearby cells. When we process a bucket of cells, those creatures can perceive each other in the same tick. This is better than random distribution for gameplay smoothness!

---

## Integration with Dual-Spatial-Grid

See: `docs/performance/todo/dual-spatial-grid.md`

The dual-spatial-grid proposal introduces:
- **L0 (Fine Grid)**: 20m cells, stores EntityIDs, for immediate interactions
- **L1 (Coarse Grid)**: 100m cells (5x coarser), stores BioSignatures, for navigation

### Synergy Analysis

| Factor | Impact | Notes |
|--------|--------|-------|
| L1 has fewer cells | ✅ Excellent | 25x fewer cells to iterate for bucketing |
| Nearby creatures same bucket | ✅ Excellent | 100m regions update together |
| Phase 2 early exit | ✅ Excellent | Empty-area creatures already skip work |
| L1 aggregation timing | ⚠️ Consideration | Must run every tick regardless |

### Recommended Strategy

**If dual-spatial-grid is implemented first:**
- Bucket by L1 (coarse) cells instead of L0 (fine)
- `coarse_cell_index % divisor` gives even bigger regions updating together
- 100m² regions = ~25 creatures per bucket (at 1 creature/400m²)
- Phase 1 (coarse scan) and Phase 3 (fine scan) run together for bucketed creatures

**If system-frequency is implemented first:**
- Use current 10m grid cells for bucketing
- When dual-spatial-grid lands, migrate to L1 bucketing (simple change)

### Implementation Order Recommendation

1. **Implement system-frequency first** (this doc) - using current grid
2. **Implement dual-spatial-grid second** - adds L1 coarse grid
3. **Migrate frequency bucketing to L1** - trivial refactor, bigger regions

The approaches are **complementary, not conflicting**.

---

## Review Findings (Specialist Agents)

**Reviewed by:** rusty-ron (Rust), frontend-fanny (Dev-UI), ecs-emma (ECS Architecture)

| System | Verdict | Rationale |
|--------|---------|-----------|
| Perception | ✅ Adjustable | Cognitive, stale data acceptable |
| Behavior Transition | ✅ Adjustable | Decision-making, not physics |
| Steering | ✅ Adjustable | Decision-making, stale neighbors OK |
| Movement | ❌ NEVER | Physics integration requires every-tick |
| Spatial Grid | ❌ NEVER | Perception accuracy depends on current positions |

---

## Implementation

### FreqConfig Resource

**File:** `apps/simulation/src/simulation/core/resources.rs`

```rust
#[derive(Resource, Clone, Debug)]
pub struct FreqConfig {
    pub perception_divisor: u8,      // 1 = every tick, 10 = every 10th tick
    pub behavior_divisor: u8,
    pub steering_divisor: u8,
}

impl Default for FreqConfig {
    fn default() -> Self {
        Self {
            perception_divisor: 1,    // Full rate by default
            behavior_divisor: 1,
            steering_divisor: 1,
        }
    }
}
```

### Files to Modify

| File | Change |
|------|--------|
| `src/simulation/core/resources.rs` | Add FreqConfig resource |
| `src/simulation/core/simulation.rs` | Insert FreqConfig resource |
| `src/simulation/perception/systems.rs` | Add cell-bucket throttle path |
| `src/simulation/creatures/behaviors/transitions/systems.rs` | Add cell-bucket throttle path |
| `src/simulation/creatures/steering/system.rs` | Add cell-bucket throttle path |
| `src/ipc/sim_command.rs` | Add SetSystemFrequency command |
| `src/simulation/creatures/components/update_slice.rs` | **DELETE** |
| `src/simulation/creatures/builder.rs` | Remove UpdateSlice from spawn |

### IPC Command

```rust
pub enum Command {
    SetSystemFrequency {
        system: String,  // "perception", "behavior", "steering"
        divisor: u8,     // 1-100
    },
}
```

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Bucketing strategy | Spatial grid cells | Zero overhead at full rate |
| Default divisors | All = 1 | Full rate by default, user opts into throttling |
| Hot path guard | `if divisor == 1` | Ensures baseline performance unchanged |
| Grid level | Current (10m) | Migrate to L1 (100m) after dual-grid lands |
| UpdateSlice component | DELETE | No longer needed |

---

## Future: Dual-Grid Integration

When `dual-spatial-grid.md` is implemented:

```rust
// FUTURE: Use L1 coarse grid (100m cells) instead of L0
let grid_ref = grid.coarse_grid();  // L1
let total_cells = grid_ref.cell_count();  // 25x fewer than L0

for cell_idx in (current_bucket..total_cells).step_by(divisor) {
    for entity in grid_ref.get_cell_entities(cell_idx) {
        bucket_entities.push(entity);
    }
}
```

Benefits:
- 25x fewer cells to iterate
- 100m regions update together (better spatial coherence)
- Natural alignment with perception Phase 1/3 split
