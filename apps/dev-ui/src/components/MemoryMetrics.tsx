import React, { useRef, useEffect } from 'react';
import { COLORS } from '../utils/cockpit';

interface Props {
  processMemoryBytes: number;
}

interface SparklineData {
  history: number[];
  maxHistory: number;
  peakValue: number;
}

const WARNING_THRESHOLD_MB = 500;
const DANGER_THRESHOLD_MB = 1000;

const bytesToMB = (bytes: number): number => bytes / (1024 * 1024);

const formatMemory = (bytes: number): string => {
  const mb = bytesToMB(bytes);
  if (mb >= 1000) {
    return `${(mb / 1024).toFixed(2)} GB`;
  }
  return `${mb.toFixed(1)} MB`;
};

const getMemoryColor = (mb: number): string => {
  if (mb >= DANGER_THRESHOLD_MB) return COLORS.critical;
  if (mb >= WARNING_THRESHOLD_MB) return COLORS.warning;
  return COLORS.success;
};

const renderSparkline = (
  canvas: HTMLCanvasElement,
  history: number[],
  maxHistory: number,
  peakValue: number
): void => {
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;
  ctx.scale(dpr, dpr);

  const width = rect.width;
  const height = rect.height;

  ctx.clearRect(0, 0, width, height);

  if (history.length < 2) return;

  const maxValue = Math.max(DANGER_THRESHOLD_MB, peakValue * 1.1);
  const xStep = width / (maxHistory - 1);
  // Right-align: newest sample on right, empty space on left while window fills
  const startX = (maxHistory - history.length) * xStep;

  ctx.beginPath();
  ctx.lineWidth = 1.5;

  history.forEach((valueMB, i) => {
    const x = startX + i * xStep;
    const normalizedValue = Math.min(valueMB / maxValue, 1);
    const y = height - normalizedValue * height;

    if (i === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  });

  const lastValue = history[history.length - 1];
  ctx.strokeStyle = getMemoryColor(lastValue);
  ctx.stroke();

  // Window-start indicator: vertical dashed line at oldest data point.
  // Moves left as the buffer fills; disappears when window is complete.
  if (startX > 0) {
    ctx.beginPath();
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.25)';
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 3]);
    ctx.moveTo(startX, 0);
    ctx.lineTo(startX, height);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  ctx.beginPath();
  ctx.strokeStyle = 'rgba(217, 72, 72, 0.3)';
  ctx.lineWidth = 1;
  ctx.setLineDash([2, 2]);
  const thresholdY = height - (DANGER_THRESHOLD_MB / maxValue) * height;
  ctx.moveTo(0, thresholdY);
  ctx.lineTo(width, thresholdY);
  ctx.stroke();
  ctx.setLineDash([]);

  const peakY = height - (peakValue / maxValue) * height;
  ctx.beginPath();
  ctx.strokeStyle = 'rgba(100, 149, 237, 0.5)';
  ctx.lineWidth = 1;
  ctx.setLineDash([4, 2]);
  ctx.moveTo(0, peakY);
  ctx.lineTo(width, peakY);
  ctx.stroke();
  ctx.setLineDash([]);
};

export const MemoryMetrics: React.FC<Props> = ({ processMemoryBytes }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const historyRef = useRef<SparklineData>({ history: [], maxHistory: 40, peakValue: 0 });

  useEffect(() => {
    const memoryMB = bytesToMB(processMemoryBytes);
    const data = historyRef.current;

    data.history.push(memoryMB);
    if (data.history.length > data.maxHistory) {
      data.history.shift();
    }

    if (memoryMB > data.peakValue) {
      data.peakValue = memoryMB;
    }

    const canvas = canvasRef.current;
    if (canvas) {
      renderSparkline(canvas, data.history, data.maxHistory, data.peakValue);
    }
  }, [processMemoryBytes]);

  const memoryMB = bytesToMB(processMemoryBytes);
  const peakMB = historyRef.current.peakValue;

  return (
    <div className="cockpit-panel">
      <div className="cockpit-panel-title">Process Memory</div>
      <div className="memory-metrics-content">
        <div className="memory-value-row">
          <span className="memory-label">Current:</span>
          <span
            className="memory-value"
            style={{ color: getMemoryColor(memoryMB) }}
          >
            {formatMemory(processMemoryBytes)}
          </span>
        </div>
        <div className="memory-value-row">
          <span className="memory-label">Peak:</span>
          <span className="memory-value memory-peak">
            {formatMemory(peakMB * 1024 * 1024)}
          </span>
        </div>
        <canvas ref={canvasRef} className="memory-sparkline" />
      </div>
    </div>
  );
};
