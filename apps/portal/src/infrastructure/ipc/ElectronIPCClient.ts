import type { IPCClient } from './IPCClient';
import type { GameState, CreatureData, PerceptionDebugData, NeighborDebugInfo, QueriedCell, L1VisionDebugEntry, L1Classification } from '../../types/GameState';
import type { TelemetryFrame } from '../../types/TelemetryFrame';
import { getBufferOffsets } from '../../types/BufferLayout';

const HEADER_SIZE = 11;
const MAX_DEBUG_NEIGHBORS = 64;
const CELL_SECTION_OFFSET = HEADER_SIZE + MAX_DEBUG_NEIGHBORS * 3; // 203
const CELL_HEADER_SIZE = 4;
const MAX_QUERIED_CELLS = 100;
const CHECKED_CELL_SECTION_OFFSET = CELL_SECTION_OFFSET + CELL_HEADER_SIZE + MAX_QUERIED_CELLS * 2; // 407
const CHECKED_CELL_HEADER_SIZE = 1;
const MAX_CHECKED_CELLS = 100;
const L1_VISION_SECTION_OFFSET = CHECKED_CELL_SECTION_OFFSET + CHECKED_CELL_HEADER_SIZE + MAX_CHECKED_CELLS * 2; // 608
const L1_VISION_HEADER_SIZE = 1;
const MAX_L1_VISION_ENTRIES = 48;
const L1_VISION_ENTRY_SIZE = 6;

export class ElectronIPCClient implements IPCClient {
  private static readonly MAX_CREATURES = 250_000;

  private latestState: GameState | null = null;
  private stateCallbacks: Set<(state: GameState) => void> = new Set();
  private telemetryCallbacks: Set<(telemetry: TelemetryFrame) => void> = new Set();
  private perceptionDebugCallbacks: Set<(data: PerceptionDebugData | null) => void> = new Set();
  private _cachedTickRateHz = NaN;
  private unsubscribers: Array<() => void> = [];

  private creatures: CreatureData[] = [];
  private creaturesInitialized = false;

  private ensureCreatureCapacity(count: number): void {
    if (this.creaturesInitialized && this.creatures.length >= count) return;

    const targetSize = Math.max(count, ElectronIPCClient.MAX_CREATURES);
    for (let i = this.creatures.length; i < targetSize; i++) {
      this.creatures.push({ id: 0, x: 0, y: 0, rotation: 0, size: 1.0 });
    }
    this.creaturesInitialized = true;
  }

