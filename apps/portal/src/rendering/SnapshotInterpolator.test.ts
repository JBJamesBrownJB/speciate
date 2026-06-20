import { describe, it, expect } from 'vitest';
import { SnapshotInterpolator } from './SnapshotInterpolator';

// Pure render-in-the-past playout clock. Snapshots arrive (push); a render clock
// advances (advance(deltaMs)); current() yields the segment {from,to,alpha} to lerp.
// Renders ~1 tick in the past so there's always a buffered target — which is what
// removes the end-of-tween "stall". Uses tiny string snapshots; no position data.

const newClock = () => new SnapshotInterpolator<string>({ tickIntervalMs: 50 });

describe('SnapshotInterpolator (render-in-the-past playout)', () => {
  it('yields no segment until it has buffered one tick ahead (render in the past)', () => {
    const c = newClock();
    expect(c.current()).toBeNull();
    c.push('S0');
    expect(c.current()).toBeNull();
    c.push('S1');
    expect(c.current()).toBeNull(); // only 2 buffered — not yet 1-ahead
    c.push('S2');
    // Now interpolate the OLDEST pair (S0->S1) while S2 is the buffered-ahead.
    expect(c.current()).toEqual({ from: 'S0', to: 'S1', alpha: 0 });
  });

  it('renders behind the newest snapshot (the lag that prevents stalls)', () => {
    const c = newClock();
    c.push('S0'); c.push('S1'); c.push('S2');
    // `to` is S1, NOT the newest S2 — we are one tick in the past.
    expect(c.current()!.to).toBe('S1');
  });

  it('advances alpha from->to over the tick interval', () => {
    const c = newClock();
    c.push('S0'); c.push('S1'); c.push('S2');
    c.advance(25); // half of 50ms
    expect(c.current()).toEqual({ from: 'S0', to: 'S1', alpha: 0.5 });
  });

  it('NEVER resets alpha when a new snapshot arrives (the core invariant)', () => {
    const c = newClock();
    c.push('S0'); c.push('S1'); c.push('S2');
    c.advance(25);
    expect(c.current()!.alpha).toBe(0.5);
    c.push('S3'); // arrival must not reset the in-flight tween
    expect(c.current()).toEqual({ from: 'S0', to: 'S1', alpha: 0.5 });
  });

  it('rolls over to the next segment carrying the remainder (continuous, no snap to 0)', () => {
    const c = newClock();
    c.push('S0'); c.push('S1'); c.push('S2');
    c.advance(60); // 50ms completes S0->S1, 10ms into S1->S2
    const seg = c.current()!;
    expect(seg.from).toBe('S1');
    expect(seg.to).toBe('S2');
    expect(seg.alpha).toBeCloseTo(0.2, 5); // carried remainder, not snapped to 0

  });

  it('holds at the newest on buffer underrun (no overshoot / NaN)', () => {
    const c = newClock();
    c.push('S0'); c.push('S1'); c.push('S2');
    c.advance(500); // far past what is buffered
    const seg = c.current()!;
    expect(seg.to).toBe('S2'); // clamped to the last we have
    expect(seg.alpha).toBe(1); // held, not >1
  });

  it('respects a changed tick interval', () => {
    const c = newClock();
    c.setTickInterval(100);
    c.push('A'); c.push('B'); c.push('C');
    c.advance(25); // quarter of 100ms
    expect(c.current()!.alpha).toBeCloseTo(0.25, 5);
  });
});
