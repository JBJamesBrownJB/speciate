# Aquatic & Vertical Habitat Layers

## Problem / Opportunity

Today every creature lives in one flat plane — all crits can reach all other crits. Real
ecosystems are stratified: fish can't leave water, most land animals can't enter it, and a
few amphibious specialists exploit both. Adding a **vertical habitat axis** (underwater /
surface / land / air) turns the world from one arena into several overlapping ones, creating
refuges, niche separation, and a whole new axis for emergent predator-prey dynamics — all on
the existing 2D engine, no real 3D required.

## Proposed Solution

Each creature has a **habitat affinity** governing which medium(s) it can occupy:

- **Water-bound** — can only exist in water (fish). Dies / is blocked on land.
- **Land-bound** — cannot enter water (most terrestrial animals).
- **Amphibious / transitional** — can move between water and land, but pays a **penalty** in
  the non-preferred medium and/or when crossing the boundary (a swim penalty vs a land
  penalty — energy drain, reduced speed, or slower turning). Exact mechanics TBD.

The same layering generalises **upward** to flight (air layer) — a flyer escapes
ground-bound threats the way a fish escapes by depth. Habitat affinity is a natural **DNA
trait/gene**, so it mutates and inherits like other traits, and the water/land/air mix of a
population emerges rather than being authored.

The decisive consequence is **cross-layer separation of interaction**: a land predator
simply *cannot* reach a fish, and a fish can't be hunted by it. Amphibians become the bridge
species that hunt across layers. Prey gain a real escape verb — **dive or take flight to a
layer the predator is locked out of** — which ties directly into the conspicuousness and
flight-initiation-distance work (the chase ends at the water's edge or the ground).

## Golden Zone

**Strong.** Cross-layer separation is an optimisation that *is* the biology: a creature only
needs to perceive / evaluate threats and prey **in layers it can actually interact with**.
A land predator skips all underwater entities entirely — fewer perception candidates *and*
the correct behaviour (it ignores the fish because it genuinely can't catch it). The refuge
mechanic (prey dives to safety) is the player-visible payoff of the same skip. Layer
membership could also bucket the spatial grid, shrinking neighbour queries per creature.

## Trade-offs

- Adds a habitat dimension to creature state and to perception/interaction filtering.
- Needs a water-vs-land substrate to exist in the world (depends on terrain/biomes).
- Balancing the transition penalty so amphibians are a viable-but-costly niche, not strictly
  dominant or strictly useless.
- "What happens at the boundary" (a land-bound crit pushed into water; a fish beached) needs
  defined rules — block, damage, or forbid pathing.

## Expert Input

Not yet consulted — logged for later. When picked up, run a `dna-consult` (zoologist-tom) on
realistic medium-transition costs and amphibious niche trade-offs, and `ecs-emma` on whether
habitat affinity should be a capability-marker (ZST) for archetype-level layer filtering.

## Dependencies

- A **water/land substrate** in the world (terrain or biome layer defining where water is) —
  see Related.
- The DNA system (habitat affinity as a gene).
- Perception/interaction filtering able to gate by layer membership.

## Related Ideas

- `docs/visuals/ideas/depth-altitude-sprite-encoding.md` — the visual cue that makes layers
  readable on a 2D renderer (this idea's rendering half).
- `docs/terrain/cellular-automata-terrain.md` — could generate the water regions the layers sit on.
- `docs/biology/done/conspicuousness-visibility.md` & `docs/biology/ideas/flight-initiation-distance.md`
  — refuge-by-diving/flying is the escape side of seeing & flee-timing.
- `docs/gameplay/ideas/high-altitude-drone.md` — existing flight-adjacent concept.

## Open Questions

- Penalty model: energy, speed, turn-rate, or a mix? Per-medium or only at the crossing?
- Is "air" a first-class layer now, or just noted as the symmetric extension?
- Discrete layers vs a continuous depth/altitude scalar per creature?
- Boundary rules (forbidden pathing vs damage vs forced transition).
- Does habitat affinity interact with size (big things can't use shallow water, etc.)?

---
*Captured: 2026-06-28*
