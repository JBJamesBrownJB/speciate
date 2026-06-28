# Biology Consultation Notes

Log of zoologist-tom consultations for biological accuracy in A-Life simulation.

---

## 2025-12-28 | Drive Integration Architecture

**Topic:** Multi-sensory drive integration for macro navigation

**Question:** How do real animals neurologically integrate multi-sensory inputs into movement decisions?

**Findings:**

Real animal brains use **layered integration**, not a single strategy:

### Two-Tier System

**Tier 0: Emergency (Priority Override)**
- Brainstem-driven, reflexive
- Immediate threats bypass higher processing
- No weighted averaging - pure priority override
- Example: Deer seeing wolf doesn't weigh hunger vs fear - fear wins absolutely

**Tier 1: Motivated (Weighted Sum with State Modulation)**
- Hypothalamus modulates drive weights based on internal state
- Hunger increases food-seeking weight 3-5x (ghrelin hormone)
- Cortisol increases vigilance, decreases exploration
- Fatigue increases rest-seeking
- Basal ganglia handles action selection (winner-take-all for incompatible, blending for compatible)

### DNA-Encoded Base Weights

Different species use different integration strategies based on ecological niche:

| Species Type | Strategy | Biological Reason |
|-------------|----------|-------------------|
| Prey animals (deer, rabbits) | Fear-dominant hierarchy | Survival requires instant flight |
| Apex predators (lions, sharks) | Hunger-dominant, low fear weight | Few threats, focus on feeding |
| Social animals (wolves, dolphins) | Strong social drive integration | Cooperation essential |
| Opportunists (crows, rats) | High flexibility, rapid weight switching | Exploit varied environments |

### Golden Zone Discovery: Freeze Response

When flee directions cancel (cornered prey), output is near-zero. This produces **tonic immobility** (freeze response) - biologically accurate AND computationally efficient (no movement to process).

### Emergent Behaviors from Architecture

- **Satiation blindness:** Low hunger → weak hunt drive → predator ignores nearby prey
- **Desperation foraging:** Starving creature has amplified food-seeking, approaches risky areas
- **Personality types:** DNA weight variation creates cautious vs bold individuals
- **Context-dependent flocking:** Danger increases social drive weight (safety in numbers)

### Recommendation Applied

Implemented in the ABC Super Sprint (see `sprint_summaries/abc-super-sprint_summary.md`):
- Two-tier system (Emergency/Motivated) - Phase B implements Motivated only
- Contribution arrays per category (flee/approach/disperse)
- DriveContribution with source, tier, vector, magnitude
- DNA weights (uniform in Phase B, modulated in future)

---

## Perception range allometry exponent: 0.35 → 0.25 (2026-06-21)

**Decision:** Lowered `SIZE_ALLOMETRY_EXPONENT` (perception range vs body size) from 0.35 to 0.25.
**Why:** A size-10 narrow-FOV (45°) creature had ~497m perception range — a third of the world — dominating perception compute (the fattest tick phase) and biologically over-generous. Real eye/acuity allometry scales ~mass^0.2-0.25 and detection range saturates from atmospheric/water extinction.
**Effect:** Large-crit range trimmed ~26% (e.g. 10m/45° FOV: ~497m→~368m); small/medium crits ~unchanged. Range scanned ∝ range, L1 cells scanned ∝ range² — so a big perception-cost cut concentrated on the largest creatures.
**Trade-off:** Giants have shorter strategic (L1) awareness — prey gain real refuge distance. Realistic range to explore further: 0.20-0.25, optionally with a hard range cap (~400-600m). Monitor apex-predator population for trophic stability.

