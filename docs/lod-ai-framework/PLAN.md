# LOD AI Framework - Design Document

**Status:** Design Complete, Implementation Deferred
**Original Sprint:** 16 (`feat/sprint-16-lod-ai-framework`)

---

## Overview

This document describes the design for an extensible LOD (Level of Detail) AI framework that reduces computation for off-screen creatures, enabling 150K+ creature simulations.

**Sprint 16 Outcome:** Foundation work completed (cell-culling fix, force multipliers refactor, biological constants audit). LOD implementation deferred to future sprint.

---

## The Framework

### Core Concept

**Every entity has an `Lod` component.** All systems branch on it to decide how much work to do.

```
┌─────────────────────────────────────────────────────────────┐
│                         Entity                              │
├─────────────────────────────────────────────────────────────┤
│  Position, Velocity, Perception, CreatureState, ...         │
│                                                             │
│  Lod  ←── LOD_1 | LOD_2 | LOD_3                            │
└─────────────────────────────────────────────────────────────┘
```

### Three LOD Levels

| Level | When | Work |
|-------|------|------|
| **LOD_1** | Inside viewport AND zoomed in enough | Full fidelity |
| **LOD_2** | Outside viewport OR too zoomed out | Reduced fidelity |
| **LOD_3** | Outside max-zoom viewport bounds | Minimal work |

### How Lod Updates

A dedicated system runs first each tick:

**Decision logic:**
```
if zoom_level > zoomed_out_threshold:
    ALL entities → LOD_2  (too zoomed out for full fidelity)
else:
    if inside current viewport:
        → LOD_1  (player can see details)
    else if inside max-zoom viewport bounds:
        → LOD_2  (could scroll into view)
    else:
        → LOD_3  (would never be visible)
```

**Spatial zones:**
```
┌─────────────────────────────────────────────────────────┐
│                        LOD_3                            │
│   ┌───────────────────────────────────────────────┐    │
│   │              LOD_2                            │    │
│   │      ┌─────────────────────────┐             │    │
│   │      │        LOD_1            │             │    │
│   │      │   (current viewport)    │             │    │
│   │      └─────────────────────────┘             │    │
│   │       (max zoom viewport bounds)             │    │
│   └───────────────────────────────────────────────┘    │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**System order:**
```
update_lod_system       ← Sets Lod for all entities
    ↓
