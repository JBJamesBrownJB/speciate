# Drive Budget

A unified model for creature motivation where all behavioral forces are constrained to sum to 1.0.

## Core Concept

Movement has two layers:

### Layer 1: Drive Budget (sums to 1.0)

Three drives compete for the force budget:

| Drive | Direction | Examples |
|-------|-----------|----------|
| **Approach** | Toward attractors | Food, mates, shelter |
| **Flee** | Away from threats | Predators, danger |
| **Rest** | Stationary | Conserve energy, wait |

These form a **simplex** - linked sliders where increasing one decreases the others.

```
        Approach
           /\
          /  \
         /    \
        /      \
       /________\
     Flee      Rest
```

### Layer 2: Avoidance (separate, lateral only)

**Avoidance is not in the budget.** It's an orthogonal steering layer:
- Only produces lateral force (perpendicular to velocity)
- Nudges left/right to dodge obstacles
- Does not compete with Approach/Flee/Rest

### Combined Force

```
drive_force = (Approach × toward_target) + (Flee × away_from_threat) + (Rest × 0)
final_force = drive_force + lateral_avoidance
```

### Braking is Emergent

No explicit braking force. Slowing happens because:
- Flee increases → Approach decreases → less forward drive
- Or: Rest increases → both Approach and Flee decrease → overall slowdown

Deceleration emerges from the budget constraint.

## Why This Matters

### Prevents Force Overflow (Zipping Bug)

The current architecture accumulates forces from multiple behaviors, then caps. This creates edge cases where the cap might fail.

With a drive budget, overflow is **impossible by construction**:
- Emergency avoidance doesn't ADD force
- It STEALS budget from approach/rest
- Total force is always bounded

### Natural Trade-offs

Biology: An animal cannot simultaneously sprint toward prey AND away from a predator. It must choose, or compromise.

The drive budget enforces this:
- High avoid → low approach (can't chase while fleeing)
- High rest → low everything (conserving energy)
- Approach + avoid balanced → cautious movement

### Emergent Complexity from Simple Rules

Three sliders. That's it. But the combinations produce rich behavior:

| Approach | Flee | Rest | Emergent Behavior |
|----------|------|------|-------------------|
| 0.8      | 0.1  | 0.1  | Hungry pursuit |
| 0.1      | 0.8  | 0.1  | Panicked flight |
| 0.1      | 0.1  | 0.8  | Resting/waiting |
| 0.4      | 0.4  | 0.2  | Cautious foraging |
| 0.0      | 0.0  | 1.0  | Catatonic |
| 0.5      | 0.5  | 0.0  | Frozen indecision |

## Integration with Existing Systems

### Influence Maps
Spatial fields that **pull the sliders**:
- Threat influence → increases Avoid
- Resource influence → increases Approach
- Safe zone influence → increases Rest

### Early-Warning Avoidance
Gradual slider shift as threats approach:
- Far threat: slight Avoid increase
- Close threat: Avoid dominates
- No discrete "flee mode" transition

### Weighted Behaviors
The slider positions ARE the weights:
- No separate "behavior priority" system
- No force multiplier constants
- The budget IS the weight

## Relationship to Current Constants

Current system has force multipliers:
- `WANDER_FORCE_MULT` = 0.25 (25%)
- `SEEK_FORCE_MULT` = 0.7 (70%)

These become slider positions in the drive budget:
- Wandering ≈ balanced state (0.3, 0.3, 0.4)?
- Seeking ≈ approach-dominant (0.7, 0.2, 0.1)?

## Worked Examples

### Example 1: Seeker with Obstacle in Path

Crit seeking food, obstacle blocks direct path.

**Behavior:** As obstacle gets closer, Avoid steals budget from Approach. Crit naturally curves around obstacle, slowing down in the process. Once clear, Approach recovers.

**Key insight:** Natural slowdown in cluttered areas is a feature - real animals do this.

**Edge case:** Obstacle directly between crit and target → vectors cancel. Solutions: lateral escape preference, noise tiebreaker, or rest/wait.

### Example 2: Threat Appears Directly in Front

Crit approaching target, threat suddenly appears between crit and target.

**What happens:**
1. Budget shifts: Flee steals from Approach (e.g., 0.7/0.1/0.2 → 0.3/0.5/0.2)
2. Avoidance activates: lateral force (left or right, needs tiebreaker)

**The vectors:**
- Approach: forward (toward target)
- Flee: backward (away from threat)
- Avoidance: lateral (perpendicular)

**Result:** Flee and Approach oppose. If Flee dominates, crit retreats while curving laterally. Unlike pure avoidance, the crit won't freeze - Flee provides clear backward intent.

**Key insight:** The budget separates "should I run?" (Flee) from "which way around?" (Avoidance). Old model conflated these.

---

## Open Questions

1. **How do sliders move?** Instantaneous snap or smooth interpolation?
2. **What sets the sliders?** Brain decisions? Continuous perception integration?
3. **Sub-drives?** Is "approach food" different from "approach mate"?
4. **Rest = zero force?** Or does rest have its own stabilizing force?

## Next Steps

- [ ] Read `influence-maps.md`, `early-warning-avoidance.md`, `weighted-behaviour.md`
- [ ] Sketch how drive budget unifies these concepts
- [ ] Design the slider mechanics (interpolation, decision points)
- [ ] Map to DNA expression (genes control slider sensitivity?)
