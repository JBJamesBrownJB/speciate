# Dynamic Range Reduction in Crowds

**Status:** ✅ Implemented
**Location:** `apps/simulation/src/simulation/perception/components.rs:100-104`

## What It Does

When a creature fills its perception neighbors array, its effective perception range is reduced for the next tick. Quadratic falloff: 100% crowded = 50% range.

## Why It Exists

**Computational efficiency:** Reduces candidate count in crowded areas (earlier rejection in perception loop).

**Biological parallel:** Approximates attentional focusing when surrounded - creatures prioritize immediate space over distant scanning.

## Key Parameters

- MAX_PERCEIVED_NEIGHBORS: 8 (triggers reduction when full)
- Maximum reduction: 50% of base range
- Falloff: Quadratic (`1.0 - 0.5 * pressure²`)

## Integration

- `effective_range` field added to Perception component
- `apply_crowd_pressure()` called at start of each perception scan
- Debug overlay shows dynamic range (shrinks visibly in crowds)

## Future Work

See ideas/:
- `crowd-tolerance-dna.md` - DNA-driven crowding tolerance
- `stress-tunnel-vision.md` - Stress-induced perception narrowing
- `energy-vigilance.md` - Energy affects scanning behavior
