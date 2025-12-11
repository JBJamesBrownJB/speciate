import React, { useState, useEffect, useRef } from 'react';
import { COLORS } from '../utils/cockpit';

interface MemorySnapshot {
  timestamp: number;
  rss: number;
  heapTotal: number;
  heapUsed: number;
  external: number;
  arrayBuffers: number;
}

interface Props {
  onTriggerGC?: () => void;
  onTakeHeapSnapshot?: () => Promise<{ success: boolean; path?: string; error?: string }>;
}

const formatBytes = (bytes: number): string => {
  const mb = bytes / (1024 * 1024);
  if (mb >= 1000) {
    return `${(mb / 1024).toFixed(2)} GB`;
  }
  return `${mb.toFixed(1)} MB`;
};

const formatDelta = (bytes: number): string => {
  const sign = bytes >= 0 ? '+' : '';
  const mb = bytes / (1024 * 1024);
  return `${sign}${mb.toFixed(1)} MB`;
};

const renderMiniSparkline = (
  canvas: HTMLCanvasElement,
  history: number[],
  maxHistory: number,
  color: string
): void => {
  const ctx = canvas.getContext('2d');
  if (!ctx || history.length < 2) return;

  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;
  ctx.scale(dpr, dpr);

  const width = rect.width;
  const height = rect.height;

  ctx.clearRect(0, 0, width, height);

  const maxValue = Math.max(...history) * 1.1;
  const xStep = width / (maxHistory - 1);

  ctx.beginPath();
  ctx.lineWidth = 1.5;
  ctx.strokeStyle = color;

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

  ctx.stroke();
};

