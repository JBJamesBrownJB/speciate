# Threat Assessment

**Status:** 💡 Idea — design phase
**Consumer:** `docs/biology/todo/drive-simplex.md` (feeds the Threat Response weight)
**Distinct from:** `docs/biology/done/ttc-avoidance.md` (lateral collision steering, a different layer)

---

## The Core Question

Does threat assessment only run when a creature is moving?

**No — it runs every brain tick, whether stationary or not.**

A grazing herbivore needs to know an elephant is approaching even while it's standing still eating. Threat assessment is always-on vigilance; movement response comes after.

---

## What It Is

Threat assessment is a continuous evaluation function that runs inside the brain decision loop. It reads from the perception system, scores each visible entity, and outputs:

1. **Threat weight** — a `[0.0, 1.0]` scalar fed into the Drive Simplex's `Threat Response` budget slot
2. **Threat direction** — a unit vector pointing toward the dominant threat (used by flight/fight sub-decision)

The Drive Simplex then decides what to *do* about that threat (fight, flee, freeze). Threat assessment only answers "how threatened am I, and from where?"

---

## Architecture Position

```
PERCEPTION  →  THREAT ASSESSMENT  →  DRIVE SIMPLEX  →  FORCE ACCUMULATOR
                  (this doc)          (drive-simplex.md)   (physics)

TTC AVOIDANCE (ttc-avoidance.md)  ────────────────────────────────────┘
  (orthogonal, lateral only, not in simplex budget)
```

Threat assessment and TTC avoidance share a concern (nearby entities) but operate in different layers:

| System | Layer | Output | When |
|--------|-------|--------|------|
| TTC Avoidance | Layer 2 | Lateral force | Collision imminent |
| Threat Assessment | Layer 1 input | Simplex weight | Always |

A creature can be threat-assessing a stalking predator 40m away (high tau, low avoidance force) while the avoidance system is silent. When that predator closes to 2m, both systems fire.

---

## Inputs

For each entity in the creature's perception range:

| Input | How | Why |
|-------|-----|-----|
| **Tau (time-to-contact)** | `tau = distance / closing_velocity` | Charging predator 100m away is more threatening than stationary one at 30m |
| **Size ratio** | `threat_mass / self_mass` | Bigger = more threatening (feeds fight/flight sub-decision) |
| **Acceleration spike** | Δclosing_velocity per tick | Predator starting a charge triggers immediate spike |
| **Closing velocity** | `dot(relative_vel, direction_to_entity)` | Only approaching entities are threats; diverging = 0 |

Closing velocity ≤ 0 → entity moving away → skip (same optimization as TTC avoidance).

---

## Threat Score per Entity

```
if closing_velocity <= 0:
    threat = 0.0   // moving away, not a threat right now

else:
    tau = edge_distance / closing_velocity
    urgency = (TAU_CRITICAL / tau).clamp(0, 1)   // 1.0 when collision imminent
    size_factor = (threat_mass / self_mass).clamp(0, MAX_SIZE_FACTOR)
    accel_spike = max(0, delta_closing_velocity)  // charge detection

    threat = (urgency * URGENCY_WEIGHT
            + size_factor * SIZE_WEIGHT
            + accel_spike * ACCEL_WEIGHT).clamp(0, 1)
```

**`TAU_CRITICAL`** — tau threshold where urgency hits maximum. Timid crits react at longer tau; bold crits hold until short tau. DNA-driven.

---

## Aggregating Multiple Threats

When multiple entities are threatening simultaneously, options:

- **Max wins** — highest individual threat score drives the response direction. Simple, realistic (animals focus on the most urgent threat).
- **Weighted sum** — all threats accumulate, capped at 1.0. Risk: flanking predators overwhelm correctly.

**Recommended: max wins for direction, sum (clamped) for weight.** This means a creature fleeing one predator stays aware that a second predator exists (elevated weight), while its escape vector points away from the primary threat.

---

## Output → Drive Simplex

```
threat_weight  → Drive Simplex Threat Response slot
threat_dir     → Threat Response sub-decision (fight/flight/freeze)
```

The simplex then allocates the budget: as `threat_weight` rises, it steals from `Approach` and `Rest`. DNA controls the response direction.

