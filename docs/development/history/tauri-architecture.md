# Tauri Architecture: Desktop Hybrid

**Last Updated:** 2025-11-10
**Status:** Active Development (Sprint 7+)
**Platform:** Windows, Mac, Linux (Steam Early Access)

---

## Executive Summary

**Speciate** uses Tauri to bundle a Rust/Bevy simulation backend with a PixiJS rendering frontend into a single desktop application. This eliminates network complexity, server costs, and enables a "pull" model where the renderer directly queries simulation state.

**Key Components:**
- **Brain (Rust/Bevy ECS):** Runs AI, physics, and state logic at 20 Hz (FixedUpdate) + 90 Hz (Update)
- **Renderer (PixiJS):** Draws sprites at 90 FPS, pulling state via Tauri IPC
- **Wrapper (Tauri):** Bundles into `.exe`/`.app`, provides IPC bridge

---

## Architecture Overview

### Data Flow: "Pull" Model

```
┌─────────────────────────────────────────────────────────────┐
│                       TAURI APPLICATION                      │
├───────────────────────────────┬─────────────────────────────┤
│  RUST BACKEND (Bevy ECS)      │  FRONTEND (PixiJS)          │
│                               │                             │
│  FixedUpdate (20 Hz)          │  app.ticker (90 FPS)        │
│  ├─ AI Systems                │  ├─ invoke('get_game_state')│
│  ├─ Decision Making           │  ├─ Update sprite positions │
│  └─ Steering Behaviors        │  └─ Render frame            │
│                               │                             │
│  Update (90 Hz)               │  Total Budget: 11ms/frame   │
│  ├─ Physics Integration       │                             │
│  ├─ Position += Velocity*dt   │                             │
│  └─ Write to SnapshotQueue────┼──> Lock-Free Read          │
│                               │                             │
└───────────────────────────────┴─────────────────────────────┘
```

**No network. No interpolation. Direct memory access via IPC.**

---

## Dual-Tick System

### Why Two Update Schedules?

**Problem:** AI is expensive (pathfinding, decision trees), rendering must be smooth (90 FPS).

**Solution:** Split into two schedules with different tick rates.

### FixedUpdate Schedule (20 Hz = 50ms)

**Purpose:** Deterministic simulation of expensive "brain" logic

**Systems:**
- `creature_brain_system` - Decision making, target selection
- `seek_system` - Steering toward targets
- `flee_system` - Escape from threats
- `wander_system` - Territory-based movement
- `avoid_obstacles_system` - Collision avoidance

**Why fixed timestep?**
- Deterministic simulation (same inputs = same outputs)
- AI doesn't need 90 Hz precision (humans react at ~200ms)
- Budget: 40-45ms per tick (generous for complex logic)

```rust
// Example: AI system runs at 20 Hz
fn creature_brain_system(
    mut creatures: Query<(&mut Creature, &Transform, &Velocity)>,
    time: Res<Time>,
) {
    // Delta time is ALWAYS 0.05 seconds (20 Hz)
    // Can safely accumulate state, plan ahead
}
```

### Update Schedule (90 Hz = 11ms)

**Purpose:** Smooth rendering and cheap physics integration

**Systems:**
- `physics_system` - Position += velocity * dt
- `boundary_system` - Wrap/clamp to world edges
- `snapshot_system` - Write state to lock-free queue for Tauri

**Why variable timestep?**
- Rendering must be fluid (no 20 Hz jitter)
- Physics integration is cheap (simple math)
- Interpolates between AI decisions smoothly

**Budget: 8-9ms per tick** (tight, but achievable for 1000 creatures)

```rust
// Example: Physics runs at 90 Hz
fn physics_system(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds(); // Variable (e.g., 0.011s at 90 FPS)

    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0 * dt; // Smooth movement
    }
}
```

---

## Lock-Free Snapshot System

### The Critical Performance Problem

**WRONG APPROACH** (Will cause stuttering):

```rust
// ❌ BAD: Tauri command locks World, blocks Bevy
#[tauri::command]
fn get_game_state(world: State<Mutex<World>>) -> GameState {
    let world = world.lock().unwrap(); // BLOCKS BEVY MID-FRAME
    serialize_world(&world) // Takes 2-3ms while Bevy waits
}
```

**Problem:** If PixiJS calls `invoke('get_game_state')` at 90 FPS, and each call locks the World for 2-3ms, Bevy's simulation stutters.

### CORRECT APPROACH: Lock-Free Ring Buffer

