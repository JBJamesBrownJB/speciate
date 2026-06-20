export interface InterpolationSegment<T> {
  from: T;
  to: T;
  alpha: number;
}

/**
 * Render-in-the-past playout clock (Valve/Bernier entity interpolation; see
 * docs/render-pipeline/). Snapshots arrive via push(); a render clock advances via
 * advance(deltaMs); current() yields the {from,to,alpha} segment to lerp.
 *
 * It renders ONE tick in the past: it won't produce a segment until a snapshot is
 * buffered *beyond* the pair being shown, so there's always a target ahead to roll
 * into. That is what removes the end-of-tween "stall" — alpha never has to reach 1.0
 * and wait. alpha is a continuous clock that rolls over between snapshots; it is
 * NEVER reset on arrival (arrival only appends to the buffer).
 *
 * Generic over the snapshot payload T so the timing is unit-testable without position
 * data; the renderer uses ring-buffer slots / position arrays as T.
 */
export class SnapshotInterpolator<T> {
  private tickIntervalMs: number;
  private readonly queue: T[] = [];
  private alpha = 0;
  private started = false;

  /** Buffer depth required before we start playing (>=3 ⇒ one tick of look-ahead). */
  private static readonly START_DEPTH = 3;

  constructor(opts?: { tickIntervalMs?: number }) {
    this.tickIntervalMs = opts?.tickIntervalMs ?? 50;
  }

  setTickInterval(ms: number): void {
    if (ms > 0) this.tickIntervalMs = ms;
  }

  /** Append a snapshot. Never resets the in-flight tween. */
  push(snapshot: T): void {
    this.queue.push(snapshot);
    if (this.queue.length >= SnapshotInterpolator.START_DEPTH) this.started = true;
  }

  /** Advance the render clock by real elapsed time. Rolls over between snapshots. */
  advance(deltaMs: number): void {
    if (!this.started || this.tickIntervalMs <= 0) return;
    this.alpha += deltaMs / this.tickIntervalMs;
    while (this.alpha >= 1) {
      // Roll only while a snapshot beyond the current pair exists (keep >=2 buffered);
      // otherwise hold at the newest (underrun) rather than overshoot.
      if (this.queue.length > 2) {
        this.queue.shift();
        this.alpha -= 1;
      } else {
        this.alpha = 1;
        break;
      }
    }
  }

  /** The current segment to interpolate, or null until playback has started. */
  current(): InterpolationSegment<T> | null {
    if (!this.started || this.queue.length < 2) return null;
    return { from: this.queue[0], to: this.queue[1], alpha: this.alpha };
  }

  /** Clear all buffered snapshots and the clock (e.g. on (re)initialize). */
  reset(): void {
    this.queue.length = 0;
    this.alpha = 0;
    this.started = false;
  }
}
