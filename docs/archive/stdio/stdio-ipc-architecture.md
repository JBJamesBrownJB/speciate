# stdio IPC Architecture (Pre-NAPI Migration)

**Date:** 2025-11-22
**Sprint:** Sprint 13 (NAPI-RS Migration) - Phase 0.7
**Purpose:** Document current stdio IPC architecture before NAPI-RS migration

---

## Executive Summary

The current architecture uses **stdio MessagePack** for bidirectional IPC between Rust subprocess and Electron main process:
- **stdin:** Electron → Rust (commands)
- **stdout:** Rust → Electron (game state frames)

**Known Bottlenecks:**
1. **IPC Serialization:** 810 μs/frame (57% of ECS time) at 27.5K creatures
2. **Writer Thread:** 19.3 ms blocking time
3. **Frame Drops:** 42 avg (channel saturation at 100%)

**Total IPC Overhead:** ~20.2 ms/frame ← **THIS WILL BE ELIMINATED**

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                     ELECTRON MAIN PROCESS                            │
├──────────────────────────┬──────────────────────────────────────────┤
│  JavaScript              │  Rust Subprocess                          │
│  (spawn child process)   │  (Bevy ECS)                              │
│                          │                                           │
│  ┌────────────────┐      │  ┌──────────────────┐                    │
│  │ stdin.write()  │──────┼─>│ stdin_reader.rs  │                    │
│  │ (Commands)     │      │  │ (Background      │                    │
│  │                │      │  │  Thread)         │                    │
│  └────────────────┘      │  └───────┬──────────┘                    │
│                          │          │                                │
│                          │          v                                │
│                          │  ┌──────────────────┐                    │
│                          │  │ mpsc::Receiver   │                    │
│                          │  │ (Bounded channel)│                    │
│                          │  └───────┬──────────┘                    │
│                          │          │                                │
│  ┌────────────────┐      │          v                                │
│  │ stdout.on      │<─────┼──┌──────────────────┐                    │
│  │ ('data')       │      │  │ Command Executor │                    │
│  │ (GameState     │      │  │ System (Bevy)    │                    │
│  │  frames)       │      │  └──────────────────┘                    │
│  └────────────────┘      │                                           │
│         ^                │  ┌──────────────────┐                    │
│         │                │  │ ECS Update Loop  │                    │
│         │                │  │ (Simulation tick)│                    │
│         │                │  └───────┬──────────┘                    │
│         │                │          │                                │
│         │                │          v                                │
│         │                │  ┌──────────────────┐                    │
│         │                │  │ Snapshot Queue   │                    │
│         │                │  │ (ArrayQueue<2>)  │                    │
│         │                │  └───────┬──────────┘                    │
│         │                │          │                                │
│         │                │          v                                │
│         │                │  ┌──────────────────┐                    │
│         └────────────────┼──│ Writer Thread    │                    │
│                          │  │ (Background)     │                    │
│                          │  │ - Serialize      │                    │
│                          │  │ - Write stdout   │                    │
│                          │  └──────────────────┘                    │
│                          │                                           │
└──────────────────────────┴──────────────────────────────────────────┘
```

---

## Component Breakdown

### 1. stdin Reader Thread

**File:** `apps/simulation/src/ipc/stdin_reader.rs`

**Purpose:** Background thread that reads length-prefixed MessagePack frames from stdin and forwards commands to the main simulation thread.

**Protocol:**
```
┌──────────┬────────────────────┐
│ 4 bytes  │    N bytes         │
│ u32 BE   │  MessagePack       │
│ (Length) │  (Command enum)    │
└──────────┴────────────────────┘
```

**Flow:**
```rust
spawn_stdin_reader_thread(tx: Sender<Command>) {
    loop {
        // 1. Read 4-byte length prefix (big-endian u32)
        let mut len_bytes = [0u8; 4];
        stdin.read_exact(&mut len_bytes)?;
        let frame_length = u32::from_be_bytes(len_bytes);

        // 2. Validate frame size (reject > 1MB)
        if frame_length > 1024 * 1024 {
            return Err("Frame too large");
        }

        // 3. Read payload
        buffer.resize(frame_length, 0);
        stdin.read_exact(buffer)?;

        // 4. Deserialize MessagePack → Command
        let command = rmp_serde::from_slice(buffer)?;

        // 5. Send to main thread via mpsc channel
        tx.send(command)?;
    }
}
```

**Error Handling:**
- **UnexpectedEof:** Exit gracefully (subprocess shutdown)
- **InvalidData:** Log error, continue reading
- **Frame too large:** Reject and continue
- **Channel closed:** Exit thread

---

### 2. Command Enum

**File:** `apps/simulation/src/ipc/commands.rs`

**Definition:**
```rust
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    DevSpawnCreature { x: f32, y: f32, dna: Option<Value> },
    DevLoadTrial { template: String },
    DevClearCreatures,
}
```

**JSON Example (before MessagePack serialization):**
```json
{
  "type": "dev_spawn_creature",
  "x": 100.5,
  "y": 200.5,
  "dna": null
}
```

**MessagePack Efficiency:**
- JSON size: ~70 bytes (above example)
- MessagePack size: ~30 bytes (57% reduction)
- No string keys in serialized format (uses indices)

---

### 3. Command Executor System

**File:** `apps/simulation/src/ipc/command_executor.rs`

**Purpose:** Bevy system that drains command queue and executes commands (spawn/despawn entities).

**Execution:** Runs at start of each ECS frame (before physics, behaviors)

**Pattern:**
```rust
fn command_executor_system(world: &mut World) {
    // 1. Drain all commands from mpsc receiver
    let commands: Vec<Command> = {
        let rx = world.resource::<CommandReceiver>().0.lock();
        let mut cmds = Vec::new();
        while let Ok(cmd) = rx.try_recv() {  // Non-blocking
            cmds.push(cmd);
        }
        cmds
    };

    // 2. Execute each command
    for cmd in commands {
        match cmd {
            Command::DevSpawnCreature { x, y, dna } => {
                world.spawn((Position { x, y }, Velocity::default(), ...));
            }
            Command::DevClearCreatures => {
                // Query all creatures and despawn
            }
            Command::DevLoadTrial { template } => {
                // Load trial config and spawn pattern
            }
        }
    }
}
```

**Channel:** `mpsc::channel()` (unbounded, stdlib implementation)

---

### 4. GameState Snapshot

**File:** `apps/simulation/src/ipc/snapshot_queue.rs`

**Structure:**
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub protocol_version: u8,
    pub tick: u64,
    pub tick_rate_hz: f32,
    pub creatures: Vec<CreatureSnapshot>,  // ALL creatures (no viewport culling)
    pub entity_count: usize,
    pub system_timings_us: SystemTimingsSnapshot,
    pub hardware_metrics: Option<HardwareSnapshot>,
    pub parallelization_metrics: Option<ParallelizationSnapshot>,
}

pub struct CreatureSnapshot {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub size: f32,
}
```

