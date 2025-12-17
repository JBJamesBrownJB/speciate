# Frontend -> Sim Spatial Indexing Communication

**Status:** TODO (Sprint 16)
**Category:** Simulation Optimizations

## Problem

We send and render all creatures, even if they are not in view of camera.

## Solution

Sim receives camera viewbox in world coordinates and only sends data for creatures within view.

## Expected Benefit

10K creatures: 10ms -> 1ms. Required for 100K+ scale.

## Related

- `todo/viewport_culling.md` - rough notes on viewport culling
- `ideas/zoom-lod-payload.md` - reduce payload at high zoom
- `ideas/lod-rendering.md` - reduce rendering at high zoom
