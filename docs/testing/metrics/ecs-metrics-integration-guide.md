# ECS Metrics Integration Guide

**Status:** Design Complete, Implementation Pending
**Sprint:** Sprint 12 - Interpolation & Perception
**Dependencies:** Existing `SystemTimings` instrumentation

---

## Quick Start

This guide shows how to integrate the comprehensive ECS metrics (from `ecs-metrics-specification.md`) into the existing instrumentation system.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    INSTRUMENTATION ARCHITECTURE                  │
├──────────────────────────┬──────────────────────────────────────┤
│  EXISTING METRICS        │  NEW ECS METRICS                     │
│  (System Timings)        │  (Data-Oriented Design)              │
├──────────────────────────┼──────────────────────────────────────┤
│  SystemTimings           │  EcsMetrics                          │
│  ├─ movement_us          │  ├─ archetype_count                  │
│  ├─ perception_us        │  ├─ cache_efficiency_score           │
│  ├─ behavior_us          │  ├─ top_3_archetype_concentration    │
│  ├─ ipc_query_us         │  ├─ fragmentation_score              │
│  └─ ...                  │  └─ ...                              │
│                          │                                      │
│  Collection: Per-system  │  Collection: Once per tick           │
│  Overhead: ~2-5μs/system │  Overhead: Target <500μs             │
│  Purpose: Bottleneck ID  │  Purpose: Cache behavior analysis    │
└──────────────────────────┴──────────────────────────────────────┘
```

---

## Step 1: Add EcsMetrics to GameState

### Modify `apps/simulation/src/ipc/mod.rs`

```rust
use crate::instrumentation::SystemTimingsSnapshot;
use crate::metrics::EcsMetrics;  // NEW

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub protocol_version: u32,
    pub tick: u64,
    pub tick_rate_hz: f32,
    pub creatures: Vec<CreatureSnapshot>,
    pub entity_count: usize,

    // Existing system timings
    pub system_timings_us: SystemTimingsSnapshot,

    // NEW: ECS metrics
    #[cfg(feature = "dev-tools")]
    pub ecs_metrics: Option<EcsMetrics>,
}
```

### Why `Option<EcsMetrics>`?

- **Collection frequency:** ECS metrics are expensive (~500μs). Collect every 10th tick (0.5s intervals) instead of every tick.
- **Feature gate:** Only in `dev-tools` feature (zero cost in production).
- **Graceful degradation:** If collection fails or is skipped, frontend still renders.

---

## Step 2: Create Metrics Collection System

### Create `apps/simulation/src/metrics/mod.rs`

```rust
mod ecs_metrics;

pub use ecs_metrics::{collect_ecs_metrics, EcsMetrics};

use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct MetricsCollectionInterval {
    pub ticks_since_last_collection: u64,
    pub collection_interval: u64,  // Collect every N ticks (default: 10)
}

impl Default for MetricsCollectionInterval {
    fn default() -> Self {
        Self {
            ticks_since_last_collection: 0,
            collection_interval: 10,  // 10 ticks @ 20Hz = 0.5s
        }
    }
}

