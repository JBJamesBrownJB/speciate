# Cover / Concealment — Foliage & Obstacles Reduce Conspicuousness

## Problem / Opportunity

Today a creature's detectability (conspicuousness) depends only on its own body — size, and soon motion. But in nature, *where* an animal stands matters as much as what it is: a deer in open grassland is exposed; the same deer in dense brush is nearly invisible. Prey use cover to break line-of-sight and stalking predators use it to close distance unseen. Adding **environmental concealment** turns the map itself into a tactical layer — refuge, ambush lanes, and exposed killing grounds emerge from terrain rather than being scripted.

## Proposed Solution

Make a creature's effective conspicuousness **scale down when it is near or inside cover** — dense foliage, tall plants, rocks, or other obstacles. The denser/taller the cover at the creature's location, the larger the visibility reduction, down to some floor (cover hides, it doesn't fully cloak — a moving creature still betrays itself).

This is a **third multiplicand** on the same conspicuousness chain that size and motion already use:

> `conspicuousness = base(size) · motion_factor(speed) · concealment_factor(local_cover)`

Keeping it multiplicative preserves the established model: any one channel collapsing toward zero (frozen *and* hidden) makes the creature nearly undetectable — exactly the deer-in-brush case. It also composes cleanly with the planned `crypsis_gene` (an *intrinsic* camouflage trait) — concealment is the *extrinsic*, location-derived analogue.

The "amount of cover at a location" is supplied by the environment — see the linked **plant-height / cover** idea, where DNA-gated, growth-dependent plant height feeds a per-location cover value. Cover could also come from terrain obstacles (rocks, burrow mouths, water edges) later.

## Golden Zone

Strong candidate, but **needs thought** (flagged by the user as not-yet-fleshed-out):

- **Cheap concealment skip:** an observer can early-out on a target sitting in high cover before running the full detection math — concealment becomes both the gameplay "it's hidden" *and* a perception-work skip, mirroring the motion-detection skip. "In thick brush = unseen AND cheap to ignore."
- **Emergent habitat use:** if hiding lowers detection, creatures that learn/evolve to linger near cover survive better — refuge-seeking and ambush-from-cover emerge from the optimization, not from scripted AI.
- **Caution:** querying local cover per creature per tick could *cost* more than it saves unless the cover value is precomputed into the spatial grid (e.g. a per-cell cover scalar refreshed when plants grow), the way conspicuousness is already precomputed at grid rebuild. The perf win is real only if the lookup is O(1) off an existing structure.

## Trade-offs

- **Spatial coupling:** conspicuousness stops being a pure function of the creature and becomes location-dependent — it must be (re)evaluated as creatures move and as cover grows/changes, not computed once at spawn.
- **Balance risk:** cover helps prey (refuge) *and* predators (ambush) simultaneously; net trophic effect is non-obvious and must be canaried (detection-rate proxy only — the engine has no birth/death yet).
- **Cover-camping degeneracy:** if hiding is strictly dominant, everything just parks in foliage forever. Needs a cost (cover is sparse, or foraging/energy pulls creatures into the open, or cover slows movement).
- **Defining "local cover":** point-sample at the creature's cell, or area-weighted over a radius? Does cover block a line-of-sight ray, or just attenuate a radius? (Ray-based is realer but far costlier — likely a later refinement; start with attenuation.)

## Expert Input

Not yet consulted — logged on request as a raw idea ("needs some thought"). A future zoologist-tom pass should pin: the concealment floor (how dark can pure cover get without motion?), whether cover attenuates linearly or saturates with density, and the cover-vs-crypsis-vs-motion interaction (does motion defeat cover the way it defeats camouflage?).

## Dependencies

- The conspicuousness system (size-based, shipped) and its motion-gated upgrade — concealment slots in as the next multiplicand on the same chain.
- A **per-location cover value** from the environment — primarily plant height/density (see Related). Without an environmental cover source this idea has nothing to read.
- Ideally a spatial structure that already carries per-cell environmental data, so the cover lookup is O(1) on the hot path.

## Related Ideas

- `docs/biology/ideas/plant-cover-height.md` — the flora side: DNA-gated, growth-dependent plant height that *produces* the cover value this idea consumes. **These two are a pair.**
- `docs/biology/done/conspicuousness-visibility.md` — v1 size-based conspicuousness (the base multiplicand).
- `docs/biology/todo/motion-gated-conspicuousness.md` — motion multiplicand; also reserves `crypsis_gene` as a separate intrinsic-camouflage multiplicand that concealment parallels.
- `docs/biology/todo/motion-detection.md` — observer-side skip; the concealment skip would be a sibling optimization.
- `docs/biology/ideas/burrowing.md`, `docs/biology/ideas/aquatic-habitat-layers.md` — other "leave the visible layer" refuge mechanics.

## Open Questions

- Concealment floor: how invisible can pure cover make a *stationary* creature, and does movement punch through it (like it does crypsis)?
- Cover from plants only, or also terrain obstacles (rocks, burrows, water edges)?
- Point-sample vs area-weighted vs line-of-sight occlusion?
- How is "cover-camping" costed so foliage isn't a strictly-dominant park spot?
- Does a creature's *own* size interact with cover (a 10 m giant can't hide in 1 m grass)? Likely cover effectiveness should scale with `cover_height / creature_size`.

---
*Captured: 2026-06-28*
