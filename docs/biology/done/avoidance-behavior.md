# Avoidance Behavior - TTC-Based Collision Prevention

**Status:** ✅ Implemented (ABC Super Sprint, Phase 4)
**Location:** `apps/simulation/src/simulation/creatures/steering/avoidance.rs`

---

## What It Does

Creatures steer away from imminent collisions using Time-to-Collision (TTC). The closer and faster a collision approaches, the stronger the avoidance force. Paths that are diverging or parallel trigger no avoidance at all.

**Result:** Smooth, physics-based collision prevention. Creatures flowing around each other naturally without jittering or over-reaction.

---

## Why TTC (Not Distance-Based)

### The Old Problem

Distance-based avoidance had a critical flaw:
- Perception detected neighbors at ~8m
- Avoidance only triggered at ~0.25m edge distance
- Not enough time/distance to steer around large obstacles

**Root cause:** The range gate filtered out neighbors perception had already detected.

### The TTC Solution

Physics question: **Given closing speed and distance, is there enough time to steer away?**

TTC naturally handles what distance-based couldn't:
- Fast approach + close = short TTC = high urgency = strong force NOW
- Slow approach + far = long TTC = low urgency = gentle steering
- Moving apart = infinite TTC = skip entirely (Golden Zone)

---

## Core Algorithm

**Location:** `steering/avoidance.rs:83` (`calculate_avoidance`)

```
closing_speed = -(relative_velocity · direction_to_neighbor)

if closing_speed <= 0:
    return 0  // Moving apart - no collision risk (Golden Zone)

time_to_collision = edge_distance / closing_speed
urgency = (critical_time / TTC).clamp(0, 1)
force = urgency² × max_accel × direction_away
```

The quadratic urgency ramp (`urgency²`) creates smooth force scaling - gentle at long TTC, aggressive when collision is imminent.

---

## Key Parameters

**Location:** `apps/simulation/src/simulation/creatures/constants/behavior.rs`

| Parameter | Value | Purpose |
|-----------|-------|---------|
| `CRITICAL_TTC_SECONDS` | 2.0 | TTC threshold for max urgency |

**Tuning notes:**
- Too short → collisions happen (not enough reaction time)
- Too long → creatures over-react to distant neighbors
- 2.0 seconds provides good balance for current creature speeds

---

## Golden Zone Optimization

**Skip calculation entirely when paths are diverging.**

This is both a performance win AND biologically accurate:
- **Performance:** Creatures only calculate avoidance for actual collision threats
- **Biology:** Real animals don't react to things moving away from them

**Implementation:** `closing_speed <= 0` check at `avoidance.rs:117`

---

## Edge Cases Handled

| Scenario | TTC | Urgency | Behavior |
|----------|-----|---------|----------|
| Head-on collision course | Short | High | Strong avoidance |
| Parallel paths, close | ∞ | 0 | No force (won't collide) |
| Stationary obstacle ahead | Finite | Based on my speed | Avoidance (I'm approaching) |
| Overtaking (same direction) | Long | Low | Gentle steering |
| Moving apart | Negative closing | 0 | Skip entirely |
| Already overlapping | Very short | Max | Strong separation force |

---

## Integration with Steering System

**Location:** `apps/simulation/src/simulation/creatures/steering/system.rs:90-118`

Avoidance is computed from the fused steering system alongside wander/seek. Forces accumulate additively then get capped to `max_accel`.

**Key integration detail:** Neighbor velocity (vx, vy) is passed from `NeighborCache` to enable TTC calculation. This was the key change from Phase 4 - the old system only had position/radius.

---

## Biological Rationale

### Real Animal Collision Avoidance

Animals use motion cues (optical flow, looming) to predict collisions:
- **Looming response:** Rate of size expansion predicts time-to-impact
- **Tau variable:** τ = distance / closing_velocity (exactly TTC!)
- **Neural basis:** Looming-sensitive neurons in optic tectum (fish, birds) and superior colliculus (mammals)

**Examples:**
- Pigeons time their escape based on τ, not distance
- Fish schools maintain spacing through velocity matching and TTC estimation
- Locusts have dedicated looming detectors that trigger jump reflex

### Why Quadratic Urgency

The urgency² scaling matches biological response curves:
- **Startle threshold:** Response only triggers past a threshold
- **Graded response:** Force increases non-linearly as threat approaches
- **Smooth transitions:** No hard boundaries, natural blending

---

## Future Work

### DNA Integration (Planned)

**Gene: `reaction_time` (0.5-3.0 seconds)**
- Low: Nervous, hair-trigger (prey species)
- High: Calm, late-reactor (confident predators)

**Gene: `collision_tolerance` (0.0-1.0)**
- Low: Gives wide berth, conservative paths
- High: Cuts close, takes risks

### Size-Asymmetric TTC

Currently all creatures use same `CRITICAL_TTC_SECONDS`. Future work could scale by size:
- Large creatures need more time to turn (higher TTC threshold)
- Small creatures are more agile (can use lower TTC)

---

## Related Systems

- **Perception:** Provides `NeighborCache` with neighbor positions and velocities
- **Steering:** Avoidance forces accumulate with wander/seek in fused system
- **Movement:** Integrated forces applied in `integrate_motion_system`

---

## References

- Lee, D.N. (1976) - Tau variable and visual collision detection
- Fotowat, H. & Bhattacharya, S. (2008) - Looming-sensitive neurons in locust
- Reynolds, C.W. (1987) - Boids flocking (separation behavior)
- `sprint_summaries/abc-super-sprint_summary.md` - ABC Super Sprint summary (design record)

---

**Last Updated:** 2025-12-23
