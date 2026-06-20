# ECS Metrics Specification for Data-Oriented Design

**Author:** ecs-eddy (Bevy ECS Architect & Simulation Performance Engineer)
**Date:** 2025-11-20
**Status:** Design Specification
**Target:** 150K-200K entities @ 20Hz (50ms tick budget)

---

## Executive Summary

This specification defines comprehensive ECS metrics that expose Data-Oriented Design patterns and enable cache-conscious optimization. These metrics are designed to answer the critical question: **"Is our data layout helping or hurting performance?"**

**Key Design Principles:**
- Collection overhead < 1ms (within 50ms tick budget)
- Leverage Bevy's built-in World/Archetype API
- Expose cache behavior and memory layout patterns
- Validate DOD decisions (e.g., "Did moving DNA to cold storage help?")

---

## 1. Complete `EcsMetrics` Rust Struct

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EcsMetrics {
    // ============================================================
    // ARCHETYPE HEALTH
    // ============================================================

    /// Total number of unique archetypes in the World
    /// Growth indicates fragmentation (too many component combinations)
    pub archetype_count: usize,

    /// Number of entities in the largest archetype
    /// High values = good cache locality (entities share same layout)
    pub largest_archetype_size: usize,

    /// Number of archetypes with zero entities
    /// Empty archetypes waste memory (archetype tables never destroyed)
    pub empty_archetype_count: usize,

    /// Total number of archetype-components across all archetypes
    /// Archetype with N components = N archetype-components
    /// Measures total memory overhead of archetype metadata
    pub total_archetype_components: usize,

    /// Average entities per non-empty archetype
    /// Low values = fragmentation, poor cache behavior
    pub avg_entities_per_archetype: f32,

    /// Percentage of archetypes that are empty
    /// High percentage = wasted memory, archetype churn
    pub empty_archetype_percentage: f32,

    // ============================================================
    // ENTITY DISTRIBUTION
    // ============================================================

    /// Total entities in the World
    pub entity_count: usize,

    /// Top 5 largest archetypes (sorted descending by entity count)
    /// Format: [(entity_count, component_count), ...]
    /// Identifies which component combinations dominate
    pub top_archetypes: Vec<(usize, usize)>,

    /// Percentage of entities in the largest archetype
    /// High concentration = good (most entities share optimal layout)
    pub largest_archetype_concentration_pct: f32,

    /// Percentage of entities in top 3 archetypes
    /// Should be > 80% for good cache behavior
    pub top_3_archetype_concentration_pct: f32,

    // ============================================================
    // COMPONENT DENSITY
    // ============================================================

    /// Average number of components per entity
    /// Too low = sparse data, cache misses
    /// Too high = bloated entities, wasted loads
    pub avg_components_per_entity: f32,

    /// Number of unique component types in the World
    /// Growth over time may indicate scope creep
    pub unique_component_types: usize,

    /// Percentage of entities with "hot" components (Position, Velocity, Acceleration)
    /// Should be ~100% for simulation entities
    pub hot_component_coverage_pct: f32,

    /// Percentage of entities with "cold" components (DNA, BodySize)
    /// Measures success of hot/cold split optimization
    pub cold_component_coverage_pct: f32,

    // ============================================================
    // SYSTEM EXECUTION (from existing instrumentation)
    // ============================================================

    /// Per-system execution times (microseconds)
    /// Already tracked in SystemTimings, included for completeness
    pub system_timings_us: SystemTimingsSnapshot,

    /// Total tick time (microseconds)
    /// Should stay < 50,000 for 20Hz target
    pub total_tick_us: u64,

    // ============================================================
    // PERFORMANCE INDICATORS
    // ============================================================

    /// Estimated cache efficiency score (0.0 - 1.0)
    /// Based on archetype concentration and component density
    /// 1.0 = perfect (all entities same archetype, hot data only)
    /// 0.0 = terrible (maximum fragmentation)
    pub cache_efficiency_score: f32,

    /// Archetype fragmentation score (0.0 - 1.0)
    /// 0.0 = no fragmentation (few archetypes, high entity concentration)
    /// 1.0 = severe fragmentation (many archetypes, low concentration)
    pub fragmentation_score: f32,

    /// Estimated memory overhead from archetype metadata (bytes)
    /// Archetype tables persist forever, this tracks waste
    pub archetype_overhead_bytes: usize,
}
```

---

## 2. Implementation: `collect_ecs_metrics(world: &World)`

```rust
use bevy_ecs::prelude::*;
use bevy_ecs::archetype::ArchetypeId;

