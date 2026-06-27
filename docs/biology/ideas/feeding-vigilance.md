# Feeding-State Vigilance

**Status:** 💡 Idea — design ready
**Zoologist consultation:** 2026-06-27
**Consumer:** `threat-assesment.md` (modulates flee threshold), `drive-simplex.md` (Rest weight during feeding)

---

## Core Insight

Feeding and vigilance are **mechanically incompatible** — a head-down creature cannot simultaneously scan the horizon. But the two systems do not simply trade off: while detection frequency drops during feeding, the *startle magnitude* on a successful threat-detect rises, because the animal knows it is vulnerable. This asymmetry is what generates the emergent "won't easily leave food" behaviour we want.

The reluctance to leave a food source is not a special state. It emerges from the **ghrelin↑ vs cortisol↑ tug-of-war**: hunger hormone raises the threshold at which cortisol (threat-driven) can force a departure. Model it with one continuous formula, not a binary flee-gate.

---

## Biological Mechanism

**Hormones driving the tug-of-war:**

| Hormone | Source | Effect |
|---------|--------|--------|
| **Ghrelin** | Stomach wall (empty) | ↑ risk tolerance, ↑ food-seeking drive, raises flee threshold |
| **Leptin** | Adipose tissue (full) | ↓ feeding motivation, lowers flee threshold (can afford caution) |
| **NPY (Neuropeptide Y)** | Hypothalamus | ↑ appetite, potentiates feeding-over-fleeing |
| **Cortisol / corticosterone** | Adrenal gland (threat) | Overrides ghrelin, triggers departure |

Model in the sim: `ghrelin` ≈ `1 - energy_fraction`, `cortisol` ≈ `perceived_threat`. The flee threshold is the balance point between them.

