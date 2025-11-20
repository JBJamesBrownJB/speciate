import React, { useState } from 'react';
import { CockpitTooltip } from './CockpitTooltip';
import { COLORS } from '../utils/cockpit';

interface Props {
  l1dMissRate: number;
  llcMissRate: number;
  backendStallRatio: number;
}

interface BarState {
  color: string;
  label: string;
  value: string;
}

const getL1State = (missRate: number, backendStall: number): BarState => {
  if (missRate > 10 && backendStall > 5) {
    return {
      color: COLORS.critical,
      label: '⚠️ Thrashing',
      value: `${missRate.toFixed(1)}%`,
    };
  }
  if (missRate > 10 && backendStall < 5) {
    return {
      color: COLORS.streaming,
      label: 'Streaming',
      value: `${missRate.toFixed(1)}%`,
    };
  }
  return {
    color: COLORS.success,
    label: 'Perfect Locality',
    value: `${missRate.toFixed(1)}%`,
  };
};

const getL3State = (llcMissRate: number): BarState => {
  if (llcMissRate < 0) {
    return {
      color: COLORS.neutral,
      label: 'N/A',
      value: 'N/A',
    };
  }
  if (llcMissRate > 20) {
    return {
      color: COLORS.critical,
      label: '⚠️ RAM Bound',
      value: `${llcMissRate.toFixed(1)}%`,
    };
  }
  if (llcMissRate > 10) {
    return {
      color: COLORS.warning,
      label: 'Stressed',
      value: `${llcMissRate.toFixed(1)}%`,
    };
  }
  return {
    color: COLORS.success,
    label: 'Catching',
    value: `${llcMissRate.toFixed(1)}%`,
  };
};

const getRAMState = (backendStall: number): BarState => {
  if (backendStall > 20) {
    return {
      color: COLORS.critical,
      label: '⚠️ Blocked',
      value: `${backendStall.toFixed(1)}%`,
    };
  }
  if (backendStall > 5) {
    return {
      color: COLORS.warning,
      label: 'Waiting',
      value: `${backendStall.toFixed(1)}%`,
    };
  }
  return {
    color: COLORS.neutral,
    label: 'No Wait',
    value: `${backendStall.toFixed(1)}%`,
  };
};

export const CacheFirewall: React.FC<Props> = ({ l1dMissRate, llcMissRate, backendStallRatio }) => {
  const [hoveredBar, setHoveredBar] = useState<'l1' | 'l3' | 'ram' | null>(null);

  const l1State = getL1State(l1dMissRate, backendStallRatio);
  const l3State = getL3State(llcMissRate);
  const ramState = getRAMState(backendStallRatio);

  const getL1Tooltip = () => (
    <CockpitTooltip
      header="L1 Data Cache"
      current={`Current: ${l1dMissRate.toFixed(1)}% miss rate (${l1State.label} ✓)`}
      sections={[
        {
          title: 'What this means:',
          items: [
            { text: `Your code misses L1 cache ${l1dMissRate.toFixed(1)}% of the time` },
            { text: `BUT: Backend stalls are only ${backendStallRatio.toFixed(1)}%` },
            { text: 'This means the prefetcher is catching misses in L3' },
            { text: 'This is GOOD for streaming workloads like yours.', type: 'success' },
            { text: 'You are limited by L3 bandwidth, not latency.' },
          ],
        },
        {
          title: 'What to watch for:',
          items: [
            { text: '⚠️ L1 miss >10% + Backend stall >5% → Cache thrashing' },
            { text: '→ Implement spatial grid for perception system', indent: true },
          ],
        },
      ]}
      target="Current diagnosis: Optimal streaming performance"
    />
  );

  const getL3Tooltip = () => (
    <CockpitTooltip
      header="L3 (Last-Level Cache)"
      current={`Current: ${l3State.label} ✓`}
      sections={[
        {
          title: 'What this means:',
          items: [
            { text: 'L3 is successfully catching what L1 misses' },
            { text: 'Data is being prefetched from RAM efficiently' },
            { text: 'No memory bottleneck' },
          ],
        },
        {
          title: 'What to watch for:',
          items: [
            { text: '⚠️ LLC miss rate >20% → Going to RAM too often' },
            { text: '→ Reduce data per entity', indent: true },
            { text: '→ Consider structure-of-arrays layout', indent: true },
          ],
        },
      ]}
      target="Current diagnosis: L3 working optimally"
    />
  );

  const getRAMTooltip = () => (
    <CockpitTooltip
      header="RAM Stall (Backend Pipeline)"
      current={`Current: ${ramState.label} ✓`}
      sections={[
        {
          title: 'What this means:',
          items: [
            { text: 'Backend pipeline not stalled on memory' },
            { text: 'Memory bandwidth is sufficient' },
            { text: 'Prefetcher is keeping up' },
          ],
        },
        {
          title: 'What to watch for:',
          items: [
            { text: '⚠️ Backend stalls >20% → Memory bottleneck' },
            { text: '→ Reduce memory bandwidth usage', indent: true },
            { text: '→ Compress component data', indent: true },
          ],
        },
      ]}
      target="Current diagnosis: Memory bandwidth sufficient"
    />
  );

  return (
    <div className="cockpit-panel">
      <div className="cockpit-panel-title">Memory</div>
      <div className="cache-bars-container">
        <div
          className="cache-bar-wrapper"
          onMouseEnter={() => setHoveredBar('l1')}
          onMouseLeave={() => setHoveredBar(null)}
        >
          <div className="cache-bar-background">
            <div
              className="cache-bar-fill"
              style={{
                height: `${Math.max(0, Math.min(100, l1dMissRate))}%`,
                backgroundColor: l1State.color,
              }}
            >
            </div>
          </div>
          <div className="cache-bar-label">L1</div>
          <div className="cache-bar-status" style={{ color: l1State.color }}>
            {l1State.label}
          </div>
          {hoveredBar === 'l1' && getL1Tooltip()}
        </div>

        <div
          className="cache-bar-wrapper"
          onMouseEnter={() => setHoveredBar('l3')}
          onMouseLeave={() => setHoveredBar(null)}
        >
          <div className="cache-bar-background">
            <div
              className="cache-bar-fill"
              style={{
                height: `${Math.max(0, Math.min(100, llcMissRate))}%`,
                backgroundColor: l3State.color,
              }}
            >
            </div>
          </div>
          <div className="cache-bar-label">L3</div>
          <div className="cache-bar-status" style={{ color: l3State.color }}>
            {l3State.label}
          </div>
          {hoveredBar === 'l3' && getL3Tooltip()}
        </div>

        <div
          className="cache-bar-wrapper"
          onMouseEnter={() => setHoveredBar('ram')}
          onMouseLeave={() => setHoveredBar(null)}
        >
          <div className="cache-bar-background">
            <div
              className="cache-bar-fill"
              style={{
                height: `${Math.max(0, Math.min(100, backendStallRatio))}%`,
                backgroundColor: ramState.color,
              }}
            >
            </div>
          </div>
          <div className="cache-bar-label">RAM Stall</div>
          <div className="cache-bar-status" style={{ color: ramState.color }}>
            {ramState.label}
          </div>
          {hoveredBar === 'ram' && getRAMTooltip()}
        </div>
      </div>
    </div>
  );
};
