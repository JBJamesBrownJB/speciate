# Minimap

**Status:** ✅ Implemented
**Location:** `apps/portal/src/rendering/minimap/`

## What It Does

A 180x180px overlay showing the player's current viewport position within the world.

## Features

- **Viewport indicator:** White border rectangle showing current camera view
- **Click-to-teleport:** Click anywhere on minimap to move camera
- **Toggle visibility:** Press **M** to show/hide
- **Position:** Bottom-right corner of screen

## Implementation

- `Minimap.ts` - Main minimap component
- Updates based on camera position and zoom level
- Viewport rectangle scales with zoom

## Future Work

See `docs/gameplay/ideas/minimap-density-heatmap.md` for planned creature density visualization.
