const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const path = require('path');
const fs = require('fs');
const { createFrameDelivery } = require('./frameDelivery.cjs');
const { FLOATS_PER_CREATURE, MAX_CREATURES, creatureBufferFloats } = require('./bufferLayout.cjs');

let mainWindow;
let devToolsWindow = null;
let simulationEngine = null;
let pollingInterval = null; // legacy fallback poll (only when push-on-swap unavailable)
let telemetryInterval = null; // status heartbeat (decoupled from the position doorbell)
let plantInterval = null; // plant buffer push every 2s
let shuttingDown = false;

// Buffer layout (FLOATS_PER_CREATURE, MAX_CREATURES) is sourced from bufferLayout.cjs
// (the JS twin of the Rust producer cap). See that file for the seam contract.

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

// Persistent buffers for zero-allocation polling (memory leak fix)
let creatureBuffer = null;       // Float32Array for creature data
let perceptionBuffer = null;     // Float32Array for perception debug

// DEBUG: Set to true to isolate memory leak sources
const DISABLE_BUFFER_CALLS = false;
const DISABLE_TELEMETRY_CALLS = false;

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

    // Create persistent buffers for zero-allocation polling (memory leak fix).
    // Sized to the MAX_CREATURES cap (matches the Rust producer buffer); at 1M that is
    // 5M floats = 20MB. Smaller than this would truncate positions at the seam.
    creatureBuffer = new Float32Array(creatureBufferFloats());
    // Perception debug buffer: query required size from Rust (single source of truth)
    if (!simulationEngine.getPerceptionDebugBufferSize) {
      throw new Error('[Electron NAPI] FATAL: getPerceptionDebugBufferSize() unavailable - rebuild native addon with dev-tools feature');
    }
    const perceptionBufferSize = simulationEngine.getPerceptionDebugBufferSize();
    perceptionBuffer = new Float32Array(perceptionBufferSize);
    const creatureBufferMB = (creatureBuffer.byteLength / 1024 / 1024).toFixed(0);
    console.log(`[Electron NAPI] ✅ Persistent buffers created (creature: ${creatureBufferMB}MB for ${MAX_CREATURES.toLocaleString()} max, perception: ${perceptionBufferSize} floats)`);

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

    // --- Frame delivery: push-on-swap (event-driven), with a poll fallback ---
    // deliverFrame (extracted to frameDelivery.cjs for testing) ships positions +
    // perception once per Rust buffer swap. Telemetry runs on its own light timer
    // (below) so the status heartbeat isn't tied to the drop-prone position doorbell.
    const deliverFrame = createFrameDelivery({
      getEngine: () => simulationEngine,
      getMainWindow: () => mainWindow,
      isShuttingDown: () => shuttingDown,
      creatureBuffer,
      perceptionBuffer,
      floatsPerCreature: FLOATS_PER_CREATURE,
      disableBufferCalls: DISABLE_BUFFER_CALLS,
      onMemorySample: () => {
        const mem = process.memoryUsage();
        console.log(`[Memory] RSS: ${(mem.rss / 1024 / 1024).toFixed(1)}MB, Heap: ${(mem.heapUsed / 1024 / 1024).toFixed(1)}/${(mem.heapTotal / 1024 / 1024).toFixed(1)}MB, External: ${(mem.external / 1024 / 1024).toFixed(1)}MB, ArrayBuffers: ${(mem.arrayBuffers / 1024 / 1024).toFixed(1)}MB`);
      },
    });

    // Register the buffer-ready doorbell BEFORE start(): Rust clones the callback once
    // at thread spawn, so registering after start() would be a silent no-op.
    const hasDoorbell = typeof simulationEngine.onBufferReady === 'function';
    if (hasDoorbell) {
      simulationEngine.onBufferReady((tick) => deliverFrame(tick));
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

    const targetSimHz = simulationEngine.getTargetHz();
    if (hasDoorbell) {
      console.log(`[Electron NAPI] ✅ Simulation started: ${targetSimHz}Hz — push-on-swap delivery enabled (event-driven, no polling)`);
    } else {
      // Fallback: the addon predates onBufferReady (not rebuilt). Poll at 2x sim rate,
      // calling the SAME deliverFrame so there is no duplicated delivery logic.
      const pollHz = targetSimHz * 2;
      const pollIntervalMs = Math.floor(1000 / pollHz);
      console.warn(`[Electron NAPI] ⚠️ onBufferReady unavailable — falling back to polling at ${pollHz}Hz (rebuild the addon to enable push-on-swap)`);
      pollingInterval = setInterval(() => deliverFrame(simulationEngine.getTick()), pollIntervalMs);
    }

    // Plant buffer: push sparse snapshot to renderer on startup and every 2s.
    // Plants update slowly (CA ticks every 1-2s), so per-frame delivery is wasteful.
    const deliverPlants = () => {
      if (!simulationEngine || shuttingDown) return;
      try {
        const plantBuf = simulationEngine.getPlantBuffer();
        if (mainWindow && !mainWindow.isDestroyed()) {
          mainWindow.webContents.send('plant-buffer-update', plantBuf);
        }
      } catch (error) {
        console.error('[Electron NAPI] Plant buffer error:', error);
      }
    };
    // Deliver once at startup (allow sim a moment to initialize), then every 2s.
    setTimeout(deliverPlants, 500);
    plantInterval = setInterval(deliverPlants, 2000);

    // Telemetry/status heartbeat on its own light timer (~2x/sec), decoupled from the
    // position doorbell so a dropped frame never skips a status update.
    telemetryInterval = setInterval(() => {
      if (!simulationEngine || shuttingDown) return;
      if (DISABLE_TELEMETRY_CALLS) return;
      try {
        const telemetry = JSON.parse(simulationEngine.getTelemetry());
        const bufferStats = JSON.parse(simulationEngine.getBufferStats());
        telemetry.napiBufferCapacityPct = bufferStats.utilizationPct;
        telemetry.napiBufferUsed = bufferStats.used;
        telemetry.napiBufferCapacity = bufferStats.capacity;
        if (mainWindow && !mainWindow.isDestroyed()) {
          mainWindow.webContents.send('telemetry-update', telemetry);
        }
        if (devToolsWindow && !devToolsWindow.isDestroyed()) {
          devToolsWindow.webContents.send('telemetry-update', telemetry);
        }
      } catch (error) {
        console.error('[Electron NAPI] Telemetry error:', error);
      }
    }, 500);

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

  // Forward dev-tools renderer console to the terminal — this window has no other
  // observability path, so a React throw here would otherwise be silent.
  devToolsWindow.webContents.on('console-message', (event, level, message, line, sourceId) => {
    const levels = ['', 'INFO', 'WARNING', 'ERROR'];
    console.log(`[DevTools Renderer ${levels[level]}] ${message} (${sourceId}:${line})`);
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
/**
 * IPC relay: render-pipeline metrics from the portal (game window) → dev-tools window.
 * These are renderer-origin (interpolation cadence), so they don't ride the Rust
 * telemetry channel. DEV-only: the portal only sends them in dev builds.
 */
ipcMain.on('render-metrics', (event, metrics) => {
  if (devToolsWindow && !devToolsWindow.isDestroyed()) {
    devToolsWindow.webContents.send('render-metrics-update', metrics);
  }
});

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
        // Spawn single creature at position (x, y) with optional DNA
        const sizeGene = command.dna?.size_gene ?? null;
        const fovGene = command.dna?.fov_gene ?? null;
        simulationEngine.spawnCreatureAt(command.x, command.y, sizeGene, fovGene);
        console.log(`[Electron NAPI] Spawned creature at (${command.x}, ${command.y}) with DNA: size=${sizeGene}, fov=${fovGene}`);
        break;

      case 'dev_clear_creatures':
        simulationEngine.killAll();
        console.log('[Electron NAPI] Cleared all creatures');
        break;

      case 'dev_clear_plants':
        simulationEngine.clearAllPlants();
        console.log('[Electron NAPI] Cleared all plants');
        break;

      case 'dev_load_trial':
        const randomize = command.randomizeDna || false;
        const trialSizeGene = command.dna?.size_gene ?? null;
        const trialFovGene = command.dna?.fov_gene ?? null;
        simulationEngine.loadTrial(command.template, randomize, trialSizeGene, trialFovGene);
        console.log(`[Electron NAPI] Loading trial: ${command.template} (randomizeDna: ${randomize}, dna: size=${trialSizeGene}, fov=${trialFovGene})`);
        break;

      default:
        console.warn('[Electron NAPI] Unknown command type:', command.type);
    }
  } catch (error) {
    console.error('[Electron NAPI] Failed to execute command:', error);
  }
});

