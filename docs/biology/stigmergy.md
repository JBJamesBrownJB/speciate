# Stigmergy: Environmental Modification as Communication

**Status:** ⏳ FUTURE FEATURE (environmental modification)

**Related:** `influence-maps.md`, `cellular-automata-terrain.md`

## Core Concept

Stigmergy is indirect communication where agents modify the environment, and those modifications influence future agent behavior. This creates positive feedback loops that generate emergent patterns.

**Types of Stigmergy:**
- **Chemical Stigmergy**: Agents leave chemical markers (ant pheromone trails)
- **Physical Stigmergy**: Agents physically alter the environment (path trampling, nest building)

## Physical Stigmergy: Path Formation

Instead of programming critters to "follow paths," paths emerge naturally from repeated use through environmental modification.

### The Feedback Loop

1. A herd wanders from A to B
2. The ground gets slightly worn
3. The worn path is now slightly faster to walk on
4. The next critter's AI rates the worn path as "more desirable"
5. More critters use the path, wearing it down further
6. Eventually, a clearly defined Dirt Path emerges

**This is the engine of all emergent behavior.**

## Implementation: The Trample Mechanic

### 1. Critter Weight/Size

Give critters a physical impact stat:
- Big Crit: `Weight = 10`
- Medium Crit: `Weight = 5`
- Small Crit: `Weight = 1`

**DNA-Driven**: Weight should be encoded in DNA, varying by species and individual genetics.

### 2. Tile Ground Density

Your world grid tiles get a new property: `Ground_Density` (toughness/resistance to wear)

**Initial Values:**
- Rock: `Density = 1,000,000` (effectively indestructible)
- Grass: `Density = 100`
- Dry Earth: `Density = 50`

### 3. Trampling Rule

Every second a critter stands on a tile (or every step taken):

```
Tile.Density -= Critter.Weight * Time.DeltaTime
```

**Visual Transitions:**
- `Density < 80`: Grass → Worn_Grass (sprite change)
- `Density < 30`: Worn_Grass → Dirt_Path (sprite change)
- `Density < 10`: Dirt_Path → Compacted_Earth (hard-packed trail)

## The Consequence: Behavioral Feedback

The new Dirt Path tile must change simulation rules to create the feedback loop.

### Movement Speed Bonus

Dirt Path tiles have lower movement cost:
- Grass: `MoveSpeed = 1.0x` (baseline)
- Worn_Grass: `MoveSpeed = 1.1x`
- Dirt_Path: `MoveSpeed = 1.5x`
- Compacted_Earth: `MoveSpeed = 1.8x`

**Result**: Attraction Rating pipeline or pathfinding (A*) inherently favors these paths. Critters use them because they are objectively better routes, not because they're programmed to "look for" paths.

### Ecological Impact

Add a Cellular Automata rule:

**Rule**: Plants (like Grass) cannot grow on Dirt_Path tiles.

**Consequences:**
- Heavily-used migration routes create permanent landscape scars
- "Dead zones" with no food
- Reduced total food in the area
- Herds forced to find new grazing lands
- New paths get trampled in the process
- Old unused paths can heal and regrow (see below)

## Integration with Attraction Rating System

Ground Density is just another influence map layer.

When a critter in Wander state scores potential targets:

```
FinalScore = (HabitatScore * w1) + (PathScore * w2) + (FoodScore * w3)
```

**PathScore Calculation:**
- High-density Grass: `PathScore = 0` (no bonus)
- Medium-density Worn_Grass: `PathScore = 5`
- Low-density Dirt_Path: `PathScore = 10`
- Very-low Compacted_Earth: `PathScore = 15`

**Result**: Critters feel an innate "pull" toward paths. Far more natural than programming them to actively seek paths.

## Path Healing: Cellular Automata Recovery

Unused paths should heal over time using CA rules.

### Recovery Rules

**Rule 1: Density Recovery**
- Every tick, if no critter is on a tile: `Tile.Density += 0.1 * DeltaTime`
- Recovery is slow (days/weeks of game time)

