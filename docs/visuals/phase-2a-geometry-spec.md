# Phase 2A: Custom PixiJS Geometry Setup - Technical Specification

**Sprint:** 14 (Frontend GPU Interpolation)
**Phase:** 2A
**Owners:** shader-sarah (lead), frontend-fanny (PixiJS integration)
**Status:** IN PROGRESS
**Updated:** 2025-11-25

---

## Overview

Phase 2A establishes the foundation for GPU-accelerated creature rendering by replacing PixiJS's high-level ParticleContainer API with custom Geometry and vertex buffers. This enables Phase 2B's vertex shader interpolation for smooth 60 FPS rendering from 22.2Hz simulation data.

**Goal:** Create custom PixiJS Geometry with interleaved vertex buffers that store both START and END creature positions/rotations for GPU interpolation.

---

## Current Architecture (to be replaced)

### Rendering Pipeline
```
Rust Simulation (22.2Hz)
    ↓ NAPI zero-copy buffer (SoA layout)
JavaScript StateManager
    ↓ Parse to CreatureData[]
CreatureRenderer (ParticleContainer)
    ↓ Sprite pool management
PixiJS Renderer (60 FPS, but stutters at 22.2Hz updates)
```

### Current Buffer Format (Rust → JS)

**Layout:** Structure of Arrays (SoA)
```
Float32Array = [
    ID₁, ID₂, ..., IDₙ,           // Creature IDs
    X₁, X₂, ..., Xₙ,               // X positions
    Y₁, Y₂, ..., Yₙ,               // Y positions
    Rot₁, Rot₂, ..., Rotₙ          // Rotations
]

Length: creatureCount * 4
```

**Rust Source:** `apps/simulation/src/napi_addon/simulation_engine.rs:363-365`
```rust
/// **Layout (SoA):**
/// [ID₁, ID₂, ..., IDₙ, X₁, X₂, ..., Xₙ, Y₁, Y₂, ..., Yₙ, Rot₁, Rot₂, ..., Rotₙ]
```

### Current Renderer

**File:** `apps/portal/src/rendering/CreatureRenderer.ts`

**Approach:**
- Uses PixiJS ParticleContainer (high-level API)
- ParticlePool manages sprite lifecycle
- Updates sprite.x, sprite.y, sprite.rotation every frame
- Direct property assignment (CPU-side)

**Performance:**
- Good for <50K entities
- Bottleneck at 100K+ (CPU-bound sprite updates)

---

## Target Architecture (Phase 2A)

### New Rendering Pipeline
```
Rust Simulation (22.2Hz)
    ↓ NAPI zero-copy buffer (SoA layout)
JavaScript InterpolationBufferManager
    ↓ Parse SoA → write to interleaved AoS layout
    ↓ On tick: copy END → START, write new data to END
Custom PixiJS Geometry (interleaved vertex buffer)
    ↓ GPU reads START/END per vertex
Vertex Shader (GLSL)
    ↓ Interpolate: mix(START, END, uInterpolation)
PixiJS Renderer (60 FPS smooth!)
```

### Key Changes

1. **Replace ParticleContainer** → Custom Geometry + Mesh
2. **Interleaved vertex buffer** → START and END per creature (8 floats)
3. **GPU interpolation** → Vertex shader computes final position
4. **Zero sprite updates** → All interpolation on GPU

---

## Interleaved Buffer Layout Design

### Per-Creature Vertex Data (8 floats)

```typescript
// Interleaved layout (Array of Structures - AoS)
const FLOATS_PER_CREATURE = 8;

Float32Array = [
    // Creature 0
    startX₀, startY₀, endX₀, endY₀, startRot₀, endRot₀, size₀, id₀,

    // Creature 1
    startX₁, startY₁, endX₁, endY₁, startRot₁, endRot₁, size₁, id₁,

    // ... Creature N
    startXₙ, startYₙ, endXₙ, endYₙ, startRotₙ, endRotₙ, sizeₙ, idₙ,
]

Length: creatureCount * 8
```

**Field Breakdown:**
- `startX, startY` (2 floats) - Position at previous tick (for interpolation)
- `endX, endY` (2 floats) - Position at current tick (target)
- `startRot` (1 float) - Rotation at previous tick (radians)
- `endRot` (1 float) - Rotation at current tick (radians)
- `size` (1 float) - Creature size (for scaling sprite)
- `id` (1 float) - Creature ID (for debugging, optional)

**Why Interleaved?**
- GPU fetches contiguous memory → better cache locality
- Single buffer bind (no switching between buffers)
- Standard practice for instanced rendering

