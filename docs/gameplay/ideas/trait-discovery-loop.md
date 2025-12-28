# Trait Discovery Gameplay Loop

## Problem / Opportunity

Players need a compelling reason to explore the simulation world and engage with creatures beyond observation. Special traits (armor, venom, echolocation) exist in the simulation, but without a discovery/collection mechanic, they're just visual variety.

## Proposed Solution

A **three-phase progression** where the player fantasy evolves:

### Phase 1: Collector (Early Game)

**Fantasy:** "Gotta catch 'em all"
**Activity:** Discover and catalog creature variants in the wild
**Reward:** Unlock entries in trait encyclopedia, achievement milestones

- Player explores biomes to find creatures with visible special traits
- First sighting unlocks trait in "discovery journal"
- No capture required initially - just observation
- Drives exploration of different environments (forest = camouflage, caves = echolocation)

### Phase 2: Hunter (Mid Game)

**Fantasy:** "Track the rare prize"
**Activity:** Hunt specific rare traits using clues and tracking
**Reward:** Capture specimens for breeding program

- Game director spawns rare trait creatures occasionally
- Player receives hints: "Bioluminescent creature spotted in coral reef sector"
- Tracking mechanics: follow scent trails, check feeding grounds, set up observation blinds
- Capture methods unlock through tech tree:
  - Basic: Tranq dart (requires line of sight)
  - Advanced: DNA sampler (works on remains)
  - Expert: Pheromone lure (attracts specific traits)

### Phase 3: Breeder (Late Game)

**Fantasy:** "Create the ultimate creature"
**Activity:** Combine discovered traits through genetic breeding
**Reward:** Custom creatures with synergistic trait combinations

- Captured specimens provide genetic material
- Breeding interface shows trait inheritance probability
- Experimentation discovers trait combinations (some synergize, some conflict)
- Genetic modification tech allows direct trait insertion (expensive, late-game)

### Discovery Methods (Multiple Available)

| Method | Requires | Best For |
|--------|----------|----------|
| **Observation** | Proximity + time | Common traits, first discoveries |
| **Capture** | Containment tech | Breeding specimens |
| **DNA Sample** | Tranq dart or remains | Rare/dangerous creatures |
| **Research** | Lab facility + time | Understanding trait mechanics |
| **Trade** | Other players (multiplayer) | Traits from other biomes |

### Game Director Integration

The game-director controls trait spawn rates to ensure:

- **Occasional excitement:** Rare traits appear often enough to hunt
- **Not trivial:** Can't just wait and catch everything immediately
- **Regional variation:** Different biomes have different trait pools
- **Dynamic events:** "Migration brings venomous creatures to your region"

### Trait Rarity and Spawn Control

| Rarity | Base Rate | Game Director Boost |
|--------|-----------|---------------------|
| Common | 1-5% of population | None needed |
| Uncommon | 0.1-1% | Guaranteed 1 per region |
| Rare | 0.01-0.1% | Event spawns, tracked individuals |
| Legendary | <0.01% | World events, unique specimens |

### Unlock Progression

```
Observe trait → Journal entry unlocked
                     ↓
Capture specimen → Breeding material available
                     ↓
Research trait → Mechanics revealed (counters, synergies)
                     ↓
Master trait → Genetic modification available
```

## Golden Zone

### 1. Tracking Creates Engagement, Not Just Waiting

Player actively searches using environmental clues rather than random encounters.

**Performance:** Game director spawns interesting creatures near player activity zones
**Gameplay:** Feels earned, not lucky

### 2. Collection Drives Exploration

Different biomes have different trait pools - completionists must explore everywhere.

**Performance:** Distributes player attention across simulation space
**Gameplay:** Natural world-building through discovery

### 3. Breeding Extends Engagement

Late-game breeding experiments use collected traits in new ways.

**Performance:** Captured creatures become resources, not just trophies
**Gameplay:** Theorycrafting optimal combinations

## Trade-offs

### Costs
- **UI complexity:** Discovery journal, breeding interface, tracking systems
- **Balance challenge:** Rare enough to be exciting, common enough to be achievable
- **Content creation:** Each trait needs capture animations, journal entries, research text

### Benefits
- **Core engagement loop:** Always something to hunt for
- **Natural progression:** Observe → Capture → Breed matches skill growth
- **Replayability:** Different trait combinations each playthrough
- **Social hooks:** "I found a legendary echolocator!" sharing moments

## Dependencies

- **Special traits system** - Traits must exist to discover
- **Game director** - Controls spawn rates and events
- **Capture/containment system** - Player must be able to catch creatures
- **Breeding system** - Traits must be heritable and combinable
- **Tech tree** - Progression unlocks capture methods

## Related Ideas

- `special-traits.md` - The traits being discovered
- `special-senses.md` - Rare sensory unlocks
- `game-director.md` - Controls spawn rates
- `taming-system.md` - Related capture mechanics
- `dna-collector.md` - DNA sampling gameplay
- `big-game-trophy-hunter.md` - Related hunting fantasy
- `terrarium.md` - Creature containment for breeding

## Open Questions

- **Multiplayer trading:** Can players trade genetic material?
- **Extinction risk:** Can over-hunting eliminate traits from world?
- **Breeding interface:** Slider-based? Selection-based? Random with probabilities?
- **Research time:** Real-time or simulation-time?
- **Failure states:** What if player can't find a rare trait? Hints? Guaranteed spawns?

---

*Captured: 2025-12-28*
