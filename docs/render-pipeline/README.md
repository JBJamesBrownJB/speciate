# Render Pipeline — Smooth Motion Across the Rust↔JS Seam

**Category:** ✅ Done / 📖 Reference. Both parts shipped 2026-06-20 (the task records live in [`done/`](./done/)). The explainer below is standing reference knowledge for the problem and the fix.

> **Promoted.** The algorithm is now reference architecture: [`../architecture/snapshot-interpolation.md`](../architecture/snapshot-interpolation.md), with a one-line mention in the root `README.md` (credit Valve / Bernier + Fiedler).

Related: the bug — [`../testing/bugs/jitter-high-populations.md`](../testing/bugs/jitter-high-populations.md) · the live metrics — [`../scale/dev-ui-metrics-reference.md`](../scale/dev-ui-metrics-reference.md).

---

## 1. The problem (in one breath)

Creatures move **jerkily** even though the simulation is healthy — it hits its 20 Hz budget with room to spare (~30 ms of a 50 ms tick at 500k creatures). The jerk is present **with a single creature** moving in a straight line, so it is not about load or the creatures — it is purely a **timing defect in how the renderer shows motion**.

---

## 2. The mental model — a flipbook

```
 Simulation:  a new position every 50 ms (20 Hz)   ●         ●         ●         ●
 Your screen: redraws every ~8 ms (120 Hz)         |||||||||||||||||||||||||||||||
 Renderer's job:  smoothly SLIDE between the sim's positions, so 20 jumps a
                  second look like one continuous motion.
```

That slide is **interpolation**. The progress of one slide is **α (alpha)**, from `0` (still at the old position) to `1.0` (arrived at the new one).

**Smooth** = each slide reaches `1.0` *exactly* as the next position arrives, then immediately starts the next slide. **Jerk** = the timing doesn't line up.

---

## 3. Why it jerks — the cadence mismatch

The simulation produces positions on a steady ~50 ms beat. But a separate, free-running **poll** on the Electron side grabs whatever is in the buffer on *its own* clock (~31 ms), unsynchronised with the producer:

```
 Sim produces:   |——50ms——|——50ms——|——50ms——|        steady
 Poll grabs:     |—31—|—31—|—31—|—31—|—31—|—31—|       its own, faster clock
                  └────────── out of phase ──────────┘
 Net effect: the renderer first SEES each new position at uneven times:
             ...27ms... 68ms ...41ms... 55ms...   (wobble of σ ≈ 16 ms)
```

Because the renderer assumes a fixed 50 ms slide **and resets the slide every time new data arrives**, that wobble becomes two visible failures:

```
 gap SHORTER than 50 ms → slide only ~60% done, yanked to the next position → SNAP forward
 gap LONGER  than 50 ms → slide hits 100%, creature sits frozen, then jumps  → FREEZE
```

It alternated snap and freeze — that alternation *was* the jerk. (See the bug doc for the measured numbers: σ16, ~22% frozen frames — now resolved.)

---

## 4. Seeing it — the dev-ui "Render Pipeline" panel

The dev-tools window visualises this live (DEV builds only; metrics are renderer-origin, relayed portal → main → dev-ui). Full definitions: [`../scale/dev-ui-metrics-reference.md`](../scale/dev-ui-metrics-reference.md). The headline reads:

- **Snapshot gap** — time between new positions. Want a steady 50 ms with **low σ** (sigma = standard deviation = the wobble).
- **Stall frames** — frames frozen at the end of a tween. Want **~0%**. This is the render-side verification signal.
- Plus duplicate frames, delivery interval, snapshot rate, and a jitter (σ) sparkline with green/red good/bad reference lines.

> The original panel also carried a **Lerp completion (α@reset)** metric, but the fix made the renderer stop resetting α on arrival, so α@reset measured nothing and was **removed**. Stall frames is the surviving signal.

This panel was the **before/after instrument** for the fix below.

---

## 5. The industry fix — "render in the past" (snapshot interpolation)

