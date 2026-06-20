# ECS Optimization Playbook

**For:** ecs-eddy (Bevy ECS Architect)
**Purpose:** Quick reference during performance optimization sessions
**Source:** Data-Oriented Design principles + Bevy 0.14 best practices

---

## The Golden Rules

1. **Cache Misses Kill Performance** - Optimize for data locality first, algorithms second
2. **Archetype Stability > Feature Flexibility** - Add all components at spawn, mutate state instead
3. **Hot/Cold Data Separation** - Keep frequently-accessed data together, rarely-accessed data separate
4. **Measure, Don't Guess** - Use ECS metrics to validate every optimization

---

## Optimization Decision Tree

```
Is total_tick_us > 50,000μs?
├─ NO → You're fine, monitor trends
└─ YES → Is cache_efficiency_score < 0.7?
    ├─ YES → DATA LAYOUT PROBLEM (use this playbook)
    └─ NO → ALGORITHMIC PROBLEM (profile systems, not metrics)
```

---

## Quick Diagnostics

### Symptoms → Root Cause

| Symptom | Root Cause | Fix |
|---------|------------|-----|
| `cache_efficiency_score` < 0.5 | Severe fragmentation | Consolidate archetypes |
| `top_3_archetype_concentration_pct` < 60% | Too many component combos | Add all capabilities at spawn |
| `archetype_count` > 100 | Component explosion | Audit component design |
| `empty_archetype_percentage` > 25% | Lifecycle churn | Use `Dead` marker, batch despawn |
| `avg_components_per_entity` > 15 | Bloated entities | Hot/cold split |
| `avg_components_per_entity` < 6 | Sparse data | Missing opportunity for batching |
| `largest_archetype_size` < 50% entities | Population fragmented | Identify dominant archetype pattern |
| Movement system slow + good cache score | Not a cache problem | Algorithmic issue (spatial grid?) |

---

## Optimization Patterns

### Pattern 1: Consolidate Archetypes (Capability Marker Strategy)

**Problem:** Entities have different capabilities → 2^N archetypes

```rust
// ❌ BAD: Conditional component spawning
if dna.can_seek() {
    commands.spawn((Position::default(), CanSeek));
} else {
    commands.spawn((Position::default()));
}

// Result: 2 archetypes, fragmentation
```

**Solution:** Add ALL capabilities at spawn, control via state

```rust
// ✅ GOOD: Uniform archetype
commands.spawn((
    Position::default(),
    Velocity::default(),
    Acceleration::default(),
    CanSeek,       // Always present
    CanFlee,       // Always present
    CanWander,     // Always present
    BehaviorState::Catatonic,  // Controlled via enum
));

// Result: 1 archetype, perfect locality
```

**Metrics impact:**
- `archetype_count`: 8 → 1
- `top_3_archetype_concentration_pct`: 45% → 100%
- `cache_efficiency_score`: 0.42 → 0.95

### Pattern 2: Hot/Cold Component Split

**Problem:** Systems load many components but only use a few → cache pollution

```rust
// ❌ BAD: Movement system loads 12 components, uses 3
fn movement_system(
    mut query: Query<(
        &Position,        // HOT: used every tick
        &Velocity,        // HOT: used every tick
        &Acceleration,    // HOT: used every tick
        &DNA,             // COLD: never used in movement
        &BodySize,        // COLD: never used in movement
        &CreatureState,   // COLD: never used in movement
        // ... 6 more components ...
    )>
) {
    for (pos, vel, accel, ...) in query.iter_mut() {
        // Only touch pos, vel, accel
        // Other 9 components pollute cache lines
    }
}
```

**Solution:** Query only what you need

```rust
// ✅ GOOD: Movement system only loads hot data
fn movement_system(
    mut query: Query<(&mut Position, &mut Velocity, &Acceleration)>
) {
    // 3 components = tight cache lines
}

// Cold data accessed in separate system (runs less frequently)
fn energy_consumption_system(
    query: Query<(&DNA, &BodySize, &mut CreatureState)>
) {
    // DNA and BodySize only loaded when needed
}
```

**Metrics impact:**
- `avg_components_per_entity`: Same (still attached)
- Movement system time: 18,000μs → 7,000μs (61% faster)
- `cache_efficiency_score`: 0.64 → 0.82

**Key insight:** Archetype doesn't change, but query filtering improves cache usage.

### Pattern 3: Avoid Archetype Churn (Death Handling)

