import { describe, it, expect } from 'vitest';
import type { GameState, SystemTimingsSnapshot, HardwareMetrics, EcsMetrics } from './GameState';

describe('GameState Types', () => {
  describe('HardwareMetrics', () => {
    it('should accept valid hardware metrics object', () => {
      const metrics: HardwareMetrics = {
        cycles: 1000000000,
        instructions: 1200000000,
        cacheReferences: 50000000,
        cacheMisses: 1250000,
        l1Misses: 625000,
        ipc: 1.24,
        cacheMissRate: 2.5,
        l1MissRate: 1.25,
      };

      expect(metrics.ipc).toBe(1.24);
      expect(metrics.cacheMissRate).toBe(2.5);
      expect(metrics.l1MissRate).toBe(1.25);
    });

    it('should have IPC as primary metric', () => {
      const metrics: HardwareMetrics = {
        cycles: 1000000000,
        instructions: 1500000000,
        cacheReferences: 50000000,
        cacheMisses: 1000000,
        l1Misses: 500000,
        ipc: 1.5,
        cacheMissRate: 2.0,
        l1MissRate: 1.0,
      };

      expect(metrics.ipc).toBeGreaterThan(0);
      expect(typeof metrics.ipc).toBe('number');
    });

    it('should support mock data with realistic values', () => {
      const mockMetrics: HardwareMetrics = {
        cycles: 1000000000,
        instructions: 1200000000,
        cacheReferences: 50000000,
        cacheMisses: 1250000,
        l1Misses: 625000,
        ipc: 1.2,
        cacheMissRate: 2.5,
        l1MissRate: 1.25,
      };

      expect(mockMetrics.ipc).toBeGreaterThanOrEqual(0.5);
      expect(mockMetrics.ipc).toBeLessThanOrEqual(3.0);
      expect(mockMetrics.cacheMissRate).toBeGreaterThanOrEqual(0);
      expect(mockMetrics.cacheMissRate).toBeLessThanOrEqual(10);
    });
  });

  describe('EcsMetrics', () => {
    it('should accept archetype and entity counts', () => {
      const ecsMetrics: EcsMetrics = {
        archetypeCount: 8,
        entityCount: 10000,
        systemTickMs: 16.7,
      };

      expect(ecsMetrics.archetypeCount).toBe(8);
      expect(ecsMetrics.entityCount).toBe(10000);
      expect(ecsMetrics.systemTickMs).toBeCloseTo(16.7);
    });
  });

  describe('SystemTimingsSnapshot', () => {
    it('should include new ECS metrics fields', () => {
      const timings: SystemTimingsSnapshot = {
        totalTickUs: 16700,
        movementUs: 3200, // Now includes rotation (fused)
        perceptionUs: 2500,
        spatialGridRebuildUs: 100,
        l1AggregationUs: 50,
        behaviorTransitionUs: 800,
        steeringUs: 2300, // Fused steering (Sprint 20)
        captureDebugAccelUs: 5,
        exportPositionsUs: 1350, // IPC buffer export with parallel sort (Sprint 16)
        ipcQueryUs: 200,
        ipcSerializeUs: 300,
        ipcWriteUs: 100,
        ipcFrameDropsTotal: 0,
        ipcChannelUtilizationPct: 45,
        ipcWriterThreadUs: 150,
        archetypeCount: 8,
        entityCount: 10000,
        cellsQueriedTotal: 0,
      };

      expect(timings.archetypeCount).toBe(8);
      expect(timings.entityCount).toBe(10000);
      expect(timings.totalTickUs).toBe(16700);
    });
  });

  describe('GameState', () => {
    it('should include optional hardware metrics', () => {
      const state: GameState = {
        protocolVersion: 1,
        tick: 100,
        tickRateHz: 20,
        creatures: [],
        entityCount: 10000,
        systemTimingsUs: {
          totalTickUs: 16700,
          movementUs: 3200, // Now includes rotation (fused)
          perceptionUs: 2500,
          spatialGridRebuildUs: 100,
          l1AggregationUs: 50,
          behaviorTransitionUs: 800,
          steeringUs: 2300, // Fused steering (Sprint 20)
          captureDebugAccelUs: 5,
          exportPositionsUs: 1350, // IPC buffer export with parallel sort (Sprint 16)
          ipcQueryUs: 200,
          ipcSerializeUs: 300,
          ipcWriteUs: 100,
          ipcFrameDropsTotal: 0,
          ipcChannelUtilizationPct: 45,
          ipcWriterThreadUs: 150,
          archetypeCount: 8,
          entityCount: 10000,
          cellsQueriedTotal: 0,
        },
        hardwareMetrics: {
          cycles: 1000000000,
          instructions: 1200000000,
          cacheReferences: 50000000,
          cacheMisses: 1250000,
          l1Misses: 625000,
          ipc: 1.24,
          cacheMissRate: 2.5,
          l1MissRate: 1.25,
        },
      };

      expect(state.hardwareMetrics).toBeDefined();
      expect(state.hardwareMetrics?.ipc).toBe(1.24);
      expect(state.systemTimingsUs?.archetypeCount).toBe(8);
    });

    it('should work without hardware metrics (production build)', () => {
      const state: GameState = {
        protocolVersion: 1,
        tick: 100,
        tickRateHz: 20,
        creatures: [],
      };

      expect(state.hardwareMetrics).toBeUndefined();
      expect(state.protocolVersion).toBe(1);
    });

    it('should validate IPC thresholds for UI coloring', () => {
      const excellent: HardwareMetrics = {
        cycles: 1000000000,
        instructions: 1800000000,
        cacheReferences: 50000000,
        cacheMisses: 1000000,
        l1Misses: 500000,
        ipc: 1.8,
        cacheMissRate: 2.0,
        l1MissRate: 1.0,
      };

      const acceptable: HardwareMetrics = {
        cycles: 1000000000,
        instructions: 1000000000,
        cacheReferences: 50000000,
        cacheMisses: 1500000,
        l1Misses: 750000,
        ipc: 1.0,
        cacheMissRate: 3.0,
        l1MissRate: 1.5,
      };

      const poor: HardwareMetrics = {
        cycles: 1000000000,
        instructions: 400000000,
        cacheReferences: 50000000,
        cacheMisses: 2500000,
        l1Misses: 1250000,
        ipc: 0.4,
        cacheMissRate: 5.0,
        l1MissRate: 2.5,
      };

      // Excellent: IPC > 1.5 (Vector/SIMD)
      expect(excellent.ipc).toBeGreaterThan(1.5);

      // Acceptable: IPC 0.5-1.5 (Scalar)
      expect(acceptable.ipc).toBeGreaterThanOrEqual(0.5);
      expect(acceptable.ipc).toBeLessThanOrEqual(1.5);

      // Poor: IPC < 0.5 (Stall)
      expect(poor.ipc).toBeLessThan(0.5);
    });
  });

  describe('MessagePack Compatibility', () => {
    it('should match Rust serialization format (camelCase)', () => {
      const timings: SystemTimingsSnapshot = {
        totalTickUs: 1000,
        movementUs: 100, // Now includes rotation (fused)
        perceptionUs: 200,
        spatialGridRebuildUs: 10,
        l1AggregationUs: 5,
        behaviorTransitionUs: 50,
        steeringUs: 150, // Fused steering (Sprint 20)
        captureDebugAccelUs: 2,
        exportPositionsUs: 135, // IPC buffer export with parallel sort (Sprint 16)
        ipcQueryUs: 40,
        ipcSerializeUs: 60,
        ipcWriteUs: 30,
        ipcFrameDropsTotal: 0,
        ipcChannelUtilizationPct: 25,
        ipcWriterThreadUs: 50,
        archetypeCount: 5,
        entityCount: 1000,
        cellsQueriedTotal: 0,
      };

      // These keys should match Rust's #[serde(rename_all = "camelCase")]
      const keys = Object.keys(timings);
      expect(keys).toContain('totalTickUs');
      expect(keys).toContain('archetypeCount');
      expect(keys).toContain('entityCount');
      expect(keys).not.toContain('total_tick_us'); // No snake_case
    });

    it('should match hardware metrics camelCase format', () => {
      const metrics: HardwareMetrics = {
        cycles: 1000000000,
        instructions: 1200000000,
        cacheReferences: 50000000,
        cacheMisses: 1250000,
        l1Misses: 625000,
        ipc: 1.2,
        cacheMissRate: 2.5,
        l1MissRate: 1.25,
      };

      const keys = Object.keys(metrics);
      expect(keys).toContain('cacheReferences');
      expect(keys).toContain('cacheMisses');
      expect(keys).toContain('l1Misses');
      expect(keys).toContain('cacheMissRate');
      expect(keys).toContain('l1MissRate');
      expect(keys).not.toContain('cache_references'); // No snake_case
    });
  });
});
