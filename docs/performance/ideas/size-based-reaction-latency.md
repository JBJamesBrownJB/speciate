# Size-Based Reaction Latency

**Status:** Idea
**Category:** Simulation Optimizations

## Problem

All creatures react at same 20Hz AI tick rate, ignoring biological size constraints.

## Solution

Reaction delay derived from body length: 100ms (<=1m) to 1000ms (20m creatures). Creatures commit to decisions for their reaction time.

## Benefits

- Enables size-based behavior diversity
- Large creatures slower but deliberate
- No god-tier builds

## Notes

Future sprint after dual-tick architecture.