See `drive-simplex.md` for how threat_weight becomes actual movement force.

---

## DNA Modulation

| Gene | Effect |
|------|--------|
| **Boldness** | Shifts tau threshold. Bold: reacts only at low tau. Timid: reacts early from long range |
| **Aggression** | Shifts fight/flight/freeze threshold given size ratio (aggressive crits fight bigger opponents) |
| **Vigilance** | Baseline threat sensitivity even at low urgency — always slightly elevated |

These interact with `energy-vigilance.md`: hungry crits scan wider, so they detect threats earlier — but also have reduced personal space tolerance per `movement-physics.md` (ghrelin dampens avoidance).

---

## Worked Example: Grazing Goat + Approaching Elephant

```
Elephant 80m away, closing at 3 m/s:
  tau = 80 / 3 = 26.7s → urgency = low
  size_factor = high (elephant >> goat)
  threat = 0.15 (size alone elevates threat despite long tau)

Simplex: Approach=0.7, Threat=0.15, Rest=0.15
→ Goat continues grazing but with mild alertness
→ TTC avoidance: no force (tau too long, not on collision path)

Elephant 20m away, closing at 5 m/s:
  tau = 20 / 5 = 4s → urgency = high (approaching TAU_CRITICAL)
  size_factor = high
  threat = 0.75

Simplex: Approach=0.1, Threat=0.75, Rest=0.15
DNA (timid) → flight direction (away from elephant)
→ Goat moves away — not a mode switch, just budget shift
→ TTC avoidance: lateral nudge begins if on intersection path
```

The behavior emerges from one continuous calculation, not a state flip.

---

## Relationship to Existing Systems

| Doc | Relationship |
|-----|-------------|
| `drive-simplex.md` | **Primary consumer** — threat_weight feeds the Threat Response slot |
| `ttc-avoidance.md` | **Orthogonal** — lateral collision layer, shares closing_velocity math but separate output |
| `brain-decision-timing.md` | **Gating** — threat assessment runs when `can_decide()` is true; panic override bypasses cooldown |
| `stress-tunnel-vision.md` | **Consequence** — high threat_weight over time accumulates stress → narrows FOV |
| `feeding-vigilance.md` | **State modulation** — eating raises flee threshold (ghrelin), reduces detection frequency, adds startle multiplier on detect. Hungry crits at food don't leave easily |
| `energy-vigilance.md` | **Modulation** — hungry crits scan wider, detect threats at longer range (when not eating) |
| `perception-system.md` | **Input source** — entity list, distances, velocities from perception pass |

---

## Feeding-State Modulation

When a creature is actively eating, threat assessment still runs but the flee threshold is raised by hunger:

```
effective_flee_threshold = base_flee_threshold
                         + hunger_risk_tolerance × (1.0 - energy_fraction)
                         - satiety_flightiness × energy_fraction
```

Additionally, while `is_feeding`, run only the short-range + same-tier-threat sweep (skip the expensive long-range scan). Full scan fires on `vigilance_interval` interrupts only. See `feeding-vigilance.md`.

---

## Open Questions

1. **Stationary predator:** No closing velocity → threat = 0? Real prey animals fear stationary predators too (ambush risk). May need a size-weighted ambient vigilance floor even at tau = ∞.
2. **Multiple threat aggregation:** Max-wins vs sum — needs playtesting.
3. **Threat memory:** Should threat_weight decay gradually after entity leaves FOV, or drop instantly? `drive-simplex.md` has persistence notes — likely: gradual decay governed by boldness gene.
4. **Non-creature threats:** Does a fast-moving rock or environmental hazard count? Probably yes — same tau math applies.

---

## Implementation Notes

- Runs inside the brain decision loop, gated by `can_decide()` (same as other behavioral decisions)
- Shares closing_velocity computation with TTC avoidance — calculate once, share across both systems
- Early-exit: entities with closing_velocity ≤ 0 skip all further math
- SIMD potential: tau and urgency calculation across N neighbors is data-parallel

**Blocker for Drive Simplex:** Until threat assessment exists, the Threat Response weight has no meaningful input.