**Architecture:**

```rust
use crossbeam::queue::ArrayQueue;

// Shared state (accessible to both Bevy and Tauri)
pub struct SnapshotQueue {
    queue: ArrayQueue<GameState>,
}

// Bevy: Write snapshots (never waits)
fn snapshot_system(
    query: Query<(&Transform, &Creature, &Velocity)>,
    queue: Res<SnapshotQueue>,
) {
    let snapshot = GameState {
        creatures: query.iter()
            .map(|(t, c, v)| CreatureState {
                id: c.id,
                x: t.translation.x,
                y: t.translation.y,
                vx: v.0.x,
                vy: v.0.y,
                behavior: c.behavior_state.clone(),
            })
            .collect(),
    };

    let _ = queue.queue.push(snapshot); // If full, drops newest (fine)
}

// Tauri: Read latest snapshot (lock-free)
#[tauri::command]
fn get_game_state(queue: State<SnapshotQueue>) -> Option<GameState> {
    queue.queue.pop() // Lock-free, instant
}
```

**Benefits:**
- **Bevy never blocks:** Writes happen during `Update`, take ~1-2ms
- **Tauri never blocks:** Reads are lock-free (< 0.5ms)
- **Total overhead:** ~2.5ms per frame (acceptable within 11ms budget)

### ~~Alternative: Double-Buffered State~~ (NOT USED)

> **DEPRECATED:** This approach was considered but NOT implemented. RwLock is not truly lock-free and can cause brief blocking. See the ArrayQueue implementation above for the actual pattern used in this project.

**Simpler implementation, slightly higher latency (NOT RECOMMENDED):**

```rust
use std::sync::RwLock;

pub struct GameStateBuffer {
    front: RwLock<GameState>, // Tauri reads from here
    back: RwLock<GameState>,  // Bevy writes here
}

// Bevy: Swap buffers at end of frame
fn snapshot_system(
    query: Query<(&Transform, &Creature)>,
    buffer: Res<GameStateBuffer>,
) {
    let mut back = buffer.back.write().unwrap();
    *back = serialize_query(&query);

    // Atomic swap
    std::mem::swap(
        &mut *buffer.front.write().unwrap(),
        &mut *back
    );
}

// Tauri: Always reads from front
#[tauri::command]
fn get_game_state(buffer: State<GameStateBuffer>) -> GameState {
    buffer.front.read().unwrap().clone()
}
```

**Trade-offs:**
- **Simpler:** No crossbeam dependency, easier to debug
- **Latency:** Up to 1 frame delay (front buffer may be stale)
- **Blocking:** `write().unwrap()` can briefly block if Tauri reads mid-swap

**Recommendation:** Use **lock-free ring buffer** for guaranteed non-blocking.

---

## Data Serialization Strategy

### Full f32 Coordinates (No Quantization)

**MMO version:** Quantized to i16 with 0.1 precision (network bandwidth limit)
**Tauri version:** Full f32 precision (bandwidth irrelevant, IPC is in-process)

**Benefits:**
- No quantization artifacts
- Simpler code (no encode/decode)
- Higher precision rendering

### Serialization Format

**Option 1: JSON (Simple, Debuggable)**

```rust
#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub creatures: Vec<CreatureState>,
    pub timestamp: f64,
}
```

**Pros:** Easy to debug (inspect in browser DevTools)
**Cons:** Larger payload (~50-100 KB for 1000 creatures)

**Option 2: MessagePack (Efficient, Still Debuggable)**

```rust
use rmp_serde::{Serializer, Deserializer};

let snapshot = rmp_serde::to_vec(&game_state).unwrap();
```

**Pros:** 30-40% smaller than JSON, still human-readable with tools
**Cons:** Requires dependency

**Recommendation:** Start with **JSON** (simplicity), switch to **MessagePack** if serialization time > 2ms.

---

## Performance Targets & Budgets

### 90 FPS Rendering (11ms budget per frame)

| Component | Budget | Actual (Target) |
|-----------|--------|-----------------|
| **Bevy Update (physics)** | 8ms | 6-7ms (1000 creatures) |
| **Snapshot serialization** | 2ms | 1.5-2ms (JSON) |
| **Tauri IPC overhead** | 0.5ms | 0.3-0.5ms |
| **PixiJS rendering** | Remaining (~8ms) | 5-6ms (1000 sprites) |

**Total:** ~13-15ms actual (allows 75-80 FPS worst case)

