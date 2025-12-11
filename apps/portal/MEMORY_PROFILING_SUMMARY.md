# Memory Profiling Implementation Summary

## Problem Statement

Memory leak in Electron application:
- Memory grows continuously even with 0 creatures
- Rust side proven clean (dhat profiler: 0 bytes growth)
- Leak confirmed to be in JavaScript/Electron/V8 layer
- Application runs at 40Hz polling rate

## Solution Delivered

Complete V8/Node.js memory profiling toolkit for Electron applications.

## Files Created

### 1. Core Profiling Infrastructure

**`/home/dev/dev/speciate/apps/portal/electron/napi-memory-profile.cjs`**
- Enhanced Electron main process with full memory tracking
- V8 `process.memoryUsage()` polling at 1Hz
- Manual GC trigger support (requires `--expose-gc`)
- Heap snapshot capability via V8 API
- JSON Lines log output to `docs/performance/memory-profile.jsonl`
- IPC telemetry to dev-ui

**Key Features:**
- Logs memory at startup, after SimulationEngine creation, after start()
- Exposes 4 new IPC handlers: `memory-update`, `trigger-gc`, `take-heap-snapshot`
- Automatic cleanup on shutdown

**`/home/dev/dev/speciate/apps/portal/electron/preload-memory-profile.cjs`**
- Extended preload script for memory profiling mode
- Adds `window.electron.onMemoryUpdate()` for real-time tracking
- Adds `window.electron.triggerGC()` for manual garbage collection
- Adds `window.electron.takeHeapSnapshot()` for V8 profiling

### 2. Analysis Tools

**`/home/dev/dev/speciate/apps/portal/analyze-memory.js`**
- Node.js script to parse `memory-profile.jsonl`
- Calculates baseline vs current memory usage
- Computes growth deltas and rates (MB/s)
- Auto-diagnoses leak type (heap vs external vs ArrayBuffer)

**Output Example:**
```
=== MEMORY PROFILE ANALYSIS ===

Duration: 60.0s (60 samples)

BASELINE (start):
  RSS:          245.32 MB
  Heap Used:    28.45 MB
  External:     12.34 MB
  ArrayBuffers: 8.12 MB

DELTA (growth):
  RSS:          +152.40 MB
  Heap Used:    +108.20 MB
  ArrayBuffers: +92.10 MB

GROWTH RATE (per second):
  RSS:          +2.54 MB/s
  Heap Used:    +1.80 MB/s
  ArrayBuffers: +1.54 MB/s

=== DIAGNOSIS ===

V8 Heap Leak Detected:
  Heap growing at +1.80 MB/s
  Likely cause: JavaScript objects not being garbage collected

ArrayBuffer Leak Detected:
  ArrayBuffer memory growing at +1.54 MB/s
  Likely cause: Typed arrays (Float32Array) not being released
```

### 3. UI Components

**`/home/dev/dev/speciate/apps/dev-ui/src/components/V8HeapProfiler.tsx`**
- React component for dev-ui
- Real-time sparklines for:
  - Heap Used (V8 JavaScript objects)
  - External Memory (C++ objects, ArrayBuffers)
  - ArrayBuffers (subset of external)
- Interactive controls:
  - "Trigger GC" button (forces `global.gc()`)
  - "Heap Snapshot" button (saves `.heapsnapshot` file)
  - "Reset Baseline" button (resets growth comparison)
- Color-coded health indicators
- Growth delta display

**`/home/dev/dev/speciate/apps/dev-ui/src/types-memory.ts`**
- TypeScript type definitions
- `MemorySnapshot` interface
- `HeapSnapshotResult` interface
- `window.electron` extensions

### 4. Scripts & Documentation

**`/home/dev/dev/speciate/apps/portal/memory-profile.sh`**
- Bash launch script
- Sets `NODE_ENV=development`
- Passes `--expose-gc` flag to Electron
- Prints instructions to console

**`/home/dev/dev/speciate/docs/performance/MEMORY_PROFILING.md`**
- Comprehensive 400+ line guide
- Step-by-step debugging workflow
- Common fixes with code examples
- Chrome DevTools analysis instructions
- Success criteria definitions