/**
 * IPC handler: Select creature for perception debug (portal)
 */
ipcMain.on('select-creature-debug', (event, creatureId) => {
  console.log(`[Electron NAPI] select-creature-debug received: ${creatureId}`);

  if (!simulationEngine) {
    console.error('[Electron NAPI] Cannot select creature: simulation not running');
    return;
  }

  try {
    simulationEngine.selectCreatureDebug(creatureId);
    console.log(`[Electron NAPI] selectCreatureDebug(${creatureId}) called successfully`);
  } catch (error) {
    console.error('[Electron NAPI] Failed to select creature:', error);
  }
});

/**
 * IPC handler: Set pause state (portal)
 */
ipcMain.on('set-paused', (event, paused) => {
  if (!simulationEngine) {
    console.error('[Electron NAPI] Cannot set paused: simulation not running');
    return;
  }

  try {
    simulationEngine.setPaused(paused);
    console.log(`[Electron NAPI] Simulation ${paused ? 'PAUSED' : 'RESUMED'}`);
  } catch (error) {
    console.error('[Electron NAPI] Failed to set paused:', error);
  }
});

/**
 * IPC handler: Set time scale (portal)
 */
ipcMain.on('set-time-scale', (event, scale) => {
  if (!simulationEngine) {
    console.error('[Electron NAPI] Cannot set time scale: simulation not running');
    return;
  }

  try {
    simulationEngine.setTimeScale(scale);
    console.log(`[Electron NAPI] Time scale set to ${scale}x`);
  } catch (error) {
    console.error('[Electron NAPI] Failed to set time scale:', error);
  }
});