**Size Analysis (27.5K creatures):**
- Creature data: 27,500 × 20 bytes = 550 KB (MessagePack)
- Metadata + metrics: ~5 KB
- **Total frame size:** ~555 KB

**Frequency:** Every frame (30Hz tick rate = 33.3ms interval)

---

### 5. Snapshot Queue

**File:** `apps/simulation/src/ipc/snapshot_queue.rs`

**Implementation:**
```rust
pub struct SnapshotQueue {
    queue: Arc<ArrayQueue<GameState>>,  // Lock-free queue
}

impl SnapshotQueue {
    pub fn new(capacity: usize) -> Self {
        Self { queue: Arc::new(ArrayQueue::new(capacity)) }
    }

    pub fn push(&self, state: GameState) {
        // CRITICAL: Drop oldest frame if full
        if self.queue.is_full() {
            let _ = self.queue.pop();  // Discard oldest
        }
        let _ = self.queue.push(state);
    }
}
```

**Capacity:** 2 frames (allows 1 frame overlap)

**Behavior:**
- **Producer:** Main ECS thread (after each tick)
- **Consumer:** Writer thread (background)
- **Overflow handling:** Drop oldest frame (graceful degradation)

**Frame Drops (from baseline):**
- 27.5K creatures: **42 avg frame drops**
- Cause: Writer thread can't keep up with 30Hz production rate

---

### 6. Writer Thread

**File:** `apps/simulation/src/stdio/` (hooks.rs uses snapshot queue)

**Purpose:** Background thread that serializes GameState to MessagePack and writes to stdout.

**Flow:**
```
loop {
    // 1. Pop snapshot from queue (non-blocking)
    if let Some(state) = queue.pop() {
        // 2. Serialize to MessagePack (BOTTLENECK)
        let payload = rmp_serde::to_vec(&state)?;  // 810 μs

        // 3. Write 4-byte length prefix
        let len = payload.len() as u32;
        stdout.write_all(&len.to_be_bytes())?;

        // 4. Write payload
        stdout.write_all(&payload)?;

        // 5. Flush
        stdout.flush()?;
    }

    // 6. Sleep if queue empty
    thread::sleep(Duration::from_millis(1));
}
```

