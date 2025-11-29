# Sprint 16: Spatial Grid for Scalable Perception

**Theme:** Break the O(N²) perception bottleneck to enable 150K+ creature populations

**Goal:** Replace brute-force neighbor detection with spatial partitioning, achieving O(N×k) complexity where k ≈ 180 neighbors instead of N comparisons.

**Prerequisites:** Sprint 15 complete (Rayon parallelization, vision split queries)

**Expected Duration:** 5 days

**Target Performance:** 150K creatures @ <45ms tick, perception <10ms

---

## High-Level Phases

### Phase 1: Spatial Grid Data Structure
**Outcome:** FxHashMap-based grid with 200m cells, entity position caching, and world-to-cell coordinate mapping

**Key Decisions:**
- Cell size: 200m (2× max perception range for safety margin)
- Use FxHashMap (2-5× faster than std HashMap for integer keys)
- Cache positions to avoid redundant component queries

### Phase 2: Incremental Grid Updates
**Outcome:** Efficient system that only moves entities when they cross cell boundaries (~0.8% of creatures per tick)

**Key Decisions:**
- Track previous cell per entity to detect boundary crossings
- Batch insertions/removals for cache efficiency
- Run BEFORE perception system in schedule

### Phase 3: Grid-Accelerated Perception
**Outcome:** Perception system queries 3×3 grid cells instead of entire population, reducing comparisons by 833×

**Key Decisions:**
- Query radius determines cell range (typically 9 cells)
- Flatten iterator over cells for clean API
- Keep stochastic vision compatible (if implemented in Sprint 18)

### Phase 4: Rayon Parallelization
**Outcome:** Multi-core perception using read-only grid during parallel queries

**Key Decisions:**
- Collect entity data → parallel grid queries → write-back pattern
- Grid is immutable during perception (no sync overhead)
- Expected 4-8× speedup on multi-core CPUs

---

## Guidance Notes

### Technical Context

**Why Grid?** O(N²) fails at 25K+ creatures even with perfect parallelization. Must fix algorithmic complexity first.

**Current:** 150K × 150K = 22.5 billion comparisons → 3,825ms even on 8 cores
**With Grid:** 150K × 180 = 27 million comparisons → ~7ms on 8 cores

**Architecture Pattern:** Incremental updates are critical - full grid rebuild every tick would cost ~15ms @ 150K creatures.

### Performance Scaling

| Creatures | Current (Parallel) | With Grid (Sequential) | With Grid (Parallel) |
|-----------|-------------------|------------------------|----------------------|
| 50K | 425ms | ~40ms | ~7ms |
| 150K | 3,825ms | ~120ms | ~20ms |

### Biological Context

Spatial grids mirror real animal cognition - creatures don't evaluate every entity in the world, only those in local proximity. This is both performant AND biologically accurate.

---

## Success Criteria

- [ ] Spatial grid supports 150K creatures @ <45ms total tick time
- [ ] Perception system uses <10ms @ 150K (currently 70% of budget)
- [ ] Grid update overhead <2ms (incremental updates only)
- [ ] All existing tests pass (zero behavioral regression)
- [ ] Rayon parallel queries engage all CPU cores
- [ ] Validated at 200K creatures (stretch goal)