impl MetricsCollectionInterval {
    pub fn should_collect(&mut self) -> bool {
        self.ticks_since_last_collection += 1;

        if self.ticks_since_last_collection >= self.collection_interval {
            self.ticks_since_last_collection = 0;
            true
        } else {
            false
        }
    }
}
```

### Create `apps/simulation/src/metrics/ecs_metrics.rs`

Copy the complete implementation from `ecs-metrics-specification.md` Section 2.

---

## Step 3: Integrate with Snapshot System

### Modify `apps/simulation/src/stdio/hooks.rs`

```rust
#[cfg(feature = "dev-tools")]
pub fn snapshot_and_write_frame(world: &World) -> std::io::Result<()> {
    use crate::time_system;
    use crate::instrumentation::SystemTimings;

    let timings = world.resource::<SystemTimings>();

    // Existing timing for snapshot
    time_system!(timings, "ipc_query");
    let tick = world.resource::<PhysicsTick>().get();
    let tick_rate = world.resource::<ActualTickRate>().0;
    let creatures = /* ... existing query ... */;

    // NEW: Collect ECS metrics periodically
    let mut metrics_interval = world.resource_mut::<MetricsCollectionInterval>();
    let ecs_metrics = if metrics_interval.should_collect() {
        // Time the collection
        let start = std::time::Instant::now();
        let metrics = crate::metrics::collect_ecs_metrics(world);
        let elapsed = start.elapsed().as_micros() as u64;

        // Log if collection takes too long
        if elapsed > 1000 {  // > 1ms
            log::warn!("ECS metrics collection took {}μs (target: <500μs)", elapsed);
        }

        Some(metrics)
    } else {
        None
    };

    // Serialize GameState with optional ECS metrics
    time_system!(timings, "ipc_serialize");
    let state = GameState {
        protocol_version: 1,
        tick,
        tick_rate_hz: tick_rate,
        creatures,
        entity_count: creatures.len(),
        system_timings_us: timings.snapshot(),
        ecs_metrics,  // NEW
    };

    // ... rest of existing code ...
}
```

---

## Step 4: Update TypeScript Types

### Modify `apps/portal/src/types/GameState.ts`

```typescript
export interface GameState {
  protocolVersion: number;
  tick: number;
  tickRateHz: number;
  creatures: CreatureSnapshot[];
  entityCount: number;
  systemTimingsUs: SystemTimingsSnapshot;
  ecsMetrics?: EcsMetrics;  // NEW: Optional
}

export interface EcsMetrics {
  // Archetype Health
  archetypeCount: number;
  largestArchetypeSize: number;
  emptyArchetypeCount: number;
  totalArchetypeComponents: number;
  avgEntitiesPerArchetype: number;
  emptyArchetypePercentage: number;

  // Entity Distribution
  entityCount: number;
  topArchetypes: [number, number][];  // [(entityCount, componentCount), ...]
  largestArchetypeConcentrationPct: number;
  top3ArchetypeConcentrationPct: number;

  // Component Density
  avgComponentsPerEntity: number;
  uniqueComponentTypes: number;
  hotComponentCoveragePct: number;
  coldComponentCoveragePct: number;

  // System Execution
  systemTimingsUs: SystemTimingsSnapshot;
  totalTickUs: number;

  // Performance Indicators
  cacheEfficiencyScore: number;
  fragmentationScore: number;
  archetypeOverheadBytes: number;
}
```

### Also update `apps/dev-ui/src/types.ts` (same interface)

---

## Step 5: Create ECS Metrics Panel (Dev-UI)

### Create `apps/dev-ui/src/components/EcsMetricsPanel.tsx`

```typescript
import React, { useEffect, useRef } from 'react';
import type { EcsMetrics } from '../types';

interface Props {
  metrics: EcsMetrics | undefined;
}