**Measured Overhead (27.5K creatures):**
- **Serialization:** 810 μs (57% of ECS time)
- **Total writer thread time:** 19.3 ms
- **Blocking ratio:** 100% (channel saturated)

**Why so slow?**
1. **MessagePack serialization:** CPU-intensive (traverses entire GameState tree)
2. **Memory allocation:** Vec<CreatureSnapshot> copied during serialization
3. **No zero-copy:** Data is duplicated (Bevy → GameState → MessagePack → stdout buffer)

---

## Known Bottlenecks (Measured)

### Bottleneck #1: IPC Serialization (810 μs)

**Location:** Writer thread, MessagePack serialization

**Measurement (from baseline snapshot):**
```
ipc_serialize_us: 810 μs (57% of total ECS time)
```

**Root Cause:**
- MessagePack must traverse entire GameState structure
- 27,500 creatures × 5 fields × serialize operation
- No SIMD optimization
- Heap allocations during serialization

**Impact:**
- At 27.5K creatures: 810 μs/frame
- At 150K creatures (projected): ~4,400 μs (4.4 ms) ← **UNACCEPTABLE**

**Post-NAPI Projection:**
- Zero-copy buffer read: **<10 μs** (99% reduction)

---

### Bottleneck #2: Writer Thread Blocking (19.3 ms)

**Location:** Writer thread execution time

**Measurement (from baseline snapshot):**
```
ipc_writer_thread_us: 19,355 μs (19.3 ms)
```

**Root Cause:**
- Serialization (810 μs) + I/O write + flush
- 100% channel utilization (queue always full)
- Frame drops when writer can't keep up

**Impact:**
- Main thread waits for channel to drain
- Effective tick rate drops when writer is saturated
- 42 avg frame drops = 42 frames/second not sent to frontend

**Post-NAPI Projection:**
- **Writer thread ELIMINATED** (no serialization, no background thread)

---

### Bottleneck #3: Frame Drops (42 avg)

**Location:** Snapshot queue overflow

**Measurement (from baseline snapshot):**
```
ipc_frame_drops_total: 42 avg
ipc_channel_utilization_pct: 100%
```

