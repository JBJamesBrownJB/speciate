# ABC Super Sprint Summary

**Dates:** 2025-12-28 to 2026-01-18
**Branch:** `update-freq-poc`

## Goal

Performance infrastructure enabling 500K+ creatures with hierarchical perception.

## Delivered

| Phase | Feature | Documentation |
|-------|---------|---------------|
| A | Dual Spatial Grid (L0/L1) | `docs/performance/done/hierarchical-spatial-grid.md` |
| 4 | TTC-Based Avoidance | `docs/biology/done/ttc-avoidance.md` |
| C | Frequency Control | `docs/performance/done/system-update-frequency.md` |
| 5+6 | L1 Cone Perception | `docs/biology/done/l1-cone-perception.md` |

## Deferred (Not Required for 500K Goal)

| Phase | Feature | Reason |
|-------|---------|--------|
| H | L2 Strategic Grid | Cone-based L1 made it unnecessary |
| B | Drive Simplex | Deferred to future sprint |

## Key Achievements

- **500K creatures validated** at acceptable tick latency
- **L1 cells match FOV cone exactly** - gray cells track teal cone
- **Size domination working** - giants ignore mice (asymmetric perception)
- **TTC avoidance eliminates jitter** - smooth collision avoidance
- **Dead L2 code removed** - infrastructure was built but not needed

## Technical Highlights

### Hierarchical Spatial Grid (Phase A)

Two-level grid hierarchy for efficient perception:
- L0 (20m): Entity scanning with PerceptionProxy data
- L1 (60m = 3x3 L0): Area awareness with BioSignature aggregation

BioSignature tracks `total_mass`, `max_size`, `creature_count` per L1 cell. Enables early-exit when all creatures in a cell are below perception threshold.

### TTC Avoidance (Phase 4)

Time-to-Collision based urgency replaces distance-only avoidance:
- Fast approach = high urgency = strong avoidance
- Diverging paths skipped entirely (Golden Zone optimization)
- Eliminates over-correction jitter for non-threatening neighbors

### Frequency Control (Phase C)

Runtime-adjustable update frequency for cognitive systems:
- Entity-ID bucketing with power-of-2 divisors (bitwise AND)
- Perception and behavior systems throttleable
- Steering throttling removed (caused jerky movement)

### L1 Cone Perception (Phases 5+6)

Replaced fixed 8-cell ring scan with actual cone intersection:
- Variable cell count based on perception range and FOV
- Uses same `is_in_fov()` function as L0 entity perception
- Visualization matches biological perception

## L2 Grid Decision

L2 infrastructure (180m cells, 3x3 L1) was fully implemented but ultimately not needed:
- Cone-based L1 perception naturally scales with perception range
- Large creatures perceive more L1 cells due to longer range
- L2 strategic layer adds complexity without sufficient benefit

**Decision:** Remove L2 dead code as part of sprint closure.

## Test Results

All tests passing. No compiler warnings.

## Files Changed

### Created
- `docs/performance/done/hierarchical-spatial-grid.md`
- `docs/biology/done/ttc-avoidance.md`
- `docs/biology/done/l1-cone-perception.md`

### Modified (L2 Removal)
- `apps/simulation/src/simulation/spatial/constants.rs` - Removed L2_CELL_SIZE
- `apps/simulation/src/simulation/spatial/coarse_grid.rs` - Removed L2 fields/methods/tests
- `apps/simulation/src/simulation/spatial/hierarchical.rs` - Removed L2 methods/tests

### Deleted
- `ABC-SUPER_SPRINT/` folder (all planning docs)
