const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const { spawn } = require('child_process');
const { decode, encode } = require('@msgpack/msgpack');
const path = require('path');
const fs = require('fs');

let mainWindow;
let devToolsWindow = null;
let simulationProcess;
let latestState = null;
let shuttingDown = false;  // Flag to prevent duplicate shutdown handling

// Environment detection
const isDev = process.env.NODE_ENV === 'development';
const platform = process.platform;

console.log(`[Electron] Mode: ${isDev ? 'DEVELOPMENT' : 'PRODUCTION'}`);

/**
 * Spawn Rust simulation binary using child_process.spawn
 * Direct binary execution for native performance
 */
function startSimulation() {
  // Path to Rust binary (debug in dev, release in production)
  // Use RUST_BUILD_TYPE env var to override, otherwise auto-detect
  const buildType = process.env.RUST_BUILD_TYPE || (isDev ? 'debug' : 'release');
  const binaryName = platform === 'win32' ? 'speciate.exe' : 'speciate';
  const binaryPath = path.join(__dirname, '../../simulation/target', buildType, binaryName);

  console.log(`[Electron] Rust binary: ${binaryPath}`);

  // Verify binary exists
  if (!fs.existsSync(binaryPath)) {
    console.error(`\n❌ Simulation binary not found at: ${binaryPath}\n`);
    if (isDev) {
      console.error('Development mode: Build debug binary with:');
      console.error('  npm run dev:rust  (30 seconds)\n');
    } else {
      console.error('Production mode: Build release binary with:');
      console.error('  npm run build:rust  (3-5 minutes)\n');
    }
    console.error('Or run first-time setup:');
    console.error('  npm run setup\n');
    app.quit();
    return;
  }

  // Spawn Rust binary as child process with auto-resume from last session
  // --load-snapshot (no path) → auto-discovers most recent snapshot in ./snapshots/
  // If no snapshots found → gracefully falls back to default config
  simulationProcess = spawn(binaryPath, ['--load-snapshot'], {
    stdio: ['pipe', 'pipe', 'pipe'],  // stdin (pipe for commands), stdout (pipe), stderr (pipe)
  });

  let buffers = [];
  let totalLength = 0;

  // Read stdout frames (length-prefixed MessagePack)
  simulationProcess.stdout.on('data', (chunk) => {
    buffers.push(chunk);
    totalLength += chunk.length;

    // Process all complete frames in buffer
    while (totalLength >= 4) {
      // Concatenate only when we need to read
      let buffer = buffers.length === 1 ? buffers[0] : Buffer.concat(buffers);

      // Read 4-byte length prefix (big-endian u32)
      const frameLength = buffer.readUInt32BE(0);

      // Wait for complete frame
      if (totalLength < 4 + frameLength) {
        // Store back if we concatenated
        if (buffers.length > 1) {
          buffers = [buffer];
        }
        break;
      }

      // Extract MessagePack payload
      const payload = buffer.slice(4, 4 + frameLength);

      // Keep remainder
      const remainder = buffer.slice(4 + frameLength);
      buffers = remainder.length > 0 ? [remainder] : [];
      totalLength = remainder.length;

      try {
        // Send raw binary to portal (zero-copy, avoid re-encoding overhead)
        const binaryData = new Uint8Array(payload);

        if (mainWindow && !mainWindow.isDestroyed()) {
          mainWindow.webContents.send('state-update-binary', binaryData);
        }

        // Decode ONLY for telemetry extraction (dev-ui) and latestState
        const state = decode(payload);

        // Send lightweight telemetry to dev-ui (no creature positions!)
        if (devToolsWindow && !devToolsWindow.isDestroyed()) {
          const telemetry = {
            tick: state.tick,
            creatureCount: state.creatures?.length || 0,
            tickRateHz: state.tickRateHz,
            systemTimingsUs: state.systemTimingsUs,
            hardwareMetrics: state.hardwareMetrics,
            parallelizationMetrics: state.parallelizationMetrics,
          };
          devToolsWindow.webContents.send('telemetry-update', telemetry);
        }

        // Cache decoded state for latestState IPC handler
        latestState = state;
      } catch (error) {
        console.error('[Electron] Failed to process MessagePack:', error);
      }
    }
  });

  // Log stderr output (errors and debug messages from Rust)
  simulationProcess.stderr.on('data', (data) => {
    console.error('[Simulation stderr]', data.toString());
  });

  // Handle process exit
  simulationProcess.on('exit', (code) => {
    if (code !== 0 && code !== null) {
      console.error(`[Electron] Simulation process exited unexpectedly with code ${code}`);
    }
    simulationProcess = null;
  });

  // Handle spawn errors
  simulationProcess.on('error', (error) => {
    console.error('[Electron] Failed to spawn simulation process:', error);
  });
}

