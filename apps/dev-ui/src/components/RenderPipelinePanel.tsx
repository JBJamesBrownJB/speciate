/**
 * RenderPipelinePanel
 *
 * Frontend / render-pipeline metrics (interpolation cadence between the 20 Hz sim
 * and the renderer). These are renderer-origin (portal), relayed via the main
 * process — they do NOT come from the Rust telemetry channel. DEV-only.
 *
 * Each metric is self-documenting: an always-visible one-line blurb plus a hover
 * tooltip explaining what it measures, what's healthy, and what the jitter bug
 * looks like. See docs/testing/bugs/jitter-high-populations.md.
 */

import React, { useRef, useEffect } from 'react';
import { COLORS } from '../utils/cockpit';
import type { RenderPipelineMetrics } from '../types';

interface Props {
  metrics?: RenderPipelineMetrics;
}

interface MetricRowProps {
  label: string;
  value: string;
  color: string;
  blurb: string;
  measures: string;
  healthy: string;
  bug: string;
}

const MetricRow: React.FC<MetricRowProps> = ({ label, value, color, blurb, measures, healthy, bug }) => (
  <div className="render-metric-row">
    <div className="rm-head">
      <span className="rm-label">{label}</span>
      <span className="rm-value" style={{ color }}>{value}</span>
    </div>
    <div className="rm-blurb">{blurb}</div>
    <div className="rm-tooltip">
      <div className="rm-tip-line"><strong>Measures:</strong> {measures}</div>
      <div className="rm-tip-line rm-tip-good"><strong>Healthy:</strong> {healthy}</div>
      <div className="rm-tip-line rm-tip-bad"><strong>Jitter bug:</strong> {bug}</div>
    </div>
  </div>
);

const MAX_HISTORY = 90;

const renderSparkline = (
  canvas: HTMLCanvasElement,
  history: number[],
  refValue: number | null,
  refMax: number
): void => {
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;
  ctx.scale(dpr, dpr);
  const { width, height } = rect;
  ctx.clearRect(0, 0, width, height);
  if (history.length < 2) return;

  const maxValue = Math.max(refMax, ...history) * 1.1 || 1;
  const xStep = width / (MAX_HISTORY - 1);

  if (refValue !== null) {
    ctx.beginPath();
    ctx.strokeStyle = 'rgba(74, 222, 128, 0.35)';
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 3]);
    const y = height - (refValue / maxValue) * height;
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  ctx.beginPath();
  ctx.lineWidth = 1.5;
  ctx.strokeStyle = COLORS.streaming;
  history.forEach((v, i) => {
    const x = i * xStep;
    const y = height - Math.min(v / maxValue, 1) * height;
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  });
  ctx.stroke();
};

const pick = (v: number, good: number, warn: number, invert = false): string => {
  const ok = invert ? v <= good : v >= good;
  const mid = invert ? v <= warn : v >= warn;
  if (ok) return COLORS.success;
  if (mid) return COLORS.warning;
  return COLORS.critical;
};

