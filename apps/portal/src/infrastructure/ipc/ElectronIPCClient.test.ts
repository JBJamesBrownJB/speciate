import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ElectronIPCClient } from './ElectronIPCClient';
import type { GameState } from '../../types/GameState';
import { FLOATS_PER_CREATURE, getBufferOffsets } from '../../types/BufferLayout';

// Helper: Create mock NAPI buffer with SoA layout
// Layout: [ID₁...IDₙ, X₁...Xₙ, Y₁...Yₙ, Rot₁...Rotₙ, Size₁...Sizeₙ]
function createMockNAPIBuffer(creatureCount: number): number[] {
  const buffer = new Array(creatureCount * FLOATS_PER_CREATURE);
  const offsets = getBufferOffsets(creatureCount);

  for (let i = 0; i < creatureCount; i++) {
    buffer[offsets.id + i] = i + 1; // ID
    buffer[offsets.x + i] = 100.0 + i * 10; // X
    buffer[offsets.y + i] = 200.0 + i * 10; // Y
    buffer[offsets.rot + i] = 1.5 + i * 0.1; // Rotation
    buffer[offsets.size + i] = 0.5 + i * 0.1; // Size
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

      // Verify first creature (size = 0.5 + 0*0.1 = 0.5)
      expect(state!.creatures[0]).toEqual({
        id: 1,
        x: 100.0,
        y: 200.0,
        rotation: 1.5,
        size: 0.5,
      });

      // Verify second creature (size = 0.5 + 1*0.1 = 0.6)
      expect(state!.creatures[1]).toEqual({
        id: 2,
        x: 110.0,
        y: 210.0,
        rotation: 1.6,
        size: 0.6,
      });

      // Verify third creature (size = 0.5 + 2*0.1 = 0.7)
      expect(state!.creatures[2]).toEqual({
        id: 3,
        x: 120.0,
        y: 220.0,
        rotation: 1.7,
        size: 0.7,
      });
    });

    it('should parse size field from NAPI buffer correctly', async () => {
      let capturedCallback: ((data: { buffer: number[], creatureCount: number }) => void) | null = null;

      mockElectronAPI.onNAPIBufferUpdate.mockImplementation((callback) => {
        capturedCallback = callback;
      });

      await client.connect();

      // Create buffer with distinct size values to verify correct offset
      const creatureCount = 5;
      const buffer = new Array(creatureCount * FLOATS_PER_CREATURE);
      const offsets = getBufferOffsets(creatureCount);

      for (let i = 0; i < creatureCount; i++) {
        buffer[offsets.id + i] = i + 1;
        buffer[offsets.x + i] = 0;
        buffer[offsets.y + i] = 0;
        buffer[offsets.rot + i] = 0;
        buffer[offsets.size + i] = 10.0 + i; // Distinct sizes: 10, 11, 12, 13, 14
      }

      capturedCallback!({ buffer, creatureCount });

      const state = client.getLatestState();
      expect(state).not.toBeNull();

      // Verify each creature has correct size from buffer
      expect(state!.creatures[0].size).toBe(10.0);
      expect(state!.creatures[1].size).toBe(11.0);
      expect(state!.creatures[2].size).toBe(12.0);
      expect(state!.creatures[3].size).toBe(13.0);
      expect(state!.creatures[4].size).toBe(14.0);
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

  describe('perception debug buffer parsing', () => {
    it('should parse queryRadius at correct buffer offset', async () => {
      let capturedCallback: ((buffer: Float32Array) => void) | null = null;

      mockElectronAPI.onPerceptionDebugUpdate.mockImplementation((callback) => {
        capturedCallback = callback;
        return () => {};
      });

      const receivedData: any[] = [];
      await client.connect();
      client.onPerceptionDebugUpdate((data) => {
        receivedData.push(data);
      });

      // Create buffer with specific values at expected offsets
      // Layout: [0]=has_data, [1]=id, [2]=x, [3]=y, [4]=perceptionRange, [5]=queryRadius, [6]=fovAngle, ...
      const buffer = new Float32Array(608);
      buffer[0] = 1.0; // has_data
      buffer[1] = 42; // entity_id
      buffer[2] = 100.0; // x
      buffer[3] = 200.0; // y
      buffer[4] = 50.0; // perceptionRange
      buffer[5] = 60.0; // queryRadius (new field)
      buffer[6] = 1.57; // fovAngle
      buffer[7] = 0.5; // rotation
      buffer[8] = 1.0; // ax
      buffer[9] = 2.0; // ay
      buffer[10] = 0; // neighbor_count

      capturedCallback!(buffer);

      expect(receivedData.length).toBe(1);
      expect(receivedData[0].entityId).toBe(42);
      expect(receivedData[0].perceptionRange).toBe(50.0);
      expect(receivedData[0].queryRadius).toBe(60.0);
      expect(receivedData[0].fovAngle).toBeCloseTo(1.57, 2);
      expect(receivedData[0].rotation).toBe(0.5);
      expect(receivedData[0].ax).toBe(1.0);
      expect(receivedData[0].ay).toBe(2.0);
    });

    it('should call callback with null when has_data is false', async () => {
      let capturedCallback: ((buffer: Float32Array) => void) | null = null;

      mockElectronAPI.onPerceptionDebugUpdate.mockImplementation((callback) => {
        capturedCallback = callback;
        return () => {};
      });

      const receivedData: any[] = [];
      await client.connect();
      client.onPerceptionDebugUpdate((data) => {
        receivedData.push(data);
      });

      const buffer = new Float32Array(608);
      buffer[0] = 0.0; // has_data = false

      capturedCallback!(buffer);

      expect(receivedData.length).toBe(1);
      expect(receivedData[0]).toBeNull();
    });

    it('should parse cell data at correct section offset', async () => {
      let capturedCallback: ((buffer: Float32Array) => void) | null = null;

      mockElectronAPI.onPerceptionDebugUpdate.mockImplementation((callback) => {
        capturedCallback = callback;
        return () => {};
      });

      const receivedData: any[] = [];
      await client.connect();
      client.onPerceptionDebugUpdate((data) => {
        receivedData.push(data);
      });

      const buffer = new Float32Array(608);
      buffer[0] = 1.0; // has_data
      buffer[10] = 0; // neighbor_count = 0

      // Cell section starts at offset 203 (HEADER_SIZE=11 + MAX_DEBUG_NEIGHBORS*3=192)
      const CELL_SECTION_OFFSET = 203;
      buffer[CELL_SECTION_OFFSET] = 10.0; // cell_size
      buffer[CELL_SECTION_OFFSET + 1] = 2; // num_queried_cells
      buffer[CELL_SECTION_OFFSET + 2] = 5; // creature_cell_x
      buffer[CELL_SECTION_OFFSET + 3] = 7; // creature_cell_y

      // Queried cells start at CELL_SECTION_OFFSET + 4
      buffer[CELL_SECTION_OFFSET + 4] = 4; // cell 0 x
      buffer[CELL_SECTION_OFFSET + 5] = 6; // cell 0 y
      buffer[CELL_SECTION_OFFSET + 6] = 5; // cell 1 x
      buffer[CELL_SECTION_OFFSET + 7] = 7; // cell 1 y

      capturedCallback!(buffer);

      expect(receivedData.length).toBe(1);
      expect(receivedData[0].cellSize).toBe(10.0);
      expect(receivedData[0].creatureCell).toEqual({ x: 5, y: 7 });
      expect(receivedData[0].queriedCells.length).toBe(2);
      expect(receivedData[0].queriedCells[0]).toEqual({ x: 4, y: 6 });
      expect(receivedData[0].queriedCells[1]).toEqual({ x: 5, y: 7 });
    });
  });
});
