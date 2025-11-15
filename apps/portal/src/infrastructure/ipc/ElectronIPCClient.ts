import type { IPCClient } from './IPCClient';
import type { GameState } from '../../types/GameState';

export class ElectronIPCClient implements IPCClient {
  private latestState: GameState | null = null;
  private stateCallbacks: Set<(state: GameState) => void> = new Set();

  async connect(): Promise<void> {
    if (!window.electron) {
      throw new Error('ElectronIPCClient: window.electron not available (not running in Electron)');
    }

    window.electron.onStateUpdate((state: GameState) => {
      this.latestState = state;

      this.stateCallbacks.forEach(callback => {
        try {
          callback(state);
        } catch (error) {
          console.error('[ElectronIPCClient] Error in state update callback:', error);
        }
      });
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
