import { describe, it, expect } from 'vitest';
import { createZoomRateLimiter } from './zoomRateLimiter';

const CONFIG = { sensitivity: 0.001, maxSpeed: 2.0 };

describe('createZoomRateLimiter', () => {
  it('scroll up yields a zoom-in factor (> 1), scroll down zoom-out (< 1)', () => {
    const step = createZoomRateLimiter(CONFIG);
    expect(step(-100, 1000)!).toBeGreaterThan(1);

    const step2 = createZoomRateLimiter(CONFIG);
    expect(step2(100, 1000)!).toBeLessThan(1);
  });

  it('returns null for a zero-delta wheel event (no zoom, no clock reset)', () => {
    const step = createZoomRateLimiter(CONFIG);
    expect(step(0, 1000)).toBeNull();
  });

  it('the first event is not rate-limited (no prior zoom to limit against)', () => {
    const step = createZoomRateLimiter(CONFIG);
    // Raw delta would be 0.5 in log space — allowed in full on the first event.
    expect(step(-500, 0)!).toBeCloseTo(Math.exp(0.5), 10);
  });

  it('clamps rapid successive zooms to maxSpeed (log-units per second)', () => {
    const step = createZoomRateLimiter(CONFIG);
    step(-500, 1000);

    // 10ms later: budget = 2.0 * 0.010 = 0.02 log units, request is 0.5 → clamped.
    const factor = step(-500, 1010)!;
    expect(factor).toBeCloseTo(Math.exp(0.02), 10);
  });

  it('clamping preserves direction for zoom-out too', () => {
    const step = createZoomRateLimiter(CONFIG);
    step(500, 1000);
    const factor = step(500, 1010)!;
    expect(factor).toBeCloseTo(Math.exp(-0.02), 10);
  });

  it('a long pause restores the full budget', () => {
    const step = createZoomRateLimiter(CONFIG);
    step(-500, 1000);
    // 1s later: budget = 2.0 log units — the 0.5 request passes unclamped.
    expect(step(-500, 2000)!).toBeCloseTo(Math.exp(0.5), 10);
  });
});
