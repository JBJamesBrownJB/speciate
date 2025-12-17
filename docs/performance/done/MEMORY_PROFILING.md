# Memory Profiling Guide

## The Problem

Memory leak detected in Electron application:
- Memory grows continuously even with 0 creatures
- Rust side proven clean (dhat profiler showed 0 bytes growth)
- Leak is in JavaScript/Electron/V8 layer
- App runs at 40Hz polling rate

## Tools Provided

### 1. Memory Profiling Mode

**File:** `/home/dev/dev/speciate/apps/portal/electron/napi-memory-profile.cjs`

Enhanced Electron main process with:
- V8 heap usage tracking (`process.memoryUsage()`)
- Manual GC trigger capability (requires `--expose-gc`)
- Memory telemetry sent to dev-ui (1Hz)
- Heap snapshot capability
- JSON Lines memory log file

**Run:**
```bash
cd apps/portal
chmod +x memory-profile.sh
./memory-profile.sh
```

**Or manually:**
```bash
cd apps/portal
NODE_ENV=development ELECTRON_DISABLE_SANDBOX=1 electron --expose-gc electron/napi-memory-profile.cjs
```

### 2. Memory Analysis Script

**File:** `/home/dev/dev/speciate/apps/portal/analyze-memory.js`

Parses `memory-profile.jsonl` and prints:
- Baseline vs current memory usage
- Memory growth deltas
- Growth rates (MB/s)
- Automated diagnosis (heap vs external vs ArrayBuffer)

**Run:**
```bash
cd apps/portal
node analyze-memory.js
```

### 3. V8 Heap Profiler UI Component

**File:** `/home/dev/dev/speciate/apps/dev-ui/src/components/V8HeapProfiler.tsx`

React component for dev-ui that displays:
- Real-time heap usage sparklines
- External memory tracking
- ArrayBuffer tracking
- Manual GC trigger button
- Heap snapshot button
- Baseline comparison

## Step-by-Step Debugging

### Phase 1: Establish Baseline

1. **Start memory profiling mode:**
   ```bash
   cd apps/portal
   ./memory-profile.sh
   ```

2. **Let it run for 60 seconds with 0 creatures**
   - Watch console output: `[Memory BASELINE]`, `[Memory AFTER simulation.start()]`
   - Check for growth even before any creatures spawn

3. **Run analysis:**
   ```bash
   node analyze-memory.js
   ```

4. **Check output:**
   ```
   === DIAGNOSIS ===

   V8 Heap Leak Detected:
     Heap growing at +5.2 MB/s
     Likely cause: JavaScript objects not being garbage collected

   ArrayBuffer Leak Detected:
     ArrayBuffer memory growing at +3.1 MB/s
     Likely cause: Typed arrays (Float32Array) not being released
   ```

### Phase 2: Identify Leak Source

Based on diagnosis output:

#### If Heap Leak (JavaScript objects):

1. **Trigger manual GC from dev-ui:**
   - Click "Trigger GC" button in V8 Heap Profiler panel
   - Watch console: `[Memory BEFORE GC]` / `[Memory AFTER GC]`
   - If memory **drops significantly** → objects are GC-able but not collected fast enough
   - If memory **stays high** → objects are being retained (true leak)

2. **Take heap snapshot:**
   - Click "Heap Snapshot" button in dev-ui
   - Opens file at: `docs/performance/snapshots/heap-YYYY-MM-DD_HH-MM-SS.heapsnapshot`

3. **Analyze in Chrome DevTools:**
   ```bash
   google-chrome --new-window
   # F12 → Memory tab → Load Profile → select .heapsnapshot file
   ```

