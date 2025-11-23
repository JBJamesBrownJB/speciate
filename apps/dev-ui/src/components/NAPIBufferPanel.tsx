import { TelemetryFrame } from '../types';

interface NAPIBufferPanelProps {
  telemetry: TelemetryFrame | null;
}

export function NAPIBufferPanel({ telemetry }: NAPIBufferPanelProps) {
  if (!telemetry ||
      telemetry.napiBufferCapacityPct === undefined ||
      telemetry.napiBufferUsed === undefined ||
      telemetry.napiBufferCapacity === undefined) {
    return null;
  }

  const utilizationPct = telemetry.napiBufferCapacityPct;
  const used = telemetry.napiBufferUsed;
  const capacity = telemetry.napiBufferCapacity;

  const getColorClass = (pct: number): string => {
    if (pct >= 80) return 'danger';
    if (pct >= 60) return 'warning';
    return '';
  };

  const colorClass = getColorClass(utilizationPct);
  const isHealthy = utilizationPct < 80;

  return (
    <div style={{ marginBottom: '16px' }}>
      <div style={{
        padding: '8px 12px',
        background: isHealthy ? '#1e3a1e' : '#4a2020',
        borderRadius: '4px',
        marginBottom: '8px',
        border: isHealthy ? '1px solid #3e6e3e' : '1px solid #8e4e4e',
      }}>
        {isHealthy ? (
          <span style={{ color: '#9adb9a' }}>
            ✓ NAPI Buffer Healthy: {utilizationPct}%
          </span>
        ) : (
          <span style={{ color: '#f88' }}>
            ⚠️ High Buffer Usage: {utilizationPct}%
          </span>
        )}
      </div>

      <div style={{ marginBottom: '4px', fontSize: '13px', color: '#ccc' }}>
        <span style={{ fontWeight: 500 }}>Buffer Utilization</span>
        <span style={{
          float: 'right',
          color: colorClass === 'danger' ? '#d94848' : colorClass === 'warning' ? '#f0a830' : '#6fb83f',
          fontWeight: 600,
        }}>
          {used.toLocaleString()} / {capacity.toLocaleString()} ({utilizationPct}%)
        </span>
      </div>

      <div className="progress-bar-container">
        <div
          className={`progress-bar-fill ${colorClass}`}
          style={{ width: `${utilizationPct}%` }}
        />
      </div>
    </div>
  );
}
