# Time Bubble

## Problem / Opportunity

Players want to experiment with evolution, protect endangered populations, and see rapid adaptation - but the main simulation runs at normal speed. Waiting hours to see meaningful evolutionary change breaks the flow of gameplay.

A localized time acceleration tool would enable:
- **Accelerated evolution experiments** - see mutations and adaptations in minutes
- **Safe breeding havens** - endangered populations recover quickly in protected zones
- **Speciation sandbox** - create new species through isolation and selection
- **"What if" scenarios** - test ecological changes in isolated regions

## Proposed Solution

**Time Bubble:** A player-placed beacon that creates a spherical region where simulation runs at 1000x speed.

### Core Mechanics

**Placement**
- Player places beacon device at desired location
- Sphere of influence appears (adjustable radius)
- "Fire and forget" - runs for fixed duration or until creature count threshold

**Boundary Behavior**
- Creatures can freely cross boundaries
- Entering: creature transfers from main sim to bubble sim (time-synced)
- Exiting: creature transfers back to main sim
- Brief "temporal adaptation" period (reduced speed/perception for 3 seconds after exit)

**Containment Strategy**
- Combine with upgraded barrier thumpers (repel creatures) to create fully isolated zones
- Without barriers: open ecosystem with migration across boundary
- With barriers: controlled breeding ground or evolution experiment

### Technical Architecture

**Dual Simulation Approach**
- Main simulation continues at normal speed
- Bubble spawns a second simulation instance on dedicated thread
- Creatures crossing boundary are serialized, despawned from source, spawned in destination
- Use existing creature ID system as stable identifier across simulations

**Visualization**
- Inside bubble: abstract rendering (shimmering particles, not individual creatures at 1000x speed)
- Overlay stats: population count, generation counter, mean trait values
- Boundary: visual shimmer effect indicating time distortion

### Player Experience

**What Players See**
- Rapidly pulsing population numbers inside bubble
- Generation counter ticking up (~17 generations per minute for short-lived creatures)
- Visible genetic drift - color/size distributions shifting over time
- Notification when significant changes occur ("New species emerging!")

**Strategic Decisions**
- Which creatures to include (small initial population = faster drift)
- How long to run bubble (more generations = more divergence)
- Whether to contain (barriers) or allow migration (gene flow)
- When to harvest evolved creatures back into main sim

## Speed Modes: Real Sim vs Population Genetics

Time Bubble supports **two modes** based on speed, with seamless transitions:

| Speed Range | Mode | Visual | Experience |
|-------------|------|--------|------------|
| 1x - 100x | Real Simulation | Creatures moving fast | Watch actual hunting, fleeing, breeding |
| 100x - 500x | Hybrid | Motion blur, trails | Real sim, simplified rendering |
| 500x+ | Population Genetics | Shimmering blur, stats overlay | Hyper-speed evolution, no individuals |

**Seamless Transition:**
- Speed up past threshold → creatures blur → swirling particles with stats
- Slow down below threshold → creatures "crystallize" out of blur → resume real sim

This means players can:
- Watch real creatures at moderate speedup (visually cool)
- Push to hyper-speed for rapid evolution (stats-based)
- Slow back down to see what evolved

See `docs/biology/ideas/population-genetics-algorithm.md` for the statistical model that enables 10,000x+ speeds.

## Golden Zone

**Seamless Mode Switching**

The transition between real simulation and population genetics IS the Golden Zone:
- Real sim below threshold = watch the action
- Pop gen above threshold = O(1) computation for unlimited speed
- Blur transition = visually indicates "time is moving too fast to perceive"

Biologically justified: You can't perceive individual events at 1000x speed anyway - the statistical view is what you'd actually see if you could observe evolution in fast-forward.

## Trade-offs

| Benefit | Cost |
|---------|------|
| Rapid evolution visible in minutes | Requires dual simulation architecture |
| Player experimentation and creativity | Complexity in boundary transfer logic |
| Teaches real evolutionary biology | Risk of "mutational meltdown" confusing players |
| Isolated safe havens for endangered species | Performance cost of second sim thread |
| Enables player-driven speciation | Must cap creature count in bubble (500-1000 max) |

### Biological Gotchas

**Inbreeding Depression**
Small bubble populations become inbred over 20-50 generations. Players might expect "breeding haven" to produce super-creatures but instead get sickly, low-fitness individuals. Solution: Require minimum viable population (100+ creatures) or warn player.

**Mutation Load**
At 1000x speed with small populations, deleterious mutations accumulate faster than selection can purge them. Bubble creatures may be genetically "damaged." This is biologically accurate but potentially confusing.

**Ecological Simplification**
Isolated ecosystems tend toward monocultures. The "best" strategy dominates; diversity crashes. Players may expect rich ecosystems but get homogeneity. Solution: Bubble must contain multiple niches/resources.

**Predator-Prey Desynchronization**
A single predator entering bubble could exterminate all prey in "seconds" (external time). Creates strategic gameplay (be careful what enters) but also risk of accidental catastrophe.

## Expert Input

**ECS Consultation (2025-12-28):**

Dual-simulation approach is technically sound. Bevy supports multiple independent World instances. Key insights:
- Use existing creature ID system as stable identifier across Worlds
- Transfer uses existing serialization infrastructure (save/load)
- Both Worlds must share identical initialization to match archetypes
- Dedicated thread for bubble sim to avoid blocking main sim
- Cap bubble creature count (500-1000) to stay within CPU budget (~50ms per main tick)

Recommended: Build Phase 1 prototype at 10x (not 1000x) to validate transfer mechanics before scaling up.

**Zoologist Consultation (2025-12-28):**

Biologically plausible and rich. At 1000x with short-lived creatures, players see ~170 generations per 10-minute session - enough for measurable evolutionary change (adaptation, genetic drift, speciation precursors).

Key biological phenomena that emerge:
- **Genetic drift** becomes visible (allele frequencies shifting in real-time)
- **Founder effects** if small starting population (rare traits can fix rapidly)
- **Ecological succession** (boom-bust cycles, predator-prey oscillations compressed)
- **Observable speciation** after 3-4 sessions of continuous isolation

Golden Zone recommendation: Statistical population model inside bubble (O(1) cost, biologically accurate, visually interesting).

Critical: Implement temporal adaptation period (brief debuff) when creatures exit bubble - justified as metabolic resynchronization.

## Dependencies

- Dual Bevy World architecture (second simulation instance)
- Creature serialization system (already exists for save/load)
- Boundary detection and transfer logic
- Barrier thumper upgrades (to create contained zones)
- Population genetics equations (for statistical model in Phase 2)
- Overlay UI for bubble statistics

## Related Ideas

- `docs/biology/ideas/population-genetics-algorithm.md` - **Core engine for hyper-speed mode (500x+)**
- `docs/gameplay/ideas/thumper.md` - Barrier thumpers create containment zones
- `docs/gameplay/ideas/fast-forward-game-start.md` - Similar time acceleration concept for game start
- `docs/biology/ideas/dna-driven-design.md` - DNA system drives observable evolution
- `docs/gameplay/ideas/dna-collector.md` - Harvesting evolved DNA from bubble creatures

## Open Questions

- Should multiple bubbles be allowed simultaneously? (TBD - probably limited number)
- What's the minimum/maximum bubble radius?
- How long should temporal adaptation debuff last after exiting?
- Should resources inside bubble regenerate at 1000x or normal speed?
- Can player adjust bubble speed in real-time or fixed at placement?
- What happens if player removes beacon while creatures are inside?
- Should there be a gradient at boundary (10x → 100x → 1000x) to smooth transition?

---
*Captured: 2025-12-28*
