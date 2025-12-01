# Perception System (Current Implementation)

**Status:** Implemented
**Location:** `apps/simulation/src/simulation/perception/`

## What It Does

Provides FOV-based neighbor detection within a directional cone. Creatures detect nearby entities within their field of view each tick. FOV angle determines both the arc width and perception range through a biological tradeoff (narrow FOV = longer range).

**See also:** `docs/biology/done/dna-fov.md` for detailed FOV documentation.

## Key Components

### Perception Component

**Location:** `perception/components.rs`

**Key features:**
- Fixed-size array (21 neighbors max) - no heap allocations, cache-friendly
- FOV cone perception - creatures only see within their field of view
- Range derived from FOV angle (narrow = longer, wide = shorter)
- `MAX_PERCEIVED_NEIGHBORS = 21` (constants.rs)

**Scaling:**
- `from_body_size(body_length)` - Range = body_length × 10.0 (at 180 FOV)
- Range varies with FOV: `range = base_range * (180/fov)^0.4`

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

**Algorithm:** O(N²) brute force (see `SPRINTS/spatial-grid/SPRINT_PLAN.md` for O(N) optimization)

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
| **Current (Brute Force)** | O(N²) | ~50ms | All-pairs distance checks |
| **Spatial Grid (Planned)** | O(N) | ~3-5ms | See `SPRINTS/spatial-grid/SPRINT_PLAN.md` |
| **Stochastic Updates (Future)** | O(N) | ~1-2ms @ 200K | Only ~10% update per tick |

## Current Limitations

### Instant Updates (No Reaction Time)

**Problem:** Perception updates every tick, all creatures simultaneously.

**Biological reality:**
- Small animals: ~68ms reaction time (fast)
- Large animals: ~500ms reaction time (slow)

**Fix:** Planned `VisionTiming` component with stochastic updates.

## Future Work

### Stochastic Vision

Planned reaction-time-gated perception updates to reduce CPU load and add biological realism.

### Additional DNA Vision Genes

- `visual_range_multiplier` - Independent range control (4-25x)
- `neural_speed` - Reaction time modifier (0.5-2.0)

**See:** `docs/biology/todo/dna-driven-fov.md` for full vision gene roadmap

### Spatial Awareness (Future)

- Obstacle detection (currently placeholder, needs terrain/wall collision)
- Predator-prey dynamics (brain system will use perception to trigger flee/chase)
- Social behaviors (flocking, herding requires perception of group mates)

## References

- `apps/simulation/src/simulation/perception/` - Implementation
- `docs/biology/done/movement-physics.md` - Perception constants rationale

---

**Last Updated:** 2025-11-30
