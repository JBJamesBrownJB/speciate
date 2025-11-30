# Spatial Grid Optimization Journey

**Sprint Focus:** Optimizing perception system for 20K+ creatures
**Status:** In Progress (reverted to baseline, next attempt tomorrow)
**Date Started:** 2025-11-29

---

## TL;DR

1. **Topological distance sorting WORKED** (correct simulation) but **KILLED performance** (292ms)
2. **Rayon parallelization = MASSIVE win** (15ms) - we WILL use this
3. **But first:** Try incremental cell tracking approach to reduce algorithmic complexity
4. **Goal:** Incremental updates + Rayon = best of both worlds
5. **Diagnostic overlay:** Need to visualise perception in portal

---

## The Problem

Perception system became a bottleneck as creature count scaled toward 20K target:
- Initial perception time: ~105ms at scale
- After topological neighbor changes: **292ms** (regression)
- Target: <15ms per tick

---

### Ideas

- What if, a seperate system updated a new component on a crit which was something like 'cellid'. Then current 'temporal' system used that id, not entity id... 

## Journey Timeline

### Phase 1: Initial State (Pre-Optimization)

**Architecture:**
- Dense 2D Vec grid (`Vec<Vec<Vec<Entity>>>`)
- Fixed grid dimensions based on world bounds
- Full grid rebuild every tick
- Distance-based neighbor sorting (heap sort)

**Performance:** ~105ms perception at 10K creatures

---

### Phase 2: Topological Neighbor Sorting (CORRECT but SLOW)

**Goal:** Sort neighbors by distance so creatures interact with nearest neighbors first.

**Implementation:** Used a binary heap to sort neighbors by distance during perception queries.

**Result:** **Simulation accuracy was CORRECT** - creatures behaved properly, nearest neighbors were prioritized.

**Problem:** **Performance killed** - went from ~105ms to **292ms**

**Why:** Distance calculation + heap operations for every neighbor pair is expensive at scale.

**Lesson:** The approach was logically correct, but the computational cost was too high. Need a cheaper way to get "roughly ordered" neighbors.

---

### Phase 3: HashMap Conversion Attempt (ALSO FAILED)

**Hypothesis:** Sparse HashMap would be more efficient than dense Vec for large worlds with clustered creatures.

**Change:**
```rust
// Before: Dense Vec
cells: Vec<Vec<Vec<(Entity, f32, f32, f32)>>>

// After: Sparse HashMap
cells: HashMap<(i32, i32), Vec<(Entity, f32, f32, f32)>>
```

**Result:** Still slow - allocation issues compounded the problem.

**Why it failed:**
- `HashMap::clear()` deallocates all inner Vecs
- Every tick: allocate → populate → clear → deallocate
- Allocation churn killed performance

**Lesson:** Naive HashMap conversion without considering allocation patterns is worse than dense Vec.

---

### Phase 4: Grid Size Capping (Partial Fix)

**Problem:** 11 million cells in the dense Vec were being iterated.

**Fix:** Cap grid to `MAX_CELLS_PER_AXIS = 500` (250K cells max)

**Result:** Down to ~105ms

**Issue:** Cell size became 4000m vs 60m perception range = 99.98% false positives per cell query.

---

### Phase 5: Sparse HashMap with Occupied Cell Tracking (Better)

**Key insight from ECS-Emma agent:** Track which cells are occupied, only clear those.

**Implementation:**
```rust
pub struct SpatialGrid {
    cells: HashMap<(i32, i32), Vec<(Entity, f32, f32, f32)>>,
    occupied_cells: Vec<(i32, i32)>,  // Only track non-empty cells
}

pub fn clear(&mut self) {
    // Only clear occupied cells, don't deallocate inner Vecs
    for key in self.occupied_cells.drain(..) {
        if let Some(cell) = self.cells.get_mut(&key) {
            cell.clear();  // Keeps capacity!
        }
    }
}
```

**Result:** ~115ms - modest improvement, still not enough

**Why:** Still doing full rebuild every tick (O(all entities) insert)

---

### Phase 6: Rayon Parallelization (MASSIVE WIN - Will Use Later)

**Change:** Added Rayon parallel iteration to perception queries.

```rust
// Collect all entities for parallel processing
let mut entity_data: Vec<_> = query.iter_mut().collect();

// Parallel perception updates
entity_data.par_iter_mut().for_each(|(entity, pos, perception, ...)| {
    // Query neighbors in parallel
});
```

**Result:** **~15ms** - 7.5x improvement!

**Why it worked:**
- Perception queries are read-only on grid (no lock contention)
- Each creature's neighbor lookup is independent
- Perfect workload for data parallelism

