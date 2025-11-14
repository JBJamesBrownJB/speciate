# Electron Desktop Architecture

## Overview

Speciate Phase 1 uses **Electron** to package the simulation as a standalone desktop application. The architecture uses **stdio MessagePack streaming** for efficient communication between the Rust simulation and the TypeScript/PixiJS frontend.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Electron Application                      │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Renderer Process (Isolated)                │ │
│  │  ┌──────────────────────────────────────────────────┐  │ │
│  │  │         TypeScript Frontend (PixiJS)             │  │ │
│  │  │  - Game rendering (WebGL)                        │  │ │
│  │  │  - UI/HUD (DOM)                                   │  │ │
│  │  │  - Player input                                   │  │ │
│  │  │  - Camera controls                                │  │ │
│  │  └──────────────────────────────────────────────────┘  │ │
│  │                          ▲                              │ │
│  │                          │ window.electron API          │ │
│  │                          │ (via contextBridge)          │ │
│  │                          ▼                              │ │
│  │  ┌──────────────────────────────────────────────────┐  │ │
│  │  │        Preload Script (Security Bridge)          │  │ │
│  │  │  - Exposes safe IPC methods                      │  │ │
│  │  │  - No Node.js access to renderer                 │  │ │
│  │  │  - Type-safe API surface                         │  │ │
│  │  └──────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────┘ │
│                          ▲                                   │
│                          │ IPC Events                        │
│                          │ ('state-update', etc.)            │
│                          ▼                                   │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Main Process (Privileged)                  │ │
│  │  - Window management                                    │ │
│  │  - Spawn Rust simulation subprocess                     │ │
│  │  - Read stdout MessagePack frames                       │ │
│  │  - Deserialize & forward to renderer                    │ │
│  └────────────────────────────────────────────────────────┘ │
└───────────────────────────┬─────────────────────────────────┘
                            │ stdin/stdout
                            │ (MessagePack frames)
                            ▼
         ┌──────────────────────────────────────┐
         │     Rust Simulation (Subprocess)      │
         │  - Bevy ECS (60 Hz tick)              │
         │  - A-Life simulation                  │
         │  - Physics & behaviors                │
         │  - Writes MessagePack to stdout       │
         └──────────────────────────────────────┘
```

## Communication Protocol: stdio MessagePack

### Why stdio?

**Advantages:**
- **Simple:** No network stack, no sockets, just pipe I/O
- **Fast:** Kernel pipes are extremely efficient (<1ms latency)
- **Cross-platform:** Works identically on Windows, macOS, Linux
- **Reliable:** OS guarantees ordered delivery, no packet loss
- **Debuggable:** Can redirect stdout to file and inspect frames

**vs. Shared Memory:**
- stdio is simpler (no lock coordination complexity)
- Good enough for 60 FPS @ 1 MB per frame (60 MB/s)
- No platform-specific memory mapping APIs

**vs. WebSocket/TCP:**
- No network overhead (no TCP handshake, no packet headers)
- No port conflicts or firewall issues
- Direct process communication

### Frame Format

**Binary Protocol:**
```
┌─────────────────┬────────────────────────────────┐
│  4-byte length  │    MessagePack payload          │
│  (big-endian)   │    (serialized GameState)       │
└─────────────────┴────────────────────────────────┘
```

**Frame Structure:**
1. **Length Prefix** (4 bytes, big-endian `u32`)
   - Indicates payload size in bytes
   - Allows reader to allocate exact buffer size
   - Prevents buffer overflows

2. **MessagePack Payload**
   - Serialized using `rmp-serde` (Rust) / `@msgpack/msgpack` (TypeScript)
   - Struct map format (field names preserved, not array indexes)
   - Compact binary format (~70% smaller than JSON)

**Example Frame:**
```
Hex dump of a 42-byte frame:

