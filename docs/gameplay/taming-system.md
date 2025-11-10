# Taming System: DNA-Driven Creature Domestication

**Last Updated:** 2025-11-10
**Status:** Phase 1.5 Feature (Post-Early Access)
**Design Principle:** Emergent from DNA traits, not scripted befriending

---

## Overview

The taming system allows players to domesticate wild creatures for protection, companionship, and strategic purposes. Unlike traditional "pet" systems, taming in Speciate is **DNA-driven**—creatures with high `social_learning` and low `aggression` traits tame faster, while territorial apex predators resist domestication.

**Key Philosophy:** Taming is NOT a friendship meter. It's a **behavioral shift driven by DNA-encoded social instincts.**

---

## Taming Tiers

###Tier 1: Beacon Zones (Early Game)

**Unlock:** Default (available from game start)
**Tech Level:** Low (basic crafting)
**Best For:** Docile herbivores, small creatures, Drongo colonization

#### Mechanics

**Setup:**
1. Player crafts **Taming Beacon** (biomass + basic materials)
2. Places beacon in desired location
3. Beacon emits pheromone signal (invisible to player, visible to creatures)

**Effect:**
- Creatures within **beacon radius** (25-50m) gradually reduce `aggression` over time
- DNA-driven: High `social_learning` species respond faster
- Low `aggression` species (herbivores, Drongos) tame in minutes
- High `aggression` species (predators) resist or ignore beacon

**Duration:**
- Requires player presence (beacon inactive if player >100m away)
- Taming progress resets if creature leaves radius before completion
- Once tamed, creature "bonds" with beacon zone (returns if it wanders)

#### DNA Trait Modifiers

| Trait | Effect on Taming Speed |
|-------|------------------------|
| `social_learning: 0.8-1.0` | 3x faster (Drongos, social herbivores) |
| `aggression: 0.0-0.2` | 2x faster (docile species) |
| `flocking: true` | Group taming (whole herd bonds together) |
| `aggression: 0.7-1.0` | Cannot be beacon-tamed (too territorial) |

#### Example: Drongo Taming

```javascript
// Drongo DNA
drongo = {
  social_learning: 0.9,   // Very high
  aggression: 0.1,        // Very low
  flocking: true          // Group species
}

// Taming calculation
base_taming_time = 300 sec (5 min)
multiplier = social_learning * (1 - aggression) * (flocking ? 1.5 : 1.0)
multiplier = 0.9 * 0.9 * 1.5 = 1.215

actual_time = 300 / 1.215 = 247 sec (~4 min)
```

**Result:** Drongos tame quickly and bring their whole group.

---

### Tier 2: Harpoon Capture (Mid Game)

**Unlock:** Crafted tech (requires rare materials from exploration)
**Tech Level:** Medium (advanced crafting, hunting skill)
**Best For:** Selective breeding, aggressive species, individual capture

#### Mechanics

**Setup:**
1. Player crafts **Tranquilizer Harpoon** (biomass + sedative plants + metal)
2. Aims and fires at target creature (skill-based, can miss)
3. Hit creature enters "sedated" state (temporary immobilization)

**Effect:**
- Sedated creature can be **captured** (moved to player base or containment)
- Allows selective taming (choose specific DNA traits)
- Works on aggressive species (overrides their resistance)
- **Risk:** Aggressive creatures may attack before sedation takes effect

**Danger Mechanics:**
- Large/fast creatures require multiple harpoon hits
- Pack species call for backup (player may be swarmed)
- Failed capture = creature flees or becomes hostile (remembers player scent)

#### DNA Trait Interactions

| Trait | Harpoon Effect |
|-------|----------------|
| `size: 5.0+` | Requires 2-3 hits (large body mass resists sedative) |
| `speed: 8.0+` | Hard to hit (aiming challenge) |
| `aggression: 0.8+` | Attacks player during sedation window |
| `pack_behavior: true` | Summons nearby pack members (dangerous) |
| `metabolism: 2.0+` | Sedation wears off faster (narrow capture window) |

#### Strategic Use Cases

**Breeding Programs:**
- Capture male + female with desired traits (size, speed, intelligence)
- Breed in controlled environment (sanctuary zone)
- Offspring inherit combined DNA (genetic crossover)

**Apex Predator Taming:**
- Harpoon lone predator (avoid packs)
- Sedated predator can be relocated to base
- Requires sustained feeding (high metabolism) to maintain loyalty

**Conservation:**
- Capture endangered species for breeding sanctuary
- Protect from ecosystem collapse (overhunting, habitat loss)

---

### Tier 3: DNA Cloning (Late Game)

**Unlock:** High-tech facility (found in wreckage or crafted late-game)
**Tech Level:** High (rare materials, power cells, genetic knowledge)
**Best For:** Perfect specimens, extinct species revival, army building

#### Mechanics

**Setup:**
1. Player discovers/builds **Genetic Lab** (end-game structure)
2. Analyzes creature DNA (requires captured specimen or tissue sample)
3. Clones creature with **exact DNA replication**