### 20 Hz Simulation (50ms budget per frame)

| Component | Budget | Actual (Target) |
|-----------|--------|-----------------|
| **AI systems (FixedUpdate)** | 40-45ms | 13-14ms (current) |
| **Remaining for future systems** | 5-10ms | 26-37ms headroom |

**Current:** Massive headroom (13ms used of 50ms budget)

---

## Tauri Command Interface

### Core Commands

```rust
// Get latest simulation state
#[tauri::command]
fn get_game_state(queue: State<SnapshotQueue>) -> Option<GameState> {
    queue.queue.pop()
}

// Spawn creature (player action)
#[tauri::command]
fn spawn_creature(
    x: f32,
    y: f32,
    dna: Option<String>,
    world: State<Mutex<CommandQueue>>,
) -> Result<u32, String> {
    // Add spawn command to queue (Bevy processes next tick)
    world.lock().unwrap().push(Command::Spawn { x, y, dna });
    Ok(next_id())
}

// Save/load game
#[tauri::command]
fn save_game(path: String, world: State<Mutex<World>>) -> Result<(), String> {
    // Serialize World to disk
}

#[tauri::command]
fn load_game(path: String, world: State<Mutex<World>>) -> Result<(), String> {
    // Deserialize World from disk
}
```

### Frontend Integration (PixiJS)

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// In app.ticker (90 FPS loop)
app.ticker.add(async () => {
    // Pull latest state
    const state = await invoke('get_game_state');

    if (state) {
        // Update sprite positions directly (no interpolation)
        state.creatures.forEach(creature => {
            const sprite = spriteMap.get(creature.id);
            if (sprite) {
                sprite.x = creature.x;
                sprite.y = creature.y;
                sprite.rotation = Math.atan2(creature.vy, creature.vx);
            }
        });
    }
});

// Player spawns creature (click event)
function onCanvasClick(event) {
    const worldPos = screenToWorld(event.x, event.y);
    invoke('spawn_creature', { x: worldPos.x, y: worldPos.y });
}
```

---

## Refactoring Plan (Sprint 7)

### Phase 1: Remove Network Code

**Delete:**
- `apps/broadcaster/` (entire Node.js service)
- `simulation/crates/nats_client/`
- `apps/portal/src/services/WebSocketClient.ts`
- Interpolation logic in PixiJS (`lerp()`, `old_state`/`new_state`)
- Quantization/delta encoding

**Keep (for now):**
- Economy Ledger API (if DNA trading still planned)
- PostgreSQL schema (creature lineage tracking)

### Phase 2: Implement Tauri IPC

**Add:**
- `simulation/crates/tauri_bridge/` (new crate)
- `SnapshotQueue` struct with `crossbeam::ArrayQueue`
- `snapshot_system` in Bevy `Update` schedule
- Tauri commands (`get_game_state`, `spawn_creature`, etc.)

**Update:**
- Move AI systems to `FixedUpdate` schedule (20 Hz)
- Keep physics in `Update` schedule (90 Hz)
- Remove all NATS publishing logic

### Phase 3: Frontend Simplification

**PixiJS changes:**
- Remove interpolation (`lerp`, `smoothDamp`)
- Direct sprite updates from Tauri state
- Remove WebSocket connection logic
- Add Tauri `invoke()` calls in ticker loop

### Phase 4: Packaging & Testing

**Tauri setup:**
- Create `src-tauri/` directory
- Configure `tauri.conf.json` (window size, permissions, etc.)
- Build scripts for Windows, Mac, Linux
- Test cross-platform builds

**Performance validation:**
- Profile `get_game_state` latency (must be <2ms p99)
- Verify 90 FPS stability with 1000 creatures
- Check memory usage (no leaks from snapshot queue)

---

## Architecture Decision Records

### ADR-001: Lock-Free Ring Buffer

**Decision:** Use `crossbeam::ArrayQueue` instead of `Mutex<World>`

**Rationale:**
- Prevents Bevy blocking on Tauri commands
- Guarantees <0.5ms read latency (lock-free)
- Acceptable trade-off: May drop frames if queue full (unlikely at 90 FPS with 20 Hz writes)

**Alternatives considered:**
- Double-buffered `RwLock`: Simpler but can block briefly
- Tauri events (push model): Decouples but adds complexity

**Status:** Approved by architect-andy (2025-11-10)

---

### ADR-002: Dual-Tick Schedule

**Decision:** AI at 20 Hz (FixedUpdate), Physics at 90 Hz (Update)

**Rationale:**
- Standard practice in game engines (Unity, Unreal)
- AI doesn't need 90 Hz precision (determinism more important)
- Physics must match rendering for smooth visuals

**Alternatives considered:**
- Single tick at 20 Hz: Jittery rendering
- Single tick at 90 Hz: Wastes CPU on AI recalculations

**Status:** Approved (aligns with game engine best practices)

---

### ADR-003: Archive MMO Code

**Decision:** Move NATS/Broadcaster to `archive/mmo-streaming-v1` branch

**Rationale:**
- Git preserves history (can resurrect if needed)
- Cleaner main branch (no dead code confusion)
- Documented in `docs/architecture/archived/MMO_STREAMING.md`

**Timeline:** After Tauri validated (Sprint 7 end)

**Status:** Pending execution

---

## Telemetry & Monitoring

### Performance Metrics to Track

```rust
// Add to snapshot_system
fn snapshot_system(
    query: Query<(&Transform, &Creature)>,
    queue: Res<SnapshotQueue>,
) {
    let start = Instant::now();

    let snapshot = serialize_query(&query);
    let serialize_time = start.elapsed();

    let _ = queue.queue.push(snapshot);
    let total_time = start.elapsed();

    if serialize_time.as_millis() > 2 {
        warn!("SLOW serialize: {:?}", serialize_time);
    }

    // Expose via metrics endpoint for profiling
    metrics::histogram!("snapshot.serialize_ms", serialize_time.as_millis() as f64);
}
```

### Key Metrics

- **Snapshot serialization time:** p50, p95, p99 (target: <2ms p99)
- **Tauri command latency:** p50, p95, p99 (target: <1ms p99)
- **Frame time:** Bevy Update, FixedUpdate (target: <11ms, <50ms)
- **Queue depth:** SnapshotQueue size (should stay near 0-2)

---

## Risk Mitigation

### What if 90 FPS target fails?

**Fallback: Tauri Events (Push Model)**

```rust
// Bevy: Emit events instead of queue
fn snapshot_system(
    query: Query<(&Transform, &Creature)>,
    app: Res<AppHandle>,
) {
    let snapshot = serialize_query(&query);
    app.emit_all("game_state", snapshot).ok();
}

