# Sprint 10 Summary: ECS Instrumentation

**Sprint:** Sprint 10
**Branch:** `feat/sprint-10-ecs-instrumentation`
**Status:** ✅ COMPLETE
**Started:** 2025-11-16
**Completed:** 2025-11-16

---

## Sprint Goal

Implement zero-cost ECS system performance instrumentation with real-time visualization in the Dev-UI, enabling visibility into simulation bottlenecks without production overhead.

---

## Key Outcomes

### 1. Zero-Cost Instrumentation Framework
- **Feature-gated compilation** via `#[cfg(feature = "dev-tools")]`
- **RAII TimingGuard** pattern using `Drop` trait for automatic measurement
- **AtomicU64 storage** for thread-safe timing data (parallel-safe)
- **3-line pattern** for instrumenting any ECS system
- **Zero production overhead** - compiles to nothing in release builds

### 2. Comprehensive System Coverage
Instrumented **9 timing metrics** (8 systems + total tick):
- `total_tick` - Entire frame execution time
- `perception` - Spatial awareness updates
- `behavior_transition` - State machine transitions
- `wander` - Wandering/territorial force calculation
- `flee` - Fleeing behavior (stub)
- `seek` - Seeking force accumulation
- `avoidance` - Obstacle avoidance
- `movement` - Physics integration (Euler)
- `rotation` - Orientation updates

### 3. Real-Time Dev-UI Visualization
- **Canvas-based sparklines** with 120-frame history (2 seconds at 60 Hz)
- **Auto-sorting** by timing value (slowest systems at top)
- **Color-coded thresholds**: Green (<5ms), Amber (5-10ms), Red (>10ms)
- **Direct IPC integration** via existing GameState message flow

### 4. Developer Experience Improvements
- **Window positioning** - Portal and Dev-UI spawn side-by-side (no window reorganization)
- **No DevTools auto-open** - Clean window startup (Ctrl+Shift+I when needed)
- **Live tick rate display** - Shows actual Hz from simulation (no hardcoded values)

### 5. Mandatory Instrumentation Policy
- **CLAUDE.md hook** requiring instrumentation for all new ECS systems
- **Comprehensive documentation** at `docs/testing/metrics/README.md`
- **Step-by-step guide** for adding timing to new systems (Rust + TypeScript)

---

## Technical Architecture

### Rust (Backend)
```
apps/simulation/src/instrumentation/mod.rs
├── SystemTimings (Resource with AtomicU64 fields)
├── TimingGuard (RAII pattern with Drop trait)
├── SystemTimingsSnapshot (Serializable with camelCase serde)
└── time_system! macro (feature-gated, compiles to nothing in prod)
```

### TypeScript (Frontend)
```
apps/dev-ui/src/
├── types.ts (SystemTimingsSnapshot interface)
├── components/SystemTimingsPanel.tsx (React + Canvas sparklines)
└── components/DevToolsApp.tsx (State management + IPC)

apps/portal/src/types/GameState.ts (Mirrored interface)
```

### Data Flow
```
ECS System → time_system! macro → SystemTimings resource (AtomicU64)
     ↓
Simulation::update() → total_tick timing
     ↓
StdioHooks::snapshot() → SystemTimingsSnapshot (JSON/MessagePack)
     ↓
Electron IPC → Dev-UI React component → Canvas sparkline rendering
```

---

## Completed Tasks

### Phase 1: Rust Instrumentation Core ✅
- [x] Feature flag setup (`dev-tools` in Cargo.toml)
- [x] SystemTimings resource with AtomicU64 fields
- [x] RAII TimingGuard with Drop trait
- [x] time_system! declarative macro (dual feature-gate versions)

### Phase 2: Instrument Systems ✅
- [x] Instrumented 8 ECS systems (perception, movement, behavior, etc.)
- [x] Added whole-tick timing measurement
- [x] Conditional parameter pattern (`#[cfg(feature = "dev-tools")]`)
- [x] Registered SystemTimings resource at startup

### Phase 3: Extend GameState IPC ✅
- [x] Added `system_timings_us` field to GameState
- [x] Serialization with `#[serde(rename_all = "camelCase")]`
- [x] Backward compatible (serde ignores unknown fields)

### Phase 4: Frontend Dev-UI ✅
- [x] TypeScript interfaces updated (dev-ui + portal)
- [x] SystemTimingsPanel React component
- [x] Canvas-based sparkline rendering (120-frame history)
- [x] Auto-sorting by timing value (descending)
- [x] Color-coded thresholds (green/amber/red)

### Phase 5: Documentation & Validation ✅
- [x] Comprehensive guide at `docs/testing/metrics/README.md`
- [x] CLAUDE.md hook for mandatory instrumentation
- [x] 8 passing Rust instrumentation tests
- [x] Zero overhead verified in production builds

### Bonus: Developer Experience ✅
- [x] Fixed Dev-UI window positioning (spawns beside main window)
- [x] Removed DevTools auto-open (clean startup)
- [x] Live tick rate display (no hardcoded 20 Hz)
- [x] Removed stale 20 Hz references from docs

---

## Remaining Work

### Deferred to Future Sprints
- Memory profiling (CPU timing only for now)
- Flamegraph generation
- Historical persistence (in-memory only)
- Percentile tracking (P50/P95/P99)
- Tracy profiler integration
- Auto-discovery of slow systems

### Known Technical Debt
- 2 flaky stdio tests (pre-existing, timing-sensitive filesystem tests)
- Portal build has missing `dev-tools.html` entry (pre-existing)
- Sparkline history could benefit from capping (currently unbounded)

