import type { IPCClient } from './IPCClient';
import type { GameState } from '../../types/GameState';

// Note: window.electron types are defined in src/global.d.ts

/**
 * Electron IPC client implementation
 *
 * Uses secure contextBridge API exposed by preload.js
 * Deserializes state updates and manages subscriptions
 */
export class ElectronIPCClient implements IPCClient {
  private latestState: GameState | null = null;
  private stateCallbacks: Set<(state: GameState) => void> = new Set();

  async connect(): Promise<void> {
    if (!window.electron) {
      throw new Error('ElectronIPCClient: window.electron not available (not running in Electron)');
    }

    // Subscribe to state updates from main process
    window.electron.onStateUpdate((state: GameState) => {
      // Cache latest state for synchronous access
      this.latestState = state;

      // Notify all subscribers
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
    // Validate callback
    if (typeof callback !== 'function') {
      throw new Error('ElectronIPCClient: callback must be a function');
    }

    // Add to subscribers
    this.stateCallbacks.add(callback);

    // Return unsubscribe function
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
