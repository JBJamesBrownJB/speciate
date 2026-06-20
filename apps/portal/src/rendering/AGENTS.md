See ../../AGENTS.md and root /AGENTS.md for global rules.

This file holds only the non-inheritable footguns of the rendering layer.

## `FLOATS_PER_CREATURE` name collision (live footgun)

There are TWO unrelated constants with this name. Do not conflate them.

- **Here (rendering): `FLOATS_PER_CREATURE = 7`** — `InterpolatedCreatureRenderer.ts:15` and `InterpolationBufferManager.ts:11`.
  Interleaved **AoS** GPU vertex layout, one record per creature:
  `[startX, startY, endX, endY, startRot, endRot, size]` with `STRIDE = 28` (7 × 4 bytes) at `InterpolatedCreatureRenderer.ts:80`.
  Attribute offsets confirm the order: `aStartPos`@0, `aEndPos`@8, `aStartRot`@16, `aEndRot`@20, `aSize`@24.
  This is the per-instance interpolation geometry consumed by the instanced shader.

- **IPC layer: `FLOATS_PER_CREATURE = 5`** — `src/types/BufferLayout.ts:10`.
  A **SoA** contract, different value, different layout, different sync target (mirrors the Rust `export_positions()` export). Pointed at here only to CONTRAST — see that file for its specifics; do not re-document it here.

Same name, different value, different memory layout. An agent editing one must not assume the other follows.

## Sanctioned `any` escapes

These are the only blessed `any` escapes in `portal/src` (the avoid-`any` principle is otherwise inherited). They cover PixiJS v8 typing gaps for a hand-written `#version 300 es` instanced shader (its own world→NDC transform, double-buffered geometry to avoid GPU stalls):

- `InterpolatedCreatureRenderer.ts:47` — `new Mesh(...) as any` (custom `Geometry` not accepted by the v8 `Mesh` constructor type).
- `InterpolatedCreatureRenderer.ts:125` — `(geometry as any).indexBuffer = null`.
- `InterpolatedCreatureRenderer.ts:126` — `(geometry as any).vertexCount = 4` (4 quad vertices per instance, triangle-strip).