**Problem:** Removing components during gameplay creates empty archetypes (never GC'd)

```rust
// ❌ BAD: Remove components on death
fn death_system(mut commands: Commands, query: Query<(Entity, &CreatureState)>) {
    for (entity, state) in query.iter() {
        if state.energy <= 0.0 {
            commands.entity(entity)
                .remove::<CanSeek>()       // Archetype change
                .remove::<CanFlee>()       // Archetype change
                .remove::<CanWander>()     // Archetype change
                .remove::<Velocity>();     // Archetype change
            // Creates new archetype (Position only)
            // Old archetype becomes empty (wasted memory)
        }
    }
}
```

**Solution:** Add `Dead` marker, don't remove components

```rust
// ✅ GOOD: Add marker, filter in queries
#[derive(Component)]
struct Dead;

fn death_system(mut commands: Commands, query: Query<(Entity, &CreatureState), Without<Dead>>) {
    for (entity, state) in query.iter() {
        if state.energy <= 0.0 {
            commands.entity(entity).insert(Dead);  // 1 archetype change total
        }
    }
}

// Living entities filter out dead
fn movement_system(
    query: Query<(&Position, &Velocity, &Acceleration), Without<Dead>>
) {
    // Dead entities automatically skipped
}

// Cleanup runs periodically, batch despawn
fn corpse_cleanup_system(
    mut commands: Commands,
    query: Query<(Entity, &Dead)>,
    time: Res<Time>,
) {
    // After decay period, batch despawn
    for (entity, _) in query.iter() {
        commands.entity(entity).despawn();
    }
}
```

**Metrics impact:**
- `empty_archetype_count`: 42 → 2
- `empty_archetype_percentage`: 68% → 3%
- `archetype_overhead_bytes`: 78 KB → 4 KB

### Pattern 4: Component Density Optimization

**Problem:** Too few components = entities spread across many tables

```rust
// ❌ BAD: Minimal components, lots of lookups
#[derive(Component)]
struct Position { x: f32, y: f32 }

// 100 systems all query Position
// Each entity fetch = cache miss (no co-located data)
```

**Solution:** Batch related data into components

```rust
// ✅ GOOD: Related data together
#[derive(Component)]
struct MotionState {
    position: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
}

// Single component fetch = all motion data
// Better cache locality
```

**When to use:**
- **Do:** Batch data that's ALWAYS accessed together
- **Don't:** Batch unrelated data (creates different problem)

**Target:** 8-12 components per entity for simulation entities

### Pattern 5: System Ordering for Cache Reuse

**Problem:** Systems access same components in scattered order → cache eviction

```rust
// ❌ BAD: Interleaved system order
schedule.add_systems((
    movement_system,     // Touches Position, Velocity
    perception_system,   // Touches Position, Perception
    behavior_system,     // Touches Position, BehaviorState
    rotation_system,     // Touches Position, Rotation
));

// Position loaded/evicted 4 times
```

**Solution:** Group systems by component access patterns

```rust
// ✅ GOOD: Batch systems that touch same components
schedule.add_systems((
    // Physics phase: all touch Position/Velocity
    movement_system,
    rotation_system,
    boundary_system,

    // AI phase: all touch BehaviorState
    perception_system,
    behavior_transition_system,
    steering_system,
));

// Position stays hot in cache across movement/rotation/boundary
```

**Bevy automatically parallelizes within phase if queries don't conflict.**

---

## System Profiling Workflow

### Step 1: Identify Bottleneck

```
Check SystemTimingsSnapshot:
- Which system takes > 10% of tick budget?
- Is it consistent or spiky?
```

### Step 2: Determine Problem Type

```
Is the slow system's query result set HUGE?
├─ YES → Algorithmic problem (spatial partitioning, caching)
└─ NO → Cache problem (check EcsMetrics)

Is cache_efficiency_score good (>0.7) but system slow?
├─ YES → Not a cache issue, profile algorithm
└─ NO → Data layout problem, optimize archetype
```

### Step 3: Profile Query Access

```rust
fn slow_system(
    query: Query<(/* many components */)>
) {
    let result_count = query.iter().count();
    log::info!("Query result set size: {}", result_count);

    // If result_count is 100K+ → algorithm problem
    // If result_count is small but slow → cache problem
}
```

### Step 4: Optimize Based on Diagnosis

**Large result set:**
- Add query filters (`With<X>`, `Without<Y>`)
- Implement spatial partitioning (grid, quadtree)
- Cache results (update every N ticks)

**Cache problem:**
- Reduce component count in query
- Check archetype fragmentation (EcsMetrics)
- Consolidate archetypes

---

## Bevy-Specific Gotchas

### 1. Empty Archetype Never Destroyed

**Fact:** Once created, archetypes persist for World lifetime.

**Implication:** Spawning entities with different component sets creates permanent memory overhead.

**Solution:** Standardize component sets, spawn uniform entities.

### 2. Adding/Removing Components is Expensive

**Fact:** Component add/remove = entity moves to new archetype table.

**Cost:** O(components) copy operation + archetype metadata update.

**Solution:** Add all components at spawn, mutate state enums instead.

### 3. Query Filters Don't Change Archetype

**Fact:** `With<X>` / `Without<Y>` filters are FREE (no archetype change).

**Use this:** Control behavior via filters, not component add/remove.

```rust
// ✅ FREE
Query<&Position, With<CanSeek>>   // Filter at query time

// ❌ EXPENSIVE
commands.entity(e).insert(CanSeek);  // Archetype change
```

### 4. Component Order in Query Doesn't Matter

**Fact:** Bevy iterates archetypes, not entities.

**Implication:** Query iteration order is archetype-dependent (not insertion order).

**Don't rely on:** Entity spawn order for system logic.

### 5. Bevy Parallelizes Systems Automatically

**Fact:** Systems with non-conflicting queries run in parallel.

**Conflict:**
```rust
fn system_a(q: Query<&mut Position>) {}  // Mutable access
fn system_b(q: Query<&mut Position>) {}  // Conflict! Sequential.
```

**No conflict:**
```rust
fn system_a(q: Query<&mut Position>) {}
fn system_b(q: Query<&Velocity>) {}      // Parallel!
```

**Optimization:** Minimize mutable queries to maximize parallelism.

---

## Metrics Thresholds (Quick Ref)

| Metric | Green | Yellow | Red |
|--------|-------|--------|-----|
| `cache_efficiency_score` | >0.8 | 0.5-0.8 | <0.5 |
| `top_3_archetype_concentration_pct` | >85% | 60-85% | <60% |
| `archetype_count` | <20 | 20-50 | >50 |
| `empty_archetype_percentage` | <5% | 5-20% | >20% |
| `avg_components_per_entity` | 8-12 | 6-8 or 12-15 | <6 or >15 |
| `total_tick_us` | <40,000 | 40-50,000 | >50,000 |
| `largest_archetype_size` | >80% entities | 50-80% | <50% |

**Priority:** Fix red metrics first, monitor yellow metrics.

---

## Pre-Optimization Checklist

Before optimizing, answer these:

- [ ] Have I collected baseline ECS metrics?
- [ ] Do I know which system is slow (SystemTimings)?
- [ ] Is `cache_efficiency_score` < 0.7? (If yes → data layout problem)
- [ ] Have I identified the dominant archetype(s)? (Check `top_archetypes`)
- [ ] Do I understand why archetypes are fragmented? (Review component spawning)
- [ ] Am I adding/removing components during gameplay? (Archetype churn)
- [ ] Are queries loading unused components? (Hot/cold split opportunity)

**If all checks pass, proceed with optimization. If not, gather more data.**

---

## Post-Optimization Validation

After optimization:

- [ ] Collect new ECS metrics
- [ ] Compare `cache_efficiency_score` (should increase)
- [ ] Compare `top_3_archetype_concentration_pct` (should increase)
- [ ] Compare `archetype_count` (should decrease or stay same)
- [ ] Compare system timings (should improve for affected systems)
- [ ] Verify behavior correctness (run integration tests)
- [ ] Check for regressions in unrelated systems

**Document:**
- What changed (code diff)
- Metrics before/after
- Performance impact (% improvement)
- Lessons learned

---

## Common Mistakes

### 1. Premature Optimization

**Mistake:** Optimizing before measuring.

**Fix:** Always collect metrics FIRST. Optimization without data is guessing.

### 2. Optimizing the Wrong Thing

**Mistake:** Focusing on system with highest μs count.

**Fix:** Consider % of tick budget. A 5ms system that's 10% of budget may be fine. A 2ms system that should be 0.1ms is the real problem.

### 3. Breaking Archetype Stability

**Mistake:** "I'll just add this component when needed..."

**Fix:** Plan component set at design time. Add all at spawn, mutate state.

### 4. Over-Batching Components

**Mistake:** "I'll put everything in one big component for locality!"

**Fix:** Only batch data accessed TOGETHER. Unrelated data in same component creates different cache problems.

### 5. Ignoring Empty Archetypes

**Mistake:** "Empty archetypes are harmless..."

**Fix:** Empty archetypes = permanent memory waste. Investigate lifecycle bugs.

---

## When to Consult This Playbook

**Use this playbook when:**
- Starting a new optimization sprint
- `total_tick_us` exceeds 50,000μs
- `cache_efficiency_score` drops below 0.7
- Adding new component types to the simulation
- Experiencing performance regression after feature add
- Confused about archetype fragmentation in ECS metrics

**Don't use this playbook when:**
- Implementing new features (focus on correctness first)
- `cache_efficiency_score` > 0.8 (you're doing fine)
- Problem is clearly algorithmic (spatial partitioning, O(N²) loop)
- Production debugging (this is for dev environment only)

---

## Quick Wins Ranking

Ordered by effort/impact ratio:

1. **Add all capabilities at spawn** - 10 min effort, massive impact
2. **Hot/cold query splitting** - 30 min effort, large impact
3. **Replace component remove with `Dead` marker** - 20 min effort, medium impact
4. **Group systems by component access** - 15 min effort, small-medium impact
5. **Consolidate related components** - 2 hours effort, variable impact

**Start with #1 and #2 for maximum ROI.**

---

## Resources

- **ECS Metrics Spec:** `docs/scale/ecs-metrics-specification.md`
- **Bevy Archetype Docs:** `https://docs.rs/bevy/latest/bevy/ecs/archetype/`
- **Biology Notes:** `docs/biology/biology-notes.md` (for trait design)
- **System AGENTS:** `apps/simulation/AGENTS.md` (ECS architecture patterns)

---

**Remember: In Data-Oriented Design, data layout IS the algorithm. Optimize the data, and the code optimizes itself.**