export const RenderPipelinePanel: React.FC<Props> = ({ metrics }) => {
  const jitterHist = useRef<number[]>([]);
  const alphaHist = useRef<number[]>([]);
  const jitterCanvas = useRef<HTMLCanvasElement>(null);
  const alphaCanvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!metrics) return;
    const jh = jitterHist.current;
    jh.push(metrics.distinctGapStdMs);
    if (jh.length > MAX_HISTORY) jh.shift();
    const ah = alphaHist.current;
    ah.push(metrics.alphaResetMean);
    if (ah.length > MAX_HISTORY) ah.shift();
    if (jitterCanvas.current) renderSparkline(jitterCanvas.current, jh, null, 16);
    if (alphaCanvas.current) renderSparkline(alphaCanvas.current, ah, 1.0, 1.0);
  }, [metrics]);

  if (!metrics) {
    return (
      <div className="cockpit-panel">
        <div className="cockpit-panel-title">Render Pipeline</div>
        <p className="no-data">Waiting for render-pipeline metrics…</p>
        <p className="hint">Frontend/interpolation metrics (DEV builds only).</p>
      </div>
    );
  }

  const m = metrics;
  const total = m.distinctCount + m.duplicateCount;
  const dupePct = total > 0 ? Math.round((m.duplicateCount / total) * 100) : 0;
  const stallPct = m.totalFrames > 0 ? Math.round((m.stallFrames / m.totalFrames) * 100) : 0;

  return (
    <div className="cockpit-panel render-pipeline-panel">
      <div className="cockpit-panel-title">
        Render Pipeline <span className="badge-tag">frontend</span>
      </div>

      <MetricRow
        label="Snapshot gap"
        value={`${m.distinctGapMeanMs.toFixed(0)} ms · σ${m.distinctGapStdMs.toFixed(0)} (${m.distinctGapMinMs.toFixed(0)}–${m.distinctGapMaxMs.toFixed(0)})`}
        color={pick(m.distinctGapStdMs, 5, 12, true)}
        blurb="Time between new position frames; want a steady ~50 ms (20 Hz). σ = sigma = standard deviation (the wobble) — lower is smoother."
        measures="Wall-clock gap between distinct (changed) position snapshots arriving at the renderer."
        healthy="Mean ~50 ms with low spread — σ (sigma / standard deviation) ≲ 5 ms."
        bug="Mean ~50 ms but high σ (sigma) = big timing wobble: the tween window keeps changing, so motion alternates snap and freeze."
      />

      <MetricRow
        label="Lerp completion (α@reset)"
        value={`${m.alphaResetMean.toFixed(2)} (${m.alphaResetMin.toFixed(2)}–${m.alphaResetMax.toFixed(2)})`}
        color={pick(m.alphaResetMean, 0.95, 0.85)}
        blurb="How far each tween finishes (0–1) before new data resets it. 1.0 = smooth."
        measures="Interpolation alpha at the instant a new snapshot resets it to 0."
        healthy="≈1.00 — the tween completes just as the next frame arrives."
        bug="< 1.0 means the creature is yanked toward the next position mid-move → a visible forward snap."
      />

      <MetricRow
        label="Stall frames"
        value={`${m.stallFrames}/${m.totalFrames} (${stallPct}%)`}
        color={pick(stallPct, 2, 10, true)}
        blurb="Render frames frozen at the end of a tween, waiting for the next snapshot."
        measures="Render frames where alpha was pinned at 1.0 (nowhere left to interpolate)."
        healthy="~0% — new data arrives before the tween runs out."
        bug="High % = freeze-then-jump: the snapshot arrived late, so the creature sat still then teleported."
      />

      <MetricRow
        label="Duplicate frames"
        value={`${dupePct}% (${m.duplicateCount}/${total})`}
        color={COLORS.neutral}
        blurb="Buffers re-read with no new data (the poll runs faster than the sim ticks)."
        measures="Deliveries carrying positions identical to the previous frame."
        healthy="Expected ≈ pollRate/tickRate − 1; harmless on its own."
        bug="Very high or erratic indicates the poll/produce mismatch that feeds the snapshot-gap jitter."
      />

      <MetricRow
        label="Delivery interval"
        value={`${m.deliveryMeanMs.toFixed(0)} ms`}
        color={COLORS.neutral}
        blurb="Cadence of every buffer received (including duplicates) — the poll rate."
        measures="Mean time between buffer deliveries to the renderer, before change detection."
        healthy="Steady; context for the duplicate-frame %."
        bug="If it drifts well off the sim's 50 ms beat, duplicates and phase jitter grow."
      />

      <MetricRow
        label="Snapshot rate"
        value={`${m.distinctCount}/s`}
        color={Math.abs(m.distinctCount - 20) <= 2 ? COLORS.success : COLORS.warning}
        blurb="Distinct position frames per second — should match the sim tick rate (20 Hz)."
        measures="Count of changed snapshots observed in the last second."
        healthy="≈20 (the sim tick rate)."
        bug="Below 20 = dropped frames; above 20 = duplicate leakage past the change detector."
      />

      <div className="rm-spark-grid">
        <div className="rm-spark">
          <div className="rm-spark-label">Jitter — σ sigma (std-dev), ms — want flat &amp; low</div>
          <canvas ref={jitterCanvas} className="memory-sparkline" />
        </div>
        <div className="rm-spark">
          <div className="rm-spark-label">Lerp completion (α alpha, 0–1) — want pinned at 1.0</div>
          <canvas ref={alphaCanvas} className="memory-sparkline" />
        </div>
      </div>

      <p className="rm-legend">
        <strong>σ</strong> (sigma) = standard deviation — how much a value wobbles around its
        average; lower = steadier. <strong>α</strong> (alpha) = how far a slide finished, 0–1.
      </p>
    </div>
  );
};

export default RenderPipelinePanel;
