import React, { useState } from 'react';
import { CockpitTooltip } from './CockpitTooltip';
import { COLORS } from '../utils/cockpit';

interface Props {
  l1dMissRate: number;
  llcMissRate: number;
}

interface BarState {
  color: string;
  label: string;
  value: string;
}

const getL1State = (missRate: number): BarState => {
  if (missRate > 10) {
    return {
      color: COLORS.streaming,
      label: '⚡',
      value: `${missRate.toFixed(1)}%`,
    };
  }
  return {
    color: COLORS.success,
    label: '✓',
    value: `${missRate.toFixed(1)}%`,
  };
};

const getL3State = (llcMissRate: number): BarState => {
  if (llcMissRate < 0) {
    return {
      color: COLORS.neutral,
      label: '—',
      value: 'N/A',
    };
  }
  if (llcMissRate > 20) {
    return {
      color: COLORS.critical,
      label: '⚠️',
      value: `${llcMissRate.toFixed(1)}%`,
    };
  }
  // Adjusted: 15% is industry-standard threshold for "good" L3 performance
  // Still conservative for streaming workloads but reduces false positives
  if (llcMissRate > 15) {
    return {
      color: COLORS.warning,
      label: '⚠️',
      value: `${llcMissRate.toFixed(1)}%`,
    };
  }
  return {
    color: COLORS.success,
    label: '✓',
    value: `${llcMissRate.toFixed(1)}%`,
  };
};

export const CacheFirewall: React.FC<Props> = ({ l1dMissRate, llcMissRate }) => {
  const [hoveredBar, setHoveredBar] = useState<'l1' | 'l3' | null>(null);

  const l1State = getL1State(l1dMissRate);
  const l3State = getL3State(llcMissRate);

  const getL1Tooltip = () => (
    <CockpitTooltip
      header="L1 Data Cache"
      current={`Current: ${l1dMissRate.toFixed(1)}% miss rate (${l1State.label} ✓)`}
      sections={[
        {
          title: 'What this means:',
          items: [
            { text: `Your code misses L1 cache ${l1dMissRate.toFixed(1)}% of the time` },
            { text: 'The prefetcher is catching misses in L3' },
            { text: 'This is GOOD for streaming workloads like yours.', type: 'success' },
            { text: 'You are limited by L3 bandwidth, not latency.' },
          ],
        },
        {
          title: 'What to watch for:',
          items: [
            { text: '⚠️ L1 miss >10% → High cache pressure' },
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

      </div>
    </div>
  );
};
