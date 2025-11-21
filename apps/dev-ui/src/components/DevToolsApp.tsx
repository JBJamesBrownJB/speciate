/**
 * Dev Tools Main Application
 *
 * Container for all dev tools functionality:
 * - Spawn creatures manually
 * - Load trial templates
 * - View simulation state
 */

import React, { useState, useEffect, useRef } from 'react';
import { SpawnForm } from './SpawnForm';
import { TrialSelector } from './TrialSelector';
import { ControlBar } from './ControlBar';
import { useSmoothedMetrics } from '../hooks/useSmoothedMetrics';
import { calculateStatistics } from '../utils/statistics';
import { snapshotToTelemetry } from '../utils/snapshotConverter';
import { MetricsColumn } from './MetricsColumn';
import type { SystemTimingsSnapshot, HardwareMetrics, ParallelizationMetrics, TelemetryFrame, MetricsSnapshot } from '../types';
import '../styles/cockpit.css';

const SAMPLE_DURATION_MS = 3000;
const TARGET_SAMPLES = 90;

export const DevToolsApp: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [tick, setTick] = useState(0);
  const [creatureCount, setCreatureCount] = useState(0);
  const [tickRateHz, setTickRateHz] = useState(0);
  const [systemTimings, setSystemTimings] = useState<SystemTimingsSnapshot | undefined>(undefined);
  const [rawHardwareMetrics, setRawHardwareMetrics] = useState<HardwareMetrics | undefined>(undefined);
  const [parallelizationMetrics, setParallelizationMetrics] = useState<ParallelizationMetrics | undefined>(undefined);
  const hardwareMetrics = useSmoothedMetrics(rawHardwareMetrics, 0.3);
  const lastHardwareUpdateRef = useRef<number>(0);

  const [isSampling, setIsSampling] = useState(false);
  const isSamplingRef = useRef(false);
  const samplesRef = useRef<TelemetryFrame[]>([]);
  const samplingStartTimeRef = useRef<number>(0);
  const [sampleCount, setSampleCount] = useState(0);
  const [loadedSnapshot, setLoadedSnapshot] = useState<MetricsSnapshot | null>(null);

  const processSamplesAndSave = async (collectedSamples: TelemetryFrame[], startTime: number, endTime: number) => {
    if (collectedSamples.length === 0) {
      alert('No samples collected');
      return;
    }

    const tickValues = collectedSamples.map(s => s.tick);
    const creatureCountValues = collectedSamples.map(s => s.creatureCount);
    const tickRateValues = collectedSamples.map(s => s.tickRateHz);

    const systemTimingsStats: Record<string, any> = {};
    if (collectedSamples[0].systemTimingsUs) {
      const keys = Object.keys(collectedSamples[0].systemTimingsUs) as Array<keyof SystemTimingsSnapshot>;
      for (const key of keys) {
        const values = collectedSamples
          .filter(s => s.systemTimingsUs && typeof s.systemTimingsUs[key] === 'number')
          .map(s => s.systemTimingsUs![key] as number);
        if (values.length > 0) {
          systemTimingsStats[key] = calculateStatistics(values);
        }
      }
    }

    const hardwareMetricsStats: Record<string, any> = {};
    const samplesWithHardware = collectedSamples.filter(s => s.hardwareMetrics);
    if (samplesWithHardware.length > 0) {
      const keys = Object.keys(samplesWithHardware[0].hardwareMetrics!) as Array<keyof HardwareMetrics>;
      for (const key of keys) {
        const values = samplesWithHardware
          .filter(s => s.hardwareMetrics && typeof s.hardwareMetrics[key] === 'number')
          .map(s => s.hardwareMetrics![key] as number);
        if (values.length > 0) {
          hardwareMetricsStats[key] = calculateStatistics(values);
        }
      }
    }

    let hardwareMetricsDerived = undefined;
    if (samplesWithHardware.length > 0) {
      const totalCycles = samplesWithHardware.reduce((sum, s) => sum + (s.hardwareMetrics!.cyclesDelta || 0), 0);
      const totalInstructions = samplesWithHardware.reduce((sum, s) => sum + (s.hardwareMetrics!.instructionsDelta || 0), 0);
      const totalCacheRefs = samplesWithHardware.reduce((sum, s) => sum + (s.hardwareMetrics!.cacheRefsDelta || 0), 0);
      const totalCacheMisses = samplesWithHardware.reduce((sum, s) => sum + (s.hardwareMetrics!.cacheMissesDelta || 0), 0);
      const totalBranchInstructions = samplesWithHardware.reduce((sum, s) => sum + (s.hardwareMetrics!.branchInstructionsDelta || 0), 0);
      const totalBranchMisses = samplesWithHardware.reduce((sum, s) => sum + (s.hardwareMetrics!.branchMissesDelta || 0), 0);
      const totalStalledFrontend = samplesWithHardware.reduce((sum, s) => sum + (s.hardwareMetrics!.stalledFrontendDelta || 0), 0);
      const totalStalledBackend = samplesWithHardware.reduce((sum, s) => sum + (s.hardwareMetrics!.stalledBackendDelta || 0), 0);

      hardwareMetricsDerived = {
        ipc: totalCycles > 0 ? totalInstructions / totalCycles : 0,
        l1dMissRate: hardwareMetricsStats.l1dMissRate?.avg ?? 0,
        l1iMissRate: hardwareMetricsStats.l1iMissRate?.avg ?? 0,
        llcMissRate: totalCacheRefs > 0 ? (totalCacheMisses / totalCacheRefs) * 100 : 0,
        branchMissRate: totalBranchInstructions > 0 ? (totalBranchMisses / totalBranchInstructions) * 100 : 0,
        frontendStallRatio: totalCycles > 0 ? (totalStalledFrontend / totalCycles) * 100 : 0,
        backendStallRatio: totalCycles > 0 ? (totalStalledBackend / totalCycles) * 100 : 0,
      };
    }

    const parallelizationMetricsStats: Record<string, any> = {};
    const samplesWithParallelization = collectedSamples.filter(s => s.parallelizationMetrics);
    if (samplesWithParallelization.length > 0) {
      const keys = Object.keys(samplesWithParallelization[0].parallelizationMetrics!) as Array<keyof ParallelizationMetrics>;
      for (const key of keys) {
        const values = samplesWithParallelization
          .filter(s => s.parallelizationMetrics && typeof s.parallelizationMetrics[key] === 'number')
          .map(s => s.parallelizationMetrics![key] as number);
        if (values.length > 0) {
          parallelizationMetricsStats[key] = calculateStatistics(values);
        }
      }
    }

    const snapshot: MetricsSnapshot = {
      metadata: {
        sampleCount: collectedSamples.length,
        durationMs: endTime - startTime,
        startTime: new Date(startTime).toISOString(),
        endTime: new Date(endTime).toISOString(),
      },
      tick: calculateStatistics(tickValues),
      creatureCount: calculateStatistics(creatureCountValues),
      tickRateHz: calculateStatistics(tickRateValues),
      systemTimings: systemTimingsStats,
      hardwareMetrics: Object.keys(hardwareMetricsStats).length > 0 ? hardwareMetricsStats : undefined,
      hardwareMetricsDerived: hardwareMetricsDerived,
      parallelizationMetrics: Object.keys(parallelizationMetricsStats).length > 0 ? parallelizationMetricsStats : undefined,
    };

    try {
      const result = await window.electron?.saveMetricsSnapshot?.(snapshot);
      if (result?.success) {
        alert(`Snapshot saved successfully!\n${collectedSamples.length} samples over ${(endTime - startTime).toFixed(0)}ms\n${result.path}`);
      } else {
        alert(`Failed to save snapshot: ${result?.error || 'Unknown error'}`);
      }
    } catch (error) {
      alert(`Error saving snapshot: ${error}`);
    }
  };

  useEffect(() => {
    if (window.electron?.sendCommand) {
      setIsConnected(true);
    }

    const handleTelemetryUpdate = (telemetry: TelemetryFrame) => {
      setTick(telemetry.tick);
      setCreatureCount(telemetry.creatureCount);
      setTickRateHz(telemetry.tickRateHz);
      setSystemTimings(telemetry.systemTimingsUs);

      const now = Date.now();
      if (telemetry.hardwareMetrics && (now - lastHardwareUpdateRef.current) >= 200) {
        setRawHardwareMetrics(telemetry.hardwareMetrics);
        lastHardwareUpdateRef.current = now;
      }

      if (telemetry.parallelizationMetrics) {
        setParallelizationMetrics(telemetry.parallelizationMetrics);
      }

      if (isSamplingRef.current) {
        samplesRef.current.push(telemetry);
        setSampleCount(samplesRef.current.length);

        const elapsed = now - samplingStartTimeRef.current;

        if (samplesRef.current.length >= TARGET_SAMPLES || elapsed >= SAMPLE_DURATION_MS) {
          processSamplesAndSave(samplesRef.current, samplingStartTimeRef.current, now);
          isSamplingRef.current = false;
          setIsSampling(false);
          samplesRef.current = [];
          setSampleCount(0);
        }
      }
    };

    window.electron?.onTelemetryUpdate?.(handleTelemetryUpdate);

    return () => {
      window.electron?.removeStateUpdateListener?.();
    };
  }, []);

  const handleSpawn = (x: number, y: number) => {
    window.electron?.sendCommand?.({
      type: 'dev_spawn_creature',
      x,
      y,
      dna: null,
    });
  };

  const handleLoadTrial = (template: string) => {
    window.electron?.sendCommand?.({
      type: 'dev_load_trial',
      template,
    });
  };

  const handleClearCreatures = () => {
    if (window.confirm('Clear all creatures? This cannot be undone.')) {
      window.electron?.sendCommand?.({
        type: 'dev_clear_creatures',
      });
    }
  };

  const handleRecordSnapshot = () => {
    if (!systemTimings) {
      alert('No metrics available to snapshot');
      return;
    }

    isSamplingRef.current = true;
    samplesRef.current = [];
    samplingStartTimeRef.current = Date.now();
    setIsSampling(true);
    setSampleCount(0);
  };

  const handleLoadSnapshot = async () => {
    try {
      const snapshot = await window.electron?.loadMetricsSnapshot?.();
      if (snapshot) {
        setLoadedSnapshot(snapshot);
        await window.electron?.resizeWindow?.(1500);
      }
    } catch (error) {
      alert(`Error loading snapshot: ${error}`);
    }
  };

  const handleClearSnapshot = async () => {
    setLoadedSnapshot(null);
    await window.electron?.resizeWindow?.(850);
  };

  const snapshotTelemetry = loadedSnapshot ? snapshotToTelemetry(loadedSnapshot) : null;
  const showComparison = !!loadedSnapshot;

  return (
    <div>
      <h1>Speciate Dev Tools</h1>

      <ControlBar
        isConnected={isConnected}
        tick={tick}
        creatureCount={creatureCount}
        isSampling={isSampling}
        sampleCount={sampleCount}
        systemTimings={systemTimings}
        loadedSnapshot={loadedSnapshot}
        onRecordSnapshot={handleRecordSnapshot}
        onLoadSnapshot={handleLoadSnapshot}
        onClearSnapshot={handleClearSnapshot}
      />

      <SpawnForm onSpawn={handleSpawn} disabled={!isConnected} />
      <TrialSelector onLoadTrial={handleLoadTrial} disabled={!isConnected} />

      <div className="section">
        <h2>Danger Zone</h2>
        <button
          className="danger"
          onClick={handleClearCreatures}
          disabled={!isConnected}
        >
          Clear All Creatures
        </button>
      </div>

      {showComparison && snapshotTelemetry ? (
        <div className="comparison-layout">
          <MetricsColumn
            label="🔴 LIVE"
            labelClass="live-header"
            tick={tick}
            creatureCount={creatureCount}
            tickRateHz={tickRateHz}
            systemTimings={systemTimings}
            hardwareMetrics={hardwareMetrics}
            parallelizationMetrics={parallelizationMetrics}
          />
          <MetricsColumn
            label="📁 SNAPSHOT"
            labelClass="snapshot-header"
            tick={snapshotTelemetry.tick}
            creatureCount={snapshotTelemetry.creatureCount}
            tickRateHz={snapshotTelemetry.tickRateHz}
            systemTimings={snapshotTelemetry.systemTimingsUs}
            hardwareMetrics={snapshotTelemetry.hardwareMetrics}
            parallelizationMetrics={snapshotTelemetry.parallelizationMetrics}
          />
        </div>
      ) : (
        <MetricsColumn
          label=""
          labelClass=""
          tick={tick}
          creatureCount={creatureCount}
          tickRateHz={tickRateHz}
          systemTimings={systemTimings}
          hardwareMetrics={hardwareMetrics}
          parallelizationMetrics={parallelizationMetrics}
        />
      )}
    </div>
  );
};
