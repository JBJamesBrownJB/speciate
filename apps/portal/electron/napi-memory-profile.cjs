/**
 * Memory Profiling Version of NAPI Main Process
 *
 * Run with: MEMORY_PROFILE=1 electron --expose-gc electron/napi-memory-profile.cjs
 *
 * This version adds:
 * - V8 heap usage tracking (process.memoryUsage())
 * - Manual GC trigger capability
 * - Memory telemetry sent to dev-ui
 * - Heap snapshot capability
 * - JSON Lines memory log file
 */

const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const path = require('path');
const fs = require('fs');

let mainWindow;
let devToolsWindow = null;
let simulationEngine = null;
let pollingInterval = null;
let memoryProfilingInterval = null;
let shuttingDown = false;

const isDev = process.env.NODE_ENV === 'development';
const platform = process.platform;
const MEMORY_PROFILING_ENABLED = true;

console.log(`[Electron NAPI] Mode: ${isDev ? 'DEVELOPMENT' : 'PRODUCTION'}`);
console.log('[Electron NAPI] MEMORY PROFILING MODE ENABLED');

if (global.gc) {
  console.log('[Electron NAPI] Manual GC available (--expose-gc)');
} else {
  console.warn('[Electron NAPI] Manual GC NOT available (restart with --expose-gc)');
}

function formatBytes(bytes) {
  return (bytes / 1024 / 1024).toFixed(2) + ' MB';
}

function getMemorySnapshot() {
  const mem = process.memoryUsage();
  return {
    timestamp: Date.now(),
    rss: mem.rss,
    heapTotal: mem.heapTotal,
    heapUsed: mem.heapUsed,
    external: mem.external,
    arrayBuffers: mem.arrayBuffers,
  };
}

function logMemorySnapshot(label) {
  const mem = process.memoryUsage();
  console.log(`[Memory ${label}]`);
  console.log(`  RSS:          ${formatBytes(mem.rss)}`);
  console.log(`  Heap Total:   ${formatBytes(mem.heapTotal)}`);
  console.log(`  Heap Used:    ${formatBytes(mem.heapUsed)}`);
  console.log(`  External:     ${formatBytes(mem.external)}`);
  console.log(`  ArrayBuffers: ${formatBytes(mem.arrayBuffers)}`);
}

function startMemoryProfiling() {
  const MEMORY_LOG_FILE = path.join(__dirname, '../../../docs/performance/memory-profile.jsonl');
  console.log(`[Memory Profiler] Logging to: ${MEMORY_LOG_FILE}`);

  let memoryLogStream;
  try {
    const logDir = path.dirname(MEMORY_LOG_FILE);
    if (!fs.existsSync(logDir)) {
      fs.mkdirSync(logDir, { recursive: true });
    }
    memoryLogStream = fs.createWriteStream(MEMORY_LOG_FILE, { flags: 'w' });
  } catch (error) {
    console.error('[Memory Profiler] Failed to create log file:', error);
    return;
  }

  logMemorySnapshot('BASELINE');

  memoryProfilingInterval = setInterval(() => {
    const snapshot = getMemorySnapshot();

    memoryLogStream.write(JSON.stringify(snapshot) + '\n');

    if (devToolsWindow && !devToolsWindow.isDestroyed()) {
      devToolsWindow.webContents.send('memory-update', snapshot);
    }
  }, 1000);

  console.log('[Memory Profiler] Started (1Hz logging)');
}

