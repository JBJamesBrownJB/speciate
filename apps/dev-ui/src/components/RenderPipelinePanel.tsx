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
  label?: string;
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

interface RefLine {
  value: number;
  color: string;
  label: string;
}

/**
 * Sparkline with a fixed [yMin, yMax] scale, dashed good/bad reference lines (so a
 * value's height is meaningful), axis labels, and a trace coloured by severity.
 */
const renderSparkline = (
  canvas: HTMLCanvasElement,
  history: number[],
  yMin: number,
  yMax: number,
  refLines: RefLine[],
  traceColor: string,
  topLabel: string,
  bottomLabel: string
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

  const span = yMax - yMin || 1;
  const yOf = (v: number) => height - Math.max(0, Math.min((v - yMin) / span, 1)) * height;

  ctx.font = '9px system-ui, sans-serif';

  // Dashed good/bad reference lines with right-aligned labels — these ARE the scale.
  refLines.forEach(({ value, color, label }) => {
    const y = yOf(value);
    ctx.globalAlpha = 0.5;
    ctx.strokeStyle = color;
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 3]);
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.globalAlpha = 1;
    ctx.fillStyle = color;
    ctx.textAlign = 'right';
    ctx.fillText(label, width - 3, Math.min(height - 2, Math.max(9, y - 2)));
  });

  // Y-axis scale labels (top = yMax, bottom = yMin) so the absolute scale is clear.
  ctx.fillStyle = '#64748b';
  ctx.textAlign = 'left';
  ctx.fillText(topLabel, 3, 9);
  ctx.fillText(bottomLabel, 3, height - 2);

  if (history.length < 2) return;
  const xStep = width / (MAX_HISTORY - 1);
  ctx.beginPath();
  ctx.lineWidth = 1.5;
  ctx.strokeStyle = traceColor;
  history.forEach((v, i) => {
    const x = i * xStep;
    const y = yOf(v);
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

export const RenderPipelinePanel: React.FC<Props> = ({ metrics, label }) => {
  const suffix = label ? ` — ${label}` : '';
  const jitterHist = useRef<number[]>([]);
  const jitterCanvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!metrics) return;
    const jh = jitterHist.current;
    jh.push(metrics.distinctGapStdMs);
    if (jh.length > MAX_HISTORY) jh.shift();

    if (jitterCanvas.current) {
      // Jitter (σ ms): lower is better. 0–30 ms scale; good ≤5, bad ≥12.
      renderSparkline(
        jitterCanvas.current,
        jh,
        0,
        30,
        [
          { value: 5, color: COLORS.success, label: '5 good' },
          { value: 12, color: COLORS.critical, label: '12 bad' },
        ],
        pick(metrics.distinctGapStdMs, 5, 12, true),
        '30 ms',
        '0'
      );
    }
  }, [metrics]);

  if (!metrics) {
    return (
      <div className="cockpit-panel">
        <div className="cockpit-panel-title">Render Pipeline{suffix}</div>
        <p className="no-data">Waiting for render-pipeline metrics…</p>
        <p className="hint">Frontend/interpolation metrics (DEV builds only).</p>
      </div>
    );
  }

  const m = metrics;
  // Counts are integers live, but averaged floats in a loaded snapshot — round for display.
  const distinct = Math.round(m.distinctCount);
  const dupes = Math.round(m.duplicateCount);
  const total = distinct + dupes;
  const dupePct = total > 0 ? Math.round((dupes / total) * 100) : 0;
  const stalls = Math.round(m.stallFrames);
  const frames = Math.round(m.totalFrames);
  const stallPct = frames > 0 ? Math.round((stalls / frames) * 100) : 0;

  return (
    <div className="cockpit-panel render-pipeline-panel">
      <div className="cockpit-panel-title">
        Render Pipeline{suffix} <span className="badge-tag">frontend</span>
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
        label="Stall frames"
        value={`${stalls}/${frames} (${stallPct}%)`}
        color={pick(stallPct, 2, 10, true)}
        blurb="Render frames frozen at the end of a tween, waiting for the next snapshot."
        measures="Render frames where alpha was pinned at 1.0 (nowhere left to interpolate)."
        healthy="~0% — new data arrives before the tween runs out."
        bug="High % = freeze-then-jump: the snapshot arrived late, so the creature sat still then teleported."
      />

      <MetricRow
        label="Duplicate frames"
        value={`${dupePct}% (${dupes}/${total})`}
        color={COLORS.neutral}
        blurb="Buffers delivered with no new data. With event-driven push-on-swap this should be ~0."
        measures="Deliveries carrying positions identical to the previous frame."
        healthy="~0% — each buffer swap fires the doorbell exactly once."
        bug="Sustained non-zero means duplicate sends crept back in, feeding the snapshot-gap jitter."
      />

      <MetricRow
        label="Delivery interval"
        value={`${m.deliveryMeanMs.toFixed(0)} ms`}
        color={COLORS.neutral}
        blurb="Cadence of every buffer received — driven by the sim's buffer-swap doorbell (push-on-swap)."
        measures="Mean time between buffer deliveries to the renderer, before change detection."
        healthy="Steady ~50 ms, tracking the 20 Hz sim beat."
        bug="If it drifts well off the sim's 50 ms beat, duplicates and phase jitter grow."
      />

      <MetricRow
        label="Snapshot rate"
        value={`${distinct}/s`}
        color={Math.abs(distinct - 20) <= 2 ? COLORS.success : COLORS.warning}
        blurb="Distinct position frames per second — should match the sim tick rate (20 Hz)."
        measures="Count of changed snapshots observed in the last second."
        healthy="≈20 (the sim tick rate)."
        bug="Below 20 = dropped frames; above 20 = duplicate leakage past the change detector."
      />

      <div className="rm-spark-grid">
        <div className="rm-spark">
          <div className="rm-spark-label">
            Jitter — σ std-dev (ms) · dashed: green 5 = good, red 12 = bad · want flat &amp; below green
          </div>
          <canvas ref={jitterCanvas} className="memory-sparkline" />
        </div>
      </div>
    </div>
  );
};

export default RenderPipelinePanel;
