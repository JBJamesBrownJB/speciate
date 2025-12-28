# Apex Pheromone Harvesting

## Problem / Opportunity

Thumper deterrents require continuous energy and technology unlocks. Players need an alternative protection strategy that rewards dangerous exploration and hunting - a consumable resource system that mirrors real ecology (harvesting predator scent to deter prey).

## Proposed Solution

**Collect pheromones from apex predators** and apply them to machines or player avatar. Creatures smaller than the apex predator in the food chain will avoid the area, believing a dangerous predator is present.

### Collection Mechanics

**Sources (all biologically grounded):**
- Territorial scent marks (urine deposits on rocks, trees, substrate)
- Scat/feces (contains anal gland secretions)
- Carcass sites (residual fluids from kills)
- Bedding areas (accumulated skin/gland secretions)

**Active Collection:**
1. Player locates fresh territorial marks (visual indicators: discoloration, disturbed ground)
2. Use extraction tool (consumes time, makes noise → attracts creatures)
3. Yields raw pheromone sample

**Passive Collection:**
1. Place automated "scent trap" in high-traffic apex territory
2. Lures predators to mark the trap (energy cost, bait cost)
3. Periodic extraction when trap is full
4. Risk: trap may attract predator while player is harvesting

### Processing Chain

Raw material requires refinement:

```
Raw secretion → Extraction → Concentration → Stabilization → Deployable pheromone
```

Each stage:
- Requires machinery/technology unlock
- Improves potency and duration
- Early game: crude extracts (weak, short duration)
- Late game: synthetic blends (customizable, long-lasting)

### Application

**Deployable forms:**
- Spray bottle (manual application to machines, self)
- Automated dispenser (slow release, long duration, energy cost)
- Scent beacon (area denial, high concentration)

**Potency decay:**
```
potency(t) = initial_potency × e^(-decay_rate × t)
```

| Environment | Half-Life | Gameplay Impact |
|-------------|-----------|-----------------|
| Hot/dry | 2-4 game hours | Frequent reapplication required |
| Temperate | 8-12 game hours | Standard protection window |
| Enclosed (buildings) | 48+ game hours | Long-term machine defense |

Players must monitor protection levels and reapply before potency drops below deterrent threshold.

## Golden Zone

| Optimization | Free Biological Behavior |
|--------------|--------------------------|
| Wind-direction culling | Only check pheromone influence for creatures downwind - upwind creatures genuinely cannot detect scent (skip computation) |
| Panic-state perception skip | Creatures in fear zone (inside pheromone radius, fear > panic threshold) skip normal perception and execute flee behavior directly |
| Species-specific vs. apex-general | Specific pheromones = strong effect on fewer species (targeted computation), generic = weak effect on many (broad computation) |

**Wind creates strategic depth:** Pheromone protection is directional. Player must:
- Monitor wind direction (visual indicator)
- Reposition defenses when wind shifts
- Place machines with wind patterns in mind (prevailing winds protect more reliably)

## Trade-offs

**Hunting danger:** Collecting from apex predators requires entering their territory. Risk scales with reward - strongest deterrents come from most dangerous creatures.

**Resource scarcity:** Apex predators are rare (top of food chain, low population density). Pheromone supply is limited, forcing strategic allocation.

**Consumable vs. renewable:** Unlike energy (renewable via generators), pheromones deplete. Player must choose between hoarding (security) and using (exploration).

**Processing investment:** Raw pheromones are weak. Effective protection requires machinery investment (upfront cost, space, energy drain).

**Freshness matters:** Older pheromone sources yield weaker extracts. Player must track apex territories and time collection carefully.

## Expert Input

### Zoologist (zoologist-tom)

**Biological validation:**
- Real-world landscape of fear: elk avoid 94% of cougar-marked areas, deer abandon habitat with wolf urine
- Pheromones encode predator identity (species-specific), fitness (parasite load, health), and recency (volatile compound decay)
- Collection is plausible: commercial deer repellents use predator urine, researchers harvest scent marks routinely

**Recommended system design:**

| Pheromone Type | Collection Source | Effect Scope | Potency |
|----------------|-------------------|--------------|---------|
| Species-specific | Fresh marks from identified apex | Deters that species' natural prey only | Very strong |
| Genus-level | Mixed samples from related species | Partial deterrence (related species overlap) | Medium |
| Apex-general (synthetic) | Lab-produced blend | Weak broad deterrence | Weak |

**Progression arc:**
- Early: collect crude samples (species-specific only, short duration)
- Mid: refine processing (genus-level blends, longer duration)
- Late: synthesize custom pheromones (adjustable risk/reward profiles)

### Complementary to Thumpers

**Thumpers:** Broad-area, low-maintenance, energy-continuous, technology-locked
**Pheromones:** Targeted, high-maintenance, consumable, exploration-locked

**Combined defense strategy:**
- Perimeter thumpers deter medium threats (constant cost)
- Apex pheromones on critical machines (renewable via hunting, no energy)
- Player manages dual economy: power generation + pheromone stockpiles

## Dependencies

- Apex predator creatures with territorial behavior (marking, patrolling)
- Chemical scent system (see `chemical-scent.md` - signal architecture)
- Wind direction simulation (environmental factor for scent propagation)
- Inventory and crafting system (collect, refine, store, deploy)
- Fear/flee behavior in prey creatures (response to predator cues)

## Related Ideas

- `chemical-scent.md` - Foundation pheromone signal architecture (required)
- `machine-thumper-deterrent.md` - Complementary deterrent system (energy-based)
- `pheromone-mate-attraction.md` - Side-effect risk of using pheromones (creates strategic tension)
- `dna-collector.md` - Similar hunting/collection gameplay loop (harvesting from dangerous creatures)

## Open Questions

- Should pheromone quality degrade if player stores too long (inventory decay)?
- Can creatures detect "fake" pheromones (synthetic blends less effective over time)?
- Should different body parts yield different pheromone strengths (glands > urine > scat)?
- Do pheromones stack (multiple applications = stronger effect)?
- Can players "contaminate" pheromones by mixing incompatible species?
- Should environmental events (rain, storms) wash away applied pheromones?

---
*Captured: 2025-12-28*