pub fn collect_ecs_metrics(world: &World) -> EcsMetrics {
    let archetypes = world.archetypes();

    // ============================================================
    // ARCHETYPE HEALTH
    // ============================================================

    let archetype_count = archetypes.len();
    let total_archetype_components = archetypes.archetype_components_len();

    let mut archetype_sizes: Vec<usize> = archetypes
        .iter()
        .map(|arch| arch.len())
        .collect();

    let entity_count = world.entities().len();
    let empty_archetype_count = archetype_sizes.iter().filter(|&&size| size == 0).count();
    let largest_archetype_size = *archetype_sizes.iter().max().unwrap_or(&0);

    let non_empty_archetypes: Vec<usize> = archetype_sizes.iter()
        .copied()
        .filter(|&size| size > 0)
        .collect();

    let avg_entities_per_archetype = if !non_empty_archetypes.is_empty() {
        non_empty_archetypes.iter().sum::<usize>() as f32 / non_empty_archetypes.len() as f32
    } else {
        0.0
    };

    let empty_archetype_percentage = if archetype_count > 0 {
        (empty_archetype_count as f32 / archetype_count as f32) * 100.0
    } else {
        0.0
    };

    // ============================================================
    // ENTITY DISTRIBUTION
    // ============================================================

    let mut archetype_info: Vec<(usize, usize)> = archetypes
        .iter()
        .map(|arch| (arch.len(), arch.components().count()))
        .collect();

    // Sort by entity count descending
    archetype_info.sort_by(|a, b| b.0.cmp(&a.0));

    let top_archetypes: Vec<(usize, usize)> = archetype_info
        .iter()
        .take(5)
        .copied()
        .collect();

    let largest_archetype_concentration_pct = if entity_count > 0 {
        (largest_archetype_size as f32 / entity_count as f32) * 100.0
    } else {
        0.0
    };

    let top_3_entity_count: usize = archetype_info.iter().take(3).map(|(count, _)| count).sum();
    let top_3_archetype_concentration_pct = if entity_count > 0 {
        (top_3_entity_count as f32 / entity_count as f32) * 100.0
    } else {
        0.0
    };

    // ============================================================
    // COMPONENT DENSITY
    // ============================================================

    let total_components: usize = archetype_info.iter()
        .map(|(entity_count, component_count)| entity_count * component_count)
        .sum();

    let avg_components_per_entity = if entity_count > 0 {
        total_components as f32 / entity_count as f32
    } else {
        0.0
    };

    // Count unique component types across all archetypes
    let mut unique_components = std::collections::HashSet::new();
    for arch in archetypes.iter() {
        for component_id in arch.components() {
            unique_components.insert(component_id);
        }
    }
    let unique_component_types = unique_components.len();

    // Hot component coverage (Position, Velocity, Acceleration)
    let hot_component_coverage_pct = calculate_component_coverage(
        world,
        &[
            std::any::TypeId::of::<Position>(),
            std::any::TypeId::of::<Velocity>(),
            std::any::TypeId::of::<Acceleration>(),
        ],
    );

    // Cold component coverage (DNA, BodySize)
    // TODO: Implement when DNA component exists
    let cold_component_coverage_pct = 0.0;

    // ============================================================
    // PERFORMANCE INDICATORS
    // ============================================================

    // Cache efficiency: weighted by archetype concentration and density
    let cache_efficiency_score = calculate_cache_efficiency(
        top_3_archetype_concentration_pct,
        avg_components_per_entity,
    );

    // Fragmentation: inversely proportional to concentration
    let fragmentation_score = 1.0 - (top_3_archetype_concentration_pct / 100.0);

    // Archetype overhead estimation
    // Approximation: 128 bytes per archetype metadata + 8 bytes per archetype-component
    let archetype_overhead_bytes = (archetype_count * 128) + (total_archetype_components * 8);

    // ============================================================
    // SYSTEM EXECUTION (from existing instrumentation)
    // ============================================================

    #[cfg(feature = "dev-tools")]
    let system_timings_us = world
        .get_resource::<crate::instrumentation::SystemTimings>()
        .map(|t| t.snapshot())
        .unwrap_or_default();

    #[cfg(not(feature = "dev-tools"))]
    let system_timings_us = SystemTimingsSnapshot::default();

    let total_tick_us = system_timings_us.total_tick_us;

    EcsMetrics {
        archetype_count,
        largest_archetype_size,
        empty_archetype_count,
        total_archetype_components,
        avg_entities_per_archetype,
        empty_archetype_percentage,

        entity_count,
        top_archetypes,
        largest_archetype_concentration_pct,
        top_3_archetype_concentration_pct,

        avg_components_per_entity,
        unique_component_types,
        hot_component_coverage_pct,
        cold_component_coverage_pct,

        system_timings_us,
        total_tick_us,

        cache_efficiency_score,
        fragmentation_score,
        archetype_overhead_bytes,
    }
}

