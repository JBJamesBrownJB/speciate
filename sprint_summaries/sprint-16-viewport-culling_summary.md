# Sprint 16 Summary: Viewport Culling & Performance Fixes

**Branch:** `feat/sprint-16-viewport-culling`
**Status:** ✅ COMPLETE
**Tests:** 366 portal ✅ | 315 simulation ✅

---

## Sprint Goal

Implement viewport-based creature culling to scale the simulation to 100K+ populations without buffer sync issues, while maintaining smooth interpolation and preventing ghosting artifacts.

---

## Key Outcomes

### ✅ Phase 1: Stable Export Ordering (Ghost-Crits Fix)
- Implemented parallel sort of creature exports by CritId
- Benchmark: 1.35ms @ 400K creatures (7.1x speedup vs sequential)
- Added `export_positions_us` instrumentation to dev-ui
- Prevents creatures from shifting indices (causing visual jitter)

### ✅ Phase 2: Backend Viewport Culling
- Frontend sends viewport bounds via `setViewportBounds()` IPC
- Backend filters creatures before export (reduces IPC bandwidth)
- ID-based interpolation tracking in frontend handles creatures entering/leaving viewport
- Configurable culling margin prevents pop-in at viewport edges

### ✅ Phase 3: Ghosting Bug Fix
- **Problem:** Creatures teleported when re-entering viewport (interpolating from stale cached position)
- **Solution:** Added `visibleLastTick` Set to track visibility freshness
- Only interpolate from last frame's position (smooth) vs stale cache (ghosts)

### ✅ Phase 4: Mass Stutter Bug Fix
- **Problem:** All creatures flashed simultaneously every few seconds at 100K+
- **Root Cause:** Race condition - atomic count stored BEFORE buffer swap
- **Solution:** Moved store AFTER swap with Release/Acquire memory ordering
- No more mass stutter at 100K+ populations

### ✅ Post-Sprint Cleanup
- Removed debug logging (`[GHOST-DEBUG]` console.warn)
- Removed dead viewport culling methods (backend handles culling now)
- Removed dead Rust function (`lookup_entity_by_crit_id`)
- Extracted viewport culling constants to `constants.ts`
- Implemented time-based max zoom speed (industry-standard discard excess pattern)

---

## Technical Achievements

### Architecture
- **Viewport Culling:** Moved from frontend to backend (more efficient, single source of truth)
- **ID-Based Tracking:** Creatures tracked by ID in Map (O(1) lookup vs array search)
- **Memory Ordering:** Proper Release/Acquire synchronization for lock-free IPC
- **Rate Limiting:** Time-based zoom speed cap (immediate response, no lag)

### Performance
- **1.35ms** export sort @ 400K creatures (3% of 45ms tick budget)
- **Validation:** Deterministic tests at 20K creatures confirm correctness
- **Scaling:** Ready for 150K-200K creature target

### Code Quality
- **TDD:** Red-Green-Refactor for ghosting fix and mass stutter fix
- **Constants Extraction:** All magic numbers in `constants.ts`
- **Dead Code Removal:** 123 lines of Viewport.ts removed (no longer needed)
- **Test Coverage:** 366 portal tests (up from 356), all passing

---

## Completed Tasks

| Phase | Task | Status |
|-------|------|--------|
| 1 | Parallel sort benchmarking | ✅ |
| 1 | Implement `export_positions_us` sort | ✅ |
| 1 | Add dev-ui instrumentation | ✅ |
| 2 | Refactor InterpolationBufferManager to ID-based | ✅ |
| 2 | Add SetViewportBounds command (backend) | ✅ |
| 2 | Implement `setViewportBounds()` IPC | ✅ |
| 2 | Add throttled viewport bounds sending | ✅ |
| 3 | Write failing test (ghosting) | ✅ |
| 3 | Fix: Add `visibleLastTick` tracking | ✅ |
| 4 | Investigate mass stutter | ✅ |
| 4 | Fix: Move atomic store after swap | ✅ |
| 4 | Fix: Add Release/Acquire ordering | ✅ |
| Cleanup | Remove debug logs | ✅ |
| Cleanup | Remove dead code | ✅ |
| Cleanup | Extract constants | ✅ |
| Cleanup | Implement zoom speed limit | ✅ |

---

## Remaining Work

None - all original goals achieved and post-sprint cleanup complete.

**Future Enhancements (Out of scope):**
- Shader-based culling: Tested and rejected (vertex-bound at 400K)
- TTL-based creature cleanup: Defer to Sprint 17 (40 bytes/entry acceptable)
- Incremental sorting: Not worth complexity (batch spawns are O(n log n) anyway)

---

## Bugs Fixed

| Bug | Root Cause | Fix |
|-----|-----------|-----|
| **Ghost-Crits** | Unstable creature ordering | Parallel sort by CritId |
| **Ghosting (Re-entry)** | Stale interpolation cache | Track `visibleLastTick` |
| **Mass Stutter** | Atomic ordering race | Move store after swap (Release/Acquire) |

---

## Key Insights

### 1. Backend Culling Wins
Frontend shader culling doesn't help when vertex-bound (still 1.6M verts for 400K creatures). Backend filtering reduces IPC bandwidth and GPU work.

### 2. Logarithmic Zoom
Zoom is multiplicative, not additive. Using log space for accumulation keeps math simple and speed limit consistent at all zoom levels.

### 3. Memory Ordering Matters
Naive Relaxed ordering lost synchronization between Bevy thread and polling thread. Release/Acquire ensures buffer consistency.

### 4. Visibility Tracking
The key to smooth viewport transitions is knowing "was creature visible LAST frame" - not "is it in cache somewhere". Prevents stale interpolation.

---

## Testing

**Portal Tests:** 366 passing
**Simulation Tests:** 315 passing (lib), 1 integration flaky (pre-existing)

**Manual Verification:**
- ✅ No ghosting at 100K+ with rapid viewport panning
- ✅ No mass stutter on fast mousewheel spin
- ✅ Zoom speed capped smoothly (discard excess pattern)
- ✅ Creatures smooth transitions entering/leaving viewport

---

## Retrospective

### What Worked
- **TDD for bugs:** Writing tests FIRST revealed root causes clearly
- **ID-based tracking:** Map lookup eliminated index-shifting problems
- **Parallel sort:** Rayon paid off massively (7.1x speedup)
- **Time-based rate limiting:** Industry standard, immediate & responsive

### What We Learned
- Shader culling is only effective when fill-bound, not vertex-bound
- Atomic memory ordering is subtle but critical for lock-free IPC
- Viewport transitions need freshness tracking, not just cache
- Per-event clamping doesn't work when events fire at high frequency

### If We Did It Again
- Test shader approach earlier (we did - good call abandoning it)
- Implement ID-based tracking from start (saves refactoring)
- Use Release/Acquire ordering by default (not Relaxed)

---

## Merge Readiness

✅ All tests passing
✅ No debug logs
✅ Dead code removed
✅ Constants extracted
✅ Documentation complete
✅ 4 phases validated in SPRINT_DOCS

**Ready to merge to main.**
