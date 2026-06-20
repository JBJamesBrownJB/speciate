# Push-on-Swap (deliver positions once, on time, tagged)

**Status:** ✅ Implemented 2026-06-20 — the first of the two-part render-smoothness fix (root-cause: event-driven delivery replaced the free-running poll).
**Dependencies:** none. **Enabled** [`snapshot-interpolation.md`](./snapshot-interpolation.md) with steady, deduped, tick-tagged delivery (so its ring buffer stays shallow → lower latency).
**Area:** Sim / IPC (Rust + NAPI + Electron main).

**Implementation:** Rust doorbell — `apps/simulation/src/napi_addon/simulation_engine.rs` (`on_buffer_ready`, a `ThreadsafeFunction<u32>` fired once after each buffer swap, carrying the tick; registered before `start()`). Electron delivery — `apps/portal/electron/frameDelivery.cjs` (`createFrameDelivery`, DI-factored for unit testing) wired in `apps/portal/electron/napi-main.cjs` (push on the doorbell; telemetry on its own ~500 ms interval; poll kept only as a no-doorbell fallback). Tests: `apps/portal/electron/frameDelivery.test.ts`.

## Goal

Replace the free-running poll with an **event push**: the Rust run loop delivers each position buffer **once, the instant it swaps it** (right after a tick commits), **tagged with its sim tick number**. This removes duplicate frames and most of the delivery jitter at the source, and gives the render side accurate per-frame timestamps.

## Why (the problem it removes)

Today the Electron main process polls on a `setInterval` at ~2× the sim rate (`apps/portal/electron/napi-main.cjs:137-148`), on a clock unsynchronised with the producer. The buffer carries no tick id (`apps/simulation/src/ipc/bridge/double_buffer.rs`). Result (measured in the Render Pipeline panel): **~38% duplicate frames** and a **σ≈16 ms** wobble in when new positions are first seen. See [`../README.md`](../README.md) §3.

## Design (high-level)

- **Push from the swap point.** The sim already swaps the buffer only when a tick commits (`apps/simulation/src/napi_addon/simulation_engine.rs` around the export/swap, ~`:262`/`:283`). Emit a notification from there using a NAPI **`ThreadsafeFunction`** — the same mechanism telemetry already uses to call JS from the Bevy thread (`simulation_engine.rs:179-181`, `:305-313`). No new polling timer.
- **Tag each delivery with the sim tick.** The tick number is already available (`TickController.total_ticks`, `tick_controller.rs:47`; `get_tick`, `simulation_engine.rs`). Attach it to each pushed frame.
- **Forward + dedupe.** Main relays the pushed buffer to the renderer; the renderer ignores any frame whose tick id ≤ the last one seen (replaces the lossy 6-creature position hash in `apps/portal/src/core/ChangeDetection.ts`).
- **Keep the poll as a fallback only if needed** (e.g. a slow initial frame); the steady-state path is push.

Do **not** change tick timing or the fixed-timestep model (`tick_controller.rs`). This is purely *delivery*.

## Testing (automated, written first — TDD)

- **Rust (`cargo test --features dev-tools`):**
  - The tick id attached to a delivered frame is **monotonically increasing** and matches `total_ticks`.
  - The push fires **exactly once per committed tick**: zero fires on a frame where `ticks_this_frame == 0`; one fire when ≥1 tick committed (the multi-tick catch-up case still emits once, carrying the latest tick).
- **TS (`vitest`, portal):**
  - Feeding two frames with the **same** tick id → the dedupe drops the second (no `onSimulationTick`).
  - Feeding **increasing** tick ids → each is accepted exactly once.

## Results (measured live in the Render Pipeline panel, single creature)

| Metric | Before | After this task |
|--------|--------|-----------------|
| **Duplicate frames** | ~38% | **0%** ✅ (one delivery per swap — no duplicates to suppress) |
| **Delivery interval** | ~31 ms | **~50 ms** ✅ (matches the 20 Hz producer) |
| **Snapshot rate** | ~20/s | ~20/s (unchanged, as expected) |
| **Snapshot gap σ** | ~16 ms | **dropped sharply** ✅ (jitter sparkline fell to the green line) |
| **Stall frames** | ~22% | ~15% — improved, **but not gone** until the render-side fix |

The residual stalls confirmed the plan's premise: event-driven delivery removes the *delivery* jitter, but the renderer still reset its tween on arrival, so an async-boundary wobble could still freeze a frame. That last bit is what [`snapshot-interpolation.md`](./snapshot-interpolation.md) closed.

## Acceptance Criteria — met

- ✅ Rust + TS tests pass; full suites green; no tick-throughput regression.
- ✅ Panel shows duplicates 0% and delivery ~50 ms with one creature.
- ✅ Prod build unaffected (the dev probe stays DEV-gated and dead-code-eliminated).

---

**Document Owner:** render pipeline · **Last Updated:** 2026-06-20