// ============================================================
// HELPER FUNCTIONS
// ============================================================

fn calculate_component_coverage(world: &World, type_ids: &[std::any::TypeId]) -> f32 {
    let entity_count = world.entities().len();
    if entity_count == 0 {
        return 0.0;
    }

    // Count entities that have ALL specified components
    let mut matching_entities = 0;

    for archetype in world.archetypes().iter() {
        // Check if archetype contains ALL required components
        let has_all = type_ids.iter().all(|&type_id| {
            archetype.components().any(|component_id| {
                world.components()
                    .get_info(component_id)
                    .map(|info| info.type_id() == Some(type_id))
                    .unwrap_or(false)
            })
        });

        if has_all {
            matching_entities += archetype.len();
        }
    }

    (matching_entities as f32 / entity_count as f32) * 100.0
}

fn calculate_cache_efficiency(
    top_3_concentration: f32,
    avg_components: f32,
) -> f32 {
    // Target: 80%+ concentration, 8-12 components per entity
    let concentration_score = (top_3_concentration / 80.0).min(1.0);

    // Penalize too few (<6) or too many (>15) components
    let density_score = if avg_components >= 6.0 && avg_components <= 15.0 {
        1.0
    } else if avg_components < 6.0 {
        avg_components / 6.0
    } else {
        15.0 / avg_components
    };

    // Weighted average: concentration matters more
    (concentration_score * 0.7) + (density_score * 0.3)
}
```

---

## 3. Interpretation Guide: What Do These Metrics Tell Us?

### Archetype Health Metrics

**`archetype_count`**
- **What it means:** Number of unique component combinations in your World
- **Good:** < 20 archetypes for 150K entities (tight, focused design)
- **Warning:** > 50 archetypes (fragmentation, design smell)
- **Critical:** > 100 archetypes (severe fragmentation, cache thrashing)
- **Action:** Investigate why entities have so many different component sets

**`largest_archetype_size`**
- **What it means:** How many entities share the optimal layout
- **Good:** > 100,000 entities (80%+ of total)
- **Warning:** < 50,000 entities (fragmented population)
- **Critical:** < 10,000 entities (severe fragmentation)
- **Action:** Consolidate components to create a dominant archetype

**`empty_archetype_count` / `empty_archetype_percentage`**
- **What it means:** Wasted archetype tables (never destroyed, persist forever)
- **Good:** 0-5% empty archetypes
- **Warning:** 10-20% empty (spawning/despawning churn)
- **Critical:** > 25% empty (architectural problem)
- **Action:** Review entity lifecycle, avoid add/remove component patterns

**`avg_entities_per_archetype`**
- **What it means:** Average population density per archetype
- **Good:** > 5,000 entities/archetype
- **Warning:** < 1,000 entities/archetype
- **Critical:** < 100 entities/archetype
- **Action:** Merge capabilities, reduce component combinations

### Entity Distribution Metrics

**`top_archetypes`**
- **What it means:** Top 5 archetypes sorted by entity count
- **Example:** `[(120000, 8), (25000, 6), (3000, 12), (1500, 4), (500, 10)]`
- **Interpretation:** 120K entities with 8 components (dominant archetype ✅)
- **Action:** Inspect component composition of dominant archetypes

**`largest_archetype_concentration_pct`**
- **What it means:** % of entities in the single largest archetype
- **Good:** > 70% (excellent locality)
- **Warning:** 40-70% (moderate fragmentation)
- **Critical:** < 40% (poor locality)
- **Action:** Identify why entities differ, consolidate components

**`top_3_archetype_concentration_pct`**
- **What it means:** % of entities in top 3 archetypes
- **Good:** > 85% (most entities use similar layouts)
- **Warning:** 60-85% (acceptable but improvable)
- **Critical:** < 60% (excessive variety)
- **Action:** This is THE most important cache metric - optimize this first

### Component Density Metrics

**`avg_components_per_entity`**
- **What it means:** Average component count per entity
- **Good:** 8-12 components (balanced, focused entities)
- **Warning:** < 6 (too sparse, cache misses) OR > 15 (bloated)
- **Critical:** < 4 (minimal entity) OR > 20 (god entity anti-pattern)
- **Action:** Review component design, ensure hot/cold split

**`unique_component_types`**
- **What it means:** Total component variety in the World
- **Good:** < 30 component types (focused domain)
- **Warning:** 30-60 component types (moderate complexity)
- **Critical:** > 60 component types (scope creep)
- **Action:** Audit components, consider consolidation

**`hot_component_coverage_pct` / `cold_component_coverage_pct`**
- **What it means:** Success of hot/cold data separation
- **Hot (Position/Velocity/Acceleration):** Should be 100% for simulation entities
- **Cold (DNA/BodySize):** Lower is better if rarely accessed
- **Action:** Move infrequently-accessed data to separate storage

### Performance Indicators

**`cache_efficiency_score`**
- **What it means:** Overall data layout quality (0.0 - 1.0)
- **Calculation:** 70% archetype concentration + 30% component density
- **Good:** > 0.8 (excellent cache behavior)
- **Warning:** 0.5-0.8 (room for improvement)
- **Critical:** < 0.5 (poor data layout)
- **Action:** This is your north star - optimize to increase this score

**`fragmentation_score`**
- **What it means:** Inverse of archetype concentration (0.0 - 1.0)
- **Good:** < 0.2 (minimal fragmentation)
- **Warning:** 0.2-0.4 (moderate fragmentation)
- **Critical:** > 0.4 (severe fragmentation)
- **Action:** Reduce archetype variety, consolidate components

**`archetype_overhead_bytes`**
- **What it means:** Memory wasted on archetype metadata
- **Calculation:** ~128 bytes/archetype + 8 bytes/archetype-component
- **Good:** < 10 KB (tight design)
- **Warning:** 10-50 KB (acceptable overhead)
- **Critical:** > 50 KB (excessive archetypes)
- **Action:** Reduce archetype count to reclaim memory

---

## 4. Performance Thresholds: When Should We Be Concerned?

### Red Alerts (Immediate Action Required)

| Metric | Threshold | Symptom | Root Cause |
|--------|-----------|---------|------------|
| `cache_efficiency_score` | < 0.5 | Poor performance | Fragmented data layout |
| `top_3_archetype_concentration_pct` | < 60% | Cache thrashing | Too many component combinations |
| `archetype_count` | > 100 | High memory overhead | Component explosion |
| `empty_archetype_percentage` | > 25% | Wasted memory | Excessive spawning/despawning |
| `fragmentation_score` | > 0.5 | Cache misses | Archetype churn |
| `total_tick_us` | > 50,000 | Missed tick budget | System bottleneck |

### Yellow Warnings (Monitor Closely)

| Metric | Threshold | Action |
|--------|-----------|--------|
| `cache_efficiency_score` | 0.5-0.7 | Investigate archetype distribution |
| `top_3_archetype_concentration_pct` | 60-80% | Consolidate component usage |
| `archetype_count` | 50-100 | Audit component combinations |
| `avg_components_per_entity` | < 6 OR > 15 | Review component design |
| `largest_archetype_size` | < 50% of entity_count | Analyze top archetypes |

### Green Zones (Optimal Performance)

| Metric | Target | Meaning |
|--------|--------|---------|
| `cache_efficiency_score` | > 0.8 | Excellent data layout |
| `top_3_archetype_concentration_pct` | > 85% | Tight archetype clustering |
| `archetype_count` | < 20 | Focused design |
| `empty_archetype_percentage` | < 5% | Minimal waste |
| `avg_components_per_entity` | 8-12 | Balanced entity design |
| `total_tick_us` | < 40,000 | Headroom for scaling |

---

## 5. Example Optimization Scenarios

### Scenario 1: Fragmentation from Capability Markers

**Symptoms:**
- `archetype_count`: 48
- `top_3_archetype_concentration_pct`: 52%
- `cache_efficiency_score`: 0.45

**Diagnosis:** Entities have different combinations of capability markers (CanSeek, CanFlee, CanWander), creating 2^N archetypes.

**Solution:**
1. Add ALL capability markers at spawn (even if unused)
2. Use `BehaviorState` enum to toggle behavior instead of add/remove components
3. Query with `With<CanSeek>` filters instead of archetype changes

**Expected Improvement:**
- `archetype_count`: 48 → 3-5
- `top_3_archetype_concentration_pct`: 52% → 95%
- `cache_efficiency_score`: 0.45 → 0.88

### Scenario 2: Hot/Cold Component Split

**Symptoms:**
- `avg_components_per_entity`: 18
- `cache_efficiency_score`: 0.62
- `system_timings_us.movement_us`: 18,000 μs (high)

**Diagnosis:** Movement system loads 18 components but only uses 3 (Position, Velocity, Acceleration). DNA, BodySize, etc. pollute cache lines.

**Solution:**
1. Separate "hot" (frequently accessed) from "cold" (rarely accessed) components
2. Query only hot components in movement system
3. Access cold components in separate systems (perception, spawning)

**Expected Improvement:**
- `avg_components_per_entity`: 18 → 8 (hot) + 10 (cold storage)
- `system_timings_us.movement_us`: 18,000 → 8,000 μs (55% faster)
- `cache_efficiency_score`: 0.62 → 0.81

### Scenario 3: Empty Archetype Accumulation

**Symptoms:**
- `empty_archetype_count`: 42
- `empty_archetype_percentage`: 68%
- `archetype_overhead_bytes`: 78 KB

**Diagnosis:** Spawning/despawning with different component sets creates persistent empty archetypes (never destroyed).

**Solution:**
1. Use `Dead` marker component instead of despawning
2. Add ALL components at spawn (no incremental component addition)
3. Batch despawn during cleanup phase (not mid-tick)

**Expected Improvement:**
- `empty_archetype_count`: 42 → 3
- `empty_archetype_percentage`: 68% → 5%
- `archetype_overhead_bytes`: 78 KB → 6 KB

### Scenario 4: System Bottleneck Identification

**Symptoms:**
- `total_tick_us`: 62,000 (exceeds 50,000 budget)
- `system_timings_us.perception_us`: 45,000 μs (90% of tick)
- `cache_efficiency_score`: 0.73 (not terrible)

**Diagnosis:** Perception system is O(N²) spatial query, not a data layout problem.

**Solution:**
1. Implement spatial partitioning (grid or quadtree)
2. Cache perception results (update every 2-3 ticks)
3. Use archetype filtering to skip catatonic entities

**Expected Improvement:**
- `system_timings_us.perception_us`: 45,000 → 12,000 μs (73% faster)
- `total_tick_us`: 62,000 → 29,000 μs (fits budget with headroom)
- `cache_efficiency_score`: 0.73 → 0.76 (minor improvement from better queries)

### Scenario 5: Validating DNA Migration

**Symptoms (Before DNA migration):**
- `avg_components_per_entity`: 8
- `unique_component_types`: 42
- Many hardcoded constants in systems

**Solution:**
1. Add `DNA` component to all entities
2. Migrate hardcoded constants to DNA gene expression
3. Measure impact on cache behavior

**Expected Metrics (After):**
- `avg_components_per_entity`: 8 → 9 (added DNA)
- `cold_component_coverage_pct`: 0% → 100% (DNA is cold)
- `cache_efficiency_score`: Monitor for < 5% degradation
- **Validation:** If score drops > 10%, consider separate DNA storage table

---

## 6. Integration with Instrumentation Architecture

### Collection Point

```rust
// In main simulation loop (after schedule.run())
#[cfg(feature = "dev-tools")]
fn snapshot_system(world: &World) {
    let ecs_metrics = collect_ecs_metrics(world);

    // Send to dev-ui via existing IPC channel
    world.resource_mut::<IpcWriter>().send_metrics(ecs_metrics);
}
```

### Performance Budget

- **Target:** < 500 μs collection overhead
- **Measurement:** Add timing guard around `collect_ecs_metrics()`
- **Frequency:** Once per tick (20Hz = every 50ms)
- **Impact:** < 1% of 50ms tick budget

### Dev-UI Display

Extend `SystemTimingsPanel.tsx`:

```typescript
interface EcsMetricsPanel {
  // Real-time sparklines
  archetypeCount: Sparkline;
  cacheEfficiencyScore: Sparkline;
  top3Concentration: Sparkline;

