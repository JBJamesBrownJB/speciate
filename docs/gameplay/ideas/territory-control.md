# Territory Control Gameplay

## Problem / Opportunity

Players need meaningful long-term goals beyond creature observation. Territory control provides both competitive PvP mechanics and sandbox achievement tracking.

## Proposed Solution

### Core Mechanic: Map Coverage

Track percentage of map occupied by "your" creatures (creatures friendly to you). Win condition or achievement based on reaching coverage thresholds.

**Measurement:**
- L1 cells containing player-affiliated creatures count as "controlled"
- Coverage = controlled cells / total cells
- Could weight by creature biomass or count

### Creature Affiliation

Creatures become "friendly" to a player through:
- Direct spawning (creatures you create)
- Taming/bonding mechanics
- Lineage (offspring of friendly creatures inherit affiliation)
- Control technology (late-game tech upgrades)

### Game Modes

**Competitive PvP:**
- Multiple players compete for map control
- Creatures from different players compete for territory
- Could have predator-prey dynamics between factions
- Win condition: reach X% coverage or have highest at time limit

**Sandbox Goal:**
- Personal achievement metric
- Milestones at 25%, 50%, 75%, 100% coverage
- No opponent, just tracking player's ecosystem dominance

### Control Technology Progression

Late-game tech tree allowing more sophisticated creature direction:
- **Beacon:** Creatures within range attracted to location
- **Pheromone dispenser:** Guide creature movement paths
- **Neural interface:** Direct control of individual creature
- **Hive signal:** Coordinate group behavior

## Golden Zone

N/A - pure gameplay feature (no performance/biology overlap)

## Trade-offs

- **Complexity vs accessibility:** Too many control tools may overwhelm new players
- **Emergent vs directed:** Heavy control mechanics could undermine "watch ecosystem evolve" appeal
- **Competitive balance:** Predator-focused strategies might dominate herbivore strategies (or vice versa)

## Expert Input

No biological consultation required - pure gameplay design.

## Dependencies

- Creature affiliation/ownership system
- L1 grid for territory calculation
- Player technology/upgrade system (for control tech)

## Related Ideas

- `docs/gameplay/ideas/taming-system.md` - How creatures become friendly
- `docs/gameplay/ideas/environmental-migration.md` - Indirect population control
- `docs/gameplay/ideas/repulsion-field.md` - Player defensive equipment

## Open Questions

- How does affiliation transfer through generations? (100% inherited? Decay over generations?)
- Can creatures change affiliation (defection, capture)?
- Should "controlled" require active presence or just historical claim?
- How to visualize territory on map (color overlay by faction)?

---
*Captured: 2025-12-29*
