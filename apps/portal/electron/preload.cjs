const { contextBridge, ipcRenderer } = require('electron');

/**
 * Secure IPC bridge using contextBridge
 *
 * IMPORTANT: Never expose entire ipcRenderer to renderer!
 * Only expose specific, controlled methods to prevent security vulnerabilities.
 *
 * References:
 * - https://www.electronjs.org/docs/latest/tutorial/context-isolation
 * - https://www.electronjs.org/docs/latest/api/context-bridge
 */

// Expose ONLY specific methods to renderer process
contextBridge.exposeInMainWorld('electron', {
  /**
   * Subscribe to state updates from simulation
   * Callback receives plain JS object (not binary data)
   *
   * @param {Function} callback - Function to call with state updates
   */
  onStateUpdate: (callback) => {
    // Validate callback is a function
    if (typeof callback !== 'function') {
      throw new Error('onStateUpdate: callback must be a function');
    }

    // Listen for state-update events from main process
    ipcRenderer.on('state-update', (event, state) => {
      callback(state);
    });
  },

  /**
   * Remove all state update listeners (cleanup)
   */
  removeStateUpdateListener: () => {
    ipcRenderer.removeAllListeners('state-update');
  },

  /**
   * Get latest cached state (synchronous, non-blocking)
   * Returns null if no state received yet
   *
   * @returns {Promise<Object|null>} Latest game state or null
   */
  getLatestState: async () => {
    return await ipcRenderer.invoke('get-latest-state');
  },
});
