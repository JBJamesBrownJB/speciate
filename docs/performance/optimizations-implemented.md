# Implemented Optimizations Log

---

## 2025-11-17: Binary IPC & PixiJS ParticleContainer Refactor

**Problem:** 7 FPS rendering with 10k creatures despite IPC optimizations. Electron IPC was re-serializing decoded MessagePack objects, and main.ts was a 396-line god object.

**Solutions Implemented:**

### 1. Binary IPC Zero-Copy Transfer
- **Issue:** Main process decoded MessagePack → object, then Electron IPC re-serialized it
- **Fix:** Pass raw `Uint8Array` directly to renderer (zero-copy structured clone)
- **Impact:** Eliminated double serialization overhead

### 2. ParticleContainer Implementation
- **Issue:** Regular Container with 10k Sprites (not batch-rendered)
- **Fix:** Switched to PixiJS ParticleContainer with WebGL batching
- **Configuration:** Position, rotation, scale dynamic properties enabled
- **Bounds:** Set static world bounds (`Rectangle(-1M, -1M, 2M, 2M)`) to skip O(n) bounds calculation
- **Impact:** 10x rendering performance improvement

### 3. Stale Entity Cleanup
- **Issue:** Creatures removed from simulation remained visible on screen
- **Fix:** Added `ParticlePool.getStaleEntities()` and `removeEntity()` methods
- **Pattern:** Track current IDs per frame, remove particles not in current set
- **Impact:** Visual correctness, no zombie sprites

### 4. Code Refactoring (Clean Architecture)
- **Issue:** main.ts was 396 lines doing everything (rendering, UI, IPC, camera)
- **Extracted Classes:**
  - `FPSSparkline` (76 lines) - FPS graph rendering
  - `ScaleBarManager` (51 lines) - Scale bar logic with nice intervals
  - `HUDManager` (54 lines) - Centralized HUD updates
  - `CreatureRenderer` (52 lines) - Particle lifecycle management
- **Constants Extracted:**
  - `CAMERA_CONFIG.ZOOM_SENSITIVITY`
  - `SCALE_BAR_CONFIG.TARGET_PIXEL_WIDTH` & `NICE_INTERVALS`
- **Result:** main.ts reduced to 236 lines (40% reduction)
- **Impact:** Improved maintainability, testability, single responsibility

### 5. Debug Code Removal
- **Removed:** All console.logs (frame count, stale cleanup, timing measurements)
- **Kept:** Only `console.error()` for genuine errors (per code standards)
- **Impact:** Cleaner production code, no console spam

**Performance Metrics:**
- **Before:** 7 FPS with 10k creatures
- **After:** 60 FPS with 10k creatures
- **Improvement:** ~850% FPS increase

**Files Changed:**
- `apps/portal/electron/main.cjs` - Binary IPC
- `apps/portal/src/main.ts` - Refactored from 396→236 lines
- `apps/portal/src/ui/FPSSparkline.ts` - NEW
- `apps/portal/src/ui/ScaleBarManager.ts` - NEW
- `apps/portal/src/ui/HUDManager.ts` - NEW
- `apps/portal/src/rendering/CreatureRenderer.ts` - NEW
- `apps/portal/src/core/constants.ts` - Added SCALE_BAR_CONFIG, CAMERA_CONFIG.ZOOM_SENSITIVITY
- `apps/portal/src/infrastructure/ParticlePool.ts` - Added cleanup methods
- `apps/dev-ui/src/components/DevToolsApp.tsx` - Binary IPC support
- `apps/dev-ui/package.json` - Switched to @msgpack/msgpack

---

## 2025-11-16: Skip Catatonic Crits in Perception

**Problem:** Perception system computed neighbors for ALL crits, including inactive ones.

**Solution:** Added `BehaviorMode::is_active()` check to skip catatonic crits in AI systems.

**Notes:** Pattern reusable across all AI systems. No archetype thrashing (enum vs marker).

---
