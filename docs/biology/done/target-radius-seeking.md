# Target Radius Seeking (Edge-to-Edge Arrival)

**Status:** ✅ Implemented
**Location:** `apps/simulation/src/simulation/creatures/behaviors/seek.rs`

## What It Does

Creatures stop at the **edge** of targets rather than racing to the center point. A creature seeking a food patch (radius 2m) will stop when its body edge touches the patch edge, creating realistic arrival behavior for area-based resources.

**Example:**
- Creature (0.5m radius) → Food patch (2.0m radius)
- Center distance: 10m
- **Edge distance:** 10m - 0.5m - 2.0m = 7.5m (actual distance to arrival)
- Creature stops when edge distance reaches 0.05m tolerance

## Why It Exists

**Biological realism:**
- Animals don't seek the center of resources - they stop when they can access them
- Drinking from water: Stop when mouth reaches edge, not center of lake
- Grazing: Stop at meadow boundary, don't walk to the center
- Resting zones: Lie down when reaching the boundary

**Gameplay foundation:**
- Enables resource competition (multiple creatures can't occupy same space)
- Sets up occupancy detection (target has limited capacity)
- Allows area-based targets (water holes, food patches, rest zones)

## Key Mechanics

### Target Component

**Location:** `apps/simulation/src/simulation/creatures/components/perception.rs:8-22`

Targets now have a `radius` field in addition to position:
- `Target::new(x, y)` - Point target (radius = 0.0)
- `Target::with_radius(x, y, radius)` - Area target

### Edge Distance Calculation

**Formula:** `edge_distance = center_distance - self_radius - target_radius`

**Location:** `seek.rs:32-37`

This calculation ensures large creatures stop further from center than small creatures, maintaining proper edge alignment.

### Three-Stage Arrival

**1. Slow Zone (30× personal space from target edge)**
- Speed decreases exponentially as creature approaches
- Formula: `desired_speed = max_speed × e^(k×ratio) / e^k` where k=1.5
- Prevents overshoot while maintaining speed far from target
- See `SLOW_ZONE_MULTIPLIER` in `movement/constants.rs`

**2. Pounce (< 0.1m edge distance, moving slowly)**
- Snap to target when very close and speed < 5.5 m/s
- Prevents endless creeping from asymptotic deceleration
- Transitions to Catatonic state (stopped)
- Clean visual: feels intentional, not numerical instability

**3. Emergency Brake (< 0.05m edge distance)**
- Hard counter-force (70N) prevents overshoot
- Applied when arrival threshold breached
- Stops perpetual circling from momentum

## Constants

**See:** `apps/simulation/src/simulation/movement/constants.rs` (SEEKING struct)

| Constant | Purpose |
|----------|---------|
| `arrival_tolerance` | Physics stability tolerance |
| `pounce_distance` | Snap-to-target distance |
| `pounce_speed` | Max speed for pounce |
| `brake_force` | Emergency brake force |
| `slow_zone_decay` | Exponential deceleration factor |
| `SLOW_ZONE_MULTIPLIER` | Slow zone size multiplier |

## Visual Example: Multiple Creatures

```
      ┌─────────────────┐
      │                 │
   C1 →  Target (2.0m)  ← C2
      │                 │
      └─────────────────┘
           ↑
           C3

All arrive at target edge simultaneously.
Currently: All can occupy the same edge (overlap).
Future: Occupancy detection → transition to Waiting state.
```

## Integration with Other Systems

**Movement Physics:** Seek system writes to `Acceleration`, physics integration applies forces

**State Transitions:** Seeking → Catatonic when arrived (pounce or brake complete)

**Size Component:** Uses `size.radius()` for automatic scaling - large creatures stop further from center

## Future Work

### Occupancy Detection (Planned)

**Problem:** Multiple creatures can currently overlap at the same target edge.

**Solution:**
1. Check if another creature is within `target.radius + other.radius` of target center
2. If occupied, transition to `BehaviorMode::Waiting`
3. Periodically retry or select alternative target

**Implementation location:** Planned for `apps/simulation/src/simulation/creatures/behaviors/transitions.rs`

### Target Priority System (Future)

- Select nearest unoccupied target
- Fall back to next-best if primary occupied
- Abandon target if wait time exceeds threshold

### Resource Depletion (Future)

- Food patch radius shrinks as biomass consumed
- Eventually disappears (radius → 0)
- Creatures must seek new targets or starve

## Design Rationale

**Why 0.05m tolerance?**
- At 22.2Hz tick rate, creatures move up to 0.25m per frame
- 0.05m tolerance prevents oscillation from overshooting
- 2× smaller than minimum collision detection threshold (0.5m)

**Why pounce mechanic?**
- Without it: Creatures approach asymptotically, never quite stopping
- With it: Clean snap when close and slow enough
- Better visual polish and gameplay feel

**Why edge-to-edge vs center-to-center?**
- Large food patches should be accessible from edge, not just center
- Multiple creatures can access different edges of same resource
- Foundation for occupancy/capacity mechanics

## References

- `docs/biology/done/movement-physics.md` - Arrival & deceleration algorithms
- `apps/simulation/src/simulation/movement/constants.rs` - Seeking constants

---

**Last Updated:** 2025-11-29
