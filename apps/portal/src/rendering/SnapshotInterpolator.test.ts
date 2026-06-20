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

// maxQueue bounds the buffer so the renderer's slot pool can safely recycle slots
// (a slot still referenced in the queue must never be overwritten). It is the cap
// that makes the GC ring sound: poolSize >= maxQueue + 2.
describe('SnapshotInterpolator maxQueue (ring-safety cap)', () => {
  const capped = (maxQueue: number) =>
    new SnapshotInterpolator<string>({ tickIntervalMs: 50, maxQueue });

  it('defaults to unbounded — the buffer grows and keeps the oldest pair', () => {
    const c = new SnapshotInterpolator<string>({ tickIntervalMs: 50 });
    for (let i = 0; i < 10; i++) c.push(`S${i}`);
    // No cap: still rendering the oldest pair, one tick in the past.
    expect(c.current()).toEqual({ from: 'S0', to: 'S1', alpha: 0 });
  });

  it('caps the buffer at maxQueue by dropping the OLDEST (keeps the freshest)', () => {
    const c = capped(3);
    for (let i = 0; i < 6; i++) c.push(`S${i}`); // S0..S5
    // Only the newest 3 survive: S3,S4,S5 → oldest renderable pair is S3->S4.
    expect(c.current()).toEqual({ from: 'S3', to: 'S4', alpha: 0 });
  });

  it('dropping the oldest at the cap does NOT reset the in-flight alpha', () => {
    const c = capped(3);
    c.push('S0'); c.push('S1'); c.push('S2');
    c.advance(25); // alpha 0.5 on S0->S1
    expect(c.current()!.alpha).toBe(0.5);

    c.push('S3'); // exceeds cap of 3 → drops S0; clock value must be untouched
    const seg = c.current()!;
    expect(seg.from).toBe('S1'); // pair advanced (oldest dropped)...
    expect(seg.to).toBe('S2');
    expect(seg.alpha).toBe(0.5); // ...but the playout clock was NOT reset
  });

  it('never drops below the look-ahead depth needed to play', () => {
    const c = capped(3);
    c.push('A'); c.push('B'); c.push('C');
    // A cap of 3 still leaves >=2 buffered + 1 ahead, so playback is live.
    expect(c.current()).not.toBeNull();
  });
});
