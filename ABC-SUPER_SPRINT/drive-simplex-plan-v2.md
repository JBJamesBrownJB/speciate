# Drive Simplex Implementation Plan v2

**Status:** ⏸️ ON HOLD - Pending Hierarchical Perception v2
**Previous Attempt:** See `drive-simplex-plan-FAILED.md` for lessons learned
**Blocked By:** `hierarchical-perception-v2.md` must complete first - the multi-level perception architecture may change how drives consume L1/L2 data

---

## Goal

Replace discrete BehaviorMode enum with continuous drive-based behavior using L1 cell perception.

---

## Phases

### Phase 1: L1Vision (CURRENT)

Creatures perceive and record L1 cells in their FOV cone.

**Deliverables:**
- [x] L1 grid coordinates confirmed working (hover shows cell info)
- [ ] Rename `L1Perceptions` component to `L1Vision`
- [ ] During L0 scan, check if parent L1 cell is in FOV cone
- [ ] If in FOV, record to `L1Vision` component with classification
- [ ] Portal visualization: gray lines from creature to perceived L1 cells
- [ ] Lines visible when creature selected + L1 grid overlay enabled

**Implementation Details:**
- Reuse existing FOV cone check (`is_in_fov()` from `fov_patterns.rs`)
- L1 cells already being iterated for size domination early-exit
- Just add FOV check and push to L1Vision component
- Max 4 unique L1 cells per creature (3x3 L0 neighborhood spans at most 4 L1 cells)

### Phase 2: L1 Classification Visualization

Color-code L1 vision lines by classification.

**Deliverables:**
- [ ] Color lines by classification (not just gray):
  - Red = Threat (contains larger creature)
  - Orange = Prey (contains smaller creature)
  - Yellow = Crowded (neutral)
  - Green = Empty (safe)
- [ ] Dev overlay shows classification legend

### Phase 3: Drive Simplex

Replace behavior states with continuous drives.

**Deliverables:**
- [ ] Add `DriveState` component
- [ ] L1 drive system computes gradient from L1Vision:
  - Repulsion from Threat cells
  - Attraction to Prey cells (for predators)
  - Attraction to Empty cells (for dispersal)
- [ ] Remove `BehaviorMode` enum
- [ ] Remove wandering system (drive produces emergent wandering)

**See:** `2-simple-drive-simplex.md` for full drive system design.

### Phase 4: Border Repulsion

Use L1 border cells for natural edge avoidance.

**Deliverables:**
- [ ] Flag border L1 cells at world init
- [ ] Border cells auto-classify as "avoid"
- [ ] Predators feel weaker repulsion (Golden Zone: edge hunting)

---

## Architecture

### L1Vision Component

```rust
pub struct L1Vision {
    count: u8,
    entries: [L1VisionEntry; MAX_L1_VISION],  // MAX = 48
}

pub struct L1VisionEntry {
    pub cell_idx: u32,
    pub classification: L1Classification,  // Empty, Threat, Prey, Crowded
    pub direction_x: f32,
    pub direction_y: f32,
}
```

### Data Flow

```
Creature Position + Facing
         │
         ▼
┌─────────────────────┐
│  L0 Perception Scan │
│  (9 adjacent cells) │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Parent L1 Lookup   │  For each L0 cell, get parent L1
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  L1 Classification  │  Empty/Threat/Prey/Crowded
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  FOV Cone Check     │  Is L1 cell center in creature FOV?
└──────────┬──────────┘
           │ (if in FOV)
           ▼
┌─────────────────────┐
│  Push to L1Vision   │  Record cell + classification + direction
└─────────────────────┘
```

### Portal Visualization

When creature selected + L1 grid overlay enabled:
- Draw lines from creature center to L1 cell centers
- Line color indicates classification (gray for Phase 1)
- Similar to neighbor perception lines

---

## Files to Modify

### Rust (simulation)
| File | Change |
|------|--------|
| `perception/components.rs` | Rename L1Perceptions → L1Vision |
| `perception/systems.rs` | Populate L1Vision during L0 scan |
| `perception/mod.rs` | Update exports |
| `napi_addon/` | Expose L1Vision for selected creature |

### TypeScript (portal)
| File | Change |
|------|--------|
| `types/` | Add L1VisionEntry type |
| `electron/preload.ts` | Add IPC handler |
| `rendering/overlays/PerceptionOverlay.ts` | Render L1 vision lines |

---

## Validation Checklist

### Phase 1
- [ ] Select creature + press G twice (L1 mode) → gray lines to L1 cells
- [ ] Lines only to non-Empty cells in FOV cone
- [ ] No lines when grid overlay off
- [ ] No performance regression

### Phase 2
- [ ] Lines colored by classification
- [ ] Colors match legend

### Phase 3
- [ ] Creatures disperse naturally (no explicit wandering)
- [ ] Small creatures avoid areas with large creatures
- [ ] No BehaviorMode enum in codebase

### Phase 4
- [ ] Creatures avoid world borders
- [ ] Predators can approach borders more closely than prey
