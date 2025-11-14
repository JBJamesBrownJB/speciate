import type { GameState } from '../../types/GameState';

/**
 * Platform-agnostic IPC client interface
 * Currently supports Electron desktop backend
 *
 * This abstraction decouples the frontend from specific IPC implementations,
 * enabling future support for browser-based backends if needed.
 */
export interface IPCClient {
  /**
   * Initialize connection to backend simulation
   * @throws Error if connection fails
   */
  connect(): Promise<void>;

  /**
   * Subscribe to state updates from simulation
   * Callback is invoked whenever backend emits new state
   *
   * @param callback - Function to call with new state
   * @returns Unsubscribe function to stop receiving updates
   */
  onStateUpdate(callback: (state: GameState) => void): () => void;

  /**
   * Get latest cached state (synchronous, non-blocking)
   * Used for high-frequency rendering (60+ FPS) from low-frequency updates (20 Hz)
   *
   * @returns Latest state or null if no state received yet
   */
  getLatestState(): GameState | null;

  /**
   * Disconnect from backend and cleanup resources
   */
  disconnect(): Promise<void>;
}
