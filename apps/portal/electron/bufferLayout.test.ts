import { describe, it, expect } from 'vitest';
// @ts-ignore - plain CJS module under test; types not needed for this spec
import { MAX_CREATURES, FLOATS_PER_CREATURE, creatureBufferFloats } from './bufferLayout.cjs';

// The Electron-main receive buffer must be sized to match the Rust producer cap
// (apps/simulation/src/ipc/bridge/double_buffer.rs MAX_CREATURES). If this buffer is
// smaller than what Rust delivers, positions are silently truncated at the seam — so
// these tests pin the cap and the SoA sizing math (the JS twin of the Rust cap).

describe('bufferLayout — Electron-main receive buffer sizing', () => {
  it('carries at least one million creatures (matches the Rust producer cap)', () => {
    expect(MAX_CREATURES).toBeGreaterThanOrEqual(1_000_000);
  });

  it('uses the 5-float SoA layout (ID, X, Y, Rot, Size)', () => {
    expect(FLOATS_PER_CREATURE).toBe(5);
  });

  it('sizes the creature buffer as maxCreatures * floatsPerCreature', () => {
    expect(creatureBufferFloats()).toBe(MAX_CREATURES * FLOATS_PER_CREATURE);
    expect(creatureBufferFloats()).toBeGreaterThanOrEqual(1_000_000 * FLOATS_PER_CREATURE);
  });

  it('accepts an explicit creature count for sizing', () => {
    expect(creatureBufferFloats(123)).toBe(123 * FLOATS_PER_CREATURE);
  });
});
