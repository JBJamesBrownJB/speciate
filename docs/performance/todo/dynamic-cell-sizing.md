# Dynamic CELL_SIZE Optimization

## Goal
Automatically tune spatial grid `CELL_SIZE` based on creature perception ranges to optimize query performance.

## Team Review

| Reviewer | Verdict | Feedback |
|----------|---------|----------|
| ECS-Emma | BLOCKED naive | O(N log N) sort is 2-5ms at 100K. Use histogram/event-driven. |
| Architect-Andy | APPROVED | Coupling perception→cell_size is correct abstraction. |
| Web Research | CONFIRMED | Industry standard: cell_size ≈ 2× query radius |

## Key Insight

Grid already rebuilds every tick via `rebuild_spatial_grid_system`. The `set_cell_size()` method exists. We just need to calculate the optimal value.

**Formula:** `CELL_SIZE = P75(perception_ranges)` ensures 3×3 queries for 75% of population.

---

## Options Comparison

| Option | Tick Cost | Improvement | Effort | Best For |
|--------|-----------|-------------|--------|----------|
| **1. Histogram** | 0.003ms avg | 10-20% | 1 day | Volatile populations |
| **3. Multi-grid** | ~0.1ms | 30-40% | 3-5 days | Bimodal populations |
| **6. Event-driven** | 0ms | 10-20% | 1 day | Stable ecosystems |

---

## Option 1: Histogram-Based Adaptive

**How:** Every 100 ticks, iterate all creatures O(N), build histogram, find P75, update cell size if >10% change.

**Characteristics:**
- O(1) on 99% of ticks (just decrement counter)
- O(N) every ~5 seconds (0.3ms at 100K)
- 80 bytes memory (histogram fits L1 cache)
- Transparent to user

**Files:** `spatial/systems.rs`, `spatial/adaptive.rs` (new resource)

---

## Option 3: Multi-Resolution Grid

**How:** Maintain two grids simultaneously - fine (15m) for small creatures, coarse (60m) for large. Query appropriate grid based on perception range.

**Characteristics:**
- Optimal for ALL creature sizes
- 2× memory, 2× rebuild cost
- No runtime statistics calculation
- Complex query routing

**Files:** `spatial/multi_grid.rs` (new), perception system updates

---

## Option 6: Event-Driven + Periodic Pause

**How:** Track histogram incrementally on spawn/death events. When distribution shifts >20%, pause sim, show "Optimizing..." overlay, recalculate, resume.

**Characteristics:**
- Zero per-tick overhead
- User sees feedback (intentional, not laggy)
- Natural fit for save/load optimization
- Brief interruption (~100-500ms)

**Trigger conditions:**
1. Population milestone (every 10K creatures)
2. P75 changed >25%
3. Time-based (every 5-10 min)
4. Manual (player menu)

**Files:** `spatial/stats.rs` (new), spawn/death system hooks, Portal overlay component

---

## Expected Improvement

| Metric | Before (40m fixed) | After (adaptive) |
|--------|-------------------|------------------|
| Cells per typical query | 9 (3×3) | 9 (3×3) |
| Cells per large query | 49 (7×7) | 25 (5×5) |
| Perception system | Baseline | 10-20% faster |

---

## Recommendation

**Start with Option 6** - zero tick overhead, natural UX.

**Fallback to Option 1** if population is volatile or pause feels bad.

**Consider Option 3** only if perception is >20% of tick budget with bimodal population.

---

## Sources
- [Game Programming Patterns: Spatial Partition](https://gameprogrammingpatterns.com/spatial-partition.html)
- [Spatial Hashing Tutorial](https://www.gamedev.net/articles/programming/general-and-gameplay-programming/spatial-hashing-r2697/)
