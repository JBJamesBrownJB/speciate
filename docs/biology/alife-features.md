# A-Life Core Features

## The "Life": Evolution and Adaptation

This is the "A-Life" part. You don't just create critters; you create a system that creates critters.

### Genetic Algorithms (GAs)

This is the core concept.

**Genome**: A critter's "DNA." It's just a list of numbers that define it: [MaxSpeed, SensorRange, Metabolism, HabitatPreference_Forest, HabitatPreference_Plains, ...]

**Reproduction**: When two critters reproduce, they create a new genome.

- **Crossover**: The new critter gets some genes from Parent A and some from Parent B.
- **Mutation**: There's a tiny (e.g., 0.5%) chance that any one gene is randomly changed slightly. (e.g., MaxSpeed of 5.0 becomes 5.1).

**Fitness & Selection**: You don't program "fitness." The environment is the fitness test. Critters with "bad" genes (e.g., slow metabolism in a low-food area) will die before they can reproduce. Critters with "good" genes (e.g., a preference for a habitat that's full of food) will thrive and pass on those genes.

This is how your "habitat desire" appears! You don't program it; you let it evolve.

### Pheromone Trails (Stigmergy)

A classic A-Life mechanic.

Critters can leave "data" in the world. This is a simple, emergent form of communication.

- Instead of just wandering, a critter that finds food can lay down a "Food Scent" trail on its way back to its nest.
- Other critters of its species smell the trail and get a massive bonus to their "attraction rating" to follow it.

**Emergent Result**: You'll see "highways" of critters forming between nests and resource hotspots, all from one simple rule. This is called Stigmergy (indirect communication by modifying the environment).

## The "World": Dynamic Environments

Your critters need a world that pushes back and changes. This creates the "selection pressure" that makes evolution work.

### Cellular Automata (CA)

This is the classic A-Needs-Based Utility AItem for dynamic worlds.

Instead of a static "habitat" map, the world is a grid where every cell has rules.

**Example 1: Grass**
- A Dirt cell next to a Grass cell has a 1% chance to turn into Grass.
- A Grass cell that's "eaten" turns to Dirt.

**Example 2: Fire**
- A Tree cell next to a Fire cell has a 30% chance to become Fire.
- A Fire cell has a 100% chance to become Ash next tick.

**Example 3: Water**
- You can simulate water flow, erosion, and "wetness" this way.

**Why it's so good**: This system is incredibly cheap (computationally) and creates a massively complex, dynamic, and believable world for your critters to live (and die) in.
