# Weighted Behavior System

**Status:** 📋 Idea (reviewed Sprint 20)
**Related:** `early-warning-avoidance.md`

---

## Concept

Rather than discrete states like a state machine, have behaviour "weights".

Movement/steering decided by weighted states rather than discrete switch:
- A crit with `flee=0.3` and `wander=0.7` appears "spooked" after an encounter
- Weights react to different stimuli at different distances (early warning)
- More varied speeds and acceleration profiles across scenarios

---

## Specialist Review (Sprint 20)

| Specialist | Verdict | Key Insight |
|------------|---------|-------------|
| zoologist-tom | **Strong support** | Biologically realistic - real animals blend behaviors, don't switch discretely |
| ecs-emma | **Viable but premature** | +0.4ms at 100K acceptable, BUT missing threat perception blocker |
| architect-andy | **Sound architecture** | Fused steering makes migration easier; phased approach recommended |

---

## Performance Question (Answered)

> "Would this hurt performance? because every behaviour steering algorithm would be triggered for every crit every time?!?!?"

**Answer: No, acceptable cost.**
- ~+0.4ms at 100K creatures (current tick budget ~45ms = <1% overhead)
- Branch prediction not helping much with Rayon anyway
- Early-exit optimization (`if weight < 0.01, skip`) mitigates further
- SIMD potential for weight multiplication

---

## Biological Rationale (zoologist-tom)

Real animals blend behavioral tendencies rather than switching discretely:

**Neuroscience:** Basal ganglia runs multiple "behavioral programs" simultaneously with weighted outputs. Winner-take-all only in extreme situations.

**Observable examples:**
- Deer grazing near forest edge: feeding (head down) + vigilance (head lifts) + positioning (drifting to cover) - ratios shift with threat level
- Fish schooling: cohesion + alignment + avoidance + foraging + predator-flee all active with varying weights
- Approach-avoidance gradients produce characteristic "darting approach" behavior impossible with discrete states

**The "spooked after encounter" phenomenon is biologically accurate:** After a threat retreats, vigilance weight remains elevated for minutes to hours (not instant return to baseline).

---

## Prerequisite: Threat Perception

**Blocker:** Flee behavior needs threat perception data that doesn't exist yet.

Before weighted behaviors can work, creatures need:
- Threat detection in perception system (who is a predator?)
- Threat distance tracking (how far away?)
- This aligns with "early warning avoidance" as shared prerequisite

---

## Proposed Architecture

### BehaviorWeights Component

```
BehaviorWeights {
    // DNA-derived (set at spawn, immutable)
    base_wander: f32,  // from curiosity gene
    base_seek: f32,    // from hunger drive gene
    base_flee: f32,    // from fear response gene

    // Situational (updated per tick)
    flee_urgency: f32,    // 0.0-1.0, from threat proximity
    seek_urgency: f32,    // 0.0-1.0, from hunger level
    activity_level: f32,  // 0.0-1.0, fatigue suppression
}

effective_weight = base_weight * situational_modifier
```

### Weight Dynamics

Weights shift continuously based on inputs - no explicit "transition" code:

| Threat Distance | Flee Weight | Behavior |
|-----------------|-------------|----------|
| Beyond perception | 0.0 | Normal activity |
| Outer awareness (80-100%) | 0.1 | Slight drift, occasional looks |
| Alert zone (50-80%) | 0.3 | "Spooked" - hesitant |
| Danger zone (20-50%) | 0.6 | Active retreat |
| Critical (<20%) | 0.95 | Full flight |

---

## Implementation Roadmap

### Phase 0: Threat Perception (PREREQUISITE)
- Add threat detection to perception system
- Track threat distances for flee weight calculation

### Phase 1: BehaviorWeights Component
- New component parallel to existing BehaviorMode
- Initialize from current behavior for testing
- DNA genes: curiosity, hunger_drive, fear_response

### Phase 2: Weight-Driven Steering
- Modify fused steering to calculate ALL behaviors
- Multiply each by effective weight
- Early exit: skip if weight < 0.01

### Phase 3: Response Curves
- Replace discrete transitions with continuous curves
- DNA controls curve shape (bold vs timid)
- Integrates with early warning avoidance zones

### Phase 4: Cleanup
- Remove BehaviorMode enum (or keep for debug)
- Optimize with SIMD for weight calculations

---

## Alternative: Urgency Modulation (Simpler)

If full weights are overkill, add `urgency: f32` to CreatureState:
- `urgency = 0.7` = exhausted/sluggish
- `urgency = 1.0` = normal
- `urgency = 1.5` = spooked/twitchy

Achieves "spooked" behavior without calculating all forces. DNA-derivable from temperament gene.

**Limitation:** No true behavior blending (flee + wander simultaneously).

---

## Integration with Early Warning Avoidance

This system and early-warning-avoidance.md are complementary:

- **Early warning** defines the graduated ZONES (distances)
- **Weighted behavior** defines the graduated RESPONSE (weights)

The response curves in Phase 3 directly implement early warning zones via weight dynamics.

---

**Reviewed:** 2025-12-16 (Sprint 20)