**`/home/dev/dev/speciate/apps/portal/MEMORY_PROFILING_QUICKSTART.md`**
- TL;DR version for rapid debugging
- Copy-paste commands
- Key metrics reference
- Common leak sources checklist

## Usage Instructions

### Step 1: Start Memory Profiling

```bash
cd /home/dev/dev/speciate/apps/portal
chmod +x memory-profile.sh
./memory-profile.sh
```

### Step 2: Observe Console Output

```
[Electron NAPI] MEMORY PROFILING MODE ENABLED
[Electron NAPI] Manual GC available (--expose-gc)

[Memory BASELINE]
  RSS:          245.32 MB
  Heap Total:   45.12 MB
  Heap Used:    28.45 MB
  External:     12.34 MB
  ArrayBuffers: 8.12 MB

[Memory AFTER SimulationEngine]
  RSS:          246.12 MB
  Heap Used:    28.95 MB

[Memory AFTER simulation.start()]
  RSS:          248.12 MB
  Heap Used:    29.01 MB

[Memory Profiler] Logging to: docs/performance/memory-profile.jsonl
[Memory Profiler] Started (1Hz logging)
```

### Step 3: Run Analysis (After 60+ Seconds)

```bash
# In another terminal
cd /home/dev/dev/speciate/apps/portal
node analyze-memory.js
```

### Step 4: Interpret Results

#### If Heap Leak Detected:

1. **From Dev-UI:**
   - Click "Trigger GC" button
   - Watch console for `[Memory BEFORE GC]` / `[Memory AFTER GC]`
   - If memory drops > 50% → Objects are GC-able but not collected fast enough
   - If memory stays high → True leak (objects are retained)

2. **Take Heap Snapshot:**
   - Click "Heap Snapshot" button in dev-ui
   - File saved to: `docs/performance/snapshots/heap-YYYY-MM-DD_HH-MM-SS.heapsnapshot`

3. **Analyze in Chrome DevTools:**
   ```bash
   google-chrome
   # F12 → Memory tab → Load Profile → select .heapsnapshot
   ```

4. **Look for:**
   - Large arrays growing over time
   - Closures holding references
   - Event listeners not cleaned up

#### If ArrayBuffer Leak Detected:

**Most Likely Culprit:** Polling loop creating new views at 40Hz

**Location:** `electron/napi-memory-profile.cjs:201-207`

```javascript
pollingInterval = setInterval(() => {
  const fullBuffer = simulationEngine.getBuffer();
  const buffer = fullBuffer.subarray(0, usedSize);  // New view every 25ms
  mainWindow.webContents.send('napi-buffer-update', {
    buffer: buffer,  // Structured clone copies this
    creatureCount,
  });
}, pollIntervalMs);  // 25ms = 40Hz
```

**Test:**
```javascript
// Comment out buffer send
// mainWindow.webContents.send('napi-buffer-update', {...});
```

**Re-run profiling. If leak stops → Confirmed diagnosis.**

## Expected Fixes

### Fix 1: Throttle Buffer Sends

```javascript
// Current: 40Hz (every 25ms)
// Problem: Structured clone algorithm saturates memory

// Fix: Throttle to 60Hz (every 16ms) and reuse buffer
let lastSendTime = 0;
const MIN_SEND_INTERVAL = 16;  // 60Hz

pollingInterval = setInterval(() => {
  const now = Date.now();

  if (now - lastSendTime >= MIN_SEND_INTERVAL) {
    const fullBuffer = simulationEngine.getBuffer();
    const usedSize = bufferCreatureCount * 4;

    // Send full buffer + size (let renderer slice)
    mainWindow.webContents.send('napi-buffer-update', {
      buffer: fullBuffer,
      usedSize,
      creatureCount,
    });

    lastSendTime = now;
  }
}, pollIntervalMs);
```

### Fix 2: Manual GC Hint

```javascript
// Force GC every 5 seconds (200 ticks at 40Hz)
let tickCount = 0;

pollingInterval = setInterval(() => {
  tickCount++;

  // ... normal polling logic

  if (global.gc && tickCount % 200 === 0) {
    global.gc();
  }
}, pollIntervalMs);
```

