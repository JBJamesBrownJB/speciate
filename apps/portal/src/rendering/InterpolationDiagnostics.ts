/**
 * InterpolationDiagnostics — DEV-ONLY render-pipeline probe.
 *
 * Measures snapshot delivery cadence and render stalls (the dev-ui Render Pipeline
 * panel). It diagnosed — and now guards against regressions of — the jerky-visuals
 * bug (docs/testing/bugs/jitter-high-populations.md, RESOLVED). Healthy looks like a
 * tight ~50 ms snapshot gap (low σ), ~0 duplicate frames, and ~0 stall frames.
 *
 * Cost controls:
 * - Every call site is guarded by `import.meta.env.DEV`, so this whole file is
 *   dead-code-eliminated from production builds (the player portal).
 * - Running accumulators only — no per-frame allocations, no per-frame I/O.
 * - Produces ONE structured snapshot per second (consumed by the dev-ui Render
 *   Pipeline panel via main), so the probe cannot perturb what it measures.
 */

/** Structured one-interval snapshot, sent to the dev-ui Render Pipeline panel. */
export interface RenderPipelineMetrics {
  distinctGapMeanMs: number;
  distinctGapStdMs: number;
  distinctGapMinMs: number;
  distinctGapMaxMs: number;
  deliveryMeanMs: number;
  stallFrames: number;
  totalFrames: number;
  distinctCount: number;
  duplicateCount: number;
}

const REPORT_INTERVAL_MS = 1000;

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

  /** One render frame; `clampedAtEnd` = alpha was pinned at 1.0 (no motion this frame). */
  recordFrame(clampedAtEnd: boolean): void {
    this.totalFrames++;
    if (clampedAtEnd) this.stallFrames++;
  }

  /**
   * Once per interval: returns a structured snapshot for the dev-ui Render Pipeline
   * panel, then resets the window. Returns null between intervals. The caller
   * forwards the snapshot to the dev-ui over IPC.
   */
  maybeReport(now: number): RenderPipelineMetrics | null {
    if (!this.lastReportT) {
      this.lastReportT = now;
      return null;
    }
    if (now - this.lastReportT < REPORT_INTERVAL_MS) return null;
    this.lastReportT = now;

    const d = this.distinct;

    const metrics: RenderPipelineMetrics = {
      distinctGapMeanMs: d.mean,
      distinctGapStdMs: d.std,
      distinctGapMinMs: d.lo,
      distinctGapMaxMs: d.hi,
      deliveryMeanMs: this.delivery.mean,
      stallFrames: this.stallFrames,
      totalFrames: this.totalFrames,
      distinctCount: this.distinctCount,
      duplicateCount: this.duplicateCount,
    };

    this.delivery.reset();
    this.distinct.reset();
    this.stallFrames = 0;
    this.totalFrames = 0;
    this.distinctCount = 0;
    this.duplicateCount = 0;

    return metrics;
  }
}

/** Shared dev-only singleton. Marked pure so bundlers drop it when unused (prod). */
export const interpDiag = /*@__PURE__*/ new InterpolationDiagnostics();
