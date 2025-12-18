/**
 * Buffer layout constants for NAPI creature data.
 *
 * CRITICAL: These values MUST match the Rust export_positions() layout in:
 *   apps/simulation/src/ipc/bridge/bevy_app.rs
 *
 * Layout: [ID₁...IDₙ, X₁...Xₙ, Y₁...Yₙ, Rot₁...Rotₙ, Size₁...Sizeₙ]
 */

export const FLOATS_PER_CREATURE = 5;

export interface BufferOffsets {
  id: number;
  x: number;
  y: number;
  rot: number;
  size: number;
}

export const getBufferOffsets = (creatureCount: number): BufferOffsets => ({
  id: 0,
  x: creatureCount,
  y: creatureCount * 2,
  rot: creatureCount * 3,
  size: creatureCount * 4,
});
