# TTC-Based Collision Avoidance

**Status:** Implemented (ABC Super Sprint - Phase 4)
**Location:** `apps/simulation/src/simulation/steering/avoidance.rs`

---

## What It Does

Creatures steer away from potential collisions using Time-to-Collision (TTC) calculations. The urgency of avoidance scales with how quickly a collision would occur.

Formula:
```
closing_speed = dot(relative_velocity, direction_to_neighbor)
TTC = edge_distance / closing_speed
urgency = (critical_time / TTC).clamp(0, 1)
```

---

## Why It Exists

**Biological realism:** Animals react more urgently to fast-approaching threats than slow-moving ones. A predator lunging requires immediate evasion; a distant creature drifting closer can be avoided gently.

**Performance (Golden Zone):** Creatures moving apart (`closing_speed <= 0`) are skipped entirely. This is both biologically correct (no collision risk) and computationally efficient (fewer force calculations).

---

## Key Behaviors

| Scenario | TTC | Urgency | Avoidance |
|----------|-----|---------|-----------|
| Head-on collision course | Short | High | Strong force |
| Parallel paths, close | Infinite | 0 | No force |
| Diverging paths | N/A | Skip | No calculation |
| Overtaking (same direction) | Long | Low | Gentle steering |

---

## Golden Zone Optimization

**Skip diverging paths:** If `closing_speed <= 0`, creatures are moving apart and no collision is possible. The system skips all avoidance calculations for these pairs.

This optimization:
- Reduces CPU work by ~30-50% in typical simulations
- Is biologically accurate (no need to avoid things moving away)
- Eliminates jittery over-correction for non-threatening neighbors

---

## Key Parameters

| Parameter | Value | Purpose |
|-----------|-------|---------|
| `CRITICAL_TTC_SECONDS` | 1.5s | TTC threshold for maximum urgency |
| Urgency curve | Quadratic | Smooth force ramp (`urgency * urgency`) |

**Location:** `steering/avoidance.rs`

---

## Integration

- Avoidance system receives neighbor velocities via `NeighborCache`
- Forces accumulate into `Acceleration` component
- Runs after perception, before movement integration
