# Murmuration — Flock as Collective Signal

## Problem / Opportunity

Individual creatures in a tight flock are perceived and avoided one-by-one by outsiders,
which neither reflects real predator/prey biology nor gives the engine any opportunity to
collapse redundant per-member work. A sufficiently tight, coordinated group should present
to the outside world as a single large entity — too big to attack, too coherent to resolve
into individuals — while stragglers on the fringe remain individually visible and attackable.

## Proposed Solution

Creatures within a **tight core** — defined by having at least K neighbors within ~2 body-lengths
(local density, not distance-to-centroid) — collectively project a **composite signal** into the
L1 spatial layer. Outsiders perceive and steer around this composite as if it were one large
creature. Creatures outside the tight core (stragglers) appear as ordinary individuals.

The tight-core threshold is **DNA-driven** via a `cohesion_preference` gene that encodes
preferred neighbor spacing in body-lengths. Species with wide spacing never reach the
murmuration threshold; species with tight spacing produce strong collective signals. Even within
a tight-flocking species, creatures at the edge of the spacing range drift in and out of
the tight core each tick — the collective signal pulses and breathes organically.

**Hysteresis is required:** enter the tight core at ≤2 BL spacing, leave at ≥3 BL. Without
hysteresis, edge members flicker every tick and the signal is noisy.