We are borrowing a **networking** technique for a **local** app — because our **Rust↔JS NAPI seam behaves like a tiny network**: producer and consumer are decoupled and delivery is jittery.

**Origin:**
- **Yahn W. Bernier (Valve), GDC 2001** — *"Latency Compensating Methods in Client/Server In-game Protocol Design and Optimization"* — introduced **entity interpolation**: render the world slightly *in the past* and interpolate between two received snapshots. Shipped in the Source engine.
- **Valve "Source Multiplayer Networking"** — the `cl_interp` knob (default **0.1 s = 100 ms** behind): *"if objects were rendered only at the positions received, movement would look choppy and jittery — the fix is to go back in time for rendering."*
- **Glenn Fiedler — "Fix Your Timestep!" / "Snapshot Interpolation"** — the underlying math: `alpha = elapsed / dt`, a true previous+current buffer, and the buffered-snapshot variant.

**The idea:**

```
 snapshots arriving (each tagged with its sim tick):   A ───── B ───── C (latest)
 render clock  =  now − interpolationDelay (≈ 1 tick):          ●
                                                                │ render HERE,
                                                                │ between B and C
   alpha = (renderClock − B.time) / (C.time − B.time)   ← driven by the CLOCK, not by arrival
```

Key properties:
- **Never reset the slide on arrival.** A new snapshot just *appends to a buffer*; it never resets the clock or teleports the creature. This is what removes the snap/freeze.
- **Always a snapshot ahead.** Rendering ~1 tick in the past keeps the buffer ≥1 deep, so delivery jitter, duplicates, and even a dropped snapshot get **absorbed instead of shown**.
- **Cost:** ~50 ms of added latency (you render one tick in the past). Imperceptible for a creature sim; Valve uses 100 ms for networked play.

---

## 6. The fix was two complementary parts (both shipped)

| Order | Part | Side | What it does | Record |
|-------|------|------|--------------|--------|
| **1st** | **Push-on-swap** | Sim / IPC (Rust) | Replaced the polling messenger with an **event**: Rust rings a "new positions ready" doorbell on each buffer swap, tagged with its tick. Killed duplicate work + delivery jitter at the source. Touches Rust + Electron main only — **not** the renderer. | [`done/push-on-swap.md`](./done/push-on-swap.md) |
| **2nd** | **Snapshot interpolation** | Render (TS) | Renders in the past from a buffered queue of snapshots; drives α from a real-time clock; never resets on arrival. Mopped up the jitter the async boundary still left. | [`done/snapshot-interpolation.md`](./done/snapshot-interpolation.md) |

**Why this order (validated):** push-on-swap is the better *engineering* (event-driven beats polling — the messenger stopped doing ~38% wasted duplicate work) **and** fixed the root cause, in a layer (Rust + Electron main) that doesn't touch the renderer — no rework risk. The panel confirmed it: duplicates → 0%, σ → low, delivery → ~50 ms, **stalls 22% → ~15%**. The residual stalls were the async boundary's last wobble surviving reset-on-arrival; snapshot interpolation removed reset-on-arrival and drove **stalls → ~0%**. Together they are the complete, textbook-correct solution — motion is now "butter".

---

## 7. References

- **Bernier, Yahn W.** (Valve Software). *"Latency Compensating Methods in Client/Server In-game Protocol Design and Optimization."* Game Developers Conference (GDC), 2001. — the origin of entity interpolation ("render in the past").
- **Valve Developer Community.** *"Source Multiplayer Networking."* — entity interpolation in the Source engine; default `cl_interp 0.1` = render 100 ms in the past.
- **Fiedler, Glenn.** *"Fix Your Timestep!"* (2004) and *"Snapshot Interpolation."* gafferongames.com — the underlying fixed-timestep + interpolation math (`alpha = elapsed / dt`, previous+current buffer, buffered snapshots).

---

**Document Owner:** render pipeline · **Status:** ✅ Resolved 2026-06-20 · **Last Updated:** 2026-06-20
