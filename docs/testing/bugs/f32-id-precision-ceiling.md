# 🔴 CRITICAL (deferred) — Creature ids lose precision above 16.7M cumulative spawns

> **Status: 📋 Planned — CRITICAL, deferred.** Not a problem today (current sessions
> spawn far fewer than 16.7M creatures cumulatively, and there is no mortality system
> yet). It becomes a **correctness bug** once long-running sessions with creature death
> + continuous respawn push the cumulative id counter past the f32-exact range. Logged
> now because the 1M population cap (`docs/scale/`) makes it easy to reach sooner.

## Summary

Creature ids cross the Rust↔JS seam as **`f32`**. An `f32` represents integers exactly
only up to **2²⁴ = 16,777,216**. `CritId` is a **monotonic `u32` counter that is never
recycled**, so after ~16.7M cumulative spawns the id written to the seam can no longer
be represented exactly — distinct creatures collide onto the same `f32` id. The
renderer matches `from`→`to` **by id** for interpolation, so colliding ids produce
**wrong tweens: teleports, ghosting, and frozen/yanked creatures**.

## Where (file:line)

- **Producer cast (the defect):** `apps/simulation/src/ipc/bridge/bevy_app.rs` — `export_positions` writes `write_slice[i] = id.0 as f32` (the id lane of the SoA buffer).
- **Id source:** `apps/simulation/src/simulation/creatures/systems.rs` — `NextCreatureId { next_id: u32 }` starts at 1 and only ever does `next_id += 1` (`identity.rs`: `pub struct CritId(pub u32)`). No reuse on death.
- **Consumer (where collisions bite):** the renderer matches creatures by id —
  `apps/portal/src/rendering/CreatureFramePool.ts` (`idToIndex`) and
  `apps/portal/src/rendering/interleavedBuffer.ts` (`from.idToIndex.get(to.ids[i])`).
  Two live creatures sharing one `f32` id → one is matched to the other's previous
  position → a visible snap/teleport, exactly the class of artifact the
  snapshot-interpolation fix removed (`docs/architecture/snapshot-interpolation.md`).

## Why it is latent now / when it bites

- **Concurrent** population is fine: 1M *live* creatures with ids `1..1,000,000` are all
  well under 2²⁴. The cap raise to 1M does **not** by itself trigger this.
- It is **cumulative** spawns that matter. The counter is monotonic and never reused, so:
  - **Mortality + respawn** (planned — `docs/biology/ideas/mortality.md`): to hold a
    population steady, dead creatures are continuously replaced, so the counter climbs
    indefinitely regardless of the concurrent count.
  - **Long-running sessions** (Steam EA / "sim runs for ages"): even a modest spawn rate
    crosses 16.7M over hours/days.
- Above 2²⁴, consecutive integers are no longer all representable in `f32` (gaps of 2,
  then 4, …), so collisions begin **at** ~16.7M, not at the u32 ceiling (~4.29B).

## Impact

- **Correctness, not just visuals:** any consumer keying on the seam id (interpolation
  matching today; potentially selection/picking, overlays, debug tooling) can mis-associate.
- Manifests as intermittent teleport/ghost/jitter that **worsens over session lifetime**
  and is hard to reproduce from a fresh start — a nasty long-tail bug if not pre-empted.

## Fix directions (ranked)

1. **Bit-cast `u32` → `f32` in the existing id lane (recommended).** Keep the SoA buffer
   a `Float32Array`, but store the id's *bits*, not its numeric value: Rust
   `f32::from_bits(id.0)` on write; JS reads that lane through a `Uint32Array` view of the
   same `ArrayBuffer` (`new Uint32Array(buf.buffer, byteOffset, count)`). Exact for the
   full `u32` range, **zero extra bandwidth, no layout change** — only the id lane's
   interpretation changes. (Precedent: the codebase already bit-casts `f32`↔`u32` for the
   atomic time-scale in `simulation_engine.rs`.)
2. **Separate integer id channel.** Deliver ids in their own `Uint32Array`/`Int32Array`
   alongside the position `Float32Array`. Clean but adds a second buffer to the seam.
3. **Recycle / compact CritIds on death.** A free-list keeps concurrent ids below 2²⁴ so
   `f32` stays exact. Keeps the wire format but adds id-reuse semantics, which can confuse
   interpolation across reuse (a recycled id inherits a stale `from`) — least attractive.

## Test-first plan (when fixed)

- **Rust (`cargo test`):** a creature with `id > 16_777_216` round-trips through the seam
  buffer **exactly** (write then read back the id lane → equal `u32`).
- **Rust:** two ids differing only above the f32-exact range (e.g. `2^24 + 1` vs `2^24 + 2`)
  remain **distinct** after transport (they currently collide).
- **TS (`vitest`):** `interleavedBuffer` / `CreatureFramePool` match large ids correctly
  (`from`/`to` keyed on ids beyond 2²⁴ interpolate the right creature, no cross-match).
- All existing renderer/interpolation suites stay green (no regression for small ids).

## Acceptance criteria

- Ids up to the full `u32` range survive the seam without collision.
- No visual regression at small populations; the large-id tests above pass.
- The `as f32` cast site in `export_positions` is gone (or provably lossless).

---

**Document Owner:** rendering / IPC seam · **Severity:** Critical (deferred) · **Last Updated:** 2026-06-20
