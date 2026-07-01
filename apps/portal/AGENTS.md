# apps/portal — Area Guide (PixiJS Frontend + Electron Host)

**`@simulation/portal` is the PLAYER-FACING game client.** PixiJS-rendered world + minimal HUD, hosted in Electron, consuming the simulation over a zero-copy NAPI `Float32Array` seam. This app is also the Electron host (`electron/` lives here).

This file holds area-specific rules. **See `/AGENTS.md` for global guardrails** (TDD, DNA-driven design, doc taxonomy, the Rust↔JS thesis). Closest-file-wins: rules here override the root for this directory.

---

## Commands (verified — run from `apps/portal/`)

| Command | What it does |
|---|---|
| `npm run setup` | Full bootstrap: install deps + build debug NAPI addon + build frontend |
| `npm run dev` | Parallel Vite + Electron, game window only (debug Rust, hot reload) — Vite on port **5173** |
| `npm run dev:release` | Build release NAPI addon, then Vite + Electron |
| `npm run dev:tools` | Like `dev`, but Electron gets `--dev-tools` so the dev-ui window opens too |
| `npm run dev:rust` | Rebuild debug NAPI addon (`cd ../simulation && npm run build:debug`) |
| `npm run dev:rust-release` | Rebuild release NAPI addon |
| `npm test` | `vitest` (frontend unit tests, co-located `*.test.ts`) |
| `npm run test:coverage` | `vitest --coverage` |
| `npm run type-check` | `tsc --noEmit` — **the only enforced quality gate** |
| `npm run build` | All `build:*` (release NAPI addon + `tsc && vite build`) |
| `npm run package` / `package:win` / `package:mac` / `package:linux` | `npm run build && electron-builder [--platform]` |

Helper (run from the **repo root**, not `apps/portal/`): `scripts/dev.sh` launches portal (5173) + dev-ui (5174) together.

---

## Portal vs Dev-UI — never mix (structurally enforced)

- **This app = the game, for PLAYERS.** Game world, creatures, player controls, minimal HUD only.
- **`apps/dev-ui` (`@speciate/dev-ui`) = developer metrics/profiling, for DEVELOPERS.** Separate React/Vite app on port **5174**. Hardware counters, system timings, NAPI buffer panels, heap profiler, spawn forms.
- **Local rule:** they are different npm packages on different ports — keep dev metrics/profiling/charts out of portal entirely. (The "would a player see this?" heuristic is a root global.)

---

## PixiJS interaction — use the event system, never raw DOM

All world-space pointer interaction MUST go through PixiJS's event system, NOT raw DOM listeners or `getBoundingClientRect`. PixiJS owns its coordinate system; manual DOM conversion is error-prone and races the render loop.

**Canonical reference:** `src/interaction/InteractionManager.ts`
- Set `eventMode = 'static'` on the hit area, then `.on('pointerdown' | 'pointermove' | 'pointerout', ...)` (lines 35–40).
- World coordinates via `event.getLocalPosition(this.worldContainer)` (lines 66, 82) — never `event.clientX` + rect math.
- Clean up in `destroy()` with `.off(...)` + `.destroy()` (lines 112–117).

Exception: keyboard / mouse-wheel pan-zoom lives in `src/input/InputManager.ts`, where DOM keyboard events are acceptable. Pointer hit-testing on the world still goes through PixiJS.

See the [PixiJS v8 Events guide](https://pixijs.com/8.x/guides/components/events).

---

## Binary IPC consumption — zero-copy Float32Array, no JSON on the hot path

Per-tick creature data arrives as a zero-copy `Float32Array` over NAPI/Electron IPC (the binary-not-JSON-on-the-hot-path rule is a root global; the portal-specific path is below).

End-to-end path:
1. Rust `export_positions()` fills a JS-owned `Float32Array` and returns the count.
2. `electron/napi-main.cjs` pre-allocates the buffer (1M-creature capacity — see `electron/bufferLayout.cjs`), calls `simulationEngine.fillBuffer(...)`, slices the active range, and sends `'napi-buffer-update'`.
   - **Gotcha (`napi-main.cjs:170–173`):** copy out with `.slice(0, usedSize)`, **never** `.subarray()`. `.subarray()` returns a view into the full backing `ArrayBuffer`, so Electron's structured clone serializes the entire 10MB buffer every tick.
3. `electron/preload.cjs` exposes a `contextBridge` surface (`onNAPIBufferUpdate`, `onTelemetryUpdate`, `onPerceptionDebugUpdate`, command senders) — raw `ipcRenderer` is never exposed. `onStateUpdateBinary` is `@deprecated` (dead stdio/MessagePack path).
4. `src/infrastructure/ipc/ElectronIPCClient.ts` (implements the `IPCClient` interface) parses the **SoA layout** `[IDs, Xs, Ys, Rots, Sizes]` via `getBufferOffsets(count)`, mutating a pre-allocated creature object pool **in place** each tick (zero-allocation-per-tick discipline).

**Load-bearing contract:** `src/types/BufferLayout.ts` — `FLOATS_PER_CREATURE = 5`. Its layout MUST match Rust `export_positions()` in `apps/simulation/src/ipc/bridge/bevy_app.rs`. **Any layout change must be made on both sides simultaneously.**

Note: the perception-debug buffer layout is hard-coded in `ElectronIPCClient.ts` (lines 6–17) — a fragile dual-maintained layout; change it on both sides too.

---

## TypeScript code quality

- **`tsc` strict is the enforced gate** (`strict`, `noUnusedLocals`, `noUnusedParameters`, `noImplicitReturns`, `noFallthroughCasesInSwitch`). There is **no ESLint** — the rules below are conventions, machine-checked only by `tsc`. Run `npm run type-check` before claiming done.
- `console.log`: **zero** in non-test `src/` — the root no-`console.log` rule is upheld here.
- `any`: the only sanctioned exception in portal `src/` is the PixiJS v8 Mesh/geometry typing escapes in `src/rendering/` — see [`src/rendering/AGENTS.md`](./src/rendering/AGENTS.md).
- Tests are **Vitest**, co-located beside source. Path alias `@/*` → `src/*`.

---

## Architecture layering

Clean-architecture layers under `src/`:
- `domain/` — pure logic (`Camera`, `Viewport`, `Creature`, `CameraController`, `WorldBounds`)
- `infrastructure/ipc/` — `IPCClient` interface + `ElectronIPCClient`
- `rendering/` — instanced-Mesh `InterpolatedCreatureRenderer`, `minimap/`, `overlays/`
- `input/`, `interaction/`, `systems/`, `core/`, `ui/`, `types/`

**Note:** `apps/portal/ARCHITECTURE.md` is partly stale — its clean-architecture / world-container / sprite-scaling sections are still useful, but ignore its WebSocket transport, `SpritePool`, and "Sprint" framing. The live transport is Electron NAPI IPC; rendering is Mesh-instanced.

---

Claude Code users also have specialized agents and slash commands under `.claude/` — optional extras, not required for any workflow here.