**Giving-up density (Charnov's Marginal Value Theorem, 1976):** An animal leaves a food patch when:
```
marginal_predation_risk > marginal_energy_gain × hunger_state
```
This is the `flee if threat > base_flee_threshold + hunger_risk_tolerance × (1 - energy)` formula, not a hardcoded severity cutoff.

---

## FOV Reduction During Active Feeding

**Not an anatomical narrowing — a functional one** from head-down posture and attentional lock.

| Phenotype | `feeding_fov_multiplier` | Notes |
|-----------|--------------------------|-------|
| Herbivore (ungulate) | **0.4 – 0.6** | Retain wide peripheral arc (laterally-placed eyes, 270–340° natural FOV); only forward binocular cone collapses |
| Herbivore (browser/picker) | **0.5 – 0.7** | More upright posture, less extreme reduction |
| Carnivore at kill | **0.1 – 0.3** | Frontal eyes, no panoramic fallback; actively gorging = near-blind to non-immediate threats |

This reduction applies **only during active bite bouts**, which are short (2–8s for grazers). Between bouts, the creature returns to full FOV for a head-up scan (`vigilance_interval`).

---

## Vigilance Interrupt Cycle

Feeding is not continuous attention suppression. Animals **interleave** feeding and scanning in short cycles:

```
[head-down bite 2–8s] → [head-up scan 1–3s] → [head-down bite] → ...
```

**Gene:** `vigilance_interval` — how often the creature interrupts feeding for a head-up scan.

| Trophic position | `vigilance_interval` | Biological basis |
|------------------|---------------------|-----------------|
| Small prey | Short (1–4s) | High predation rate; can't afford long attention gaps |
| Large herbivore | Medium (4–10s) | Moderate predation risk |
| Apex carnivore | Long (20–60s) | Few threats; focus on intake rate |

Allometric rule (fits with `brain-decision-timing.md`): smaller creatures interrupt more frequently — they are more heavily predated and have faster neural cycles.

**During a head-up scan:** full FOV, full threat assessment, normal flee threshold.
**During a bite bout:** `feeding_fov_multiplier` applied, long-range scan skipped.

---

## The Startle Asymmetry

While detection is reduced during a bite bout, a **successful** threat-detect triggers a *larger* response than normal:

```
if is_feeding AND threat_detected:
    effective_flee_force *= FEEDING_STARTLE_MULTIPLIER  // ~1.4–1.8
```

**Biological basis:** Feeding animals know they are exposed. A partial threat cue during head-down triggers explosive acceleration (wildebeest "pronk" response). The lower detection rate is compensated by higher escape urgency when a threat does break through.

---

## Hunger ↔ Flee Threshold (Continuous, Not Binary)

```
effective_flee_threshold = base_flee_threshold
                         + hunger_risk_tolerance × (1.0 - energy_fraction)
```

| Scenario | Result |
|----------|--------|
| Starving at food (energy ~0) | Threshold raised by full `hunger_risk_tolerance` — stays through moderate threats |
| Half-hungry (energy ~0.5) | Partial raise — leaves on significant threats, stays for mild ones |
| Satiated (energy ~1.0) | No raise — low leptin, can afford caution, flees readily |

**Gene:** `hunger_risk_tolerance` — how strongly hunger suppresses threat response.
- Low (0.0–0.2): always cautious, threshold barely rises with hunger (paranoid prey archetype)
- High (0.6–1.0): dramatically risk-tolerant when starving (desperate forager archetype)

---

## Herbivore vs Carnivore Vigilance During Feeding

Real herbivores devote **20–50% of foraging time to head-up scanning** (Thomson's gazelle, impala, elk field studies). Carnivores at a kill: **5–15%** of time vigilant.

This **emerges** from `trophic_position` + `vigilance_interval` + `feeding_fov_multiplier`. Do not hardcode "herbivore = vigilant." The correct DNA primitives produce it:

| Trait | Herbivore archetype | Carnivore archetype |
|-------|---------------------|---------------------|
| `feeding_fov_multiplier` | 0.4–0.6 | 0.1–0.3 |
| `vigilance_interval` | 4–10s | 20–60s |
| `hunger_risk_tolerance` | 0.3–0.6 | 0.5–0.9 (desperation hunts) |
| `satiety_flightiness` | 0.4–0.7 | 0.1–0.3 |

---

## Satiety Flightiness

A well-fed herbivore is *more* flighty, not less — low ghrelin means the cost of abandoning food is trivial. The flee threshold *drops* after a good meal:

```
effective_flee_threshold = base_flee_threshold
                         - satiety_flightiness × energy_fraction
```

**Gene:** `satiety_flightiness` — how much energy level *lowers* the flee threshold.

Note: "flees more readily" = *earlier decision*, not faster movement. Keep separate from locomotion cost.

---

## Many-Eyes Effect (Conspecific Density)

Individual vigilance burden drops with nearby conspecifics — each animal in the group covers part of the arc, so each can afford more time head-down:

```
effective_vigilance_interval = base_vigilance_interval
                             × (1.0 + k × nearby_conspecific_count)
```

**Biological grounding:** Thomson's gazelle scan rate drops from ~40% solo to ~10% in groups of 20+ (Pulliam 1973, many replications). Fish schools and starling murmurations show the same.

**Golden Zone:** This is the many-eyes dilution effect, and it is a free performance win — a creature in a dense herd runs fewer head-up scan evaluations per second. A 1M-creature herd distributes the perception cost.

Gene for DNA variation: `social_vigilance_sensitivity` — how much nearby conspecifics reduce individual scan burden.

---

## Carnivore Threat-Type Selectivity at Kill

Carnivores at a carcass are **not globally blind** — they tunnel on the category of threat:
- **Ignore:** distant large herbivores, ambient movement
- **Track actively:** rival carnivores, scavengers, kleptoparasites (hyenas at a lion kill)

This is threat-type selectivity, not global perception collapse. Model it with the `trophic_position` classification gene:

```
while is_feeding:
    if threat_entity.trophic_position >= my_trophic_position:
        run full threat assessment (competitor risk)
    else:
        skip (non-threat phenotype while feeding)
```

Performance: the candidate set for "same-tier or higher" is small. Cheap sweep AND accurate biology.

---

## Satiated Predator State Flip

The deep tunnel applies during active gorging (early kill, high NPY). As they fill (leptin rises), behaviour flips:
- Reduce feeding intensity → `Rest` weight rises in Drive Simplex
- Raise head, resume normal vigilance (guarding behaviour)
- Eventually leave carcass for cover

**Transition:** `feeding_intensity = (1.0 - energy_fraction).clamp(0, 1)` — fades naturally as energy fills.

---

## DNA Primitives Summary

| Gene | Range | Effect |
|------|-------|--------|
| `feeding_fov_multiplier` | 0.1–0.6 | Attention tunnel depth during bite bout |
| `vigilance_interval` | 1s–60s | How often feeding is interrupted for head-up scan |
| `hunger_risk_tolerance` | 0.0–1.0 | Ghrelin gain: how much hunger raises flee threshold |
| `satiety_flightiness` | 0.0–0.8 | Leptin gain: how much satiety lowers flee threshold |
| `social_vigilance_sensitivity` | 0.0–1.0 | Many-eyes benefit: vigilance drop per nearby conspecific |

These produce herbivore/carnivore differences as *emergence* from `trophic_position` DNA rather than a species flag.

---

## Golden Zone Opportunities

1. **Skip far-perception scan while `is_feeding`** — matches head-down attention collapse. Only run short-range + threat-type-filtered scan during bite bout. Full scan on vigilance interrupt only.
2. **Satiated predator drops tunnel on energy fill** — cheap state flip; guarding is low-compute (just watch same-tier entities).
3. **Many-eyes: vigilance_interval scales with conspecific count** — herd members run fewer full-perception evaluations. With 1M creatures in herds, massive aggregate saving.

---

## Integration

| System | Interaction |
|--------|-------------|
| `threat-assesment.md` | Feeding state modulates `effective_flee_threshold`; startle multiplier on threat-detect while feeding |
| `drive-simplex.md` | `is_feeding` contributes to `Rest` weight; `vigilance_interval` controls when full simplex re-evaluates |
| `brain-decision-timing.md` | `vigilance_interval` maps to brain decision cooldown during feeding; panic override still fires (amygdala hijack) |
| `stress-tunnel-vision.md` | Sustained high threat_weight during feeding accumulates stress → can further narrow FOV on top of feeding reduction |
| `energy-vigilance.md` | Hungry crits scan MORE between meals (not feeding); hungry crits stay AT meals despite threats. These are distinct states. |
| `hunger-gating.md` | Satiated predators already skip prey detection; extend: satiation also lifts feeding-tunnel and raises vigilance |
| `herbivore-competition.md` | A hungry herbivore at a contested plant cell has raised flee threshold → is more likely to contest rather than yield |

---

## Open Questions

1. **Vigilance interrupt as animation cue?** Shader-sarah can use `vigilance_interval` ticks to drive head-raise posture lerp — same data that controls gameplay drives the visual rhythm.
2. **Stacking FOV reductions:** Stress-tunnel-vision + feeding FOV multiplier could bring effective FOV near zero. Apply a floor (minimum ~15°) or multiply with a cap?
3. **Partial threat-detect during bite bout:** If FOV is reduced but a threat appears at the edge, do we fire full threat-assessment or a reduced one? Recommendation: fire full assessment (the detection is the gate, not the assessment), apply startle multiplier.
