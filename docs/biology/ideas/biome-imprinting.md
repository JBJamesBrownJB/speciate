# Natal Biome Imprinting — Emergent Habitat Preference

## Problem / Opportunity

Creatures currently wander without any tie to *where* they belong. Biomes are, in effect, cosmetic backdrop — a crit born in arid scrub behaves identically to one born by water. Real animals are not uniform across a map: they cluster in habitat that resembles where they were born and thrived (philopatry, natal habitat preference, salmon-style natal homing). Without this, populations spread homogeneously, biomes carry no behavioural meaning, and there's no emergent niche partitioning or migration.

We want crits to **gravitate toward / stay near / seek out their preferred biome** — and crucially for that preference to **emerge** rather than be a hardcoded "this species likes deserts" flag.

## Proposed Solution

Two complementary mechanisms — a **setpoint** and a **drive**:

**1. Natal imprinting (the setpoint).** When a crit is born (today: spawned — the engine has no reproduction yet), it samples and stores a compact **signature of its local biome**: aridity, water proximity, dominant plant type, (later) temperature, elevation, cover density, etc. This imprinted signature becomes the creature's notion of "home/normal" — a learned vector carried for life, not a per-species constant. Two siblings born in different biomes grow up preferring different habitat, from identical genetics.

**2. Habitat-comfort drive (the behaviour).** Each tick the creature compares its *current* local biome to its imprinted signature. The mismatch (a distance in biome-feature space) produces a **discomfort** that biases movement back toward matching habitat; a match produces comfort (contentment, reduced urge to roam). This is **one more contribution on the existing drive simplex** — it competes with hunger, threat, social drives, etc., so a starving or fleeing crit will still leave its comfort zone, and only a sated, safe crit indulges its habitat preference. That competition is what makes the result lifelike rather than crits being magnetised to a biome.

The simplex already integrates drives; this idea supplies a *specific new drive signal* plus the **imprinting mechanism that gives each individual its own setpoint** — which is the part the simplex alone doesn't define.

**Inheritance angle (open):** primary proposal is *imprinting* (experience at birth), per the user's framing. A blend is the richer long-term model: a heritable genetic *bias* (canalised over generations — Baldwin effect) plus natal imprinting that fine-tunes within a lifetime. Start with pure natal imprinting; reserve a `habitat_plasticity` gene (how strongly birth-biome overrides inherited bias) as a future lever.

## Golden Zone

Promising — habitat preference doubles as a work-skipper and a spatial-distribution shaper:

- **Contentment throttle:** a crit sitting in well-matched habitat with low discomfort can damp its wandering and run perception/decision work less often (a settled, content animal explores less). Comfort = cheaper to simulate. Discontent (wrong biome) raises activity — exactly when you *want* it spending cycles to relocate.
- **Automatic niche partitioning:** because crits imprint on their *natal* biome (not a globally "best" one), the population self-distributes across biomes instead of all stampeding to one optimum — which spreads load across the spatial grid and reduces local contention, *and* is the biology (different lineages occupy different niches).
- **Emergent migration:** if a biome shifts (drought dries a wetland, grazing strips plant cover — see plant-cover idea), resident crits' comfort drops and they relocate as a wave — migration emerges from the drive, unscripted.

## Trade-offs

- **Per-crit state:** every creature carries an imprinted biome signature vector — more memory and a per-tick compare. Must stay small (a handful of floats) and SoA-friendly; the compare must be cheap (weighted L1/L2 distance, not anything fancy) since it runs in the hot loop or on a throttle.
- **Environment must be queryable:** needs per-location biome features (aridity, water, plant type…) readable cheaply at a creature's position — depends on the procedural environment exposing that data, ideally precomputed per grid cell.
- **Clumping degeneracy:** if the homing drive is too strong, crits freeze in their birth-spot forever. Mitigate via drive competition (hunger/threat pull them out), tolerance width (a *range* of acceptable biomes, not a point), and the fact that imprinting is to the natal biome — not a single global magnet.
- **Defining the signature & metric:** which features, how many, how weighted, and how wide the comfort tolerance — all sensitive dials needing a zoologist pass.

## Expert Input

Not yet consulted — logged on request as a raw idea. Future zoologist-tom pass should pin: imprinted-vs-genetic-vs-blend, the biome-feature dimensions and their weights, comfort tolerance width, and whether preference can *re-imprint* with prolonged experience (acclimatisation) or is fixed at birth. environment-eddy for what biome features the procedural environment can actually expose per location.

## Dependencies

- **Per-location biome data** (aridity, water, plant type, …) the environment can expose at a creature's position — without this there's nothing to imprint on or compare against.
- **Drive simplex** — the integrator this habitat-comfort signal plugs into as one more drive contribution.
- **Birth/reproduction** for *true* natal imprinting across generations — not present yet (engine has no birth/death). Until then, imprint at spawn; the homing drive works regardless of how the crit came to exist.

## Related Ideas

- `docs/biology/ideas/sensory-simplex.md` and the drive-simplex it references — the drive architecture this becomes an input to (this is the *where-I-want-to-be* signal; the simplexes are *how I sense* and *how drives combine*).
- `docs/biology/ideas/crowd-tolerance-dna.md` — a sibling "inherited/individual preference shapes movement" pattern; same shape, different axis (density vs habitat).
- `docs/biology/ideas/plant-cover-height.md`, `docs/biology/ideas/aquatic-habitat-layers.md` — plant type and water/medium are biome-signature dimensions this would read.
- `docs/biology/ideas/herbivore-competition.md` — grazing reshapes biomes, which would drive emergent migration via this preference.
- `docs/biology/ideas/burrowing.md`, `docs/biology/ideas/memory.md` — related home-site / spatial-memory mechanics.

## Open Questions

- Imprinted, genetic, or blended (and is there a `habitat_plasticity` gene weighting the two)?
- Which biome dimensions form the signature, and how are they weighted in the distance metric?
- How wide is the comfort tolerance — a point preference, or a band?
- Does preference re-imprint with prolonged time in a new biome (acclimatisation), or is it locked at birth?
- How strong is the habitat-comfort drive relative to hunger/threat/social on the simplex?
- Does it interact with the conspicuousness/cover work — e.g. preferring biomes that also offer concealment?

---
*Captured: 2026-06-28*
