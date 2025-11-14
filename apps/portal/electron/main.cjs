const { app, BrowserWindow, ipcMain } = require('electron');
const { spawn } = require('child_process');
const msgpack = require('msgpack-lite');
const path = require('path');
const fs = require('fs');

let mainWindow;
let simulationProcess;
let latestState = null;

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
  const buildType = isDev ? 'debug' : 'release';
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

  // Spawn Rust binary as child process
  simulationProcess = spawn(binaryPath, [], {
    stdio: ['ignore', 'pipe', 'pipe'],  // stdin (ignored), stdout (pipe), stderr (pipe)
  });

  let buffer = Buffer.alloc(0);

  // Read stdout frames (length-prefixed MessagePack)
  simulationProcess.stdout.on('data', (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);

    // Process all complete frames in buffer
    while (buffer.length >= 4) {
      // Read 4-byte length prefix (big-endian u32)
      const frameLength = buffer.readUInt32BE(0);

      // Wait for complete frame
      if (buffer.length < 4 + frameLength) {
        break;
      }

      // Extract MessagePack payload
      const payload = buffer.slice(4, 4 + frameLength);
      buffer = buffer.slice(4 + frameLength);

      try {
        // Deserialize MessagePack in main process (Node.js Buffers are fast!)
        const state = msgpack.decode(payload);

        // Store latest state (in-memory cache for getLatestState())
        latestState = state;

        // Notify renderer (send plain JS object, not binary data)
        if (mainWindow && !mainWindow.isDestroyed()) {
          mainWindow.webContents.send('state-update', state);
        }
      } catch (error) {
        console.error('[Electron] Failed to decode MessagePack:', error);
      }
    }
  });

  // Log stderr output
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
    width: 1920,
    height: 1080,
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
        if (isDev) {
          mainWindow.webContents.openDevTools();
        }
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
 * IPC Command Validation Framework
 *
 * Future-proof validation for bidirectional IPC commands (Phase 2+)
 * Current: Only read-only 'get-latest-state' exists
 * Future: spawn_creature, set_camera_zoom, etc. will use this pattern
 */
const COMMAND_VALIDATORS = {
  // Example validators for future commands:
  spawn_creature: (params) => {
    if (typeof params !== 'object' || params === null) {
      throw new Error('spawn_creature: params must be an object');
    }
    if (typeof params.x !== 'number' || typeof params.y !== 'number') {
      throw new Error('spawn_creature: x and y must be numbers');
    }
    if (!Number.isFinite(params.x) || !Number.isFinite(params.y)) {
      throw new Error('spawn_creature: x and y must be finite numbers');
    }
    // Bounds checking (world is ±1M units)
    if (Math.abs(params.x) > 1_000_000 || Math.abs(params.y) > 1_000_000) {
      throw new Error('spawn_creature: coordinates out of world bounds');
    }
  },

  set_camera_zoom: (params) => {
    if (typeof params !== 'object' || params === null) {
      throw new Error('set_camera_zoom: params must be an object');
    }
    if (typeof params.level !== 'number') {
      throw new Error('set_camera_zoom: level must be a number');
    }
    if (params.level < 0 || params.level > 100) {
      throw new Error('set_camera_zoom: level must be 0-100');
    }
  },
};

/**
 * Generic validated command handler (for future bidirectional IPC)
 *
 * Usage from renderer:
 *   await window.electron.executeCommand('spawn_creature', { x: 100, y: 200 })
 *
 * Security: Whitelist + parameter validation prevents injection attacks
 * Performance: ~100ns overhead, not in 60 Hz streaming path (zero impact)
 */
ipcMain.handle('execute-command', async (event, command, params) => {
  // Whitelist validation (O(1) lookup)
  const validator = COMMAND_VALIDATORS[command];
  if (!validator) {
    throw new Error(`Unknown command: ${command}`);
  }

  // Parameter validation (~50-100ns overhead)
  try {
    validator(params);
  } catch (err) {
    console.error(`[Electron] Command validation failed: ${err.message}`);
    throw err;
  }

  // TODO: Forward validated command to simulation via stdin
  // This will be implemented in Sprint 8 (bidirectional IPC)
  throw new Error('Bidirectional IPC not yet implemented (Phase 2)');
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
 * App lifecycle: Before quit
 */
app.on('quit', () => {
  console.log('[Electron] Quitting app, killing simulation process...');
  if (simulationProcess) {
    simulationProcess.kill();
    simulationProcess = null;
  }
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