The **effective radius** of the composite signal scales with the spatial envelope of the dense
core (approximately N^0.5 of member count), capped near the largest real creature size.
Predators with an attack-size threshold will decline to engage a flock whose effective radius
exceeds that threshold — biologically, this models both confusion (can't lock an individual)
and intimidation (object too large to attack profitably).

## Golden Zone

**The optimization IS the confusion effect.**

A tight flock collapses N individual perception targets into one aggregate entity. Outside
perceivers query a single composite signal instead of N neighbors; avoidance reduces from
N pairwise force contributions to one centroid-directed force scaled by effective radius.
Perception cost for a 10k-member flock drops from O(N) to ~O(1).

This is biologically accurate: predator confusion is modeled as the engine not iterating
individuals. The expensive computation maps precisely onto the biologically attackable
population — only stragglers outside the tight core get individual checks, and those are
exactly the creatures a predator can actually target.

Complement to the existing size-domination skip ("giants ignore mice"): instead of ignoring
small things, we aggregate many small things into one large thing. Same Golden-Zone family.

## Trade-offs

- **Abstraction honesty:** "flock perceived as one large creature" is a deliberate model of
  confusion + intimidation, not a claim that predators literally misidentify a flock as a single
  animal. The functional outcome (predator declines, stragglers remain attackable) is accurate
  even if the cognitive mechanism is simplified.
- **L1 cell fragmentation:** a flock straddling a 60 m L1 boundary splits across two cells and
  weakens the aggregate signal. This can be made a feature rather than a bug by bounding the
  DNA cohesion threshold so a "tight" flock fits within roughly one L1 cell — cell granularity
  becomes a natural biological cohesion bound.
- **Self-exclusion required:** a member steering toward its own flock centroid needs the full
  aggregate (including self); an outsider avoiding the flock needs self excluded. The composite
  sums must support both views.
- **Effective-mass saturation:** linear scaling (10k members = 10k× mass) produces biologically
  absurd objects that nothing would ever approach. Cap effective radius near the largest plausible
  creature size; intimidation saturates.
- **Velocity coherence gate (optional):** a random crowd in one L1 cell is not a murmuration.
  Consider requiring heading alignment (sum of velocity vectors / N above a threshold) before
  the composite signal activates — distinguishes a panicking scatter from a coordinated flock.

## Expert Input

**Zoologist-tom:**
- Grounded in selfish-herd theory (Hamilton 1971), fish bait balls, musk-ox rings, defensive
  aggregations. The tight-core/straggler split is the general structure of aggregation defence,
  not a starling special case.
- "Flock as one creature" primarily models confusion effect + prey-size preference (predators
  decline objects outside their attack-size envelope), not literal perceptual misidentification.
  Log this as a deliberate abstraction, not literal ethology.
- Local density (≥K neighbors within ~2 BL) is the correct membership criterion. Centroid
  distance breaks at scale — a tight 10k-bird flock is physically huge, so a fixed centroid
  radius would exclude most members.
- Hysteresis (in at 2 BL, out at 3 BL) prevents per-tick flicker for edge members.
- Marginal predation + selfish herd emerge for free: edge = dangerous → selection pressure
  toward center → tight ball maintained → margin perpetually refreshed by drifters.
- Recommend consulting shader-sarah for the visual agitation/density wave — the pulse as
  members enter/leave the tight core maps to a traveling-wave shader.

**ECS-emma:**
- The L1 BioSignature already aggregates mass and creature count per cell — a tight flock is
  already a high-mass/high-count cell. The composite signal is four running sums away:
  `sum_x`, `sum_y`, `sum_x²`, `sum_y²` per L1 cell enable centroid and spatial spread in a
  single pass over the existing aggregate loop (no second iteration, no new system).
- **Hot/cold split strongly recommended:** keep the existing BioSignature lean for the hot
  per-cell classification path; put the flock sums in a parallel structure indexed identically,
  touched only when a Flocking-capable perceiver runs its cone scan. Avoids doubling the L1
  working set for a feature only some creatures use.
- **No separate flock entity.** Spawn/despawn churn as flocks form/dissolve creates
  archetype moves and command-flush sync points — an anti-pattern at 1M population.
- **No per-creature membership component** (archetype churn for dynamic membership every tick
  would kill par_iter throughput). Use: a ZST capability marker `Flocking` (DNA-driven
  eligibility, added at spawn, never removed) + a bit in CreatureState for current tight-core
  status (cheap mutation, zero archetype move). Stragglers are naturally visible as L0
  proxies; only tight-core members contribute to the composite signal.
- Use **L1-cell-local coordinates** for the running sums to avoid f32 cancellation across a
  10 km world.
- Existing schedule order (rebuild → aggregate_l1 → perception → steering) is already correct;
  no new system or sync point needed if the flock sums fold into the existing aggregate pass.

## Dependencies

- DNA system with `cohesion_preference` gene (preferred neighbor spacing in body-lengths)
- Basic flocking forces (alignment + cohesion steering) so creatures can actually form tight
  groups — see `docs/biology/todo/crowding-affinity.md` (DEFERRED) and the `flocking: bool`
  gene in `docs/biology/ideas/dna-driven-design.md`
- L1 BioSignature composite extension (sum_x/y/x²/y² + optional velocity sums)
- Predator attack-size threshold — needed for the "decline flock core" decision rule

## Related Ideas

- [`docs/biology/todo/crowding-affinity.md`](../todo/crowding-affinity.md) — `crowding_affinity` gene for schooling/flocking attraction (DEFERRED; provides the cohesion force this idea depends on)
- [`docs/biology/ideas/dna-driven-design.md`](dna-driven-design.md) — `flocking: bool` gene (the eligibility marker this idea assumes)
- [`docs/biology/ideas/crowd-tolerance-dna.md`](crowd-tolerance-dna.md) — boldness + crowding_tolerance genes; "obligate schooler" phenotype is the target species for murmuration
- [`docs/biology/ideas/flocking-calls.md`](flocking-calls.md) — vocal coordination signals that maintain cohesion when visual contact is lost; complementary layer
- [`docs/gameplay/ideas/environmental-migration.md`](../../gameplay/ideas/environmental-migration.md) — `flocking_tendency` gene; 80% of herd skips individual perception by following leaders (same Golden Zone family)

## Open Questions

- What is K (minimum neighbor count to qualify for tight-core membership)? Starling research
  suggests 6–7 topological neighbors; fish schools 4–7. Needs biological tuning per species.
- Does the composite signal replace L0 individual proxies for tight-core members, or does it
  layer on top? (Replacing risks breaking existing individual-level perception for members
  perceiving each other; layering on top risks double-counting.)
- Velocity coherence gate: should heading alignment be required to activate the composite
  signal, or is density alone sufficient?
- How does this interact with the existing `MAX_PERCEIVED_NEIGHBORS = 7` cap? A creature on
  the edge of a 10k-member flock that sees 7 neighbors is already implicitly in a flock;
  the composite signal would be an additional L1-layer effect on top.

---
*Captured: 2026-06-26*
*Expert input: zoologist-tom, ecs-emma*
*Related: crowding-affinity (DEFERRED), dna-driven-design, crowd-tolerance-dna, flocking-calls*
