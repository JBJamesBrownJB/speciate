# Neural Network Social Learning

**Status:** ⏳ FUTURE FEATURE (advanced AI)

**Related:** `dna-driven-design.md`, `influence-maps.md`, `drongo-species.md`

## Core Concept: Social Learning and Cultural Transmission

One of the most advanced and fascinating areas of A-Life is combining genetic traits with learned behaviors to create believable social learning.

### The Emergent Formula (1+1=3)

**Simple Rules:**
1. **Genetic Rule**: "Stay near your mother" (hard-coded DNA trait)
2. **Learning Rule**: "Observe nearby critters and update neural net to mimic their actions" (flexible adaptation)

**Emergent Result**: "I learn from my mother" (complex, believable social behavior)

This is powerful because the "Follow" trait selects the data set that the "Learn" trait uses for training.

### Nature vs Nurture

This system creates two parallel adaptation tracks:

- **Genetic Traits (DNA)**: Inherited, slow evolution through natural selection
- **Learned Traits (NN)**: Acquired, fast adaptation through observation and imitation

The combination creates realistic "nature vs nurture" dynamics.

## Practical Neural Network Implementation

### The Key Insight: NN as Tuner, Not Decision Maker

Running a neural network for every decision for thousands of critters is computationally prohibitive. Instead:

**The NN doesn't make decisions directly. It tunes the decision-making system.**

**Architecture:**
- **Utility AI / Attraction Rating**: The critter's "instinct" or "gut" (fast, handles 99% of decisions)
- **Neural Network**: The "learning" part of the brain (slower, runs in background)
- **NN's Job**: Adjust weights and parameters of the instinct system based on observations

### Example: Learning Food Preferences

**1. Initial Instinct (Genetic Starting Point)**

Young critter has no learned preference:
```
Score(Eat_Red_Berry) = Hunger * 0.5
Score(Eat_Blue_Berry) = Hunger * 0.5
```
It will eat whichever berry is closer (no preference).

**2. Observation (Social Learning)**

The critter follows its mother (genetic "stay close" trait). It observes:
- Mother eats Blue_Berry: 3 times
- Mother eats Red_Berry: 0 times

**3. Learning (NN Processing)**

Observations are fed into the critter's simple NN as training data:
```
Training Set:
  (State: Hungry, Action: Eat_Blue_Berry) × 3
  (State: Hungry, Action: Eat_Red_Berry) × 0
```

The NN's job is to output new weights for the Utility AI.

**4. New Instinct (Learned Behavior)**

After training, the critter's Utility AI parameters are permanently updated:
```
Score(Eat_Red_Berry) = Hunger * 0.2  ← Decreased
Score(Eat_Blue_Berry) = Hunger * 0.8  ← Increased
```

**Result**: The critter has learned its mother's food preference. It will now seek blue berries preferentially. It never consciously "thought" about it—its instincts were simply re-weighted by observation.

## Advanced A-Life Concepts

### Behavioral Cloning / Imitation Learning

This is the formal AI/ML term for observe-and-mimic learning.

**Process:**
1. **Student** critter observes **Teacher** critter
2. Records `(state, action)` pairs: e.g., `(hungry, eat_blue_berry)`
3. Updates its own policy (the NN) to produce that same action in that same state
4. Student's behavior gradually converges toward teacher's behavior

### Memetics: Cultural Transmission

If a critter can learn from any nearby adult (not just its mother), you get **culture**.

**The Meme Spread:**
1. One critter discovers a clever hunting technique (random mutation or accident)
2. Nearby critters observe this new behavior
3. Their NNs learn it through imitation
4. The technique spreads through the population like a virus—a **"meme"**
5. Population adapts to environmental changes **much faster** than genetic evolution

**Cultural Evolution:**
- Parallel to genetic evolution
- Operates on different timescales (days/weeks vs generations)
- Can be lost (population crash = knowledge lost)
- Can be transmitted across species (if observation range allows)

### Regional Cultures

Over time, spatially separated populations develop distinct learned behaviors:

- **Northern herds**: Learn to hunt in packs (effective in open tundra)
- **Southern herds**: Learn to hunt solo (effective in dense forests)
- **Mountain herds**: Learn cliff-jumping escape tactics

**Player Observation**: Distinct regional "cultures" emerge without explicit programming.

## Technical Architecture

### Integration with Utility AI

The NN adjusts the weights in the creature's existing decision-making system.

