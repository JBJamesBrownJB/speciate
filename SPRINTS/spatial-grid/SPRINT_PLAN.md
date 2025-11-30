# Sprint 16: Spatial Grid for Scalable Perception

**Theme:** Break the O(N²) perception bottleneck to enable 150K+ creature populations

**Goal:** Replace brute-force neighbor detection with spatial partitioning, achieving O(N×k) complexity where k ≈ 180 neighbors instead of N comparisons.

**Prerequisites:** Sprint 15 complete (Rayon parallelization, vision split queries)

**Expected Duration:** 5 days

**Target Performance:** 150K creatures @ <45ms tick, perception <10ms

---

## High-Level Phases

### Phase 1: Spatial Grid Data Structure + Full Rebuild
**Outcome:** FxHashMap-based grid with 50m cells, rebuilt every frame

**Key Decisions:**
- Cell size: **50m** (max perception ~35m, use 1.5× for safety)
- Use FxHashMap via `rustc-hash` crate (2-5× faster than std HashMap)
- Store `(Entity, x, y, radius)` in cells to avoid component double-lookup
- **Full rebuild per frame** - simpler, ~1-3ms overhead acceptable
- Replace existing `PerceptionScratchBuffer` with grid (not additive)

**Why not 200m?** At 200m cells with uniform distribution, you'd have ~1,500 creatures/cell = 13,500 comparisons per query. At 50m, ~100 creatures/cell = ~900 comparisons.

### Phase 2: Grid-Accelerated Perception
**Outcome:** Perception system queries nearby grid cells instead of entire population

**Key Decisions:**
- Query radius determines cell range (typically 3×3 = 9 cells)
- Flatten iterator over cells for clean API
- **Use `Res<SpatialGrid>` not `ResMut`** (immutable during perception, enables Rayon)
- Keep existing FOV optimizations (sqrt-free, early-exit) in inner loop

### Phase 3: Rayon Parallelization
**Outcome:** Multi-core perception using read-only grid during parallel queries

**Key Decisions:**
- Collect entity data → parallel grid queries → write-back pattern
- Grid is immutable during perception (no sync overhead)
- Expected 4-8× speedup on multi-core CPUs

### Phase 4: Incremental Updates (DEFERRED - Only If Needed)
**Outcome:** Only move entities when they cross cell boundaries

**Key Decisions:**
- Only implement if Phase 1 profiling shows rebuild > 5ms
- Requires tracking previous cell per entity
- Requires `RemovedComponents<Position>` for despawn handling
- **Higher complexity, marginal gain** - defer until proven necessary

---

## Guidance Notes

### Technical Context

**Why Grid?** O(N²) fails at 25K+ creatures even with perfect parallelization. Must fix algorithmic complexity first.

**Current:** 150K × 150K = 22.5 billion comparisons → 3,825ms even on 8 cores
**With Grid:** 150K × 180 = 27 million comparisons → ~7ms on 8 cores

**Architecture Pattern:** Incremental updates are critical - full grid rebuild every tick would cost ~15ms @ 150K creatures.

### Pre-Sprint Optimizations (Already Implemented)

The following FOV optimizations are already in place and should be preserved:

1. **Sqrt-free FOV check:** `rough_dot² >= cos_half_fov_sq × center_dist_sq` (no sqrt/division in hot path)
2. **Cached `cos_half_fov_sq`:** Pre-computed in `Perception` component at construction
3. **Early-exit for behind:** `if rough_dot <= 0.0 { continue; }` skips ~50% of candidates
4. **Dot product FOV:** Replaced atan2 with dot product comparison

These reduce per-comparison cost significantly. The grid reduces comparison COUNT.

### Cell Size Rationale

**50m chosen based on actual perception range analysis:**

```
PERCEPTION_MULTIPLIER = 10.0  (base_range = body_size × 10)
FOV_RANGE_EXPONENT = 0.4
Max body_size = 2.0 → base_range = 20m
Max FOV bonus (45° narrow) = 1.74× → max_range = 34.8m
```

**Cell size = 50m (1.5× max perception):**
- 3×3 query = 9 cells
- At uniform distribution: ~100 creatures/cell
- ~900 comparisons per query (vs 400M for O(n²))

**Benchmarking guidance:** Start at 50m. If perception ranges change via DNA, validate cell size still appropriate. Add debug assertion:
```rust
debug_assert!(perception.range <= CELL_SIZE, "Perception exceeds cell size");
```

### Current Baseline (Measured)

At 20K creatures with all FOV optimizations (sqrt-free, cached values, early-exit):
- **Perception:** ~50ms (O(n²) = 400M comparisons)
- **Total tick:** ~55-60ms

This confirms O(n²) is the bottleneck - cache optimizations gave only marginal gains.

### Performance Scaling

| Creatures | Current (Parallel) | With Grid (Sequential) | With Grid (Parallel) |
|-----------|-------------------|------------------------|----------------------|
| 20K | 50ms | ~3-5ms | ~1ms |
| 50K | 425ms | ~40ms | ~7ms |
| 150K | 3,825ms | ~120ms | ~20ms |

### Biological Context

Spatial grids mirror real animal cognition - creatures don't evaluate every entity in the world, only those in local proximity. This is both performant AND biologically accurate.

### Future Optimizations (Post-Sprint)

**Stochastic Vision (Sprint 18?):** DNA-driven reaction times. Small creatures check every tick, large creatures every 5-10 ticks. Biologically realistic AND 5× cheaper.

**Hot/Cold Component Split:** Current `Perception` is ~192 bytes (3 cache lines). Could split to:
- `PerceptionConfig` (16 bytes): range, cos_half_fov_sq - read-only hot data
- `PerceptionResults` (separate): neighbors array - write-only
- Delete unused `obstacles: Vec<Entity>` - heap allocation for nothing

---

## Success Criteria

- [ ] Spatial grid supports 150K creatures @ <45ms total tick time
- [ ] Perception system uses <10ms @ 150K (down from 70% of budget)
- [ ] Grid rebuild overhead <5ms @ 150K (full rebuild per tick)
- [ ] Grid memory footprint <200MB @ 150K creatures
- [ ] All existing tests pass (zero behavioral regression)
- [ ] Rayon parallel queries engage all CPU cores
- [ ] Cell size validated via benchmarking (start 50m, tune if needed)
- [ ] Validated at 200K creatures (stretch goal)

## Dependencies (Add to Cargo.toml)

```toml
rustc-hash = "2.0"  # FxHashMap for fast integer hashing
```

## Pre-Sprint Cleanup

- [ ] Delete unused `obstacles: Vec<Entity>` from Perception component (dead heap allocation)
- [ ] Remove `PerceptionScratchBuffer` (replaced by SpatialGrid)