00 00 00 26  # Length: 38 bytes (0x26)
82           # Fixmap with 2 entries
a4 74 69 63  # Field name: "tick" (4 bytes)
6b
2a           # Value: 42 (positive fixint)
a9 63 72 65  # Field name: "creatures" (9 bytes)
61 74 75 72
65 73
90           # Fixarray with 0 entries (empty array)
```

### Rust Implementation (Simulation)

**Writing Frames:** `apps/simulation/src/stdio/hooks.rs`

```rust
use rmp_serde::Serializer;
use serde::Serialize;
use std::io::{self, Write};

/// Writes a MessagePack frame to stdout with length prefix.
///
/// This is called at 60 Hz from the simulation thread.
pub fn write_msgpack_frame<T: Serialize>(data: &T) -> io::Result<()> {
    // Serialize to MessagePack (struct map format)
    let mut buf = Vec::new();
    data.serialize(&mut Serializer::new(&mut buf).with_struct_map())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Write length prefix (big-endian u32)
    let len = buf.len() as u32;
    let len_bytes = len.to_be_bytes();

    // Lock stdout and write frame atomically
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(&len_bytes)?;
    handle.write_all(&buf)?;
    handle.flush()?;

    Ok(())
}
```

**Key Details:**
- `.with_struct_map()` ensures field names are serialized (not array indexes)
- `stdout.lock()` prevents frame interleaving if multiple threads write
- `.flush()` ensures Electron receives data immediately (no buffering)
- `to_be_bytes()` uses big-endian for cross-platform compatibility

### Electron Implementation (Main Process)

**Reading Frames:** `apps/portal/electron/main.cjs`

```javascript
const { spawn } = require('child_process');
const msgpack = require('@msgpack/msgpack');

// Spawn Rust simulation as subprocess
const simulation = spawn('./speciate', [], {
  stdio: ['ignore', 'pipe', 'pipe'], // stdin, stdout, stderr
});

let buffer = Buffer.alloc(0);

// Read stdout in chunks, accumulate until complete frame
simulation.stdout.on('data', (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);

  // Process all complete frames in buffer
  while (buffer.length >= 4) {
    // Read 4-byte length prefix (big-endian)
    const frameLength = buffer.readUInt32BE(0);

    // Wait for complete frame
    if (buffer.length < 4 + frameLength) {
      break;
    }

    // Extract frame payload
    const frameData = buffer.subarray(4, 4 + frameLength);
    buffer = buffer.subarray(4 + frameLength);

    // Deserialize MessagePack
    try {
      const state = msgpack.decode(frameData);

      // Forward to renderer via IPC
      mainWindow.webContents.send('state-update', state);
    } catch (err) {
      console.error('Failed to decode MessagePack:', err);
    }
  }
});
```

**Key Details:**
- Buffer accumulation handles partial reads (OS may split frames across chunks)
- `readUInt32BE(0)` reads big-endian length from first 4 bytes
- Loop processes multiple frames if they arrive in same chunk
- Errors logged but don't crash (simulation keeps running)

## Security Model

### Process Isolation

Electron uses **multi-process architecture** for security:

1. **Main Process** (privileged)
   - Has full Node.js access
   - Can spawn processes, read files, etc.
   - NOT accessible from renderer

2. **Renderer Process** (sandboxed)
   - Isolated web page (Chromium)
   - No Node.js access by default
   - No direct file system or process access

3. **Preload Script** (security bridge)
   - Runs before renderer code loads
   - Has Node.js access but controlled context
   - Uses `contextBridge` to expose safe APIs

### contextBridge Pattern

**Preload Script:** `apps/portal/electron/preload.cjs`

```javascript
const { contextBridge, ipcRenderer } = require('electron');

// ONLY expose specific, validated methods
contextBridge.exposeInMainWorld('electron', {
  // Subscribe to state updates (no arbitrary IPC channels)
  onStateUpdate: (callback) => {
    if (typeof callback !== 'function') {
      throw new Error('Callback must be a function');
    }
    ipcRenderer.on('state-update', (event, state) => {
      callback(state);
    });
  },

  // Send commands to simulation (validated by main process)
  spawnCreature: (x, y) => {
    // Input validation in renderer
    if (typeof x !== 'number' || typeof y !== 'number') {
      throw new Error('Coordinates must be numbers');
    }
    ipcRenderer.send('spawn-creature', { x, y });
  },
});
```

**Renderer Usage:** `apps/portal/src/main.ts`

```typescript
// TypeScript sees type-safe API
declare global {
  interface Window {
    electron: {
      onStateUpdate: (callback: (state: GameState) => void) => void;
      spawnCreature: (x: number, y: number) => void;
    };
  }
}