### PixiJS Geometry Attributes

```typescript
import { Geometry, Buffer } from 'pixi.js';

const geometry = new Geometry();

// Interleaved vertex buffer
const vertexBuffer = new Buffer(
    interleaved Float32Array,
    false,  // static = false (we update every tick)
    false   // index = false
);

// Define attributes (stride = 8 * 4 bytes = 32 bytes per creature)
const STRIDE = 32;

geometry
    .addAttribute('aStartPos', vertexBuffer, 2, false, FLOAT, STRIDE, 0)   // Offset 0: startX, startY
    .addAttribute('aEndPos', vertexBuffer, 2, false, FLOAT, STRIDE, 8)     // Offset 8: endX, endY
    .addAttribute('aStartRot', vertexBuffer, 1, false, FLOAT, STRIDE, 16)  // Offset 16: startRot
    .addAttribute('aEndRot', vertexBuffer, 1, false, FLOAT, STRIDE, 20)    // Offset 20: endRot
    .addAttribute('aSize', vertexBuffer, 1, false, FLOAT, STRIDE, 24)      // Offset 24: size
    .addAttribute('aCreatureId', vertexBuffer, 1, false, FLOAT, STRIDE, 28); // Offset 28: id
```

**Vertex Shader Access (Phase 2B):**
```glsl
// Vertex shader will receive these per-instance
attribute vec2 aStartPos;
attribute vec2 aEndPos;
attribute float aStartRot;
attribute float aEndRot;
attribute float aSize;
attribute float aCreatureId;

uniform float uInterpolation;  // 0.0 to 1.0 (time since last tick / 45ms)

void main() {
    // Interpolate position
    vec2 worldPos = mix(aStartPos, aEndPos, uInterpolation);

    // Interpolate rotation (Phase 2B will handle angle wrapping)
    float rotation = mix(aStartRot, aEndRot, uInterpolation);

    // ... rest of vertex shader (Phase 2B)
}
```

---

## Buffer Update Strategy

### Initialization (First Tick)

```typescript
class InterpolationBufferManager {
    private buffer: Float32Array;
    private creatureCount: number = 0;

    initialize(initialCreatures: CreatureData[]) {
        this.creatureCount = initialCreatures.length;
        this.buffer = new Float32Array(this.creatureCount * 8);

        // First tick: START = END (no interpolation)
        for (let i = 0; i < initialCreatures.length; i++) {
            const c = initialCreatures[i];
            const offset = i * 8;

            // START and END are identical initially
            this.buffer[offset + 0] = c.x;         // startX
            this.buffer[offset + 1] = c.y;         // startY
            this.buffer[offset + 2] = c.x;         // endX (same)
            this.buffer[offset + 3] = c.y;         // endY (same)
            this.buffer[offset + 4] = c.rotation;  // startRot
            this.buffer[offset + 5] = c.rotation;  // endRot (same)
            this.buffer[offset + 6] = c.size;      // size
            this.buffer[offset + 7] = c.id;        // id
        }
    }
}
```

### Update on Simulation Tick (22.2Hz)

```typescript
class InterpolationBufferManager {
    update(newCreatures: CreatureData[]) {
        // 1. Handle creature count changes (spawn/despawn)
        if (newCreatures.length !== this.creatureCount) {
            this.resize(newCreatures.length);
        }

        // 2. Swap: END → START
        for (let i = 0; i < newCreatures.length; i++) {
            const offset = i * 8;

            // Copy END to START
            this.buffer[offset + 0] = this.buffer[offset + 2];  // endX → startX
            this.buffer[offset + 1] = this.buffer[offset + 3];  // endY → startY
            this.buffer[offset + 4] = this.buffer[offset + 5];  // endRot → startRot
        }

        // 3. Write new data to END
        for (let i = 0; i < newCreatures.length; i++) {
            const c = newCreatures[i];
            const offset = i * 8;

            // Write END positions
            this.buffer[offset + 2] = c.x;         // endX
            this.buffer[offset + 3] = c.y;         // endY
            this.buffer[offset + 5] = c.rotation;  // endRot
            this.buffer[offset + 6] = c.size;      // size (can change with growth)
            this.buffer[offset + 7] = c.id;        // id

            // startX, startY, startRot already set in step 2
        }

        // 4. Mark buffer as dirty (PixiJS will upload to GPU)
        this.geometry.getBuffer('vertexBuffer').update();
    }
}
```

### Render Loop (60 FPS)