**Before Learning:**
```rust
struct UtilityWeights {
    food_red_berry: f32,    // 0.5 (default)
    food_blue_berry: f32,   // 0.5 (default)
    avoid_predator: f32,    // 0.8 (genetic)
    seek_water: f32,        // 0.6 (genetic)
}
```

**After Observing Mother:**
```rust
struct UtilityWeights {
    food_red_berry: f32,    // 0.2 (learned ↓)
    food_blue_berry: f32,   // 0.8 (learned ↑)
    avoid_predator: f32,    // 0.8 (genetic, unchanged)
    seek_water: f32,        // 0.6 (genetic, unchanged)
}
```

### NN as Parameter Tuner

**Input Layer**: Current state (hungry, near_water, predator_visible, etc.)

**Hidden Layers**: 1-2 small hidden layers (8-16 neurons each)

**Output Layer**: Adjustment deltas for Utility AI weights
```
Output: [Δfood_red, Δfood_blue, Δavoid_predator, ...]
```

**Weight Update Rule:**
```rust
utility_weights.food_red_berry += learning_rate * nn.output[0];
utility_weights.food_blue_berry += learning_rate * nn.output[1];
```

### Training Data: Observation Buffer

Each critter maintains a small observation buffer:

```rust
struct Observation {
    state: StateVector,    // What the teacher was experiencing
    action: Action,        // What the teacher did
    timestamp: Time,       // When this was observed
}

struct ObservationBuffer {
    observations: Vec<Observation>,  // Max 100 recent observations
    teacher_id: EntityId,            // Who are we learning from?
}
```

**Collection:**
- Every second, if a "teacher" is within observation radius
- Record teacher's `(state, action)` pair
- Store in circular buffer (FIFO, max 100 entries)

**Training:**
- Not every frame (too expensive)
- Batch training every 10-30 seconds
- Use entire observation buffer as training set
- Update NN weights via backpropagation or simple gradient descent

## Learning Mechanisms

### Observation Radius

Who can the critter learn from?

**DNA-Encoded Trait**: `observation_range`
- High observation range: Learns from many distant critters
- Low observation range: Only learns from very close critters (mother/siblings)

**Perception-Based:**
- Must be within visual range
- Can only observe critters it can see (obstructions matter)

### Teacher Selection

Different strategies create different behaviors:

**1. Mother-Only Learning (Familial)**
```rust
if teacher.is_mother_of(self) && distance < observation_range {
    record_observation(teacher);
}
```
- Strong family units
- Isolated populations develop unique cultures

**2. Adult Learning (Social)**
```rust
if teacher.age > ADULT_AGE && distance < observation_range {
    record_observation(teacher);
}
```
- Learn from any mature critter
- Rapid meme spread
- Regional cultures emerge

**3. Success-Based Learning (Meritocratic)**
```rust
if teacher.fitness > self.fitness && distance < observation_range {
    record_observation(teacher);
}
```
- Only learn from successful critters
- Accelerates adaptation
- "Experts" have disproportionate cultural influence

### Training Frequency

**Performance vs Realism Trade-off:**

**Option 1: Continuous (Realistic but Expensive)**
- Train NN every 10 seconds with latest observations
- Creatures adapt quickly
- High CPU cost for thousands of critters

**Option 2: Life-Stage Based (Efficient)**
- Intensive learning during "juvenile" period
- Minimal/no learning as adults
- Mirrors real animal development
- Much lower CPU cost

**Option 3: Triggered Learning (Event-Based)**
- Only train after significant events (predator encounter, food scarcity)
- Sparse but meaningful updates
- Good performance/realism balance

### Forgetting and Decay

Old learned behaviors should fade without reinforcement.

**Decay Rule:**
```rust
// Every day, learned weights decay toward genetic defaults
utility_weights.food_red_berry +=
    decay_rate * (genetic_weights.food_red_berry - utility_weights.food_red_berry);
```

**Result:**
- Unused learned behaviors gradually disappear
- Genetic instincts reassert themselves
- Forces continuous observation to maintain learned culture

## DNA + Neural Network Integration

### Learning Ability as a Gene

The **capacity to learn** is itself DNA-encoded and can evolve.

**Gene: `learning_rate`**
- High learning rate: Fast learner, quickly adapts to observed behaviors
- Low learning rate: Slow learner, relies heavily on genetic instincts

