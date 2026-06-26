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
   * Host OS, so the dev-ui can label Linux-only metrics and gate Windows-only panels.
   * One of 'win32' | 'darwin' | 'linux' | ... (Node's process.platform values).
   */
  platform: process.platform,

  /**
   * DEV-only: forward render-pipeline metrics from the portal (game window) to the
   * main process, which relays them to the dev-tools window. Renderer-origin metrics
   * (interpolation cadence) don't ride the Rust telemetry channel, so they need this hop.
   *
   * @param {Object} metrics - RenderPipelineMetrics snapshot
   */
  sendRenderMetrics: (metrics) => {
    if (typeof metrics !== 'object' || metrics === null) return;
    ipcRenderer.send('render-metrics', metrics);
  },

  /**
   * Subscribe to render-pipeline metric updates (dev-tools window).
   *
   * @param {Function} callback - called with each RenderPipelineMetrics snapshot
   * @returns {Function} Unsubscribe function
   */
  onRenderMetricsUpdate: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onRenderMetricsUpdate: callback must be a function');
    }
    const handler = (_event, metrics) => callback(metrics);
    ipcRenderer.on('render-metrics-update', handler);
    return () => ipcRenderer.removeListener('render-metrics-update', handler);
  },

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
   * Subscribe to P0 plant grid snapshot updates.
   * Callback receives a Float32Array in sparse format:
   *   [count, x₀, y₀, density₀, type₀, x₁, y₁, density₁, type₁, ...]
   * Pushed at startup and after each CA tick (~every 2s). Frontend should cache
   * the last snapshot and re-render on each update.
   *
   * @param {Function} callback - called with each Float32Array snapshot
   * @returns {Function} Unsubscribe function
   */
  onPlantBufferUpdate: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onPlantBufferUpdate: callback must be a function');
    }
    const handler = (_event, buffer) => callback(buffer);
    ipcRenderer.on('plant-buffer-update', handler);
    return () => ipcRenderer.removeListener('plant-buffer-update', handler);
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

  /**
   * Set simulation pause state
   *
   * @param {boolean} paused - true to pause, false to resume
   */
  setPaused: (paused) => {
    if (typeof paused !== 'boolean') {
      throw new Error('setPaused: paused must be a boolean');
    }
    ipcRenderer.send('set-paused', paused);
  },

  /**
   * Set simulation time scale
   *
   * @param {number} scale - time scale (1.0 = normal, 2.0 = 2x speed, 0.5 = half speed)
   */
  setTimeScale: (scale) => {
    if (typeof scale !== 'number') {
      throw new Error('setTimeScale: scale must be a number');
    }
    ipcRenderer.send('set-time-scale', scale);
  },

  /**
   * Set cognitive system update frequency divisor (dev tools only)
   *
   * Controls how often perception, behavior, and steering systems run.
   * divisor=1 means every tick (full rate), divisor=2 means every 2nd tick, etc.
   *
   * @param {string} systemName - 'perception', 'behavior', or 'steering'
   * @param {number} divisor - frequency divisor (1-10)
   */
  setSystemFrequency: (systemName, divisor) => {
    if (typeof systemName !== 'string') {
      throw new Error('setSystemFrequency: systemName must be a string');
    }
    if (typeof divisor !== 'number') {
      throw new Error('setSystemFrequency: divisor must be a number');
    }
    ipcRenderer.send('set-system-frequency', { systemName, divisor });
  },

  /**
   * Set viewport bounds for backend culling
   *
   * When set, the backend only exports creatures within these bounds,
   * reducing IPC bandwidth and GPU work when zoomed in.
   *
   * @param {Object} bounds - Viewport bounds in world units
   * @param {number} bounds.minX - Left edge
   * @param {number} bounds.minY - Bottom edge
   * @param {number} bounds.maxX - Right edge
   * @param {number} bounds.maxY - Top edge
   * @param {number} bounds.margin - Extra padding (prevents pop-in at edges)
   */
  setViewportBounds: (bounds) => {
    if (typeof bounds !== 'object' || bounds === null) {
      throw new Error('setViewportBounds: bounds must be an object');
    }
    if (typeof bounds.minX !== 'number' || typeof bounds.minY !== 'number' ||
        typeof bounds.maxX !== 'number' || typeof bounds.maxY !== 'number' ||
        typeof bounds.margin !== 'number') {
      throw new Error('setViewportBounds: all bounds properties must be numbers');
    }
    ipcRenderer.send('set-viewport-bounds', bounds);
  },

  /**
   * Query L1 cell metadata at world position (dev-tools only)
   *
   * Returns cell info if the cell contains creatures, null otherwise.
   *
   * @param {number} worldX - X coordinate in world units
   * @param {number} worldY - Y coordinate in world units
   * @returns {Promise<{cellX: number, cellY: number, creatureCount: number, totalMass: number, maxSize: number, avgSize: number, cellSize: number} | null>}
   */
  queryL1Cell: async (worldX, worldY) => {
    if (typeof worldX !== 'number' || typeof worldY !== 'number') {
      throw new Error('queryL1Cell: worldX and worldY must be numbers');
    }
    return await ipcRenderer.invoke('query-l1-cell', worldX, worldY);
  },

  /**
   * Spawn a plant at the given world position (P0 mode click).
   * The position is snapped to the nearest P0 cell on the Rust side.
   *
   * @param {number} worldX - X coordinate in world units
   * @param {number} worldY - Y coordinate in world units
   */
  spawnPlant: (worldX, worldY) => {
    if (!Number.isFinite(worldX) || !Number.isFinite(worldY)) {
      throw new Error('spawnPlant: worldX and worldY must be finite numbers');
    }
    ipcRenderer.send('spawn-plant', { worldX, worldY });
  },
});