### Fix 3: SharedArrayBuffer (Advanced)

```javascript
// Replace Float32Array with SharedArrayBuffer
// Zero-copy transfer between processes

// Rust side: Create shared memory region
// JS side: Map directly without structured clone
```

## Key Metrics Reference

### process.memoryUsage() Fields

| Field | Description | Leak Indicator |
|-------|-------------|----------------|
| `rss` | Total process memory | Any growth > 0.5 MB/s |
| `heapTotal` | V8 heap capacity | Grows with `heapUsed` |
| `heapUsed` | JS objects in heap | Growth > 0.1 MB/s = leak |
| `external` | C++ objects (ArrayBuffers) | Growth > 0.1 MB/s = leak |
| `arrayBuffers` | Typed array backing buffers | Growth > 0.05 MB/s = leak |

### Health Thresholds

**Stable (no leak):**
- `heapUsed`: < 0.1 MB/s growth
- `external`: < 0.1 MB/s growth
- `arrayBuffers`: ~0.0 MB/s growth

**Warning (possible leak):**
- `heapUsed`: 0.1 - 0.5 MB/s growth
- `external`: 0.1 - 0.5 MB/s growth

**Critical (confirmed leak):**
- `heapUsed`: > 0.5 MB/s growth
- `external`: > 0.5 MB/s growth

## Integration with Dev-UI

To add V8HeapProfiler component to dev-ui:

**1. Update `apps/dev-ui/src/components/DevToolsApp.tsx`:**

```typescript
import { V8HeapProfiler } from './V8HeapProfiler';

// In component tree:
<V8HeapProfiler />
```

**2. Import type definitions (if using TypeScript):**

```typescript
import type { MemorySnapshot, HeapSnapshotResult } from '../types-memory';
```

Component will automatically:
- Subscribe to `memory-update` IPC events
- Display real-time sparklines
- Provide interactive GC/snapshot controls

## Success Criteria

Profiling complete when:

1. **Growth Rate < 0.1 MB/s:**
   - `heapUsed`: < 0.1 MB/s
   - `external`: < 0.1 MB/s
   - `arrayBuffers`: ~0.0 MB/s

2. **GC Effectiveness > 50%:**
   - Manual GC drops `heapUsed` by > 50%
   - No retained objects in heap snapshot comparison

3. **Long-Term Stability:**
   - 10-minute run with 0 creatures: < 50 MB total growth
   - 10-minute run with 10K creatures: predictable, bounded growth

## Files Summary

```
apps/portal/
├── electron/
│   ├── napi-memory-profile.cjs        # Enhanced main process
│   └── preload-memory-profile.cjs     # Extended preload script
├── memory-profile.sh                  # Launch script
├── analyze-memory.js                  # Log analysis tool
├── MEMORY_PROFILING_QUICKSTART.md     # Quick reference
└── MEMORY_PROFILING_SUMMARY.md        # This file

apps/dev-ui/
└── src/
    ├── components/
    │   └── V8HeapProfiler.tsx         # React component
    └── types-memory.ts                # TypeScript definitions

docs/performance/
├── MEMORY_PROFILING.md                # Full guide
├── memory-profile.jsonl               # Log file (runtime)
└── snapshots/
    └── heap-*.heapsnapshot            # Snapshots (on demand)
```

## Next Actions

1. **Run memory profiling mode** (60+ seconds)
2. **Analyze results** with `analyze-memory.js`
3. **Identify leak type** (heap vs ArrayBuffer)
4. **Test hypothesis** (comment out suspect code)
5. **Apply fix** (throttle sends, manual GC, SharedArrayBuffer)
6. **Validate fix** (re-run profiling, check growth rate < 0.1 MB/s)

## References

- [Node.js process.memoryUsage()](https://nodejs.org/api/process.html#processmemoryusage)
- [V8 Heap Snapshots](https://developer.chrome.com/docs/devtools/memory-problems/heap-snapshots/)
- [Electron IPC Performance](https://www.electronjs.org/docs/latest/tutorial/ipc#performance-considerations)
- [Structured Clone Algorithm](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm)
