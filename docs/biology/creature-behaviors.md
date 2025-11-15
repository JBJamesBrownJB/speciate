# Attraction Rating (Influence Maps)

**Status**: Design Concept
**Last Updated**: 2025-11-09
**Related**: [dna-driven-design.md](./dna-driven-design.md), [alife-features.md](./alife-features.md)

This is your core concept, and it's the right way to go for an A-Life sim. You can think of your game world as having multiple "layers" of data stacked on top of the visual one. An "Influence Map" is just a grid of values representing one of these layers.

When a critter needs to pick a wander target, it doesn't just pick a random (x, y) coordinate. It samples several potential (x, y) locations and "scores" them.

Your "pipeline" is the scoring function. A simple version could look like this:

```
FinalScore = (HabitatScore * w1) + (FoodScore * w2) - (DangerScore * w3)
```

...and this whole thing is multiplied by an ObstacleScore (which is 0.0 for obstacles/borders and 1.0 for all other areas).

## Implementation Ideas

### Habitat Layer

This is your genetic idea.

- **Critter Genetics**: A critter has genes for [Prefers_Forest: 0.8, Prefers_Plains: 0.2].
- **World Map**: The map has a Habitat_Layer where "Forest" tiles have a value of 1.0 and "Plains" tiles have a value of 0.0.
- **Scoring**: The critter's score for a "Forest" tile is high (1.0 * 0.8), and its score for a "Plains" tile is low (0.0 * 0.8 + 1.0 * 0.2).

### Obstacle Layer (Your "Soft Zone" refined)

This layer is key. Instead of an exponential reduction, just use multiplication.

Create a layer called Passable_Layer:
- Anywhere a critter can walk: 1.0
- The "soft zone" 50m from the border: 0.1 (or some low value)
- Inside the hard border/obstacle: 0.0

When you calculate the final score for any potential target, multiply it by the Passable_Layer value.

```
FinalScore = (HabitatScore + ...) * PassableScore
```

- Any target in the hard border gets a score of 0, so it's never chosen.
- Any target in the "soft zone" gets its score reduced by 90%, making it highly unlikely but not impossible (which is more natural!).

## The "Repulsion" Idea (Potential Fields)

Your first idea about "repulsion" is also extremely useful, but for a different problem.

- **Influence Maps**: Best for strategic, long-range decisions. (e.g., "Where should I wander to?")
- **Potential Fields**: Best for tactical, short-range movement. (e.g., "How do I avoid that rock in front of me while walking?")

This concept is often called Steering Behaviors or Potential Fields. Imagine every object emits a "force":

- The wander target has an attractive force (pulls the critter).
- Obstacles, borders, and other critters have a repulsive force (pushes the critter away).

The critter's movement in real-time is the sum of all these forces acting on it. This creates beautiful, natural, curved paths as critters dynamically flow around obstacles without ever "thinking" about pathfinding.

## Putting It All Together (The Best of Both)

You can combine both of your ideas for a complete, two-tiered AI system:

### The "Brain" (Strategy)

Uses the Influence Map / Attraction Rating.

- **Job**: To make high-level decisions.
- **Action**: When in the Wander state, the brain samples 10-20 random points in a large radius. It uses your "pipeline" to score each point based on Habitat, Food, Danger, and the Obstacle/Border Layer.
- **Result**: It picks the highest-scoring point and sets it as the Current_Wander_Target.

### The "Body" (Tactics)

Uses the Steering Behaviors / Repulsion.

- **Job**: To move the critter.
- **Action**: Every frame, the body calculates its movement vector. It gets a strong "Attract" force from the Current_Wander_Target (set by the brain) and "Repel" forces from any nearby obstacles, borders, or critters.
- **Result**: The critter moves smoothly toward its goal while naturally flowing around impediments.

### Questions
- Avoiding obstacles should be less about 'repelling' and more about 'avoiding' So I feel like the applied force should be perpendicular / right angle from the vector between location and the target and the direct oposite vector. This means that within a comfort zone of another critter, it 'tends' to navigate round it, rather than 'bump into' the comfort zone sphere. For obstacles, we should ignore comfort zone, the crit doesn't mind being close to an obstacle, just navigate round it for logistics. Obstacles should also have clamping logic to it does actually represent a real impeneterable barrier, I wonder what the perf cost is if we do this and if we roll this out to crits as well?

We will create 'trials' see docs/testing/trials which will tune this.

This combined system means your critters will make smart decisions (like heading to a preferred habitat) and execute those decisions in a believable, non-robotic way.
