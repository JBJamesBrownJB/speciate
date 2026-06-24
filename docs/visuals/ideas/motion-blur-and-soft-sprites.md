# 💡 Idea: Soft-Edged Creatures + Per-Creature Velocity Streak (motion blur)

> **Category: 💡 IDEAS (backlog).** Pillar 2 (Prove Spectacle). **Deferred — not
> scheduled.** Logged 2026-06-24 so it isn't lost; the engine (Pillar 1) comes
> first. This is a *look* upgrade, not a smoothness fix — see the honest framing
> below before anyone reaches for it.
>
> **Legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND
>
> Composes with [`procedural-gait-synthesis.md`](./procedural-gait-synthesis.md)
> (gait deforms the body; the streak stretches the whole sprite along its motion
> vector — orthogonal, stackable) and [`shader-animation.md`](./shader-animation.md).

---

## Where this came from

The author's CV ([jbjamesbrownjb.github.io/cv](https://jbjamesbrownjb.github.io/cv/)) runs a
background Boids flock whose motion looks *silky*. Reading the source, the "blur" is three
stacked Canvas2D tricks in its `draw()`:

1. **Alpha-fade trail** — instead of clearing, it paints a translucent background over the
   previous frame (`fillRect` at `rgba(10,15,17, 0.22)`). Old pixels decay ×0.78/frame, so
   each boid leaves a ~20-frame comet tail.
2. **Additive blend** (`globalCompositeOperation='lighter'`) — overlapping boids sum their
   light, so flocks bloom/glow.
3. **Soft radial sprites** — each agent is a fuzzy gradient blob, not a hard dot, so no
   aliasing.

## Honest framing (read this first)

- **That demo runs ~300–1,900 agents and steps its sim *every frame*, so its trail is purely
  aesthetic** — it is *not* compensating for a low tick rate. Speciate already solves temporal
  smoothness a better-for-scale way: 20 Hz sim + **GPU snapshot interpolation** (`mix()` +
  shortest-path rotation slerp in the vertex shader, `InterpolatedCreatureRenderer.ts:185-220`).
  So what the CV trick offers Speciate is **look, not smoothness.** Pillar 2, not Pillar 1.
- **A direct port does not survive the scale jump.** At 1,000,000 GPU-instanced creatures a
  naive cross-frame fade-trail washes the screen to mud, and naive additive blending blows out
  to white in dense regions. The *principle* transfers; the *method* must be re-derived for 1M.
- The one real motion artifact our interpolation *does* have — a piecewise-linear "corner" in
  each creature's path at every 50 ms tick boundary — a streak/soft-edge would visually soften.
  Minor, but a genuine secondary win.

## Two options (build B first — C depends on it)

### Option B — soft-edged creature sprites (this is also the aliasing fix)
Replace the hard quad fill with a **radial alpha falloff** computed procedurally in the
fragment shader (`smoothstep` on distance-from-centre in `vTextureCoord`), `NORMAL`
(premultiplied-alpha) blend. Optional additive *glow* only as a separate, lower-res pass —
never by blending 1M sprites additively.

Why it leads: **a soft alpha edge *is* analytic antialiasing** — there's no jagged silhouette
left for MSAA to fix. It kills the aliasing the author currently sees, enables glow, and is the
substrate C needs. Three wins from one texture change.

### Option C — per-creature velocity streak ⭐ (the scale-appropriate "blur")
The vertex shader **already has `aStartPos` and `aEndPos`** (this tick's displacement = velocity
× 50 ms — free). Stretch each creature's quad along that displacement so fast creatures render
as streaks and slow ones stay crisp dots. Bounded **per creature** (no cross-frame
accumulation), which is why it can't wash out at 1M. Golden-Zone flavour: **streak length reads
as speed** — a darting predator vs a grazing herbivore, legible at a glance.

Design (from a shader-sarah consult, 2026-06-24):
- **Stretch along velocity, not heading.** They diverge most exactly during a turn — the moment
  a streak is most informative. Texture sampling still uses the existing heading-rotation.
- **Anchor the head, trail the tail backward** (not symmetric scaling) → reads as forward
  motion with drag, not vibration.
- **Length** `= min(speed × uStreakScale, size × uStreakCap)` — zero at rest, linear with speed,
  **capped in body-lengths** so a fast whale can't smear across the viewport. Both constants are
  live dev-UI-tunable uniforms (add to the `UniformGroup` at
  `InterpolatedCreatureRenderer.ts:237`, expose via `getUniforms()` at `:380`).
- **Tail alpha-fade** via a `vTailT` varying (`alpha *= 1 - vTailT`) — the bit that makes it
  read as *blur* not *wedge*. Only works on a soft sprite → **C depends on B.**
- Consult [zoologist-tom] on the two defaults and whether the cap should scale allometrically
  (small creatures twitchy → relatively *longer* streak; giants → shorter).

## The antialias finding (the author's question, 2026-06-24)

`antialias: false` at `apps/portal/src/main.ts:65` is **not a considered visual call** — it was
flipped from `true` during the Electron migration (commit `7aa5b2b`) with the comment *"Disable
AA to reduce GPU load during init."* A perf-panic toggle, never revisited.

**Verdict: leave it `false`; the soft sprite (B) is the right fix.** MSAA only antialiases
*geometry edges*, not the sprite interiors that actually look jagged at 1M sub-10px creatures,
and it costs the most exactly at 1M on a `low-power` GPU. It's also an init-time flag (flipping
it means recreating the WebGL context). The analytic soft edge is free and smooths *every* edge,
including the streak taper MSAA could never handle.

**Separate, real perf lever found alongside it:** `powerPreference: 'low-power'` (`main.ts:63`)
tells hybrid-GPU laptops to use the *integrated* GPU. For a million-creature showcase we almost
certainly want `'high-performance'` (the discrete card). Untested; likely free frames. Worth a
benchmark independent of any visual work.

## Test-first plan (when scheduled)

The GLSL itself isn't unit-testable, but the load-bearing math is — extract it as a pure TS
mirror the shader is a verified hand-port of (repo mandate: test-FIRST):

- `apps/portal/src/rendering/streakGeometry.ts` → `streakGeometry(startX,startY,endX,endY,size,scale,cap) -> {length, dirX, dirY}`.
  Red tests: zero velocity → no streak; direction normalized (3,4 → 0.6,0.8); linear below cap;
  **clamped above cap** (the load-bearing one); `scale=0` → off switch.
- The radial falloff (B) → pure `radialFalloff(r) -> alpha` (r=0 → 1, r=1 → 0, monotonic).
- A uniform-contract test asserting `getUniforms()` exposes `uStreakScale`/`uStreakCap` with
  documented defaults (guards the dev-UI contract).

**Build order:** B (soft sprite — also the aliasing fix) → C (streak) behind a dev-UI toggle +
two sliders, A/B'd live at 1M → consult tom on constants. Side-quest: benchmark
`low-power → high-performance`.

---

**Owner:** Pillar 2 (Prove Spectacle) · **Status:** 💡 Idea / deferred · **Logged:** 2026-06-24
