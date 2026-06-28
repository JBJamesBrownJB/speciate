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
│  │  - Bevy ECS (20Hz simulation tick)                          │ │
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

### Payload tiering — keep the hot streaming set minimal (📋 Planned principle)

The per-tick creature buffer is the **load-bearing streaming set**: it is copied/streamed
to the renderer every simulation tick (20 Hz), so its per-creature width directly bounds
how many creatures we can deliver before the delivery cost blows the frame budget. At 1M
creatures, every extra `f32` per creature is **+4 MB/tick (~+80 MB/s)**.

**Rule: the hot buffer carries only what changes every tick and is needed for every
creature on screen** — i.e. *kinematics*: id, position, rotation, size. Nothing else earns
a seat on the hot path.

**Everything else rides a separate, low-frequency, viewport-scoped channel.** Visual/identity
traits that change rarely (or never) — anything that tells the frontend *how to draw* a
creature rather than *where it is* — are delivered on their own infrequent update, and only
for creatures currently in view.

> Example: a creature has horns. The renderer needs to know so it can attach the "horny"
> shader/variant. That fact is static for the creature's life, so it must **not** sit in the
> 20 Hz kinematics buffer. Instead it arrives once (and on change) via the low-freq
> in-view trait channel; the renderer keys it to the creature id and applies the variant.
> The hot buffer stays 5-wide no matter how rich creatures become.

This keeps the streaming set's width constant as creatures gain biological/visual
complexity, so scale (Pillar 1) and spectacle (Pillar 2) don't trade off against each other.
Implementation of the low-freq trait channel is **not yet built** — this records the
contract any such feature must follow.

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
| Simulation tick rate | 20 Hz |
| IPC overhead | <1ms per frame |
| Creature capacity | 1M target (stretch) · 500K Linux validated · ~900K Windows peak run (single session, not yet CI-benchmarked) |
| CPU utilization | 16 cores via Rayon |

## Security Model

Electron's process isolation ensures:
- Renderer process has NO Node.js access
- All simulation access via preload-exposed API
- contextBridge whitelists specific methods only
