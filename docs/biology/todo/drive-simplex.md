# Drive Simplex

A unified model for creature motivation where behavioral forces are constrained to sum to 1.0.

```
┌─────────────────────────────────────────────────────────────────┐
│                        DRIVE SIMPLEX                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  LAYER 1: SIMPLEX (sums to 1.0)                         │   │
│   │                                                         │   │
│   │     Approach ←───────→ Threat Response ←───────→ Rest   │   │
│   │        0.6                  0.3                  0.1    │   │
│   │                              │                          │   │
│   │                              ▼                          │   │
│   │                    ┌─────────────────┐                  │   │
│   │                    │ DNA + Context   │                  │   │
│   │                    │ Fight/Flight/   │                  │   │
│   │                    │ Freeze decision │                  │   │
│   │                    └─────────────────┘                  │   │
│   │                                                         │   │
│   │  Output: Forward/backward intent vector                 │   │
│   └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│                           [ADD]                                 │
│                              ▲                                  │
│                              │                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  LAYER 2: AVOIDANCE (separate, not in budget)           │   │
│   │                                                         │   │
│   │              ← Lateral force only →                     │   │
│   │                                                         │   │
│   │  Output: Left/right steering vector                     │   │
│   └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│                      ┌──────────────┐                           │
│                      │ FINAL FORCE  │                           │
│                      └──────────────┘                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Simplex:** A mathematical structure where coordinates sum to a constant (here, 1.0). Like a probability distribution, but for force allocation.

## Core Concept

Movement has two layers:

### Layer 1: Drive Budget (sums to 1.0)

Three drives compete for the force budget:

| Drive | Direction | DNA Influence |
|-------|-----------|---------------|
| **Approach** | Toward attractors | Hunger, curiosity, libido |
| **Threat Response** | Depends on sub-decision | Aggression, boldness |
| **Rest** | Stationary (zero vector) | Energy conservation |

```
        Approach
           /\
          /  \
         /    \
        /      \
       /________\
   Threat     Rest
