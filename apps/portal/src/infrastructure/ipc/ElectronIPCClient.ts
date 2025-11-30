import type { IPCClient } from './IPCClient';
import type { GameState, CreatureData, PerceptionDebugData, NeighborDebugInfo } from '../../types/GameState';
import type { TelemetryFrame } from '../../types/TelemetryFrame';

const HEADER_SIZE = 6;
const MAX_DEBUG_NEIGHBORS = 64;

export class ElectronIPCClient implements IPCClient {
  private latestState: GameState | null = null;
  private stateCallbacks: Set<(state: GameState) => void> = new Set();
  private telemetryCallbacks: Set<(telemetry: TelemetryFrame) => void> = new Set();
  private perceptionDebugCallbacks: Set<(data: PerceptionDebugData | null) => void> = new Set();
  private _cachedTickRateHz = NaN;

  async connect(): Promise<void> {
    if (!window.electron) {
      throw new Error('ElectronIPCClient: window.electron not available (not running in Electron)');
    }

    // Listen for telemetry updates (tick rate, perception debug, etc.)
    window.electron.onTelemetryUpdate((telemetry) => {
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

    // Use new NAPI buffer updates
    window.electron.onNAPIBufferUpdate((data: { buffer: number[], creatureCount: number }) => {
      try {
        const { buffer, creatureCount } = data;

        // Parse SoA layout: [ID₁...IDₙ, X₁...Xₙ, Y₁...Yₙ, Rot₁...Rotₙ]
        const creatures: CreatureData[] = [];
        const xOffset = creatureCount;
        const yOffset = creatureCount * 2;
        const rotOffset = creatureCount * 3;

        for (let i = 0; i < creatureCount; i++) {
          creatures.push({
            id: buffer[i],
            x: buffer[xOffset + i],
            y: buffer[yOffset + i],
            rotation: buffer[rotOffset + i],
            size: 1.0, // Match BodySize::default() (NAPI doesn't provide this yet)
          });
        }

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

    // Listen for perception debug buffer updates (every tick)
    window.electron.onPerceptionDebugUpdate((buffer: Float32Array) => {
      try {
        // Parse buffer layout:
        // [0]: has_data, [1]: target_id, [2]: x, [3]: y, [4]: range, [5]: neighbor_count
        // [6..70]: neighbor_ids, [70..134]: neighbor_xs, [134..198]: neighbor_ys
        const neighborCount = Math.min(buffer[5], MAX_DEBUG_NEIGHBORS);

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

        const debugData: PerceptionDebugData = {
          entityId: buffer[1],
          x: buffer[2],
          y: buffer[3],
          perceptionRange: buffer[4],
          neighbors,
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