```typescript
class CreatureRenderer {
    private interpolationAlpha: number = 0.0;
    private lastTickTime: number = 0;

    render(deltaMS: number) {
        // Calculate interpolation alpha (0.0 to 1.0)
        const tickInterval = 1000 / 22.2; // ~45ms
        this.interpolationAlpha += deltaMS / tickInterval;

        // Reset on simulation tick
        if (receivedNewSimulationData) {
            this.interpolationAlpha = 0.0;
            this.lastTickTime = performance.now();
        }

        // Clamp to [0, 1] (extrapolation handled in Phase 2B)
        this.interpolationAlpha = Math.max(0.0, Math.min(1.0, this.interpolationAlpha));

        // Update shader uniform
        this.shader.uniforms.uInterpolation = this.interpolationAlpha;

        // PixiJS renders (shader does interpolation on GPU)
    }
}
```

---

## Integration with NAPI Zero-Copy Buffers

### Current Data Flow

```typescript
// apps/portal/src/core/StateManager.ts (current)
const rawBuffer = this.simulation.getBuffer();  // Float32Array (SoA layout)
const creatureCount = rawBuffer.length / 4;

const creatures: CreatureData[] = [];
for (let i = 0; i < creatureCount; i++) {
    creatures.push({
        id: rawBuffer[i],
        x: rawBuffer[creatureCount + i],
        y: rawBuffer[creatureCount * 2 + i],
        rotation: rawBuffer[creatureCount * 3 + i],
        size: 10.0  // Hardcoded for now (TODO: from DNA)
    });
}
```

### New Data Flow (Phase 2A)

```typescript
// New approach: Parse SoA → write directly to interleaved buffer
class StateManager {
    private bufferManager: InterpolationBufferManager;

    private parseNAPIBuffer(rawBuffer: Float32Array): void {
        const creatureCount = rawBuffer.length / 4;

        // Allocate/resize interleaved buffer if needed
        if (this.bufferManager.creatureCount !== creatureCount) {
            this.bufferManager.resize(creatureCount);
        }

        // Swap END → START
        this.bufferManager.swapBuffers();

        // Write SoA data directly to interleaved END positions
        for (let i = 0; i < creatureCount; i++) {
            const id = rawBuffer[i];
            const x = rawBuffer[creatureCount + i];
            const y = rawBuffer[creatureCount * 2 + i];
            const rotation = rawBuffer[creatureCount * 3 + i];

            this.bufferManager.writeEnd(i, x, y, rotation, 10.0, id);
        }

        // Mark GPU buffer dirty
        this.bufferManager.markDirty();
    }
}
```

**Benefits:**
- Single allocation (interleaved buffer)
- No intermediate CreatureData[] array
- Direct write to GPU-bound buffer
- Zero-copy from Rust → minimal parsing → GPU

---

## PixiJS Implementation Details

### Geometry + Mesh Setup

```typescript
import { Geometry, Buffer, Shader, Mesh, Texture } from 'pixi.js';

class InterpolatedCreatureRenderer {
    private geometry: Geometry;
    private shader: Shader;
    private mesh: Mesh;
    private buffer: Float32Array;

    constructor(texture: Texture, maxCreatures: number) {
        // Allocate interleaved buffer
        this.buffer = new Float32Array(maxCreatures * 8);

        // Create PixiJS geometry
        this.geometry = new Geometry();

        const vertexBuffer = new Buffer(
            this.buffer,
            false,  // dynamic (updated every tick)
            false   // not an index buffer
        );

        // Add attributes (8 floats per creature, stride = 32 bytes)
        this.geometry
            .addAttribute('aStartPos', vertexBuffer, 2, false, FLOAT, 32, 0)
            .addAttribute('aEndPos', vertexBuffer, 2, false, FLOAT, 32, 8)
            .addAttribute('aStartRot', vertexBuffer, 1, false, FLOAT, 32, 16)
            .addAttribute('aEndRot', vertexBuffer, 1, false, FLOAT, 32, 20)
            .addAttribute('aSize', vertexBuffer, 1, false, FLOAT, 32, 24)
            .addAttribute('aCreatureId', vertexBuffer, 1, false, FLOAT, 32, 28);

        // Create shader (Phase 2B will implement GLSL)
        this.shader = Shader.from(
            vertexShaderSource,  // TBD in Phase 2B
            fragmentShaderSource, // TBD in Phase 2B
            {
                uInterpolation: 0.0,
                uGameTime: 0.0,
                uProjection: new Matrix(),
                uTexture: texture,
            }
        );

        // Create mesh
        this.mesh = new Mesh(this.geometry, this.shader);

        // Configure mesh for instanced rendering
        this.mesh.drawMode = DRAW_MODES.TRIANGLES;
        this.geometry.instanceCount = 0;  // Updated dynamically
    }

    update(creatureCount: number) {
        // Update instance count (PixiJS renders N instances)
        this.geometry.instanceCount = creatureCount;

        // Mark buffer dirty (upload to GPU)
        this.geometry.getBuffer('vertexBuffer').update();
    }
}
```

