import type { IPCClient } from './IPCClient';
import type { GameState, CreatureData } from '../../types/GameState';

export class ElectronIPCClient implements IPCClient {
  private latestState: GameState | null = null;
  private stateCallbacks: Set<(state: GameState) => void> = new Set();
  private _cachedTickRateHz = NaN; // Default NaN until first telemetry sample

  async connect(): Promise<void> {
    if (!window.electron) {
      throw new Error('ElectronIPCClient: window.electron not available (not running in Electron)');
    }

    // Listen for telemetry updates (tick rate, etc.)
    window.electron.onTelemetryUpdate((telemetry) => {
      if (telemetry.tickRateHz !== undefined) {
        this._cachedTickRateHz = telemetry.tickRateHz;
      }
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

  getLatestState(): GameState | null {
    return this.latestState;
  }

  async disconnect(): Promise<void> {
    if (window.electron) {
      window.electron.removeStateUpdateListener();
    }
    this.stateCallbacks.clear();
    this.latestState = null;
  }
}