perception_system       ← Reads Lod, branches
avoidance_system        ← Reads Lod, branches
behavior_system         ← Reads Lod, branches
movement_system         ← Reads Lod, branches
```

### How Systems Use Lod

Each system reads `&Lod` and decides what to do:

**Perception example:**
- LOD_1 → 8 neighbors, topological sorting (high fidelity)
- LOD_2 → 4 neighbors, pseudo-random sorting (fast)
- LOD_3 → 1 neighbor, update every 4th tick (minimal)

**Avoidance example:**
- LOD_1 → Full steering vectors per neighbor
- LOD_2 → Binary repulsion from neighbor centroid
- LOD_3 → Skip entirely

**Behavior transitions example:**
- LOD_1 → Check every tick
- LOD_2 → Check every 10 ticks
- LOD_3 → Check every 50 ticks

---

## LOD Tax Analysis

The `update_lod_system` runs every tick. This is the cost of having LOD.

### Per-Entity Work

```
dx = pos.x - viewport.center_x     // 1 subtract
dy = pos.y - viewport.center_y     // 1 subtract
dist_sq = dx*dx + dy*dy            // 2 multiply, 1 add
compare vs 2 thresholds            // 2 comparisons
write Lod enum                     // 1 byte write
```

~10 simple operations per entity.

### Cost Estimates

| Scale | Compute | Memory | Estimated Tax |
|-------|---------|--------|---------------|
| 10K | 100K ops | 90 KB | ~0.05ms |
| 50K | 500K ops | 450 KB | ~0.2ms |
| 150K | 1.5M ops | 1.35 MB | ~0.3-0.5ms |

**Memory breakdown (150K):**
- Read: 150K × 8 bytes (Position) = 1.2 MB
- Write: 150K × 1 byte (Lod) = 150 KB

### Is It Worth It?

Tick budget: **45ms**
LOD tax: **~0.5ms** (worst case at 150K)

**Tax is ~1% of tick budget.**

If LOD saves even 5% on other systems, it's a net win. Expected savings are much higher since ~70% of creatures will be Distant (minimal work).

---

## Why This Design

### Why a component (not marker components)?

Marker components cause **archetype thrashing**. At 150K entities, adding/removing markers = massive memory shuffling every time the camera moves.

An enum field in a permanent component = cheap mutation, zero archetype changes.

### Why per-entity (not computed on-demand)?

- Cheap to store (1 byte per entity = 150KB total)
- Multiple systems can read the same Lod without recomputing
- Enables hysteresis (don't flicker at boundaries)

### Why 3 levels?

Simple and covers all cases:
- **LOD_1:** Must look perfect (player sees it)
- **LOD_2:** Should behave reasonably (might scroll into view, or zoomed out)
- **LOD_3:** Just needs to exist (outside max-zoom bounds, nobody will ever see)

More levels add complexity without clear benefit.

---

## First Optimization: Perception Sorting

### Current State

Perception uses **topological sorting** - cells sorted by distance, finds closest neighbors first.

**Location:** `apps/simulation/src/simulation/perception/systems.rs:161`

### Proposed Change

- **OnScreen:** Keep topological sorting (high fidelity)
- **OffScreen/Distant:** Use pseudo-random sorting (faster)

### Benchmark Plan

Before implementing LOD framework, validate the optimization:

1. Implement pseudo-random neighbor finding
2. Benchmark at 10K, 20K, 50K creatures
3. Measure tick time difference
4. If >30% faster → proceed with LOD framework
5. If not → reconsider approach

---

## Implementation Tasks

### Phase 1: Benchmark Sorting (First)

- [ ] Implement `find_neighbors_pseudorandom()` function
- [ ] Create benchmark comparing topological vs pseudo-random
- [ ] Run at multiple creature scales
- [ ] Document results

### Phase 2: LOD Framework (If benchmarks pass)

- [ ] Create `Lod` enum component (LOD_1, LOD_2, LOD_3)
- [ ] Add `Lod` to creature spawn bundle
- [ ] Create `Viewport` resource (current bounds, zoom level, max-zoom bounds)
- [ ] Implement `update_lod_system` with zoom + position logic
- [ ] Add viewport IPC from frontend (bounds + zoom level)

### Phase 3: Perception Integration

- [ ] Modify perception system to branch on `Lod`
- [ ] LOD_1: topological, 8 neighbors
- [ ] LOD_2: pseudo-random, 4 neighbors
- [ ] LOD_3: pseudo-random, 1 neighbor, tick skipping

### Phase 4: Extend to Other Systems

- [ ] Avoidance system LOD branching
- [ ] Behavior transition LOD branching
- [ ] Movement system LOD branching (if beneficial)

---

## Viewport Communication

Frontend sends viewport info to Rust via NAPI command whenever camera moves/zooms:

```
Frontend (camera change) → NAPI command → Viewport resource → update_lod_system
```

**Viewport resource contains:**
- Current viewport bounds (min_x, max_x, min_y, max_y) — from frontend
- Current zoom level — from frontend

**LOD constants (in Rust):**
- `MAX_ZOOM_BOUNDS` — the largest possible viewport (fixed)
- `ZOOM_THRESHOLD` — zoom level above which all entities are LOD_2

These are constants, not configurable at runtime.

---

## Success Criteria

1. [ ] Benchmark shows pseudo-random sorting is faster
2. [ ] LOD framework implemented with 3 levels
3. [ ] All systems branch on Lod appropriately
4. [ ] 150K creatures at <45ms tick time
5. [x] All existing tests pass (230 unit + 10 spec tests)
6. [x] Behavior is deterministic (same seed = same results)

---

## Future Extensions

The framework supports adding more LOD optimizations later:

- **Tick skipping:** Distant entities update every Nth tick
- **Simplified physics:** Distant entities use cheaper integration
- **Behavior simplification:** Distant entities use simpler state machines
- **Aggregation:** Very distant entities merge into statistical groups

Each optimization just adds more branches in the relevant system's `match lod` block.

---

## Files to Create/Modify

**New files:**
- `apps/simulation/src/simulation/lod/mod.rs` - Lod component, Viewport resource
- `apps/simulation/src/simulation/lod/systems.rs` - update_lod_system

**Modified files:**
- `apps/simulation/src/simulation/perception/systems.rs` - Branch on Lod
- `apps/simulation/src/simulation/creatures/builder.rs` - Add Lod to spawn
- `apps/simulation/src/simulation/core/simulation.rs` - Add lod system to schedule
- `apps/portal/src/...` - Send viewport updates via NAPI

---

## References

- Current perception: `apps/simulation/src/simulation/perception/systems.rs`
- Spatial grid: `apps/simulation/src/simulation/spatial/grid.rs`
- Creature spawning: `apps/simulation/src/simulation/creatures/builder.rs`
