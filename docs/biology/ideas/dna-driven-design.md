# DNA-Driven Design Principle

**Status:** ❌ NOT IMPLEMENTED (design only - code contains TODO markers, no DNA struct exists)

---

## What Exists vs What's Planned

### ✅ What EXISTS (Design Phase)
- **This design document** - Complete architectural vision for DNA system
- **TODO markers in code** - Constants flagged for future DNA migration (e.g., `// TODO: from DNA`)
- **Zoologist consultation protocol** - Process for validating biological realism
- **Hooks and agents** - Infrastructure ready to support DNA validation

### ❌ What DOES NOT EXIST (Not Implemented)
- **DNA struct** - No DNA component in Rust code
- **Gene expression pipeline** - No DNA → phenotype → behavior system
- **Genetic operations** - No crossover, mutation, or inheritance
- **Per-creature variation** - All creatures use same hardcoded constants
- **Species system** - No DNA-based species identification

**This document describes the VISION, not current reality.** Code contains only placeholder TODO markers.

---

## Core Principle

**Every physical attribute and behavioral characteristic of a creature (crit) MUST be encoded in its DNA.**

DNA is a simple array structure that defines how a creature looks, moves, perceives, and behaves. This DNA is used for:
- **Genetic crossover** - Sexual reproduction combines parent DNA
- **Species identification** - Similar DNA = same species
- **Emergent behavior** - Rich variety of strategies and niches
- **Evolutionary dynamics** - Natural selection acts on DNA, not hardcoded values

## Why DNA-Driven Design Matters

### Emergent Gameplay
DNA-driven traits create unpredictable, dynamic ecosystems:
- Predators evolve better eyesight → prey evolve better camouflage
- Fast creatures dominate open terrain → slow creatures thrive in dense cover
- Aggressive species create niches for avoidance specialists

### Genetic Diversity
No two creatures are identical unless clones:
- Siblings share similar but not identical traits
- Populations adapt to local conditions over generations
- Rare mutations create novel strategies

### Player Engagement
DNA makes creatures feel alive:
- Players discover species with unique characteristics
- Breeding programs yield predictable yet varied offspring
- Conservation matters when species represent unique genetic lineages

### Technical Benefits
- Single source of truth for creature characteristics
- Easy to serialize/deserialize for save games
- Genetic algorithms naturally emerge from DNA operations
- Species clustering happens automatically via DNA similarity

## Primitive Traits vs Emergent Behavior

**CRITICAL DESIGN PHILOSOPHY:**

DNA encodes **primitive traits** - simple, fundamental characteristics. Complex behaviors **emerge** from combinations of these primitives interacting with the environment and other creatures.

**DON'T encode:**
- "Sociality" - This should emerge from personal_space + flocking behavior
- "Intelligence" - This should emerge from perception range + reaction time
- "Dominance" - This should emerge from aggression + size + energy level

**DO encode:**
- Simple physical parameters (size, speed, perception distance)
- Basic behavioral thresholds (personal space, hunger threshold)
- Binary flags (flocking yes/no, diurnal/nocturnal)

### Examples of Emergence:

**Social Behavior emerges from:**
- Small personal_space (tolerates close proximity)
- Flocking flag enabled (seeks nearby creatures)
- Low aggression (doesn't attack neighbors)
- = **Result:** Creatures form herds without explicit "sociality" gene

**Territorial Behavior emerges from:**
- Large personal_space (wants distance from others)
- High aggression (attacks intruders)
- = **Result:** Creatures defend territory without explicit "territorial" gene

**Stealth Predator emerges from:**
- Low energy consumption (can wait)
- Short perception range (must get close)
- High aggression (attacks when in range)
- = **Result:** Ambush hunting strategy without explicit "stealth" gene

## Primitive Trait Categories

### Physical Primitives
Simple body parameters that define capabilities:

| Trait | Gameplay Impact | Example Range |
|-------|-----------------|---------------|
| **Body Size** | Movement speed, energy needs, predator/prey dynamics | 0.5m - 10.0m |
| **Perception Range** | Detection distance for food, threats, mates | 5m - 500m |
| **Base Speed** | Maximum velocity potential | 0.5 m/s - 15 m/s |
| **Energy Capacity** | Time between feeding, starvation resistance | 100 - 10,000 units |
| **Growth Rate** | Time to maturity, vulnerability window | 0.1 - 5.0 scale/day |
| **Pigmentation** | Camouflage effectiveness, mate selection, species recognition | RGB + pattern |

### Behavioral Primitives
Simple thresholds and flags that drive decisions:

| Trait | Gameplay Impact | Example Range |
|-------|-----------------|---------------|
| **Personal Space** | Distance maintained from obstacles/others | 0.5m - 5.0m |
| **Aggression Threshold** | How easily provoked to attack | 0.0 - 1.0 |
| **Flocking Flag** | Seeks vs avoids nearby creatures | Boolean |
| **Hunger Threshold** | Energy level that triggers feeding behavior | 0.0 - 1.0 |
| **Flee Threshold** | Threat level that triggers escape | 0.0 - 1.0 |
| **Activity Cycle** | Active during day vs night | Diurnal/Nocturnal/Crepuscular |

## Systemic Trade-offs: No God-Tier Creatures

**CRITICAL: Every advantage must have a cost.**

The simulation must prevent "god-tier" creatures that are perfect at everything. Trade-offs are **systemic** (built into physics/biology), not arbitrary balance numbers.

### Physical Trade-offs

**Large Size:**
- ✅ Advantages: Higher top speed, better vision (height), intimidates smaller creatures
- ❌ Costs: Massive energy consumption (scales with mass), slower acceleration, larger target
- **Result:** Large creatures dominate open terrain but starve if food is scarce

**High Speed:**
- ✅ Advantages: Escape predators, catch prey, cover large territories
- ❌ Costs: Energy burns rapidly during movement, requires frequent feeding
- **Result:** Fast creatures excel in rich environments but die in lean times

**Long Perception Range:**
- ✅ Advantages: Detect food/threats early, strategic advantage
- ❌ Costs: Cognitive load (energy for brain processing), overwhelmed in cluttered environments
- **Result:** Long-range vision dominates plains, useless in dense forests

**Fast Growth Rate:**
- ✅ Advantages: Reach maturity quickly, reproduce sooner
- ❌ Costs: Intense energy demands during growth, vulnerable if food runs out
- **Result:** Fast growers thrive in abundance, die during scarcity

### Behavioral Trade-offs

**High Aggression:**
- ✅ Advantages: Secure resources, defend territory, drive off competitors
- ❌ Costs: Fight injuries, energy wasted on conflicts, shorter lifespan
- **Result:** Aggressive creatures dominate resources but die young

**Flocking Behavior:**
- ✅ Advantages: Safety in numbers, shared vigilance, find food cooperatively
- ❌ Costs: Competition within flock, disease spreads quickly, resource depletion
- **Result:** Flocking works until food runs low, then flock starves together

**Small Personal Space:**
- ✅ Advantages: Navigate dense terrain, tolerate crowds, access hidden resources
- ❌ Costs: Frequent collisions, higher stress, vulnerable to ambush
- **Result:** Tight-space navigators excel in forests, struggle in open combat

### Systemic Constraints

These are **not balance knobs** - they're biological/physical laws:

**Energy Budget (Metabolism):**
- Base metabolism scales with mass (Kleiber's law: larger = more energy needed)
- Movement cost scales with speed squared (physics: kinetic energy)
- Perception costs cognitive energy (brain processing)
- Growth requires intense energy investment
- **Result:** Large, fast, perceptive creatures starve unless constantly feeding

**Speed Limits (Physics):**
- Top speed increases with size (but not linearly - allometric scaling)
- Acceleration decreases with size (mass vs muscle power)
- **Result:** Large creatures are fast but can't turn/start/stop quickly

**Perception Limits (Biology):**
- Vision range modified by body size, time of day, terrain
- High perception in cluttered environments = cognitive overload
- **Result:** Long-range vision is a liability in complex terrain

### No Perfect Build

**Large + Fast + Aggressive = Starves**
- Burns energy too quickly
- Must feed constantly or die
- Vulnerable during lean seasons

**Small + Slow + Passive = Easy Prey**
- Can't escape predators
- Loses resource competition
- Only survives in niches (hiding, night activity)

**Long Vision + Open Plains = Works... until forests**
- Perfect on plains, useless in dense cover
- Geographic specialization creates niches

**Short Personal Space + Flocking + Low Aggression = Disease Vulnerability**
- Crowded groups spread illness
- No defense against aggressive competitors
- Works until pathogens or predators arrive

### The Goal: Viable Niches, Not Balance

Trade-offs aren't about making everything equally good. They're about creating **viable ecological niches** where different strategies succeed in different contexts:

- **Fast herbivores** dominate rich grasslands
- **Slow scavengers** survive on leftovers without energy waste
- **Ambush predators** thrive in forests where vision doesn't matter
- **Social prey** survive through numbers despite individual weakness

**Every strategy has a place. No strategy dominates everywhere.**

## Gene Expression Pipeline

**DNA → Phenotype → Behavior**

Genes in the DNA array are not used directly. Instead they are expressed through a pipeline:

1. **DNA** - Raw genetic data (numeric values in array)
2. **Phenotype** - Observable traits (actual perception distance, movement speed, visual appearance)
3. **Behavior** - Decision-making driven by phenotype (when to flee, feed, avoid obstacles)

### Environmental Modifiers
Gene expression should account for environmental factors:
- **Time of day** - Diurnal creatures have reduced vision at night
- **Terrain** - Dense forests reduce effective perception range
- **Energy level** - Low energy reduces cognitive performance (vision, decision quality)
- **Size scaling** - Larger creatures see farther, move differently than small creatures

### Genetic Operations

**Crossover (Sexual Reproduction):**
- Offspring inherit genes randomly from each parent (50/50 chance per gene)
- Creates unique combinations while preserving parental traits
- Siblings are similar but not identical

**Mutation:**
- Small random variations (±10%) applied to genes with low probability
- Bounded by min/max to prevent breaking simulation
- Source of novelty and adaptation over generations

## Zoologist Consultation Protocol

**CRITICAL: Before defining trait boundaries, ALWAYS consult zoologist-tom.**

### When to Consult
- Adding a new trait to the DNA structure
- Defining min/max bounds for a trait
- Creating behavioral decision rules
- Implementing trait interactions (e.g., size affects speed)
- Balancing species niches

### What to Ask
- "What is a realistic range for [trait] in nature?"
- "How does [trait A] affect [trait B] in real animals?"
- "What constraints ensure diverse species can coexist?"
- "What metabolic/physical laws should govern [behavior]?"

### Expected Output
Zoologist-tom will provide:
- **Biological rationale** - Why certain ranges are realistic
- **Mathematical formulas** - Allometric scaling laws (e.g., speed ∝ size^0.25)
- **Min/max bounds** - Ensuring variety without breaking simulation
- **Trade-offs** - Larger size = more energy needs
- **Niche opportunities** - How traits create ecological roles

### Example Consultation
**Request:** "We're adding perception range to DNA. What's a realistic range for creature vision distance, and how should it scale with body size?"

**Zoologist Response (from BIOLOGY_NOTES.md):**
```
Vision Range Design
- Small creatures (0.5m): 5-20m vision (prey detection, personal space)
- Medium creatures (2-5m): 50-200m vision (territory awareness)
- Large creatures (8-10m): 200-500m vision (apex predators, wide territory)
- Scaling formula: vision_range = base_vision * size^0.5
- Trade-off: Larger vision costs more cognitive energy (metabolism increase)
- Niche: Low-vision species could evolve other senses (future: hearing, smell)
```

### Recording Consultations
ALL zoologist consultations MUST be logged in `/workspace/BIOLOGY_NOTES.md`:
```
2025-11-07 | Perception Range | vision_range = 5-500m, scales with size^0.5 | Implemented in DNA struct
```

## Conceptual Examples

### Example 1: Perception Range (Eyesight)
**DNA Gene:** Vision range (5m - 500m)

**How it works:**
- Gene defines base perception distance
- Modified by body size (larger creatures see farther)
- Reduced at night for diurnal species
- Affected by terrain density and weather

**Emergent Gameplay:**
- Short-range vision creatures stay near food sources, excel in dense terrain
- Long-range vision creatures dominate open plains, detect threats early
- Predators evolve long vision → prey evolve camouflage (arms race)
- Niche opportunity: Low-vision species succeed in caves/forests where vision matters less

**Trade-offs:**
- Better vision costs more cognitive energy (higher metabolism)
- Long-range specialists vulnerable in cluttered environments

### Example 2: Social Behavior (EMERGENT)
**Primitive Genes:** Personal space (0.5m - 5.0m) + Flocking flag (boolean) + Aggression (0.0 - 1.0)

**How it emerges:**
- Small personal_space = tolerates close proximity
- Flocking flag enabled = seeks out nearby creatures
- Low aggression = doesn't attack neighbors
- **Result:** Creatures naturally form herds/schools without explicit "social" gene

**Emergent Gameplay:**
- Social creatures move in coordinated groups, share vigilance
- Groups find food faster (collective searching)
- Safety in numbers against predators
- But: disease spreads quickly, competition for resources within group

**Opposite combination creates solitary creatures:**
- Large personal_space + flocking disabled + high aggression = territorial loners

**Trade-offs:**
- Social: Safety vs competition, coordination vs resource sharing
- Solitary: Freedom vs vulnerability, exclusive territory vs isolation

### Example 3: Aggression Threshold
**DNA Gene:** Aggression level (0.0 passive - 1.0 aggressive)

**How it works:**
- Determines willingness to fight vs. flee
- Combined with energy level and size advantage to make decisions
- High aggression = attack even when disadvantaged
- Low aggression = flee unless overwhelming advantage

**Emergent Gameplay:**
- Aggressive creatures fight frequently, secure prime territory and mates
- Passive creatures conserve energy, avoid injury, outlast aggressive neighbors
- Context-dependent strategies emerge (aggressive when strong, passive when weak)
- Aggressive species create niches for avoidance specialists
- Mixed populations create dynamic, unpredictable interactions

**Trade-offs:**
- High aggression = energy waste on fights, injury risk, shorter lifespan
- Low aggression = lose resources, poor territory, fewer mating opportunities

## Species Identification

Species are not manually defined - they emerge from DNA similarity:

**Similarity Calculation:**
- Compare DNA across multiple dimensions (physical traits, behavior traits, appearance)
- Calculate distance/difference for each gene
- Aggregate into overall similarity score (0.0 = completely different, 1.0 = identical)

**Thresholds:**
- High similarity (>0.8) = same species, can interbreed effectively
- Medium similarity (0.5-0.8) = related species, hybrids possible but less fit
- Low similarity (<0.5) = different species, cannot interbreed

**Gameplay Implications:**
- Species boundaries are fuzzy and dynamic
- Isolated populations diverge over time (speciation)
- Players discover species by observing DNA clustering
- Conservation becomes meaningful (unique genetic lineages can be lost)

## Migration Path for Existing Traits

### Current Hardcoded Traits (Technical Debt)
These traits exist but are NOT yet DNA-encoded:

| Trait | Current Location | Status | Migration Priority |
|-------|------------------|--------|-------------------|
| `max_speed` | CreatureState struct | Hardcoded default | HIGH (Phase 1) |
| `energy` | CreatureState struct | Hardcoded default | HIGH (Phase 1) |
| `age` | CreatureState struct | Runtime only | LOW (age not genetic) |
| `width/height` | Creature message | Hardcoded 1x1 | HIGH (Phase 1 - size genes) |

### Migration Steps (Future Work)

**Current Status:** DNA system not yet implemented. Codebase contains `// TODO: from DNA` markers where hardcoded constants should eventually be replaced.

**Proposed Implementation Plan:**
1. **Phase 1** - Create DNA component struct with basic size/speed genes
2. **Phase 2** - Implement gene expression pipeline (DNA → phenotype)
3. **Phase 3** - Add perception, behavior genes
4. **Phase 4** - Implement genetic crossover and mutation
5. **Phase 5** - Species identification and clustering

**No specific sprint assignments** - this is a major feature requiring dedicated sprint planning.

### Interim: Flagging Hardcoded Traits
The DNA consultation hook will WARN (not block) when it detects hardcoded creature traits that should be DNA-encoded. This helps track technical debt and prevents adding more hardcoded values.

## Hook Enforcement

### `.claude/hooks/dna-consultation-check.sh`
Soft-enforcement hook that triggers on creature code changes:

**Behavior:**
- Detects new struct fields, enum variants, hardcoded constants
- Outputs warning with link to this document
- Prompts to consult zoologist-tom
- Checks if BIOLOGY_NOTES.md updated
- Flags existing hardcoded traits

**Mode:** Warning + guidance (non-blocking)

**Philosophy:** Education over prevention - help developers internalize the principle

## Anti-Patterns to Avoid

### DON'T: Hardcode Traits
**Problem:** Using magic numbers or global constants for creature attributes
- Eliminates genetic variation (all creatures identical)
- Prevents evolution and adaptation
- Removes player agency in breeding
- Makes ecosystem predictable and boring

**Example:** Setting all creatures to perceive 50m, avoid obstacles at 2m

### DO: Derive from DNA
**Solution:** Read trait values from each creature's individual DNA
- Every creature has unique characteristics
- Populations evolve over time
- Players breed for desired traits
- Ecosystem remains dynamic and surprising

### DON'T: Add Global Config Constants
**Problem:** Game-wide settings that override genetic diversity
- Defeats the entire DNA system
- Makes all creatures behave identically
- Removes strategic depth

**Example:** Global "creature_speed" setting that all creatures use

### DO: Per-Creature DNA Variation
**Solution:** Each creature's behavior comes from its own DNA
- Speed varies by individual genetics
- Environmental factors modify expression (low energy = slower)
- Strategic diversity emerges naturally

### DON'T: Use Arbitrary Bounds
**Problem:** Pulling numbers out of thin air without biological rationale
- Breaks simulation (giant creatures, impossible speeds)
- Creates dominant strategies (one "best" build)
- Misses opportunities for interesting niches

**Example:** Setting creature size range to 1m - 1000m (unrealistic)

### DO: Consult Zoologist for Realistic Bounds
**Solution:** Use biologically-informed constraints from zoologist-tom
- Realistic ranges that feel lifelike (0.5m - 10m body size)
- Mathematical scaling laws (speed ∝ size^0.25)
- Trade-offs that create viable niches
- Bounds documented in BIOLOGY_NOTES.md with rationale

## Vision: Fully DNA-Driven Ecosystem

**End State (Future):**
- Zero hardcoded creature attributes
- All traits emerge from DNA
- Species self-organize into ecological niches
- Players breed creatures for desired traits
- Natural selection drives visible evolution
- Ecosystems collapse/thrive based on genetic diversity

**The DNA is the creature. Everything else is just expression.**

---

---

**Last Updated:** 2025-11-29

**See also:** `docs/biology/done/` for implemented features, `.claude/agents/zoologist-tom.md` for consultation protocol