  async connect(): Promise<void> {
    if (!window.electron) {
      throw new Error('ElectronIPCClient: window.electron not available (not running in Electron)');
    }

    // Listen for telemetry updates (tick rate, perception debug, etc.)
    const unsubTelemetry = window.electron.onTelemetryUpdate((telemetry) => {
      if (telemetry.tickRateHz !== undefined) {
        this._cachedTickRateHz = telemetry.tickRateHz;
      }

      this.telemetryCallbacks.forEach(callback => {
        try {
          callback(telemetry);
        } catch (error) {
          console.error('[ElectronIPCClient] Error in telemetry callback:', error);
        }
      });
    });
    this.unsubscribers.push(unsubTelemetry);

    // Use new NAPI buffer updates
    const unsubBuffer = window.electron.onNAPIBufferUpdate((data: { buffer: Float32Array, creatureCount: number, tick?: number }) => {
      try {
        const { buffer, creatureCount, tick } = data;

        // Ensure pre-allocated array has capacity (one-time allocation)
        this.ensureCreatureCapacity(creatureCount);

        // Parse SoA layout: [ID₁...IDₙ, X₁...Xₙ, Y₁...Yₙ, Rot₁...Rotₙ, Size₁...Sizeₙ]
        // Update objects IN-PLACE (zero allocations per tick)
        const offsets = getBufferOffsets(creatureCount);

        for (let i = 0; i < creatureCount; i++) {
          const creature = this.creatures[i];
          creature.id = buffer[offsets.id + i];
          creature.x = buffer[offsets.x + i];
          creature.y = buffer[offsets.y + i];
          creature.rotation = buffer[offsets.rot + i];
          creature.size = buffer[offsets.size + i];
        }

        // Return view of active creatures (slice creates new array ref, but objects are reused)
        const creatures = this.creatures.slice(0, creatureCount);

        const state: GameState = {
          protocolVersion: 2, // NAPI protocol version
          tick: tick ?? 0, // sim tick from the push-on-swap doorbell (0 in poll fallback)
          tickRateHz: this._cachedTickRateHz, // Updated from telemetry
          creatures,
          entityCount: creatureCount,
        };

        this.latestState = state;

        this.stateCallbacks.forEach(callback => {
          try {
            callback(state);
          } catch (error) {
            console.error('[ElectronIPCClient] Error in state update callback:', error);
          }
        });
      } catch (error) {
        console.error('[ElectronIPCClient] Failed to parse NAPI buffer:', error);
      }
    });
    this.unsubscribers.push(unsubBuffer);

    // Listen for perception debug buffer updates (every tick)
    const unsubPerception = window.electron.onPerceptionDebugUpdate((buffer: Float32Array) => {
      try {
        // Check has_data flag first (buffer[0] = 1.0 means valid data, 0.0 means no selection)
        if (buffer[0] < 0.5) {
          // No selection - call callbacks with null
          this.perceptionDebugCallbacks.forEach(callback => {
            try {
              callback(null);
            } catch (error) {
              console.error('[ElectronIPCClient] Error in perception debug callback:', error);
            }
          });
          return;
        }

        // Parse buffer layout:
        // [0]: has_data, [1]: target_id, [2]: x, [3]: y, [4]: range, [5]: query_radius, [6]: fov_angle, [7]: rotation, [8]: ax, [9]: ay, [10]: neighbor_count
        // [11..75]: neighbor_ids, [75..139]: neighbor_xs, [139..203]: neighbor_ys
        // [203]: cell_size, [204]: num_cells, [205]: creature_cell_x, [206]: creature_cell_y
        // [207..]: queried cells as (x, y) pairs
        const neighborCount = Math.min(buffer[10], MAX_DEBUG_NEIGHBORS);

        const neighbors: NeighborDebugInfo[] = [];
        const idOffset = HEADER_SIZE;
        const xOffset = HEADER_SIZE + MAX_DEBUG_NEIGHBORS;
        const yOffset = HEADER_SIZE + MAX_DEBUG_NEIGHBORS * 2;

        for (let i = 0; i < neighborCount; i++) {
          neighbors.push({
            id: buffer[idOffset + i],
            x: buffer[xOffset + i],
            y: buffer[yOffset + i],
          });
        }

        // Parse cell section
        const cellSize = buffer[CELL_SECTION_OFFSET];
        const numCells = Math.min(buffer[CELL_SECTION_OFFSET + 1], MAX_QUERIED_CELLS);
        const creatureCell: QueriedCell = {
          x: buffer[CELL_SECTION_OFFSET + 2],
          y: buffer[CELL_SECTION_OFFSET + 3],
        };

        const queriedCells: QueriedCell[] = [];
        const cellsOffset = CELL_SECTION_OFFSET + CELL_HEADER_SIZE;
        for (let i = 0; i < numCells; i++) {
          queriedCells.push({
            x: buffer[cellsOffset + i * 2],
            y: buffer[cellsOffset + i * 2 + 1],
          });
        }

        // Parse checked cells section
        const numCheckedCells = Math.min(buffer[CHECKED_CELL_SECTION_OFFSET], MAX_CHECKED_CELLS);
        const checkedCells: QueriedCell[] = [];
        const checkedCellsOffset = CHECKED_CELL_SECTION_OFFSET + CHECKED_CELL_HEADER_SIZE;
        for (let i = 0; i < numCheckedCells; i++) {
          checkedCells.push({
            x: buffer[checkedCellsOffset + i * 2],
            y: buffer[checkedCellsOffset + i * 2 + 1],
          });
        }

        // Parse L1 vision section
        const numL1Vision = Math.min(buffer[L1_VISION_SECTION_OFFSET], MAX_L1_VISION_ENTRIES);
        const l1Vision: L1VisionDebugEntry[] = [];
        const l1VisionDataOffset = L1_VISION_SECTION_OFFSET + L1_VISION_HEADER_SIZE;
        for (let i = 0; i < numL1Vision; i++) {
          const entryOffset = l1VisionDataOffset + i * L1_VISION_ENTRY_SIZE;
          l1Vision.push({
            cellIdx: buffer[entryOffset],
            classification: buffer[entryOffset + 1] as L1Classification,
            centerX: buffer[entryOffset + 2],
            centerY: buffer[entryOffset + 3],
            directionX: buffer[entryOffset + 4],
            directionY: buffer[entryOffset + 5],
          });
        }

        const debugData: PerceptionDebugData = {
          entityId: buffer[1],
          x: buffer[2],
          y: buffer[3],
          perceptionRange: buffer[4],
          queryRadius: buffer[5],
          fovAngle: buffer[6],
          rotation: buffer[7],
          ax: buffer[8],
          ay: buffer[9],
          neighbors,
          cellSize,
          creatureCell,
          queriedCells,
          checkedCells,
          l1Vision: l1Vision.length > 0 ? l1Vision : undefined,
        };

        this.perceptionDebugCallbacks.forEach(callback => {
          try {
            callback(debugData);
          } catch (error) {
            console.error('[ElectronIPCClient] Error in perception debug callback:', error);
          }
        });
      } catch (error) {
        console.error('[ElectronIPCClient] Failed to parse perception debug buffer:', error);
      }
    });
    this.unsubscribers.push(unsubPerception);
  }

  onStateUpdate(callback: (state: GameState) => void): () => void {
    if (typeof callback !== 'function') {
      throw new Error('ElectronIPCClient: callback must be a function');
    }

    this.stateCallbacks.add(callback);

    return () => {
      this.stateCallbacks.delete(callback);
    };
  }

  onTelemetryUpdate(callback: (telemetry: TelemetryFrame) => void): () => void {
    if (typeof callback !== 'function') {
      throw new Error('ElectronIPCClient: callback must be a function');
    }

    this.telemetryCallbacks.add(callback);

    return () => {
      this.telemetryCallbacks.delete(callback);
    };
  }

  onPerceptionDebugUpdate(callback: (data: PerceptionDebugData | null) => void): () => void {
    if (typeof callback !== 'function') {
      throw new Error('ElectronIPCClient: callback must be a function');
    }

    this.perceptionDebugCallbacks.add(callback);

    return () => {
      this.perceptionDebugCallbacks.delete(callback);
    };
  }

  getLatestState(): GameState | null {
    return this.latestState;
  }

  async disconnect(): Promise<void> {
    // Call all unsubscribe functions from connect()
    for (const unsub of this.unsubscribers) {
      try {
        unsub();
      } catch (error) {
        console.error('[ElectronIPCClient] Error in unsubscriber:', error);
      }
    }
    this.unsubscribers = [];

    // Also use legacy cleanup
    if (window.electron) {
      window.electron.removeStateUpdateListener();
    }
    this.stateCallbacks.clear();
    this.telemetryCallbacks.clear();
    this.perceptionDebugCallbacks.clear();
    this.latestState = null;
  }

  selectCreatureDebug(creatureId: number | null): void {
    window.electron?.selectCreatureDebug?.(creatureId);
  }
}
