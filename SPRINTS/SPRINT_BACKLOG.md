# Sprint Backlog

## Sprint: Basic Terrain (L0-Aligned Obstacles)

**Branch:** `basic-terrain`
**Status:** In Progress
**Started:** 2026-01-18

### Sprint Goal
Add impassable terrain obstacles aligned to L0 cells (20m). Creatures perceive and avoid obstacles independently from other creatures using dual-cache architecture. Minimal performance impact by leveraging existing spatial grid architecture.

### Key Outcomes
1. Creatures stop at blocked cells with no clipping
2. Creatures steer around obstacles using separate ObstacleCache (4 slots)
3. Terrain rendered as cliff sprites in portal
4. No performance regression at 20K creatures

### Key Constraints
- Must align with L0 cell size (20m)
- Bitmap storage only (~8KB)
- Separate caches for obstacles vs creatures
- TDD mandatory (tests before implementation)

### Architecture
- **TerrainGrid Resource:** Bitmap storage (250×250 cells)
- **ObstacleCache Component:** 4-slot cache, updated on cell change
- **Movement blocking:** Check terrain before position update
- **Avoidance:** Distance-based repulsion for obstacles

---

## Task Breakdown

### Phase 1: Core Implementation

#### 1. TerrainGrid Resource
- [ ] Create `terrain/mod.rs` with TerrainGrid struct
- [ ] Implement bitmap storage (BitVec or [u64; 977])
- [ ] Add world↔cell conversion methods
- [ ] Add `is_blocked()` methods
- [ ] Register as Bevy resource
- [ ] Write unit tests for coordinate conversion

#### 2. ObstacleCache Component
- [ ] Create `terrain/components.rs`
- [ ] Implement ObstacleCache (4 slots)
- [ ] Implement PerceivedObstacle struct
- [ ] Add `last_cell` field for early-exit
- [ ] Add to CritBundle
- [ ] Write unit tests

#### 3. Movement Blocking
- [ ] Modify `movement/systems.rs` to check terrain
- [ ] Implement cell-edge clamping
- [ ] Zero velocity into obstacles
- [ ] Write tests: creature stops at blocked cell

#### 4. Obstacle Perception System
- [ ] Create `terrain/systems.rs`
- [ ] Implement `update_obstacle_cache_system`
- [ ] Add early-exit optimization (check last_cell)
- [ ] Scan 3×3 neighborhood
- [ ] Register system before avoidance
- [ ] Write tests: cache updates on cell change

#### 5. Avoidance: Dual Cache
- [ ] Modify `avoidance.rs` to accept ObstacleCache
- [ ] Implement `calculate_obstacle_repulsion()`
- [ ] Add obstacle avoidance constants
- [ ] Keep existing TTC logic for creatures
- [ ] Write tests: steering around obstacles
- [ ] Write tests: both caches work simultaneously

#### 6. IPC: Terrain Data
- [ ] Add terrain buffer to NAPI interface
- [ ] Send blocked cells on init
- [ ] Add TypeScript types

#### 7. Portal Rendering
- [ ] Create TerrainLayer.ts
- [ ] Implement cliff sprite pool
- [ ] Render blocked cells as sprites
- [ ] Add viewport culling

#### 8. Integration & Testing
- [ ] Create test world with hardcoded obstacles
- [ ] Manual testing: navigation around obstacles
- [ ] Test: cache independence
- [ ] Performance benchmark: 20K creatures + 1000 obstacles
- [ ] Verify no clipping

---

## Acceptance Criteria

- [ ] Creatures stop at blocked cells (no clipping)
- [ ] Creatures steer around obstacles before collision
- [ ] Obstacle awareness doesn't crowd out creature awareness (separate caches)
- [ ] Blocked cells render as cliff sprites
- [ ] No performance regression (<5% FPS drop at 20K creatures)
- [ ] All tests pass

---

## Notes

**Design Docs:**
- `SPRINTS/SPRINT_BASIC_TERRAIN/PLAN.md` - High-level design
- `SPRINTS/SPRINT_BASIC_TERRAIN/TECHNICAL_NOTES.md` - Implementation details
- `SPRINTS/SPRINT_BASIC_TERRAIN/CHECKLIST.md` - Task checklist

**Future Phases:**
- Phase 2: Terrain types (Blocked/Slow/Dangerous)
- Phase 3: Cellular automata terrain

**Documentation to Create:**
- `docs/biology/done/terrain-avoidance.md` (after sprint complete)