```

### Threat Response Sub-Decision (DNA + Context)

When threat is detected, DNA and context determine the response direction:

| Response | Direction | When |
|----------|-----------|------|
| **Fight** | Toward threat | High aggression, threat smaller than self |
| **Flight** | Away from threat | Low aggression, threat bigger than self |
| **Freeze** | Zero vector | Uncertain, cautious, or hiding |

This is a separate decision from the budget allocation. The budget says "how much energy for threat response?" The sub-decision says "fight, flight, or freeze?"

### Velocity-Based Threat Assessment

**A charging predator is far more threatening than a stationary one.**

Real prey animals don't just measure distance—they compute **time-to-contact (tau)**:

```
closing_velocity = how fast predator is approaching (not raw speed)
tau = distance / closing_velocity
```

A predator 100m away charging at 20m/s (τ = 5s) triggers more urgent flight than one 30m away but stationary (τ = ∞).

| Factor | Effect on Threat Response |
|--------|---------------------------|
| **Closing velocity** | Only approach matters—fast predator moving away = low threat |
| **Time-to-contact** | Low tau = urgent response, high tau = time to assess |
| **Sudden acceleration** | Predator starts charging → immediate threat spike |
| **Stationary predator** | Triggers vigilance (elevated awareness), not immediate flight |

**DNA modulation:**

| Trait | Bold Crit | Timid Crit |
|-------|-----------|------------|
| Velocity sensitivity | Low (waits to assess) | High (reacts to any movement) |
| Tau threshold | Short (flees at τ < 2s) | Long (flees at τ < 5s) |
| Acceleration response | Dampened | Amplified |

**Emergent behaviors:**
- Timid crits flee early from slow-approaching predators
- Bold crits hold ground until the charge begins
- Ambush predators benefit from slow stalking (low closing velocity)

**Biological basis:** Looming detection neurons (LGMD pathway) respond to rate of angular expansion, not absolute size. Fish lateral lines detect water displacement velocity.

### Layer 2: Avoidance (separate, lateral only)

**Avoidance is not in the budget.** It's an orthogonal steering layer:
- Only produces lateral force (perpendicular to velocity)
- Nudges left/right to dodge obstacles
- Does not compete with the drive budget

### Distance Zones (affects both layers)

Both Simplex and Avoidance respond to **proximity zones**:

| Zone | Distance | Simplex Effect | Avoidance Effect |
|------|----------|----------------|------------------|
| **Early Warning** | 3-5× personal space | Threat Response begins rising | Gentle lateral nudge (15% force) |
| **Personal Space** | 1× personal space | Threat Response moderate | Active lateral steering |
| **Panic** | 0.5× personal space | Threat Response maximal | Maximum lateral force |

**Gradient, not binary:** Force ramps smoothly across zones. No sudden jumps.

**Speed adaptation:** Fast crits need more warning distance (stopping distance = v²/2a). Warning zone extends with speed.

**Energy modulation:** Hungry crits tolerate closer proximity. Formula: `effective_distance = base × (0.4 + 0.6 × energy)`

**Biological basis:**
- Birds in murmurations start adjusting at 7-10 body lengths
- Fish schools react at 3-4 body lengths, emergency at <0.5
- Neural processing takes 160-410ms, so early detection is survival-critical

**References:** Ballerini et al. (2008), Partridge & Pitcher (1980), Helbing & Molnár (1995)

### Combined Force

```
drive_force = (Approach × toward_target) + (Threat × response_direction) + (Rest × 0)
final_force = drive_force + lateral_avoidance
```

### Braking is Emergent

No explicit braking force. Slowing happens when:
- Threat Response increases → Approach decreases → less forward drive
- Rest increases → everything else decreases → overall slowdown

### Wandering is Target Selection

Wandering is not a special movement mode. It's just **choosing where to go**:
1. Brain picks a wander target (random point, interesting area, home region)
2. Approach drives the crit toward that target
3. All simplex rules apply normally

The difference between "seeking food" and "wandering" is only the target, not the movement physics.

---

## No Behavior State Machine

The Drive Simplex **replaces discrete behavior states**. There is no `BehaviorMode` enum.

| Old State | Emergent From |
|-----------|---------------|
| Catatonic | Simplex (0, 0, 1) - all Rest |
| Wandering | Simplex (0.6, 0.1, 0.3) + wander target |
| Seeking | Simplex (0.8, 0.1, 0.1) + food/mate target |
| Fleeing | Simplex (0.1, 0.8, 0.1) + DNA→flight |
| Fighting | Simplex (0.1, 0.8, 0.1) + DNA→fight |
| Frozen | Simplex (0.1, 0.8, 0.1) + DNA→freeze |

**Benefits:**
- No state transition logic
- Smooth blending (no hard switches)
- Single unified steering calculation
- Behaviors emerge from simple rules

Labels like "fleeing" or "seeking" become **descriptions** of simplex regions, not code paths.

---

## Why This Matters

### Prevents Force Overflow (Zipping Bug)

With a drive budget, overflow is **impossible by construction**. Total force is always bounded because drives sum to 1.0.

### Natural Trade-offs

An animal cannot simultaneously sprint toward prey AND away from a predator. The budget enforces this physically.

### Emergent Complexity

| Approach | Threat | Rest | Behavior |
|----------|--------|------|----------|
| 0.8 | 0.1 | 0.1 | Hungry pursuit |
| 0.1 | 0.8 | 0.1 | Threat response (fight/flight/freeze per DNA) |
| 0.1 | 0.1 | 0.8 | Resting/waiting |
| 0.4 | 0.4 | 0.2 | Cautious foraging |
| 0.0 | 0.0 | 1.0 | Catatonic |

---

## Worked Examples

### Example 1: Seeker with Obstacle

Crit seeking food, obstacle blocks path.

**Behavior:** Avoidance provides lateral nudge. Budget unchanged (still approach-dominant). Crit curves around obstacle.

**Key insight:** Obstacles don't change intent, just steering.

### Example 2: Threat Appears

Crit approaching target, threat appears in front.

**Behavior:**
1. Budget shifts: Threat Response steals from Approach
2. DNA decides: Fight (toward) / Flight (away) / Freeze (stop)
3. Avoidance adds lateral steering

**Key insight:** Budget answers "how urgent?" DNA answers "which response?"

### Example 3: Stalking vs Charging Predator

Same predator, same distance, different velocities.

**Stalking (slow approach):**
- Closing velocity low → tau high → time to assess
- Threat Response rises gradually
- Bold crits continue foraging with elevated vigilance
- Timid crits begin early retreat

**Charging (fast approach):**
- Closing velocity high → tau low → urgent response
- Threat Response spikes immediately
- All crits shift to flight-dominant simplex
- Acceleration detection amplifies response further

**Key insight:** Distance alone doesn't determine urgency—closing velocity does.

---

## Research Summary

| Topic | Key Takeaway | Link |
|-------|--------------|------|
| **Approach-Avoidance Conflict** | Psychology validates fight/flight/freeze as distinct responses to threat | [PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC5548639/) |
| **Hull's Drive Theory** | Classic drives are additive, not constrained - our simplex is novel | [Wikipedia](https://en.wikipedia.org/wiki/Drive_theory) |
| **Reynolds Steering** | Additive forces + clipping has known cancellation problems | [GDC99](https://www.red3d.com/cwr/steer/gdc99/) |
| **Subsumption** | Priority/override model, not budget sharing | [Wikipedia](https://en.wikipedia.org/wiki/Subsumption_architecture) |

**What's novel:** Simplex constraint (sum=1) for force allocation appears unique. Prevents overflow by construction rather than post-hoc capping.

---

## Simplex Dynamics

### Persistence (No Instant Snap-Back)

Simplex values don't reset instantly. A spooked crit stays in elevated Threat Response for a **decay period**:

| Event | Simplex Change | Decay |
|-------|----------------|-------|
| Threat appears | Threat Response spikes | - |
| Threat retreats | Threat Response decays over seconds/minutes | Gradual |
| Food found | Approach increases | Immediate |
| Exhaustion | Rest increases | Gradual |

**Biological basis:** After a threat retreats, vigilance remains elevated for minutes to hours - not instant return to baseline.

### DNA Response Curves

DNA controls how quickly and strongly crits respond:

| Gene | Effect |
|------|--------|
| **Boldness** | Bold crits: slow Threat Response rise, fast decay. Timid crits: fast rise, slow decay |
| **Aggression** | Shifts fight/flight/freeze threshold |
| **Curiosity** | Higher baseline Approach weight |

### Early-Exit Optimization

Performance: Skip calculations when weight < 0.01
- ~+0.4ms overhead at 100K creatures (acceptable, <1% of tick budget)
- SIMD potential for weight multiplication

---

## Prerequisites

### Threat Perception (BLOCKER)

Before Drive Simplex can work fully, creatures need:
- Threat detection in perception system (who is dangerous?)
- Threat size comparison (bigger than me?)
- Threat distance tracking (how close?)
- Threat velocity tracking (closing speed, acceleration)

Without this, Threat Response slider has no input.

---

## Specialist Reviews

| Specialist | Verdict | Key Insight |
|------------|---------|-------------|
| zoologist-tom | **Strong support** | Biologically realistic - real animals blend behaviors, don't switch discretely |
| ecs-emma | **Viable** | +0.4ms at 100K acceptable |
| architect-andy | **Sound** | Fused steering makes migration easier |

**Biological examples (zoologist-tom):**
- Deer grazing: feeding + vigilance + positioning all active, ratios shift with threat
- Fish schooling: cohesion + alignment + avoidance + foraging weighted simultaneously
- "Darting approach" behavior impossible with discrete states

---

## Open Questions

1. **Decay rates:** How fast do simplex values return to baseline?
2. **Threat assessment:** How does crit evaluate "threat bigger than self"?
3. **Multiple threats:** How do multiple threats combine into single Threat Response?
4. **Target priority:** When multiple attractors exist, how is target chosen?

## Next Steps

- [ ] Define DNA genes for aggression, boldness, curiosity
- [ ] Design threat assessment (size comparison, threat type, velocity)
- [ ] Define tau thresholds and velocity sensitivity genes
- [ ] Prototype slider mechanics
