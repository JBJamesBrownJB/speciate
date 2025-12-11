# Project Brief: GPU-Accelerated Entity Rendering Pipeline

**To:** Development Team
**From:** Technical Lead
**Subject:** Implementation of Shader-Based Interpolation and Procedural Animation (Rust/PixiJS)

## 1. Executive Summary

We are migrating our simulation visualization from a CPU-bound sprite update loop to a GPU-bound instanced rendering pipeline.

**The Goal:** Decouple the backend simulation rate (20Hz) from the frontend render rate (60Hz+), ensuring butter-smooth movement and consistent organic animation for 10,000+ entities with minimal CPU overhead.

**The Strategy:**

- **Backend (Rust):** Sends raw snapshots.
- **Frontend (PixiJS):** Updates binary buffers.
- **GPU (Vertex Shader):** Handles all position interpolation and vertex deformation.

## 2. Architecture Overview

- **State Management:** Snapshot Interpolation. The client buffers the "Current" and "Next" server states.
- **Rendering Method:** PIXI.Mesh or PIXI.Geometry with a custom PIXI.Shader.
- **Data Flow:**
  - Rust sends Snapshot T and Snapshot T+1.
  - JS updates a Float32Array (Interleaved Buffer) with the new target positions/rotations.
  - JS ticker updates a single Uniform uInterpolation (0.0 to 1.0) every frame.
  - Vertex Shader calculates the final pixel position of every entity.

## 3. Phased Implementation Plan

We will execute this in two distinct phases. Phase 1 must be verified and performant before beginning Phase 2.

### Phase 1: The Foundation (Kinematic Smoothing)

**Objective:** Achieve perfectly smooth linear movement and rotation on the GPU, masking the 20Hz server tick.

#### Technical Requirements

- **Buffer Structure:** Create a custom PIXI.Geometry with an interleaved buffer.
  - Attributes per entity:
    - aStartPos (vec2): Position at Tick A.
    - aEndPos (vec2): Position at Tick B.
    - aStartRot (float): Rotation at Tick A.
    - aEndRot (float): Rotation at Tick B.
    - aTextureCoord (vec2): Standard UVs.

- **The Update Loop (JS):**
  - On Server Tick: Copy End data to Start. Load new server data into End. Reset uInterpolation to 0.
  - On Render Tick: Increment uInterpolation based on deltaMS / tickDuration.

- **The Vertex Shader:**
  - Implement mix(start, end, t) for position.
  - Implement rotation interpolation.
  - Crucial Edge Case: Handle "Rotation Wrapping" (e.g., interpolating from 350° to 10° should rotate 20° CW, not 340° CCW). Recommendation: Pre-calculate the 'shortest path' delta in JS before uploading to the buffer, or use direction vectors (vec2) instead of angles.

#### Acceptance Criteria (Phase 1)

- [ ] Entities move smoothly at 60FPS/144FPS despite 20Hz data updates.
- [ ] No visual "rubber banding" or stuttering.
- [ ] CPU usage remains low (< 15% increase) when rendering 10,000 entities.

### Phase 2: The Polish (Organic Wiggle)

**Objective:** Inject "life" into the entities using vertex manipulation to simulate flexible bodies (e.g., fish/snakes) without adding CPU overhead.

#### Technical Requirements

- **Shader Expansion:** Modify the Phase 1 Vertex Shader.
  - Concept: Deform the mesh in Local Space before applying the Rotation and World Position from Phase 1.

- **New Uniforms:**
  - uGameTime: A continuous float that increases indefinitely (for the sine wave).

- **The Algorithm (Wiggle):**
  - Calculate a sine wave based on uGameTime and aTextureCoord.y (vertical position along the sprite).
  - Apply a lag factor: sin(time - uv.y * lag).
  - Apply an amplitude factor: offset * uv.y (Head at uv.y=0 should remain fixed; tail moves most).

- **Dynamic Coupling (The "Nice to Have"):**
  - Calculate the distance between aStartPos and aEndPos inside the shader.
  - Use this distance to modulate the frequency/speed of the sine wave (Faster movement = Furious wiggling. Idle = Gentle drift).

#### Acceptance Criteria (Phase 2)

- [ ] Entities appear to "swim" or move organically.
- [ ] The tail lags behind the head movement.
- [ ] Wiggling intensity correlates with movement speed.
- [ ] Performance metrics (FPS) remain identical to Phase 1.

## 4. Implementation Specs (Reference)

### Data Structure (Interleaved Buffer)

We will minimize GPU draw calls by packing all entity data into one buffer.

```javascript
// Layout for Float32Array
// [ StX, StY, EndX, EndY, StRot, EndRot, UVx, UVy, ...next vertex ]
const FLOATS_PER_VERTEX = 8;
```

### Shader Logic Flow (Pseudocode)

```glsl
void main() {
    // 1. CALCULATE PHASES
    float t = uInterpolation; // 0.0 to 1.0

    // 2. PHASE 2: ORGANIC WIGGLE (Local Space)
    // Calculate sideways offset based on time and UV.y
    float wiggleOffset = calculateWiggle(uGameTime, aTextureCoord.y, moveSpeed);
    vec2 localPos = aVertexPosition;
    localPos.x += wiggleOffset;

    // 3. ROTATION (Local -> World Orientation)
    // Interpolate rotation for current frame
    float currentRot = mixAngle(aStartRot, aEndRot, t);
    vec2 rotatedPos = rotate(localPos, currentRot);

    // 4. PHASE 1: TRANSLATION (World Space)
    // Interpolate world position
    vec2 worldPos = mix(aStartPos, aEndPos, t);

    // 5. FINAL OUTPUT
    gl_Position = projection * (worldPos + rotatedPos);
}
```

## 5. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Rotation Wrapping | Entities spin 360 degrees wildly when crossing the 0/360 angle boundary. | Sanitize angles in Rust or JS before buffer upload to ensure shortest path. |
| Overshooting | If network lags, uInterpolation might exceed 1.0. | Clamp uInterpolation at 1.0. If lag persists >100ms, pause animation or extrapolate (predict). |
| Sprite Batching | Standard PIXI.Sprite features (tinting, z-ordering) are lost. | If we need tints, add aColor attribute to the buffer. If we need Z-sorting, enable the depth buffer in WebGL state. |

## Questions for the Dev Team

1. Do our current entity sprites have the correct UV orientation (facing UP) for the wiggle math, or do we need to rotate the source textures?

2. Does the Rust backend currently provide a unique ID for entities so we can consistently map Snapshot A to Snapshot B in the buffer?

---

This diagram illustrates the buffering strategy required for Phase 1. Please ensure the team understands the concept of "rendering in the past" to ensure the Start and End snapshots are always valid.
