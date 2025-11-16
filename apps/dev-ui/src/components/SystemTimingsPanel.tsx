import React, { useRef, useEffect, useState } from 'react';
import type { SystemTimingsSnapshot } from '../types';

interface Props {
  timings?: SystemTimingsSnapshot;
}

interface SparklineData {
  history: number[];
  maxHistory: number;
}

const WARNING_THRESHOLD_US = 5000;
const DANGER_THRESHOLD_US = 10000;

const renderSparkline = (
  canvas: HTMLCanvasElement,
  history: number[],
  maxHistory: number
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

  const maxValue = Math.max(DANGER_THRESHOLD_US, ...history);
  const xStep = width / (maxHistory - 1);

  ctx.beginPath();
  ctx.lineWidth = 1.5;

  history.forEach((value, i) => {
    const x = i * xStep;
    const normalizedValue = Math.min(value / maxValue, 1);
    const y = height - normalizedValue * height;

    if (i === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  });

  const lastValue = history[history.length - 1];
  const strokeColor =
    lastValue >= DANGER_THRESHOLD_US
      ? '#d94848'
      : lastValue >= WARNING_THRESHOLD_US
      ? '#f0a830'
      : '#6fb83f';

  ctx.strokeStyle = strokeColor;
  ctx.stroke();

  ctx.beginPath();
  ctx.strokeStyle = 'rgba(217, 72, 72, 0.3)';
  ctx.lineWidth = 1;
  ctx.setLineDash([2, 2]);
  const thresholdY = height - (DANGER_THRESHOLD_US / maxValue) * height;
  ctx.moveTo(0, thresholdY);
  ctx.lineTo(width, thresholdY);
  ctx.stroke();
  ctx.setLineDash([]);
};

const formatTiming = (valueUs: number): string => {
  const valueMs = valueUs / 1000;
  return `${valueMs.toFixed(2)} ms`;
};

const getTimingClass = (valueUs: number): string => {
  if (valueUs >= DANGER_THRESHOLD_US) return 'danger';
  if (valueUs >= WARNING_THRESHOLD_US) return 'warning';
  return '';
};

interface TimingRowProps {
  name: string;
  valueUs: number;
  canvasRef: React.RefObject<HTMLCanvasElement | null>;
}

const TimingRow: React.FC<TimingRowProps> = ({ name, valueUs, canvasRef }) => (
  <div className="timing-row">
    <div className="timing-header">
      <span className="timing-name">{name}</span>
      <span className={`timing-value ${getTimingClass(valueUs)}`}>
        {formatTiming(valueUs)}
      </span>
    </div>
    <canvas ref={canvasRef as React.RefObject<HTMLCanvasElement>} className="timing-sparkline" />
  </div>
);

export const SystemTimingsPanel: React.FC<Props> = ({ timings }) => {
  const totalTickCanvasRef = useRef<HTMLCanvasElement>(null);
  const perceptionCanvasRef = useRef<HTMLCanvasElement>(null);
  const behaviorTransitionCanvasRef = useRef<HTMLCanvasElement>(null);
  const wanderCanvasRef = useRef<HTMLCanvasElement>(null);
  const fleeCanvasRef = useRef<HTMLCanvasElement>(null);
  const behaviorCanvasRef = useRef<HTMLCanvasElement>(null);
  const avoidanceCanvasRef = useRef<HTMLCanvasElement>(null);
  const movementCanvasRef = useRef<HTMLCanvasElement>(null);
  const rotationCanvasRef = useRef<HTMLCanvasElement>(null);

  const totalTickHistoryRef = useRef<SparklineData>({ history: [], maxHistory: 120 });
  const perceptionHistoryRef = useRef<SparklineData>({ history: [], maxHistory: 120 });
  const behaviorTransitionHistoryRef = useRef<SparklineData>({ history: [], maxHistory: 120 });
  const wanderHistoryRef = useRef<SparklineData>({ history: [], maxHistory: 120 });
  const fleeHistoryRef = useRef<SparklineData>({ history: [], maxHistory: 120 });
  const behaviorHistoryRef = useRef<SparklineData>({ history: [], maxHistory: 120 });
  const avoidanceHistoryRef = useRef<SparklineData>({ history: [], maxHistory: 120 });
  const movementHistoryRef = useRef<SparklineData>({ history: [], maxHistory: 120 });
  const rotationHistoryRef = useRef<SparklineData>({ history: [], maxHistory: 120 });

  const [sortOrder, setSortOrder] = useState<string[]>([
    'total_tick',
    'perception',
    'behavior_transition',
    'wander',
    'flee',
    'seek',
    'avoidance',
    'movement',
    'rotation',
  ]);

  useEffect(() => {
    if (!timings) return;

    const updateHistory = (data: SparklineData, value: number) => {
      data.history.push(value);
      if (data.history.length > data.maxHistory) {
        data.history.shift();
      }
    };

    updateHistory(totalTickHistoryRef.current, timings.totalTickUs);
    updateHistory(perceptionHistoryRef.current, timings.perceptionUs);
    updateHistory(behaviorTransitionHistoryRef.current, timings.behaviorTransitionUs);
    updateHistory(wanderHistoryRef.current, timings.wanderUs);
    updateHistory(fleeHistoryRef.current, timings.fleeUs);
    updateHistory(behaviorHistoryRef.current, timings.behaviorUs);
    updateHistory(avoidanceHistoryRef.current, timings.avoidanceUs);
    updateHistory(movementHistoryRef.current, timings.movementUs);
    updateHistory(rotationHistoryRef.current, timings.rotationUs);

    if (totalTickCanvasRef.current) {
      renderSparkline(
        totalTickCanvasRef.current,
        totalTickHistoryRef.current.history,
        totalTickHistoryRef.current.maxHistory
      );
    }
    if (perceptionCanvasRef.current) {
      renderSparkline(
        perceptionCanvasRef.current,
        perceptionHistoryRef.current.history,
        perceptionHistoryRef.current.maxHistory
      );
    }
    if (behaviorTransitionCanvasRef.current) {
      renderSparkline(
        behaviorTransitionCanvasRef.current,
        behaviorTransitionHistoryRef.current.history,
        behaviorTransitionHistoryRef.current.maxHistory
      );
    }
    if (wanderCanvasRef.current) {
      renderSparkline(
        wanderCanvasRef.current,
        wanderHistoryRef.current.history,
        wanderHistoryRef.current.maxHistory
      );
    }
    if (fleeCanvasRef.current) {
      renderSparkline(
        fleeCanvasRef.current,
        fleeHistoryRef.current.history,
        fleeHistoryRef.current.maxHistory
      );
    }
    if (behaviorCanvasRef.current) {
      renderSparkline(
        behaviorCanvasRef.current,
        behaviorHistoryRef.current.history,
        behaviorHistoryRef.current.maxHistory
      );
    }
    if (avoidanceCanvasRef.current) {
      renderSparkline(
        avoidanceCanvasRef.current,
        avoidanceHistoryRef.current.history,
        avoidanceHistoryRef.current.maxHistory
      );
    }
    if (movementCanvasRef.current) {
      renderSparkline(
        movementCanvasRef.current,
        movementHistoryRef.current.history,
        movementHistoryRef.current.maxHistory
      );
    }
    if (rotationCanvasRef.current) {
      renderSparkline(
        rotationCanvasRef.current,
        rotationHistoryRef.current.history,
        rotationHistoryRef.current.maxHistory
      );
    }
  }, [timings]);

  if (!timings) {
    return (
      <div className="section">
        <h2>System Timings</h2>
        <p className="muted">Waiting for timing data...</p>
      </div>
    );
  }

  const timingEntriesMap: Record<string, { name: string; valueUs: number; canvasRef: React.RefObject<HTMLCanvasElement | null> }> = {
    'total_tick': { name: 'total_tick', valueUs: timings.totalTickUs, canvasRef: totalTickCanvasRef },
    'perception': { name: 'perception', valueUs: timings.perceptionUs, canvasRef: perceptionCanvasRef },
    'behavior_transition': { name: 'behavior_transition', valueUs: timings.behaviorTransitionUs, canvasRef: behaviorTransitionCanvasRef },
    'wander': { name: 'wander', valueUs: timings.wanderUs, canvasRef: wanderCanvasRef },
    'flee': { name: 'flee', valueUs: timings.fleeUs, canvasRef: fleeCanvasRef },
    'seek': { name: 'seek', valueUs: timings.behaviorUs, canvasRef: behaviorCanvasRef },
    'avoidance': { name: 'avoidance', valueUs: timings.avoidanceUs, canvasRef: avoidanceCanvasRef },
    'movement': { name: 'movement', valueUs: timings.movementUs, canvasRef: movementCanvasRef },
    'rotation': { name: 'rotation', valueUs: timings.rotationUs, canvasRef: rotationCanvasRef },
  };

  const handleSort = () => {
    const sorted = Object.values(timingEntriesMap)
      .sort((a, b) => b.valueUs - a.valueUs)
      .map((entry) => entry.name);
    setSortOrder(sorted);
  };

  const timingEntries = sortOrder.map((name) => timingEntriesMap[name]);

  return (
    <div className="section">
      <div className="section-header">
        <h2>System Timings</h2>
        <button onClick={handleSort} className="sort-button">
          Sort
        </button>
      </div>
      <div className="timings-grid">
        {timingEntries.map((entry) => (
          <TimingRow
            key={entry.name}
            name={entry.name}
            valueUs={entry.valueUs}
            canvasRef={entry.canvasRef}
          />
        ))}
      </div>
    </div>
  );
};
