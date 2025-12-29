# Environmental Migration via Terrain Manipulation

## Problem / Opportunity

Players need tools to influence creature populations without direct control. Environmental manipulation creates strategic depth where players shape the world to guide emergent behavior rather than commanding creatures directly.

## Proposed Solution

Players can force herd migration by manipulating environmental conditions:

**Push mechanics (make area undesirable):**
- Deplete vegetation through overconsumption or player action
- Reduce water availability (drought mechanics)
- Introduce predators or hazards

**Pull mechanics (make adjacent area attractive):**
- Enrich with plant growth
- Create water sources
- Remove threats

**Trigger sources:**
- Player terraforming actions (direct manipulation)
- Natural seasonal/climate cycles (automatic environmental shift)
- Creature impact (overgrazing depletes, creates natural push)

### Emergent Migration

Migration should NOT be encoded as a discrete behavior. Instead, it emerges from existing DNA primitives:

| Primitive Trait | Effect on Migration |
|----------------|---------------------|
| `resource_sensitivity` | How strongly creature responds to food gradient |
| `exploration_radius` | How far creature searches when local resources deplete |
| `site_fidelity` | Tendency to return to known areas vs explore new |
| `hunger_threshold` | When to abandon current location |
| `flocking_tendency` | How much movement influenced by nearby conspecifics |

High resource_sensitivity + high flocking + low site_fidelity = Migratory species
Low resource_sensitivity + low flocking + high site_fidelity = Territorial species

## Golden Zone

| Optimization | Biological Behavior |
|--------------|---------------------|
| Skip resource scanning when satiated (hunger < threshold) | Well-fed animals rest, don't forage - players observe "lazy" herds |
| Creatures follow local gradient, not global optimal | Animals use local sensing, not GPS - can get "trapped" in local valleys |
| Flocking reduces individual computation | Followers copy leader destination - 80% of herd skips perception |

**Gameplay emergent from optimization:**
- Recently-fed herds harder to move (hunger gating)
- Player can create vegetation "corridors" to guide herds through terrain
- Shooting lead animal causes confusion (followers lose reference)

## Trade-offs

- **Lag time required:** Creatures use local gradient sensing, not instant global awareness - realistic but slower player feedback
- **Hysteresis needed:** Animals shouldn't ping-pong between areas - commitment threshold prevents rapid reversals
- **Stale information:** Density snapshots can mislead creatures toward cells that emptied - creates realistic foraging waves

## Expert Input

**Zoologist-tom consultation (2025-12-29):**

- Strongly biologically plausible - resource-driven migration is fundamental to animal movement
- Serengeti wildebeest follow rainfall/grass patterns via individual foraging decisions aggregated across herd
- Key insight: "Animals do not possess a migration instinct as a discrete behavior" - it emerges from resource preference + avoidance + social following
- Recommended against giving creatures global map awareness - use local gradients only
- Seasonal memory trait (`temporal_memory`) could enable return-to-calving-ground patterns later

## Dependencies

- L1 spatial grid with resource/density information
- Plant/resource depletion and regrowth mechanics
- Hunger system driving foraging behavior
- Flocking/social following system

## Related Ideas

- `docs/biology/ideas/crowd-tolerance-dna.md` - Boldness affects density preferences
- `docs/biology/ideas/l1-border-repulsion.md` - Border avoidance shapes migration paths

## Open Questions

- How fast should vegetation recover after depletion?
- Should player have direct "terraform" tools or only indirect influence (spawn plants, remove water)?
- How to visualize resource gradients to player (heatmap overlay)?

---
*Captured: 2025-12-29*