**Evolutionary Trade-offs:**
- **High learning_rate**:
  - ✅ Rapid adaptation to new environments
  - ❌ Higher energy cost (brain activity)
  - ❌ Can learn "bad" behaviors from poor teachers
- **Low learning_rate**:
  - ✅ Lower energy cost
  - ✅ Stable, reliable instincts
  - ❌ Slow adaptation to change

**Natural Selection:**
- Stable environments: Low learning rate genes dominate (instinct is sufficient)
- Changing environments: High learning rate genes dominate (adaptation critical)

### Other Learnable Genes

**Gene: `observation_skill`**
- High: Accurately perceives teacher behavior
- Low: Noisy observations (learns incorrectly)

**Gene: `memory_capacity`**
- High: Retains many learned behaviors (large observation buffer)
- Low: Forgets quickly (small buffer, high turnover)

**Gene: `imitation_bias`**
- High: Strongly mimics teachers (conformist)
- Low: Blends learned + genetic behaviors (independent)

### The Baldwin Effect

Learned behaviors can guide genetic evolution.

**Process:**
1. Environment changes (e.g., new predator appears)
2. Critters with high `learning_rate` learn to avoid predator (survive)
3. Critters with low `learning_rate` fail to adapt (die)
4. High `learning_rate` gene spreads in population
5. Over many generations, "avoid predator" behavior gets encoded in DNA
6. Learning becomes less necessary (genetic instinct now handles it)
7. `learning_rate` gene decreases (reduces energy cost)

**Result:** Learned behaviors bootstrap genetic evolution. Learning buys time for genetic adaptation to catch up.

## Emergent Gameplay Opportunities

### Cultural Diffusion

**Player Observation:**
- Watch a new hunting technique spread from one critter to an entire herd
- Trace the "patient zero" who discovered it
- Observe barriers to diffusion (rivers, mountains, territorial boundaries)

**Metrics:**
- Meme spread rate
- Cultural diversity index
- Innovation frequency

### Knowledge Extinction

**Scenario:**
- A population develops a specialized hunting technique
- Environmental catastrophe kills most of the population
- Surviving juveniles never learned the technique (no adults to teach)
- Knowledge is permanently lost
- Population must rediscover or evolve alternative strategies

**Player Impact:**
- Create "zoos" to preserve populations
- Introduce "teachers" from other regions
- Document and catalog cultural knowledge

### Player Teaching

**Mechanic Idea:**
Can the player act as a "teacher"?

**Implementation:**
- Player-controlled avatar moves through world
- Demonstrates specific behaviors (eating specific plants, avoiding areas)
- Nearby critters with high `learning_rate` observe and imitate
- Player can intentionally spread or suppress behaviors

**Gameplay:**
- Train critters to avoid dangerous areas
- Teach herbivores to eat invasive plants
- Create artificial "cultural hotspots"

### Regional Cultures

**Observable Differences:**

**Desert Population:**
- Learned: Dig for underground water sources
- Learned: Nocturnal activity (avoid heat)
- Learned: Follow rare rainstorms

**Forest Population:**
- Learned: Climb trees to escape predators
- Learned: Follow fruit ripening seasons
- Learned: Use specific trails (stigmergy + learning combo!)

**Migration Events:**
- When populations mix, cultures blend
- Hybrid behaviors emerge
- Cultural dominance battles (which meme wins?)

## Performance Considerations

### NN Size Limits

**Constraint:** Thousands of critters, each with an NN.

**Solution: Tiny Networks**
- Input: 4-8 state variables (hunger, thirst, fear, etc.)
- Hidden: 8-16 neurons (1 layer)
- Output: 4-8 utility weight deltas

**Total Parameters:** ~100-200 per NN
**Memory:** 400-800 bytes per critter
**Cost:** 10K critters = 4-8 MB (acceptable)

### Update Frequency

**Don't train every frame.**

**Strategy: Amortized Updates**
- Divide population into 10 groups
- Each group trains on a different frame (staggered)
- Each critter trains once per second (60 frames)
- CPU cost spread evenly over time

**Alternative: Async Training**
- Training happens on a background thread
- Doesn't block simulation
- Weights updated when training completes

### Sparse Learning

**Not all critters are learning simultaneously.**

**Filters:**
1. Only juveniles learn actively (adults already learned)
2. Only critters with nearby teachers
3. Only critters in "learning mode" (state-dependent)

**Result:** Maybe 10-20% of population actively learning at any time.

### Observation Data Structures

