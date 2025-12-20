Specification: Hierarchical Perception System (Sensory Mipmapping)

Goal: Decouple perception cost from simulation scale, allowing large creatures to navigate massive populations efficiently while enabling emergent "Size Domination" behaviors.

1. Core Philosophy: "The World as a Heatmap"

To a Whale, a single Krill is invisible noise. To a Krill, a Whale is a massive moving wall.
We achieve this by maintaining two parallel representations of the world:

Fine Reality (L0): The actual entities. Used for eating, mating, and bumping into things.

Coarse Reality (L1): A "blurred" heatmap of biomass. Used for navigation and long-range sensing.

2. Dual Spatial Grid Architecture

L0: Fine Grid (The "Tactile" Layer)

Resolution: 20m cell size (matched to ~2x max creature size).

Content: Stores specific EntityIDs.

Purpose: Immediate interactions (Collision, Consumption, Mating).

Query Range: Very short (e.g., < 40m).

L1: Coarse Grid (The "Visual" Layer)

Resolution: 100m cell size (5x Fine resolution).

Content: Stores Aggregates (Virtual Data), not Entities.

Data Structure (BioSignature):

TotalMass: Sum of all mass in the cell.

HerbivoreMass: Scent of food for predators.

CarnivoreMass: Scent of danger for prey.

MaxSize: The size of the largest single entity in this cell (Threat assessment).

CenterOfMass: The weighted average position of all entities.

Purpose: Long-range navigation (Steering, Migration).

Query Range: Long (e.g., up to 2km).

3. The Perception Pipeline (Per Critter Tick)

To maximize performance, perception runs in a strict "Fail-Fast" pipeline.

Phase 1: The Coarse Scan (Navigation)

Input: Long-Range Field of View (FOV).

Logic:

Calculate the set of Coarse Grid keys overlapping the FOV.

Threshold Filter: Compare Cell.TotalMass against the critter's Perception Threshold (see below).

If Mass < Threshold: The cell is effectively empty. Ignore it.

If Mass >= Threshold: Generate a Steering Force towards Cell.CenterOfMass.

Output: Accumulate steering forces (Seek Biomass / Flee Danger).

Phase 2: The Precision Gate (Optimization)

Logic: Before checking for individual entities, ask: "Is there any biomass nearby?"

Check: Query the Coarse Grid for the cell the critter is currently inside.

Early Exit: If the local coarse cell is empty (or below threshold), STOP. Do not scan L0.

Result: Creatures in open space run almost zero logic.

Phase 3: The Fine Scan (Interaction)

Condition: Only runs if Phase 2 detected local biomass.

Input: Immediate neighbors (9 cells) in the Fine Grid.

Logic: Iterate actual EntityIDs.

Resolves Collisions (Physics).

Checks for Food (Eating).

Checks for Mates (Reproduction).

4. Emergent Behavior: Size Domination

We do not code "Big things ignore small things." We code Sensory Thresholds.

The Mechanism

Perception Threshold: Defined as a percentage of the observer's own mass (e.g., 5%).

Scenario: A Giant (Mass 1000) walks near a Mouse (Mass 1).

The Glitch in the Matrix:

The Mouse exists in the Fine Grid.

The Mouse adds 1.0 mass to the Coarse Grid cell.

The Giant scans the Coarse Grid. It sees Mass 1.0.

The Giant's Threshold is 50.0.

Result: To the Giant, that cell is empty. It generates zero avoidance force.

The Consequence: The Giant walks in a straight line.

If the Mouse doesn't move, it gets trampled (High-Impulse Collision).

If the Mouse sees the Giant, the Mouse triggers a Flee response (because the Giant is above the Mouse's threshold).

"Preying on the Invisible"

How does a Whale eat invisible Krill?

Logic: It tracks Aggregate Biomass, not entities.

Behavior: It steers towards cells with high HerbivoreMass.

Feeding: When it arrives (Phase 3), it simply "vacuums" the Fine Grid, consuming anything in range without ever targeting a specific ID.

5. Implementation Roadmap for Agent

Refactor Grid: Split SpatialGrid into HierarchicalGrid containing both fine (Entity map) and coarse (BioSignature map).

Implement Aggregation: Create the system that reduces L0 data into L1 BioSignatures at the start of the physics tick.

Refactor Vision Component: Add perception_threshold (derived from body mass).

Refactor Vision System: Implement the 3-Phase Pipeline (Coarse -> Gate -> Fine).

ALL WORK MUST INCLUDE:

- Biological rationale and check with tom
- Tests
- Portal overlays to visualise and appreciate the grids and their usage
- Cycle through grids by pressing g key 