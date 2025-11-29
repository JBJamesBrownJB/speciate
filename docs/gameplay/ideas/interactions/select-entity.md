# Entity Selection Design

## Core Approach

Single centralized click handler + backend spatial query + selective data transmission.

**Frontend:**
- Single event handler at canvas level (not per-entity)
- PixiJS handles hover for instant local feedback
- On user click commit, send position to simulator via IPC

**Backend:**
- Add spatial query API to collision system:
  ```rust
  pub fn query_point(&self, position: Vec2) -> Option<EntityId>
  pub fn query_aabb(&self, rect: Rect) -> Vec<EntityId>
  ```
- Use existing spatial partitioning (grid/quadtree) for O(log n) lookup
- Return entity ID + verbose metadata only for selected entity

**Benefits:**
- Reuses existing collision detection infrastructure
- Single event handler vs. thousands (massive memory savings)
- Minimal IPC overhead (only transmits selected entity data)
- Server-authoritative (simulator owns truth about entity positions)
- Extends cleanly to drag-select (query AABB instead of point)
- Foundation for Phase 2 MMO architecture

## Alternative Considered

Injecting click as temporary entity into collision system was considered but rejected:
- Spatial query API is cleaner and more explicit
- Avoids entity lifecycle overhead for each click
- Collision system can optimize queries differently than collision detection

## Implementation Notes

Future sprint will implement:
1. Spatial query methods in collision system
2. IPC message types for selection requests/responses
3. Frontend hover preview (PixiJS-local)
4. Selection state management in UI