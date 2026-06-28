# Conspicuousness / Visibility Allometry ("a giant is a lighthouse")

**Status:** ✅ Implemented (engine + overlay) — ⏳ trophic canary deferred (see Future Work)
**Location:** `apps/simulation/src/simulation/core/components.rs` (`BodySize::conspicuousness`)
**Consult:** zoologist-tom, 2026-06-28 — full reasoning in `docs/biology/biology-notes.md`

## What It Does

A creature's **body size** now determines how far away **others** can detect it. Before
this, a target contributed only its physical radius (`length / 2`) to the detection
distance — so a 10 m giant announced itself no further than its own skin, identical in
reach to a 0.5 m minnow, even though the giant's *own* perception range is ~210 m. Large
creatures were "seen too late."

Each creature gains a **conspicuousness** distance:

```
conspicuousness(length) = 0.71 · length^1.5,  clamped [0.1, 60] m
```

This **replaces** the target's bare-radius term in the perception detection check (mass
and physics still use the true physical radius — no double-count).

**Magnitudes** (coefficient pinned so the population median is unchanged):

| Body length | Physical radius | Conspicuousness | vs old (radius) |
|-------------|-----------------|-----------------|-----------------|
| 0.5 m (median) | 0.25 m | **0.25 m** | unchanged (pinned) |
| 1 m | 0.5 m | 0.71 m | +0.21 m |
| 2 m | 1.0 m | 2.0 m | ×2 |
| 5 m | 2.5 m | 7.9 m | ×3.2 |
| 10 m (giant) | 5.0 m | **22.4 m** | ×4.5 |

A median creature spots a 10 m giant at ~27.6 m instead of ~10.25 m (~2.7× earlier); a
giant spotting another giant barely moves (+8%). The boost concentrates in the
**small-observer → giant** channel, which is exactly where "seen too late" hurt.

## Why It Exists

**Biological basis (zoologist-tom):** detection is driven by **apparent angular size** —
a target of size `L` is resolved when the angle it subtends, `θ ≈ L/D`, exceeds the
observer's acuity floor, giving `D ∝ L`. **Area summation (Ricco's law)** pushes the
exponent toward `L²` ("you can't hide an elephant"); **atmospheric/water extinction**
(Koschmieder) trims the far tail. Real animals sit at **k ≈ 1.0–1.5**. We use **k = 1.5**:
being-seen (silhouette ~L²) outpaces seeing (observer reach ~L^1.25) — *"a giant is a
lighthouse before it is a telescope."*

This is the dual of the existing observer-side allometry (`perception/components.rs`):
big eyes see farther, but big bodies are seen from even farther.

## Key Mechanics

### The pure function
**Location:** `core/components.rs` `BodySize::conspicuousness()`

`= C · length² · inv_sqrt_length`, where the cached `inv_sqrt_length` turns `length^1.5`
into two multiplies — **no `powf`**. Constants in
`creatures/constants/perception.rs` (`CONSPICUOUSNESS_*`); `C = 1/√2` is the pin that
makes `conspicuousness(0.5) = 0.25` exactly.

### Precompute, never per-check
**Location:** `spatial/systems.rs` (grid rebuild) → `spatial/grid.rs` (`PerceptionProxy`)

Conspicuousness is computed **once per creature** at spatial-grid rebuild and stored on
the `PerceptionProxy` (28 → 32 bytes, a clean 2-per-cache-line). The perception hot loop
examines tens of millions of candidate pairs per tick at 1 M scale, so any per-check
`powf`/`sqrt` would be catastrophic — the proxy carries the finished value.

### Folded into BOTH perception checks
**Location:** `perception/systems.rs` (two detection sites)

1. **Broad-phase cull:** `max_dist = range + self_radius + proxy.conspicuousness`.
2. **Binding check:** `should_perceive_entity` is fed
   `effective_range_sq = (range + conspicuousness)²` instead of `range²`.

The binding check previously compared *center distance ≤ range*, ignoring target size
entirely — so changing only the broad-phase cull would have done nothing. Both had to
change. (`entity_filter.rs` `should_perceive_entity` now documents the effective-range
contract.)

## Constants

**See:** `apps/simulation/src/simulation/creatures/constants/perception.rs`

| Constant | Value | Purpose |
|----------|-------|---------|
| `CONSPICUOUSNESS_COEFFICIENT` | `1/√2 ≈ 0.7071` | Pins median (0.5 m) to 0.25 m |
| `CONSPICUOUSNESS_EXPONENT` | `1.5` | Super-linear "being-seen" growth |
| `CONSPICUOUSNESS_MIN` | `0.1 m` | Floor (smallest stay detectable) |
| `CONSPICUOUSNESS_MAX` | `60 m` | Ceiling (extinction analogue) |

**Tuning rule:** raise **k** (exponent) to lift only giants; the coefficient lifts the
98% median band and should stay pinned.

## Visualization

**Location:** `apps/portal/src/rendering/overlays/PerceptionOverlay.ts`,
`apps/portal/src/domain/conspicuousness.ts`

Selecting a creature with the perception overlay ('p') on draws an **amber ring** at its
conspicuousness radius — how far away it can be seen — next to the cyan FOV wedge (what it
can see). The client mirrors the Rust formula; `domain/conspicuousness.test.ts` pins the
same reference points as the Rust tests so the two can't drift.

## Tests

- **Math** (`core/components.rs`): median pin (`0.5 → 0.25`), giant magnitude
  (`10 → 22.36`), `powf`-reference equality, monotonicity, clamps.
- **Grid seam** (`spatial/grid.rs`): proxy carries conspicuousness independent of physical
  radius (no swap, no double-count into mass).
- **End-to-end** (`perception/tests.rs`): a small observer perceives a giant placed
  *beyond* the old radius-only gate, while a same-distance small control stays invisible —
  proving the extra reach is **size-driven**, not a blanket widening.
- **Frontend**: helper unit tests + overlay ring tests (renders, omitted radius is
  backward-compatible, resets on clear).

## Future Work

### Trophic canary (deferred — not yet runnable)
The protocol (`biology-notes.md`): *reject if apex OR grazer steady-state population shifts
> ±20%.* The engine currently has **no birth/death/reproduction** — population is static,
energy only modulates behaviour — so a steady-state population shift cannot be measured
yet. This change is therefore **held on its branch, not merged to main**, gated behind the
canary for when trophic dynamics land. First mitigation if apex later breaches: motion-gate
(below). Gentler fallback curve: `k = 1.3, C = 0.616` (tune **k**, not C).

### Counterbalance levers (reserved)
1. **Motion-gated conspicuousness** `conspic *= f(speed)` — strongest, and a Golden Zone: a
   frozen ambush giant goes dark *and* skips detection work.
2. **`crypsis_gene`** multiplier `[0.3, 1.0]` — keep the signature
   `conspicuousness(length, crypsis = 1.0)`-shaped so it drops in without re-plumb.
3. **Size-domination de-weighting** — small predators decline giant prey ("seen ≠ predated").
4. **Herd detection** — many-eyes effect.

### DNA slot
Pure function of `length` today (no gene). Natural extension is the `crypsis_gene`
multiplier above.

## References

- `docs/biology/biology-notes.md` — full zoologist-tom consult (2026-06-28)
- `docs/biology/done/fov-perception.md` — the observer-side allometry this is dual to
- `apps/simulation/src/simulation/creatures/constants/perception.rs` — all constants

---

**Last Updated:** 2026-06-28
