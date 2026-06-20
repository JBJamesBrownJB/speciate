See ../AGENTS.md (and root /AGENTS.md) for global rules.

This app is the canonical NO side of Portal-vs-DevUI: developer metrics/profiling/debugging only, never shipped, Vite dev server on port 5174.

## Local commands

- `npm run dev` — codegen + `vite --port 5174` (`package.json:9`).
- `npm run build` — codegen + `tsc && vite build` (`package.json:10`).
- `npm run generate:trials` — spec-template codegen only (`package.json:8`).

## Traps (dev-ui specific)

### `npm test` is NOT a real gate
`test` chains `test:ipc` + `test:trial` (`package.json:12-14`). `test:ipc` runs `test-ipc.cjs`, which `require('msgpack-lite')` (`test-ipc.cjs:12`) and spawns the stdio binary `../simulation/target/debug/speciate` (`test-ipc.cjs:15`). The NAPI migration deleted that stdio path, so this test cannot pass and proves nothing. Do not treat a green/red here as signal.

### `dev` and `build` silently run codegen first
Both prepend `generate:trials` (`package.json:9-10`), which runs `scripts/generate-trial-list.cjs`. That script scans `apps/simulation/specs/**/*.toml` (`generate-trial-list.cjs:14`) and emits `src/generated/trial-templates.ts` (git-ignored). This is non-obvious cross-app coupling: adding/renaming a spec `.toml` in the simulation app changes the dev-ui trial dropdown. Never hand-edit the generated file — re-run the codegen.

### `@msgpack/msgpack` is a vestige
Declared at `package.json:19` but unused anywhere in `src/`. Do not build new IPC on it; it is a leftover from the dead stdio era.

### `snapshotConverter.ts` copies fields blind
`src/utils/snapshotConverter.ts` maps snapshots via `(obj as any)[key]` (`:6`, `:14`, `:27`). There is no compile-time link between source keys and destination interfaces — a renamed timing/hardware field is dropped SILENTLY (no error, just a missing metric). After any rename upstream, eyeball the rendered metric, not the build.

### `SystemTimingsSnapshot` has already drifted from portal
dev-ui's `SystemTimingsSnapshot` (`src/types.ts:50`) is missing the IPC timing fields portal already carries — portal's (`apps/portal/src/types/GameState.ts:28-43`) has `ipcQueryUs`, `ipcSerializeUs`, `ipcWriteUs`, `ipcWriterThreadUs` (plus `ipcFrameDropsTotal`, `ipcChannelUtilizationPct`); dev-ui lacks them. These interfaces are maintained independently, so they fall out of sync.

When adding a new instrumented system, follow the authoritative checklist in `apps/simulation/AGENTS.md` (do not duplicate it here). The dev-ui-side touch-points are only:
- add the field to the `SystemTimingsSnapshot` interface (`src/types.ts:50`);
- wire its sparkline in `src/components/SystemTimingsPanel.tsx` (timing fields render automatically via the per-key loop; for a friendly label add it to the label map near `:189`, and remember count metrics get the separate count-sparkline path).