## Random-DNA size distribution: UNIFORM → LOG-NORMAL (2026-06-21)
**Decision:** "Random DNA" body size now samples a LOG-NORMAL (right-skewed) distribution, not uniform. Range unchanged 0.1–10 m.
- Sample in log space: median ~0.5 m (mu_ln = ln 0.5), sigma = 0.45 in log10 (≈1.036 natural). Clamp to [0.1, 10]; size_gene = (size - 0.1)/9.9 feeds the existing linear express_gene.
- Note: spec originally quoted sigma_log10=0.40 but that yields p99≈4.1m, falling short of the "rare giants reach 5m+" goal; 0.45 gives p99≈5.4m and ~1-2% giants (>5m), matching the intended trophic pyramid.
- Target mix: ~50% <0.5 m, ~33% 0.5–1.5 m, ~12% 1.5–3 m, ~4% 3–5 m, ~1–2% >5 m (giants rare, ~1 in 50–100).
- FOV: left UNIFORM — real FOV is bimodal by niche (panoramic prey vs forward predator), so a bell curve would be LESS realistic. Future hook: weakly bias FOV narrow-for-large.
- Why: body size is multiplicative; log-normal matches the real mammal size spectrum and restores a functional energy pyramid / trophic stack (vs the uniform "half are giants" smear). Side effect: large perception-cost reduction (giants dominate the L1 cone scan).
- Scope: spawner sampling only; the genome gene stays a normalized [0,1] primitive so mutation/inheritance keep operating on the raw gene.
- Tuning knobs: raise median → bigger world; raise sigma_log10 → fatter giant tail. Impl: rand_distr 0.4 LogNormal (pinned for determinism).

---

## FOV ↔ Trophic Position — Do Not Hard-Gate (2026-06-27)

**Decision:** `fov_gene` remains full-range (160°–340° attentional cone) for all creatures. No trophic clamp.

**Why:** Trophic role correlates with FOV but doesn't cause it — eye placement (frontal vs lateral) is the real driver. Hard-gating would forbid real archetypes: tarsier (prey, narrow FOV for arboreal depth), dragonfly (apex predator, ~360° panoramic), praying mantis (predator, wide + binocular wedge). The correlation should emerge from fitness pressure, not gene range restriction.

**How to get the correlation emergently — three opposing costs derived from `fov_gene`:**
1. **Frontal blind spot** grows above ~300° (rabbit/horse model) → wide-FOV predators miss strikes
2. **Binocular overlap** (depth/strike accuracy) scales inversely with FOV → narrow = accurate closing, wide = can't judge distance
3. **Rear blind arc** = 360° − FOV → narrow-FOV prey get ambushed, drifting lineages wide

**Floor note:** `MIN_FOV_DEGREES = 45.0` is defensible as an attentional-cone floor but should be treated as an exotic rare adaptation; realistic carnivore cluster is 160–220°.

**Optional:** Soft mutation bias toward trophic-appropriate band without closing off the full range (not a clamp, just a prior).

**Full design:** `docs/biology/todo/dna-driven-fov.md` (Trophic Gating section)

---

## Feeding-State Vigilance & Threat Modulation (2026-06-27)

**Topic:** How active eating changes perception/FOV, and how hunger modulates threat-flee threshold.

**Key findings:**

- **Feeding and vigilance are mechanically incompatible** — head-down posture collapses the forward detection cone. Model as `feeding_fov_multiplier` applied during bite bouts only.
- **Herbivores:** 50–70% effective FOV reduction during bite; BUT retain wide peripheral arc (laterally-placed eyes, 270–340° natural). Interrupt to scan every 4–10s. Ungulate vigilance: 20–50% of foraging time head-up.
- **Carnivores at kill:** 70–90% reduction; frontal eyes → no panoramic fallback. Vigilance: 5–15% of feeding time. NOT globally blind — still actively track same-tier/competitor threats (kleptoparasitism). Use threat-type selectivity, not global perception collapse.
- **Startle asymmetry:** Detection frequency ↓ during feeding, but response magnitude ↑ on successful detect. Apply `FEEDING_STARTLE_MULTIPLIER` (~1.4–1.8×) when threat breaks through during bite bout.
- **Reluctance to leave food = ghrelin vs cortisol tug-of-war.** Not a special state — emerges from: `flee_threshold = base + hunger_risk_tolerance × (1 - energy_fraction)`. Starving animal tolerates moderate threats; satiated animal (leptin↑) flees readily. Continuous, not binary.
- **Satiety flightiness** (leptin): well-fed animals flee at lower threat level. Model: `flee_threshold -= satiety_flightiness × energy_fraction`.
- **Many-eyes effect:** Individual vigilance ∝ 1/conspecific_density. Thomson's gazelle drops from ~40% solo to ~10% in groups of 20+. Also a Golden Zone: herd members run fewer perception evaluations.
- **Satiated carnivores lift head:** as energy fills (leptin rises), feeding intensity drops, vigilance returns, animal guards rather than eats. Natural from `feeding_intensity = (1 - energy_fraction)`.

