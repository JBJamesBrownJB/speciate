# Chemical Scent: Pheromones and Scent Marking

**Status:** Idea (DNA-driven signal type)

**Depends on:** `signal-broadcasting.md` (foundation architecture)

**Related:** `stigmergy.md` (complementary - environmental modification vs real-time signals)

## Problem / Opportunity

Chemical signaling (scent) differs fundamentally from seismic and vocal signals:
- **Persistence:** Chemicals linger long after the emitter leaves
- **Trails:** Movement creates scent paths that can be followed
- **Territory:** Scent marks define boundaries

This enables tracking, territory, and delayed communication that other signal types cannot provide.

## Proposed Solution

### Emission: DNA-Driven Trait

Chemical signaling requires specialized DNA traits:

**Scent Gland Gene:**
- Encodes ability to emit chemical signals
- Affects scent strength (detectability range)
- Affects scent character (species/individual signature)
- Energy cost per emission

**Emission Types:**

| Type | Trigger | Duration | Purpose |
|------|---------|----------|---------|
| Passive trail | Movement | Long (50-200 ticks) | Tracking, territory |
| Active marking | Deliberate action | Very long | Territory boundaries |
| Alarm pheromone | Threat detection | Medium (10-30 ticks) | Warning conspecifics |
| Mating pheromone | Reproductive state | Long | Mate attraction |

### Persistence and Decay

Chemical signals have the **longest persistence** of all signal types:
- Half-life: 50-200 ticks (vs 2-5 for seismic, 5-10 for vocal)
- Enables detection of creatures that passed through hours/days ago
- Creates "scent history" in L1 cells

### Trail Following

When a creature moves, it leaves a scent trail:
- L0 cells along path accumulate scent
- Predators with scent-sensing trait can follow the trail
- Trail strength indicates time since passage (fresher = stronger)
- Direction inference: scent gradient points toward recent passage

### Territory Marking

Deliberate scent marking (higher energy cost):
- Marks L1 cell boundaries
- Other creatures of same species recognize territory
- Territorial behavior: avoid or challenge based on DNA traits
- Marks need refreshing (decay) or territory "fades"

## Golden Zone Opportunities

| Optimization | Free Biological Behavior |
|--------------|--------------------------|
| Only process scent if has receptor trait | Species without noses ignore scent |
| Passive trail from movement | No explicit "leave trail" action needed |
| Decay creates freshness gradient | Track direction inference |
| L1 territory marks | O(cells) territory detection |
| Stronger scent = more detectable | Size/health affects marking strength |

**Trail Tracking Golden Zone:**
- Predator enters L0 cell with prey scent
- Compares adjacent cell scent strengths
- Moves toward stronger scent (fresher trail)
- No pathfinding required - gradient following emerges
- Performance: local comparison only, O(1) per decision

## Trade-offs

**Scent reveals presence:** Marking territory also advertises your location to predators. Strong markers are easier to find.

**Energy cost:** Active marking is expensive. Weak creatures mark weakly or infrequently, creating smaller/weaker territories.

**Detection trade-off:** Creatures with strong scent-sensing may have reduced visual acuity (neural real-estate).

**Persistence double-edge:** Your scent trail helps pack-mates find you, but also helps predators track you.

## Expert Input

### Zoologist (zoologist-tom)

Real-world chemical signaling:
- **Ants:** Pheromone trails create shortest-path emergence
- **Wolves:** Territory marking creates buffer zones
- **Moths:** Pheromone detection over kilometers (extreme sensitivity)
- **Cats:** Scent marking includes individual identity

**Key insight:** Chemical signals encode more information than other types:
- Species identity (strong)
- Individual identity (possible)
- Reproductive state (pheromones)
- Emotional state (alarm pheromones)
- Health/fitness (parasite load affects scent)

### Distinction from Stigmergy

`stigmergy.md` describes **environmental modification** (trampled paths, built structures).
Chemical scent is **signal broadcast** (information layer, doesn't modify terrain).

They complement each other:
- Stigmergy: "Many creatures walked here" (physical evidence)
- Scent: "A specific creature was here recently" (chemical evidence)

## Dependencies

- `signal-broadcasting.md` architecture (required)
- DNA system for scent gland trait
- DNA system for scent receptor trait
- Energy system (to pay marking cost)

## Related Ideas

- `signal-broadcasting.md` - Foundation architecture
- `stigmergy.md` - Complementary (physical vs chemical trails)
- `mating-calls.md` - Alternative mating signal (vocal vs chemical)
- `memory.md` - Could store scent "signatures" of known individuals

## Open Questions

- Should individual creatures have unique scent signatures (recognition)?
- How does wind/water affect scent propagation? (environmental factors)
- Can creatures mask their scent? (stealth hunting adaptation)
- Should scent trails have "width" (larger creatures leave wider trails)?
- How does scent interact with burrowing? (underground scent channels)

---
*Captured: 2025-12-28*