**Status:** This is a MASSIVE performance win that we WILL take advantage of. However, we want to try the incremental cell tracking approach FIRST to see how much we can gain from algorithmic improvements alone. Then we'll layer Rayon on top for the full benefit.

---

### Phase 7: Incremental Cell Tracking (NEXT ATTEMPT - Tomorrow)

**The Insight:** We're still rebuilding the entire grid every tick, even though most creatures don't change cells.

**The Idea (from user):**
- Each creature tracks its current `CellId` as a component
- Movement system checks if cell changed (super quick conditional)
- Only update grid for creatures that actually moved cells
- Perception uses "lazy fill" - grab first N neighbors without distance sorting

**Why this should work:**
1. At 60m cell size, most creatures stay in same cell between ticks
2. ~5% cell-change rate means 95% less grid operations
3. Lazy fill gives "good enough" spatial ordering without expensive heap sort
4. Pre-topological perception was fast because it used entity ID ordering (temporal locality)
5. Cell-based ordering gives similar benefit (spatial locality)

**Architecture:**
```
1. Movement System
   - Update Position
   - Check if cell changed: new_cell = floor(pos / cell_size)
   - If changed: write to CellChangeBuffer, update CellId component

2. Process Cell Changes System
   - Drain CellChangeBuffer
   - grid.remove(entity, old_cell)
   - grid.insert(entity, x, y, radius)

3. Populate New Entities System
   - Query<Added<CellId>>
   - Insert into grid, set initial CellId

4. Perception System
   - Lazy fill: take first N neighbors within range
   - No distance-based heap sorting
   - Spatial locality from cell iteration order
```

**Key Components:**
```rust
#[derive(Component)]
pub struct CellId(pub i32, pub i32);

#[derive(Resource)]
pub struct CellChangeBuffer {
    changes: Mutex<Vec<CellChange>>,
}

pub struct CellChange {
    entity: Entity,
    old_cell: (i32, i32),
    new_cell: (i32, i32),
    x: f32, y: f32, radius: f32,
}
```

**Implementation completed but reverted** - needs more testing/refinement.

---

## Key Learnings

### 1. Allocation Patterns Matter More Than Data Structure Choice
HashMap vs Vec is less important than whether you're allocating/deallocating every frame.

### 2. Cell Size Must Match Perception Range
Cell size >> perception range = massive false positive rate in queries.
Optimal: cell_size ~ perception_range (60m in our case)

### 3. Rayon is Free Performance for Read-Heavy Workloads
Perception queries that only read from shared grid are perfectly parallel.

### 4. Incremental > Full Rebuild
Don't rebuild what hasn't changed. Track changes, apply incrementally.

### 5. "Good Enough" Ordering Beats Precise Sorting
Pre-topological perception was fast because entity ID ordering gave temporal locality.
Cell-based iteration gives spatial locality - "roughly ordered" without sorting cost.

---

## Performance Summary

| Approach | Time | Notes |
|----------|------|-------|
| Dense Vec (baseline) | ~105ms | Full rebuild every tick |
| + Topological distance sorting | 292ms | Correct but expensive heap operations |
| HashMap (naive) | worse | Allocation churn disaster |
| HashMap + occupied tracking | 115ms | Better but still full rebuild |
| + Rayon parallelization | **15ms** | MASSIVE win (will use later) |
| Incremental cell tracking (next) | TBD | Try algorithmic fix first |
| Incremental + Rayon (goal) | <5ms? | Best of both worlds |

---

## Next Steps (Tomorrow)

**Phase A: Incremental Cell Tracking (Try First)**
1. Re-implement CellId component tracking
2. Implement CellChangeBuffer with Mutex for thread-safety
3. Modify movement to track cell changes
4. Create process_cell_changes_system
5. Lazy fill perception (no distance heap)
6. Benchmark at 10K, 15K, 20K creatures
7. Validate behavior correctness (creatures still perceive correctly)

**Phase B: Add Rayon (After Incremental Works)**
8. Layer Rayon parallelization on top of incremental approach
9. Benchmark combined approach
10. Expect best-of-both-worlds performance

---

## Files Modified (For Reference)

When re-implementing:

| File | Change |
|------|--------|
| `perception/components.rs` | Add CellId, CellChangeBuffer |
| `perception/spatial_grid.rs` | Add remove(), update_entity() |
| `perception/systems.rs` | process_cell_changes_system, lazy fill |
| `movement/systems.rs` | Track cell changes after position update |
| `core/simulation.rs` | Reorder systems |
| `creatures/builder.rs` | Add CellId to CritBundle |

---

## References

- ECS-Emma consultation: Cell size mismatch analysis
- Gemini Oracle: Incremental update pattern
- Sprint 15 CLAUDE.md: Rayon parallelization patterns
