/**
 * InterpolationDiagnostics — DEV-ONLY render-pipeline probe.
 *
 * Measures snapshot delivery cadence vs the interpolation clock to diagnose the
 * jerky-visuals bug (docs/testing/bugs/jitter-high-populations.md). Theory: the
 * renderer assumes a fixed 50 ms lerp window and resets alpha on each snapshot,
 * but snapshots are delivered by a free-running 40 Hz poll of a 20 Hz producer,
 * so the real gap jitters and alpha either stalls at 1.0 (freeze) or resets
 * mid-lerp (snap).
 *
 * Cost controls:
 * - Every call site is guarded by `import.meta.env.DEV`, so this whole file is
 *   dead-code-eliminated from production builds (the player portal).
 * - Running accumulators only — no per-frame allocations, no per-frame I/O.
 * - Emits ONE console line per second, so the probe cannot perturb what it measures.
 *
 * Read the line with a SINGLE creature spawned (the cleanest test rig):
 *   healthy  -> distinct-gap ~50ms tight (σ low), α@reset ~1.0, stalls ~0
 *   the bug  -> distinct-gap spread 25–75ms (σ high), α@reset scattered <1.0, stalls > 0
 */

class Accumulator {
  private n = 0;
  private sum = 0;
  private sumSq = 0;
  private min = Infinity;
  private max = -Infinity;

  add(v: number): void {
    this.n++;
    this.sum += v;
    this.sumSq += v * v;
    if (v < this.min) this.min = v;
    if (v > this.max) this.max = v;
  }

  get mean(): number {
    return this.n ? this.sum / this.n : 0;
  }

  get std(): number {
    if (!this.n) return 0;
    const m = this.mean;
    return Math.sqrt(Math.max(0, this.sumSq / this.n - m * m));
  }

  get lo(): number {
    return Number.isFinite(this.min) ? this.min : 0;
  }

  get hi(): number {
    return Number.isFinite(this.max) ? this.max : 0;
  }

  reset(): void {
    this.n = 0;
    this.sum = 0;
    this.sumSq = 0;
    this.min = Infinity;
    this.max = -Infinity;
  }
}

export class InterpolationDiagnostics {
  private lastDeliveryT = 0;
  private lastDistinctT = 0;
  private lastReportT = 0;

  private readonly delivery = new Accumulator(); // ms between every buffer received
  private readonly distinct = new Accumulator(); // ms between *changed* snapshots
  private readonly alphaReset = new Accumulator(); // alpha value when a snapshot resets it

  private stallFrames = 0; // render frames sitting clamped at alpha = 1.0 (freeze)
  private totalFrames = 0;
  private distinctCount = 0;
  private duplicateCount = 0;

  /** Every buffer delivered to the renderer (before change detection). */
  recordDelivery(now: number): void {
    if (this.lastDeliveryT) this.delivery.add(now - this.lastDeliveryT);
    this.lastDeliveryT = now;
  }

  /** Each state update; `isDistinct` = the change detector saw new positions. */
  recordSnapshot(now: number, isDistinct: boolean): void {
    if (isDistinct) {
      if (this.lastDistinctT) this.distinct.add(now - this.lastDistinctT);
      this.lastDistinctT = now;
      this.distinctCount++;
    } else {
      this.duplicateCount++;
    }
  }

  /** Interpolation alpha at the instant a new snapshot resets it to 0. */
  recordAlphaReset(alpha: number): void {
    this.alphaReset.add(alpha);
  }

  /** One render frame; `clampedAtEnd` = alpha was pinned at 1.0 (no motion this frame). */
  recordFrame(clampedAtEnd: boolean): void {
    this.totalFrames++;
    if (clampedAtEnd) this.stallFrames++;
  }

  /** Emits a one-line summary at most once per second, then resets the window. */
  maybeReport(now: number): void {
    if (!this.lastReportT) {
      this.lastReportT = now;
      return;
    }
    if (now - this.lastReportT < 1000) return;
    this.lastReportT = now;

    const total = this.distinctCount + this.duplicateCount;
    const dupePct = total > 0 ? Math.round((this.duplicateCount / total) * 100) : 0;
    const d = this.distinct;
    const a = this.alphaReset;

    // console.info (not log — banned; not error — this is not an error). DEV-only.
    console.info(
      `[interp] distinct-gap ${d.mean.toFixed(0)}ms (${d.lo.toFixed(0)}–${d.hi.toFixed(0)}, σ${d.std.toFixed(0)}) | ` +
        `delivery ${this.delivery.mean.toFixed(0)}ms | ` +
        `α@reset ${a.mean.toFixed(2)} (${a.lo.toFixed(2)}–${a.hi.toFixed(2)}) | ` +
        `stalls ${this.stallFrames}/${this.totalFrames}f | ` +
        `dupes ${dupePct}% (${this.distinctCount} new / ${this.duplicateCount} dup)`
    );

    this.delivery.reset();
    this.distinct.reset();
    this.alphaReset.reset();
    this.stallFrames = 0;
    this.totalFrames = 0;
    this.distinctCount = 0;
    this.duplicateCount = 0;
  }
}

/** Shared dev-only singleton. Marked pure so bundlers drop it when unused (prod). */
export const interpDiag = /*@__PURE__*/ new InterpolationDiagnostics();
