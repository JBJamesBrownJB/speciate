import { describe, it, expect } from 'vitest';
import {
  conspicuousness,
  CONSPICUOUSNESS_MIN,
  CONSPICUOUSNESS_MAX,
} from './conspicuousness';

// Mirrors the Rust source of truth: BodySize::conspicuousness
// (apps/simulation/src/simulation/core/components.rs) and its constants
// (apps/simulation/src/simulation/creatures/constants/perception.rs). These
// reference points are the SAME ones asserted in the Rust unit tests, so any
// drift between the two formulas fails here.
describe('conspicuousness (mirrors Rust BodySize::conspicuousness)', () => {
  it('pins the population median (0.5 m) to its physical radius 0.25', () => {
    expect(conspicuousness(0.5)).toBeCloseTo(0.25, 5);
  });

  it('makes a 10 m giant a lighthouse (~22.36 m, ~4.5x its 5 m radius)', () => {
    expect(conspicuousness(10)).toBeCloseTo(22.3607, 3);
    expect(conspicuousness(10)).toBeGreaterThan((10 / 2) * 4);
  });

  it('matches C * size^1.5 at reference points', () => {
    expect(conspicuousness(1)).toBeCloseTo(0.7071068, 5);
    expect(conspicuousness(2)).toBeCloseTo(2.0, 5);
    expect(conspicuousness(5)).toBeCloseTo(7.9057, 3);
  });

  it('is monotonic increasing in size', () => {
    expect(conspicuousness(0.5)).toBeLessThan(conspicuousness(1));
    expect(conspicuousness(1)).toBeLessThan(conspicuousness(5));
    expect(conspicuousness(5)).toBeLessThan(conspicuousness(10));
  });

  it('clamps to [MIN, MAX]', () => {
    expect(conspicuousness(0.05)).toBe(CONSPICUOUSNESS_MIN);
    expect(conspicuousness(10000)).toBe(CONSPICUOUSNESS_MAX);
  });
});