export const V8HeapProfiler: React.FC<Props> = ({ onTriggerGC, onTakeHeapSnapshot }) => {
  const [currentSnapshot, setCurrentSnapshot] = useState<MemorySnapshot | null>(null);
  const [baselineSnapshot, setBaselineSnapshot] = useState<MemorySnapshot | null>(null);
  const [snapshotStatus, setSnapshotStatus] = useState<string>('');
  const [gcTriggers, setGcTriggers] = useState<number>(0);

  const heapUsedHistory = useRef<number[]>([]);
  const externalHistory = useRef<number[]>([]);
  const arrayBufferHistory = useRef<number[]>([]);

  const heapCanvasRef = useRef<HTMLCanvasElement>(null);
  const externalCanvasRef = useRef<HTMLCanvasElement>(null);
  const arrayBufferCanvasRef = useRef<HTMLCanvasElement>(null);

  const MAX_HISTORY = 120;

  useEffect(() => {
    const handleMemoryUpdate = (snapshot: MemorySnapshot) => {
      setCurrentSnapshot(snapshot);

      if (!baselineSnapshot) {
        setBaselineSnapshot(snapshot);
      }

      const heapUsedMB = snapshot.heapUsed / (1024 * 1024);
      const externalMB = snapshot.external / (1024 * 1024);
      const arrayBufferMB = snapshot.arrayBuffers / (1024 * 1024);

      heapUsedHistory.current.push(heapUsedMB);
      externalHistory.current.push(externalMB);
      arrayBufferHistory.current.push(arrayBufferMB);

      if (heapUsedHistory.current.length > MAX_HISTORY) {
        heapUsedHistory.current.shift();
        externalHistory.current.shift();
        arrayBufferHistory.current.shift();
      }

      if (heapCanvasRef.current) {
        renderMiniSparkline(heapCanvasRef.current, heapUsedHistory.current, MAX_HISTORY, COLORS.primary);
      }
      if (externalCanvasRef.current) {
        renderMiniSparkline(externalCanvasRef.current, externalHistory.current, MAX_HISTORY, COLORS.warning);
      }
      if (arrayBufferCanvasRef.current) {
        renderMiniSparkline(arrayBufferCanvasRef.current, arrayBufferHistory.current, MAX_HISTORY, COLORS.secondary);
      }
    };

    if (window.electron?.onMemoryUpdate) {
      window.electron.onMemoryUpdate(handleMemoryUpdate);
    }

    return () => {
      if (window.electron?.removeMemoryUpdateListener) {
        window.electron.removeMemoryUpdateListener(handleMemoryUpdate);
      }
    };
  }, [baselineSnapshot]);

  const handleTriggerGC = () => {
    if (onTriggerGC) {
      onTriggerGC();
      setGcTriggers(prev => prev + 1);
      setSnapshotStatus('GC triggered, waiting 100ms...');
      setTimeout(() => setSnapshotStatus(''), 2000);
    } else if (window.electron?.triggerGC) {
      window.electron.triggerGC();
      setGcTriggers(prev => prev + 1);
      setSnapshotStatus('GC triggered, waiting 100ms...');
      setTimeout(() => setSnapshotStatus(''), 2000);
    }
  };

  const handleTakeSnapshot = async () => {
    setSnapshotStatus('Taking heap snapshot...');

    try {
      let result;
      if (onTakeHeapSnapshot) {
        result = await onTakeHeapSnapshot();
      } else if (window.electron?.takeHeapSnapshot) {
        result = await window.electron.takeHeapSnapshot();
      } else {
        throw new Error('Heap snapshot not available');
      }

      if (result.success) {
        setSnapshotStatus(`Saved: ${result.path}`);
        setTimeout(() => setSnapshotStatus(''), 5000);
      } else {
        setSnapshotStatus(`Error: ${result.error}`);
        setTimeout(() => setSnapshotStatus(''), 5000);
      }
    } catch (error) {
      setSnapshotStatus(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
      setTimeout(() => setSnapshotStatus(''), 5000);
    }
  };

  const handleResetBaseline = () => {
    if (currentSnapshot) {
      setBaselineSnapshot(currentSnapshot);
      setSnapshotStatus('Baseline reset');
      setTimeout(() => setSnapshotStatus(''), 2000);
    }
  };

  if (!currentSnapshot) {
    return (
      <div className="cockpit-panel">
        <div className="cockpit-panel-title">V8 Heap Profiler</div>
        <div style={{ padding: '12px', color: COLORS.text, opacity: 0.6 }}>
          Waiting for memory data...
          <br />
          <small>(Only available in memory profiling mode)</small>
        </div>
      </div>
    );
  }

  const heapGrowth = baselineSnapshot ? currentSnapshot.heapUsed - baselineSnapshot.heapUsed : 0;
  const externalGrowth = baselineSnapshot ? currentSnapshot.external - baselineSnapshot.external : 0;
  const arrayBufferGrowth = baselineSnapshot ? currentSnapshot.arrayBuffers - baselineSnapshot.arrayBuffers : 0;

  const heapGrowthColor = heapGrowth > 10 * 1024 * 1024 ? COLORS.critical : COLORS.success;
  const externalGrowthColor = externalGrowth > 10 * 1024 * 1024 ? COLORS.critical : COLORS.success;
  const arrayBufferGrowthColor = arrayBufferGrowth > 10 * 1024 * 1024 ? COLORS.critical : COLORS.success;

  return (
    <div className="cockpit-panel">
      <div className="cockpit-panel-title">V8 Heap Profiler</div>

      <div style={{ padding: '8px 12px', fontSize: '11px' }}>
        <div style={{ marginBottom: '12px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
            <span style={{ color: COLORS.text, opacity: 0.8 }}>Heap Used:</span>
            <span style={{ color: COLORS.primary }}>{formatBytes(currentSnapshot.heapUsed)}</span>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
            <span style={{ color: COLORS.text, opacity: 0.8, fontSize: '10px' }}>Growth:</span>
            <span style={{ color: heapGrowthColor, fontSize: '10px' }}>{formatDelta(heapGrowth)}</span>
          </div>
          <canvas ref={heapCanvasRef} style={{ width: '100%', height: '30px', display: 'block' }} />
        </div>

        <div style={{ marginBottom: '12px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
            <span style={{ color: COLORS.text, opacity: 0.8 }}>External:</span>
            <span style={{ color: COLORS.warning }}>{formatBytes(currentSnapshot.external)}</span>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
            <span style={{ color: COLORS.text, opacity: 0.8, fontSize: '10px' }}>Growth:</span>
            <span style={{ color: externalGrowthColor, fontSize: '10px' }}>{formatDelta(externalGrowth)}</span>
          </div>
          <canvas ref={externalCanvasRef} style={{ width: '100%', height: '30px', display: 'block' }} />
        </div>

        <div style={{ marginBottom: '12px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
            <span style={{ color: COLORS.text, opacity: 0.8 }}>ArrayBuffers:</span>
            <span style={{ color: COLORS.secondary }}>{formatBytes(currentSnapshot.arrayBuffers)}</span>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
            <span style={{ color: COLORS.text, opacity: 0.8, fontSize: '10px' }}>Growth:</span>
            <span style={{ color: arrayBufferGrowthColor, fontSize: '10px' }}>{formatDelta(arrayBufferGrowth)}</span>
          </div>
          <canvas ref={arrayBufferCanvasRef} style={{ width: '100%', height: '30px', display: 'block' }} />
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px', marginTop: '12px' }}>
          <button
            onClick={handleTriggerGC}
            style={{
              padding: '6px',
              fontSize: '10px',
              backgroundColor: COLORS.warning,
              color: '#000',
              border: 'none',
              cursor: 'pointer',
              borderRadius: '3px',
            }}
          >
            Trigger GC ({gcTriggers})
          </button>
          <button
            onClick={handleTakeSnapshot}
            style={{
              padding: '6px',
              fontSize: '10px',
              backgroundColor: COLORS.primary,
              color: '#000',
              border: 'none',
              cursor: 'pointer',
              borderRadius: '3px',
            }}
          >
            Heap Snapshot
          </button>
        </div>

        <button
          onClick={handleResetBaseline}
          style={{
            padding: '6px',
            fontSize: '10px',
            backgroundColor: COLORS.secondary,
            color: '#000',
            border: 'none',
            cursor: 'pointer',
            borderRadius: '3px',
            width: '100%',
            marginTop: '8px',
          }}
        >
          Reset Baseline
        </button>

        {snapshotStatus && (
          <div
            style={{
              marginTop: '8px',
              padding: '6px',
              fontSize: '9px',
              backgroundColor: 'rgba(0, 0, 0, 0.3)',
              borderRadius: '3px',
              color: COLORS.text,
              wordBreak: 'break-all',
            }}
          >
            {snapshotStatus}
          </div>
        )}
      </div>
    </div>
  );
};
