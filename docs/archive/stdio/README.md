# Stdio IPC Architecture (ARCHIVED)

**Archived:** 2025-11-23
**Reason:** Superseded by NAPI-RS bridge in Sprint 13
**Status:** No longer maintained

---

## Overview

This directory contains the legacy stdio-based IPC system that was used to communicate between the Rust simulation and Electron frontend before the NAPI-RS migration.

## Architecture (Deprecated)

The stdio IPC system used:
- **stdout** for sending game state frames (MessagePack serialized)
- **stdin** for receiving commands from Electron
- **Length-prefixed frames** for message delimiting

### Components:

1. **`hooks.rs`** - StdioHooks implementation
   - Serialized game state to MessagePack
   - Wrote frames to stdout with length prefix
   - Background writer thread for non-blocking I/O

2. **`stdin_reader.rs`** - Command reader
   - Read commands from stdin
   - Parsed MessagePack frames
   - Dispatched to simulation

3. **`main.rs.bak`** - CLI entry point
   - Legacy main function using stdio hooks
   - Replaced by NAPI-RS native module

## Why It Was Replaced

**Problems with stdio IPC:**
- **No zero-copy** - All data must be serialized/deserialized
- **No direct TypeScript integration** - Requires separate subprocess management
- **Complex frame protocol** - Manual length-prefix handling
- **Limited type safety** - MessagePack schema not enforced
- **Debugging difficulty** - Binary stdio streams hard to inspect

**Advantages of NAPI-RS:**
- **Zero-copy buffers** - Direct TypeScript access to Rust memory
- **Native module** - No subprocess, runs in same process
- **Type safety** - TypeScript definitions generated from Rust types
- **Better debugging** - Native debugger support
- **Performance** - ~10x faster than stdio (benchmarked in Sprint 13)

## Timeline

- **Sprint 1-12:** Stdio IPC used exclusively
- **Sprint 13:** NAPI-RS migration completed
  - Double-buffer implementation
  - Zero-copy position export
  - ThreadsafeFunction for telemetry
- **2025-11-23:** Stdio code archived and removed from active codebase

## Migration Notes

The NAPI migration was documented in:
- `SPRINT_DOCS/SPRINT_PLAN_sprint-13-napi-rs-migration.md`
- `docs/architecture/electron-architecture.md` (updated)

Key differences:
- **Before:** Subprocess with stdio IPC
- **After:** Native Node.js addon (`.node` file)
- **Before:** ~30ms frame serialization overhead
- **After:** <1ms for buffer swap (zero-copy)

## Lessons Learned

1. **Start with NAPI** - If targeting Electron, use NAPI from day one
2. **Stdio has its place** - Good for CLI tools, not ideal for high-frequency IPC
3. **Benchmarking matters** - Stdio issues weren't apparent until 100K+ creatures
4. **Migration was smooth** - Clean separation of concerns made switch easy

## See Also

- `docs/architecture/napi-architecture.md` - Current NAPI implementation
- `apps/simulation/src/napi_addon/` - NAPI bridge code
- `apps/simulation/src/ipc/bridge/` - Double-buffer implementation

---

**Files in this archive:**
- `hooks.rs` - StdioHooks (443 lines, 13,881 bytes)
- `mod.rs` - Module re-exports (3 lines, 108 bytes)
- `stdin_reader.rs` - Command reader (212 lines, 6,112 bytes)
- `main.rs.bak` - Legacy CLI entry point (383 lines, 13,197 bytes)

**Total archived:** ~1,041 lines, ~33KB of dead code removed from active codebase
