/**
 * WindowsMetricsPanel
 *
 * Windows-only process telemetry shown where the Linux PMU hardware counters
 * are unavailable: process cycle rate (QueryProcessCycleTime), page-fault rate
 * and working set (GetProcessMemoryInfo). Cycles are "reference cycles"
 * (RDTSC-based), not true core-clock cycles. See the Rust WindowsMetricsSnapshot.
 */

import React, { useRef, useEffect } from 'react';
import { COLORS } from '../utils/cockpit';
import type { WindowsMetrics } from '../types';

interface Props {
  metrics: WindowsMetrics;
}

const formatPerSec = (v: number): string => {
  if (v >= 1e9) return `${(v / 1e9).toFixed(2)} G/s`;
  if (v >= 1e6) return `${(v / 1e6).toFixed(2)} M/s`;
  if (v >= 1e3) return `${(v / 1e3).toFixed(1)} K/s`;
  return `${v.toFixed(0)} /s`;
};

const formatBytes = (bytes: number): string => {
  const mb = bytes / (1024 * 1024);
  return mb >= 1000 ? `${(mb / 1024).toFixed(2)} GB` : `${mb.toFixed(1)} MB`;
};

const MAX_HISTORY = 120;

const renderSparkline = (canvas: HTMLCanvasElement, history: number[]): void => {
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

  const maxValue = Math.max(...history, 1);
  const xStep = width / (MAX_HISTORY - 1);

  ctx.beginPath();
  ctx.lineWidth = 1.5;
  ctx.strokeStyle = COLORS.streaming;
  history.forEach((value, i) => {
    const x = i * xStep;
    const y = height - Math.min(value / maxValue, 1) * height;
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  });
  ctx.stroke();
};

export const WindowsMetricsPanel: React.FC<Props> = ({ metrics }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const historyRef = useRef<number[]>([]);

  useEffect(() => {
    const history = historyRef.current;
    history.push(metrics.processCyclesPerSec);
    if (history.length > MAX_HISTORY) history.shift();
    if (canvasRef.current) renderSparkline(canvasRef.current, history);
  }, [metrics.processCyclesPerSec]);

  return (
    <div className="cockpit-panel">
      <div className="cockpit-panel-title">
        Windows Process <span className="badge-tag">Windows</span>
      </div>
      <div className="memory-metrics-content">
        <div className="memory-value-row">
          <span className="memory-label">Cycles:</span>
          <span className="memory-value" style={{ color: COLORS.streaming }}>
            {formatPerSec(metrics.processCyclesPerSec)}
          </span>
        </div>
        <div className="memory-value-row">
          <span className="memory-label">Page faults:</span>
          <span className="memory-value">{formatPerSec(metrics.pageFaultsPerSec)}</span>
        </div>
        <div className="memory-value-row">
          <span className="memory-label">Working set:</span>
          <span className="memory-value">{formatBytes(metrics.workingSetBytes)}</span>
        </div>
        <canvas ref={canvasRef} className="memory-sparkline" />
        <p className="hint">Reference cycles (RDTSC), not true core-clock cycles.</p>
      </div>
    </div>
  );
};

export default WindowsMetricsPanel;
