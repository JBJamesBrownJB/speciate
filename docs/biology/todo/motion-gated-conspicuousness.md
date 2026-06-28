# Motion-Gated Conspicuousness

**Status:** 📋 Planned — next upgrade to the shipped conspicuousness feature (do after the system-fusion perf work).
**Builds on:** `docs/biology/done/conspicuousness-visibility.md` (the size-only v1, already shipped).
**Dual of:** `docs/biology/todo/motion-detection.md` (observer-side, binary motion skip — see Relationship).
**Consult:** zoologist-tom, 2026-06-28 (reasoning below; constant *names* corrected to the real code).

---

## The idea

Today a creature's conspicuousness (how far away others can detect it) is a pure function of body **size**:

`conspicuousness(length) = 0.71 · length^1.5`, clamped `[0.1, 60]` m.

Add **movement** as a second axis: a creature **grows** more visible as it moves faster and **shrinks** toward invisibility as it slows/freezes. Freezing becomes a real evasion tactic; sprinting a real risk; ambush predators that creep go dark, then flare when they commit to the strike.

`conspicuousness(length, speed) = base(length) · motion_factor(speed)` — multiplicative, with `base` = the v1 formula.

## Biological basis (Tom)

Motion is the **dominant** detection cue, above static silhouette — vertebrate retinas have dedicated motion-detection circuitry; a frog literally can't resolve a *stationary* insect ("bug detector"); mammalian peripheral/rod vision is far more motion- than detail-sensitive. Hence the universal **freeze response** (deer/rabbits/fawns) and **stalk-and-ambush** creeping (felids, herons, mantises). Movement instantly defeats background-matching crypsis.

**Multiplicative is correct:** detection ≈ `silhouette_size × motion_salience × pattern_match`. Multiplying means any one channel collapsing toward zero (frozen, or frozen+camouflaged) makes the creature nearly undetectable — matching reality. Additive would wrongly let a big creature never hide.

## Recommended formula

```
s            = clamp(speed / max_speed(length), 0, 1)     // RELATIVE speed — see decision below
motion_factor = MOTION_FACTOR_MIN
              + (MOTION_FACTOR_MAX - MOTION_FACTOR_MIN) * s^MOTION_FACTOR_EXPONENT
final        = (base(length) * motion_factor).clamp(CONSPICUOUSNESS_MIN, CONSPICUOUSNESS_MAX)
```

New constants (alongside the existing `CONSPICUOUSNESS_*` in `creatures/constants/perception.rs`):

| Constant | Value | Meaning |
|---|---|---|
| `MOTION_FACTOR_MIN` | **0.45** | Floor — frozen ≠ invisible; a still creature keeps ~45% of its silhouette visibility (halves detection radius, no cloak). |
| `MOTION_FACTOR_MAX` | **2.5** | Cap — a flat-out sprint is a 2.5× beacon. |
| `MOTION_FACTOR_EXPONENT` | **1.5** (convex) | Slow movement stays near the floor (stalk/creep regime), detectability ramps hard only on sprint commitment; keeps typical cruise (`s≈0.45`) near neutral ≈1.0 so v1's size tuning is preserved. |

`motion_factor` curve:

| `s` (frac of own max speed) | regime | motion_factor |
|---|---|---|
| 0.0 | frozen | **0.45** |
| 0.25 | stalk/creep | 0.71 |
| 0.45 | cruise | ~1.07 (≈ neutral) |
| 0.75 | fast | 1.78 |
| 1.0 | sprint | **2.5** |

Convex (not saturating) on purpose: the gameplay+biology payoff is at the **two ends** — freeze/creep to go dark vs sprint and light up. A saturating curve would blunt the sprint-risk. (A smoothstep sigmoid is the "purer" dual-threshold shape but pushes cruise above 1.0 for no gameplay gain — skip for v1.)

## The key decision: RELATIVE speed (fraction of own `max_speed`)

**Use `s = speed / max_speed(length)`, not absolute m/s.** This is the load-bearing call.

- `base(length)` **already owns the size axis** (giants are intrinsically conspicuous). Absolute speed would **double-count size**: `max_speed ∝ length^0.25`, so giants would be both bigger *and* always "fast" → god-tier visibility *and* structurally unable to stalk; mice would be permanently "slow" → unable to ever light up. The lever dies at exactly the extremes where we want it.
- Self-betrayal is behavioral, normalized to one's own envelope: a leopard creeping at 10% of its capacity is being stealthy; a gazelle bolting at 100% betrays itself — independent of body size.
- Relative speed gives **every body plan the full 0.45×–2.5× range**, so freeze and sprint are meaningful for a 0.5 m mouse and a 10 m apex alike. Orthogonal decomposition: size = intrinsic visibility; motion = "how hard am I pushing my own throttle right now."

`clamp(…,0,1)` protects against transient `speed > max_speed`.

## Magnitudes (through the `[0.1, 60]` clamp)