**Effect:**
- Bypasses natural breeding (no RNG, no waiting for offspring)
- Can clone deceased creatures (if tissue sample preserved)
- Can modify DNA sliders before cloning (experimental, risky)

**Resource Cost:**
- High biomass consumption (10-50x normal creature energy cost)
- Power cells (limited resource from wreckage sites)
- Genetic stabilizers (rare plants or chemical compounds)

#### Advanced Features

**DNA Modification (Experimental):**
- Adjust trait sliders ±10% before cloning
- Risk: Unstable DNA = clone may die or mutate unpredictably
- Example: Increase `speed` by +10%, decrease `aggression` by -10%

**Batch Cloning:**
- Clone multiple identical creatures (army building)
- Ethical cost? (game doesn't judge, but players may feel uneasy)
- High biomass drain = unsustainable without large ecosystem

**Extinct Species Revival:**
- Find fossilized DNA in ruins or ancient sites
- Clone species that went extinct (ecosystem collapse, player over-hunting)
- Enables "conservation win" (restore balance)

#### DNA Trait Cloning Fidelity

| Trait | Cloning Accuracy |
|-------|------------------|
| All physical traits | 100% (size, speed, strength) |
| Behavioral traits | 100% (aggression, social_learning) |
| Learned behaviors | 0% (clones are "blank slate" mentally) |
| Mutations | 5% chance (random trait ±5% deviation) |

**Implication:** Cloned Drongos don't know learned behaviors (player must teach again).

---

## Creature Commands (Late Game)

### Tech Thumper (Endgame Unlock)

**Unlock:** Late-game tech (requires rare materials, possibly Drongo crafting assistance)
**Purpose:** Mass creature coordination for combat or migration

#### Mechanics

**Activation:**
1. Player equips **Thumper Totem** (staff/spear-like tool)
2. Raises totem and slams into ground (cinematic animation)
3. Shockwave pulses outward (visual + audio effect)

**Effect:**
- All tamed creatures within **perception_range** (100-200m) receive "call to action"
- DNA-driven response:
  - High `social_learning` = Immediate response (Drongos, herd animals)
  - Low `social_learning` = Delayed or no response (solitary predators)
  - High `aggression` = Interpret as attack command (charge toward target)
  - Low `aggression` = Interpret as follow command (gather around player)

**Use Cases:**
- **Assault:** Call tamed army to attack Karg territory (gauntlet strategy)
- **Defense:** Summon protectors when player base under attack
- **Migration:** Redirect herd to new grazing area
- **Dramatic Moments:** Riding armored steed, army streaming behind

#### DNA-Driven Command Interpretation

```javascript
// Example: Thumper response logic
if (creature.tamed && distance_to_player < perception_range) {
    if (creature.dna.social_learning > 0.6) {
        respond_immediately();
    }

    if (creature.dna.aggression > 0.7) {
        action = "attack" (charge toward player's look direction)
    } else if (creature.dna.flocking) {
        action = "gather" (form protective circle around player)
    } else {
        action = "follow" (move toward player)
    }
}
```

**Result:** Same command, different creature responses based on DNA.

---

### Herding (Mid Game)

**Unlock:** Crafted pheromone lure (mid-game tech)
**Purpose:** Redirect migrations, guide creatures to specific locations

#### Mechanics

**Setup:**
1. Player crafts **Pheromone Lure** (biomass + scent glands from creatures)
2. Throws lure in desired direction
3. Lure emits chemical trail for 60-120 seconds

**Effect:**
- Creatures with `flocking: true` follow lure trail
- Non-flocking creatures ignore (solitary predators don't herd)
- DNA-driven: High `social_learning` species follow more reliably

**Use Cases:**
- **Obstacle Bypass:** Herd blocking path, redirect migration around base
- **Distraction:** Lure creatures into Karg territory (gauntlet strategy)
- **Resource Management:** Guide grazers to overgrown plant areas (ecosystem balance)

---

### Sentinel Mode (Drongo-Specific)

**Unlock:** Automatically available once Drongos are tamed
**Purpose:** Early warning system for predator detection

#### Mechanics

**Passive Ability:**
- Tamed Drongos with high `perception_range` (150-200m) automatically act as sentinels
- If predator enters perception zone, Drongo emits alarm call
- Player receives UI notification: "Drongos detected threat: NW, 120m"

**Strategic Use:**
- Build base with Drongo colony on perimeter (living alarm system)
- Safe exploration (bring Drongos, they spot predators before player sees them)
- Nighttime safety (Drongos don't sleep, continuous watch)

**DNA Trade-off:**
- High perception = High caloric cost (Drongos eat more)
- Player must sustain Drongo population with food

---

## Taming Progression Design

### Early Game (Beacon Zones)

**Available creatures:**
- Docile herbivores (grazers, browsers)
- Small omnivores (if low aggression)
- Drongo species (intelligent helpers)

**Purpose:**
- Learn taming mechanics (low risk)
- Build trust with ecosystem (non-aggressive species)
- Establish base defense (small creature perimeter)

**Limitations:**
- Cannot tame predators (too aggressive for beacons)
- Requires player presence (time investment)
- Limited control (creatures wander within beacon zone)

---

### Mid Game (Harpoon Capture)

**Available creatures:**
- Medium predators (solo hunters)
- Fast grazers (breeding stock for speed trait)
- Rare species (genetic diversity)

**Purpose:**
- Selective breeding (choose specific DNA traits)
- Tactical taming (capture specific individuals for roles)
- Risk vs. reward (aggressive creatures fight back)

**Limitations:**
- High skill requirement (aiming, timing)
- Dangerous (player can be injured or killed)
- Resource cost (harpoons are consumable)

---

### Late Game (DNA Cloning)

**Available creatures:**
- Apex predators (Karg territory survivors)
- Extinct species (fossil DNA revival)
- Custom-modified specimens (experimental DNA tweaks)

**Purpose:**
- Army building (clone guardian horde for final assault)
- Conservation (restore extinct species)
- Min-max breeding (perfect trait combinations)

**Limitations:**
- Massive resource cost (unsustainable without large ecosystem)
- Ethical questions (is mass cloning "natural"?)
- Clones lack learned behaviors (blank slate)

---

## Integration with Game Goal

### Daughter Rescue Campaign

**Act 1: Survival**
- Beacon taming: Establish base, protect from small predators
- Drongo colony: Gain intelligent helpers, learn taming basics

**Act 2: Exploration**
- Harpoon capture: Build diverse creature team for fog of war navigation
- Sentinel Drongos: Early warning for predator territories
- Selective breeding: Prepare for endgame (speed, strength, loyalty)

**Act 3: Gauntlet**
- DNA cloning: Mass-produce guardian army (if resources permit)
- Tech Thumper: Command creature horde to assault Karg territory
- Herding: Lure wild migrations to distract Kargs during rescue

**Victory moment:** Player rides into Karg nest on armored steed, surrounded by tamed creatures and Drongo scouts, uses Thumper to coordinate assault, rescues daughter amid chaos.

---

## DNA-Driven Design Validation

### How This Preserves Emergence

**NOT hardcoded "pet" system:**
- No universal "taming meter" (response varies by DNA)
- Aggressive species resist taming (true to their genetics)
- Clones aren't automatically loyal (must re-earn trust if low social_learning)

**Emergent interactions:**
- High social_learning species observe player, mimic behaviors
- Pack species defend each other (tame one, others may follow or attack)
- Metabolism affects loyalty (hungry creatures abandon player if unfed)

**Trade-offs:**
- Taming apex predators = High maintenance (feeding, control risk)
- Mass cloning = Ecosystem collapse (too many predators = prey extinction)
- Beacon zones = Time investment (player must be present)

---

## Balancing & Playtesting

### Prevent Exploits

**Problem:** Player tames 100 apex predators, becomes invincible

**Solution:**
- Cloning cost scales exponentially (1st clone cheap, 100th impossible)
- Tamed creatures still eat (ecosystem cannot sustain infinite predators)
- Large creature herds attract mega-predators (emergent threat)

**Problem:** Beacon taming is too slow, players get bored

**Solution:**
- Social_learning species tame in 2-5 minutes (tolerable)
- Aggressive species cannot beacon-tame (forces harpoon use)
- Multiple beacons stack (2 beacons = 2x speed)

---

## Success Metrics

### Phase 1.5 Launch

- **50%+ players use taming** (core mechanic, not optional)
- **Avg. 5-10 tamed creatures per player** (meaningful engagement)
- **25%+ use DNA cloning** (late-game depth)
- **Positive feedback on "creature army" moments** (emotional payoff)

### Community Engagement

- **"Breeding programs" emerge** (players share optimal DNA combos)
- **Taming guides appear** (community-driven content)
- **"Conservation runs" become popular** (save endangered species)

---

## Next Steps

1. **Prototype beacon taming** (simplest tier, validate DNA-driven speed)
2. **Test harpoon aiming** (skill-based mechanic, fun or frustrating?)
3. **Design cloning UI** (how to present DNA modification controls)
4. **Playtest with 5-10 tamed creatures** (performance, pathfinding, command clarity)
5. **Balance ecosystem impact** (do mass-tamed predators collapse prey populations?)

**Target:** Phase 1.5 (post-Early Access launch, 3-6 months)

---

## References

- **Rain World:** Taming via food offerings (behavioral, not scripted)
- **ARK Survival Evolved:** Tiered taming (passive, knockout, breeding)
- **Minecraft:** Breeding mechanics (selective trait inheritance)
- **No Man's Sky:** Creature companions (feeding, loyalty system)

---

**Status:** Design complete, pending Phase 1.5 implementation
**Owner:** backend-simulation-sam (taming logic), frontend-fanny (UI/commands)
**Consultant:** zoologist-tom (DNA trait validation)
