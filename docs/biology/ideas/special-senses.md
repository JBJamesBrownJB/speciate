# Special Senses - Unlockable Advanced Perception

## Problem / Opportunity

Base senses (vision, hearing, smell) create a continuous budget trade-off, but some creatures in nature have evolved **entirely new sensory modalities** that operate differently. These represent "prize" discoveries that unlock new playstyles and counter existing strategies.

## Proposed Solution

**Special senses** are binary unlockable abilities that **replace** allocation in an existing base sense:

- Echolocation replaces hearing allocation (bats sacrifice normal hearing for sonar)
- Infrared pits replace visual acuity (pit vipers trade sharp vision for thermal)
- Electroreception replaces smell allocation (sharks trade chemical detection for bioelectric)

This maintains the sensory budget constraint while adding new capabilities.

### Special Sense Catalog

| Sense | Replaces | What It Detects | Real Examples |
|-------|----------|-----------------|---------------|
| **Echolocation** | Hearing | Shape/distance in darkness, ignores camouflage | Bats, dolphins, shrews |
| **Infrared Pits** | Vision (partial) | Heat signatures through obstacles | Pit vipers, boas |
| **Electroreception** | Smell | Muscle bioelectricity (hidden creatures) | Sharks, platypus, rays |
| **Magnetoreception** | N/A (bonus) | Earth's magnetic field (navigation) | Birds, sea turtles, salmon |
| **Lateral Line** | Hearing (aquatic) | Pressure waves in water | Fish, aquatic amphibians |
| **Polarized Light** | Vision (partial) | Polarization patterns (navigation, communication) | Mantis shrimp, bees |

### Counter System

Each special sense has explicit counters - rock-paper-scissors dynamics:

| Special Sense | Countered By | Why |
|---------------|--------------|-----|
| **Echolocation** | Stationary + soft surfaces | No echo return from motionless, sound-absorbing prey |
| **Infrared** | Cold-blooded creatures | No heat signature to detect |
| **Electroreception** | Motionless creatures | No muscle activity = no bioelectric signal |
| **Magnetoreception** | N/A | Navigation aid, not detection |
| **Lateral Line** | Stationary creatures | No pressure wave generation |

### Environmental Effectiveness

Special senses excel in specific conditions:

| Sense | Excels In | Weak In |
|-------|-----------|---------|
| **Echolocation** | Darkness, caves, dense vegetation | Open areas (echo dissipation), high wind |
| **Infrared** | Night, through foliage | Hot environments (thermal noise) |
| **Electroreception** | Water, murky conditions | Dry land (requires conductivity) |
| **Magnetoreception** | Long-distance migration | Magnetic anomalies, interference |

### Night Vision Interaction

**Vision is reduced at night** (light level modifier on base vision effectiveness). This creates niche for:

- Echolocation users (unaffected by darkness)
- Infrared users (thermal contrast higher at night)
- Creatures who evolved high night-vision sub-modality

**Design note:** Need to implement/verify night vision mechanics. Check `dna-driven-fov.md` for related work.

## Golden Zone

### 1. Echolocation = Spherical Range Check

Skip vision cone + occlusion raycast. Simple distance check in all directions.

**Performance:** O(1) distance check vs O(n) raycast
**Biology:** Echolocation penetrates obstacles, uniform spherical range

### 2. Electroreception Ignores Camouflage

Camouflaged creatures still have bioelectric signatures - electroreception counters chromatophores.

**Performance:** Electroreceptive creatures skip camouflage checks
**Biology:** Sharks detect hidden prey by muscle electricity

### 3. Infrared Ignores Foliage Occlusion

Heat passes through vegetation - infrared creatures skip vegetation occlusion checks.

**Performance:** Skip raycast through foliage
**Biology:** Pit vipers strike accurately through grass/leaves

### 4. Motionless Counters Multiple Senses

Stationary creatures evade echolocation, electroreception, AND lateral line simultaneously.

**Performance:** One "stationary" flag skips multiple detection systems
**Biology:** Freeze response is universal defense against motion-based detection

## Trade-offs

### What You Gain
- **Alternative detection:** Works when base senses fail
- **Counter play:** Defeats camouflage, works in darkness
- **Niche specialization:** Dominate specific environments

### What You Lose
- **Base sense allocation:** Must sacrifice hearing/vision/smell budget
- **Vulnerability to counters:** Cold-blooded prey invisible to infrared
- **Environmental dependency:** Echolocation useless in high wind

## Expert Input

**Zoologist-tom consultation (2025-12-28):**

- **Echolocation** requires larynx modifications + auditory processing - genuinely complex, should be rare
- **Electroreception** uses ampullae of Lorenzini (sharks) - requires aquatic/semi-aquatic body plan
- **Infrared** uses pit organs with temperature-sensitive receptors - relatively simple, could be uncommon
- **Hard incompatibility:** Echolocation + stealth movement (broadcasts your position)
- Suggested **counter priority:** Motionless = universal counter to motion-based special senses

## Dependencies

- **Sensory simplex** - Budget that special senses replace into
- **Light level system** - Night reduces vision effectiveness
- **Special traits system** - Framework for binary unlocks
- **Motion detection** - Freeze response as counter mechanic

## Related Ideas

- `sensory-simplex.md` - Base sense budget that special senses modify
- `special-traits.md` - Parent system for binary unlocks
- `motion-detection.md` - Stationary creatures evade detection
- `dna-driven-fov.md` - Vision sub-modalities including night vision
- `chemical-scent.md` - Base smell sense (electroreception replaces)

## Open Questions

- **Magnetoreception:** Bonus sense or replaces something? (Currently marked as bonus)
- **Lateral line:** Aquatic-only or extend to terrestrial vibration sensing?
- **Night vision mechanic:** How much does darkness reduce vision? Need to verify/implement
- **Rarity tiers:** Echolocation legendary? Infrared rare? Electroreception aquatic-only?
- **Hybrid creatures:** Can a creature have multiple special senses? (Probably not - budget constraint)

---

*Captured: 2025-12-28*
*Zoologist consultation: ae94a6c (shared with special-traits)*