function loadNAPIModule() {
  let modulePath;

  if (isDev) {
    const platformSuffix = {
      linux: 'linux-x64-gnu',
      darwin: 'darwin-x64',
      win32: 'win32-x64-msvc',
    }[platform];

    modulePath = path.join(__dirname, '../../simulation', `speciate.${platformSuffix}.node`);
  } else {
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

function startSimulation() {
  const addon = loadNAPIModule();
  if (!addon) return;

  try {
    addon.initLogger();
    console.log('[Electron NAPI] Logger initialized');

    logMemorySnapshot('BEFORE SimulationEngine');

    simulationEngine = new addon.SimulationEngine();
    console.log('[Electron NAPI] ✅ SimulationEngine created');

    logMemorySnapshot('AFTER SimulationEngine');

    const assetsPath = path.join(__dirname, '../../simulation');
    const saveStatesDir = path.join(assetsPath, 'save-states');

    let mostRecentSaveState = null;

    if (fs.existsSync(saveStatesDir)) {
      const files = fs.readdirSync(saveStatesDir)
        .filter(file => file.endsWith('.msgpack'))
        .sort();

      if (files.length > 0) {
        mostRecentSaveState = path.join(saveStatesDir, files[files.length - 1]);
      }
    }

    if (mostRecentSaveState) {
      console.log('[Electron NAPI] 💾 Found save state, loading from:', mostRecentSaveState);
      simulationEngine.start(0, assetsPath, () => {}, mostRecentSaveState);
    } else {
      console.log('[Electron NAPI] No save state found, starting fresh simulation (100 creatures)');
      simulationEngine.start(100, assetsPath, () => {}, null);
    }

    logMemorySnapshot('AFTER simulation.start()');

    const targetSimHz = simulationEngine.getTargetHz();
    const pollHz = targetSimHz * 2;
    const pollIntervalMs = Math.floor(1000 / pollHz);

    console.log(`[Electron NAPI] ✅ Simulation started: ${targetSimHz}Hz, polling at ${pollHz}Hz`);

    pollingInterval = setInterval(() => {
      if (!simulationEngine || shuttingDown) {
        clearInterval(pollingInterval);
        return;
      }

      try {
        const bufferCreatureCount = simulationEngine.getBufferCreatureCount();
        const fullBuffer = simulationEngine.getBuffer();
        const usedSize = bufferCreatureCount * 4;
        const buffer = fullBuffer.subarray(0, usedSize);

        if (mainWindow && !mainWindow.isDestroyed()) {
          mainWindow.webContents.send('napi-buffer-update', {
            buffer: buffer,
            creatureCount: bufferCreatureCount,
          });
        }

        if (simulationEngine.getPerceptionDebug) {
          const debugBuffer = simulationEngine.getPerceptionDebug();
          if (debugBuffer[0] > 0.5 && mainWindow && !mainWindow.isDestroyed()) {
            mainWindow.webContents.send('perception-debug-update', debugBuffer);
          }
        }

        const tick = simulationEngine.getTick();
        if (tick % 30 === 0) {
          const telemetryJson = simulationEngine.getTelemetry();
          const telemetry = JSON.parse(telemetryJson);

          const bufferStatsJson = simulationEngine.getBufferStats();
          const bufferStats = JSON.parse(bufferStatsJson);
          telemetry.napiBufferCapacityPct = bufferStats.utilizationPct;
          telemetry.napiBufferUsed = bufferStats.used;
          telemetry.napiBufferCapacity = bufferStats.capacity;

          if (mainWindow && !mainWindow.isDestroyed()) {
            mainWindow.webContents.send('telemetry-update', telemetry);
          }

          if (devToolsWindow && !devToolsWindow.isDestroyed()) {
            devToolsWindow.webContents.send('telemetry-update', telemetry);
          }
        }
      } catch (error) {
        console.error('[Electron NAPI] Polling error:', error);
      }
    }, pollIntervalMs);

    startMemoryProfiling();

  } catch (error) {
    console.error('[Electron NAPI] Failed to start simulation:', error);
    app.quit();
  }
}

async function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1600,
    height: 1000,
    x: 0,
    y: 0,
    webPreferences: {
      preload: path.join(__dirname, 'preload-memory-profile.cjs'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
      webSecurity: true,
      allowRunningInsecureContent: false,
      devTools: isDev,
    },
  });

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

  mainWindow.webContents.on('console-message', (event, level, message, line, sourceId) => {
    const levels = ['', 'INFO', 'WARNING', 'ERROR'];
    console.log(`[Renderer ${levels[level]}] ${message} (${sourceId}:${line})`);
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

async function createDevToolsWindow() {
  devToolsWindow = new BrowserWindow({
    width: 750,
    height: 1300,
    x: 1920,
    y: 0,
    title: 'Speciate Dev Tools (Memory Profiling)',
    webPreferences: {
      preload: path.join(__dirname, 'preload-memory-profile.cjs'),
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

ipcMain.on('send-command', (event, command) => {
  if (!simulationEngine) {
    console.error('[Electron NAPI] Cannot send command: simulation not running');
    return;
  }

  try {
    switch (command.type) {
      case 'dev_spawn_creature':
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

ipcMain.on('select-creature-debug', (event, creatureId) => {
  if (!simulationEngine) {
    console.error('[Electron NAPI] Cannot select creature: simulation not running');
    return;
  }

  try {
    simulationEngine.selectCreatureDebug(creatureId);
    if (creatureId !== null) {
      console.log(`[Electron NAPI] Selected creature ${creatureId} for debug`);
    } else {
      console.log('[Electron NAPI] Cleared creature debug selection');
    }
  } catch (error) {
    console.error('[Electron NAPI] Failed to select creature:', error);
  }
});

ipcMain.on('trigger-gc', () => {
  if (global.gc) {
    console.log('[Memory Profiler] Triggering manual GC...');
    logMemorySnapshot('BEFORE GC');
    global.gc();
    setTimeout(() => {
      logMemorySnapshot('AFTER GC');
    }, 100);
  } else {
    console.warn('[Memory Profiler] GC not available (start with --expose-gc)');
  }
});

ipcMain.handle('take-heap-snapshot', async () => {
  try {
    const v8 = require('v8');
    const timestamp = new Date().toISOString().replace(/:/g, '-').split('.')[0];
    const snapshotPath = path.join(__dirname, '../../../docs/performance/snapshots', `heap-${timestamp}.heapsnapshot`);

    const snapshotDir = path.dirname(snapshotPath);
    if (!fs.existsSync(snapshotDir)) {
      fs.mkdirSync(snapshotDir, { recursive: true });
    }

    console.log('[Memory Profiler] Taking heap snapshot...');
    v8.writeHeapSnapshot(snapshotPath);
    console.log(`[Memory Profiler] Heap snapshot saved to: ${snapshotPath}`);

    return { success: true, path: snapshotPath };
  } catch (error) {
    console.error('[Memory Profiler] Failed to take heap snapshot:', error);
    return { success: false, error: error.message };
  }
});

ipcMain.handle('save-metrics-snapshot', async (event, snapshot) => {
  try {
    const timestamp = new Date(snapshot.metadata.endTime);
    const dateStr = timestamp.toISOString().replace(/:/g, '-').split('.')[0];
    const defaultFilename = `snapshot_${dateStr}.json`;

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

app.commandLine.appendSwitch('no-sandbox');
app.commandLine.appendSwitch('disable-gpu-sandbox');
app.commandLine.appendSwitch('disable-dev-shm-usage');
app.commandLine.appendSwitch('enable-logging');
app.commandLine.appendSwitch('log-level', '0');
app.commandLine.appendSwitch('expose-gc');
app.commandLine.appendSwitch('js-flags', '--expose-gc');

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

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('before-quit', (event) => {
  if (!simulationEngine || shuttingDown) {
    return;
  }

  event.preventDefault();
  shuttingDown = true;

  console.log('[Electron NAPI] Shutting down simulation...');

  if (pollingInterval) {
    clearInterval(pollingInterval);
    pollingInterval = null;
  }

  if (memoryProfilingInterval) {
    clearInterval(memoryProfilingInterval);
    memoryProfilingInterval = null;
  }

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

app.on('render-process-gone', (event, webContents, details) => {
  console.error('[Electron NAPI] 💥 Renderer process crashed!');
  console.error('[Electron NAPI] Crash details:', JSON.stringify(details, null, 2));
});
