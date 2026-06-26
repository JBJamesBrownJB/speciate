# Plant P0 Grid

**Status:** 🚧 In-progress — branch `feature/plant-p0-grid`

**Context:** Food is the first gameplay pillar. Plants are the base of the food chain. This document covers the architectural decision on how plants are represented in the simulation and the phased implementation plan.

---

## Decision: Tile-based (P0 grid), not ECS entities

Plants are a **property of space**, not independent agents. A `PlantGrid` resource holds a flat 2-D array of cells at 4m resolution — fine enough for meaningful CA spread dynamics and credible creature feeding, coarse enough to be computationally tractable.

**Why not ECS entities:**
- No perception, no steering, no per-frame behaviour — entity overhead with no benefit
- Millions of plants would dominate archetype memory budgets
- CA dynamics (spread, regrowth) are naturally expressed as grid sweeps, not entity iteration

**Why 4m cells:**
- At L0 = 20m, each L0 cell covers exactly 25 plant cells (5×5 block) — clean aggregation for FoodScore influence map
- A 0.5m creature occupies ~⅛ of a cell width — eating depletes a 4m² patch (tangible without being atomic)
- 10km × 10km world → 2500×2500 = 6.25M cells → ~31MB RAM, ~25MB VRAM as texture — fine
- CA spread is a 4m hop → grass wavefronts look organic at scale

---

## Data model

### P0 cell (per-cell, flat Vec)
```
plant_density : f32   // 0.0 = bare, 1.0 = fully vegetated
plant_type    : u8    // index into PlantSpecies table (0 = empty)
```

`plant_type` encodes the "DNA" via lookup. The `PlantSpecies` table is stored once in RAM:
```
growth_rate   : f32   // CA spread probability per tick
biomass_yield : f32   // energy per density unit consumed
max_density   : f32   // carrying capacity
color_r/g/b   : u8    // base tint for the ground shader
noise_seed    : u8    // selects procedural noise layer in shader
blade_height  : f32   // visual scale in shader
```

Up to 255 species. Species 0 is reserved for "bare ground".

### Grid indexing
```
cell_idx = row * cols + col
row      = floor((world_y - min_y) / P0_CELL_SIZE)
col      = floor((world_x - min_x) / P0_CELL_SIZE)
```

---

## IPC design

Plants update slowly (CA runs every 20–40 ticks ≈ 1–2s). The IPC channel is **separate from the creature double buffer** and uses a **sparse push model**:

- Only non-empty cells are serialized: `[count, x₀, y₀, density₀, type₀, ...]` (4 f32s per live cell + 1 header f32)
- Push triggered after each CA tick, not every frame
- Frontend caches the last plant snapshot; renderer re-uses it until a new push arrives

For the future full-texture path (once we have the ground shader), the buffer becomes a contiguous density texture instead of sparse quads. IPC shape is identical — just the fill strategy changes.

---

## Creature feeding (how a small creature eats)

The creature's world position determines its plant cell (`floor(pos / 4m)`). When feeding:
1. Read `cell.density` and `species.biomass_yield`
2. Add `density × yield × bite_size` to creature energy
3. Decrement `cell.density` by `bite_size`
4. Shader shows the cell visually depleting over subsequent frames

The creature is drawn at its exact world position (sub-cell precision). The feeding animation plays at the creature's location. The ground shader shows the cell thinning. No coupling between visual precision and simulation grain.

---

## Visual rendering (shader path)

No ground shader currently exists. The phased visual plan:
1. **Lean slice** — simple PixiJS `Graphics` circles at plant cell centres (green dots visible on map, no shader)
2. **Texture pass** — plant density uploaded as a 2500×2500 RGBA texture; fragment shader bilinearly samples density, adds procedural noise for fine detail, blends species colours
3. **Art pass** — shader-sarah consults on final look: texture splatting between species, blade-height parallax, wind noise

---

## CA growth (future)

Cellular automata growth runs on a Bevy `FixedUpdate` system, throttled to every 20–40 ticks. Rules:
- **Spread**: adjacent empty cells have a `growth_rate × density` probability of seeding each tick
- **Grazing depletion**: creatures decrement density as above
- **Regrowth**: bare cells near seeded cells recover at `growth_rate / 10` per tick
- **Carrying capacity**: density clamps at `max_density`

Dirty-flag optimization: only sweep cells with non-zero density or adjacent to recently changed cells.

L1 FoodScore integration: each L0 cell aggregates its 25 plant sub-cells into a single FoodScore value, updated after each CA tick.

---

## Implementation phases

### Phase 1 — Lean slice (current)
- [x] `PlantGrid` resource, 4m cells, seeded at startup with 200 grass plants
- [x] `get_plant_buffer()` NAPI method — sparse `[count, x, y, density, type, ...]`
- [x] Electron main: push plant buffer on a 2s interval
- [x] `PlantRenderer.ts` — green circles in PixiJS worldContainer
- [ ] Tests green: PlantGrid unit, PlantRenderer unit

### Phase 2 — Hunger / energy
- `HungerState` component, `hunger_system` depletes energy per tick
- `FeedingSystem` reads plant cell density, credits energy, decrements cell
- Link creature state machine: hungry → seek FoodScore, full → rest/roam

### Phase 3 — CA growth
- `PlantGrowthSystem` throttled to every 20 ticks
- Dirty-cell tracking
- L1 FoodScore aggregate channel

### Phase 4 — Ground shader
- WebGL texture from plant density buffer
- Fragment shader: bilinear density, procedural noise, species colour
- Replace `PlantRenderer.ts` circles with the shader overlay

---

## Related docs
- `docs/biology/todo/hunger-gating.md` — energy/hunger component design
- `docs/biology/todo/influence-maps.md` — FoodScore as third influence layer
- `docs/biology/ideas/stigmergy.md` — FoodScore spatial layer context