**Root Cause:**
- Queue capacity: 2 frames
- Production rate: 30 Hz (33.3 ms interval)
- Consumption rate: slower than 30 Hz (writer can't keep up)
- **Overflow → oldest frame dropped**

**Impact:**
- Frontend sees stuttering (missing frames)
- Sprite positions interpolate incorrectly
- Player experience degradation

**Post-NAPI Projection:**
- **Zero frame drops** (no queue, direct buffer access)

---

## Comparison Table: Pre vs Post NAPI

| Component | Pre-NAPI (Current) | Post-NAPI (Target) | Improvement |
|-----------|--------------------|--------------------|-------------|
| **IPC Serialization** | 810 μs | <10 μs | **99% reduction** |
| **Buffer Access** | N/A (MessagePack) | 350 μs (zero-copy) | (new overhead) |
| **Writer Thread** | 19.3 ms | **Eliminated** | **100% reduction** |
| **Frame Drops** | 42 avg | **0** | **100% reduction** |
| **Total IPC Overhead** | ~20.2 ms | <0.4 ms | **98% reduction** |
| **Channel Utilization** | 100% (saturated) | N/A (no channel) | N/A |

**Net Gain:** ~19.8 ms/frame freed for simulation or higher tick rate

---

## Data Flow Diagrams

### stdin (Commands): Electron → Rust

```
┌──────────────┐
│  Electron    │
│  Main Process│
└──────┬───────┘
       │
       │ JSON (from renderer)
       │
       v
┌──────────────┐
│ JSON.parse   │
│ + validation │
└──────┬───────┘
       │
       │ { type: "dev_spawn_creature", x: 100, y: 200 }
       │
       v
┌──────────────┐
│ MessagePack  │
│ serialize    │
└──────┬───────┘
       │
       │ Binary payload (30 bytes)
       │
       v
┌──────────────┐
│ Length prefix│
│ (4 bytes BE) │
└──────┬───────┘
       │
       │ [0x00, 0x00, 0x00, 0x1E, <msgpack>]
       │
       v
┌──────────────┐
│ stdin.write()│
└──────┬───────┘
       │
       ├─────────────────── IPC BOUNDARY ───────────────────┐
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ Rust stdin   │                                            │
│ reader thread│                                            │
└──────┬───────┘                                            │
       │                                                      │
       │ read_exact(4 bytes) → u32 length                   │
       │ read_exact(length) → Vec<u8> payload               │
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ rmp_serde::  │                                            │
│ from_slice() │                                            │
└──────┬───────┘                                            │
       │                                                      │
       │ Command::DevSpawnCreature { x: 100.0, y: 200.0 }   │
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ mpsc::send() │                                            │
└──────┬───────┘                                            │
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ Bevy ECS     │                                            │
│ Command      │                                            │
│ Executor     │                                            │
│ System       │                                            │
└──────────────┘                                            │
```

**Latency:** <1 ms (commands are small, rarely sent)

---

### stdout (GameState): Rust → Electron

```
┌──────────────┐
│ Bevy ECS     │
│ Update Loop  │
│ (30 Hz tick) │
└──────┬───────┘
       │
       │ Every frame (33.3 ms)
       │
       v
┌──────────────┐
│ Query all    │
│ creatures    │ ← 27,500 entities
└──────┬───────┘
       │
       │ Vec<CreatureSnapshot>
       │
       v
┌──────────────┐
│ Build        │
│ GameState    │
│ struct       │
└──────┬───────┘
       │
       │ GameState { tick, creatures, timings, ... }
       │
       v
┌──────────────┐
│ Push to      │
│ ArrayQueue   │ ← Capacity: 2
│ (lock-free)  │
└──────┬───────┘
       │
       │ If full → drop oldest frame (FRAME DROP)
       │
       ├─────────────────── THREAD BOUNDARY ────────────────┐
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ Writer Thread│                                            │
│ (Background) │                                            │
└──────┬───────┘                                            │
       │                                                      │
       │ Pop from queue (non-blocking)                      │
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ rmp_serde::  │  ← BOTTLENECK: 810 μs                     │
│ to_vec()     │                                            │
└──────┬───────┘                                            │
       │                                                      │
       │ Vec<u8> (555 KB for 27.5K creatures)               │
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ Write length │                                            │
│ prefix       │                                            │
│ (4 bytes BE) │                                            │
└──────┬───────┘                                            │
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ stdout.write │                                            │
│ _all(payload)│                                            │
└──────┬───────┘                                            │
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ stdout.flush │                                            │
└──────┬───────┘                                            │
       │                                                      │
       ├─────────────────── IPC BOUNDARY ───────────────────┤
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ Electron     │                                            │
│ stdout.on    │                                            │
│ ('data')     │                                            │
└──────┬───────┘                                            │
       │                                                      │
       │ Buffer accumulation                                 │
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ Length-prefix│                                            │
│ frame parser │                                            │
└──────┬───────┘                                            │
       │                                                      │
       │ Complete frame detected                             │
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ msgpack.     │                                            │
│ decode()     │                                            │
└──────┬───────┘                                            │
       │                                                      │
       │ GameState object                                    │
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ IPC forward  │                                            │
│ to renderer  │                                            │
└──────┬───────┘                                            │
       │                                                      │
       v                                                      │
┌──────────────┐                                            │
│ PixiJS       │                                            │
│ Update       │                                            │
│ Sprites      │                                            │
└──────────────┘                                            │
```

**Total Latency:** ~20 ms (dominated by serialization)

---

## Memory Layout (Pre-NAPI)

### GameState Structure in Memory

```
GameState {
    protocol_version: u8,           // 1 byte
    tick: u64,                      // 8 bytes
    tick_rate_hz: f32,              // 4 bytes
    creatures: Vec<CreatureSnapshot>, // 27,500 × 20 bytes = 550 KB
    entity_count: usize,            // 8 bytes
    system_timings_us: SystemTimingsSnapshot,  // ~200 bytes
    hardware_metrics: Option<HardwareSnapshot>, // ~150 bytes
    parallelization_metrics: Option<...>,       // ~100 bytes
}

// Total heap size: ~551 KB
```

### CreatureSnapshot (AoS Layout)

```
Array of Structs (inefficient for cache):
[
  CreatureSnapshot { id: 0, x: 10.0, y: 20.0, rotation: 1.5, size: 1.0 },
  CreatureSnapshot { id: 1, x: 15.0, y: 25.0, rotation: 2.1, size: 1.2 },
  ...
  CreatureSnapshot { id: 27499, x: ..., y: ..., rotation: ..., size: ... },
]
```

**Memory access pattern:**
- PixiJS updates: X position of creature 0, then X position of creature 1, ...
- AoS requires: Load entire struct, extract X, discard rest, repeat
- **Cache thrashing:** Only 1/5 of loaded cache line is used

---

## Limitations of Current Architecture

### 1. MessagePack Serialization Overhead

**Problem:** CPU-intensive traversal and encoding of entire GameState tree

**Evidence:**
- 810 μs for 27.5K creatures
- Linear scaling: O(n) where n = creature count
- Projected 4.4 ms for 150K (unacceptable)

**Root Cause:** No zero-copy path (data must be copied and encoded)

---

### 2. Writer Thread Contention

**Problem:** Background thread becomes bottleneck at scale

**Evidence:**
- 19.3 ms total execution time
- 100% channel utilization
- 42 avg frame drops

**Root Cause:** Single-threaded serialization can't keep up with 30Hz production rate

---

### 3. Frame Drops from Queue Saturation

**Problem:** Oldest frames discarded when queue fills

**Evidence:**
- Queue capacity: 2
- Production: 30 Hz
- Consumption: <30 Hz (when saturated)
- Result: 42 avg dropped frames

**Root Cause:** Fixed-size queue with drop-oldest policy

---

### 4. No Viewport Culling

**Problem:** All creatures sent to frontend (even off-screen)

**Evidence:**
- 27,500 creatures in buffer
- No filtering based on camera view
- Frontend wastes CPU rendering invisible sprites

**Mitigation (future):** Send camera bounds to Rust, filter before serialization

**Note:** Post-NAPI, this becomes less critical (buffer read is cheap)

---

## Post-NAPI Migration Changes

### What Gets Eliminated

1. ✅ **stdin reader thread** → Direct NAPI function calls
2. ✅ **stdout writer thread** → Eliminated
3. ✅ **MessagePack serialization** → Zero-copy shared memory
4. ✅ **Snapshot queue (ArrayQueue)** → Double buffer (atomic swap)
5. ✅ **mpsc channel** → Direct function calls

### What Gets Added

1. **DoubleBuffer:** Lock-free atomic pointer swap
2. **NAPI FFI layer:** Rust ↔ JavaScript bindings
3. **ThreadsafeFunction:** For telemetry callbacks
4. **SoA buffer layout:** Better cache locality

### Architecture After Migration

```
┌─────────────────────────────────────────────────────────────────┐
│                   ELECTRON APPLICATION                           │
├──────────────────────────┬──────────────────────────────────────┤
│  JavaScript (Renderer)   │  Rust (NAPI Native Addon)            │
│                          │                                       │
│  ┌────────────────┐      │  ┌──────────────────┐                │
│  │ spawnCreatures │──────┼─>│ NAPI function    │                │
│  │ (100)          │      │  │ call (direct)    │                │
│  └────────────────┘      │  └───────┬──────────┘                │
│                          │          │                            │
│                          │          v                            │
│                          │  ┌──────────────────┐                │
│                          │  │ Bevy World       │                │
│                          │  │ (spawn entities) │                │
│                          │  └──────────────────┘                │
│                          │                                       │
│  ┌────────────────┐      │  ┌──────────────────┐                │
│  │ getBuffer()    │──────┼─>│ DoubleBuffer     │                │
│  │ → Float32Array │<─────┼──│ (atomic read)    │                │
│  └────────────────┘      │  └──────────────────┘                │
│         │                │          ^                            │
│         v                │          │                            │
│  ┌────────────────┐      │  ┌──────────────────┐                │
│  │ PixiJS ticker  │      │  │ ECS Update       │                │
│  │ (90 Hz)        │      │  │ (30 Hz)          │                │
│  │ Update sprites │      │  │ Atomic swap      │                │
│  └────────────────┘      │  └──────────────────┘                │
│                          │                                       │
└──────────────────────────┴──────────────────────────────────────┘
```

**Key Difference:** Direct memory access (no serialization, no IPC)

---

## Conclusion

**Current stdio IPC architecture has THREE critical bottlenecks:**

1. **MessagePack serialization:** 810 μs (57% of ECS time)
2. **Writer thread blocking:** 19.3 ms total
3. **Frame drops:** 42 avg from queue saturation

**Total overhead:** ~20.2 ms/frame (60% of 33.3ms frame budget at 30Hz)

**Post-NAPI migration will eliminate:**
- 99% of serialization overhead (<10 μs vs 810 μs)
- 100% of writer thread overhead (no background thread)
- 100% of frame drops (no queue)

**Net result:** ~19.8 ms/frame freed for simulation → enables 150K-200K creatures

**Confidence:** High (validated by Phase 0.6 dry-run benchmark showing 350 μs buffer read for 27.5K)

---

**End of Documentation**
