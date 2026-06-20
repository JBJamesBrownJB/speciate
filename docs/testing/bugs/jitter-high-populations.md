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

---

## Theory (2026-06-20, triangulated by 3 investigations + the snapshot-interpolation literature)

**Correction (2026-06-20): the jerk is present even with a SINGLE creature.** This rules out all content/load-driven explanations (perception-skip lumpiness, population-dependent hash fragility, multi-tick coalescing under load) — a lone creature wanders at near-constant velocity, so any visible stepping is a pure timing artifact. This strengthens, not weakens, the cadence theory below: even perfectly smooth underlying motion looks jerky, which can only be a render-interpolation timing defect. A single creature is therefore the ideal test rig for the instrumentation.

**Root cause: delivery-cadence mismatch + alpha-reset-on-arrival.** The interpolation math and double buffer are correct; the defect is architectural. The renderer assumes a fixed 50 ms interpolation window and resets alpha to 0 on each new snapshot, but snapshots are not delivered every 50 ms.

Pipeline (file:line):
- Rust produces a fresh snapshot ~every 50 ms (one swap per tick, `apps/simulation/src/napi_addon/simulation_engine.rs:262,283`); the `DoubleBuffer` carries **no tick/sequence id** (`apps/simulation/src/ipc/bridge/double_buffer.rs:35`).
- Electron **polls** at a free-running **40 Hz / 25 ms `setInterval`** (`apps/portal/electron/napi-main.cjs:139-140,148`) and re-sends whatever is in the read slice — not phase-locked to the producer.
- Renderer: `interpolationAlpha += deltaMS / tickIntervalMs` (fixed 50 ms), clamped [0,1], **reset to 0** on each distinct snapshot (`apps/portal/src/rendering/InterpolatedCreatureRenderer.ts:247,263-264,352-353`).

Because the 25 ms poll and ~50 ms producer aren't phase-locked, the gap between *distinct* snapshots jitters ~25–75 ms while the renderer assumes exactly 50 ms:
- gap > 50 ms → alpha clamps at 1.0 early → **freeze then jump**;
- gap < 50 ms → alpha reset before reaching 1.0 → **snap backward and re-lerp**.

Aggravators:
- Duplicate polls (40 Hz over a 20 Hz buffer) only weakly suppressed by a 6-creature position hash (`apps/portal/src/core/ChangeDetection.ts:29-50`) — a leaked duplicate = frozen frame.
- Multi-tick coalescing exports only the final tick (`simulation_engine.rs:262`) → occasional double-distance jump.
- `PERCEPTION_SKIP_TICKS > 0` (now removed from Rust source) made motion lumpy (stale-then-corrected forces), amplifying the snaps.

This matches the canonical anti-pattern in Gaffer's *Snapshot Interpolation* and Valve's `cl_interp` ("rendering at the latest snapshot with reset-on-arrival stutters even when the producer keeps up").

**Cheapest confirmation before any fix:** log, on the renderer, the wall-clock delta between consecutive *distinct* snapshots (at the `changeDetector.shouldUpdate === true` site, `apps/portal/src/main.ts:258`) and the value of `interpolationAlpha` at each reset. Expect a spread around 50 ms (not tight) and alpha frequently pinned at 1.0 or reset well below 1.0.

**Fix direction (ranked):**
1. Snapshot interpolation: attach a tick/sequence id to each exported buffer; buffer snapshots; render ~1 tick in the past; drive alpha from a real-time clock between two timestamped snapshots; never reset on arrival. (Most robust — the Valve/Gaffer model.)
2. Interim: drive alpha from *measured* (smoothed) elapsed-since-last-snapshot instead of a hardcoded 50 ms; ignore duplicate snapshots via the sequence id.
3. Delivery-side: push one IPC message per Rust buffer swap instead of a 40 Hz poll (removes duplicates/phase beat at the source).

Sources: Gaffer "Fix Your Timestep" & "Snapshot Interpolation"; Valve "Source Multiplayer Networking" (`cl_interp 0.1`).

### CONFIRMED by measurement (2026-06-20, single creature)

The DEV probe (`apps/portal/src/rendering/InterpolationDiagnostics.ts`) reported, steadily, with one creature:

```
distinct-gap 50ms (27–68, σ16) | delivery 32ms | α@reset 0.84 (0.60–1.00) | stalls ~22/101f | dupes 38%
```

Every predicted signature is present:
- **distinct-gap**: mean is correct (50 ms) but variance is large (σ16, range 27–68 ms) — snapshot delivery jitters, exactly as theorised.
- **α@reset = 0.84 avg, down to 0.60**: the lerp is frequently truncated well before 1.0 → the renderer snaps to the next tick mid-move (the buffer sets `start = previous end`, so the rendered position jumps forward to the old target, then re-lerps).
- **stalls ~22%**: when a gap runs long (>50 ms), alpha clamps at 1.0 and the creature freezes until the next snapshot.
- **dupes 38%**: the ~32 Hz poll re-reads the unchanged 20 Hz buffer ~1 in 3 times.

So the motion alternates snap (short gaps) and freeze (long gaps). The mean gap being 50 ms is why average speed looks right while the frame-to-frame motion is jerky. **Diagnosis confirmed; proceed to the fix.** The probe doubles as the before/after verifier: a correct fix should drive α@reset → ~1.0, stalls → ~0, and distinct-gap σ → low.