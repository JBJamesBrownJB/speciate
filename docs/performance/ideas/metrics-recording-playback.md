# Metrics Recording & Playback (snapshot → recording)

**Status:** 💡 Idea — not committed.

## Concept

Today the dev-ui **snapshot** captures a *statistical summary* (avg / min / max / p95 …) of a sampling window and shows it as a single **frozen** column beside the live system. Upgrade it to a **recording**: capture the full time-series of samples and **play it back as an animated column** alongside (or scrubbable against) live — so **both sides move**, not just the live one.

## Why

- A frozen average hides the dynamics that matter — spikes, ramps, oscillation. A recording shows behaviour *over time*.
- Lets you compare a past run's **trajectory** to a live run, not just its averages (e.g. "did jitter spike at the same population as last time?").
- It's the natural home for **renderer-origin, time-series metrics** (the new Render Pipeline panel — snapshot gap, α, stalls) that the *static* snapshot does not capture today.

## What it enables

- Play / pause / scrub a recorded run.
- Sync playback to live on a shared tick/time axis → true side-by-side animation.
- Regression diffing: overlay recorded vs live sparklines.

## Rough approach (not committed)

- Persist the raw **sample stream** (the `TelemetryFrame[]` already collected in `processSamplesAndSave`, `apps/dev-ui/src/components/DevToolsApp.tsx`) rather than only the reduced `MetricsSnapshot` — plus the render-pipeline metric stream.
- A playback clock drives an index into the recording; each frame feeds the existing `MetricsColumn`, which already animates from live frames.
- Keep the current statistical snapshot as a "summary" view *of* a recording.

## Relationship to current code

- Builds on the existing capture/save/load path; today it reduces samples to stats (`MetricsSnapshot`) — a recording keeps the frames.
- The comparison `MetricsColumn` already renders from frames, so playback reuses it.
- Render-pipeline metrics arrive on a separate channel (portal → main → dev-ui), so a recording would capture **both** streams on one time axis.

## Caveats

- Recording size: raw frames are larger than a stat summary — bound the duration and/or downsample.
- Prompted by: the static snapshot dropping new metrics (e.g. windowsMetrics) — fixed for the snapshot, but a recording is the longer-term shape.

---

**Document Owner:** dev-ui profiling · **Last Updated:** 2026-06-20
