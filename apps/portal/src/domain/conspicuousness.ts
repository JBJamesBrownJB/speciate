// Conspicuousness / visibility allometry — client-side mirror of the Rust source
// of truth: `BodySize::conspicuousness`
// (apps/simulation/src/simulation/core/components.rs) and its constants
// (apps/simulation/src/simulation/creatures/constants/perception.rs).
//
// WHY a client-side mirror: only used to DRAW the conspicuousness ring around the
// selected creature in the perception overlay. The selected creature's size is
// already client-side, so this avoids widening the binary debug-IPC buffer.
// conspicuousness.test.ts pins the same reference points as the Rust unit tests,
// so any divergence from the engine formula is caught by a failing test.

/** C in `conspicuousness = C · length^1.5`; pinned to 1/√2 so conspic(0.5 m) = 0.25 m. */
export const CONSPICUOUSNESS_COEFFICIENT = Math.SQRT1_2;
export const CONSPICUOUSNESS_EXPONENT = 1.5;
/** Floor so the smallest creatures don't vanish from the detection term. */
export const CONSPICUOUSNESS_MIN = 0.1;
/** Ceiling — atmospheric/water extinction analogue. */
export const CONSPICUOUSNESS_MAX = 60.0;

/**
 * Detection distance (world units) that a creature's SIZE grants observers —
 * how far away it can be seen. `size` is body length (the same value the engine
 * ships as `CreatureData.size`; physical radius is `size / 2`).
 */
export function conspicuousness(size: number): number {
  const raw = CONSPICUOUSNESS_COEFFICIENT * Math.pow(size, CONSPICUOUSNESS_EXPONENT);
  return Math.min(Math.max(raw, CONSPICUOUSNESS_MIN), CONSPICUOUSNESS_MAX);
}