| creature | base | frozen ×0.45 | cruise ×1 | sprint ×2.5 | radius swing |
|---|---|---|---|---|---|
| 0.5 m (median) | 0.25 | **0.11** (floor) | 0.27 | 0.63 | ~6× |
| 2 m | 2.0 | 0.90 | 2.1 | 5.0 | ~5.5× |
| 10 m (apex) | 22.4 | **10.1** | 24 | **56** | ~5.5× |

A frozen giant ambusher drops from a 22 m beacon to ~10 m (just above its 5 m skin) — it goes dark. A sprinting giant flares to 56 m. Detection-radius swing ≈ 5.5×; since at-risk area ∝ radius², the effective spot-chance swing is **~30×** — freezing is genuine evasion, sprinting genuine risk. Conservative dial if too swingy: `MIN=0.5, MAX=2.0` (≈4× radius swing).

## Trophic / gameplay + Golden Zone

**Dynamics:** ambush predator creeps → near the 0.45 floor → detected late → sprints to strike → flares to 2.5× but the strike is committed (heron/felid pattern, emergent). Prey that detects a threat and freezes halves its visibility (a small one floors to 0.1 m = point-blank only); movement defeats it.

**Trophic-canary caveat (honest):** there's a real **predator tilt** — stalkers darken on approach *and* fleeing prey light up, both raising predation-detection rates (partial self-balance: predators must sprint-and-light during the chase; prey that reach cover/freeze go dark). **The engine has no birth/death/reproduction, so this canNOT be validated as a population shift** — only as a per-role **detection/interaction-rate** metric, branch-only. Gate: per-role detection-event rate within ±20% of the size-only baseline; if approach-darkening pushes predator detection-success >20%, raise `MOTION_FACTOR_MIN` toward 0.55–0.6 (less darkening) rather than touching the cap. (See the no-population-dynamics constraint — same as v1.)

**Golden Zone — where the perf actually lives (be precise):**
- *Target-side (this feature):* a frozen creature's visibility radius shrinks (median floors to 0.1 m), shrinking its query/insert footprint — **modest** saving unless creatures are bucketed by conspicuousness tier so observers can skip the low-tier bucket at range.
- *Observer-side (`motion-detection.md`):* observers skip near-stationary targets entirely — **this is the big compute saving** (the literal frog-retina model).
- **They compose:** a frozen creature both shrinks its own footprint (here) *and* gets skipped by motion-keyed observers (there) → the full "freeze = dark **and** cheap" payoff. Honest framing: this feature is primarily a **gameplay/balance** lever with a modest target-side perf win; build it paired with the observer-side skip for the headline Golden Zone.

## `crypsis_gene` interaction (future)

Keep crypsis a **separate, third multiplicand**: `base(length) · motion_factor(speed) · crypsis_factor(crypsis_gene)` — do NOT fold it into `motion_factor`. Orthogonal axes: size (intrinsic) × motion (transient/behavioral) × crypsis (evolved/genetic). Best biology: let `crypsis_gene` lower the **floor** far more than the **cap** — because **motion defeats camouflage**. Clean encoding: scale crypsis's effect by `(1 - s)` so it only pays off while slow → "evolve camouflage ⇒ adopt freeze behavior" emerges rather than being scripted. Keep the v1 signature shape `conspicuousness(length, speed, crypsis=1.0)`-ready.

## Implementation notes

- `BodySize::conspicuousness` (`core/components.rs`) gains a `speed: f32` arg; reuse the existing `max_speed()`. The real v1 constant is `CONSPICUOUSNESS_COEFFICIENT` (= 1/√2 ≈ 0.7071), not "scale".
- **Stays precompute-once, not per-check.** Conspicuousness is already computed per-creature at spatial-grid rebuild and stored on `PerceptionProxy`; the rebuild tuple already carries `(vx, vy)`, so `speed = √(vx²+vy²)` is available there — no new hot-loop cost. Caveat: that's ~one `sqrt` per creature per rebuild (≈1M `sqrt`/tick). Acceptable once-per-creature, but if it shows on the rebuild phase, reformulate in `speed²` (compare `s² = speed²/max_speed²`; `s^1.5 = (s²)^0.75`) or use a fast inverse-sqrt to avoid the transcendental.
- Median behaviour is **no longer pinned** — a moving median creature now ranges 0.11–0.63 m (vs the flat 0.25 m of v1). That's intended (the whole point), but note it for the canary baseline.

## Tests to write first (red → green)

- frozen median floors at 0.1; frozen 10 m ≈ 10.1; sprint 10 m ≈ 56 (< 60 cap); cruise (`s=0.45`) ≈ base (within ~7%).
- `s=0` and `s≥1` don't panic; monotonic increasing in `speed`.
- **relative invariance:** a 0.5 m and a 10 m creature both at `s=0.5` get the *same* `motion_factor`.
- consumer-boundary: a frozen giant is detected at ~half the distance of a sprinting one (drive the real perception path, mirroring v1's end-to-end test).

## Graduation

On implement, fold the result into `docs/biology/done/conspicuousness-visibility.md` (it has a reserved "motion-gated" counterbalance section) and flip this doc's status.

---
*Captured: 2026-06-28*