  // Threshold alerts
  fragmentationWarning: boolean;  // fragmentation_score > 0.4
  concentrationWarning: boolean;  // top_3 < 60%

  // Archetype breakdown table
  topArchetypes: { entities: number; components: number }[];
}
```

---

## 7. Future Enhancements

### Phase 2: Query Performance Tracking

**Challenge:** Bevy doesn't expose per-query metrics out-of-the-box.

**Approach:**
- Wrap queries with timing guards
- Track query result set sizes
- Measure iteration overhead

**Example:**
```rust
pub struct QueryMetrics {
    pub query_name: &'static str,
    pub result_count: usize,
    pub iteration_us: u64,
    pub access_pattern: AccessPattern,  // Immutable vs Mutable
}

enum AccessPattern {
    ImmutableOnly,      // &Component (parallel-safe)
    MutableSingle,      // &mut Component (exclusive)
    MutableMultiple,    // Multiple &mut (sequential)
}
```

### Phase 3: System Parallelism Effectiveness

**Challenge:** Measure whether systems actually run in parallel.

**Approach:**
- Use Bevy's SystemGraph to detect dependencies
- Measure wall-clock time vs CPU time
- Calculate parallelism factor (CPU time / wall time)

**Metric:**
```rust
pub struct ParallelismMetrics {
    pub potential_parallel_systems: usize,  // Systems with non-conflicting queries
    pub actual_parallel_executions: usize,  // Systems that ran simultaneously
    pub parallelism_efficiency: f32,        // actual / potential
}
```

### Phase 4: Memory Profiling

**Challenge:** Track component table memory usage.

**Approach:**
- Bevy tracks table memory internally (not exposed)
- Estimate based on archetype sizes and component sizes
- Measure with system allocator hooks (jemalloc stats)

**Metric:**
```rust
pub struct MemoryMetrics {
    pub component_table_bytes: usize,
    pub archetype_metadata_bytes: usize,
    pub total_ecs_memory_bytes: usize,
    pub bytes_per_entity: usize,
}
```

---

## 8. Testing Strategy

### Unit Tests

```rust
#[test]
fn test_collect_ecs_metrics_empty_world() {
    let world = World::new();
    let metrics = collect_ecs_metrics(&world);

    assert_eq!(metrics.entity_count, 0);
    assert_eq!(metrics.archetype_count, 1);  // Empty archetype always exists
    assert_eq!(metrics.cache_efficiency_score, 0.0);
}

