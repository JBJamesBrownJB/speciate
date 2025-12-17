# Modulus Perception Spreading

## Goal

Reduce CPU load by having each creature skip expensive systems on alternating ticks, while avoiding "hot frames" where all creatures update simultaneously.

## How It Works

**Slice assignment:** Each creature gets a `slice_id` (0 to SLICE_COUNT-1) assigned on spawn by cycling through the range. This guarantees even distribution across slices.

**Frame cycling:** A global frame counter cycles 0 to SLICE_COUNT-1 each tick.

**Early-out check:** Systems check if `slice_id == frame_counter`. If not, skip that creature this tick.

```
SLICE_COUNT = 2

Creature A (slice_id=0): updates on frames 0, 2, 4, 6...
Creature B (slice_id=1): updates on frames 1, 3, 5, 7...

Result: ~50% of creatures processed per frame, load evenly distributed
```

## Target Systems

- **Perception** - expensive spatial queries
- **Behavior transitions** - state machine logic

NOT movement physics (needs every-frame for smooth motion).

## Why Not Distance-Based LOD?

Distance-based AI LOD (near=fast, far=slow) works for player-centric action games but causes problems for ecosystem simulation:

- Distant creatures still need to hunt, flee, eat - inconsistent update rates cause ecological drift
- Camera position moves constantly in god-view - creatures oscillate between zones
- Already have viewport culling from Sprint 16

Slice-based spreading is camera-independent and treats all creatures equally.

## Key Properties

- **Even load distribution** - no hot/cold frames
- **Deterministic** - cycling spawn counter, not random
- **Tunable** - single constant controls skip rate (SLICE_COUNT=2 halves load, =4 quarters it)
- **Stacks with MAX_NEIGHBORS** - this is orthogonal to neighbor count limits

## Implementation Location

- Constant: `apps/simulation/src/simulation/creatures/constants/performance.rs`
- Component: `UpdateSlice { id: u8 }`
- Frame counter: Resource cycling each tick
