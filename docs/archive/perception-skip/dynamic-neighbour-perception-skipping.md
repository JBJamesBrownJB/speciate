# Dynamic Neighbour Perception Skipping

**Status:** Implemented
**Location:** `apps/simulation/src/simulation/perception/systems.rs`, `components.rs`

## What It Does

Creatures at neighbour capacity (7) skip perception every other tick.

## Rationale

- **50% reduction** - Crowded creatures perceive half as often
- **Self-regulating** - Optimization triggers exactly where performance problems occur (dense clusters)
- **Minimal stale data** - At most 1 tick old, neighbours haven't moved far
- **No tuning** - Simple boolean toggle, no threshold constants

## Implementation

Added `skip_ticks_remaining: u8` counter to `NeighborCache` component:

- `consume_skip()` - If counter > 0, decrements and returns true
- `schedule_skip(ticks)` - Sets counter to specified number of ticks
- `should_skip()` - Returns true if counter > 0 (for testing)

**Tuning constant:** `PERCEPTION_SKIP_TICKS` in `creatures/constants/perception.rs`
- Default: 1 (skip every other tick = 50% reduction)
- Value 2 = skip 2 ticks per perception (66% reduction)
- Higher values = more performance, more stale data

**Tick sequence with PERCEPTION_SKIP_TICKS = 2:**
1. Tick 0: perception runs, fills to 7, counter = 2
2. Tick 1: counter=2 → decrement to 1, skip
3. Tick 2: counter=1 → decrement to 0, skip
4. Tick 3: counter=0 → perception runs, if full → counter = 2
5. Repeat...

## Test Coverage

14 tests covering:
- Component tests (4): flag initialization, consume/schedule API, clear() preservation
- System tests (10): full/partial cache, zero neighbors, behavior check order, alternation, parallel safety

## Considerations

- **1-tick stale data** - Neighbours list is 1 tick old during skip. Acceptable since creatures don't move far in one tick.
- **Fairness** - Crowded creatures react slightly slower than isolated ones.

## Future Enhancement

Could extend to skip multiple ticks based on density:
- 7 neighbours (MAX): skip 1 tick
- Could add graduated skipping if 1-tick skip proves insufficient

Start simple, measure impact first.
