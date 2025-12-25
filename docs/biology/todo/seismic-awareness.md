# Seismic Awareness: Giant Detection Beyond Visual Range

**Status:** Proposed
**Golden Zone:** Yes (performance optimization IS the biological feature)
**Priority:** Medium (blocks correct perception of large creatures)

## Problem

The perception system uses a fixed L0 query radius (`L0_SCAN_RADIUS = 20m`), which only searches a 3×3 cell neighborhood. Large creatures whose **center** is beyond this radius are never queried, even if their **body** extends into perceivable range.

**Example:**
- Perceiver at origin with `range = 30m`, `self_radius = 1m`
- Giant at (50, 0) with `radius = 25m` (50m body length)
- Surface-to-surface distance: `50 - 25 - 1 = 24m` (within 30m range!)
- But giant's center is at 50m > 20m query radius → **never detected**

## Biological Justification

In nature, animals detect large creatures through multiple channels beyond direct vision:

1. **Ground vibration** - Elephants communicate via infrasound through the ground; prey animals detect approaching megafauna through seismic signals
2. **Scent plumes** - Large animals produce proportionally larger scent signatures
3. **Visual silhouettes** - Giants are visible above horizon/obstacles
4. **Acoustic presence** - Breathing, movement sounds scale with body size

This justifies detecting giants at extended range without fine-grained visual perception.

## Golden Zone Design

**The optimization IS the biological feature:**

| Optimization | Biological Behavior |
|--------------|---------------------|
| Only scan non-empty L1 cells | Don't waste cognition on empty space |
| Skip cells with small `max_size` | Small creatures don't trigger seismic response |
| Distance cutoff (`SEISMIC_RANGE`) | Vibrations attenuate with distance |
| Expand query only when body could reach | Detect presence, not precise location |

## Algorithm

```
┌─────────────────────────────────────────────────────────────────┐
│                    PERCEPTION QUERY FLOW                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. STANDARD L0 QUERY (existing)                                │
│     ┌───┬───┬───┐                                               │
│     │ L0│ L0│ L0│  3×3 L0 cells (20m each = 60m diagonal max)   │
│     ├───┼───┼───┤                                               │
│     │ L0│ ME│ L0│  Always queries these 9 cells                 │
│     ├───┼───┼───┤                                               │
│     │ L0│ L0│ L0│                                               │
│     └───┴───┴───┘                                               │
│                                                                 │
│  2. SEISMIC SCAN (new) - Only when giants exist                 │
│     ┌─────────┬─────────┬─────────┐                             │
│     │   L1    │   L1    │   L1    │  Scan L1 cells (60m each)   │
│     │max=0.5m │max=0.3m │max=4.0m │◄─ GIANT DETECTED!           │
│     ├─────────┼─────────┼─────────┤                             │
│     │   L1    │ [3×3 L0]│   L1    │  Already covered by L0      │
│     │max=0.2m │  query  │max=0.1m │                             │
│     ├─────────┼─────────┼─────────┤                             │
│     │   L1    │   L1    │   L1    │                             │
│     │max=0.0  │max=0.8m │max=0.0  │                             │
│     └─────────┴─────────┴─────────┘                             │
│                                                                 │
│  3. EXPAND QUERY                                                │
│     For L1 cells with max_size > GIANT_THRESHOLD:               │
│     - Check if giant's body could reach us                      │
│     - If yes, query that L1 cell's 3×3 L0 children              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Sketch

```rust
// Constants
const GIANT_THRESHOLD: f32 = 2.5;  // 5m body = megafauna (radius 2.5m)
const SEISMIC_RANGE: f32 = 120.0;  // How far seismic awareness reaches

fn seismic_giant_scan(
    my_pos: (f32, f32),
    my_radius: f32,
    perception_range: f32,
    l1_grid: &CoarseGrid,
) -> Vec<usize> {  // Returns L0 cell indices to also query
    let mut extra_l0_cells = Vec::new();

    // Iterate non-empty L1 cells (cheap - only cells with creatures)
    for (l1_cx, l1_cy, biosig) in l1_grid.non_empty_cells_with_data() {
        // Skip if no giant in this cell
        if biosig.max_size <= GIANT_THRESHOLD {
            continue;
        }

        // Calculate distance to L1 cell center
        let l1_center = l1_cell_center(l1_cx, l1_cy);
        let dist_to_center = distance(my_pos, l1_center);

        // Skip if beyond seismic range
        if dist_to_center > SEISMIC_RANGE {
            continue;
        }

        // Could the giant's body reach our perception range?
        // Conservative: assume giant is at closest corner of L1 cell
        let l1_half_diag = L1_CELL_SIZE * 0.7071;
        let min_center_dist = (dist_to_center - l1_half_diag).max(0.0);
        let surface_dist = min_center_dist - biosig.max_size - my_radius;

        if surface_dist <= perception_range {
            // Giant could be visible! Get L0 children of this L1 cell
            for l0_idx in l1_to_l0_children(l1_cx, l1_cy) {
                extra_l0_cells.push(l0_idx);
            }
        }
    }

    extra_l0_cells
}
```

## Integration Point

In `apps/simulation/src/simulation/perception/systems.rs`, after the existing L0 query loop (around line 379), add seismic scan:

```rust
// SEISMIC AWARENESS: Detect giants beyond L0 range
// Only runs when giants exist - zero cost otherwise
let extra_cells = seismic_giant_scan(...);
for l0_idx in extra_cells {
    // Process entities in this cell (same logic as main loop)
    for proxy in grid_ref.get_cell_proxies(l0_idx) {
        // ... existing neighbor evaluation code ...
    }
}
```

## Cost Analysis

| Scenario | Extra Work |
|----------|------------|
| No giants in world | Zero (skip all L1 cells with small max_size) |
| Giants far away | Zero (distance check fails) |
| Giant nearby | +9 L0 cells per giant L1 cell |

**Typical case:** 100K creatures, 10 giants → 10 L1 cells checked, maybe 1-2 expanded = negligible cost.

## Secondary Issue: FOV Center-Based Check

The FOV check also uses center-to-center distance, not accounting for target radius. A giant at the edge of the FOV cone might have its center outside the cone but part of its body inside.

This is lower priority since:
1. FOV has a 15° safety margin already
2. Giants are typically detected from further away (seismic) before FOV matters
3. Fix would require circle-cone intersection test (more complex)

## Combine with gameplay features

- Screen shake when they are near
- Thumping sound
- Chance to 'panic' crits..

## Prerequisites

- L1 grid already tracks `max_size` per cell via `BioSignature`
- `non_empty_cells_with_data()` iterator exists for efficient L1 scanning
- Need to add: `l1_to_l0_children()` helper to get L0 cell indices from L1 coords

## Testing Strategy

1. **Unit test:** Giant at 50m with 25m radius detected by creature with 30m perception
2. **Unit test:** Giant at 100m with 10m radius NOT detected (beyond seismic range)
3. **Unit test:** Small creature at 50m NOT detected (below giant threshold)
4. **Integration test:** Visual confirmation in dev-ui that large creatures trigger neighbor detection

## Consulted

- zoologist-tom (2025-12-25): Confirmed size range (0.1m-10m) is realistic, 10m creatures are valid megafauna scale
