# Electron Desktop Architecture

**Status:** Current (Sprint 13+ NAPI-RS)

## Overview

Speciate Phase 1 uses **Electron** to package the simulation as a standalone desktop application. The architecture uses **NAPI-RS zero-copy shared memory** for communication between the Rust simulation and TypeScript/PixiJS frontend.

**Previous architectures:**
- stdio MessagePack streaming (archived in `docs/archive/stdio/`)
- Dual-tick scheduling (archived in `docs/archive/dual-tick/`)

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    Electron Application                          │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              Renderer Process (Isolated)                    │ │
│  │  ┌──────────────────────────────────────────────────────┐  │ │
│  │  │         TypeScript Frontend (PixiJS)                 │  │ │
│  │  │  - WebGL rendering (InterpolatedCreatureRenderer)    │  │ │
│  │  │  - UI/HUD (DOM)                                      │  │ │
│  │  │  - Camera controls + viewport culling                │  │ │
│  │  └──────────────────────────────────────────────────────┘  │ │
│  │                          ▲                                  │ │
│  │                          │ window.electron API              │ │
│  │                          │ (via contextBridge)              │ │
│  │                          ▼                                  │ │
│  │  ┌──────────────────────────────────────────────────────┐  │ │
│  │  │        Preload Script (Security Bridge)              │  │ │
│  │  │  - Exposes safe IPC methods                          │  │ │
│  │  │  - Wraps NAPI buffer access                          │  │ │
│  │  └──────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────┘ │
│                          ▲                                       │
│                          │ IPC Events + Buffer Refs              │
│                          ▼                                       │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              Main Process (NAPI-RS Host)                    │ │
│  │  - Loads simulation-napi native addon                       │ │
│  │  - Runs simulation tick loop                                │ │
│  │  - Zero-copy buffer handoff to renderer                     │ │
│  │  - Viewport bounds from frontend                            │ │
│  └────────────────────────────────────────────────────────────┘ │
│                          │                                       │
│                          │ Direct Function Calls (NAPI-RS)       │
│                          ▼                                       │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              Rust Simulation (Native Addon)                 │ │
│  │  - Bevy ECS (22.2Hz physics tick)                           │ │
│  │  - Rayon parallelization (16 cores)                         │ │
│  │  - Double-buffered position data                            │ │
│  │  - Viewport-culled creature export                          │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Communication: NAPI-RS Zero-Copy

### Why NAPI-RS?

**Advantages:**
- **Zero-copy:** Rust writes directly to buffers TypeScript can read
- **No serialization:** Eliminates MessagePack encode/decode overhead
- **<1ms latency:** Direct function calls, no IPC marshalling

**vs. stdio MessagePack (old):**
- 97% reduction in IPC overhead (30ms → <1ms)
- No JSON/MessagePack parsing in hot path
- Rust memory directly accessible from JS

### Buffer Protocol

**Double-buffered SOA (Struct of Arrays):**
```
Buffer Layout (per creature):
┌────────┬────────┬──────────┬─────────┬─────────┐
│ CritId │ Pos X  │  Pos Y   │ Rot θ   │ Size    │
│ u32    │ f32    │  f32     │ f32     │ f32     │
└────────┴────────┴──────────┴─────────┴─────────┘
```

**Double-buffering:**
- Simulation writes to back buffer
- Frontend reads from front buffer
- Atomic swap between ticks
- No locking required

### Viewport Culling

Frontend sends camera viewport bounds each frame:
- `setViewportBounds(minX, minY, maxX, maxY, margin)`
- Simulation filters creatures to only export those in view
- Reduces IPC payload by 90%+ when zoomed in

## Key Files

| Component | Location |
|-----------|----------|
| NAPI addon | `apps/simulation/src/napi_addon/simulation_engine.rs` |
| Double buffer | `apps/simulation/src/ipc/bridge/double_buffer.rs` |
| Viewport culling | `apps/simulation/src/ipc/bridge/bevy_app.rs:296` |
| Main process | `apps/portal/electron/napi-main.cjs` |
| Preload | `apps/portal/electron/preload.cjs` |
| IPC client | `apps/portal/src/infrastructure/ipc/ElectronIPCClient.ts` |

## Performance Characteristics

| Metric | Value |
|--------|-------|
| Physics tick rate | 22.2 Hz |
| IPC overhead | <1ms per frame |
| Creature capacity | 200K+ (with viewport culling) |
| CPU utilization | 16 cores via Rayon |

## Security Model

Electron's process isolation ensures:
- Renderer process has NO Node.js access
- All simulation access via preload-exposed API
- contextBridge whitelists specific methods only
