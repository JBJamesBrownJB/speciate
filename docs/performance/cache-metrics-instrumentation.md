# Cache-Aware Metrics Instrumentation

**Purpose:** Expose spatial grid performance metrics to Dev UI for real-time cache behavior monitoring.

---

## Metrics to Track

### 1. Spatial Grid Efficiency

```rust
// In perception system, after query
let grid_metrics = GridMetrics {
    total_cells_queried: cells_queried_count,
    occupied_cells_hit: cells_with_entities,
    total_candidates_examined: candidate_count,
    avg_entities_per_cell: candidates / occupied_cells,
    spatial_efficiency: occupied_cells / total_cells,  // Hit rate
};
```

**Thresholds:**
- `spatial_efficiency < 0.3` → Grid too fine (wasting lookups)
- `avg_entities_per_cell > 50` → Grid too coarse (back to O(N²))

### 2. Cache-Predictive Counters

```rust
// Estimate cache behavior
let estimated_l3_misses = cells_queried_count * 0.7;  // Assume 70% HashMap miss rate
let estimated_cache_cycles = estimated_l3_misses * 80;  // L3 miss = ~80 cycles

let cache_pressure = CachePressure {
    estimated_l3_misses_per_query: estimated_l3_misses / entity_count,
    estimated_cache_stall_cycles: estimated_cache_cycles,
    pressure_level: if estimated_cache_cycles > 10000 { "HIGH" } else { "LOW" },
};
```

### 3. Query Pattern Analysis

```rust
// Track distribution of entities across cells
let cell_occupancy_histogram = vec![0; 10];  // Buckets: 0, 1-5, 6-10, 11-20, ...
for cell in grid.cells.values() {
    let bucket = match cell.len() {
        0 => 0,
        1..=5 => 1,
        6..=10 => 2,
        // ...
    };
    cell_occupancy_histogram[bucket] += 1;
}
```

**Analysis:** If most cells are in bucket 0 (empty), grid is too sparse.

---

## Implementation Plan

### Step 1: Add Metrics Struct

File: `/home/dev/dev/speciate/apps/simulation/src/simulation/spatial/metrics.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialGridMetrics {
    pub total_cells_in_grid: usize,
    pub cells_queried_this_tick: usize,
    pub occupied_cells_hit: usize,
    pub total_candidates_examined: usize,
    pub avg_entities_per_cell: f32,
    pub spatial_efficiency: f32,  // occupied / queried
    pub estimated_cache_misses_per_query: f32,
}

impl SpatialGridMetrics {
    pub fn new() -> Self {
        Self {
            total_cells_in_grid: 0,
            cells_queried_this_tick: 0,
            occupied_cells_hit: 0,
            total_candidates_examined: 0,
            avg_entities_per_cell: 0.0,
            spatial_efficiency: 0.0,
            estimated_cache_misses_per_query: 0.0,
        }
    }

    pub fn cache_pressure_level(&self) -> &'static str {
        if self.estimated_cache_misses_per_query > 10.0 {
            "CRITICAL"
        } else if self.estimated_cache_misses_per_query > 5.0 {
            "HIGH"
        } else {
            "NORMAL"
        }
    }
}
```

### Step 2: Instrument Spatial Grid

File: `/home/dev/dev/speciate/apps/simulation/src/simulation/spatial/grid.rs`

Add tracking fields:

```rust
pub struct SpatialGrid {
    cells: FxHashMap<(i32, i32), Vec<EntityData>>,
    cell_size: f32,
    inv_cell_size: f32,

    // Metrics (dev-tools only)
    #[cfg(feature = "dev-tools")]
    pub last_query_stats: std::sync::Arc<std::sync::Mutex<QueryStats>>,
}

#[cfg(feature = "dev-tools")]
#[derive(Default)]
pub struct QueryStats {
    pub cells_queried: usize,
    pub occupied_hits: usize,
    pub candidates_examined: usize,
}
```

Instrument `query_radius`:

```rust
pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> impl Iterator<Item = &EntityData> {
    let (center_cx, center_cy) = self.world_to_cell(x, y);
    let cells_radius = (radius * self.inv_cell_size).ceil() as i32;

    #[cfg(feature = "dev-tools")]
    let stats = self.last_query_stats.clone();

    (-cells_radius..=cells_radius)
        .flat_map(move |dy| {
            (-cells_radius..=cells_radius).map(move |dx| (center_cx + dx, center_cy + dy))
        })
        .filter_map(move |cell_key| {
            #[cfg(feature = "dev-tools")]
            {
                if let Ok(mut s) = stats.lock() {
                    s.cells_queried += 1;
                }
            }

            let cell = self.cells.get(&cell_key);

            #[cfg(feature = "dev-tools")]
            if cell.is_some() {
                if let Ok(mut s) = stats.lock() {
                    s.occupied_hits += 1;
                }
            }

            cell
        })
        .flatten()
        .inspect(move |_| {
            #[cfg(feature = "dev-tools")]
            if let Ok(mut s) = stats.lock() {
                s.candidates_examined += 1;
            }
        })
}
```

### Step 3: Emit Metrics from Perception System

File: `/home/dev/dev/speciate/apps/simulation/src/simulation/perception/systems.rs`

After perception loop:

```rust
#[cfg(feature = "dev-tools")]
{
    let stats = grid.last_query_stats.lock().unwrap();
    let entity_count = inputs.len() as f32;

    let metrics = SpatialGridMetrics {
        total_cells_in_grid: grid.cells.len(),
        cells_queried_this_tick: stats.cells_queried,
        occupied_cells_hit: stats.occupied_hits,
        total_candidates_examined: stats.candidates_examined,
        avg_entities_per_cell: if stats.occupied_hits > 0 {
            stats.candidates_examined as f32 / stats.occupied_hits as f32
        } else {
            0.0
        },
        spatial_efficiency: if stats.cells_queried > 0 {
            stats.occupied_hits as f32 / stats.cells_queried as f32
        } else {
            0.0
        },
        estimated_cache_misses_per_query: (stats.cells_queried as f32 / entity_count) * 0.7,
    };

    // Emit to IPC (double-buffer telemetry)
    // This will be picked up by Dev UI automatically
    let json = serde_json::json!({
        "type": "spatial_grid_metrics",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        "data": metrics
    });

    // Assuming telemetry bridge exists
    if let Some(mut bridge) = telemetry_buffer.lock().ok() {
        bridge.push_metric(json);
    }
}
```

### Step 4: Dev UI Visualization

File: `/home/dev/dev/speciate/apps/dev-ui/src/components/SpatialGridMetrics.tsx`

```tsx
import React from 'react';

interface SpatialGridData {
  total_cells_in_grid: number;
  cells_queried_this_tick: number;
  occupied_cells_hit: number;
  total_candidates_examined: number;
  avg_entities_per_cell: number;
  spatial_efficiency: number;
  estimated_cache_misses_per_query: number;
}

export const SpatialGridMetrics: React.FC<{ data: SpatialGridData | null }> = ({ data }) => {
  if (!data) return <div>No spatial grid data</div>;

  const pressureLevel =
    data.estimated_cache_misses_per_query > 10 ? 'critical' :
    data.estimated_cache_misses_per_query > 5 ? 'warning' : 'normal';

  return (
    <div className="spatial-grid-metrics">
      <h3>Spatial Grid Performance</h3>

      <div className="metric-row">
        <span>Total Cells:</span>
        <span>{data.total_cells_in_grid}</span>
      </div>

      <div className="metric-row">
        <span>Spatial Efficiency:</span>
        <span className={data.spatial_efficiency < 0.3 ? 'warning' : ''}>
          {(data.spatial_efficiency * 100).toFixed(1)}%
        </span>
      </div>

      <div className="metric-row">
        <span>Avg Entities/Cell:</span>
        <span className={data.avg_entities_per_cell > 50 ? 'warning' : ''}>
          {data.avg_entities_per_cell.toFixed(1)}
        </span>
      </div>

      <div className={`metric-row cache-pressure ${pressureLevel}`}>
        <span>Cache Pressure:</span>
        <span>
          {data.estimated_cache_misses_per_query.toFixed(1)} misses/query
          {pressureLevel !== 'normal' && ' ⚠️'}
        </span>
      </div>

      {data.spatial_efficiency < 0.3 && (
        <div className="alert">
          ⚠️ Low efficiency - grid too fine (wasting HashMap lookups)
        </div>
      )}

      {data.avg_entities_per_cell > 50 && (
        <div className="alert">
          ⚠️ High density - grid too coarse (degenerating to O(N²))
        </div>
      )}
    </div>
  );
};
```

CSS:

```css
.spatial-grid-metrics {
  padding: 10px;
  border: 1px solid #ccc;
  border-radius: 4px;
  margin: 10px 0;
}

.metric-row {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
}

.metric-row.warning {
  color: #ff9800;
  font-weight: bold;
}

.cache-pressure.critical {
  background-color: #f44336;
  color: white;
  padding: 4px;
  border-radius: 2px;
}

.cache-pressure.warning {
  background-color: #ff9800;
  color: white;
  padding: 4px;
  border-radius: 2px;
}

.alert {
  margin-top: 8px;
  padding: 8px;
  background-color: #fff3cd;
  border: 1px solid #ffc107;
  border-radius: 4px;
  font-size: 0.9em;
}
```

---

## Usage Workflow

1. **Build with dev-tools:** `cargo build --release --features dev-tools`
2. **Run with Dev UI:** `cd apps/portal && npm run dev`
3. **Monitor metrics in real-time** as creature count scales
4. **Watch for alerts:**
   - Low spatial efficiency → Increase cell size
   - High avg entities/cell → Decrease cell size
   - Critical cache pressure → Consider flat 2D array

---

## Expected Baselines

### HashMap Grid (Current)
- Spatial Efficiency: 40-60% (many empty cells queried)
- Cache Misses/Query: 6-9 (9 cells × 70% miss rate)
- Pressure Level: HIGH

### Flat 2D Array (Proposed)
- Spatial Efficiency: N/A (always hits, no HashMap lookups)
- Cache Misses/Query: 0.5-1.5 (sequential access)
- Pressure Level: NORMAL

### Brute Force (Baseline)
- Spatial Efficiency: N/A
- Cache Misses/Query: 0.2-0.5 (L1 hits)
- Pressure Level: NORMAL (but O(N²) complexity)

---

**Document Owner:** cache-carl (Performance Analyst)
**Last Updated:** 2025-12-04