/**
 * IPC handler: Set system frequency divisor (dev-ui)
 *
 * Controls update frequency for cognitive systems (perception, behavior, steering).
 * divisor=1 means every tick, divisor=2 means every 2nd tick, etc.
 */
ipcMain.on('set-system-frequency', (event, { systemName, divisor }) => {
  if (!simulationEngine) {
    console.error('[Electron NAPI] Cannot set system frequency: simulation not running');
    return;
  }

  try {
    simulationEngine.setSystemFrequency(systemName, divisor);
    console.log(`[Electron NAPI] Set ${systemName} frequency divisor to ${divisor}`);
  } catch (error) {
    console.error('[Electron NAPI] Failed to set system frequency:', error);
  }
});

/**
 * IPC handler: Set viewport bounds for culling (portal)
 *
 * When viewport bounds are set, the backend only exports creatures within
 * these bounds (plus margin). This reduces IPC bandwidth and GPU work.
 */
ipcMain.on('set-viewport-bounds', (event, bounds) => {
  if (!simulationEngine) {
    return;
  }

  try {
    simulationEngine.setViewportBounds(
      bounds.minX,
      bounds.minY,
      bounds.maxX,
      bounds.maxY,
      bounds.margin
    );
  } catch (error) {
    console.error('[Electron NAPI] Failed to set viewport bounds:', error);
  }
});

