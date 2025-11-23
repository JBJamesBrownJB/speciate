const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const path = require('path');
const fs = require('fs');

let mainWindow;
let devToolsWindow = null;
let simulationEngine = null;
let pollingInterval = null;
let shuttingDown = false;

// Environment detection
const isDev = process.env.NODE_ENV === 'development';
const platform = process.platform;

console.log(`[Electron NAPI] Mode: ${isDev ? 'DEVELOPMENT' : 'PRODUCTION'}`);

/**
 * Load NAPI module
 *
 * In development: Load from simulation directory
 * In production: Load from bundled resources
 */
function loadNAPIModule() {
  let modulePath;

  if (isDev) {
    // Development: Load from simulation build output
    const platformSuffix = {
      linux: 'linux-x64-gnu',
      darwin: 'darwin-x64',
      win32: 'win32-x64-msvc',
    }[platform];

    modulePath = path.join(__dirname, '../../simulation', `speciate.${platformSuffix}.node`);
  } else {
    // Production: Load from bundled resources
    const resourcesPath = process.resourcesPath;
    const platformSuffix = {
      linux: 'linux-x64-gnu',
      darwin: 'darwin-x64',
      win32: 'win32-x64-msvc',
    }[platform];

    modulePath = path.join(resourcesPath, 'native', `speciate.${platformSuffix}.node`);
  }

  console.log(`[Electron NAPI] Loading module from: ${modulePath}`);

  if (!fs.existsSync(modulePath)) {
    console.error(`\n❌ NAPI module not found at: ${modulePath}\n`);
    if (isDev) {
      console.error('Development mode: Build the module with:');
      console.error('  cd apps/simulation && npm run build:debug\n');
    }
    app.quit();
    return null;
  }

  try {
    const addon = require(modulePath);
    console.log('[Electron NAPI] ✅ Module loaded successfully');
    return addon;
  } catch (error) {
    console.error('[Electron NAPI] ❌ Failed to load module:', error);
    app.quit();
    return null;
  }
}

/**
 * Start simulation using NAPI SimulationEngine
 */
