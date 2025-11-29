# Cellular Automata - Dynamic Terrain

**Status:** ⏳ FUTURE FEATURE (world generation)

**Related:** `stigmergy.md` (path trampling uses CA for recovery)

---

## Core Concept

The world is a grid where every cell has rules that create dynamic, evolving terrain. Creates selection pressure for evolution without static maps.

---

## Why Cellular Automata?

**Computationally cheap:** Simple rules per cell create massive complexity
**Dynamic world:** Terrain changes over time, forcing adaptation
**Emergent patterns:** Complex ecosystems from simple rules
**Player impact:** Actions have lasting environmental consequences

---

## Example Rules

### Grass Growth
- **Rule:** Dirt cell next to Grass cell has 1% chance/tick → Grass
- **Rule:** Grass cell eaten by creature → Dirt
- **Result:** Grazing creates bare patches, grass regrows from edges

### Fire Spread
- **Rule:** Tree cell next to Fire cell has 30% chance → Fire
- **Rule:** Fire cell has 100% chance → Ash (next tick)
- **Result:** Wildfires spread realistically, create ash wastelands

### Water Flow
- **Rule:** Water cell flows downhill to adjacent cells
- **Rule:** Erosion reduces terrain height over time
- **Result:** Rivers carve valleys, lakes form in depressions

---

## Ecological Impact

### Food Availability
- Grass regrowth rate determines herbivore carrying capacity
- Overgrazing creates deserts → population crash
- Recovery cycles create boom-bust population dynamics

### Habitat Destruction
- Forest fires destroy tree cover
- Open terrain favors different species (fast vs camouflaged)
- Ecosystem composition shifts after disasters

### Climate/Weather Effects
- Rain accelerates grass regrowth (1.5× rate)
- Drought slows regrowth (0.5× rate)
- Seasonal changes drive migration patterns

---

## Technical Considerations

### Performance

**Challenge:** 2000km × 2000km world = 2 billion tiles

**Solution:** Chunk-based updates
- Only update chunks with recent activity
- Dirty flag system marks tiles needing update
- Background process for low-priority updates (regrowth)

### Data Storage

**Problem:** Storing state for 2B tiles = prohibitive memory

**Solution:** Sparse storage
- Only store modified tiles (default values implicit)
- HashMap or spatial hash for dirty tiles
- Estimated active tiles: <1% = ~120 MB

### Update Frequency

**Batched updates:**
- Creature interactions: Every tick (high priority)
- Regrowth/recovery: Once per second (low priority)
- Climate effects: Once per minute (very low priority)

---

## Integration with DNA-Driven Design

### Habitat Preferences
Creatures with DNA trait `prefers_forest: 0.8` seek Forest tiles.
CA rules change Forest → Grassland → Desert based on grazing/fire.
Selection pressure favors adaptable species.

### Resource Competition
Grass regrowth rate limits herbivore population.
Herbivore density affects predator survival.
CA-driven food availability creates realistic carrying capacity.

### Environmental Selection
Fire-prone regions select for:
- Fast migration (escape fires)
- Burrowing behavior (underground survival)
- Fire-resistant traits (future: heat tolerance gene)

---

## Future Extensions

### Seasonal Cycles
- Spring: High grass regrowth rate
- Summer: Moderate regrowth, fire risk
- Autumn: Slowed regrowth, resource abundance
- Winter: No regrowth, snow cover

### Biome Transitions
- Grassland → Forest (if ungrazed for long periods)
- Forest → Grassland (if heavily grazed or burned)
- Desert → Grassland (if rain increases)

### Player Interaction
- Plant trees to create forests
- Build irrigation to green deserts
- Control burns to manage fire risk
- Fence areas to prevent overgrazing

---

## Design Philosophy

**Environment is fitness test:**
Creatures don't just live in the world; they shape it and adapt to it.

**Emergent dynamics:**
Boom-bust cycles, migration patterns, and habitat shifts arise naturally from simple rules.

**Player as ecosystem engineer:**
Influence but don't control the world. Let CA + DNA evolution create the story.

---

## Implementation Priority

**Phase 1:** Basic grass regrowth (eaten → dirt → regrowth)
**Phase 2:** Fire spread and recovery
**Phase 3:** Water flow and erosion
**Phase 4:** Seasonal cycles and climate effects

**Current status:** Not implemented (hardcoded static terrain)
