import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ElectronIPCClient } from './ElectronIPCClient';
import type { GameState } from '../../types/GameState';

/**
 * Helper: Create mock NAPI buffer with SoA layout
 * Layout: [ID₁...IDₙ, X₁...Xₙ, Y₁...Yₙ, Rot₁...Rotₙ]
 */
function createMockNAPIBuffer(creatureCount: number): number[] {
  const buffer = new Array(creatureCount * 4);

  // Write SoA data
  for (let i = 0; i < creatureCount; i++) {
    buffer[i] = i + 1; // ID
    buffer[creatureCount + i] = 100.0 + i * 10; // X
    buffer[creatureCount * 2 + i] = 200.0 + i * 10; // Y
    buffer[creatureCount * 3 + i] = 1.5 + i * 0.1; // Rotation
  }

  return buffer;
}

describe('ElectronIPCClient', () => {
  let client: ElectronIPCClient;
  let mockElectronAPI: {
    onNAPIBufferUpdate: ReturnType<typeof vi.fn>;
    onTelemetryUpdate: ReturnType<typeof vi.fn>;
    onPerceptionDebugUpdate: ReturnType<typeof vi.fn>;
    removeStateUpdateListener: ReturnType<typeof vi.fn>;
    selectCreatureDebug: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    mockElectronAPI = {
      onNAPIBufferUpdate: vi.fn(),
      onTelemetryUpdate: vi.fn(),
      onPerceptionDebugUpdate: vi.fn(),
      removeStateUpdateListener: vi.fn(),
      selectCreatureDebug: vi.fn(),
    };

    (global as any).window = {
      electron: mockElectronAPI,
    };

    client = new ElectronIPCClient();
  });

  afterEach(() => {
    delete (global as any).window;
  });

  describe('connect', () => {
    it('should throw error if window.electron is not available', async () => {
      delete (global as any).window.electron;
      const clientWithoutElectron = new ElectronIPCClient();

      await expect(clientWithoutElectron.connect()).rejects.toThrow(
        'ElectronIPCClient: window.electron not available (not running in Electron)'
      );
    });

    it('should register NAPI buffer update listener', async () => {
      await client.connect();

      expect(mockElectronAPI.onNAPIBufferUpdate).toHaveBeenCalledOnce();
      expect(mockElectronAPI.onNAPIBufferUpdate).toHaveBeenCalledWith(expect.any(Function));
    });
  });

  describe('NAPI buffer parsing', () => {
    it('should parse NAPI buffer and update latestState', async () => {
      let capturedCallback: ((data: { buffer: number[], creatureCount: number }) => void) | null = null;

      mockElectronAPI.onNAPIBufferUpdate.mockImplementation((callback) => {
        capturedCallback = callback;
      });

      await client.connect();

      expect(capturedCallback).not.toBeNull();

      // Before buffer update
      expect(client.getLatestState()).toBeNull();

      // Simulate NAPI buffer callback
      const mockBuffer = createMockNAPIBuffer(100);
      capturedCallback!({ buffer: mockBuffer, creatureCount: 100 });

      // After buffer update
      const state = client.getLatestState();
      expect(state).not.toBeNull();
      expect(state!.creatures.length).toBe(100);
      expect(state!.entityCount).toBe(100);
      expect(state!.protocolVersion).toBe(2);
    });

    it('should correctly parse SoA layout into creature data', async () => {
      let capturedCallback: ((data: { buffer: number[], creatureCount: number }) => void) | null = null;

      mockElectronAPI.onNAPIBufferUpdate.mockImplementation((callback) => {
        capturedCallback = callback;
      });

      await client.connect();

      const mockBuffer = createMockNAPIBuffer(3);
      capturedCallback!({ buffer: mockBuffer, creatureCount: 3 });

      const state = client.getLatestState();
      expect(state).not.toBeNull();

      // Verify first creature
      expect(state!.creatures[0]).toEqual({
        id: 1,
        x: 100.0,
        y: 200.0,
        rotation: 1.5,
        size: 1,
      });

      // Verify second creature
      expect(state!.creatures[1]).toEqual({
        id: 2,
        x: 110.0,
        y: 210.0,
        rotation: 1.6,
        size: 1,
      });

      // Verify third creature
      expect(state!.creatures[2]).toEqual({
        id: 3,
        x: 120.0,
        y: 220.0,
        rotation: 1.7,
        size: 1,
      });
    });

    it('should handle empty buffer (zero creatures)', async () => {
      let capturedCallback: ((data: { buffer: number[], creatureCount: number }) => void) | null = null;

      mockElectronAPI.onNAPIBufferUpdate.mockImplementation((callback) => {
        capturedCallback = callback;
      });

      await client.connect();

      capturedCallback!({ buffer: [], creatureCount: 0 });

      const state = client.getLatestState();
      expect(state).not.toBeNull();
      expect(state!.creatures.length).toBe(0);
      expect(state!.entityCount).toBe(0);
    });

    it('should update state on each buffer update', async () => {
      let capturedCallback: ((data: { buffer: number[], creatureCount: number }) => void) | null = null;

      mockElectronAPI.onNAPIBufferUpdate.mockImplementation((callback) => {
        capturedCallback = callback;
      });

      await client.connect();

      // First update: 10 creatures
      capturedCallback!({ buffer: createMockNAPIBuffer(10), creatureCount: 10 });
      expect(client.getLatestState()!.creatures.length).toBe(10);

      // Second update: 50 creatures
      capturedCallback!({ buffer: createMockNAPIBuffer(50), creatureCount: 50 });
      expect(client.getLatestState()!.creatures.length).toBe(50);

      // Third update: 100 creatures
      capturedCallback!({ buffer: createMockNAPIBuffer(100), creatureCount: 100 });
      expect(client.getLatestState()!.creatures.length).toBe(100);
    });
  });

  describe('onStateUpdate callback', () => {
    it('should trigger callback when buffer updates', async () => {
      let capturedCallback: ((data: { buffer: number[], creatureCount: number }) => void) | null = null;

      mockElectronAPI.onNAPIBufferUpdate.mockImplementation((callback) => {
        capturedCallback = callback;
      });

      await client.connect();

      const stateCallback = vi.fn();
      client.onStateUpdate(stateCallback);

      // Trigger buffer update
      const mockBuffer = createMockNAPIBuffer(10);
      capturedCallback!({ buffer: mockBuffer, creatureCount: 10 });

      expect(stateCallback).toHaveBeenCalledOnce();

      const receivedState: GameState = stateCallback.mock.calls[0][0];
      expect(receivedState.creatures.length).toBe(10);
      expect(receivedState.creatures[0].x).toBe(100.0);
    });

    it('should allow multiple callbacks to be registered', async () => {
      let capturedCallback: ((data: { buffer: number[], creatureCount: number }) => void) | null = null;

      mockElectronAPI.onNAPIBufferUpdate.mockImplementation((callback) => {
        capturedCallback = callback;
      });

      await client.connect();

      const callback1 = vi.fn();
      const callback2 = vi.fn();
      client.onStateUpdate(callback1);
      client.onStateUpdate(callback2);

      const mockBuffer = createMockNAPIBuffer(10);
      capturedCallback!({ buffer: mockBuffer, creatureCount: 10 });

      expect(callback1).toHaveBeenCalledOnce();
      expect(callback2).toHaveBeenCalledOnce();
    });
  });

  describe('disconnect', () => {
    it('should clear state and callbacks on disconnect', async () => {
      let capturedCallback: ((data: { buffer: number[], creatureCount: number }) => void) | null = null;

      mockElectronAPI.onNAPIBufferUpdate.mockImplementation((callback) => {
        capturedCallback = callback;
      });

      await client.connect();

      const mockBuffer = createMockNAPIBuffer(10);
      capturedCallback!({ buffer: mockBuffer, creatureCount: 10 });

      expect(client.getLatestState()).not.toBeNull();

      await client.disconnect();

      expect(client.getLatestState()).toBeNull();
      expect(mockElectronAPI.removeStateUpdateListener).toHaveBeenCalledOnce();
    });
  });

  describe('selectCreatureDebug', () => {
    it('should call window.electron.selectCreatureDebug with creature id', () => {
      client.selectCreatureDebug(42);

      expect(mockElectronAPI.selectCreatureDebug).toHaveBeenCalledOnce();
      expect(mockElectronAPI.selectCreatureDebug).toHaveBeenCalledWith(42);
    });

    it('should call window.electron.selectCreatureDebug with null to clear selection', () => {
      client.selectCreatureDebug(null);

      expect(mockElectronAPI.selectCreatureDebug).toHaveBeenCalledOnce();
      expect(mockElectronAPI.selectCreatureDebug).toHaveBeenCalledWith(null);
    });

    it('should not throw if window.electron.selectCreatureDebug is undefined', () => {
      delete (mockElectronAPI as any).selectCreatureDebug;

      expect(() => client.selectCreatureDebug(42)).not.toThrow();
    });
  });
});
