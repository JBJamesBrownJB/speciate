# Memory Profiling Quick Start

## TL;DR

```bash
# 1. Start memory profiling mode
cd /home/dev/dev/speciate/apps/portal
chmod +x memory-profile.sh
./memory-profile.sh

# 2. Wait 60 seconds with 0 creatures

# 3. Analyze (in another terminal)
cd /home/dev/dev/speciate/apps/portal
node analyze-memory.js

# 4. Check diagnosis output for leak type:
#    - V8 Heap Leak → JavaScript objects not GC'd
#    - External Memory Leak → NAPI addon leak
#    - ArrayBuffer Leak → Typed array leak
```

## What You Get

1. **Memory profiling mode** (`napi-memory-profile.cjs`)
   - Logs memory every 1 second to `docs/performance/memory-profile.jsonl`
   - Sends live data to dev-ui V8HeapProfiler component
   - Enables manual GC with `--expose-gc` flag

2. **Analysis script** (`analyze-memory.js`)
   - Parses log file
   - Shows baseline vs current memory
   - Calculates growth rates (MB/s)
   - Auto-diagnoses leak type

3. **Dev-UI component** (`V8HeapProfiler.tsx`)
   - Real-time sparklines for heap/external/ArrayBuffers
   - "Trigger GC" button
   - "Heap Snapshot" button
   - Baseline comparison

## Key Commands

### From Dev-UI
- **Trigger GC:** Click button → Forces garbage collection → Check if memory drops
- **Heap Snapshot:** Click button → Saves `.heapsnapshot` → Analyze in Chrome DevTools

### From Terminal
```bash
# Start profiling
./memory-profile.sh

# Analyze logs
node analyze-memory.js

# View raw log (1 JSON object per line)
tail -f ../../docs/performance/memory-profile.jsonl
```

## What to Look For

### In analyze-memory.js output:

```
GROWTH RATE (per second):
  RSS:          +2.5 MB/s   ← Total process memory
  Heap Used:    +1.8 MB/s   ← JavaScript objects (LEAK!)
  External:     +0.5 MB/s   ← NAPI/ArrayBuffers
  ArrayBuffers: +0.4 MB/s   ← Typed arrays (LEAK!)
```

**Good (stable):**
- Heap Used: < 0.1 MB/s
- External: < 0.1 MB/s
- ArrayBuffers: ~0.0 MB/s

**Bad (leak):**
- Any value > 0.5 MB/s

### In Console Logs:

```
[Memory BASELINE]
  RSS:          245.32 MB
  Heap Total:   45.12 MB
  Heap Used:    28.45 MB
  External:     12.34 MB
  ArrayBuffers: 8.12 MB

[Memory AFTER simulation.start()]
  RSS:          248.12 MB   ← +2.8 MB (expected)
  Heap Used:    29.01 MB   ← +0.56 MB (expected)

[Memory BEFORE GC]
  Heap Used:    85.23 MB

[Memory AFTER GC]
  Heap Used:    30.12 MB   ← Dropped 55 MB (GC working!)
```

## Common Leak Sources

### 1. Polling Loop (40Hz IPC sends)
**File:** `electron/napi-memory-profile.cjs:194`
```javascript
pollingInterval = setInterval(() => {
  const buffer = fullBuffer.subarray(0, usedSize);  // New view every tick
  mainWindow.webContents.send('napi-buffer-update', {
    buffer: buffer,  // Structured clone copies this!
    creatureCount,
  });
}, pollIntervalMs);  // 25ms = 40Hz
```

**Hypothesis:** Structured clone algorithm copies buffer 40 times/second

**Test:** Comment out `mainWindow.webContents.send()` and re-profile

### 2. Perception Debug Buffer
**File:** `electron/napi-memory-profile.cjs:215`
```javascript
const debugBuffer = simulationEngine.getPerceptionDebug();
if (debugBuffer[0] > 0.5) {
  mainWindow.webContents.send('perception-debug-update', debugBuffer);
}
```

**Hypothesis:** Similar to above, but for debug buffer

**Test:** Comment out and re-profile

### 3. Telemetry JSON Parsing
**File:** `electron/napi-memory-profile.cjs:222`
```javascript
if (tick % 30 === 0) {
  const telemetryJson = simulationEngine.getTelemetry();
  const telemetry = JSON.parse(telemetryJson);  // New object every 30 ticks
}
```

**Hypothesis:** JSON parsing creates objects not GC'd fast enough

**Test:** Comment out and re-profile

## Next Steps After Diagnosis

### If Heap Leak (JavaScript):
1. Take 2 heap snapshots (60s apart)
2. Open in Chrome DevTools Memory tab
3. Use "Comparison" view
4. Find objects growing between snapshots
5. Trace retention path in "Containment" view

### If ArrayBuffer Leak:
1. Check `buffer.subarray()` usage
2. Check IPC structured clone behavior
3. Test with direct buffer reference (no subarray)
4. Test with IPC disabled

### If External Leak:
1. Check Rust NAPI addon (`simulation_engine.rs`)
2. Look for `napi::Ref` or `napi::External` not dropped
3. Check Bevy resource cleanup
4. Run dhat profiler on Rust side (already done, showed clean)

## Files Created

```
apps/portal/
  electron/napi-memory-profile.cjs       ← Memory profiling main process
  electron/preload-memory-profile.cjs    ← Extended preload with GC/snapshot IPC
  memory-profile.sh                      ← Launch script
  analyze-memory.js                      ← Log analysis script

apps/dev-ui/
  src/components/V8HeapProfiler.tsx      ← Dev-UI component

docs/performance/
  MEMORY_PROFILING.md                    ← Full guide
  memory-profile.jsonl                   ← Log file (created at runtime)
  snapshots/heap-*.heapsnapshot          ← Heap snapshots (created on demand)
```

## Full Documentation

See `/home/dev/dev/speciate/docs/performance/MEMORY_PROFILING.md`
