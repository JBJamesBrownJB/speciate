# Sprint Summary: Perception FOV Optimizations

**Branch:** `feat-perception-overlay-diagnostics`
**Date:** 2025-11-30
**Focus:** Optimizing perception FOV checks for 20K creatures

---

## Sprint Goal

Optimize the perception system's Field of View (FOV) calculations to reduce per-comparison cost at scale.

---

## Key Outcomes

### Implemented Optimizations

| Optimization | Before | After | Savings |
|--------------|--------|-------|---------|
| Dot product FOV check | atan2 + normalize_angle | dot product comparison | 80ms → 52ms |
| Early-exit for behind | None | `rough_dot <= 0.0` skip | 52ms → 47-49ms |
| Sqrt-free FOV comparison | `sqrt() + 2 divisions` | `rough_dot² >= cos_half_fov_sq × dist_sq` | ~50ms final |
| Cached `cos_half_fov` | Computed every frame | Pre-computed at construction | Minor |
| Cached `cos_half_fov_sq` | N/A | Added for sqrt-free check | Enables sqrt elimination |

### Additional Work

- **Turn rate limiting:** Implemented `MAX_TURN_RATE_RAD` to limit velocity rotation per frame
- **`normalize_angle()` function:** Added O(1) angle normalization to `math/vector_ops.rs`
- **Sprint 16 spatial grid plan:** Updated with team feedback (50m cells, full rebuild, type-aware queries)

---

## Files Modified

| File | Change |
|------|--------|
| `perception/components.rs` | Added `cos_half_fov`, `cos_half_fov_sq` fields to Perception |
| `perception/systems.rs` | Dot product FOV, early-exit, sqrt-free comparison |
| `movement/constants.rs` | Added `MAX_TURN_RATE_RAD` |
| `movement/systems.rs` | Turn rate limiting in integrate_motion_system |
| `math/vector_ops.rs` | Added `normalize_angle()` function |
| `SPRINTS/spatial-grid/SPRINT_PLAN.md` | Updated with team review feedback |

---

## Performance Results

**At 20K creatures:**
- **Before FOV optimizations:** ~80ms perception
- **After all optimizations:** ~50ms perception
- **Improvement:** 37.5% reduction in perception time

**Key insight:** O(n²) is still the bottleneck. These optimizations reduce per-comparison cost but don't change algorithmic complexity. Spatial grid (Sprint 16) is required for further improvement.

---

## Lessons Learned

1. **LLVM doesn't always optimize as expected:** Manual sqrt elimination via squared comparison was necessary
2. **Dot product > atan2:** For FOV checks, dot product is simpler and faster
3. **Early-exit matters:** Skipping entities behind (~50% of candidates) provides measurable gains
4. **O(n²) is the real enemy:** Per-comparison optimizations help but spatial indexing is required for scale

---

## Next Steps (Sprint 16: Spatial Grid)

The spatial grid sprint plan has been updated and is ready:
- Cell size: 50m (based on max perception ~35m)
- Full rebuild per frame (simpler, fast enough)
- `SpatialEntityType` enum for type-aware queries (food/creature/obstacle)
- Rayon parallelization after grid is working

Expected improvement: 50ms → 3-5ms with spatial grid alone.

---

## Team Consultations

- **ECS-Emma:** Cell size analysis (50m not 200m), SoA patterns, hot/cold split potential
- **Architect-Andy:** Grid as Resource, incremental update bugs, Rayon thread-safety
- **Instrumentation-Ian:** Cache miss analysis, theoretical performance floor

---

## Retrospective

**What went well:**
- Systematic optimization approach (measure → optimize → measure)
- Team consultation caught critical issues (200m cell size was wrong!)
- FOV math refactor is clean and maintainable

**What could improve:**
- Started with micro-optimizations before addressing O(n²) - should have done spatial grid first
- More profiling data would help (perf counters for cache misses)

**Decision:** Spatial grid is the priority for Sprint 16. FOV optimizations provide foundation for fast inner loop once comparison count is reduced.