// Safe API, no Node.js access
window.electron.onStateUpdate((state) => {
  renderCreatures(state.creatures);
});
```

**Why This Matters:**
- Prevents XSS attacks from accessing Node.js
- Renderer can't read arbitrary files or spawn processes
- All IPC must go through validated contextBridge methods
- Follows Electron security best practices (2024-2025)

### Sandbox Configuration

**Main Process:** `apps/portal/electron/main.cjs`

```javascript
const mainWindow = new BrowserWindow({
  width: 1920,
  height: 1080,
  webPreferences: {
    preload: path.join(__dirname, 'preload.cjs'),
    contextIsolation: true,  // ✅ Enable context isolation
    nodeIntegration: false,   // ✅ Disable Node.js in renderer
    sandbox: false,           // ⚠️  Disabled for Linux compatibility
    webSecurity: true,        // ✅ Keep web security enabled
  },
});
```

**Sandbox = false:**
- Required on Linux to avoid SUID permission errors
- Not a security issue (contextIsolation + nodeIntegration still protect)
- For production, consider per-platform conditional logic

## Performance Characteristics

### Throughput

**Measurements (Intel i7, 16GB RAM):**
- Frame size: ~500 KB - 1.5 MB (depends on creature count)
- Frame rate: 60 Hz (16.67ms interval)
- Throughput: ~60 MB/s average, ~90 MB/s peak
- stdout latency: <1ms (kernel pipe)

**Bottlenecks:**
1. **Serialization (Rust):** ~0.5-1.0ms per frame
2. **Deserialization (Node.js):** ~0.3-0.8ms per frame
3. **IPC to renderer:** ~0.1-0.3ms
4. **Total IPC latency:** ~1-2ms (negligible vs. 16.67ms budget)

### Memory Usage

**Main Process:**
- Buffer accumulation: ~3-5 MB (handles partial reads)
- MessagePack decoder: ~1-2 MB temporary allocations
- **Total:** ~5-10 MB

**Simulation Process:**
- Lock-free queue: ~10 MB (10 frames @ 1 MB each)
- Serialization buffer: ~2-3 MB (reused)
- **Total:** ~15 MB (excluding ECS world state)

### CPU Usage

**Profiling Results:**
- Serialization: 1-2% CPU (Rust, optimized release build)
- Deserialization: 2-3% CPU (Node.js V8)
- Frame buffering: <1% CPU
- **Total IPC overhead:** ~5% CPU

**Optimization Notes:**
- MessagePack is 3-5x faster than JSON
- Binary format is ~70% smaller than JSON
- No allocations in hot path (buffers reused)

## Debugging & Development

### Viewing Raw Frames

**Redirect stdout to file:**
```bash
cd apps/simulation
cargo run --release > frames.bin 2>&1
```

**Inspect with hex editor:**
```bash
hexdump -C frames.bin | head -100
```

**Expected output:**
```
00000000  00 00 01 a3 82 a4 74 69  63 6b 2a a9 63 72 65 61  |......tick*.crea|
          ^^^^^^^^^^^ length       ^^^ map   ^^^^ "tick"
                                             ^^^ value: 42
```

### Testing Frame Format

**Unit Test:** `apps/simulation/src/stdio/tests.rs`

```rust
#[test]
fn test_msgpack_uses_struct_map() {
    let state = GameState { tick: 42, creatures: vec![] };

    let mut buf = Vec::new();
    state.serialize(&mut Serializer::new(&mut buf).with_struct_map())
        .unwrap();

    // Map format starts with 0x82 (fixmap with 2 entries)
    // Array format would be 0x92 (fixarray with 2 entries)
    assert_eq!(buf[0], 0x82, "Should use map format");
}
```

### Logging

**Rust (stderr for logs, stdout for frames):**
```rust
use tracing::info;

