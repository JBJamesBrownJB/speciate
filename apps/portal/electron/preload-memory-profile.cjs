const { contextBridge, ipcRenderer } = require('electron');

/**
 * Extended preload script for Memory Profiling Mode
 *
 * Adds V8 heap profiling IPC handlers on top of standard handlers
 */

contextBridge.exposeInMainWorld('electron', {
  onStateUpdateBinary: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onStateUpdateBinary: callback must be a function');
    }
    ipcRenderer.on('state-update-binary', (event, binaryData) => {
      callback(binaryData);
    });
  },

  onNAPIBufferUpdate: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onNAPIBufferUpdate: callback must be a function');
    }
    console.log('[Preload] onNAPIBufferUpdate registered');
    ipcRenderer.on('napi-buffer-update', (event, data) => {
      callback(data);
    });
  },

  onTelemetryUpdate: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onTelemetryUpdate: callback must be a function');
    }
    ipcRenderer.on('telemetry-update', (event, telemetry) => {
      callback(telemetry);
    });
  },

  removeStateUpdateListener: () => {
    ipcRenderer.removeAllListeners('state-update-binary');
    ipcRenderer.removeAllListeners('telemetry-update');
    ipcRenderer.removeAllListeners('napi-buffer-update');
    ipcRenderer.removeAllListeners('perception-debug-update');
    ipcRenderer.removeAllListeners('memory-update');
  },

  getLatestState: async () => {
    return await ipcRenderer.invoke('get-latest-state');
  },

  sendCommand: (command) => {
    if (typeof command !== 'object' || command === null) {
      throw new Error('sendCommand: command must be an object');
    }
    if (typeof command.type !== 'string') {
      throw new Error('sendCommand: command.type must be a string');
    }
    ipcRenderer.send('send-command', command);
  },

  saveMetricsSnapshot: async (snapshot) => {
    if (typeof snapshot !== 'object' || snapshot === null) {
      throw new Error('saveMetricsSnapshot: snapshot must be an object');
    }
    return await ipcRenderer.invoke('save-metrics-snapshot', snapshot);
  },

  loadMetricsSnapshot: async () => {
    return await ipcRenderer.invoke('load-metrics-snapshot');
  },

  resizeWindow: async (width) => {
    if (typeof width !== 'number') {
      throw new Error('resizeWindow: width must be a number');
    }
    return await ipcRenderer.invoke('resize-window', width);
  },

  selectCreatureDebug: (creatureId) => {
    if (creatureId !== null && typeof creatureId !== 'number') {
      throw new Error('selectCreatureDebug: creatureId must be a number or null');
    }
    ipcRenderer.send('select-creature-debug', creatureId);
  },

  onPerceptionDebugUpdate: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onPerceptionDebugUpdate: callback must be a function');
    }
    ipcRenderer.on('perception-debug-update', (event, buffer) => {
      callback(buffer);
    });
  },

  /**
   * MEMORY PROFILING HANDLERS (added for memory profiling mode)
   */

  /**
   * Subscribe to V8 memory updates
   * Callback receives {timestamp, rss, heapTotal, heapUsed, external, arrayBuffers}
   *
   * @param {Function} callback - Function to call with memory snapshots
   */
  onMemoryUpdate: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('onMemoryUpdate: callback must be a function');
    }
    console.log('[Preload] onMemoryUpdate registered');
    ipcRenderer.on('memory-update', (event, snapshot) => {
      callback(snapshot);
    });
  },

  /**
   * Remove memory update listener
   */
  removeMemoryUpdateListener: (callback) => {
    if (callback) {
      ipcRenderer.removeListener('memory-update', callback);
    } else {
      ipcRenderer.removeAllListeners('memory-update');
    }
  },

  /**
   * Trigger manual garbage collection (requires --expose-gc)
   * Main process will log before/after memory stats
   */
  triggerGC: () => {
    console.log('[Preload] Triggering manual GC...');
    ipcRenderer.send('trigger-gc');
  },

  /**
   * Take V8 heap snapshot
   * Returns {success: boolean, path?: string, error?: string}
   *
   * @returns {Promise<Object>} Result object with path to .heapsnapshot file
   */
  takeHeapSnapshot: async () => {
    console.log('[Preload] Taking heap snapshot...');
    return await ipcRenderer.invoke('take-heap-snapshot');
  },
});