function startSimulation() {
  const addon = loadNAPIModule();
  if (!addon) return;

  try {
    // Initialize panic handler
    addon.initLogger();
    console.log('[Electron NAPI] Logger initialized');

    // Create SimulationEngine instance
    simulationEngine = new addon.SimulationEngine();
    console.log('[Electron NAPI] ✅ SimulationEngine created');

    // Find most recent save state (by timestamp in filename)
    const assetsPath = path.join(__dirname, '../../simulation');
    const saveStatesDir = path.join(assetsPath, 'save-states');

    let mostRecentSaveState = null;

    if (fs.existsSync(saveStatesDir)) {
      const files = fs.readdirSync(saveStatesDir)
        .filter(file => file.endsWith('.msgpack'))
        .sort();  // Lexicographic sort (works because timestamps are YYYY-MM-DD_HH-MM-SS)

      if (files.length > 0) {
        mostRecentSaveState = path.join(saveStatesDir, files[files.length - 1]);
      }
    }

    if (mostRecentSaveState) {
      console.log('[Electron NAPI] 💾 Found save state, loading from:', mostRecentSaveState);
      simulationEngine.start(0, assetsPath, () => {
        // Empty callback - we'll use polling instead
      }, mostRecentSaveState);
    } else {
      console.log('[Electron NAPI] No save state found, starting fresh simulation (100 creatures)');
      simulationEngine.start(100, assetsPath, () => {
        // Empty callback - we'll use polling instead
      }, null);
    }

    // Query target simulation Hz and calculate polling rate
    // Poll at 2x simulation rate to ensure we never miss frames
    const targetSimHz = simulationEngine.getTargetHz();
    const pollHz = targetSimHz * 2;
    const pollIntervalMs = Math.floor(1000 / pollHz);

    console.log(`[Electron NAPI] ✅ Simulation started: ${targetSimHz}Hz, polling at ${pollHz}Hz`);

    // Set up polling loop
    pollingInterval = setInterval(() => {
      if (!simulationEngine || shuttingDown) {
        clearInterval(pollingInterval);
        return;
      }

      try {
        // Get the exact number of creatures in the buffer (updated every tick)
        // NOTE: getCreatureCount() reads from telemetry (updated every 30 ticks)
        //       getBufferCreatureCount() reads the actual export count (updated every tick)
        const bufferCreatureCount = simulationEngine.getBufferCreatureCount();

        // Get buffer and slice to actual creature count (SoA layout: ID, X, Y, Rotation)
        const fullBuffer = simulationEngine.getBuffer();
        const usedSize = bufferCreatureCount * 4;  // 4 f32s per creature
        const buffer = fullBuffer.subarray(0, usedSize);

        // Send buffer to portal (Float32Array - Electron IPC handles typed arrays efficiently)
        if (mainWindow && !mainWindow.isDestroyed()) {
          mainWindow.webContents.send('napi-buffer-update', {
            buffer: buffer, // Pass Float32Array directly (structured clone algorithm)
            creatureCount: bufferCreatureCount,
          });
        }

        // Get telemetry (poll every 30 frames = ~500ms at 60Hz)
        const tick = simulationEngine.getTick();
        if (tick % 30 === 0) {
          const telemetryJson = simulationEngine.getTelemetry();
          const telemetry = JSON.parse(telemetryJson);

          // Get buffer stats and add to telemetry
          const bufferStatsJson = simulationEngine.getBufferStats();
          const bufferStats = JSON.parse(bufferStatsJson);
          telemetry.napiBufferCapacityPct = bufferStats.utilizationPct;
          telemetry.napiBufferUsed = bufferStats.used;
          telemetry.napiBufferCapacity = bufferStats.capacity;

          // Send to portal (for tick rate display)
          if (mainWindow && !mainWindow.isDestroyed()) {
            mainWindow.webContents.send('telemetry-update', telemetry);
          }

          // Send to dev-ui
          if (devToolsWindow && !devToolsWindow.isDestroyed()) {
            devToolsWindow.webContents.send('telemetry-update', telemetry);
          }
        }
      } catch (error) {
        console.error('[Electron NAPI] Polling error:', error);
      }
    }, pollIntervalMs);

  } catch (error) {
    console.error('[Electron NAPI] Failed to start simulation:', error);
    app.quit();
  }
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
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
      webSecurity: true,
      allowRunningInsecureContent: false,
      devTools: isDev,
    },
  });

  // Load frontend (Vite dev server in dev, dist/ in production)
  if (isDev) {
    const viteURL = 'http://localhost:5173';
    console.log(`[Electron NAPI] Dev mode: Loading from Vite dev server at ${viteURL}`);

    let retries = 0;
    const maxRetries = 10;

    while (retries < maxRetries) {
      try {
        await mainWindow.loadURL(viteURL);
        console.log('[Electron NAPI] ✅ Connected to Vite dev server');
        break;
      } catch (err) {
        retries++;
        if (retries >= maxRetries) {
          console.error(`\n❌ Failed to connect to Vite dev server after ${maxRetries} attempts!`);
          console.error('\nMake sure Vite is running:');
          console.error('  npm run dev:vite\n');
          app.quit();
          return;
        }
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
  } else {
    const htmlPath = path.join(__dirname, '../dist/index.html');

    if (!fs.existsSync(htmlPath)) {
      console.error(`\n❌ Frontend dist/ not found at: ${htmlPath}\n`);
      console.error('Build the frontend first:');
      console.error('  npm run build:frontend\n');
      app.quit();
      return;
    }

    console.log('[Electron NAPI] Production mode: Loading from dist/');
    await mainWindow.loadFile(htmlPath);
    console.log('[Electron NAPI] ✅ HTML loaded successfully');
  }

  // Forward renderer console messages
  mainWindow.webContents.on('console-message', (event, level, message, line, sourceId) => {
    const levels = ['', 'INFO', 'WARNING', 'ERROR'];
    console.log(`[Renderer ${levels[level]}] ${message} (${sourceId}:${line})`);
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

/**
 * Create dev tools window
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

  if (isDev) {
    const viteURL = 'http://localhost:5174';
    console.log(`[Electron NAPI] Dev Tools: Loading from dev-ui Vite at ${viteURL}`);

    let retries = 0;
    const maxRetries = 10;

    while (retries < maxRetries) {
      try {
        await devToolsWindow.loadURL(viteURL);
        console.log('[Electron NAPI] ✅ Dev Tools window loaded');
        break;
      } catch (err) {
        retries++;
        if (retries >= maxRetries) {
          console.error(`\n❌ Failed to connect to dev-ui Vite server after ${maxRetries} attempts!`);
          devToolsWindow.close();
          devToolsWindow = null;
          return;
        }
        await new Promise(resolve => setTimeout(resolve, 500));
      }
    }
  } else {
    const htmlPath = path.join(__dirname, '../dist/dev-tools.html');
    if (fs.existsSync(htmlPath)) {
      await devToolsWindow.loadFile(htmlPath);
      console.log('[Electron NAPI] ✅ Dev Tools window loaded (production)');
    } else {
      console.warn('[Electron NAPI] Dev tools HTML not found in dist/');
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
 * IPC handler: Spawn creatures (dev tools command)
 */
ipcMain.on('spawn-creatures', (event, count) => {
  if (!simulationEngine) {
    console.error('[Electron NAPI] Cannot spawn: simulation not running');
    return;
  }

  try {
    simulationEngine.spawnCreatures(count);
    console.log(`[Electron NAPI] Spawned ${count} creatures`);
  } catch (error) {
    console.error('[Electron NAPI] Failed to spawn creatures:', error);
  }
});

/**
 * IPC handler: Kill all creatures (dev tools command)
 */
ipcMain.on('kill-all', () => {
  if (!simulationEngine) {
    console.error('[Electron NAPI] Cannot kill all: simulation not running');
    return;
  }

  try {
    simulationEngine.killAll();
    console.log('[Electron NAPI] Killed all creatures');
  } catch (error) {
    console.error('[Electron NAPI] Failed to kill all:', error);
  }
});

/**
 * IPC handler: Generic command dispatcher (dev-ui uses this)
 */
ipcMain.on('send-command', (event, command) => {
  if (!simulationEngine) {
    console.error('[Electron NAPI] Cannot send command: simulation not running');
    return;
  }

  try {
    switch (command.type) {
      case 'dev_spawn_creature':
        // Spawn single creature at position (x, y)
        simulationEngine.spawnCreatureAt(command.x, command.y);
        console.log(`[Electron NAPI] Spawned creature at (${command.x}, ${command.y})`);
        break;

      case 'dev_clear_creatures':
        simulationEngine.killAll();
        console.log('[Electron NAPI] Cleared all creatures');
        break;

      case 'dev_load_trial':
        simulationEngine.loadTrial(command.template);
        console.log(`[Electron NAPI] Loading trial: ${command.template}`);
        break;

      default:
        console.warn('[Electron NAPI] Unknown command type:', command.type);
    }
  } catch (error) {
    console.error('[Electron NAPI] Failed to execute command:', error);
  }
});

/**
 * IPC handler: Save metrics snapshot (dev-ui)
 */
ipcMain.handle('save-metrics-snapshot', async (event, snapshot) => {
  try {
    // Use snapshot's actual end time for timestamp (more accurate than current time)
    const timestamp = new Date(snapshot.metadata.endTime);
    const dateStr = timestamp.toISOString().replace(/:/g, '-').split('.')[0];
    const defaultFilename = `snapshot_${dateStr}.json`;

    // Pre-populate path to docs/performance/snapshots directory
    const snapshotsDir = path.join(__dirname, '../../../docs/performance/snapshots');
    const defaultPath = path.join(snapshotsDir, defaultFilename);

    const result = await dialog.showSaveDialog(devToolsWindow || mainWindow, {
      title: 'Save Metrics Snapshot',
      defaultPath,
      filters: [
        { name: 'JSON Files', extensions: ['json'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    });

    if (result.canceled || !result.filePath) {
      return { success: false, error: 'Save canceled' };
    }

    const jsonData = JSON.stringify(snapshot, null, 2);
    fs.writeFileSync(result.filePath, jsonData, 'utf8');

    console.log(`[Electron NAPI] Saved metrics snapshot to: ${result.filePath}`);
    return { success: true, path: result.filePath };
  } catch (error) {
    console.error('[Electron NAPI] Failed to save snapshot:', error);
    return { success: false, error: error.message };
  }
});

/**
 * IPC handler: Load metrics snapshot (dev-ui)
 */
ipcMain.handle('load-metrics-snapshot', async () => {
  try {
    const result = await dialog.showOpenDialog({
      title: 'Load Metrics Snapshot',
      defaultPath: app.getPath('documents'),
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

    console.log(`[Electron NAPI] Loaded metrics snapshot from: ${filePath}`);
    return snapshot;
  } catch (error) {
    console.error('[Electron NAPI] Failed to load snapshot:', error);
    throw error;
  }
});

/**
 * IPC handler: Resize dev tools window (dev-ui)
 */
ipcMain.handle('resize-window', async (event, width) => {
  try {
    if (devToolsWindow && !devToolsWindow.isDestroyed()) {
      const [, height] = devToolsWindow.getSize();
      devToolsWindow.setSize(width, height);
      console.log(`[Electron NAPI] Resized dev tools window to ${width}x${height}`);
    }
  } catch (error) {
    console.error('[Electron NAPI] Failed to resize window:', error);
  }
});

/**
 * Linux Sandbox Workaround
 */
app.commandLine.appendSwitch('no-sandbox');
app.commandLine.appendSwitch('disable-gpu-sandbox');
app.commandLine.appendSwitch('disable-dev-shm-usage');
app.commandLine.appendSwitch('enable-logging');
app.commandLine.appendSwitch('log-level', '0');

/**
 * App lifecycle: Ready event
 */
app.whenReady().then(() => {
  createWindow();
  startSimulation();

  if (isDev) {
    console.log('[Electron NAPI] Development mode: launching dev tools window');
    createDevToolsWindow();
  }

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
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

/**
 * App lifecycle: Before quit
 */
app.on('before-quit', (event) => {
  if (!simulationEngine || shuttingDown) {
    return;
  }

  event.preventDefault();
  shuttingDown = true;

  console.log('[Electron NAPI] Shutting down simulation...');

  // Clear polling interval
  if (pollingInterval) {
    clearInterval(pollingInterval);
    pollingInterval = null;
  }

  // Stop simulation gracefully
  try {
    simulationEngine.stop();
    console.log('[Electron NAPI] ✅ Simulation stopped cleanly');
  } catch (error) {
    console.error('[Electron NAPI] Error during shutdown:', error);
  } finally {
    simulationEngine = null;
    shuttingDown = false;
    app.quit();
  }
});

/**
 * Crash reporter
 */
app.on('render-process-gone', (event, webContents, details) => {
  console.error('[Electron NAPI] 💥 Renderer process crashed!');
  console.error('[Electron NAPI] Crash details:', JSON.stringify(details, null, 2));
});