**Circular Buffer (per critter):**
```rust
struct CircularObservationBuffer {
    data: [Observation; 100],  // Fixed-size array
    head: usize,               // Write index
    count: usize,              // Number of valid entries
}
```

**Memory:** 100 observations × 20 bytes = 2 KB per critter
**Access:** O(1) write, O(n) read for training

## Evolution of Learning

### How Learning Ability Evolves

**Scenario 1: Stable Environment**
- Food sources predictable
- Predators consistent
- Genetic instincts sufficient
- High `learning_rate` wastes energy
- Natural selection favors low `learning_rate`
- **Result:** Population of instinct-driven critters

**Scenario 2: Changing Environment**
- Seasons fluctuate wildly
- New predators invade
- Food sources shift
- Genetic instincts obsolete quickly
- High `learning_rate` enables rapid adaptation
- Natural selection favors high `learning_rate`
- **Result:** Population of flexible, adaptive critters

### Trade-offs: Learning Cost vs Adaptation Speed

**Energy Budget:**
```rust
energy_cost_per_second =
    base_metabolism
    + (learning_rate * LEARNING_ENERGY_COST)
    + (memory_capacity * MEMORY_ENERGY_COST);
```

**Survival Equation:**
- High learners: Burn energy faster but adapt to change
- Low learners: Conserve energy but rigid behavior

**Equilibrium:**
- Evolutionary pressure balances cost vs benefit
- Optimal `learning_rate` emerges for each environment

### Environments Where Learning Thrives

**Favorable Conditions:**
- High environmental variability
- Long lifespan (time to learn pays off)
- Social structure (teachers available)
- Complex problems (many strategies to learn)

**Unfavorable Conditions:**
- Extreme stability (nothing to learn)
- Short lifespan (die before learning matters)
- Solitary species (no teachers)
- Simple problems (instinct handles it)

### The Baldwin Effect in Action

**Timeline:**

**Generation 0:**
- New predator appears
- Population has no genetic defense
- Random critter with high `learning_rate` learns to hide in bushes
- Survives, reproduces

**Generation 10:**
- High `learning_rate` gene common
- Most critters learn hiding behavior from parents
- Population stable

**Generation 100:**
- Critter born with genetic mutation: "hide in bushes" instinct
- Doesn't need to learn it (genetic)
- Saves energy (no learning cost)
- Reproduces more successfully

**Generation 500:**
- "Hide in bushes" instinct now universal (genetic)
- `learning_rate` gene decreases (no longer needed for this behavior)
- Population now optimized: Instinct + learning capacity for future changes

**Result:** What was learned becomes genetic. Learning paves the way for evolution.

## Implementation Roadmap

### Phase 1: Foundation (Simple NN Tuning)
- Implement tiny NN (8-16 neurons)
- NN outputs utility weight adjustments
- Manual testing with small populations
- Validate learning actually occurs

### Phase 2: Observation System
- Circular observation buffer
- Teacher selection logic (mother-only first)
- Record (state, action) pairs
- Verify data collection pipeline

### Phase 3: Training Pipeline
- Batch training every 10-30 seconds
- Simple gradient descent on observation data
- Weight decay over time
- Performance profiling

### Phase 4: DNA Integration
- Add `learning_rate` gene
- Add `observation_range` gene
- Evolve learning capacity through natural selection
- Verify evolutionary dynamics

### Phase 5: Cultural Mechanics
- Expand to "learn from any adult" mode
- Meme tracking and visualization
- Regional culture emergence
- Player observation tools

### Phase 6: Advanced Features
- Success-based teacher selection
- Multiple NN architectures (specialized learning)
- Cross-species learning
- Player teaching mechanics

## Why This Matters for A-Life

**Realism:**
- Animals don't just follow instinct—they learn
- Young animals learn from parents (observable)
- Populations adapt faster than genes alone

**Emergence:**
- Culture emerges without explicit programming
- Innovation spreads organically
- Regional diversity arises naturally

**Player Engagement:**
- Observe knowledge transmission across generations
- Watch cultures rise and fall
- Intervene to preserve or spread behaviors
- Feel the ecosystem is truly "alive"

**Systemic Depth:**
- Two-track evolution (genetic + cultural)
- Baldwin Effect creates feedback loops
- Complexity from simple rules (the A-Life dream)

This is the pinnacle of believable A-Life behavior. Combined with DNA-driven traits, stigmergy, and cellular automata, you create a living, breathing, **thinking** ecosystem.
