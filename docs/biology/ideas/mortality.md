# Mortality

**Category:** 💡 Idea — not yet implemented.

The canonical home for *how creatures die*. Today the simulation has no creature-level
mortality: entities are only removed by bulk teardown (`apps/simulation/src/simulation/core/simulation.rs`,
`spatial/systems.rs`). Death is referenced as a consequence in many docs (starvation,
predation, selection) but is not centralized or built. This doc is that center.

## Why it matters

- **Population dynamics.** Without death, populations only grow. Mortality is what creates
  carrying capacity, boom–bust cycles, and ecological balance.
- **Selection pressure.** Death is the other half of the genetic algorithm — fitness only
  means something if the unfit die. See `population-genetics-algorithm.md`.
- **Stakes.** Predation, starvation, and scarcity are only meaningful if they can be fatal.

## Death causes (emergent — DNA/physics-driven, not scripted)

- **Starvation** — energy reaches zero. See `energy-vigilance.md`, `../todo/hunger-gating.md`,
  `brain-energy.md`.
- **Predation** — killed by an attacker. See `attack.md`.
- **Future** — age / senescence, environmental hazards (`seismic-impacts.md`).

## Implementation note — deferred `Dead` marker (for when this is built)

Death must not despawn in the hot path. Despawning an entity mid-tick moves it between Bevy
archetypes, reshuffling the contiguous component tables and evicting cache lines — costly at
500K+ creatures, and the opposite of the archetype-stability the engine relies on.

Instead, when mortality is implemented:

- Add a zero-sized `Dead` marker component on death; systems exclude dead entities with a
  `Without<Dead>` filter.
- Defer the actual removal to a single batched cleanup pass (end of tick / periodic), so
  layout churn happens once, in bulk, off the hot path.
- This mirrors the capability-marker principle (markers added at spawn, never removed) that
  keeps archetypes stable. See `../../architecture/core-architectures.md` (ECS Capability Markers).

**Open question — corpses/biomass.** Does a dead creature leave a resource (carcass → biomass
for scavengers/plants) during the deferral window, or vanish at cleanup? Ties into the
biomass/botany systems and scavenging behavior.

## Related

- `population-genetics-algorithm.md` — death closes the selection loop
- `attack.md` · `energy-vigilance.md` · `../todo/hunger-gating.md` · `brain-energy.md`
- `../../architecture/core-architectures.md` — capability-marker archetype stability