/**
 * IPC handler: Spawn a plant at world position (portal P0 mode click)
 */
ipcMain.on('spawn-plant', (event, { worldX, worldY }) => {
  if (!simulationEngine) return;
  try {
    simulationEngine.spawnPlant(worldX, worldY);
    // Push updated buffer immediately (deliverPlants is scoped to start-simulation callback,
    // so inline the push here to avoid a ReferenceError)
    const plantBuf = simulationEngine.getPlantBuffer();
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('plant-buffer-update', plantBuf);
    }
  } catch (error) {
    console.error('[Electron NAPI] Failed to spawn plant:', error);
  }
});

/**
 * IPC handler: Query L1 cell at world position (dev-tools only)
 *
 * Returns cell metadata (creature count, mass, sizes) for the cell at the given position.
 * Returns null if cell is empty or simulation not running.
 */
ipcMain.handle('query-l1-cell', async (event, worldX, worldY) => {
  if (!simulationEngine || !simulationEngine.queryL1Cell) {
    return null;
  }

  try {
    return simulationEngine.queryL1Cell(worldX, worldY);
  } catch (error) {
    console.error('[Electron NAPI] Failed to query L1 cell:', error);
    return null;
  }
});

/**
 * IPC handler: Save metrics snapshot (dev-ui)
 */
ipcMain.handle('save-metrics-snapshot', async (event, snapshot) => {
  try {
    // Self-describing filename: <platform>_pop<N>_<tick>ms_<localdate>.json
    // (e.g. win_pop500k_6.8ms_2026-06-20_2009.json). Population, tick time, and OS
    // are the at-a-glance comparison keys; full stats live inside the file.
    const ts = new Date(snapshot.metadata.endTime);
    const pad = (n) => String(n).padStart(2, '0');
    const dateStr = `${ts.getFullYear()}-${pad(ts.getMonth() + 1)}-${pad(ts.getDate())}_${pad(ts.getHours())}${pad(ts.getMinutes())}`;

    const platformTag = { win32: 'win', darwin: 'mac', linux: 'linux' }[process.platform] || process.platform;

    const fmtCount = (n) => {
      if (n >= 1e6) return `${(n / 1e6).toFixed(1)}M`.replace(/\.0M$/, 'M');
      if (n >= 1e3) return `${(n / 1e3).toFixed(1)}k`.replace(/\.0k$/, 'k');
      return `${Math.round(n)}`;
    };
    const popTag = `pop${fmtCount(snapshot.creatureCount?.avg ?? 0)}`;

    const tickUs = snapshot.systemTimings?.totalTickUs?.avg ?? 0;
    const tickTag = tickUs > 0 ? `_${(tickUs / 1000).toFixed(1)}ms` : '';

    const defaultFilename = `${platformTag}_${popTag}${tickTag}_${dateStr}.json`;

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
    // Open where snapshots are actually saved (see save-metrics-snapshot), not Documents.
    const snapshotsDir = path.join(__dirname, '../../../docs/performance/snapshots');
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

  // Clear all timers (fallback poll + telemetry heartbeat + plant push)
  if (pollingInterval) {
    clearInterval(pollingInterval);
    pollingInterval = null;
  }
  if (telemetryInterval) {
    clearInterval(telemetryInterval);
    telemetryInterval = null;
  }
  if (plantInterval) {
    clearInterval(plantInterval);
    plantInterval = null;
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

    // Diagnostic: dump active handles so we know exactly what's keeping the process alive
    const handles = process._getActiveHandles();
    const requests = process._getActiveRequests();
    console.log(`[Shutdown diag] Active handles (${handles.length}):`);
    handles.forEach((h, i) => console.log(`  [${i}] ${h.constructor.name}`, h._type || '', h.fd || ''));
    console.log(`[Shutdown diag] Active requests (${requests.length}):`);
    requests.forEach((r, i) => console.log(`  [${i}] ${r.constructor.name}`));

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