/**
 * Create main application window
 */
async function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1600,
    height: 1000,
    x: 0,
    y: 0,
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,          // Security: Isolate renderer from main process
      nodeIntegration: false,           // Security: Disable Node.js in renderer
      sandbox: false,                   // CRITICAL: Disabled for Linux ESRCH crash workaround
      webSecurity: true,                // Security: Enforce same-origin policy
      allowRunningInsecureContent: false, // Security: Block mixed content
      devTools: isDev,                  // Enable DevTools only in development
    },
  });

  // Load frontend (Vite dev server in dev, dist/ in production)
  if (isDev) {
    const viteURL = 'http://localhost:5173';
    console.log(`[Electron] Dev mode: Loading from Vite dev server at ${viteURL}`);

    // Retry connection to Vite (it might not be ready yet)
    let retries = 0;
    const maxRetries = 10;

    while (retries < maxRetries) {
      try {
        await mainWindow.loadURL(viteURL);
        console.log('[Electron] ✅ Connected to Vite dev server');
        break;
      } catch (err) {
        retries++;
        if (retries >= maxRetries) {
          console.error(`\n❌ Failed to connect to Vite dev server after ${maxRetries} attempts!`);
          console.error('\nMake sure Vite is running:');
          console.error('  npm run dev:vite\n');
          console.error('Or use the combined command:');
          console.error('  npm run dev\n');
          app.quit();
          return;
        }
        // Silently retry Vite connection
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
  } else {
    // Production mode: Load from dist/
    const htmlPath = path.join(__dirname, '../dist/index.html');

    if (!fs.existsSync(htmlPath)) {
      console.error(`\n❌ Frontend dist/ not found at: ${htmlPath}\n`);
      console.error('Build the frontend first:');
      console.error('  npm run build:frontend\n');
      app.quit();
      return;
    }

    console.log('[Electron] Production mode: Loading from dist/');
    await mainWindow.loadFile(htmlPath);
    console.log('[Electron] ✅ HTML loaded successfully');
  }

  // Forward renderer console messages to main process
  mainWindow.webContents.on('console-message', (event, level, message, line, sourceId) => {
    const levels = ['', 'INFO', 'WARNING', 'ERROR'];
    console.log(`[Renderer ${levels[level]}] ${message} (${sourceId}:${line})`);
  });

  // Handle window close
  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

/**
 * Create dev tools window (launched with --dev-tools flag)
 */
async function createDevToolsWindow() {
  devToolsWindow = new BrowserWindow({
    width: 750,
    height: 1300,
    x: 1920,
    y: 0,
    title: 'Speciate Dev Tools',
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
      webSecurity: true,
      devTools: isDev,
    },
  });

  // Load dev tools page from dev-ui Vite server
  if (isDev) {
    const viteURL = 'http://localhost:5174';
    console.log(`[Electron] Dev Tools: Loading from dev-ui Vite at ${viteURL}`);

    // Retry connection (dev-ui Vite might not be ready yet)
    let retries = 0;
    const maxRetries = 10;

    while (retries < maxRetries) {
      try {
        await devToolsWindow.loadURL(viteURL);
        console.log('[Electron] ✅ Dev Tools window loaded');
        break;
      } catch (err) {
        retries++;
        if (retries >= maxRetries) {
          console.error(`\n❌ Failed to connect to dev-ui Vite server after ${maxRetries} attempts!`);
          console.error('\nMake sure dev-ui Vite is running on port 5174\n');
          devToolsWindow.close();
          devToolsWindow = null;
          return;
        }
        // Silently retry
        await new Promise(resolve => setTimeout(resolve, 500));
      }
    }
  } else {
    const htmlPath = path.join(__dirname, '../dist/dev-tools.html');
    if (fs.existsSync(htmlPath)) {
      await devToolsWindow.loadFile(htmlPath);
      console.log('[Electron] ✅ Dev Tools window loaded (production)');
    } else {
      console.warn('[Electron] Dev tools HTML not found in dist/');
      devToolsWindow.close();
      devToolsWindow = null;
    }
  }

  if (devToolsWindow) {
    devToolsWindow.on('closed', () => {
      devToolsWindow = null;
    });
  }
}

/**
 * IPC Command Validation Framework
 *
 * Future-proof validation for bidirectional IPC commands (Phase 2+)
 * Current: Only read-only 'get-latest-state' exists
 * Future: spawn_creature, set_camera_zoom, etc. will use this pattern
 */
const COMMAND_VALIDATORS = {
  // Dev tools commands (Phase 1C)
  dev_spawn_creature: (params) => {
    if (typeof params !== 'object' || params === null) {
      throw new Error('dev_spawn_creature: params must be an object');
    }
    if (typeof params.x !== 'number' || typeof params.y !== 'number') {
      throw new Error('dev_spawn_creature: x and y must be numbers');
    }
    if (!Number.isFinite(params.x) || !Number.isFinite(params.y)) {
      throw new Error('dev_spawn_creature: x and y must be finite numbers');
    }
    // Bounds checking (world is ±1M units)
    if (Math.abs(params.x) > 1_000_000 || Math.abs(params.y) > 1_000_000) {
      throw new Error('dev_spawn_creature: coordinates out of world bounds');
    }
  },

  dev_load_trial: (params) => {
    if (typeof params !== 'object' || params === null) {
      throw new Error('dev_load_trial: params must be an object');
    }
    if (typeof params.template !== 'string' || params.template.length === 0) {
      throw new Error('dev_load_trial: template must be a non-empty string');
    }
    // Basic path traversal prevention
    if (params.template.includes('..') || params.template.includes('/')) {
      throw new Error('dev_load_trial: template name contains invalid characters');
    }
  },

  dev_clear_creatures: (params) => {
    // No parameters required for clear command, just validate object exists
    if (typeof params !== 'object' || params === null) {
      throw new Error('dev_clear_creatures: params must be an object');
    }
    // Command is just { type: 'dev_clear_creatures' } - no additional validation needed
  },

  dev_clear_plants: (params) => {
    if (typeof params !== 'object' || params === null) {
      throw new Error('dev_clear_plants: params must be an object');
    }
  },
};

/**
 * Generic validated command handler for dev tools IPC
 *
 * Usage from renderer:
 *   await window.electron.sendCommand({ type: 'dev_spawn_creature', x: 100, y: 200 })
 *
 * Security: Whitelist + parameter validation prevents injection attacks
 * Performance: ~100ns overhead, not in 60 Hz streaming path (zero impact)
 */
ipcMain.on('send-command', (event, command) => {
  if (!simulationProcess || !simulationProcess.stdin) {
    console.error('[Electron] Cannot send command: simulation not running');
    return;
  }

  const commandType = command.type;
  const validator = COMMAND_VALIDATORS[commandType];

  if (!validator) {
    console.error(`[Electron] Unknown command type: ${commandType}`);
    return;
  }

  // Parameter validation
  try {
    validator(command);
  } catch (err) {
    console.error(`[Electron] Command validation failed: ${err.message}`);
    return;
  }

  // Serialize command to MessagePack
  try {
    const payload = encode(command);
    const length = Buffer.alloc(4);
    length.writeUInt32BE(payload.length, 0);

    // Write length-prefixed frame to stdin
    simulationProcess.stdin.write(length);
    simulationProcess.stdin.write(payload);

    console.log(`[Electron] Sent command: ${commandType}`, command);
  } catch (err) {
    console.error('[Electron] Failed to send command:', err);
  }
});

/**
 * IPC handler: Get latest state (synchronous polling fallback)
 *
 * NOTE: Primary state delivery is via 'state-update' events (60 Hz push)
 * This handler is for synchronous polling when event-driven updates aren't suitable
 */
ipcMain.handle('get-latest-state', () => {
  return latestState;
});

/**
 * IPC handler: Save metrics snapshot (dev tools only)
 *
 * Opens save dialog with prepopulated path and filename
 * Writes JSON snapshot to user-selected location
 */
ipcMain.handle('save-metrics-snapshot', async (event, snapshot) => {
  try {
    const timestamp = new Date(snapshot.metadata.endTime);
    const dateStr = timestamp.toISOString().replace(/:/g, '-').split('.')[0];
    const defaultFilename = `snapshot_${dateStr}.json`;

    const snapshotsDir = path.join(__dirname, '../../..', 'docs/performance/snapshots');
    const defaultPath = path.join(snapshotsDir, defaultFilename);

    const result = await dialog.showSaveDialog(devToolsWindow || mainWindow, {
      title: 'Save Metrics Snapshot',
      defaultPath: defaultPath,
      filters: [
        { name: 'JSON Files', extensions: ['json'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    });

    if (result.canceled || !result.filePath) {
      return { success: false, error: 'Save cancelled' };
    }

    const jsonData = JSON.stringify(snapshot, null, 2);
    fs.writeFileSync(result.filePath, jsonData, 'utf8');

    console.log(`[Electron] Metrics snapshot saved to: ${result.filePath}`);
    return { success: true, path: result.filePath };
  } catch (error) {
    console.error('[Electron] Failed to save metrics snapshot:', error);
    return { success: false, error: error.message };
  }
});

/**
 * IPC handler: Load metrics snapshot (dev tools only)
 *
 * Opens file dialog to select snapshot JSON file
 * Reads and parses snapshot, returns to renderer
 */
ipcMain.handle('load-metrics-snapshot', async (event) => {
  try {
    const snapshotsDir = path.join(__dirname, '../../..', 'docs/performance/snapshots');

    const result = await dialog.showOpenDialog(devToolsWindow || mainWindow, {
      title: 'Load Metrics Snapshot',
      defaultPath: snapshotsDir,
      filters: [
        { name: 'JSON Files', extensions: ['json'] },
        { name: 'All Files', extensions: ['*'] }
      ],
      properties: ['openFile']
    });

    if (result.canceled || result.filePaths.length === 0) {
      return null;
    }

    const filePath = result.filePaths[0];
    const jsonData = fs.readFileSync(filePath, 'utf8');
    const snapshot = JSON.parse(jsonData);

    console.log(`[Electron] Metrics snapshot loaded from: ${filePath}`);
    return snapshot;
  } catch (error) {
    console.error('[Electron] Failed to load metrics snapshot:', error);
    return null;
  }
});

/**
 * IPC handler: Resize dev tools window (dev tools only)
 *
 * Changes the window width while preserving height
 * Used for dynamic resizing when loading/clearing snapshots
 */
ipcMain.handle('resize-window', async (event, width) => {
  try {
    if (devToolsWindow && !devToolsWindow.isDestroyed()) {
      const [currentWidth, currentHeight] = devToolsWindow.getSize();
      devToolsWindow.setSize(width, currentHeight);
      console.log(`[Electron] Dev tools window resized to ${width}x${currentHeight}`);
      return { success: true };
    }
    return { success: false, error: 'Dev tools window not available' };
  } catch (error) {
    console.error('[Electron] Failed to resize window:', error);
    return { success: false, error: error.message };
  }
});

/**
 * Linux Sandbox Workaround
 * Disable Chromium sandbox to fix SUID permission errors
 * Keep GPU ENABLED for PixiJS WebGL rendering
 */
// Linux sandbox workaround (fixes SUID errors)
app.commandLine.appendSwitch('no-sandbox');
app.commandLine.appendSwitch('disable-gpu-sandbox');

// Shared memory fix
app.commandLine.appendSwitch('disable-dev-shm-usage');

// Debugging
app.commandLine.appendSwitch('enable-logging');
app.commandLine.appendSwitch('log-level', '0'); // Verbose logging

/**
 * App lifecycle: Ready event
 */
app.whenReady().then(() => {
  createWindow();
  startSimulation();

  // Auto-launch dev tools in development mode
  if (isDev) {
    console.log('[Electron] Development mode: launching dev tools window');
    createDevToolsWindow();
  }

  // macOS: Re-create window when dock icon is clicked
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

/**
 * App lifecycle: All windows closed
 */
app.on('window-all-closed', () => {
  // macOS: Apps stay active until user quits explicitly
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

/**
 * App lifecycle: Before quit (with delay for graceful shutdown)
 */
app.on('before-quit', (event) => {
  if (!simulationProcess || shuttingDown) {
    return; // Already shutting down or no process to kill
  }

  event.preventDefault(); // Prevent immediate quit, wait for graceful shutdown
  shuttingDown = true;

  console.log('[Electron] Quitting app, sending graceful shutdown signal to simulation...');

  // Send SIGINT (Ctrl+C) to trigger graceful shutdown with snapshot saving
  // (Don't use default SIGTERM - it kills immediately without cleanup)
  simulationProcess.kill('SIGINT');

  // Wait for process to exit, then quit
  simulationProcess.on('exit', (code, signal) => {
    console.log(`[Electron] Simulation process exited with code ${code}, signal ${signal}`);
    simulationProcess = null;
    shuttingDown = false;
    app.quit(); // Now actually quit
  });

  // Timeout fallback (force quit after 5 seconds if process doesn't exit)
  setTimeout(() => {
    if (simulationProcess) {
      console.log('[Electron] Force-killing simulation process (timeout)');
      simulationProcess.kill('SIGKILL');
      simulationProcess = null;
      shuttingDown = false;
      app.quit();
    }
  }, 5000);
});

/**
 * Crash reporter: Log detailed crash information
 */
app.on('render-process-gone', (event, webContents, details) => {
  console.error('[Electron] 💥 Renderer process crashed!');
  console.error('[Electron] Crash details:', JSON.stringify(details, null, 2));
  console.error('[Electron] Crash reason:', details.reason);
  console.error('[Electron] Exit code:', details.exitCode);
});
