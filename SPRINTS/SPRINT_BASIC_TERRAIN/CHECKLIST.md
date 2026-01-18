# Implementation Checklist: Basic Terrain

## Pre-Implementation

- [ ] Review plan with architect-andy (optional)
- [ ] Consult zoologist-tom on obstacle avoidance behavior
- [ ] Find/create cliff sprite asset

## Phase 1: Core Implementation

### 1. TerrainGrid Resource
- [ ] Create `apps/simulation/src/simulation/terrain/mod.rs`
- [ ] Implement `TerrainGrid` struct with bitmap
- [ ] Add world↔cell conversion methods
- [ ] Add `is_blocked(world_x, world_y)` method
- [ ] Add `is_blocked_cell(cell_x, cell_y)` method
- [ ] Register as Bevy resource
- [ ] Add unit tests for coordinate conversion

### 2. ObstacleCache Component (NEW)
- [ ] Create `apps/simulation/src/simulation/terrain/components.rs`
- [ ] Implement `ObstacleCache` struct (4 slots)
- [ ] Implement `PerceivedObstacle` struct
- [ ] Add `last_cell` field for early-exit optimization
- [ ] Add to CritBundle (spawn with creatures)
- [ ] Add unit tests for cache structure

### 3. Movement Blocking
- [ ] Modify `movement/systems.rs` to check terrain
- [ ] Implement cell-edge clamping when blocked
- [ ] Zero velocity into obstacle
- [ ] Add tests: creature stops at blocked cell

### 4. Obstacle Perception System (NEW)
- [ ] Create `apps/simulation/src/simulation/terrain/systems.rs`
- [ ] Implement `update_obstacle_cache_system`
- [ ] Add early-exit when still in same cell
- [ ] Scan 3×3 neighborhood for blocked cells
- [ ] Register system in schedule (before avoidance)
- [ ] Add tests: cache updates on cell change

### 5. Avoidance: Dual Cache Handling
- [ ] Modify `avoidance.rs` to accept `ObstacleCache`
- [ ] Implement `calculate_obstacle_repulsion()` function
- [ ] Add obstacle avoidance constants
- [ ] Keep existing TTC logic for creatures
- [ ] Add tests: creature steers around obstacles
- [ ] Add tests: both caches work simultaneously

### 6. IPC: Terrain Data
- [ ] Add terrain buffer to NAPI interface
- [ ] Send blocked cells on simulation init
- [ ] Add TypeScript types for terrain data

### 7. Portal: Terrain Rendering
- [ ] Create `TerrainLayer.ts`
- [ ] Implement cliff sprite pool
- [ ] Render blocked cells as cliff sprites
- [ ] Add viewport culling (only visible cells)

### 8. Integration & Testing
- [ ] Create test world with hardcoded obstacles
- [ ] Manual testing: navigation around obstacles
- [ ] Test: obstacle avoidance separate from creature awareness
- [ ] Performance benchmark: 20K creatures + 1000 obstacles
- [ ] Verify no creature clips through obstacles

## Acceptance Criteria

- [ ] Creatures stop at blocked cells (no clipping)
- [ ] Creatures steer around obstacles before collision
- [ ] Obstacle awareness doesn't crowd out creature awareness (separate caches)
- [ ] Blocked cells render as cliff sprites
- [ ] No performance regression (<5% FPS drop at 20K creatures)
- [ ] All tests pass

## Post-Implementation

- [ ] Update `docs/biology/done/` with terrain avoidance doc
- [ ] Update architecture docs if needed
- [ ] Close sprint, merge to main

---

## Quick Reference

| Constant | Value | Notes |
|----------|-------|-------|
| World size | 5000m × 5000m | ±2500m from origin |
| L0 cell size | 20m | From `spatial/constants.rs` |
| Cells per axis | 250 | 5000 / 20 |
| Total cells | 62,500 | 250 × 250 |
| Bitmap size | ~8KB | 62,500 bits |
| Cell half-diagonal | 14.14m | For obstacle radius |
| MAX_PERCEIVED_OBSTACLES | 4 | Cardinal directions |
| NeighborCache slots | 7 | Unchanged (creatures only) |

---

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Separate caches | Yes | Obstacles shouldn't crowd out creature awareness |
| ObstacleCache slots | 4 | Cardinal directions sufficient |
| Update trigger | Cell boundary | Much cheaper than per-tick |
| Avoidance algorithm | Distance-based | TTC unnecessary for static obstacles |
| Obstacle strength | Higher than creatures | Obstacles don't yield |
