# Machine Thumper Deterrent

## Problem / Opportunity

Large creatures will attack player machines (extractors, processors, storage facilities). Players need a way to protect infrastructure without constant manual defense, but protection must have costs and trade-offs to maintain gameplay tension.

## Proposed Solution

**Thumper devices** emit substrate vibrations that repel creatures within a radius. Larger/more aggressive creatures require higher-tier thumpers with greater energy consumption.

### Core Mechanics

**Tiered Technology:**
- Basic Thumper: Repels small creatures (< 2m) at 80-200 Hz
- Advanced Thumper: Repels medium creatures (2-8m) at 20-80 Hz
- Industrial Thumper: Repels large creatures (8m+) at 5-20 Hz infrasound

**Energy Cost Scaling:**
Lower frequencies require exponentially more energy (physics: energy ∝ wavelength for equivalent amplitude). Industrial thumpers protecting against apex predators drain massive power.

**Research Progression:**
- Unlock tiers through tech tree
- Each tier requires materials + knowledge from previous tier
- Forces strategic choices: protect high-value machines first, leave outlying structures vulnerable

### Gameplay Loop

1. Player places machine in dangerous territory
2. Large predators detect machine (prey-like movement triggers? resource extraction noise?)
3. Player must choose: continuous thumper energy cost vs. periodic repairs from attacks
4. Higher-tier creatures in area force thumper upgrades (escalating energy drain)

## Golden Zone

| Optimization | Free Biological Behavior |
|--------------|--------------------------|
| Size-based thumper immunity | Elephants ignore footsteps - creatures above effective size threshold ignore thumpers entirely (skip deterrence calculation) |
| Habituation decay | After N ticks of continuous exposure, creatures stop responding (skip processing) - forces player to vary tactics or relocate thumpers |
| Frequency-based culling | High-frequency thumpers don't affect large creatures (different sensory range) - tier requirement emerges naturally |

**Habituation creates strategic depth:** Thumpers lose effectiveness over time as local creatures adapt. Players must:
- Rotate thumper locations periodically
- Use randomized pulse patterns (requires upgrades)
- Combine with other deterrents (pheromones)

## Trade-offs

**Continuous energy drain:** Unlike one-time defenses (walls), thumpers consume power constantly. Player must balance protection vs. resource production.

**Attraction before repulsion:** Novel stimuli initially attract creatures (investigation phase). First activation may draw more threats before deterrence kicks in.

**May disrupt beneficial creatures:** If future gameplay includes creature-based resource production (pollination, symbiotic behaviors), thumpers might harm allies.

**Tier mismatch vulnerability:** Basic thumper against alpha predator = wasted energy. Player must scout area and match deterrent tier to threat level.

## Expert Input

### Zoologist (zoologist-tom)

**Biological foundation:**
- Real seismic deterrence: kangaroo rats drum feet to warn snakes, elephants use infrasound for long-range communication
- Size-frequency relationship: larger animals detect lower frequencies (longer wavelengths match body size)
- Habituation is real: repeated stimuli become background noise unless pattern varies

**Key insight:** Thumpers are essentially "artificial apex predator footsteps." Creatures evolved to flee from large-creature vibrations will respond to synthetic signals if frequency/amplitude match.

**Recommended DNA traits for creatures:**
- `seismic_sensitivity` (0.0-1.0) - response strength to vibrations
- `habituation_rate` (0.01-0.1) - how quickly repeated stimuli are ignored
- Size-based frequency range (derived from body length)

### Complementary to Pheromones

Thumpers = **active energy cost**, **technology-gated**, **broad area effect**
Pheromones = **consumable resource**, **hunting-gated**, **targeted deterrence**

Combined system creates layered defense:
- Perimeter thumpers (broad low-tier deterrence)
- Apex pheromones on high-value machines (targeted high-tier protection)
- Player must manage both energy and pheromone stocks

## Dependencies

- Energy system for machines (power generation, storage, consumption)
- Creature seismic sensitivity (see `seismic-impacts.md` - signal reception trait)
- Research/tech tree system (thumper tier unlocks)
- Machine attack behavior (creatures must have reason to attack infrastructure)

## Related Ideas

- `thumper.md` - Existing attractor thumper for trophy hunting (complementary use case)
- `seismic-impacts.md` - Foundation seismic signal architecture (required)
- `apex-pheromone-harvesting.md` - Alternative deterrent system (companion mechanic)
- `repulsion-field.md` - Alternative protection mechanism (compare trade-offs)

## Open Questions

- Should thumpers have directional effect (protect one side only, requiring strategic placement)?
- Do thumpers interfere with each other (overlapping fields create "dead zones")?
- Can creatures "learn" to associate thumper vibrations with food? (machines = food source nearby)
- Should environmental factors (ground type, weather) affect effectiveness?
- Do stationary creatures receive deterrence effect differently than moving ones?

---
*Captured: 2025-12-28*
