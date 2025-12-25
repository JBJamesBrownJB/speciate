import React, { useRef, useEffect, useState } from 'react';
import type { SystemTimingsSnapshot } from '../types';

interface Props {
  timings?: SystemTimingsSnapshot;
}

interface SparklineData {
  history: number[];
  maxHistory: number;
}

const WARNING_THRESHOLD_US = 20000;
const DANGER_THRESHOLD_US = 50000;

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

const formatCount = (value: number): string => {
  if (value >= 1000000) {
    return `${(value / 1000000).toFixed(1)}M`;
  }
  if (value >= 1000) {
    return `${(value / 1000).toFixed(1)}K`;
  }
  return value.toFixed(0);
};

const renderCountSparkline = (
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

  const maxValue = Math.max(...history, 1);
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

  ctx.strokeStyle = '#5c9fd4'; // Blue for count metrics
  ctx.stroke();
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

interface CountRowProps {
  name: string;
  value: number;
  canvasRef: React.RefObject<HTMLCanvasElement | null>;
}

const CountRow: React.FC<CountRowProps> = ({ name, value, canvasRef }) => (
  <div className="timing-row">
    <div className="timing-header">
      <span className="timing-name">{name}</span>
      <span className="timing-value count-value">
        {formatCount(value)}
      </span>
    </div>
    <canvas ref={canvasRef as React.RefObject<HTMLCanvasElement>} className="timing-sparkline" />
  </div>
);

// Reserved metrics that should always appear at the top
const CRITICAL_METRICS = ['totalTickUs'];

// Non-timing metrics that should be excluded from timing sparklines
const NON_TIMING_METRICS = ['archetypeCount', 'entityCount', 'cellsQueriedTotal'];

// Count metrics that get their own sparkline section
const COUNT_METRICS = ['cellsQueriedTotal'];

// Systems with frequency control (maps timing key to system name for IPC)
// Note: steering throttling removed - will revisit after drive-simplex (Phase B)
const FREQUENCY_CONTROLLABLE: Record<string, string> = {
  perceptionUs: 'perception',
  behaviorTransitionUs: 'behavior',
};

// Convert camelCase to snake_case for display
const toSnakeCase = (str: string): string => {
  return str.replace(/([A-Z])/g, '_$1').toLowerCase().replace(/^_/, '');
};

interface FrequencySliderProps {
  systemName: string;
  divisor: number;
  onChange: (divisor: number) => void;
}

const FrequencySlider: React.FC<FrequencySliderProps> = ({ systemName, divisor, onChange }) => (
  <div className="frequency-slider">
    <input
      type="range"
      min="1"
      max="10"
      value={divisor}
      onChange={(e) => onChange(Number(e.target.value))}
      title={`${systemName} frequency divisor: ÷${divisor}`}
    />
    <span className="divisor-label">÷{divisor}</span>
  </div>
);

export const SystemTimingsPanel: React.FC<Props> = ({ timings }) => {
  // Dynamic refs and history storage
  const canvasRefs = useRef<Record<string, React.RefObject<HTMLCanvasElement>>>({});
  const historyRefs = useRef<Record<string, SparklineData>>({});
  const averageRefs = useRef<Record<string, number>>({});
  const [sortedKeys, setSortedKeys] = useState<string[]>([]);

  // Frequency divisor state for controllable systems
  const [divisors, setDivisors] = useState<Record<string, number>>({
    perception: 1,
    behavior: 1,
  });

  const handleDivisorChange = (systemName: string, divisor: number) => {
    setDivisors(prev => ({ ...prev, [systemName]: divisor }));
    window.electron?.setSystemFrequency?.(systemName, divisor);
  };

  // Rolling average window: 1 second at ~30Hz = ~30 frames
  const ROLLING_WINDOW_FRAMES = 30;

  useEffect(() => {
    if (!timings) return;

    // Dynamically create refs and history for timing metrics
    const allKeys = Object.keys(timings).filter(key =>
      typeof timings[key as keyof SystemTimingsSnapshot] === 'number' &&
      !NON_TIMING_METRICS.includes(key)
    );

    allKeys.forEach(key => {
      if (!canvasRefs.current[key]) {
        canvasRefs.current[key] = React.createRef<HTMLCanvasElement>();
      }
      if (!historyRefs.current[key]) {
        historyRefs.current[key] = { history: [], maxHistory: 120 };
      }

      // Update history (for sparkline)
      const value = timings[key as keyof SystemTimingsSnapshot] as number;
      const history = historyRefs.current[key];
      history.history.push(value);
      if (history.history.length > history.maxHistory) {
        history.history.shift();
      }

      // Calculate rolling average for display (last 1 second)
      const rollingWindow = history.history.slice(-ROLLING_WINDOW_FRAMES);
      const average = rollingWindow.reduce((sum, v) => sum + v, 0) / rollingWindow.length;
      averageRefs.current[key] = average;

      // Render sparkline
      const canvas = canvasRefs.current[key]?.current;
      if (canvas) {
        renderSparkline(canvas, history.history, history.maxHistory);
      }
    });

    // Process count metrics separately (different sparkline renderer)
    COUNT_METRICS.forEach(key => {
      if (!(key in timings)) return;

      if (!canvasRefs.current[key]) {
        canvasRefs.current[key] = React.createRef<HTMLCanvasElement>();
      }
      if (!historyRefs.current[key]) {
        historyRefs.current[key] = { history: [], maxHistory: 120 };
      }

      const value = timings[key as keyof SystemTimingsSnapshot] as number;
      const history = historyRefs.current[key];
      history.history.push(value);
      if (history.history.length > history.maxHistory) {
        history.history.shift();
      }

      const rollingWindow = history.history.slice(-ROLLING_WINDOW_FRAMES);
      const average = rollingWindow.reduce((sum, v) => sum + v, 0) / rollingWindow.length;
      averageRefs.current[key] = average;

      const canvas = canvasRefs.current[key]?.current;
      if (canvas) {
        renderCountSparkline(canvas, history.history, history.maxHistory);
      }
    });

    // Initial sort if not yet sorted
    if (sortedKeys.length === 0) {
      const nonCritical = allKeys.filter(k => !CRITICAL_METRICS.includes(k));
      setSortedKeys(nonCritical);
    }
  }, [timings, sortedKeys.length]);

  if (!timings) {
    return (
      <div className="section">
        <h2>System Timings</h2>
        <p className="muted">Waiting for timing data...</p>
      </div>
    );
  }

  const handleSort = () => {
    const allKeys = Object.keys(timings).filter(key =>
      typeof timings[key as keyof SystemTimingsSnapshot] === 'number' &&
      !NON_TIMING_METRICS.includes(key)
    );
    const nonCritical = allKeys.filter(k => !CRITICAL_METRICS.includes(k));

    const sorted = nonCritical.sort((a, b) => {
      // Sort by averaged values for more stable ordering
      const aValue = averageRefs.current[a] || (timings[a as keyof SystemTimingsSnapshot] as number);
      const bValue = averageRefs.current[b] || (timings[b as keyof SystemTimingsSnapshot] as number);
      return bValue - aValue;
    });

    setSortedKeys(sorted);
  };

  // Critical metrics (always at top)
  const criticalMetrics = CRITICAL_METRICS
    .filter(key => key in timings)
    .map(key => ({
      key,
      name: toSnakeCase(key),
      valueUs: averageRefs.current[key] || (timings[key as keyof SystemTimingsSnapshot] as number),
      canvasRef: canvasRefs.current[key],
    }));

  // All other metrics (sorted or default order)
  const otherMetrics = sortedKeys
    .filter(key => key in timings)
    .map(key => ({
      key,
      name: toSnakeCase(key),
      valueUs: averageRefs.current[key] || (timings[key as keyof SystemTimingsSnapshot] as number),
      canvasRef: canvasRefs.current[key],
      systemName: FREQUENCY_CONTROLLABLE[key] || null,
    }));

  // Count metrics (cells queried, etc.)
  const countMetrics = COUNT_METRICS
    .filter(key => key in timings)
    .map(key => ({
      key,
      name: toSnakeCase(key),
      value: averageRefs.current[key] || (timings[key as keyof SystemTimingsSnapshot] as number),
      canvasRef: canvasRefs.current[key],
    }));

  return (
    <div className="section">
      <h2>Critical Timings</h2>
      <div className="critical-timings-grid">
        {criticalMetrics.map((entry) => (
          <TimingRow
            key={entry.key}
            name={entry.name}
            valueUs={entry.valueUs}
            canvasRef={entry.canvasRef}
          />
        ))}
      </div>

      <div className="section-header" style={{ marginTop: '24px' }}>
        <h2>Detailed System Timings</h2>
        <button onClick={handleSort} className="sort-button">
          Sort
        </button>
      </div>
      <div className="timings-grid">
        {otherMetrics.map((entry) => (
          <div key={entry.key} className="timing-row-container">
            <TimingRow
              name={entry.name}
              valueUs={entry.valueUs}
              canvasRef={entry.canvasRef}
            />
            {entry.systemName && (
              <FrequencySlider
                systemName={entry.systemName}
                divisor={divisors[entry.systemName]}
                onChange={(divisor) => handleDivisorChange(entry.systemName!, divisor)}
              />
            )}
          </div>
        ))}
      </div>

      {countMetrics.length > 0 && (
        <>
          <h2 style={{ marginTop: '24px' }}>Spatial Metrics</h2>
          <div className="timings-grid">
            {countMetrics.map((entry) => (
              <CountRow
                key={entry.key}
                name={entry.name}
                value={entry.value}
                canvasRef={entry.canvasRef}
              />
            ))}
          </div>
        </>
      )}
    </div>
  );
};
