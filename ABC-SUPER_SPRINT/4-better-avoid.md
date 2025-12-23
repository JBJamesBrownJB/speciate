# Phase 4: Better Avoidance (Anti-Collision)

## Scope

**This is ONLY about preventing collisions.** Flee, chase, predator/prey dynamics come later in Phase B drives.

Avoidance = "Don't hit things"

---

## Problem

Small creatures don't avoid large creatures until too late.

- Perception detects giant at ~8m
- Avoidance only kicks in at ~0.25m edge distance
- Not enough time/distance to steer around

**Root cause:** `max_interaction_distance` check in avoidance.rs filters out neighbors that perception already detected.

---

## Solution: Time-to-Collision (TTC)

The physics question: **Given closing speed and distance, is there enough time to steer away?**

```
closing_speed = dot(their_vel - my_vel, direction_to_them)

if closing_speed <= 0:
    return 0  // Moving apart - no collision risk

time_to_collision = edge_distance / closing_speed
urgency = (critical_time / TTC).clamp(0, 1)
force = urgency² * max_force * direction_away
```

**Why this works:**
- Fast approach + close = short TTC = high urgency = strong force NOW
- Slow approach + far = long TTC = low urgency = gentle steering
- Moving apart = skip entirely (Golden Zone optimization)

---

## Key Parameters

| Parameter | Purpose | Suggested Value |
|-----------|---------|-----------------|
| `critical_time` | TTC threshold for max urgency | 1.5 - 2.0 seconds |
| `min_ttc` | Emergency zone (clamp TTC floor) | 0.1 seconds |

**Tuning `critical_time`:**
- Too short → collisions happen (not enough reaction time)
- Too long → creatures over-react to distant neighbors
- Start with 2.0s, adjust based on visual results

---

## Changes Required

### 1. Remove range gate

Delete `max_interaction_distance` check in `avoidance.rs` (lines 71-77). Trust perception's filtering.

### 2. Add velocity to avoidance

Pass neighbor velocity to `calculate_avoidance_force()`. Currently only has position/radius.

### 3. TTC-based urgency

Replace current urgency formula:
```rust
// OLD: distance-based
let urgency = (danger_radius / safe_distance).powi(2);

// NEW: TTC-based
let closing_speed = relative_vel.dot(direction_to_them);
if closing_speed <= 0.0 { continue; }  // Moving apart
let ttc = edge_distance / closing_speed;
let urgency = (CRITICAL_TIME / ttc).clamp(0.0, 1.0);
```

### 4. Smooth force ramp

```rust
let force_magnitude = urgency * urgency * max_accel;  // Quadratic ramp
```

---

## Edge Cases

| Scenario | TTC | Urgency | Behavior |
|----------|-----|---------|----------|
| Head-on collision course | Short | High | Strong avoidance |
| Parallel paths, close | ∞ | 0 | No force (won't collide) |
| Stationary neighbor | ∞ | 0 | No force* |
| Overtaking (same direction) | Long | Low | Gentle steering |

*Stationary neighbors: If we're moving toward them, closing_speed > 0, TTC is finite. Only truly parallel/diverging paths have TTC = ∞.

---

## Distance Fallback?

**Question:** Should stationary obstacles trigger avoidance?

With pure TTC:
- If I'm moving toward stationary giant, closing_speed = my_speed, TTC is finite → avoidance works
- If I'm moving parallel to stationary giant, closing_speed ≈ 0, no avoidance

This seems correct for anti-collision. We only need to avoid things we're going to hit.

**Optional fallback** (if needed):
```rust
// Add distance urgency for very close stationary obstacles
let distance_urgency = if edge_distance < emergency_threshold {
    1.0 - (edge_distance / emergency_threshold)
} else {
    0.0
};
let urgency = ttc_urgency.max(distance_urgency);
```

---

## Files to Modify

| File | Change |
|------|--------|
| `steering/avoidance.rs` | Remove range gate, add TTC formula |
| `steering/system.rs` | Pass velocity data to avoidance |
| `constants/behavior.rs` | Add `CRITICAL_TTC_SECONDS` constant |

---

## Success Criteria

- [x] Creatures steer around each other without collisions
- [x] Fast approaches trigger earlier/stronger avoidance
- [x] Parallel/diverging paths don't waste force
- [x] No jittery over-reaction to distant neighbors
- [x] Code simpler than before (one clear formula)

**Status: ✅ COMPLETE (2025-12-23)**
- TTC-based avoidance implemented in `steering/avoidance.rs`
- Golden Zone optimization skips diverging paths
- Neighbor velocity now passed through `NeighborCache`
- 12 unit tests covering all edge cases

---

## What This Is NOT

- NOT flee behavior (that's Phase B drives)
- NOT predator awareness (that's L1 THREAT classification)
- NOT size-based fear (that's flee urgency)

This is purely: "Something is in my way, steer around it."
