# Frontend -> Sim Spatial Indexing Communication

**Status:** Done
**Category:** Simulation Optimizations

## What It Does

Simulation receives camera viewport bounds from frontend and only exports creatures within the visible area (plus margin). This dramatically reduces IPC payload when zoomed in.

## Implementation

**Rust side:**
- `ViewportBounds` resource stores current camera bounds
- `set_viewport_bounds` NAPI function receives bounds from frontend
- Export filter in `bevy_app.rs` culls creatures outside viewport

**Frontend side:**
- Camera sends bounds on every frame via `window.electron.setViewportBounds()`
- Bounds include margin to prevent pop-in at edges
- Only updates when bounds change significantly (>1 world unit)

## Key Files

- `apps/simulation/src/ipc/bridge/bevy_app.rs:25` - ViewportBounds resource
- `apps/simulation/src/ipc/bridge/bevy_app.rs:296-300` - Creature filtering
- `apps/simulation/src/napi_addon/simulation_engine.rs:648` - NAPI function
- `apps/portal/src/main.ts:368` - Frontend viewport sending

## Performance Impact

Only visible creatures are serialized and sent over IPC. At 100K creatures with typical zoom, this can reduce IPC payload by 90%+.

## Related

- `ideas/zoom-lod-payload.md` - Further payload reduction at high zoom
- `ideas/lod-rendering.md` - Reduce rendering complexity at high zoom
