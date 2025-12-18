# Perception Time-Slicing

**Status:** Implemented
**Location:** `apps/simulation/src/simulation/perception/systems.rs`

## What It Does

Distributes expensive per-creature computations across multiple frames. Instead of all creatures running perception every tick, creatures are divided into slices that take turns updating.

With `UPDATE_SLICE_COUNT = 2`:
- Tick 0: Slice 0 creatures update perception
- Tick 1: Slice 1 creatures update perception
- Tick 2: Slice 0 again...

Each creature runs perception every 2nd tick (50% load reduction per tick).

## Why It Exists

**Performance scaling:** Perception is O(N) per creature with spatial grid. At 500K creatures, running all every tick is prohibitive. Time-slicing spreads the load.

**Biological plausibility:** Real animals don't have instant, continuous perception. Reaction times range from ~68ms (small prey) to ~500ms (large mammals). Time-sliced updates approximate this latency.

**Movement remains smooth:** Only perception and behavior transitions are sliced. Movement integration runs every tick for all creatures, maintaining visual smoothness.

## Key Parameters

| Parameter | Location | Value |
|-----------|----------|-------|
| `UPDATE_SLICE_COUNT` | `constants/performance.rs` | 2 |

Slice assignment: `creature_id % UPDATE_SLICE_COUNT` (see `builder.rs:230`)

## Which Systems Are Sliced

**Sliced (run 1/N ticks per creature):**
- `update_perception_system` - Neighbor detection
- `behavior_transition_system` - Brain state machine updates

**Not sliced (run every tick for all):**
- `integrate_motion_system` - Position/velocity integration
- `update_steering_system` - Force calculation
- Spatial grid rebuild

## Integration

The `UpdateSlice` component stores each creature's slice ID:
- Assigned deterministically at spawn via modulus
- Properly serializable for save/load (see `update_slice.rs`)
- Used as early-return filter in perception/behavior systems

Current slice computed from `PhysicsTick`: `(tick % UPDATE_SLICE_COUNT) as u8`

## Performance Impact

Milestone: 500K creatures with time-slicing enabled (commit c0c989e)

Tuning: Higher `UPDATE_SLICE_COUNT` reduces per-tick load but increases perception staleness. Current value of 2 balances responsiveness and performance.

## Future Work

**Stochastic vision:** DNA-driven reaction times will add per-creature variation rather than uniform slicing.

**Adaptive slicing:** Could dynamically adjust slice count based on creature density or frame budget.

---

**Last Updated:** 2025-12-18