### Instanced Rendering Approach

**Goal:** Render N creatures with a single draw call.

**Technique:** Instanced rendering
- Each creature is an instance
- Vertex shader runs once per instance
- Base geometry: quad (2 triangles = 6 vertices)
- Instance data: START/END positions, rotations (from vertex buffer)

**Vertex Shader (simplified):**
```glsl
// Base quad vertices (same for all instances)
attribute vec2 aVertexPosition;  // (-1,-1), (1,-1), (1,1), (-1,1)
attribute vec2 aTextureCoord;    // (0,0), (1,0), (1,1), (0,1)

// Per-instance data (from interleaved buffer)
attribute vec2 aStartPos;
attribute vec2 aEndPos;
attribute float aStartRot;
attribute float aEndRot;
attribute float aSize;

uniform float uInterpolation;
uniform mat3 uProjection;

void main() {
    // Interpolate world position
    vec2 worldPos = mix(aStartPos, aEndPos, uInterpolation);

    // Interpolate rotation
    float rotation = mix(aStartRot, aEndRot, uInterpolation);

    // Apply rotation to quad vertex
    mat2 rotMatrix = mat2(
        cos(rotation), -sin(rotation),
        sin(rotation), cos(rotation)
    );
    vec2 rotatedVertex = rotMatrix * (aVertexPosition * aSize);

    // Final position
    vec2 finalPos = worldPos + rotatedVertex;

    // Project to screen space
    gl_Position = vec4((uProjection * vec3(finalPos, 1.0)).xy, 0.0, 1.0);
}
```

---

## Performance Considerations

### Memory Usage

**Current (ParticleContainer):**
- Sprite pool: ~128 bytes per sprite (PixiJS overhead)
- 100K creatures = ~12.8 MB

**New (Custom Geometry):**
- Interleaved buffer: 8 floats * 4 bytes = 32 bytes per creature
- 100K creatures = ~3.2 MB
- **Savings:** 75% reduction

### CPU Overhead

**Current:**
- Update 100K sprite positions: ~10-15ms per frame
- JavaScript property assignments (sprite.x = ...)

**New:**
- Buffer swap + write: ~2-3ms per tick (22.2Hz, not 60Hz!)
- GPU interpolation: <0.2ms
- **Savings:** 80-90% reduction in CPU work

### GPU Overhead

**Current:**
- 100K draw calls (one per sprite) = slow

**New:**
- 1 draw call (instanced rendering) = fast
- Vertex shader runs 600K times per frame (100K creatures * 6 vertices)
- Modern GPUs handle this easily

### Buffer Update Strategy Trade-offs

**Option A: Swap END → START (chosen)**
- Pro: Simple, predictable
- Pro: No history tracking needed
- Con: 3 memory writes per creature per tick

**Option B: Ping-pong buffers**
- Pro: Zero copies (just swap pointers)
- Con: More complex (double memory)
- Con: Not worth complexity for 22.2Hz updates

**Decision:** Use Option A (swap). At 22.2Hz, the copy overhead is negligible (~2ms).

---

## Testing Strategy

### Unit Tests

```typescript
describe('InterpolationBufferManager', () => {
    it('should initialize with START = END', () => {
        const manager = new InterpolationBufferManager();
        manager.initialize([{ x: 100, y: 50, rotation: 0, size: 10, id: 1 }]);

        const buffer = manager.getBuffer();
        expect(buffer[0]).toBe(100);  // startX
        expect(buffer[2]).toBe(100);  // endX
    });

    it('should swap END → START on update', () => {
        const manager = new InterpolationBufferManager();
        manager.initialize([{ x: 0, y: 0, rotation: 0, size: 10, id: 1 }]);
        manager.update([{ x: 100, y: 50, rotation: 1.5, size: 10, id: 1 }]);

        const buffer = manager.getBuffer();
        expect(buffer[0]).toBe(0);    // startX (was 0)
        expect(buffer[2]).toBe(100);  // endX (now 100)
    });

    it('should handle creature count changes', () => {
        const manager = new InterpolationBufferManager();
        manager.initialize([{ x: 0, y: 0, rotation: 0, size: 10, id: 1 }]);

        // Spawn more creatures
        manager.update([
            { x: 10, y: 10, rotation: 0, size: 10, id: 1 },
            { x: 20, y: 20, rotation: 0, size: 10, id: 2 },
        ]);

        expect(manager.getBuffer().length).toBe(16);  // 2 creatures * 8 floats
    });
});
```

