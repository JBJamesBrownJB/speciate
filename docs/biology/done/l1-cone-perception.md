# L1 Cone-Based Perception

**Status:** Implemented (ABC Super Sprint - Phases 5+6)
**Location:** `apps/simulation/src/simulation/perception/systems.rs`

---

## What It Does

L1 cells (60m) are scanned using the creature's actual FOV cone, not a fixed ring pattern. Only cells whose centers fall within both the perception range AND the FOV angle are perceived.

Key changes from previous implementation:
- Variable cell count based on perception range and FOV
- Respects facing direction (no cells behind creature)
- Gray L1 visualization matches teal FOV cone exactly

---

## Why It Exists

**Visual consistency:** The L1 cell overlay now matches the actual FOV cone. What you see (gray cells) matches what the creature perceives.

**Biological accuracy:** A creature facing east with 90deg FOV should not perceive L1 cells directly behind it. The cone-based approach ensures perception matches biological expectation.

**Scalable perception:** Large creatures with long perception ranges automatically scan more L1 cells. Small creatures with short ranges may scan none.

---

## Algorithm

```
if perception_range >= L1_CELL_SIZE (60m):
    max_cell_dist = ceil(perception_range / L1_CELL_SIZE)

    for each L1 cell within max_cell_dist:
        skip own cell (center)

        compute direction from creature to cell center
        if distance > perception_range: skip
        if not in FOV (via is_in_fov): skip

        classify cell and add to L1Vision
```

---

## Key Behaviors

| Creature Type | Perception Range | FOV | L1 Cells Perceived |
|---------------|-----------------|-----|-------------------|
| Small (myopic) | < 60m | Any | 0 (range too short) |
| Medium | ~120m | 90deg | 2-4 cells in front |
| Large (predator) | ~200m | 120deg | 6-10 cells forward arc |
| Large (prey) | ~200m | 270deg | 10-15 cells wide arc |

---

## FOV Gating

L1 perception uses the same `is_in_fov()` function as L0 entity perception:
- Computes dot product of direction and facing
- Compares against `cos_half_fov` threshold
- No sqrt required (uses squared comparison)

**Location:** `perception/systems.rs:507-562`

---

## Integration

- Runs after L0 entity scan, before behavior systems
- Populates `L1Vision` component with perceived cells
- Portal visualization reads `L1Vision` to draw gray cell overlay
- Classification (EMPTY/THREAT/PREY/CROWDED) drives future drive systems
