# Depth & Altitude Sprite Encoding (2.5D vertical layers)

## Problem / Opportunity

If creatures gain a vertical habitat axis (underwater / surface / land / air), the player
needs to *read* which layer a creature is in at a glance — without a real 3D camera. A cheap,
proven 2D trick conveys vertical position purely through sprite styling, giving a "2.5D" feel
on the existing flat renderer.

## Proposed Solution

Encode a creature's vertical position by modulating its sprite, scaled by how far it is from
the surface plane:

- **Depth (underwater):** render the sprite **dimmer and smaller** the deeper it is. Reads
  instantly as "below the surface, far from the camera." (Seen elsewhere; it works.)
- **Altitude (flying):** the symmetric inverse — the sprite reads as **elevated** (e.g.
  larger / brighter) **plus a drop shadow cast on the ground beneath it**, offset by altitude.
  The shadow is the key cue that separates "flying high" from "big creature on the ground."

So a single continuous vertical scalar drives: surface = normal; underwater = dim+small
trending toward dark; airborne = raised sprite + ground shadow growing with altitude. The
amount of dimming/shrinking/shadow-offset maps to depth/altitude magnitude, so transitions
(a creature diving or taking off) animate smoothly rather than snapping between states.

## Golden Zone

N/A — pure visuals feature. (Its value is making the aquatic/flight *gameplay* legible.)

## Trade-offs

- Tint + scale per creature is cheap, but **drop shadows** are extra draw work — at 1M
  creatures shadows likely need to be limited (only near-camera / only large / culled) or
  batched, else they double the sprite count. Depth dim+scale alone is nearly free (a tint
  and a transform already in the render path).
- Deep/dim sprites can become hard to click/select — selection + overlays must still work on
  faded sprites.
- Tint must not collide with other sprite color encodings (species, state) — pick a channel
  (brightness/alpha + scale) that composes with existing coloring.

## Expert Input

Not yet consulted — logged for later. When picked up, consult `shader-sarah` on the cheapest
way to do per-creature depth tint + scale in the Pixi batch, and whether ground shadows can
be a separate cheap pass or must be culled at scale.

## Dependencies

- The aquatic / vertical-habitat-layer mechanic that gives each creature a depth/altitude
  value to render (see Related). Until that exists there's nothing to encode.

## Related Ideas

- `docs/biology/ideas/aquatic-habitat-layers.md` — the mechanic this visualizes (this idea's
  gameplay half).

## Open Questions

- Discrete tiers (surface/shallow/deep) vs a continuous scalar for the visual ramp?
- Do underwater creatures need a water-surface tint/overlay over them, or is per-sprite
  dimming enough?
- Shadow budget at 1M — cull, batch, LOD, or skip shadows below a size threshold?
- Should depth also blur or desaturate, or only dim + shrink (keep it cheap)?

---
*Captured: 2026-06-28*
