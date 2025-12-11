import type { IPCClient } from './IPCClient';
import type { GameState, CreatureData, PerceptionDebugData, NeighborDebugInfo, QueriedCell } from '../../types/GameState';
import type { TelemetryFrame } from '../../types/TelemetryFrame';

const HEADER_SIZE = 8;
const MAX_DEBUG_NEIGHBORS = 64;
const CELL_SECTION_OFFSET = HEADER_SIZE + MAX_DEBUG_NEIGHBORS * 3; // 200
const CELL_HEADER_SIZE = 4;
const MAX_QUERIED_CELLS = 100;
const CHECKED_CELL_SECTION_OFFSET = CELL_SECTION_OFFSET + CELL_HEADER_SIZE + MAX_QUERIED_CELLS * 2; // 404
const CHECKED_CELL_HEADER_SIZE = 1;
const MAX_CHECKED_CELLS = 100;

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
    const unsubBuffer = window.electron.onNAPIBufferUpdate((data: { buffer: number[], creatureCount: number }) => {
      try {
        const { buffer, creatureCount } = data;

        // Ensure pre-allocated array has capacity (one-time allocation)
        this.ensureCreatureCapacity(creatureCount);

        // Parse SoA layout: [ID₁...IDₙ, X₁...Xₙ, Y₁...Yₙ, Rot₁...Rotₙ]
        // Update objects IN-PLACE (zero allocations per tick)
        const xOffset = creatureCount;
        const yOffset = creatureCount * 2;
        const rotOffset = creatureCount * 3;

        for (let i = 0; i < creatureCount; i++) {
          const creature = this.creatures[i];
          creature.id = buffer[i];
          creature.x = buffer[xOffset + i];
          creature.y = buffer[yOffset + i];
          creature.rotation = buffer[rotOffset + i];
        }

        // Return view of active creatures (slice creates new array ref, but objects are reused)
        const creatures = this.creatures.slice(0, creatureCount);

        const state: GameState = {
          protocolVersion: 2, // NAPI protocol version
          tick: 0, // Will be provided by separate telemetry
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
        // Parse buffer layout:
        // [0]: has_data, [1]: target_id, [2]: x, [3]: y, [4]: range, [5]: fov_angle, [6]: rotation, [7]: neighbor_count
        // [8..72]: neighbor_ids, [72..136]: neighbor_xs, [136..200]: neighbor_ys
        // [200]: cell_size, [201]: num_cells, [202]: creature_cell_x, [203]: creature_cell_y
        // [204..]: queried cells as (x, y) pairs
        const neighborCount = Math.min(buffer[7], MAX_DEBUG_NEIGHBORS);

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

        const debugData: PerceptionDebugData = {
          entityId: buffer[1],
          x: buffer[2],
          y: buffer[3],
          perceptionRange: buffer[4],
          fovAngle: buffer[5],
          rotation: buffer[6],
          neighbors,
          cellSize,
          creatureCell,
          queriedCells,
          checkedCells,
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
    if (window.electron?.selectCreatureDebug) {
      window.electron.selectCreatureDebug(creatureId);
    }
  }
}
