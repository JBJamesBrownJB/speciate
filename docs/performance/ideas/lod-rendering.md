# LOD Rendering

**Status:** Idea
**Category:** Rendering Optimizations

## Problem

Full sprite detail wasted when zoomed out.

## Solution

Switch to point sprites at far zoom (< 5 px/m).

## Expected Benefit

Reduces GPU memory 30%.

## Notes

Pairs well with spatial indexing.

## Related

- `todo/viewport-spatial-indexing.md` - viewport culling
- `ideas/zoom-lod-payload.md` - payload LOD