### Integration Tests

```typescript
describe('GPU Interpolation Integration', () => {
    it('should render at 60 FPS with 22.2Hz simulation', async () => {
        const app = new PixiApp();
        const renderer = new InterpolatedCreatureRenderer(texture, 1000);

        // Simulate 22.2Hz ticks
        const tickInterval = 1000 / 22.2;  // ~45ms
        let lastTick = 0;

        // Render at 60 FPS
        app.ticker.add((deltaMS) => {
            // Check if simulation tick received
            if (performance.now() - lastTick >= tickInterval) {
                renderer.onSimulationTick(mockCreatures);
                lastTick = performance.now();
            }

            // Render (GPU interpolates)
            renderer.render(deltaMS);
        });

        // Verify smooth rendering (no stuttering)
        // ... frame time measurements ...
    });
});
```

### Visual Quality Tests

```typescript
describe('Interpolation Quality', () => {
    it('should not show rubber-banding artifacts', () => {
        // Record positions over time
        // Verify smooth curve (no sudden jumps)
    });

    it('should handle rotation wraparound correctly', () => {
        // 350° → 10° should interpolate as 20° CW, not 340° CCW
        // This will be tested in Phase 2B
    });
});
```

---

## Phase 2A Deliverables

### Code

- [ ] `InterpolationBufferManager.ts` - Buffer lifecycle management
- [ ] `InterpolatedCreatureRenderer.ts` - Custom Geometry + Mesh
- [ ] Updated `StateManager.ts` - Parse NAPI buffer to interleaved format
- [ ] Updated `PixiApp.ts` - Integrate new renderer

### Tests

- [ ] Unit tests for buffer swap logic
- [ ] Integration tests for NAPI → interleaved conversion
- [ ] Performance benchmarks (buffer update time)

### Documentation

- [ ] Code comments in InterpolationBufferManager
- [ ] Architecture diagram (Rust → JS → GPU)
- [ ] Performance measurements (baseline for Phase 2B)

---

## Collaboration Points

### shader-sarah + frontend-fanny
- Buffer layout design (this doc)
- Geometry attribute mapping
- Instanced rendering setup

### shader-sarah + rusty-ron
- Verify NAPI buffer format (SoA layout confirmed)
- Discuss potential Rust-side changes (size field in buffer?)
- Coordinate on creature ID handling

### shader-sarah + architect-andy
- Performance benchmarks (Phase 2A vs current)
- Fallback strategy (if GPU doesn't support instancing)
- Cross-platform testing plan

---

## Success Criteria

Phase 2A is complete when:

1. **Custom Geometry Rendering:** Creatures render using custom PixiJS Geometry (not ParticleContainer)
2. **Interleaved Buffer:** START/END positions stored in 8-float AoS layout
3. **Buffer Swap Working:** END → START on simulation tick, new data written to END
4. **Zero Regressions:** Visual quality matches current renderer (no interpolation yet, just direct END rendering)
5. **Performance Baseline:** Buffer update time <5ms @ 100K creatures
6. **All Tests Pass:** Unit + integration tests green

**Note:** Interpolation shader is Phase 2B. Phase 2A just sets up the infrastructure (no actual interpolation yet).

---

## Next Steps (Phase 2B)

After Phase 2A completes:
1. Implement vertex shader with `mix(aStartPos, aEndPos, uInterpolation)`
2. Handle rotation interpolation (shortest path, angle wrapping)
3. Update `uInterpolation` uniform every render frame (0.0 → 1.0)
4. Test smooth 60 FPS rendering with 22.2Hz simulation
5. Profile GPU performance (<0.2ms target)

---

## References

- **Current Implementation:** `apps/portal/src/rendering/CreatureRenderer.ts`
- **NAPI Buffer:** `apps/simulation/src/napi_addon/simulation_engine.rs:363-365`
- **Shader Spec:** `docs/visuals/shader-smooth-and-wiggle.md`
- **PixiJS Geometry API:** https://pixijs.download/release/docs/PIXI.Geometry.html
- **Instanced Rendering:** https://webglfundamentals.org/webgl/lessons/webgl-instanced-drawing.html
