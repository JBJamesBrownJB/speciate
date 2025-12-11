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
   * Subscribe to state updates from simulation (BINARY - OLD stdio IPC)
   * Callback receives Uint8Array of raw MessagePack data
   *
   * @param {Function} callback - Function to call with binary state updates
   * @returns {Function} Unsubscribe function
   * @deprecated Use onNAPIBufferUpdate instead
   */
  onStateUpdateBinary: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onStateUpdateBinary: callback must be a function');
    }

    const handler = (_event, binaryData) => callback(binaryData);
    ipcRenderer.on('state-update-binary', handler);
    return () => ipcRenderer.removeListener('state-update-binary', handler);
  },

  /**
   * Subscribe to NAPI buffer updates
   * Callback receives { buffer: number[], creatureCount: number }
   *
   * Buffer layout (SoA): [ID₁...IDₙ, X₁...Xₙ, Y₁...Yₙ, Rot₁...Rotₙ]
   *
   * @param {Function} callback - Function to call with NAPI buffer updates
   * @returns {Function} Unsubscribe function
   */
  onNAPIBufferUpdate: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onNAPIBufferUpdate: callback must be a function');
    }

    const handler = (_event, data) => callback(data);
    ipcRenderer.on('napi-buffer-update', handler);
    return () => ipcRenderer.removeListener('napi-buffer-update', handler);
  },

  /**
   * Subscribe to telemetry updates from simulation (dev-tools only)
   * Callback receives plain JavaScript object with metrics
   *
   * @param {Function} callback - Function to call with telemetry updates
   * @returns {Function} Unsubscribe function
   */
  onTelemetryUpdate: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onTelemetryUpdate: callback must be a function');
    }

    const handler = (_event, telemetry) => callback(telemetry);
    ipcRenderer.on('telemetry-update', handler);
    return () => ipcRenderer.removeListener('telemetry-update', handler);
  },

  /**
   * Remove all state update listeners (cleanup)
   */
  removeStateUpdateListener: () => {
    ipcRenderer.removeAllListeners('state-update-binary');
    ipcRenderer.removeAllListeners('telemetry-update');
    ipcRenderer.removeAllListeners('napi-buffer-update');
    ipcRenderer.removeAllListeners('perception-debug-update');
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

  /**
   * Select a creature for perception debug visualization (dev tools only)
   * When selected, telemetry will include detailed perception data
   *
   * @param {number|null} creatureId - Creature ID to select, or null to clear
   */
  selectCreatureDebug: (creatureId) => {
    if (creatureId !== null && typeof creatureId !== 'number') {
      throw new Error('selectCreatureDebug: creatureId must be a number or null');
    }
    ipcRenderer.send('select-creature-debug', creatureId);
  },

  /**
   * Subscribe to perception debug buffer updates
   * Callback receives Float32Array with perception debug data
   *
   * Buffer layout:
   * - [0]: has_data (1.0 = valid)
   * - [1]: target_id
   * - [2]: target_x
   * - [3]: target_y
   * - [4]: perception_range
   * - [5]: fov_angle (radians)
   * - [6]: rotation (radians)
   * - [7]: ax (acceleration x)
   * - [8]: ay (acceleration y)
   * - [9]: neighbor_count
   * - [10..74]: neighbor_ids (max 64)
   * - [74..138]: neighbor_xs
   * - [138..202]: neighbor_ys
   *
   * @param {Function} callback - Function to call with perception debug buffer
   * @returns {Function} Unsubscribe function
   */
  onPerceptionDebugUpdate: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onPerceptionDebugUpdate: callback must be a function');
    }

    const handler = (_event, buffer) => callback(buffer);
    ipcRenderer.on('perception-debug-update', handler);
    return () => ipcRenderer.removeListener('perception-debug-update', handler);
  },
});
