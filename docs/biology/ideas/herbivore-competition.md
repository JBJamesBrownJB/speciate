# Herbivore Competition & Threat Assessment

## Problem / Opportunity

When multiple herbivores converge on the same plant cell, they currently behave as if the
other is invisible — both feed without contest. Real herbivores negotiate access through
postural threat display and size-based bluff: a larger animal simply charges or looms, and
the smaller one yields before contact. Implementing this creates believable resource
dominance hierarchies at zero additional energy cost to the engine, because the winner eats
and the loser moves on — fewer creatures lingering per cell, not more.

The deeper structural need is a reusable **threat assessment** — a pure function that any
behavior (flee, contest food, hold ground, challenge for space) can call to get a calibrated
danger level for a perceived entity. Without a shared assessment layer, each behavior
reinvents its own ad-hoc size/speed/type heuristic, leading to inconsistent creature
responses across contexts.

## Proposed Solution

### Threat Assessment (foundational infrastructure)

A decoupled, side-effect-free function (no ECS writes, no resource mutation) that takes the
perceiver's own stats and the perceived entity's signal and returns a **threat score** in
`[0.0, ∞)`:

```
threat(perceiver, target) → f32
```

Inputs from the target signal:
- **Effective size** — raw body radius, or the flock's composite effective radius if the
  target is a murmuration tight-core (see `murmuration-collective-signal.md`). A flock of
  50 small herbivores may score larger than a single bull.
- **Closing speed** — relative velocity projected onto the approach vector. A fast-approaching
  target is more threatening than a stationary one of the same size.
- **Classification** — `herbivore` (food competitor, lower base threat) vs. `predator`
  (existential threat, much higher base threat). Species classification lives in DNA and is
  readable from the perception layer without a lookup.

The function is cheap enough to call per visible entity in the perception cone without
throttling. It has no state and no memory — the behavior layer decides what to do with
the score.

### Herbivore Competition Behavior

When a creature arrives at a plant cell already occupied by another herbivore:

1. Run `threat(self, occupant)`. If `threat < 1.0` (perceiver is dominant), issue a
   **contest** — charge toward the occupant at elevated speed, briefly skipping normal
   steering forces.
2. The occupant, on its next perception update, sees the incoming challenger. If
   `threat(occupant, challenger) > 1.0` (challenger is larger), the occupant yields —
   its behavior transitions to flee/wander rather than continue eating.
3. If scores are close (`threat ≈ 1.0`), neither party yields cleanly: both circle the
   cell briefly. The one that exits first (random-weighted toward the smaller) leaves the
   food. This "standoff" resolves in 2–4 ticks, not instantly.

This does **not** require direct entity-to-entity communication or new IPC. Both creatures
run the same perception cone and make independent decisions that happen to be mutually
consistent.

## Golden Zone

**Contest logic reduces dwell time per cell.** Under pure co-feeding, N herbivores all slow
to eat the same cell simultaneously. With competition, only the dominant one feeds; the rest
scatter. Peak occupancy per plant cell drops from N to ~1. This reduces the future load of
`update_plants` (fewer simultaneous depletors per cell), and yields tighter creature
clustering that compresses perception working sets — fewer distinct entities in any one
creature's perception cone.

**Threat assessment short-circuits expensive flee logic.** A pure flee path currently
evaluates each threat independently. If `threat assessment` is a shared gate, a creature
that scores all neighbors as non-threatening can skip the avoidance steering phase entirely
— same Golden Zone as size-domination skips. The assessment is cheap; the saved avoidance
steering is not.

## Trade-offs

- **Assessment must be conservative for predators.** A false negative (classifying a predator
  as low-threat) is lethal. The function should err toward over-estimating predator threat
  score. Food competition can afford occasional miscalibration; flee cannot.
- **Standoff cost.** Two creatures of equal size repeatedly contesting the same cell burn
  energy approaching each other without eating. This is biologically accurate (contest costs)
  but needs a timeout: after ~4 ticks of unresolved standoff, both give up and wander. Prevents
  two-creature deadlocks permanently blocking a plant cell.
- **Flock effective size injection.** The murmuration composite signal is an L1-layer construct.
  The threat assessment needs to consume effective radius from there, not from raw L0 per-creature
  data. This couples the two systems; the assessment function must accept an abstract signal
  struct, not a concrete creature reference.
- **No hardcoded species hierarchy.** Threat rank must emerge from DNA-expressed size, speed,
  and classification genes — not a fixed "species A beats species B" lookup table. The
  assessment function encodes the *rules*, not the *rankings*.

## Dependencies

- Plant grid (`PlantGrid`, `update_plants`) — the contested resource that triggers competition
- Threat assessment function (new, standalone — a natural first deliverable since it's pure and
  testable in isolation)
- Murmuration collective signal (optional enrichment — effective radius fed into assessment)
- Behavior state machine extension — a `Contesting` state alongside the existing Wandering /
  Fleeing / Feeding states
- DNA classification gene distinguishing herbivore from predator (currently hardcoded; needs
  migration)

## Related Ideas

- [`size-domination.md`](size-domination.md) — the general "larger bullies smaller" principle;
  herbivore competition is the food-specific instance
- [`murmuration-collective-signal.md`](murmuration-collective-signal.md) — effective radius
  from a murmuration flock is one of the three threat inputs; a lone herbivore facing a tight
  flock may yield even if individually larger
- [`energy-vigilance.md`](energy-vigilance.md) — hungry creatures scan more aggressively;
  a starving herbivore is more likely to contest food (lower yield threshold) than a satiated one
- [`stress-tunnel-vision.md`](stress-tunnel-vision.md) — high-threat contexts narrow perception
  cone; the threat assessment score could gate stress level, which then feeds back into perception

## Open Questions

- What is the threat threshold for yielding? `threat > 1.0` is obvious in principle but the
  exact crossover (1.05? 1.5?) determines how much overlap co-feeding species tolerate. Needs
  playtesting once the plant system produces meaningful depletion.
- Should threat score be continuous (smooth avoidance gradient) or stepped (yield / standoff /
  contest)? Stepped is simpler to implement and debug; continuous enables richer blended
  behavior but is harder to tune.
- Does the `Contesting` behavior state need to be DNA-gated? Some species might be constitutively
  yielding (no contest gene) and some constitutively dominant. A `contest_boldness` gene could
  scale the yield threshold.
- How does threat assessment interact with predators that are *also* near the plant cell? A
  herbivore in contest mode that suddenly perceives a high-threat predator should immediately
  abandon the contest — the behavior priority order needs specifying.

---
*Captured: 2026-06-26*