**Rule 2: Grass Regrowth**
- A Dirt_Path tile next to a Grass tile has a 0.5% chance per tick to become Worn_Grass
- A Worn_Grass tile next to a Grass tile has a 2% chance per tick to become Grass
- Requires `Tile.Density > 60` (soil must recover first)

**Rule 3: Environmental Factors**
- Rain accelerates recovery (1.5x regrowth rate)
- Drought slows recovery (0.5x regrowth rate)
- Proximity to water sources increases regrowth probability

### Visual Result

**Active Path** (heavy use):
```
Grass → Worn_Grass → Dirt_Path → Compacted_Earth
```

**Abandoned Path** (no use for weeks):
```
Compacted_Earth → Dirt_Path → Worn_Grass → Grass
```

## Emergent Gameplay Opportunities

### Migration Corridors

Large herds create permanent migration routes between:
- Watering holes
- Seasonal grazing areas
- Breeding grounds

**Player Interaction:**
- Place resources near migration routes to attract critters
- Block paths to redirect herds
- Observe seasonal migration patterns

### Territory Marking

Territorial species create patrol paths around territory boundaries:
- Worn circular paths around den/nest
- Visible territorial boundaries
- Conflict zones where territories overlap

### Predator Highways

Predators learn to hunt along prey migration routes:
- Ambush points at choke points
- Dramatic predator-prey encounters
- Player can observe and predict predator behavior

### Ecosystem Collapse

Over-grazed areas show visible environmental damage:
- Network of dirt paths with no food
- Mass starvation events
- Population crash → path healing → ecosystem recovery cycle

## Technical Considerations

### Performance

**Grid Size**: 2000km × 2000km world
- Tile size: 1m × 1m = 2 billion tiles
- Solution: Chunk-based updates (only update tiles with recent critter activity)
- Dirty flag system: Mark tiles as "needs update" when trampled

**Update Frequency**:
- Trampling: Every critter step
- Recovery: Batched updates (once per second for dirty tiles only)
- CA regrowth: Low priority background process

### Data Storage

Per-tile data:
```rust
struct Tile {
    terrain_type: TerrainType,      // 1 byte
    ground_density: u8,              // 0-255 (sufficient resolution)
    last_trampled: Timestamp,        // 4 bytes
    flags: BitFlags,                 // growth allowed, dirty, etc.
}
```

**Memory**: 6 bytes/tile × 2B tiles = 12 GB (too large)

**Solution**: Sparse storage
- Only store modified tiles (default Grass = Density 100)
- Hashmap or spatial hash for dirty tiles
- Estimated active tiles: <1% = 120 MB

### Rendering

**Optimization**: Use sprite atlases with automatic transitions
- Blend between Grass/Worn_Grass/Dirt_Path sprites based on density
- Viewport culling (only render visible tiles)
- LOD system (distant tiles use lower-res textures)

## Future Extensions

### Multi-Agent Path Types

Different species create different path types:
- Large herbivores: Wide dirt paths
- Small rodents: Narrow worn trails (barely visible)
- Aquatic species: Water channels that erode banks

### Nest Building (Physical Stigmergy)

Beyond paths, critters could modify tiles to create structures:
- Beaver dams (change water flow)
- Termite mounds (vertical structures)
- Burrow networks (underground tunnels)

### Chemical Stigmergy

Add scent-based communication:
- Territory markers (urine scent)
- Food trail markers (like ants)
- Danger warnings (alarm pheromones)
- Mate attraction (breeding season scents)

## Why This Matters for A-Life

**Emergence Over Programming:**
- Paths aren't scripted; they emerge from simple rules
- Player sees "intelligent" behavior without complex AI
- Ecosystem feels alive and reactive

**Long-Term Dynamics:**
- Seasonal migrations create temporary paths
- Climate changes affect path recovery rates
- Player actions have lasting environmental impact

**Visual Storytelling:**
- Path networks tell the story of the ecosystem
- Abandoned paths hint at population crashes
- New paths reveal changing herd dynamics

**Player Engagement:**
- Predict animal behavior from path patterns
- Understand ecosystem health from landscape wear
- Create/destroy paths to influence creature movement

This is a top-tier A-Life concept that perfectly complements your DNA-driven design and influence map systems.