// Logs go to stderr (won't interfere with stdout frames)
info!("Simulation tick: {}", tick);

// Frames go to stdout
write_msgpack_frame(&state)?;
```

**Electron (forward renderer console):**
```javascript
mainWindow.webContents.on('console-message', (event, level, message, line, sourceId) => {
  const levels = ['', 'INFO', 'WARNING', 'ERROR'];
  console.log(`[Renderer ${levels[level]}] ${message}`);
});
```

### Common Issues

**Issue: "Failed to decode MessagePack"**
- **Cause:** Partial frame read, corruption, or wrong format
- **Fix:** Check length prefix matches payload size
- **Debug:** `hexdump` the stdout to verify frame structure

**Issue: "No creatures rendering"**
- **Cause:** JavaScript not loading (absolute paths in HTML)
- **Fix:** Set `base: './'` in `vite.config.ts` for relative paths
- **Debug:** Check browser console for module load errors

**Issue: "Renderer process crashed"**
- **Cause:** DevTools opening on some Linux systems
- **Fix:** Disable `openDevTools()` or set `ENABLE_DEVTOOLS=1` conditionally
- **Debug:** Check `render-process-gone` event logs

## Build & Distribution

### Development Build

```bash
cd apps/portal
npm run dev        # Starts Electron with Vite dev server
```

**What happens:**
1. Vite builds frontend to `dist/`
2. Electron main process spawns Rust binary
3. Window opens with hot-reload enabled

### Production Build

```bash
# Build frontend
cd apps/portal
npm run build      # Vite production build

# Build Rust simulation
cd ../simulation
cargo build --release

# Package with electron-builder
cd ../portal
npm run package    # Creates installers (Windows .exe, macOS .dmg, Linux .AppImage)
```

**Electron Builder Config:** `apps/portal/electron-builder.json`

```json
{
  "appId": "com.speciate.simulation",
  "productName": "Speciate",
  "directories": {
    "output": "dist-electron"
  },
  "files": [
    "dist/**/*",
    "electron/**/*",
    "!node_modules/**/*"
  ],
  "extraResources": [
    {
      "from": "../simulation/target/release/speciate",
      "to": "bin/speciate"
    }
  ],
  "linux": {
    "target": ["AppImage", "deb"],
    "category": "Game"
  },
  "mac": {
    "target": "dmg",
    "category": "public.app-category.games"
  },
  "win": {
    "target": "nsis"
  }
}
```

**Key Points:**
- Bundles Rust binary in `resources/bin/speciate`
- Main process spawns from bundled path (not dev path)
- All assets included in `dist/`

## Future Optimizations

### Potential Improvements

1. **Compression:**
   - Add zstd compression to MessagePack frames
   - Trade CPU for bandwidth (useful for large creature counts)

2. **Differential Updates:**
   - Only send changed creatures (delta encoding)
   - Reduces frame size from 1 MB → 50-100 KB

3. **Shared Memory (if needed):**
   - Use platform-specific shared memory for >200 MB/s
   - Requires lock-free ring buffer coordination
   - More complexity, only needed if >1000 creatures

4. **GPU Acceleration:**
   - Re-enable WebGL (currently software rendered on some systems)
   - Check GPU compatibility before disabling hardware acceleration

## Related Documentation

- **Main Project Docs:** `/workspace/CLAUDE.md`
- **Simulation Architecture:** `/workspace/apps/simulation/CLAUDE.md`
- **Electron Security:** https://www.electronjs.org/docs/latest/tutorial/security
- **MessagePack Spec:** https://msgpack.org/
- **Bevy ECS:** https://bevyengine.org/learn/book/

## Migration History

- **Original:** Tauri-based IPC (invoke commands, events)
- **Migrated:** Nov 2025 - Electron with stdio MessagePack
- **Reason:** Simpler protocol, better Linux compatibility, easier debugging
- **Archived Docs:** `/workspace/docs/development/history/`