// PixiJS: Listen for events
import { listen } from '@tauri-apps/api/event';

listen('game_state', (event) => {
    updateSprites(event.payload);
});
```

**Benefits:**
- Decouples frontend FPS from pull rate
- Bevy controls data flow (backpressure)
- Still simpler than NATS (no external broker)

**When to use:** If `invoke()` overhead exceeds 2ms or causes frame drops.

---

## Success Criteria

### Sprint 7 Goals

- [ ] 1000 creatures @ 90 FPS frontend, 20 Hz simulation
- [ ] `get_game_state` p99 latency <3ms
- [ ] No frame drops during heavy AI computation
- [ ] Tauri `.exe` boots in <2 seconds
- [ ] Cross-platform builds tested (Windows, Mac, Linux)

### Performance Validation Checklist

- [ ] Profile with `cargo flamegraph` (identify hotspots)
- [ ] Measure snapshot serialization (JSON vs. MessagePack)
- [ ] Test with 5000 creatures (stress test)
- [ ] Monitor memory usage over 60 minutes (no leaks)
- [ ] Verify save/load reliability (no crashes)

---

## Next Steps

1. **Create `simulation/crates/tauri_bridge/`** (new Rust crate)
2. **Implement `SnapshotQueue`** with `ArrayQueue<GameState>`
3. **Add `snapshot_system`** to Bevy `Update` schedule
4. **Remove interpolation** from PixiJS frontend
5. **Test with 1000 creatures** (validate <3ms snapshot time)
6. **Archive NATS code** (only after validation succeeds)

**Estimated Timeline:** Sprint 7 (5-7 days)

---

## References

- [Bevy FixedUpdate Documentation](https://bevyengine.org/learn/book/getting-started/ecs/)
- [Tauri IPC Guide](https://tauri.app/v1/guides/features/command)
- [Crossbeam Queue Documentation](https://docs.rs/crossbeam/latest/crossbeam/queue/struct.ArrayQueue.html)
- [Game Loop Design Patterns](https://gameprogrammingpatterns.com/game-loop.html)

---

**Status:** Ready for implementation (Sprint 7)
**Owner:** backend-simulation-sam (Rust), frontend-fanny (PixiJS)
**Reviewer:** architect-andy