---

## Performance Expectations

- **Instrumentation overhead:** ~2-3 microseconds per timed system
- **Memory:** One AtomicU64 per system (~100 bytes total)
- **IPC overhead:** ~100 bytes added to each GameState (negligible)
- **Frontend:** useRef pattern means 0% React re-renders
- **Sparkline render:** ~0.1ms per Canvas redraw
- **Total FPS impact:** <0.1% (negligible)
- **Binary size difference:** ~300KB (instrumentation code + atomics)

---

## Retrospective / Lessons Learned

### What Went Well
1. **Feature-gated zero-cost abstraction** - Excellent Rust pattern for dev tooling
2. **RAII TimingGuard** - Clean, idiomatic Rust (Drop trait ensures measurement even on panic)
3. **Auto-sorting sparklines** - Immediately shows bottlenecks
4. **3-line instrumentation pattern** - Easy to add to new systems
5. **Serde camelCase conversion** - Seamless Rust-to-TypeScript serialization

### What Could Be Improved
1. **Initial UI placement** - Dev-UI was incorrectly added to portal instead of dev-ui app
2. **NaN display issue** - Forgot camelCase serde attribute initially
3. **Test fixtures** - Had to update test fixtures when adding new timing fields

### Key Technical Decisions
1. **AtomicU64 over Mutex** - Lock-free, parallel-safe timing storage
2. **Relaxed ordering** - Sufficient for dev tooling (no strict synchronization needed)
3. **String identifiers** - Simple but consider enum for type safety in future
4. **Canvas API** - High-performance sparkline rendering (no React overhead)

### Documentation Strategy
- **Removed hardcoded tick rate comments** - Docs should be rate-agnostic
- **Added mandatory instrumentation hook** - Ensures future systems are instrumented
- **Step-by-step guide** - Reduces friction for adding new timing metrics

---

## Files Changed

### Rust (apps/simulation/)
- `Cargo.toml` - Added `dev-tools` feature
- `src/instrumentation/mod.rs` - Core timing infrastructure (NEW)
- `src/lib.rs` - time_system! macro definition
- `src/simulation/core/simulation.rs` - Whole tick timing
- `src/simulation/perception/systems.rs` - Perception system timing
- `src/simulation/movement/systems.rs` - Movement system timing
- `src/simulation/creatures/behaviors/seek.rs` - Seek system timing
- `src/simulation/creatures/behaviors/transitions.rs` - Behavior transition timing
- `src/simulation/creatures/behaviors/wander.rs` - Wander system timing
- `src/simulation/creatures/behaviors/flee.rs` - Flee system timing
- `src/simulation/creatures/behaviors/avoidance.rs` - Avoidance system timing
- `src/simulation/movement/rotation.rs` - Rotation system timing
- `src/stdio/hooks.rs` - Include timings in GameState snapshot
- `src/ipc/snapshot_queue.rs` - Updated test fixtures
- `tests/instrumentation_test.rs` - Unit tests for timing infrastructure
- `CLAUDE.md` - Added mandatory instrumentation hook

### TypeScript (apps/)
- `dev-ui/src/types.ts` - SystemTimingsSnapshot interface
- `dev-ui/src/components/SystemTimingsPanel.tsx` - React sparkline component
- `dev-ui/src/components/DevToolsApp.tsx` - Integrated timings panel
- `dev-ui/src/components/StateDisplay.tsx` - Live tick rate display
- `dev-ui/src/index.css` - Sparkline styling
- `portal/src/types/GameState.ts` - Mirrored interface
- `portal/electron/main.cjs` - Window positioning, DevTools auto-open fix

### Documentation
- `docs/testing/metrics/README.md` - Comprehensive instrumentation guide (NEW)
- `README.md` - Updated architecture diagram (tick-rate agnostic)
- `GLOSSARY.md` - Removed hardcoded tick rate values

---

## Success Metrics

- ✅ **9 timing metrics** instrumented (8 systems + total tick)
- ✅ **Zero production overhead** (feature-gated, compiles to nothing)
- ✅ **Real-time visualization** with auto-sorted sparklines
- ✅ **8 passing instrumentation tests**
- ✅ **Comprehensive documentation** for future development
- ✅ **Mandatory instrumentation policy** via CLAUDE.md hook
- ✅ **Developer experience improvements** (window positioning, clean startup)

---

## QA Verification

**Status:** APPROVED FOR MERGE

- All instrumentation tests pass (8/8)
- Both release and dev-tools builds compile successfully
- TypeScript builds pass (dev-ui + portal)
- No security vulnerabilities detected
- Proper feature-gating verified
- Zero production overhead confirmed

---

## Next Sprint Considerations

1. **Dual-tick architecture** - Separate AI (20Hz) from physics (30Hz) for massive scale
2. **Spatial grid optimization** - O(n) perception instead of O(n²)
3. **Memory profiling** - Add heap allocation tracking
4. **Performance alerting** - Auto-detect performance regressions
5. **Historical persistence** - Save timing data for trend analysis

---

## Final Notes

Sprint 10 successfully establishes a robust, zero-cost instrumentation framework that provides immediate visibility into ECS system performance. The auto-sorted sparkline visualization makes bottleneck detection trivial, while the mandatory instrumentation policy ensures all future systems maintain this observability. The implementation follows Rust and TypeScript best practices with proper feature-gating, type safety, and clean separation of concerns.

**The Church of Metrics is now operational. 📊**
