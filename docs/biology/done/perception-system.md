# Perception System (Current Implementation)

**Status:** ✅ Implemented
**Location:** `apps/simulation/src/simulation/perception/`

---

## What It Does

Provides FOV-based neighbor detection using a hierarchical spatial grid (L0/L1). Creatures detect nearby entities within their field of view each tick. FOV angle determines both the arc width and perception range through a biological tradeoff (narrow FOV = longer range).

**See also:** `docs/biology/done/dna-fov.md` for detailed FOV documentation.

---

## Key Components

### Perception Component

**Location:** `perception/components.rs:43-49`

**Fields:**
- `fov_angle` - Field of view in radians
- `range` - Detection radius (derived from FOV and body size)
- `cos_half_fov_sq` - Cached for sqrt-free FOV checks
- `cos_half_fov` - Cached for wide FOV checks (sign matters)
- `threshold` - L1 mass threshold for size domination filtering

**Scaling:**
- `from_body_size(body_length)` - Range = body_length × PERCEPTION_MULTIPLIER (at 180° FOV)
- Range varies with FOV: `range = base_range * (180/fov)^0.4`

### NeighborCache Component

**Location:** `perception/components.rs:51-57`

Cold neighbor cache - written by perception, read by avoidance/steering.

**Key features:**
- Fixed-size array (21 neighbors max) - no heap allocations, cache-friendly
- Includes position AND velocity (vx, vy) for TTC-based avoidance
- `MAX_PERCEIVED_NEIGHBORS = 21` (constants.rs)

---

## Perception Update System

**Location:** `perception/systems.rs`

**Algorithm:** O(N) using hierarchical spatial grid with L1 early-exit optimization.

**Process:**
1. Query L0 spatial grid cells within scan radius (always 9 adjacent cells)
2. For each L0 cell, check parent L1 classification
3. Skip L0 cell if L1 is Empty (size domination optimization)
4. For each entity in L0 cell: check distance, FOV, mass threshold
5. Topological sort: select K closest neighbors via `select_nth_unstable`

### L1 Early-Exit Optimization

**Golden Zone:** Giants skip scanning cells containing only mice.

When the L1 cell's total mass is below the creature's perception threshold (5% of body mass), the entire L0 cell scan is skipped. This provides both:
- **Performance win:** Skip entity iteration entirely
- **Biological behavior:** Size domination (large creatures ignore insignificant small ones)

### FOV Check

**Narrow FOV (≤180°):** Uses squared comparison (avoids sqrt)
```
in_fov = rough_dot > 0 && rough_dot² >= cos²(half_fov) × dist²
```

**Wide FOV (>180°):** Falls back to signed comparison with sqrt (handles negative cos)

---

## Size Domination (Phase A)

**Location:** `perception/entity_filter.rs`

Large creatures don't perceive small entities below their threshold.

**Formula:** `threshold = body_mass × PERCEPTION_THRESHOLD_FRACTION` (5%)

**Example:**
- Giant (5m, ~4375kg): threshold = 218kg
- Mouse (1m, ~35kg): below threshold → ignored
- Result: Giant walks through crowd of mice without perceiving them

---

## Integration with Steering System

**Avoidance System** (`steering/avoidance.rs`):
- Uses TTC (Time-to-Collision) based calculation
- Reads neighbor velocity from `NeighborCache`
- Skip diverging paths (Golden Zone optimization)
- See: `docs/biology/done/avoidance-behavior.md`

**Fused Steering** (`steering/system.rs`):
- Single query iteration for all steering behaviors
- Avoidance forces accumulate with wander/seek
- Forces capped to creature's max_accel

---

## Performance Characteristics

| Feature | Benefit |
|---------|---------|
| L1 early-exit | Skip entire L0 cells for sparse areas |
| Size domination | Giants have smaller neighbor sets |
| Fixed 9-cell scan | Consistent L0 query regardless of perception range |
| Topological sort | Only compute K closest (partial sort) |
| Rayon parallel | Multi-core processing |

---

## Current Limitations

### Instant Updates (No Reaction Time)

**Problem:** Perception updates every tick, all creatures simultaneously.

**Biological reality:**
- Small animals: ~68ms reaction time (fast)
- Large animals: ~500ms reaction time (slow)

**Fix:** Planned stochastic vision with per-creature update timing.

---

## Future Work

### Stochastic Vision

Planned reaction-time-gated perception updates to reduce CPU load and add biological realism.

### Frequency Control (Phase C)

Runtime-adjustable Hz for perception system using entity-ID bucketing.

### Additional DNA Vision Genes

- `visual_range_multiplier` - Independent range control (4-25x)
- `neural_speed` - Reaction time modifier (0.5-2.0)

---

## References

- `apps/simulation/src/simulation/perception/` - Implementation
- `docs/biology/done/avoidance-behavior.md` - TTC-based avoidance
- `ABC-SUPER_SPRINT/1-dual-grid.md` - L1 grid design

---

**Last Updated:** 2025-12-23
