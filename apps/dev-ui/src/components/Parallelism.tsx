import React from 'react';
import { COLORS } from '../utils/cockpit';
import type { ParallelizationMetrics, SystemTimingsSnapshot } from '../types';

interface Props {
  metrics: ParallelizationMetrics;
  systemTimings: SystemTimingsSnapshot;
}

interface SystemInfo {
  name: string;
  timeUs: number;
  isActive: boolean;
}

const extractSystemsFromTimings = (timings: SystemTimingsSnapshot): SystemInfo[] => {
  const systems: SystemInfo[] = [
    { name: 'perception', timeUs: timings.perceptionUs, isActive: timings.perceptionUs > 0 },
    { name: 'behavior_transition', timeUs: timings.behaviorTransitionUs, isActive: timings.behaviorTransitionUs > 0 },
    { name: 'wander', timeUs: timings.wanderUs, isActive: timings.wanderUs > 0 },
    { name: 'flee', timeUs: timings.fleeUs, isActive: timings.fleeUs > 0 },
    { name: 'avoidance', timeUs: timings.avoidanceUs, isActive: timings.avoidanceUs > 0 },
    { name: 'movement', timeUs: timings.movementUs, isActive: timings.movementUs > 0 },
    { name: 'rotation', timeUs: timings.rotationUs, isActive: timings.rotationUs > 0 },
  ];

  return systems.sort((a, b) => b.timeUs - a.timeUs);
};

export const Parallelism: React.FC<Props> = ({ metrics, systemTimings }) => {
  const { cpuCoresTotal, cpuCoresActive } = metrics;

  const systems = extractSystemsFromTimings(systemTimings);

  const cpuCores = Array.from({ length: cpuCoresTotal }, (_, i) => ({
    index: i,
    isActive: i < cpuCoresActive,
  }));

  return (
    <div className="cockpit-panel cockpit-panel-wide">
      <div className="cockpit-panel-title">Parallelism</div>

      <div className="parallelism-grids-container">
        <div className="parallelism-section cpu-section">
          <div className="parallelism-section-label">CPU Cores</div>
          <div className="parallelism-cpu-grid">
            {cpuCores.map((core) => (
              <div
                key={core.index}
                className="parallelism-cpu-block"
                style={{
                  backgroundColor: core.isActive ? COLORS.success : 'rgba(156, 163, 175, 0.2)',
                  opacity: core.isActive ? 1 : 0.3,
                }}
                title={`Core ${core.index}: ${core.isActive ? 'Active' : 'Idle'}`}
              />
            ))}
          </div>
        </div>

        <div className="parallelism-section ecs-section">
          <div className="parallelism-section-label">ECS Systems</div>
          <div className="parallelism-system-grid">
            {systems.map((system) => (
              <div
                key={system.name}
                className="parallelism-system-block"
                style={{
                  backgroundColor: system.isActive ? COLORS.streaming : 'rgba(156, 163, 175, 0.2)',
                  opacity: system.isActive ? 1 : 0.3,
                }}
                title={system.name.replace(/_/g, ' ')}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
