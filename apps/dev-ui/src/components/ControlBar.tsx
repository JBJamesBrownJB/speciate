import React from 'react';
import type { SystemTimingsSnapshot, MetricsSnapshot } from '../types';

const TARGET_SAMPLES = 90;

interface ControlBarProps {
  isConnected: boolean;
  tick: number;
  creatureCount: number;
  plantCount?: number;
  isSampling: boolean;
  sampleCount: number;
  systemTimings?: SystemTimingsSnapshot;
  loadedSnapshot: MetricsSnapshot | null;
  onRecordSnapshot: () => void;
  onLoadSnapshot: () => void;
  onClearSnapshot: () => void;
}

export const ControlBar: React.FC<ControlBarProps> = ({
  isConnected,
  tick,
  creatureCount,
  plantCount,
  isSampling,
  sampleCount,
  systemTimings,
  loadedSnapshot,
  onRecordSnapshot,
  onLoadSnapshot,
  onClearSnapshot,
}) => {
  return (
    <div className="status-bar">
      <div className="status-indicator">
        <div className={`status-dot ${isConnected ? '' : 'disconnected'}`} />
        <span>{isConnected ? 'Connected' : 'Disconnected'}</span>
      </div>
      <div>
        Tick: {tick} | Creatures: {creatureCount} | Plants: {plantCount ?? 0}
      </div>
      <button
        onClick={onRecordSnapshot}
        disabled={!isConnected || !systemTimings || isSampling}
        style={{ marginLeft: 'auto' }}
      >
        {isSampling ? `Sampling... (${sampleCount}/${TARGET_SAMPLES})` : '📸 Record Snapshot'}
      </button>
      <button
        onClick={onLoadSnapshot}
        disabled={!isConnected}
      >
        📁 Load Snapshot
      </button>
      {loadedSnapshot && (
        <button onClick={onClearSnapshot}>
          ✕ Clear Snapshot
        </button>
      )}
    </div>
  );
};
