/**
 * Global type declarations for Electron IPC
 */

import type { GameState } from './types/GameState';

interface Window {
  /**
   * Electron IPC API exposed via preload script
   * Only available when running in Electron context
   */
  electron?: {
    /**
     * Subscribe to state updates from simulation subprocess
     * Callback receives deserialized GameState object
     *
     * @param callback - Function to call with each state update
     */
    onStateUpdate: (callback: (state: GameState) => void) => void;

    /**
     * Remove all state update listeners (cleanup on unmount)
     */
    removeStateUpdateListener: () => void;

    /**
     * Get latest cached state (synchronous, non-blocking)
     * Returns null if no state has been received yet
     *
     * @returns Latest game state or null
     */
    getLatestState: () => Promise<GameState | null>;
  };
}