export function EcsMetricsPanel({ metrics }: Props) {
  const cacheScoreCanvasRef = useRef<HTMLCanvasElement>(null);
  const concentrationCanvasRef = useRef<HTMLCanvasElement>(null);
  const fragmentationCanvasRef = useRef<HTMLCanvasElement>(null);

  // Sparkline history refs
  const cacheScoreHistory = useRef<number[]>([]);
  const concentrationHistory = useRef<number[]>([]);
  const fragmentationHistory = useRef<number[]>([]);

  useEffect(() => {
    if (!metrics) return;

    // Update histories
    cacheScoreHistory.current.push(metrics.cacheEfficiencyScore);
    concentrationHistory.current.push(metrics.top3ArchetypeConcentrationPct);
    fragmentationHistory.current.push(metrics.fragmentationScore);

    // Limit history length (2 minutes @ 0.5s intervals = 240 samples)
    const maxHistory = 240;
    if (cacheScoreHistory.current.length > maxHistory) {
      cacheScoreHistory.current.shift();
      concentrationHistory.current.shift();
      fragmentationHistory.current.shift();
    }

    // Render sparklines
    renderSparkline(cacheScoreCanvasRef.current, cacheScoreHistory.current, 0, 1);
    renderSparkline(concentrationCanvasRef.current, concentrationHistory.current, 0, 100);
    renderSparkline(fragmentationCanvasRef.current, fragmentationHistory.current, 0, 1);
  }, [metrics]);

  if (!metrics) {
    return (
      <div className="panel ecs-metrics-panel">
        <h3>ECS Metrics</h3>
        <div className="no-data">Collecting... (every 0.5s)</div>
      </div>
    );
  }

  // Threshold warnings
  const cacheWarning = metrics.cacheEfficiencyScore < 0.5;
  const concentrationWarning = metrics.top3ArchetypeConcentrationPct < 60;
  const fragmentationWarning = metrics.fragmentationScore > 0.4;

  return (
    <div className="panel ecs-metrics-panel">
      <h3>ECS Metrics (Data-Oriented Design)</h3>

      {/* Performance Indicators */}
      <section className="metrics-section">
        <h4>Performance Indicators</h4>
        <div className={`metric ${cacheWarning ? 'warning' : ''}`}>
          <label>Cache Efficiency</label>
          <canvas ref={cacheScoreCanvasRef} width={100} height={20} />
          <span className="value">{(metrics.cacheEfficiencyScore * 100).toFixed(1)}%</span>
          {cacheWarning && <span className="alert">⚠️ CRITICAL</span>}
        </div>
        <div className={`metric ${concentrationWarning ? 'warning' : ''}`}>
          <label>Top 3 Archetype Concentration</label>
          <canvas ref={concentrationCanvasRef} width={100} height={20} />
          <span className="value">{metrics.top3ArchetypeConcentrationPct.toFixed(1)}%</span>
          {concentrationWarning && <span className="alert">⚠️</span>}
        </div>
        <div className={`metric ${fragmentationWarning ? 'warning' : ''}`}>
          <label>Fragmentation Score</label>
          <canvas ref={fragmentationCanvasRef} width={100} height={20} />
          <span className="value">{(metrics.fragmentationScore * 100).toFixed(1)}%</span>
          {fragmentationWarning && <span className="alert">⚠️</span>}
        </div>
      </section>

      {/* Archetype Health */}
      <section className="metrics-section">
        <h4>Archetype Health</h4>
        <div className="metric">
          <label>Total Archetypes</label>
          <span className="value">{metrics.archetypeCount}</span>
        </div>
        <div className="metric">
          <label>Largest Archetype Size</label>
          <span className="value">{metrics.largestArchetypeSize.toLocaleString()}</span>
        </div>
        <div className="metric">
          <label>Empty Archetypes</label>
          <span className="value">{metrics.emptyArchetypeCount} ({metrics.emptyArchetypePercentage.toFixed(1)}%)</span>
        </div>
        <div className="metric">
          <label>Avg Entities/Archetype</label>
          <span className="value">{metrics.avgEntitiesPerArchetype.toFixed(0)}</span>
        </div>
      </section>

      {/* Top Archetypes Table */}
      <section className="metrics-section">
        <h4>Top 5 Archetypes</h4>
        <table className="archetype-table">
          <thead>
            <tr>
              <th>Entities</th>
              <th>Components</th>
              <th>% of Total</th>
            </tr>
          </thead>
          <tbody>
            {metrics.topArchetypes.map(([entities, components], index) => (
              <tr key={index}>
                <td>{entities.toLocaleString()}</td>
                <td>{components}</td>
                <td>{((entities / metrics.entityCount) * 100).toFixed(1)}%</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      {/* Component Density */}
      <section className="metrics-section">
        <h4>Component Density</h4>
        <div className="metric">
          <label>Avg Components/Entity</label>
          <span className="value">{metrics.avgComponentsPerEntity.toFixed(1)}</span>
        </div>
        <div className="metric">
          <label>Unique Component Types</label>
          <span className="value">{metrics.uniqueComponentTypes}</span>
        </div>
        <div className="metric">
          <label>Hot Component Coverage</label>
          <span className="value">{metrics.hotComponentCoveragePct.toFixed(1)}%</span>
        </div>
      </section>

      {/* Memory Overhead */}
      <section className="metrics-section">
        <h4>Memory</h4>
        <div className="metric">
          <label>Archetype Overhead</label>
          <span className="value">{(metrics.archetypeOverheadBytes / 1024).toFixed(1)} KB</span>
        </div>
      </section>
    </div>
  );
}

function renderSparkline(
  canvas: HTMLCanvasElement | null,
  data: number[],
  min: number,
  max: number
) {
  if (!canvas || data.length === 0) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const width = canvas.width;
  const height = canvas.height;
  const range = max - min;

  ctx.clearRect(0, 0, width, height);
  ctx.strokeStyle = '#00ff00';
  ctx.lineWidth = 1;
  ctx.beginPath();

  data.forEach((value, index) => {
    const x = (index / (data.length - 1)) * width;
    const y = height - ((value - min) / range) * height;

    if (index === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  });

  ctx.stroke();
}
```

### Add to Dev-UI main view

```typescript
// apps/dev-ui/src/App.tsx
import { EcsMetricsPanel } from './components/EcsMetricsPanel';

function App() {
  const [gameState, setGameState] = useState<GameState | null>(null);

  return (
    <div className="app">
      <SystemTimingsPanel timings={gameState?.systemTimingsUs} />
      <EcsMetricsPanel metrics={gameState?.ecsMetrics} />  {/* NEW */}
      {/* ... other panels ... */}
    </div>
  );
}
```

---

## Step 6: Testing

### Unit Test: Metrics Collection

```rust
// apps/simulation/tests/ecs_metrics_test.rs

#[cfg(feature = "dev-tools")]
use speciate::metrics::collect_ecs_metrics;
use speciate::{Simulation, SimulationBuilder, CritBuilder};

#[test]
#[cfg(feature = "dev-tools")]
fn test_ecs_metrics_baseline() {
    let mut sim = SimulationBuilder::new().build();

    // Spawn 1000 entities with uniform archetype
    for _ in 0..1000 {
        let builder = CritBuilder::new()
            .at(0.0, 0.0)
            .with_all_capabilities();
        sim.spawn_crit(builder);
    }

    let metrics = collect_ecs_metrics(sim.world());

    // Baseline assertions
    assert_eq!(metrics.entity_count, 1000);
    assert!(metrics.archetype_count < 10, "Too many archetypes");
    assert!(metrics.cache_efficiency_score > 0.7, "Poor cache efficiency");
    assert!(metrics.top_3_archetype_concentration_pct > 80.0, "Fragmented");
    assert_eq!(metrics.empty_archetype_count, 0, "Empty archetypes exist");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_ecs_metrics_collection_performance() {
    let mut sim = SimulationBuilder::new().build();

    // Spawn 100K entities
    for _ in 0..100_000 {
        let builder = CritBuilder::new()
            .at(0.0, 0.0)
            .with_all_capabilities();
        sim.spawn_crit(builder);
    }

    let start = std::time::Instant::now();
    let _metrics = collect_ecs_metrics(sim.world());
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_micros() < 500,
        "Collection took {}μs (target: <500μs)",
        elapsed.as_micros()
    );
}
```

### Integration Test: GameState Serialization

```rust
#[test]
#[cfg(feature = "dev-tools")]
fn test_gamestate_with_ecs_metrics_serialization() {
    use speciate::ipc::GameState;
    use speciate::metrics::EcsMetrics;

    let state = GameState {
        protocol_version: 1,
        tick: 100,
        tick_rate_hz: 20.0,
        creatures: vec![],
        entity_count: 0,
        system_timings_us: Default::default(),
        ecs_metrics: Some(EcsMetrics {
            archetype_count: 5,
            entity_count: 1000,
            cache_efficiency_score: 0.85,
            // ... other fields ...
            ..Default::default()
        }),
    };

    // Serialize to MessagePack
    let serialized = rmp_serde::to_vec(&state).unwrap();

    // Deserialize
    let deserialized: GameState = rmp_serde::from_slice(&serialized).unwrap();

    assert_eq!(deserialized.tick, 100);
    assert!(deserialized.ecs_metrics.is_some());
    assert_eq!(deserialized.ecs_metrics.unwrap().archetype_count, 5);
}
```

---

## Step 7: Performance Validation

### Measure Collection Overhead

```rust
// Add timing to snapshot_and_write_frame

#[cfg(feature = "dev-tools")]
{
    let metrics_start = std::time::Instant::now();
    let ecs_metrics = collect_ecs_metrics(world);
    let metrics_elapsed = metrics_start.elapsed().as_micros() as u64;

    // Store in SystemTimings for visibility
    timings.ecs_metrics_collection_us.store(metrics_elapsed, Ordering::Relaxed);

    if metrics_elapsed > 1000 {
        log::warn!(
            "ECS metrics collection exceeded budget: {}μs (target: <500μs)",
            metrics_elapsed
        );
    }
}
```

### Add to SystemTimingsSnapshot

```rust
pub struct SystemTimingsSnapshot {
    // ... existing fields ...

    #[cfg(feature = "dev-tools")]
    pub ecs_metrics_collection_us: u64,
}
```

---

## Step 8: Deployment Checklist

- [ ] Implement `collect_ecs_metrics()` in `apps/simulation/src/metrics/ecs_metrics.rs`
- [ ] Add `MetricsCollectionInterval` resource to SimulationBuilder
- [ ] Integrate with `snapshot_and_write_frame()` in `stdio/hooks.rs`
- [ ] Update `GameState` struct to include `ecs_metrics: Option<EcsMetrics>`
- [ ] Update TypeScript types in `portal/src/types/GameState.ts`
- [ ] Update TypeScript types in `dev-ui/src/types.ts`
- [ ] Create `EcsMetricsPanel.tsx` in dev-ui
- [ ] Write unit tests for metrics collection
- [ ] Write integration tests for serialization
- [ ] Benchmark collection overhead (target: <500μs @ 100K entities)
- [ ] Add sparkline rendering for key metrics
- [ ] Document threshold alerts in dev-ui
- [ ] Validate metrics at 10K, 50K, 100K, 150K entity scales

---

## Performance Expectations

### Collection Overhead Budget

| Entity Count | Target Collection Time | Max Overhead % |
|--------------|------------------------|----------------|
| 10K | <100μs | 0.2% |
| 50K | <250μs | 0.5% |
| 100K | <500μs | 1.0% |
| 150K | <750μs | 1.5% |
| 200K | <1000μs | 2.0% |

**Note:** Collection runs every 10 ticks (0.5s), so amortized overhead is 10× lower.

### Memory Overhead

- `EcsMetrics` struct size: ~200 bytes
- History in dev-ui: 240 samples × 200 bytes = 48 KB (negligible)

---

## Future Work

### Phase 2: Per-Archetype Component Breakdown

Currently `top_archetypes` shows `[(entity_count, component_count)]`.

**Enhancement:** Show which components are in each archetype.

```rust
pub struct ArchetypeInfo {
    pub entity_count: usize,
    pub component_names: Vec<&'static str>,  // ["Position", "Velocity", ...]
}

pub top_archetypes: Vec<ArchetypeInfo>;
```

**Challenge:** Bevy's `ComponentId` → `&'static str` requires component registration metadata.

### Phase 3: Real-Time Archetype Alerts

**Feature:** Send IPC event when archetype health degrades.

```rust
pub enum MetricsAlert {
    FragmentationCritical { score: f32 },
    ConcentrationLow { pct: f32 },
    EmptyArchetypesHigh { count: usize },
}
```

**Use case:** Trigger investigation during development when cache efficiency drops.

---

## Summary

This integration extends the existing `SystemTimings` instrumentation with comprehensive ECS metrics that expose Data-Oriented Design patterns. The key benefits:

1. **Cache visibility:** `cache_efficiency_score` tracks data layout quality
2. **Fragmentation detection:** `top_3_archetype_concentration_pct` identifies archetype churn
3. **Memory waste tracking:** `empty_archetype_count` catches lifecycle bugs
4. **Performance validation:** Metrics guide optimization decisions (not just gut feel)

**Next:** Implement the specification and establish baseline metrics for the current 10K entity simulation.
