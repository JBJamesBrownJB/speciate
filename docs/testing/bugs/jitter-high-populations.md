When population is high 100k and latency is close to max 40ms or so we get jerky movement of crits.

It should be fine as latency is always below 50ms (unless our sampling is hiding breaching this though we don't get warnings of skipped / caught up frames in console).

It is even worse whith pub const PERCEPTION_SKIP_TICKS above zero.

Again, it appears that total_tick is always under 50 so there shouldn't be any jitter / jerky movement still.

---

## Update 2026-06-20 — reproduces with large headroom (Windows)

Observed on Windows at **~500k creatures** with the tick completing in **~30 ms** (i.e. ~20 ms of headroom under the 50 ms / 20 Hz budget). Movement is still jerky/jumpy, as if the lerp/interpolation is not running between ticks.

This **rules out the "latency near 50 ms" theory** — there is plenty of headroom and the 20 Hz tick is comfortably hit, yet the jitter persists. The problem therefore looks intrinsic to the render-side interpolation, not to the sim missing its budget.

**Where to look (render interpolation):**
- `apps/portal/src/rendering/InterpolatedCreatureRenderer.ts:263` advances `interpolationAlpha += deltaMS / tickIntervalMs` each frame, clamped to [0,1] (`:264`), and resets to 0 on each new snapshot (`:240,247`).
- `tickIntervalMs` is set from `getTickIntervalMs(tickRateHz)` via `setTickRate()` (`:352-353`). If `tickRateHz` is stale/unset (alpha stays at `Infinity`-derived 0) or doesn't match the *actual* snapshot delivery cadence, alpha will not span [0,1] smoothly between updates → snapping.
- Suspect mismatch between **snapshot arrival cadence** (IPC double-buffer delivery) and `tickIntervalMs`: if snapshots arrive irregularly or the alpha reaches 1.0 and stalls before the next snapshot, motion looks stepped even with CPU headroom.
- Worsened by `PERCEPTION_SKIP_TICKS > 0` (per original report), consistent with position updates effectively arriving less often than interpolation assumes.

**Next step:** instrument the actual gap between consecutive snapshot swaps vs `tickIntervalMs`, and confirm `setTickRate()` is being called with the live tick rate. Related renderer: `InterpolationBufferManager.ts`.