4. **Look for:**
   - Large arrays growing over time
   - Detached DOM nodes (shouldn't exist in main process!)
   - Closures holding references
   - Event listeners not cleaned up

#### If External Memory Leak (NAPI/C++):

1. **Check Rust NAPI addon:**
   - `apps/simulation/src/napi_addon/simulation_engine.rs`
   - Look for `napi::Ref` or `napi::External` not being dropped

2. **Check buffer management:**
   - `simulationEngine.getBuffer()` returns shared ArrayBuffer
   - `simulationEngine.getPerceptionDebug()` returns shared ArrayBuffer
   - Are these being cloned instead of referenced?

3. **Check for:**
   - `Rc<RefCell<>>` in Rust not being dropped
   - Bevy resources leaking
   - Tokio tasks not terminating

#### If ArrayBuffer Leak (Typed Arrays):

1. **Check polling loop in napi-memory-profile.cjs:**
   ```javascript
   const fullBuffer = simulationEngine.getBuffer();
   const buffer = fullBuffer.subarray(0, usedSize);
   ```

2. **Hypothesis:**
   - `buffer.subarray()` might create new ArrayBuffer views each tick (40Hz)
   - These views might not be GC'd fast enough
   - Structured clone algorithm in IPC might be copying instead of transferring

3. **Test:**
   - Comment out `mainWindow.webContents.send('napi-buffer-update', {...})`
   - Re-run profiling
   - If leak stops → IPC buffer transfer is the culprit

### Phase 3: Fix Validation

1. **Implement fix** (examples below)

2. **Re-run memory profiling:**
   ```bash
   ./memory-profile.sh
   # Let run for 120 seconds
   node analyze-memory.js
   ```

3. **Check growth rate:**
   ```
   GROWTH RATE (per second):
     RSS:          +0.05 MB/s   ← Should be < 0.1 MB/s
     Heap Used:    +0.02 MB/s   ← Should be < 0.05 MB/s
     ArrayBuffers: +0.00 MB/s   ← Should be ~0.0 MB/s
   ```

## Common Fixes

### Fix 1: Reuse Buffer Views (ArrayBuffer Leak)

**Problem:** Creating new TypedArray views each tick
```javascript
// BAD: Creates new view every tick (40Hz)
const buffer = fullBuffer.subarray(0, usedSize);
mainWindow.webContents.send('napi-buffer-update', { buffer, creatureCount });
```

**Fix:** Reuse existing view or use transferable objects
```javascript
// GOOD: Direct reference (no copy)
mainWindow.webContents.send('napi-buffer-update', {
  buffer: fullBuffer,  // Pass full buffer
  creatureCount,
  usedSize  // Renderer slices on its side
});
```

### Fix 2: Throttle IPC Sends (Heap Leak)

**Problem:** 40Hz IPC sends saturate structured clone
```javascript
// BAD: Sends every tick
pollingInterval = setInterval(() => {
  mainWindow.webContents.send('napi-buffer-update', data);
}, pollIntervalMs);  // 25ms = 40Hz
```

**Fix:** Throttle to display refresh rate
```javascript
// GOOD: Send at 60Hz max (matches screen refresh)
let lastSendTime = 0;
pollingInterval = setInterval(() => {
  const now = Date.now();
  if (now - lastSendTime >= 16) {  // 60Hz throttle
    mainWindow.webContents.send('napi-buffer-update', data);
    lastSendTime = now;
  }
}, pollIntervalMs);
```

### Fix 3: Manual GC Hint (Heap Leak)

**Problem:** V8 GC not aggressive enough
```javascript
// BAD: No GC hints
pollingInterval = setInterval(() => {
  // ... lots of allocations
}, pollIntervalMs);
```

**Fix:** Periodic manual GC (development only)
```javascript
// GOOD: Force GC every 5 seconds
let tickCount = 0;
pollingInterval = setInterval(() => {
  tickCount++;

  // ... normal work

  // Force GC every 5 seconds (40Hz * 5s = 200 ticks)
  if (global.gc && tickCount % 200 === 0) {
    global.gc();
  }
}, pollIntervalMs);
```

## Log Files

All profiling data is saved to:

- **Memory log (1Hz samples):**
  `/home/dev/dev/speciate/docs/performance/memory-profile.jsonl`

- **Heap snapshots:**
  `/home/dev/dev/speciate/docs/performance/snapshots/heap-*.heapsnapshot`

- **Console logs:**
  Check terminal running `./memory-profile.sh`

## Key Metrics

### process.memoryUsage() Fields

- **RSS (Resident Set Size):** Total memory allocated by the process (includes heap + external + code)
- **heapTotal:** V8 heap capacity (grows as needed)
- **heapUsed:** Actual V8 heap usage (JavaScript objects)
- **external:** C++ objects managed by V8 (e.g., ArrayBuffers backing typed arrays)
- **arrayBuffers:** Total size of all ArrayBuffer objects (subset of external)

### What to Watch

- **heapUsed growing:** JavaScript object leak (closures, arrays, event listeners)
- **external growing:** NAPI addon leak (Rust objects not dropped)
- **arrayBuffers growing:** Typed array leak (buffer.subarray() abuse)
- **RSS growing but heap stable:** Native memory leak (unlikely in Electron)

## Integration with Dev-UI

The `V8HeapProfiler` component is already created. To add it to dev-ui:

1. **Import in DevToolsApp.tsx:**
   ```typescript
   import { V8HeapProfiler } from './V8HeapProfiler';
   ```

2. **Add to component tree:**
   ```typescript
   <V8HeapProfiler />
   ```

3. **The component will automatically:**
   - Subscribe to `memory-update` IPC events
   - Display real-time sparklines
   - Provide GC trigger button
   - Provide heap snapshot button

## Chrome DevTools Analysis

Once you have a `.heapsnapshot` file:

1. **Open Chrome DevTools:**
   ```bash
   google-chrome
   # F12 → Memory tab
   ```

2. **Load profile:**
   - Click "Load" button
   - Select `.heapsnapshot` file

3. **Take comparison snapshot:**
   - Click "Heap Snapshot" button in dev-ui again (after 60s)
   - Load second snapshot in Chrome
   - Use "Comparison" view to see growth

4. **Look for:**
   - **Summary view:** Large object types
   - **Comparison view:** Objects allocated between snapshots
   - **Containment view:** Reference chains (why object is retained)
   - **Dominators view:** Objects holding the most memory

## Success Criteria

Memory profiling is complete when:

1. **Stable growth rate:**
   - heapUsed: < 0.1 MB/s
   - external: < 0.1 MB/s
   - arrayBuffers: ~0.0 MB/s

2. **GC effectiveness:**
   - Manual GC drops heapUsed by > 50%
   - No retained objects in heap snapshot comparison

3. **Long-term stability:**
   - 10-minute run with 0 creatures shows < 50 MB total growth
   - 10-minute run with 10K creatures shows predictable, bounded growth

## References

- [Node.js process.memoryUsage()](https://nodejs.org/api/process.html#processmemoryusage)
- [V8 Heap Snapshots](https://developer.chrome.com/docs/devtools/memory-problems/heap-snapshots/)
- [Electron IPC Performance](https://www.electronjs.org/docs/latest/tutorial/ipc#performance-considerations)
- [Structured Clone Algorithm](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm)
