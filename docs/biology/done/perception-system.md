# Perception System (Current Implementation)

**Status:** ✅ Implemented
**Location:** `apps/simulation/src/simulation/perception/`

## What It Does

Provides basic 360° neighbor detection within a fixed range. Creatures detect nearby entities each tick and use this information for avoidance steering. This is a **basic omnidirectional system** - no field-of-view (FOV) restrictions or directional awareness yet.

**Future:** Will add DNA-driven vision genes (range variation, FOV cones, neural speed, stochastic updates).

## Key Components

### Perception Component

**Location:** `perception/components.rs:12-75`

**Key features:**
- Fixed-size array (40 neighbors max) - no heap allocations, cache-friendly
- 360° awareness - no blind spots (unrealistic, planned for future)
- Range-based only - detects all entities within range, regardless of direction
- `MAX_PERCEIVED_NEIGHBORS = 40` (components.rs:5)

**Scaling:**
- `from_body_size(body_length)` - Range = body_length × 10.0
- Default range: 10.0m

### AvoidanceBehavior Component

**Location:** `perception/components.rs:77-127`

**Scaling:**
- `from_body_size(body_length)` - personal_space = body_length + 1.5m
- Default: personal_space = 2.5m (for 1.0m creature)

**Key methods:**
- `panic_threshold()` - Returns personal_space × 0.5 (emergency response distance)
- `effective_personal_space(energy_fraction)` - ✅ Energy-modulated personal space

### Energy-Modulated Personal Space ✅ Implemented

**Biological principle:** Hungry creatures tolerate closer proximity to reach resources.

**Formula:** `effective_space = base × (0.4 + 0.6 × energy_fraction)`

**Energy effects:**
- 100% energy → 1.0× modifier (full personal space maintained)
- 50% energy → 0.7× modifier (30% reduction)
- 0% energy → 0.4× modifier (60% reduction, starvation override)

**Biological basis:**
- Ghrelin (hunger hormone) reduces territorial aggression
- Real-world examples: Vultures at kills (100m spacing → body contact), wolves at food

**Implementation:** `components.rs:84-91, 118-120`

## Perception Update System

**Location:** `perception/systems.rs`

**Algorithm:** O(N²) brute force (optimized to O(N·log N) with spatial grid)

**Process:**
1. Collect all (Entity, Position, BodySize) into scratch buffer
2. For each creature with Perception component:
   - Clear previous neighbor list
   - Skip if behavior is Catatonic (inactive creatures don't perceive)
   - For each other creature: calculate edge-to-edge distance
   - If edge_distance ≤ perception.range: add to nearby list (max 40)

### Edge-to-Edge Distance

**Formula:** `edge_distance = center_distance - self_radius - other_radius`

**Why edge-to-edge?**
- Large creatures are easier to see (bigger visual profile)
- Small creatures are harder to see (smaller visual profile)
- Matches biological reality: elephants visible from further than mice

### Scratch Buffer Optimization

**Location:** `components.rs:7-10`

Reusable allocation for position queries, reduces heap pressure during perception updates.

## Constants

**See:** `apps/simulation/src/simulation/movement/constants.rs` (PERCEPTION struct)

| Constant | Purpose |
|----------|---------|
| `perception_multiplier` | Range = body_length × this |
| `personal_space` | Base spacing buffer |
| `panic_threshold_ratio` | Panic at 50% of personal space |

**Example values (1.0m creature):**
- Perception range: 10.0m
- Personal space: 2.5m (1.0 + 1.5)
- Panic threshold: 1.25m (50% of 2.5m)

## Integration with Other Systems

**Avoidance System** (`creatures/behaviors/avoidance.rs`):
- Reads `Perception::iter_neighbors()` to calculate avoidance steering forces
- Weighs force by distance (closer = stronger)
- Applies panic force (50N) if within panic threshold
- Accumulates forces into `Acceleration` component

**Brain System** (future):
- Will read perception data to make decisions (flee, seek food, etc.)
- Currently: Perception only used for avoidance steering, no higher-level AI yet

## Performance Characteristics

| Implementation | Complexity | Cost @ 20K | Notes |
|---------------|------------|------------|-------|
| **Current (Brute Force)** | O(N²) | ~120ms | All-pairs distance checks |
| **Spatial Grid (Optimized)** | O(N·log N) | ~15ms | 833× fewer comparisons |
| **Stochastic Updates (Planned)** | O(N·log N) | ~1-2ms @ 200K | Only ~10% update per tick |

## Current Limitations

### 360° Omniscient Vision (Unrealistic)

**Problem:** Creatures see in all directions simultaneously, no blind spots.

**Biological reality:**
- Predators: 90-120° FOV (binocular vision, depth perception)
- Prey: 270-340° FOV (panoramic, early warning)

**Fix:** Planned addition of FOV cones and facing direction.

### Fixed Range (No Variation)

**Problem:** All creatures have identical perception range (10× body size).

**Biological reality:**
- Hawks: Extreme long range (22× body length)
- Rabbits: Short range, wide FOV (6× body length)

**Fix:** Planned `visual_range_multiplier` gene (4.0-25.0 range).

### Instant Updates (No Reaction Time)

**Problem:** Perception updates every tick (45ms), all creatures simultaneously.

**Biological reality:**
- Small animals: ~68ms reaction time (fast)
- Large animals: ~500ms reaction time (slow)

**Fix:** Planned `VisionTiming` component with stochastic updates.

### No Directional Bias

**Problem:** Stationary and moving creatures have identical perception.

**Biological reality:**
- Stationary: 360° awareness (scanning)
- Moving: Forward-biased (attention on travel direction)

**Fix:** Will use velocity vector as facing direction for FOV cone.

## Future Work

### Future: DNA-Driven Vision & FOV

**Planned features:**
1. Vision genes: range multiplier (4-25), FOV arc (60-360°), neural speed (0.5-2.0)
2. FOV cones: Directional perception, blind spots behind creatures
3. Stochastic updates: Reaction-time-gated (68-500ms per creature)
4. Trade-offs: Wide FOV reduces effective range (0.7× penalty)

**See:** Vision system design docs in project planning

### Spatial Awareness (Future)

- Obstacle detection (currently placeholder, needs terrain/wall collision)
- Predator-prey dynamics (brain system will use perception to trigger flee/chase)
- Social behaviors (flocking, herding requires perception of group mates)

## References

- `apps/simulation/src/simulation/perception/` - Implementation
- `docs/biology/done/movement-physics.md` - Perception constants rationale

---

**Last Updated:** 2025-11-29
