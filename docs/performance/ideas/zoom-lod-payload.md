# Zoom LOD Sim Payload

**Status:** Idea
**Category:** Simulation Optimizations

## Problem

We send unnecessary info as we zoom out, such as rotation.

## Solution

Frontend notifies sim when zoom changes and sim reduces payload by removing things like rotation, size, maybe even reduces precision of x,y to just int or something.

## Related

- `todo/viewport-spatial-indexing.md` - viewport culling
- `ideas/lod-rendering.md` - rendering LOD