**Recommended DNA genes:** `feeding_fov_multiplier` (0.1–0.6), `vigilance_interval` (1–60s), `hunger_risk_tolerance` (0.0–1.0), `satiety_flightiness` (0.0–0.8), `social_vigilance_sensitivity` (0.0–1.0).

**Herbivore/carnivore split emerges** from these genes + `trophic_position` — no species flag needed.

**Golden Zone:** While `is_feeding`, skip long-range scan; run only short-range + same-tier threat filter. Full scan on vigilance interrupt only.

**Full design:** `docs/biology/ideas/feeding-vigilance.md`

---

## Conspicuousness / Visibility Allometry (2026-06-28)

**Topic:** How far away a creature can be DETECTED by others as a function of its body size. Fixes "giants seen too late" — today a target contributes only its physical radius (`length/2`) to detection distance, so a 10 m giant announces itself no more than its own skin while its *own* perception range is ~211 m. Asymmetry: observer range scales `~length^1.25` but target visibility was flat.

**Consult:** zoologist-tom.

**Key findings:**
- **Biological driver = apparent angular size** (acuity-limited): a target of size L is detected when its subtended angle `θ ≈ L/D` exceeds the observer's minimum resolvable angle → `D ∝ L^1` (pure acuity). The dual of how observer perception range is already built.
- **Area summation (Ricco's law)** pushes the exponent up toward `L^2` ("you can't hide an elephant"); **atmospheric extinction** (Koschmieder) trims the far tail sub-linear. Real animals live at **k ≈ 1.0–1.5**.
- **Motion** is a separate strong multiplier (frog/deer vision) — reserved as a FUTURE lever, not first-order geometry.
- Target visibility should scale **slightly faster** than observer reach (k=1.5 vs 1.25): acuity has diminishing returns with eye size, but silhouette grows ~L² — *being-seen outpaces seeing*. "A giant is a lighthouse before it is a telescope."

**Recommended formula (drop-in):**
`conspicuousness(length) = 0.71 * length^1.5`, clamped `[0.1, 60.0]` m, **replacing** the `target.radius` term in the detection-distance check (radius/mass untouched — no double-count). Coefficient pinned so `conspic(0.5) = 0.25` → the median (98% of pop) is **unchanged**, only rare giants gain. Magnitudes: 1 m→0.71, 2 m→2.0, 5 m→7.9, **10 m→22.4** (vs old 5.0 = +4.5×). Median crit spots a 10 m giant at ~27.6 m instead of ~10.25 m (~2.7× earlier); giant-spots-giant barely moves (+8%). Boost concentrates in the *small-observer → giant* channel.

**Trophic implications + counterbalance:** suppresses giants on BOTH sides (prey evade apex earlier → apex feeding ↓; grazer-giants found earlier → predation ↑). Self-consistent with 1–2% giant frequency but apex-canary-sensitive. **Counterbalances reserved as future levers:** (1) motion-gated conspicuousness `conspic *= f(speed)` — strongest, a Golden-Zone (frozen ambush giant goes dark AND skips detection work); (2) `crypsis_gene` multiplier `[0.3,1.0]`; (3) size-domination de-weighting (small predators decline giant prey — "seen ≠ predated"); (4) herd detection.

**Trophic-canary protocol:** baseline vs patched, same seed; **reject if apex OR grazer steady-state pop shifts >±20%.** First mitigation if apex breaches = motion-gate; gentler fallback curve `k=1.3, C=0.616` (tune **k not C** — C lifts the 98%, k lifts only giants).

**DNA future slot:** pure function of `length` today (no gene). Natural extension is a `crypsis_gene` multiplier — keep signature `conspicuousness(length, crypsis=1.0)`-shaped so it drops in without re-plumb.

**Implementation Status:** ✅ Implemented on branch `feat/conspicuousness-visibility` (engine + perception wiring + selected-creature overlay ring, all TDD-covered). **NOT merged to main** — held behind the trophic-canary gate, which is **not yet runnable** (engine has no birth/death/reproduction, so steady-state apex/grazer populations cannot shift). Full writeup: `docs/biology/done/conspicuousness-visibility.md`.
