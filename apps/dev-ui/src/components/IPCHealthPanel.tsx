/**
 * IPC Health Panel Component
 *
 * Displays IPC channel health metrics:
 * - Channel utilization (0-100%)
 * - Frame drop counter
 */

import React from 'react';
import type { SystemTimingsSnapshot } from '../types';

interface Props {
  timings?: SystemTimingsSnapshot;
}

const formatPercentage = (value: number): string => {
  return `${Math.round(value)}%`;
};

const getPercentageClass = (pct: number): string => {
  if (pct >= 80) return 'danger';
  if (pct >= 60) return 'warning';
  return '';
};

export const IPCHealthPanel: React.FC<Props> = ({ timings }) => {
  if (!timings) {
    return null;
  }

  const channelUtilization = timings.ipcChannelUtilizationPct;
  const frameDrops = timings.ipcFrameDropsTotal;
  const channelFillPercentage = Math.min(Math.max(channelUtilization, 0), 100);
  const channelColorClass = getPercentageClass(channelUtilization);

  const isHealthy = channelUtilization < 80;

  return (
    <div className="section">
      <h2>IPC Channel Health</h2>

      <div className={`ipc-status-banner ${isHealthy ? 'ok' : 'danger'}`}>
        {isHealthy ? (
          <>✓ Channel Healthy: {Math.round(channelUtilization)}%</>
        ) : (
          <>⚠️ High Channel Utilization: {Math.round(channelUtilization)}% (risk of frame drops)
            {frameDrops > 0 && ` • Total drops: ${frameDrops}`}</>
        )}
      </div>

      <div className="ipc-health-row">
        <div className="ipc-health-label">
          <span className="label-text">Channel Utilization</span>
          <span className={`ipc-health-value ${channelColorClass}`}>
            {formatPercentage(channelUtilization)}
          </span>
        </div>
        <div className="progress-bar-container">
          <div
            className={`progress-bar-fill ${channelColorClass}`}
            style={{ width: `${channelFillPercentage}%` }}
          />
        </div>
      </div>

      <p className="info-text">
        Channel shows IPC buffer fullness (bounded queue capacity: 2 frames).
      </p>
    </div>
  );
};