#[test]
fn test_collect_ecs_metrics_single_archetype() {
    let mut world = World::new();
    for _ in 0..1000 {
        world.spawn((Position::default(), Velocity::default()));
    }

    let metrics = collect_ecs_metrics(&world);

    assert_eq!(metrics.entity_count, 1000);
    assert_eq!(metrics.archetype_count, 2);  // Empty + (Position, Velocity)
    assert_eq!(metrics.largest_archetype_size, 1000);
    assert_eq!(metrics.top_3_archetype_concentration_pct, 100.0);
    assert!(metrics.cache_efficiency_score > 0.8);
}

#[test]
fn test_collect_ecs_metrics_fragmented_world() {
    let mut world = World::new();

    // Create 10 different archetypes
    for i in 0..10 {
        for _ in 0..100 {
            if i % 2 == 0 {
                world.spawn((Position::default(), Velocity::default()));
            } else {
                world.spawn((Position::default(), Acceleration::default()));
            }
        }
    }

    let metrics = collect_ecs_metrics(&world);

    assert_eq!(metrics.entity_count, 1000);
    assert!(metrics.archetype_count > 2);  // Multiple archetypes
    assert!(metrics.fragmentation_score > 0.3);
    assert!(metrics.cache_efficiency_score < 0.7);
}
```

### Performance Benchmarks

```rust
#[bench]
fn bench_collect_ecs_metrics_100k_entities(b: &mut Bencher) {
    let mut world = World::new();
    for _ in 0..100_000 {
        world.spawn((
            Position::default(),
            Velocity::default(),
            Acceleration::default(),
        ));
    }

    b.iter(|| {
        let metrics = collect_ecs_metrics(&world);
        black_box(metrics);
    });

    // Target: < 500 μs
}
```

---

## Summary

This ECS metrics specification provides **comprehensive visibility into Bevy's Data-Oriented Design patterns** at scale. The key innovation is the **cache efficiency score**, which distills complex archetype fragmentation into a single actionable metric.

**Use this system to:**
1. Validate architectural decisions (e.g., capability marker strategy)
2. Identify performance bottlenecks (fragmentation vs algorithmic complexity)
3. Guide optimization efforts (focus on top 3 archetype concentration)
4. Track regression (cache efficiency score trends over time)

**Next Steps:**
1. Implement `collect_ecs_metrics()` function
2. Add to `SystemTimings` snapshot workflow
3. Extend dev-ui with ECS metrics panel
4. Establish baseline metrics for current 10K entity simulation
5. Monitor metrics during scale-up to 150K-200K entities

---

**Remember:** In Data-Oriented Design, **cache misses are the enemy**. These metrics expose cache behavior. Optimize for tight archetype clustering, and the performance will follow.
