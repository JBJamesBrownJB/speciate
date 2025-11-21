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
   * Subscribe to state updates from simulation (BINARY)
   * Callback receives Uint8Array of raw MessagePack data
   *
   * @param {Function} callback - Function to call with binary state updates
   */
  onStateUpdateBinary: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onStateUpdateBinary: callback must be a function');
    }

    ipcRenderer.on('state-update-binary', (event, binaryData) => {
      callback(binaryData);
    });
  },

  /**
   * Subscribe to telemetry updates from simulation (dev-tools only)
   * Callback receives plain JavaScript object with metrics
   *
   * @param {Function} callback - Function to call with telemetry updates
   */
  onTelemetryUpdate: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onTelemetryUpdate: callback must be a function');
    }

    ipcRenderer.on('telemetry-update', (event, telemetry) => {
      callback(telemetry);
    });
  },

  /**
   * Remove all state update listeners (cleanup)
   */
  removeStateUpdateListener: () => {
    ipcRenderer.removeAllListeners('state-update-binary');
    ipcRenderer.removeAllListeners('telemetry-update');
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

  /**
   * Send command to simulation (dev tools only)
   * Commands are validated and forwarded to simulation via stdin
   *
   * @param {Object} command - Command object with type and parameters
   */
  sendCommand: (command) => {
    if (typeof command !== 'object' || command === null) {
      throw new Error('sendCommand: command must be an object');
    }
    if (typeof command.type !== 'string') {
      throw new Error('sendCommand: command.type must be a string');
    }
    ipcRenderer.send('send-command', command);
  },

  /**
   * Save metrics snapshot to disk (dev tools only)
   * Opens save dialog with prepopulated path, then saves JSON file
   *
   * @param {Object} snapshot - Metrics snapshot object
   * @returns {Promise<{success: boolean, path?: string, error?: string}>}
   */
  saveMetricsSnapshot: async (snapshot) => {
    if (typeof snapshot !== 'object' || snapshot === null) {
      throw new Error('saveMetricsSnapshot: snapshot must be an object');
    }
    return await ipcRenderer.invoke('save-metrics-snapshot', snapshot);
  },

  /**
   * Load metrics snapshot from disk (dev tools only)
   * Opens file dialog to select snapshot JSON file
   *
   * @returns {Promise<Object|null>} Parsed snapshot object or null if cancelled
   */
  loadMetricsSnapshot: async () => {
    return await ipcRenderer.invoke('load-metrics-snapshot');
  },

  /**
   * Resize dev-tools window (dev tools only)
   * Changes the window width while preserving height
   *
   * @param {number} width - New window width in pixels
   */
  resizeWindow: async (width) => {
    if (typeof width !== 'number') {
      throw new Error('resizeWindow: width must be a number');
    }
    return await ipcRenderer.invoke('resize-window', width);
  },
});
