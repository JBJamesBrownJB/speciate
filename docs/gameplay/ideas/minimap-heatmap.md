# Minimap Heatmap: Full-World Creature Density

## Current State

The minimap UI is implemented (`apps/portal/src/rendering/minimap/`):
- 180x180px overlay, positioned bottom-right
- Shows viewport rectangle (white border indicating current view)
- Click-to-teleport camera functionality
- Press **M** to toggle visibility

**Limitation**: The heatmap only shows creatures within the current viewport.

## Problem

The backend uses viewport culling for rendering performance:
1. Frontend sends viewport bounds via `setViewportBounds()`
2. Backend filters `export_positions()` to only include visible creatures
3. `latestCreatures` array only contains viewport-culled data
4. Minimap receives culled data, so heatmap only shows current view area

This is correct behavior for the main renderer but breaks minimap orientation.

## Proposed Solution

### Backend Density Grid

Have the Rust backend compute a 50x50 density grid covering the entire world:

```rust
pub struct MinimapDensity {
    grid: [f32; 2500],  // 50x50 normalized density values
}
```

### Computation Approach

**Option A: Modulus Spreading**

Spread density computation over 4 ticks to reduce per-tick overhead:
- Tick N+0: Update rows 0-12 (cells 0-624)
- Tick N+1: Update rows 13-24 (cells 625-1249)
- Tick N+2: Update rows 25-37 (cells 1250-1874)
- Tick N+3: Update rows 38-49 (cells 1875-2499)

Each tick processes ~25K creatures into 625 cells.

**Option B: Piggyback on Existing System**

Integrate with a system that already iterates all creatures:
- Spatial grid rebuild (already processes all positions)
- Movement system (if can add cheap accumulation)

### Data Flow

```
Rust Backend
├── compute_minimap_density() runs every N ticks
├── Accumulate creatures into 50x50 grid
├── Normalize to 0-1 range
└── Include in TelemetrySnapshot.minimap_density

JSON Telemetry (~2Hz)
└── { "minimapDensity": [0.0, 0.1, 0.5, ...] }  // 2500 floats

Frontend
├── ElectronIPCClient.onTelemetryUpdate()
├── Parse minimapDensity array
└── Minimap.updateFromBackendDensity(density)
    └── Render heatmap from pre-computed grid
```

## Performance Budget

| Operation | Cost | Frequency |
|-----------|------|-----------|
| Density accumulation (100K creatures) | ~0.5ms | Every 4 ticks (spread) |
| JSON serialization (2500 floats) | ~0.1ms | Every ~11 ticks |
| Frontend render | ~1ms | On telemetry update |

Total: ~0.5ms amortized per tick, ~10KB additional telemetry payload.

## Integration Points

### Synergy with Upcoming Work

See `better-mod-slice-approach.md` for improved modulus spreading patterns that could benefit this feature.

### Frontend Changes Required

1. Add `minimapDensity?: number[]` to telemetry types
2. Add `Minimap.updateFromBackendDensity(grid: number[])` method
3. Wire telemetry callback to minimap update

### Backend Changes Required

1. Add `minimap_density: Option<Vec<f32>>` to `TelemetrySnapshot`
2. Add density computation system (piggybacked or dedicated)
3. Include in `to_json()` serialization

## Visual Design

- Green gradient: Black (empty) → Dark green → Bright green (dense)
- 50x50 grid = 3.6x3.6 pixel cells at 180px minimap size
- Normalized: max density cell = 1.0, scaled relative

## Deferred

Implementation deferred pending `better-mod-slice-approach.md` work which may provide infrastructure for efficient modulus-spread computation.
